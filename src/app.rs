use anyhow::Result;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::models::{AppMode, Host, HostField, SortBy, Container, DeploymentScript, Project, DockerPendingAction, ScriptSection};
use crate::ssh::{
    config::{default_ssh_config_path, parse_ssh_config, write_ssh_config, SshConfig},
    metadata::{default_metadata_path, load_metadata, save_metadata, Metadata},
};
use crate::docker;

/// Types of SSH commands we can execute (for handling responses)
#[derive(Clone, Debug)]
pub enum SshCommandType {
    DockerPs,
    ListProjects,
    FindScripts { project_name: String, #[allow(dead_code)] project_path: String },
    ReadScript { project_name: String, script_path: String },
    DockerOperation { operation: String },
    ViewLogs,
    ContainerStats { container_index: usize },
    ContainerTop { container_index: usize },
    ContainerInspect { container_index: usize },
    InspectContainerEnv { container_index: usize },
    ViewScriptContent { script_path: String, container_index: usize },
    ListDirectory { path: String },
    ReadScriptForContainer { script_path: String, container_index: usize },
    WriteScript { script_path: String },
    RunScript,
    // Rsync file browser
    RsyncListDirectory { path: String },
}

pub struct PendingSshCommand {
    pub host: Host,
    pub command: String,
    pub command_type: SshCommandType,
}

/// Main application state
pub struct App {
    /// Current application mode
    pub mode: AppMode,

    /// All hosts from SSH config + metadata
    pub hosts: Vec<Host>,

    /// Currently selected host index (in filtered view)
    pub selected_index: usize,

    /// Current search/filter query
    pub search_query: String,

    /// Currently active tag filters
    pub active_tag_filters: Vec<String>,

    /// Current sort order
    pub sort_by: SortBy,

    /// SSH config manager
    ssh_config: SshConfig,

    /// Metadata manager
    metadata: Metadata,

    /// Path to metadata file
    metadata_path: PathBuf,

    /// Should the application quit
    pub should_quit: bool,

    /// Status message to display
    pub status_message: Option<String>,

    /// Error message to display
    pub error_message: Option<String>,

    /// Pending SSH connection (host to connect to)
    pub pending_connection: Option<Host>,

    /// Pending rsync execution (host, source, dest, to_host, compress)
    pub pending_rsync: Option<(Host, String, String, bool, bool)>,

    /// Whether rsync is available on this system
    pub rsync_available: bool,

    // ==================== Docker Mode Fields ====================

    /// Docker containers for connected host
    pub containers: Vec<Container>,

    /// Docker projects discovered on remote server
    pub projects: Vec<Project>,

    /// Docker scripts discovered on remote server
    pub scripts: Vec<DeploymentScript>,

    /// Index of currently selected container in docker mode
    pub docker_selected_index: usize,

    /// Pending SSH command for docker operations
    pub pending_ssh_command: Option<PendingSshCommand>,

    /// Queue of pending commands for multi-step operations
    pending_docker_commands: Vec<PendingSshCommand>,

    /// Current host index when in docker mode
    current_docker_host_index: Option<usize>,

    /// Whether to use sudo for docker commands
    pub use_sudo: bool,

    /// Default path for docker clients
    pub clients_path: String,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let metadata_path = default_metadata_path();
        let config_path = default_ssh_config_path();

        let mut ssh_config = parse_ssh_config(&config_path)?;
        let mut metadata = load_metadata(&metadata_path)?;

        // Merge metadata into hosts
        metadata.merge_into_hosts(&mut ssh_config.hosts);

        let hosts = ssh_config.hosts.clone();

        // Populate global tags from existing host tags if global_tags is empty (backwards compatibility)
        if metadata.global_tags.is_empty() {
            let mut all_tags: Vec<String> = hosts
                .iter()
                .flat_map(|h| h.tags.iter().cloned())
                .collect();
            all_tags.sort();
            all_tags.dedup();
            metadata.global_tags = all_tags;
        }

        Ok(App {
            mode: AppMode::default(),
            hosts,
            selected_index: 0,
            search_query: String::new(),
            active_tag_filters: Vec::new(),
            sort_by: SortBy::default(),
            ssh_config,
            metadata,
            metadata_path,
            should_quit: false,
            status_message: None,
            error_message: None,
            pending_connection: None,
            pending_rsync: None,
            rsync_available: crate::ssh::rsync::is_rsync_available(),
            // Docker mode fields
            containers: Vec::new(),
            projects: Vec::new(),
            scripts: Vec::new(),
            docker_selected_index: 0,
            pending_ssh_command: None,
            pending_docker_commands: Vec::new(),
            current_docker_host_index: None,
            use_sudo: false,
            clients_path: "~/clients".to_string(),
        })
    }

    /// Get filtered hosts based on search and tag filters
    pub fn filtered_hosts(&self) -> Vec<&Host> {
        let mut filtered: Vec<&Host> = self
            .hosts
            .iter()
            .filter(|host| {
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    host.matches_search(&self.search_query)
                };

                let matches_tags = host.has_any_tag(&self.active_tag_filters);

                matches_search && matches_tags
            })
            .collect();

        // Sort based on current sort order
        match self.sort_by {
            SortBy::Name => {
                filtered.sort_by(|a, b| a.host.cmp(&b.host));
            }
            SortBy::Hostname => {
                filtered.sort_by(|a, b| a.hostname.cmp(&b.hostname));
            }
            SortBy::LastUsed => {
                filtered.sort_by(|a, b| {
                    // Most recent first (reverse order)
                    b.last_used.cmp(&a.last_used)
                });
            }
            SortBy::User => {
                filtered.sort_by(|a, b| {
                    a.user.cmp(&b.user)
                });
            }
        }

        filtered
    }

    /// Get all unique tags from the global tag pool
    pub fn all_tags(&self) -> Vec<String> {
        self.metadata.get_global_tags()
    }

    /// Add a tag to the global tag pool and save
    pub fn add_global_tag(&mut self, tag: String) -> Result<()> {
        self.metadata.add_global_tag(tag);
        save_metadata(&self.metadata_path, &self.metadata)?;
        Ok(())
    }

    /// Get the currently selected host
    pub fn selected_host(&self) -> Option<&Host> {
        let filtered = self.filtered_hosts();
        filtered.get(self.selected_index).copied()
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_hosts().len();
        if filtered_count > 0 && self.selected_index < filtered_count - 1 {
            self.selected_index += 1;
        }
    }

    /// Jump to first item
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Jump to last item
    pub fn select_last(&mut self) {
        let filtered_count = self.filtered_hosts().len();
        if filtered_count > 0 {
            self.selected_index = filtered_count - 1;
        }
    }

    /// Page down
    pub fn page_down(&mut self, page_size: usize) {
        let filtered_count = self.filtered_hosts().len();
        if filtered_count > 0 {
            self.selected_index = (self.selected_index + page_size).min(filtered_count - 1);
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: usize) {
        if self.selected_index >= page_size {
            self.selected_index -= page_size;
        } else {
            self.selected_index = 0;
        }
    }

    /// Start editing a new host
    pub fn start_new_host(&mut self) {
        self.mode = AppMode::EditHost {
            host_index: None,
            editing_host: Host::new(String::new(), String::new()),
            focused_field: HostField::HostAlias,
            field_buffer: String::new(),
            editing_mode: false, // Start in navigation mode
        };
    }

    /// Start editing the selected host
    pub fn start_edit_host(&mut self) {
        if let Some(host) = self.selected_host() {
            // Find the actual index in the full host list
            if let Some(actual_index) = self.hosts.iter().position(|h| h.host == host.host) {
                let field_buffer = get_field_value(host, &HostField::HostAlias);
                self.mode = AppMode::EditHost {
                    host_index: Some(actual_index),
                    editing_host: host.clone(),
                    focused_field: HostField::HostAlias,
                    field_buffer,
                    editing_mode: false, // Start in navigation mode
                };
            }
        }
    }

    /// Start SSH key selection mode
    pub fn start_key_selection(&mut self, host_index: Option<usize>, editing_host: Host, return_field: HostField) {
        let available_keys = get_available_ssh_keys();
        self.mode = AppMode::SelectKeys {
            host_index,
            editing_host,
            available_keys,
            selected_key_index: 0,
            return_field,
        };
    }

    /// Start tag editing mode
    pub fn start_tag_editing(&mut self, host_index: Option<usize>, editing_host: Host, return_field: HostField) {
        self.mode = AppMode::EditTags {
            host_index,
            editing_host,
            tag_input: String::new(),
            selected_tag_index: 0,
            return_field,
            input_mode: false, // Start in selection mode
        };
    }

    /// Start SSH flags selection mode
    pub fn start_ssh_flags_selection(&mut self, host_index: Option<usize>, editing_host: Host, return_field: HostField) {
        self.mode = AppMode::SelectSshFlags {
            host_index,
            editing_host,
            selected_flag_index: 0,
            return_field,
        };
    }

    /// Start shell selection mode
    pub fn start_shell_selection(&mut self, host_index: Option<usize>, editing_host: Host, return_field: HostField) {
        self.mode = AppMode::SelectShell {
            host_index,
            editing_host,
            selected_shell_index: 0,
            return_field,
        };
    }

    /// Return to edit host mode with updated host
    pub fn return_to_edit(&mut self, host_index: Option<usize>, editing_host: Host, focused_field: HostField) {
        let field_buffer = get_field_value(&editing_host, &focused_field);
        self.mode = AppMode::EditHost {
            host_index,
            editing_host,
            focused_field,
            field_buffer,
            editing_mode: false, // Return in navigation mode
        };
    }

    /// Save the currently edited host
    pub fn save_edited_host(&mut self, host: Host, original_index: Option<usize>) -> Result<()> {
        // Check for duplicate host names (except when editing the same host)
        let is_duplicate = self.hosts.iter().enumerate().any(|(i, h)| {
            h.host == host.host && Some(i) != original_index
        });

        if is_duplicate {
            self.error_message = Some(format!("Host '{}' already exists", host.host));
            return Ok(());
        }

        // Validate required fields
        if host.host.is_empty() {
            self.error_message = Some("Host alias cannot be empty".to_string());
            return Ok(());
        }

        if host.hostname.is_empty() {
            self.error_message = Some("Hostname/IP cannot be empty".to_string());
            return Ok(());
        }

        match original_index {
            Some(index) => {
                // Update existing host
                self.hosts[index] = host.clone();
                self.ssh_config.update_host(index, host.clone())?;
                self.status_message = Some(format!("Updated host '{}'", host.host));
            }
            None => {
                // Add new host
                self.hosts.push(host.clone());
                self.ssh_config.add_host(host.clone());
                self.status_message = Some(format!("Added host '{}'", host.host));
            }
        }

        // Save to files
        self.save_all()?;

        // Return to table view
        self.mode = AppMode::Table;

        Ok(())
    }

    /// Start deletion confirmation
    pub fn start_delete_host(&mut self) {
        if let Some(host) = self.selected_host() {
            if let Some(actual_index) = self.hosts.iter().position(|h| h.host == host.host) {
                self.mode = AppMode::ConfirmDelete {
                    host_index: actual_index,
                };
            }
        }
    }

    /// Start rsync mode for the selected host
    pub fn start_rsync(&mut self) {
        if let Some(host) = self.selected_host() {
            if let Some(actual_index) = self.hosts.iter().position(|h| h.host == host.host) {
                self.mode = AppMode::Rsync {
                    host_index: actual_index,
                    editing_host: host.clone(),
                    source_path: String::new(),
                    dest_path: String::new(),
                    sync_to_host: true, // Default to pushing to host
                    focused_field: crate::models::app_state::RsyncField::SourcePath,
                    editing_mode: false,
                    compress: false, // Default to no compression
                };
            }
        }
    }

    /// Start file browser for rsync path selection
    pub fn start_rsync_browse(&mut self) {
        use crate::models::app_state::RsyncField;

        if let AppMode::Rsync {
            host_index,
            editing_host,
            source_path,
            dest_path,
            sync_to_host,
            focused_field,
            compress,
            ..
        } = &self.mode
        {
            let host_index = *host_index;
            let editing_host = editing_host.clone();
            let source_path = source_path.clone();
            let dest_path = dest_path.clone();
            let sync_to_host = *sync_to_host;
            let focused_field = *focused_field;
            let compress = *compress;

            // Determine if we're browsing local or remote based on field and direction
            let is_remote = match focused_field {
                RsyncField::SourcePath => !sync_to_host,  // Source is remote when pulling from host
                RsyncField::DestPath => sync_to_host,     // Dest is remote when pushing to host
            };

            // Get starting path - use home directory as default
            let start_path = if is_remote {
                "~".to_string()
            } else {
                dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string())
            };

            if is_remote {
                // For remote, queue SSH command to list directory
                let cmd = format!("ls -la {} 2>/dev/null | tail -n +2", start_path);
                self.pending_ssh_command = Some(PendingSshCommand {
                    host: editing_host.clone(),
                    command: cmd,
                    command_type: SshCommandType::RsyncListDirectory { path: start_path.clone() },
                });

                self.mode = AppMode::RsyncFileBrowser {
                    host_index,
                    editing_host,
                    current_path: start_path,
                    entries: Vec::new(),
                    selected_index: 0,
                    loading: true,
                    is_remote: true,
                    target_field: focused_field,
                    source_path,
                    dest_path,
                    sync_to_host,
                    compress,
                };
            } else {
                // For local, list directory directly
                let entries = self.list_local_directory(&start_path);

                self.mode = AppMode::RsyncFileBrowser {
                    host_index,
                    editing_host,
                    current_path: start_path,
                    entries,
                    selected_index: 0,
                    loading: false,
                    is_remote: false,
                    target_field: focused_field,
                    source_path,
                    dest_path,
                    sync_to_host,
                    compress,
                };
            }
        }
    }

    /// List entries in a local directory
    pub fn list_local_directory(&self, path: &str) -> Vec<crate::models::docker::FileEntry> {
        use crate::models::docker::FileEntry;
        use std::fs;

        let mut entries = Vec::new();

        // Add parent directory entry (unless at root)
        if path != "/" {
            entries.push(FileEntry {
                name: "..".to_string(),
                is_dir: true,
                is_script: false,
            });
        }

        // Read directory contents
        if let Ok(read_dir) = fs::read_dir(path) {
            let mut items: Vec<_> = read_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden files
                    if name.starts_with('.') {
                        return None;
                    }
                    let metadata = entry.metadata().ok()?;
                    let is_dir = metadata.is_dir();

                    Some(FileEntry {
                        name,
                        is_dir,
                        is_script: false,
                    })
                })
                .collect();

            // Sort: directories first, then alphabetically
            items.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });

            entries.extend(items);
        }

        entries
    }

    /// Navigate to a directory in rsync file browser
    pub fn rsync_navigate_to(&mut self, path: String) {
        if let AppMode::RsyncFileBrowser {
            host_index,
            editing_host,
            is_remote,
            ..
        } = &self.mode
        {
            let _host_index = *host_index;
            let editing_host = editing_host.clone();
            let is_remote = *is_remote;

            if is_remote {
                // Queue SSH command to list directory
                let cmd = format!("ls -la {} 2>/dev/null | tail -n +2", path);
                self.pending_ssh_command = Some(PendingSshCommand {
                    host: editing_host,
                    command: cmd,
                    command_type: SshCommandType::RsyncListDirectory { path: path.clone() },
                });

                // Update mode to show loading
                if let AppMode::RsyncFileBrowser {
                    current_path,
                    entries,
                    selected_index,
                    loading,
                    ..
                } = &mut self.mode
                {
                    *current_path = path;
                    entries.clear();
                    *selected_index = 0;
                    *loading = true;
                }
            } else {
                // Local: list directory directly
                let entries = self.list_local_directory(&path);

                if let AppMode::RsyncFileBrowser {
                    current_path,
                    entries: mode_entries,
                    selected_index,
                    loading,
                    ..
                } = &mut self.mode
                {
                    *current_path = path;
                    *mode_entries = entries;
                    *selected_index = 0;
                    *loading = false;
                }
            }
        }
    }

    /// Select the current path in rsync file browser and return to rsync mode
    pub fn rsync_select_current_path(&mut self) {
        use crate::models::app_state::RsyncField;

        if let AppMode::RsyncFileBrowser {
            host_index,
            editing_host,
            current_path,
            entries,
            selected_index,
            target_field,
            source_path,
            dest_path,
            sync_to_host,
            compress,
            ..
        } = &self.mode
        {
            // Determine final path - either current directory or selected file
            let final_path = if *selected_index < entries.len() {
                let entry = &entries[*selected_index];
                if entry.name == ".." {
                    current_path.clone()
                } else if entry.is_dir {
                    if current_path.ends_with('/') {
                        format!("{}{}/", current_path, entry.name)
                    } else {
                        format!("{}/{}/", current_path, entry.name)
                    }
                } else {
                    if current_path.ends_with('/') {
                        format!("{}{}", current_path, entry.name)
                    } else {
                        format!("{}/{}", current_path, entry.name)
                    }
                }
            } else {
                current_path.clone()
            };

            // Update the appropriate field
            let (new_source, new_dest) = match target_field {
                RsyncField::SourcePath => (final_path, dest_path.clone()),
                RsyncField::DestPath => (source_path.clone(), final_path),
            };

            self.mode = AppMode::Rsync {
                host_index: *host_index,
                editing_host: editing_host.clone(),
                source_path: new_source,
                dest_path: new_dest,
                sync_to_host: *sync_to_host,
                focused_field: *target_field,
                editing_mode: false,
                compress: *compress,
            };
        }
    }

    /// Cancel rsync file browser and return to rsync mode
    pub fn rsync_cancel_browse(&mut self) {
        if let AppMode::RsyncFileBrowser {
            host_index,
            editing_host,
            target_field,
            source_path,
            dest_path,
            sync_to_host,
            compress,
            ..
        } = &self.mode
        {
            self.mode = AppMode::Rsync {
                host_index: *host_index,
                editing_host: editing_host.clone(),
                source_path: source_path.clone(),
                dest_path: dest_path.clone(),
                sync_to_host: *sync_to_host,
                focused_field: *target_field,
                editing_mode: false,
                compress: *compress,
            };
        }
    }

    /// Delete the confirmed host
    pub fn delete_host(&mut self, index: usize) -> Result<()> {
        if index < self.hosts.len() {
            let host = self.hosts.remove(index);
            self.ssh_config.remove_host(index)?;
            self.metadata.remove(&host.host);

            self.save_all()?;

            self.status_message = Some(format!("Deleted host '{}'", host.host));

            // Adjust selection if needed
            if self.selected_index >= self.hosts.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }

        self.mode = AppMode::Table;
        Ok(())
    }

    /// Connect to the selected host
    pub fn connect_to_selected(&mut self) -> Result<()> {
        // Clone the host to avoid borrow checker issues
        let host_to_connect = match self.selected_host() {
            Some(host) => host.clone(),
            None => {
                self.error_message = Some("No host selected".to_string());
                return Ok(());
            }
        };

        let host_name = host_to_connect.host.clone();

        // Update the host in our list and mark as used
        if let Some(actual_index) = self.hosts.iter().position(|h| h.host == host_name) {
            self.hosts[actual_index].mark_used();
        }

        // Save metadata with updated last_used
        self.save_metadata_only()?;

        // Store pending connection instead of executing immediately
        // The main loop will handle terminal cleanup and SSH execution
        self.pending_connection = Some(host_to_connect);

        Ok(())
    }

    /// Clear the pending connection and update status
    pub fn complete_connection(&mut self, success: bool, error: Option<String>) {
        if let Some(ref host) = self.pending_connection {
            if success {
                self.status_message = Some(format!("Connected to '{}'", host.host));
            } else {
                self.error_message = error;
            }
        }
        self.pending_connection = None;
    }

    /// Enter search mode
    pub fn start_search(&mut self) {
        self.mode = AppMode::Search {
            query: self.search_query.clone(),
        };
    }

    /// Apply search query
    pub fn apply_search(&mut self, query: String) {
        self.search_query = query;
        self.selected_index = 0; // Reset selection when search changes
        self.mode = AppMode::Table;
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.selected_index = 0;
    }

    /// Cycle to next sort option
    pub fn cycle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
        self.selected_index = 0; // Reset selection when sort changes
        self.status_message = Some(format!("Sorting by: {}", self.sort_by.label()));
    }

    /// Start tag filter mode
    pub fn start_tag_filter(&mut self) {
        self.mode = AppMode::TagFilter {
            selected_tags: self.active_tag_filters.clone(),
        };
    }

    /// Apply tag filters
    pub fn apply_tag_filter(&mut self, tags: Vec<String>) {
        self.active_tag_filters = tags;
        self.selected_index = 0; // Reset selection when filter changes
        self.mode = AppMode::Table;
    }

    /// Show help overlay
    pub fn show_help(&mut self) {
        self.mode = AppMode::Help;
    }

    /// Return to table view
    pub fn return_to_table(&mut self) {
        self.mode = AppMode::Table;
    }

    /// Set status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Set error message
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Clear status and error messages
    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
    }

    /// Save both SSH config and metadata
    fn save_all(&mut self) -> Result<()> {
        // Extract metadata from hosts
        self.metadata.extract_from_hosts(&self.hosts);

        // Save SSH config
        write_ssh_config(&self.ssh_config)?;

        // Save metadata
        save_metadata(&self.metadata_path, &self.metadata)?;

        Ok(())
    }

    /// Save only metadata (for last_used updates)
    fn save_metadata_only(&mut self) -> Result<()> {
        self.metadata.extract_from_hosts(&self.hosts);
        save_metadata(&self.metadata_path, &self.metadata)?;
        Ok(())
    }

    // ==================== Docker Mode Methods ====================

    /// Start Docker mode for the selected host
    pub fn start_docker_mode(&mut self) {
        if let Some(host) = self.selected_host() {
            if let Some(actual_index) = self.hosts.iter().position(|h| h.host == host.host) {
                let host = host.clone();

                // Reset sudo flag for new connection
                self.use_sudo = false;

                // If user is not root, ask about sudo
                if host.user.as_ref().map(|u| u != "root").unwrap_or(true) {
                    self.mode = AppMode::ConfirmDockerAction {
                        action: DockerPendingAction::EnableSudo { host_index: actual_index },
                        return_mode: Box::new(AppMode::Table),
                    };
                } else {
                    // Root user, connect directly
                    self.start_fetching_containers(actual_index);
                }
            }
        }
    }

    /// Start fetching containers after connection setup
    pub fn start_fetching_containers(&mut self, host_index: usize) {
        if host_index < self.hosts.len() {
            let host = self.hosts[host_index].clone();
            self.current_docker_host_index = Some(host_index);

            // Clear previous data
            self.containers.clear();
            self.projects.clear();
            self.scripts.clear();

            // Build docker ps command (with sudo if needed)
            let docker_ps = self.docker_cmd(&docker::docker_ps_command(true));

            // Step 1: Fetch containers
            self.pending_ssh_command = Some(PendingSshCommand {
                host: host.clone(),
                command: docker_ps,
                command_type: SshCommandType::DockerPs,
            });

            // Queue step 2: List projects
            self.pending_docker_commands.push(PendingSshCommand {
                host: host.clone(),
                command: self.sudo_cmd(&docker::list_projects_command(&self.clients_path)),
                command_type: SshCommandType::ListProjects,
            });

            self.mode = AppMode::ContainerList { host_index };
            self.docker_selected_index = 0;
            self.set_status(if self.use_sudo { "Connecting (sudo)..." } else { "Connecting..." }.to_string());
        }
    }

    /// Refresh containers
    pub fn refresh_containers(&mut self) {
        if let Some(host_index) = self.current_docker_host_index {
            if host_index < self.hosts.len() {
                let host = self.hosts[host_index].clone();

                // Clear and refetch
                self.containers.clear();
                self.projects.clear();
                self.scripts.clear();

                let docker_ps = self.docker_cmd(&docker::docker_ps_command(true));

                self.pending_ssh_command = Some(PendingSshCommand {
                    host: host.clone(),
                    command: docker_ps,
                    command_type: SshCommandType::DockerPs,
                });

                self.pending_docker_commands.push(PendingSshCommand {
                    host: host.clone(),
                    command: self.sudo_cmd(&docker::list_projects_command(&self.clients_path)),
                    command_type: SshCommandType::ListProjects,
                });

                self.set_status(if self.use_sudo { "Refreshing (sudo)..." } else { "Refreshing..." }.to_string());
            }
        }
    }

    /// Handle SSH command output
    pub fn handle_ssh_output(&mut self, output: String, command_type: SshCommandType) {
        match command_type {
            SshCommandType::DockerPs => {
                let server_name = self.current_docker_host_index
                    .and_then(|i| self.hosts.get(i))
                    .map(|s| s.host.as_str())
                    .unwrap_or("unknown");
                self.containers = docker::parse_docker_ps(&output, server_name);

                // Associate containers with scripts
                self.associate_containers_with_scripts();

                self.set_status(format!("Found {} containers", self.containers.len()));
            }
            SshCommandType::ListProjects => {
                if let Some(host_index) = self.current_docker_host_index {
                    if let Some(host) = self.hosts.get(host_index).cloned() {
                        self.projects = docker::parse_project_listing(&output, &self.clients_path);

                        // Queue script discovery for each project
                        for project in &self.projects {
                            self.pending_docker_commands.push(PendingSshCommand {
                                host: host.clone(),
                                command: self.sudo_cmd(&docker::find_scripts_command(&project.path)),
                                command_type: SshCommandType::FindScripts {
                                    project_name: project.name.clone(),
                                    project_path: project.path.clone(),
                                },
                            });
                        }

                        self.set_status(format!("Found {} projects, scanning for scripts...", self.projects.len()));
                    }
                }
            }
            SshCommandType::FindScripts { project_name, project_path: _ } => {
                let script_paths = docker::parse_script_paths(&output);

                if let Some(host_index) = self.current_docker_host_index {
                    if let Some(host) = self.hosts.get(host_index).cloned() {
                        // Queue reading each script
                        for script_path in script_paths {
                            self.pending_docker_commands.push(PendingSshCommand {
                                host: host.clone(),
                                command: self.sudo_cmd(&docker::read_script_command(&script_path)),
                                command_type: SshCommandType::ReadScript {
                                    project_name: project_name.clone(),
                                    script_path,
                                },
                            });
                        }
                    }
                }
            }
            SshCommandType::ReadScript { project_name, script_path } => {
                if let Some(script) = docker::create_script_from_content(&script_path, &output, &project_name) {
                    // Add script to project
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                        project.scripts.push(script.clone());
                    }
                    self.scripts.push(script);

                    // Re-associate containers with scripts
                    self.associate_containers_with_scripts();
                }

                // Update status with script count
                let total_scripts: usize = self.projects.iter().map(|p| p.scripts.len()).sum();
                if self.pending_docker_commands.is_empty() {
                    self.set_status(format!(
                        "{} containers, {} projects, {} scripts",
                        self.containers.len(),
                        self.projects.len(),
                        total_scripts
                    ));
                }
            }
            SshCommandType::ViewLogs => {
                if let AppMode::LogsViewer { ref mut log_buffer, .. } = self.mode {
                    log_buffer.extend(output.lines().map(String::from));
                }
            }
            SshCommandType::DockerOperation { operation } => {
                self.set_status(format!("{} completed", operation));
                // Refresh container list after operation
                self.refresh_containers();
            }
            SshCommandType::ContainerStats { container_index } => {
                let stats = docker::parser::parse_docker_stats(&output);
                if let Some(host_index) = self.current_docker_host_index {
                    self.mode = AppMode::StatsViewer {
                        host_index,
                        container_index,
                        stats,
                    };
                }
            }
            SshCommandType::ContainerTop { container_index } => {
                let processes = docker::parser::parse_docker_top(&output);
                if let Some(host_index) = self.current_docker_host_index {
                    self.mode = AppMode::ProcessViewer {
                        host_index,
                        container_index,
                        processes,
                        selected_index: 0,
                    };
                }
            }
            SshCommandType::ContainerInspect { container_index } => {
                let info = docker::parser::parse_docker_inspect(&output);
                if let Some(host_index) = self.current_docker_host_index {
                    self.mode = AppMode::InspectViewer {
                        host_index,
                        container_index,
                        info,
                        selected_section: 0,
                    };
                }
            }
            SshCommandType::InspectContainerEnv { container_index } => {
                let container_vars: Vec<(String, String)> = output
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        line.find('=').map(|pos| {
                            let key = line[..pos].to_string();
                            let value = line[pos + 1..].to_string();
                            (key, value)
                        })
                    })
                    .collect();

                let var_count = container_vars.len();

                // Get script vars if available
                let script_vars: Vec<(String, String)> = if container_index < self.containers.len() {
                    let container_name = &self.containers[container_index].name;
                    self.get_script_for_container(container_name)
                        .map(|s| s.env_vars.iter().map(|e| (e.key.clone(), e.value.clone())).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                if let Some(host_index) = self.current_docker_host_index {
                    self.mode = AppMode::EnvInspector {
                        host_index,
                        container_index,
                        script_vars,
                        container_vars,
                        selected_index: 0,
                        scroll_offset: 0,
                        search_query: String::new(),
                    };
                    self.set_status(format!("Found {} environment variables. Type to search.", var_count));
                }
            }
            SshCommandType::ViewScriptContent { script_path, container_index } => {
                let lines: Vec<String> = output.lines().map(|l| l.to_string()).collect();
                if let Some(host_index) = self.current_docker_host_index {
                    self.mode = AppMode::ScriptViewer {
                        host_index,
                        container_index,
                        script_path,
                        script_content: lines,
                        scroll_offset: 0,
                    };
                }
            }
            SshCommandType::ListDirectory { path } => {
                let entries = docker::parser::parse_directory_listing(&output, &path);
                if let AppMode::FileBrowser { entries: ref mut e, loading, current_path, .. } = &mut self.mode {
                    *e = entries;
                    *loading = false;
                    *current_path = path;
                }
            }
            SshCommandType::ReadScriptForContainer { script_path, container_index } => {
                if let Some(script) = docker::create_script_from_content(&script_path, &output, "manual") {
                    // Update the container's script_path
                    if container_index < self.containers.len() {
                        self.containers[container_index].script_path = Some(script_path.clone());

                        // Save the association to metadata for persistence
                        if let Some(host) = self.get_current_docker_host() {
                            let host_name = host.host.clone();
                            let container_name = self.containers[container_index].name.clone();
                            self.metadata.set_script_path(&host_name, &container_name, script_path.clone());
                            let _ = save_metadata(&self.metadata_path, &self.metadata);
                        }
                    }

                    // Add to scripts list if not already present
                    if !self.scripts.iter().any(|s| s.path == script_path) {
                        self.scripts.push(script.clone());
                    }

                    // Switch to script edit mode
                    if let Some(host_index) = self.current_docker_host_index {
                        self.mode = AppMode::ScriptEdit {
                            host_index,
                            container_index,
                            editing_script: script,
                            focused_section: ScriptSection::EnvVars,
                            selected_index: 0,
                            editing_mode: false,
                        };
                        self.set_status("Script loaded and saved. Make changes and press Ctrl+S to save.".to_string());
                    }
                } else {
                    self.set_error("Failed to parse script file".to_string());
                    // Return to container list
                    if let Some(host_index) = self.current_docker_host_index {
                        self.mode = AppMode::ContainerList { host_index };
                    }
                }
            }
            SshCommandType::WriteScript { script_path } => {
                self.set_status(format!("Script saved: {}", script_path));
            }
            SshCommandType::RunScript => {
                self.set_status("Script executed".to_string());
                // Refresh after script run
                self.refresh_containers();
            }
            SshCommandType::RsyncListDirectory { path } => {
                let entries = docker::parser::parse_directory_listing(&output, &path);
                if let AppMode::RsyncFileBrowser { entries: ref mut e, loading, current_path, .. } = &mut self.mode {
                    *e = entries;
                    *loading = false;
                    *current_path = path;
                }
            }
        }

        // Process next queued command
        self.process_next_docker_command();
    }

    /// Process next queued docker command
    fn process_next_docker_command(&mut self) {
        if self.pending_ssh_command.is_none() && !self.pending_docker_commands.is_empty() {
            self.pending_ssh_command = Some(self.pending_docker_commands.remove(0));
        }
    }

    /// Associate containers with scripts
    fn associate_containers_with_scripts(&mut self) {
        let host_name = self.get_current_docker_host()
            .map(|h| h.host.clone())
            .unwrap_or_default();

        // Associate containers with their scripts based on saved metadata or NAME variable
        for container in &mut self.containers {
            container.script_path = None;

            // First check if there's a saved association in metadata
            if let Some(saved_path) = self.metadata.get_script_path(&host_name, &container.name) {
                container.script_path = Some(saved_path);
                continue;
            }

            // Then check discovered scripts by matching NAME variable
            for script in &self.scripts {
                if script.container_name == container.name {
                    container.script_path = Some(script.path.clone());
                    break;
                }
            }
        }
    }

    /// Get script for a container
    pub fn get_script_for_container(&self, container_name: &str) -> Option<&DeploymentScript> {
        // First try to find by container's script_path
        if let Some(container) = self.containers.iter().find(|c| c.name == container_name) {
            if let Some(ref script_path) = container.script_path {
                if let Some(script) = self.scripts.iter().find(|s| &s.path == script_path) {
                    return Some(script);
                }
            }
        }
        // Fall back to matching by script's container_name
        self.scripts.iter().find(|s| s.container_name == container_name)
    }

    /// Get current docker host
    pub fn get_current_docker_host(&self) -> Option<&Host> {
        self.current_docker_host_index.and_then(|i| self.hosts.get(i))
    }

    /// Docker navigation
    pub fn docker_select_next(&mut self) {
        if !self.containers.is_empty() && self.docker_selected_index < self.containers.len() - 1 {
            self.docker_selected_index += 1;
        }
    }

    pub fn docker_select_previous(&mut self) {
        if self.docker_selected_index > 0 {
            self.docker_selected_index -= 1;
        }
    }

    pub fn docker_select_first(&mut self) {
        self.docker_selected_index = 0;
    }

    pub fn docker_select_last(&mut self) {
        if !self.containers.is_empty() {
            self.docker_selected_index = self.containers.len() - 1;
        }
    }

    pub fn docker_page_down(&mut self) {
        if !self.containers.is_empty() {
            self.docker_selected_index = (self.docker_selected_index + 10).min(self.containers.len() - 1);
        }
    }

    pub fn docker_page_up(&mut self) {
        self.docker_selected_index = self.docker_selected_index.saturating_sub(10);
    }

    // Docker operations
    pub fn docker_pull(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                let current_mode = self.mode.clone();
                self.mode = AppMode::ConfirmDockerAction {
                    action: DockerPendingAction::DockerPull {
                        host_index,
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                        image_name: container.image.clone(),
                    },
                    return_mode: Box::new(current_mode),
                };
            }
        }
    }

    pub fn docker_restart(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                let current_mode = self.mode.clone();
                self.mode = AppMode::ConfirmDockerAction {
                    action: DockerPendingAction::DockerRestart {
                        host_index,
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                    },
                    return_mode: Box::new(current_mode),
                };
            }
        }
    }

    pub fn docker_stop(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                let current_mode = self.mode.clone();
                self.mode = AppMode::ConfirmDockerAction {
                    action: DockerPendingAction::DockerStop {
                        host_index,
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                    },
                    return_mode: Box::new(current_mode),
                };
            }
        }
    }

    pub fn docker_start(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                let current_mode = self.mode.clone();
                self.mode = AppMode::ConfirmDockerAction {
                    action: DockerPendingAction::DockerStart {
                        host_index,
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                    },
                    return_mode: Box::new(current_mode),
                };
            }
        }
    }

    pub fn docker_remove(&mut self, remove_volumes: bool, remove_image: bool) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                let current_mode = self.mode.clone();
                self.mode = AppMode::ConfirmDockerAction {
                    action: DockerPendingAction::DockerRemove {
                        host_index,
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                        image_name: container.image.clone(),
                        remove_volumes,
                        remove_image,
                    },
                    return_mode: Box::new(current_mode),
                };
            }
        }
    }

    pub fn view_logs(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_logs_command(&container.name, Some(100), false));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ViewLogs,
                    });

                    self.mode = AppMode::LogsViewer {
                        host_index,
                        container_index: self.docker_selected_index,
                        log_buffer: Vec::new(),
                        follow_mode: false,
                        scroll_offset: 0,
                        tail_count: 100,
                    };
                }
            }
        }
    }

    /// Load more log lines (increase tail count)
    pub fn load_more_logs(&mut self) {
        if let AppMode::LogsViewer { host_index, container_index, tail_count, .. } = self.mode {
            if container_index < self.containers.len() {
                let container = &self.containers[container_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    // Progressive increase: 100 -> 500 -> 2000 -> 10000 -> 50000
                    let new_tail_count = match tail_count {
                        t if t < 500 => 500,
                        t if t < 2000 => 2000,
                        t if t < 10000 => 10000,
                        _ => 50000,
                    };

                    let cmd = self.docker_cmd(&docker::docker_logs_command(&container.name, Some(new_tail_count), false));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ViewLogs,
                    });

                    self.mode = AppMode::LogsViewer {
                        host_index,
                        container_index,
                        log_buffer: Vec::new(),  // Clear buffer, will be replaced
                        follow_mode: false,
                        scroll_offset: 0,
                        tail_count: new_tail_count,
                    };

                    self.set_status(format!("Loading last {} lines...", new_tail_count));
                }
            }
        }
    }

    pub fn view_stats(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_stats_command(&container.name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ContainerStats { container_index: self.docker_selected_index },
                    });
                }
            }
        }
    }

    pub fn view_processes(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_top_command(&container.name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ContainerTop { container_index: self.docker_selected_index },
                    });
                }
            }
        }
    }

    pub fn view_inspect(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_inspect_command(&container.name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ContainerInspect { container_index: self.docker_selected_index },
                    });
                }
            }
        }
    }

    pub fn inspect_env(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_exec_env_command(&container.name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::InspectContainerEnv { container_index: self.docker_selected_index },
                    });
                }
            }
        }
    }

    pub fn view_script(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                if let Some(script_path) = &container.script_path {
                    if let Some(host) = self.hosts.get(host_index).cloned() {
                        let cmd = self.sudo_cmd(&docker::read_script_command(script_path));
                        self.pending_ssh_command = Some(PendingSshCommand {
                            host,
                            command: cmd,
                            command_type: SshCommandType::ViewScriptContent {
                                script_path: script_path.clone(),
                                container_index: self.docker_selected_index,
                            },
                        });
                    }
                }
            }
        }
    }

    pub fn edit_script(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                if let Some(script_path) = &container.script_path {
                    // Find the script
                    if let Some(script) = self.scripts.iter().find(|s| &s.path == script_path).cloned() {
                        self.mode = AppMode::ScriptEdit {
                            host_index,
                            container_index: self.docker_selected_index,
                            editing_script: script,
                            focused_section: ScriptSection::EnvVars,
                            selected_index: 0,
                            editing_mode: false,
                        };
                    }
                }
            }
        }
    }

    pub fn run_script(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];
                if let Some(script_path) = &container.script_path {
                    let current_mode = self.mode.clone();
                    self.mode = AppMode::ConfirmDockerAction {
                        action: DockerPendingAction::RunScript {
                            host_index,
                            script_path: script_path.clone(),
                        },
                        return_mode: Box::new(current_mode),
                    };
                }
            }
        }
    }

    pub fn browse_for_script(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    // When using sudo, ~ would expand to wrong user, so use /root instead
                    let start_path = if self.use_sudo && (self.clients_path == "~" || self.clients_path.starts_with("~/")) {
                        self.clients_path.replacen("~", "/root", 1)
                    } else if self.clients_path.starts_with("~/") {
                        // Expand ~ to $HOME for shell
                        self.clients_path.replacen("~", "$HOME", 1)
                    } else {
                        self.clients_path.clone()
                    };

                    let cmd = self.sudo_cmd(&docker::list_directory_command(&start_path));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::ListDirectory { path: start_path.clone() },
                    });

                    self.mode = AppMode::FileBrowser {
                        host_index,
                        container_index: self.docker_selected_index,
                        current_path: start_path,
                        entries: Vec::new(),
                        selected_index: 0,
                        loading: true,
                    };
                }
            }
        }
    }

    /// Create a new script for a container that doesn't have one
    pub fn create_script(&mut self) {
        if let AppMode::ContainerList { host_index } = self.mode {
            if self.docker_selected_index < self.containers.len() {
                let container = &self.containers[self.docker_selected_index];

                // Check if already has a script
                if container.has_script() {
                    self.set_error("Container already has a script. Press [e] to edit.");
                    return;
                }

                // Create a new script based on container info
                let script_name = format!("start{}.sh", capitalize_first(&container.name));
                let script_path = format!("{}/{}/{}", self.clients_path, container.name, script_name);

                let mut new_script = DeploymentScript::new(script_path.clone(), container.name.clone());
                new_script.container_name = container.name.clone();
                new_script.repo = container.image.clone();

                // Copy ports from container
                new_script.ports = container.ports.clone();

                self.mode = AppMode::ScriptEdit {
                    host_index,
                    container_index: self.docker_selected_index,
                    editing_script: new_script,
                    focused_section: ScriptSection::EnvVars,
                    selected_index: 0,
                    editing_mode: false,
                };

                self.set_status("Creating new script. Press Ctrl+S to save.".to_string());
            }
        }
    }

    /// Start adding a new env var in script editor
    pub fn start_add_env_var(&mut self) {
        if let AppMode::ScriptEdit { host_index, container_index, editing_script, .. } = &self.mode {
            self.mode = AppMode::EnvVarEditor {
                host_index: *host_index,
                container_index: *container_index,
                key_buffer: String::new(),
                value_buffer: String::new(),
                editing_key: true,
                is_new: true,
                editing_script: editing_script.clone(),
                var_index: None,
            };
        }
    }

    /// Start editing an existing env var
    pub fn start_edit_env_var(&mut self) {
        if let AppMode::ScriptEdit { host_index, container_index, editing_script, selected_index, focused_section, .. } = &self.mode {
            if *focused_section != ScriptSection::EnvVars {
                return;
            }
            if *selected_index < editing_script.env_vars.len() {
                let env = &editing_script.env_vars[*selected_index];
                self.mode = AppMode::EnvVarEditor {
                    host_index: *host_index,
                    container_index: *container_index,
                    key_buffer: env.key.clone(),
                    value_buffer: env.value.clone(),
                    editing_key: false,  // Start on value for editing
                    is_new: false,
                    editing_script: editing_script.clone(),
                    var_index: Some(*selected_index),
                };
            }
        }
    }

    /// Save env var from editor
    pub fn save_env_var(&mut self) {
        if let AppMode::EnvVarEditor {
            host_index,
            container_index,
            mut editing_script,
            var_index,
            key_buffer,
            value_buffer,
            ..
        } = std::mem::replace(&mut self.mode, AppMode::Table) {
            if key_buffer.is_empty() {
                self.set_error("Key cannot be empty");
                self.mode = AppMode::EnvVarEditor {
                    host_index,
                    container_index,
                    editing_script,
                    var_index,
                    key_buffer,
                    value_buffer,
                    editing_key: true,
                    is_new: var_index.is_none(),
                };
                return;
            }

            match var_index {
                Some(idx) => {
                    // Update existing
                    if idx < editing_script.env_vars.len() {
                        editing_script.env_vars[idx].key = key_buffer;
                        editing_script.env_vars[idx].value = value_buffer;
                    }
                }
                None => {
                    // Add new
                    editing_script.env_vars.push(crate::models::EnvVar::new(key_buffer, value_buffer));
                }
            }

            self.mode = AppMode::ScriptEdit {
                host_index,
                container_index,
                editing_script,
                focused_section: ScriptSection::EnvVars,
                selected_index: 0,
                editing_mode: false,
            };
        }
    }

    /// Cancel env var editing and return to script edit
    pub fn cancel_env_var_edit(&mut self) {
        if let AppMode::EnvVarEditor { host_index, container_index, editing_script, .. } = &self.mode {
            self.mode = AppMode::ScriptEdit {
                host_index: *host_index,
                container_index: *container_index,
                editing_script: editing_script.clone(),
                focused_section: ScriptSection::EnvVars,
                selected_index: 0,
                editing_mode: false,
            };
        }
    }

    /// Remove env var from current script
    pub fn remove_env_var_from_current_script(&mut self, index: usize) {
        if let AppMode::ScriptEdit { editing_script, selected_index, .. } = &mut self.mode {
            if index < editing_script.env_vars.len() {
                editing_script.env_vars.remove(index);
                *selected_index = (*selected_index).min(editing_script.env_vars.len().saturating_sub(1));
            }
        }
    }

    /// Save current script to remote server
    pub fn save_current_script(&mut self) {
        if let AppMode::ScriptEdit { host_index, editing_script, container_index, .. } = &self.mode {
            let host_index = *host_index;
            let container_index = *container_index;
            if let Some(host) = self.hosts.get(host_index).cloned() {
                // Generate updated script content
                let new_content = docker::script_parser::generate_script(editing_script);

                let cmd = self.sudo_cmd(&docker::write_script_command(&editing_script.path, &new_content));
                self.pending_ssh_command = Some(PendingSshCommand {
                    host,
                    command: cmd,
                    command_type: SshCommandType::WriteScript {
                        script_path: editing_script.path.clone(),
                    },
                });

                // Update local script copy
                let script_path = editing_script.path.clone();
                let updated_script = editing_script.clone();
                if let Some(script) = self.scripts.iter_mut().find(|s| s.path == script_path) {
                    *script = updated_script.clone();
                    script.raw_content = new_content;
                } else {
                    // New script - add to list and associate with container
                    self.scripts.push(updated_script);
                    if container_index < self.containers.len() {
                        self.containers[container_index].script_path = Some(script_path.clone());

                        // Save the association to metadata for persistence
                        let host_name = self.hosts[host_index].host.clone();
                        let container_name = self.containers[container_index].name.clone();
                        self.metadata.set_script_path(&host_name, &container_name, script_path);
                        let _ = save_metadata(&self.metadata_path, &self.metadata);
                    }
                }

                self.mode = AppMode::ContainerList { host_index };
                self.set_status("Saving script...".to_string());
            }
        }
    }

    /// Execute a docker action after confirmation
    pub fn execute_docker_action(&mut self, action: DockerPendingAction) {
        match action {
            DockerPendingAction::EnableSudo { host_index } => {
                self.use_sudo = true;
                self.start_fetching_containers(host_index);
            }
            DockerPendingAction::DockerPull { host_index, container_name, image_name, .. } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_pull_command(&image_name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::DockerOperation { operation: format!("Pull {}", container_name) },
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Pulling {}...", image_name));
                }
            }
            DockerPendingAction::DockerRestart { host_index, container_name, .. } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_restart_command(&container_name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::DockerOperation { operation: format!("Restart {}", container_name) },
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Restarting {}...", container_name));
                }
            }
            DockerPendingAction::DockerStop { host_index, container_name, .. } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_stop_command(&container_name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::DockerOperation { operation: format!("Stop {}", container_name) },
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Stopping {}...", container_name));
                }
            }
            DockerPendingAction::DockerStart { host_index, container_name, .. } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.docker_cmd(&docker::docker_start_command(&container_name));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::DockerOperation { operation: format!("Start {}", container_name) },
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Starting {}...", container_name));
                }
            }
            DockerPendingAction::DockerRemove { host_index, container_name, image_name, remove_volumes, remove_image, .. } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let rm_cmd = if remove_volumes {
                        docker::docker_rm_with_volumes_command(&container_name)
                    } else {
                        docker::docker_rm_command(&container_name)
                    };

                    let cmd = if remove_image {
                        format!("{} && {}", self.docker_cmd(&rm_cmd), self.docker_cmd(&docker::docker_rmi_command(&image_name)))
                    } else {
                        self.docker_cmd(&rm_cmd)
                    };

                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::DockerOperation { operation: format!("Remove {}", container_name) },
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Removing {}...", container_name));
                }
            }
            DockerPendingAction::RunScript { host_index, script_path } => {
                if let Some(host) = self.hosts.get(host_index).cloned() {
                    let cmd = self.sudo_cmd(&docker::run_script_command(&script_path));
                    self.pending_ssh_command = Some(PendingSshCommand {
                        host,
                        command: cmd,
                        command_type: SshCommandType::RunScript,
                    });
                    self.mode = AppMode::ContainerList { host_index };
                    self.set_status(format!("Running script {}...", script_path));
                }
            }
        }
    }

    /// Cancel a docker action
    pub fn cancel_docker_action(&mut self, return_mode: AppMode) {
        // For sudo dialog, if user says no, still connect without sudo
        if let AppMode::ConfirmDockerAction { action: DockerPendingAction::EnableSudo { host_index }, .. } = &self.mode {
            self.use_sudo = false;
            self.start_fetching_containers(*host_index);
            return;
        }
        self.mode = return_mode;
    }

    /// Go back from docker mode
    pub fn docker_go_back(&mut self) {
        // Clear docker state
        self.containers.clear();
        self.projects.clear();
        self.scripts.clear();
        self.current_docker_host_index = None;
        self.docker_selected_index = 0;
        self.use_sudo = false;
        self.pending_ssh_command = None;
        self.pending_docker_commands.clear();

        self.mode = AppMode::Table;
    }

    /// Prefix any command with sudo -i if enabled (runs in root login shell)
    /// This matches dockering's approach exactly
    fn sudo_cmd(&self, cmd: &str) -> String {
        if self.use_sudo {
            format!("sudo -i {}", cmd)
        } else {
            cmd.to_string()
        }
    }

    /// Alias for sudo_cmd - used for docker commands
    fn docker_cmd(&self, cmd: &str) -> String {
        self.sudo_cmd(cmd)
    }

    /// Execute SSH command and return output
    pub fn execute_ssh_command(&self, host: &Host, command: &str) -> Result<String> {
        let mut ssh_args = vec![
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=accept-new".to_string(),
        ];

        // Add port if specified
        if let Some(port) = host.port {
            ssh_args.push("-p".to_string());
            ssh_args.push(port.to_string());
        }

        // Add identity files if specified
        if let Some(ref keys) = host.identity_file {
            for key in keys {
                ssh_args.push("-i".to_string());
                ssh_args.push(expand_tilde(key));
            }
        }

        // Add user@host
        let user = host.user.clone().unwrap_or_else(|| "root".to_string());
        ssh_args.push(format!("{}@{}", user, host.hostname));

        // Add the command to execute
        ssh_args.push(command.to_string());

        let output = Command::new("ssh")
            .args(&ssh_args)
            .stdin(Stdio::null())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("SSH command failed: {}", stderr)
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to initialize application")
    }
}

/// Get available SSH keys from ~/.ssh directory
fn get_available_ssh_keys() -> Vec<String> {
    use std::fs;

    let ssh_dir = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ssh");

    if !ssh_dir.exists() {
        return Vec::new();
    }

    let mut keys = Vec::new();

    if let Ok(entries) = fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                // Look for common private key patterns (files without .pub extension)
                if !file_name.ends_with(".pub")
                    && !file_name.ends_with(".lock")
                    && file_name != "config"
                    && file_name != "known_hosts"
                    && file_name != "authorized_keys"
                    && !file_name.starts_with('.')
                {
                    let full_path = ssh_dir.join(&file_name);
                    if full_path.is_file() {
                        keys.push(format!("~/.ssh/{}", file_name));
                    }
                }
            }
        }
    }

    keys.sort();
    keys
}

/// Expand tilde in path
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Get the current value of a field as a string
fn get_field_value(host: &Host, field: &HostField) -> String {
    match field {
        HostField::HostAlias => host.host.clone(),
        HostField::Hostname => host.hostname.clone(),
        HostField::User => host.user.clone().unwrap_or_default(),
        HostField::Port => host.port.map(|p| p.to_string()).unwrap_or_default(),
        HostField::IdentityFiles => host
            .identity_file
            .as_ref()
            .map(|keys| keys.join(", "))
            .unwrap_or_default(),
        HostField::ProxyJump => host.proxy_jump.clone().unwrap_or_default(),
        HostField::SshFlags => host.ssh_flags.join(" "),
        HostField::Shell => host.shell.clone().unwrap_or_default(),
        HostField::Tags => host.tags.join(", "),
        HostField::Note => host.note.clone().unwrap_or_default(),
    }
}
