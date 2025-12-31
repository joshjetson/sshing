use anyhow::Result;
use std::path::PathBuf;

use crate::models::{AppMode, Host, HostField, SortBy};
use crate::ssh::{
    config::{default_ssh_config_path, parse_ssh_config, write_ssh_config, SshConfig},
    metadata::{default_metadata_path, load_metadata, save_metadata, Metadata},
};

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

    /// Path to SSH config file
    config_path: PathBuf,

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
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let config_path = default_ssh_config_path();
        let metadata_path = default_metadata_path();

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
            config_path,
            metadata_path,
            should_quit: false,
            status_message: None,
            error_message: None,
            pending_connection: None,
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
                let field_buffer = get_field_value(&host, &HostField::HostAlias);
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
