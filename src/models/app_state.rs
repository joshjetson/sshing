use crate::models::Host;
use crate::models::docker::{DeploymentScript, ContainerStats, ProcessInfo, ContainerInfo, FileEntry};

/// Application mode/state
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Main table view showing all hosts
    #[default]
    Table,

    /// Creating or editing a host
    EditHost {
        /// Index of host being edited (None for new host)
        host_index: Option<usize>,
        /// Current state of the host being edited
        editing_host: Host,
        /// Which field is currently focused
        focused_field: HostField,
        /// Current text being edited (for text fields)
        field_buffer: String,
        /// Whether the focused field is in edit mode (true) or just selected (false)
        editing_mode: bool,
    },

    /// Selecting SSH keys for a host
    SelectKeys {
        /// Index of host being edited
        host_index: Option<usize>,
        /// Host being edited
        editing_host: Host,
        /// Available SSH keys
        available_keys: Vec<String>,
        /// Currently selected key index
        selected_key_index: usize,
        /// Which field to return to after selection
        return_field: HostField,
    },

    /// Editing tags for a host
    EditTags {
        /// Index of host being edited
        host_index: Option<usize>,
        /// Host being edited
        editing_host: Host,
        /// Current tag being typed
        tag_input: String,
        /// Selected tag index for deletion
        selected_tag_index: usize,
        /// Which field to return to after editing
        return_field: HostField,
        /// Whether in input mode (true) or selection mode (false)
        input_mode: bool,
    },

    /// Selecting SSH flags for a host
    SelectSshFlags {
        /// Index of host being edited
        host_index: Option<usize>,
        /// Host being edited
        editing_host: Host,
        /// Currently selected flag index
        selected_flag_index: usize,
        /// Which field to return to after selection
        return_field: HostField,
    },

    /// Selecting shell for a host
    SelectShell {
        /// Index of host being edited
        host_index: Option<usize>,
        /// Host being edited
        editing_host: Host,
        /// Currently selected shell index
        selected_shell_index: usize,
        /// Which field to return to after selection
        return_field: HostField,
    },

    /// Search/filter mode
    Search {
        /// Current search query
        query: String,
    },

    /// Tag filter selection mode
    TagFilter {
        /// Currently selected tags for filtering
        selected_tags: Vec<String>,
    },

    /// Help overlay
    Help,

    /// Confirmation dialog for deletion
    ConfirmDelete {
        /// Index of host to delete
        host_index: usize,
    },

    /// Rsync file synchronization mode
    Rsync {
        /// Index of host being synced with
        host_index: usize,
        /// Host being synced with
        editing_host: Host,
        /// Source path for rsync
        source_path: String,
        /// Destination path for rsync
        dest_path: String,
        /// Whether syncing to (true) or from (false) the host
        sync_to_host: bool,
        /// Currently focused field (source or dest)
        focused_field: RsyncField,
        /// Whether the focused field is in edit mode
        editing_mode: bool,
        /// Enable compression (-z flag)
        compress: bool,
    },

    /// Rsync file browser for selecting source or destination paths
    RsyncFileBrowser {
        /// Index of host being synced with
        host_index: usize,
        /// Host being synced with
        editing_host: Host,
        /// Current directory being browsed
        current_path: String,
        /// Directory entries
        entries: Vec<FileEntry>,
        /// Currently selected entry index
        selected_index: usize,
        /// Whether we're loading directory contents
        loading: bool,
        /// Whether browsing remote (true) or local (false) filesystem
        is_remote: bool,
        /// Which field we're selecting for (source or dest)
        target_field: RsyncField,
        /// Current source path (to restore on cancel)
        source_path: String,
        /// Current dest path (to restore on cancel)
        dest_path: String,
        /// Direction setting (to restore)
        sync_to_host: bool,
        /// Compress setting (to restore)
        compress: bool,
    },

    // ==================== Docker Mode ====================

    /// Docker container list view
    ContainerList {
        /// Index of host we're connected to
        host_index: usize,
    },

    /// Docker logs viewer
    LogsViewer {
        host_index: usize,
        container_index: usize,
        log_buffer: Vec<String>,
        follow_mode: bool,
        scroll_offset: usize,
        tail_count: usize,
    },

    /// Docker stats viewer (CPU, memory)
    StatsViewer {
        host_index: usize,
        container_index: usize,
        stats: ContainerStats,
    },

    /// Docker process viewer (docker top)
    ProcessViewer {
        host_index: usize,
        container_index: usize,
        processes: Vec<ProcessInfo>,
        selected_index: usize,
    },

    /// Docker inspect viewer
    InspectViewer {
        host_index: usize,
        container_index: usize,
        info: ContainerInfo,
        selected_section: usize,
    },

    /// Environment variable inspector
    EnvInspector {
        host_index: usize,
        container_index: usize,
        script_vars: Vec<(String, String)>,
        container_vars: Vec<(String, String)>,
        selected_index: usize,
        scroll_offset: usize,
        search_query: String,
    },

    /// Script viewer (raw content)
    ScriptViewer {
        host_index: usize,
        container_index: usize,
        script_path: String,
        script_content: Vec<String>,
        scroll_offset: usize,
    },

    /// Script editor
    ScriptEdit {
        host_index: usize,
        container_index: usize,
        editing_script: DeploymentScript,
        focused_section: ScriptSection,
        selected_index: usize,
        editing_mode: bool,
    },

    /// Environment variable editor
    EnvVarEditor {
        host_index: usize,
        container_index: usize,
        editing_script: DeploymentScript,
        var_index: Option<usize>,
        key_buffer: String,
        value_buffer: String,
        editing_key: bool,
        is_new: bool,
    },

    /// File browser for selecting scripts
    FileBrowser {
        host_index: usize,
        container_index: usize,
        current_path: String,
        entries: Vec<FileEntry>,
        selected_index: usize,
        loading: bool,
    },

    /// Confirmation dialog for docker actions
    ConfirmDockerAction {
        action: DockerPendingAction,
        return_mode: Box<AppMode>,
    },
}

/// Fields in rsync mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsyncField {
    SourcePath,
    DestPath,
}

/// Fields in the host edit form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostField {
    HostAlias,
    Hostname,
    User,
    Port,
    IdentityFiles,
    ProxyJump,
    SshFlags,
    Shell,
    Tags,
    Note,
}

impl HostField {
    /// Get the next field (for Tab navigation)
    pub fn next(&self) -> HostField {
        match self {
            HostField::HostAlias => HostField::Hostname,
            HostField::Hostname => HostField::User,
            HostField::User => HostField::Port,
            HostField::Port => HostField::IdentityFiles,
            HostField::IdentityFiles => HostField::ProxyJump,
            HostField::ProxyJump => HostField::SshFlags,
            HostField::SshFlags => HostField::Shell,
            HostField::Shell => HostField::Tags,
            HostField::Tags => HostField::Note,
            HostField::Note => HostField::HostAlias,
        }
    }

    /// Get the previous field (for Shift+Tab navigation)
    pub fn previous(&self) -> HostField {
        match self {
            HostField::HostAlias => HostField::Note,
            HostField::Hostname => HostField::HostAlias,
            HostField::User => HostField::Hostname,
            HostField::Port => HostField::User,
            HostField::IdentityFiles => HostField::Port,
            HostField::ProxyJump => HostField::IdentityFiles,
            HostField::SshFlags => HostField::ProxyJump,
            HostField::Shell => HostField::SshFlags,
            HostField::Tags => HostField::Shell,
            HostField::Note => HostField::Tags,
        }
    }
}

/// Sort order for hosts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    /// Sort by host name (default)
    #[default]
    Name,
    /// Sort by hostname/IP
    Hostname,
    /// Sort by last used (most recent first)
    LastUsed,
    /// Sort by user
    User,
    /// Sort by tags (alphabetically by first tag)
    Tags,
}

impl SortBy {
    /// Get the next sort option (for cycling)
    pub fn next(&self) -> SortBy {
        match self {
            SortBy::Name => SortBy::Hostname,
            SortBy::Hostname => SortBy::LastUsed,
            SortBy::LastUsed => SortBy::User,
            SortBy::User => SortBy::Tags,
            SortBy::Tags => SortBy::Name,
        }
    }

    /// Get the display label for this sort option
    pub fn label(&self) -> &'static str {
        match self {
            SortBy::Name => "Name",
            SortBy::Hostname => "Hostname",
            SortBy::LastUsed => "Last Used",
            SortBy::User => "User",
            SortBy::Tags => "Tags",
        }
    }
}

// ==================== Docker-related types ====================

/// Sections in script edit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSection {
    EnvVars,
    Volumes,
    Ports,
    Network,
}

impl ScriptSection {
    pub fn next(&self) -> Self {
        match self {
            ScriptSection::EnvVars => ScriptSection::Volumes,
            ScriptSection::Volumes => ScriptSection::Ports,
            ScriptSection::Ports => ScriptSection::Network,
            ScriptSection::Network => ScriptSection::EnvVars,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ScriptSection::EnvVars => ScriptSection::Network,
            ScriptSection::Volumes => ScriptSection::EnvVars,
            ScriptSection::Ports => ScriptSection::Volumes,
            ScriptSection::Network => ScriptSection::Ports,
        }
    }
}

/// Pending docker actions requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerPendingAction {
    DockerPull { host_index: usize, container_id: String, container_name: String, image_name: String },
    DockerRestart { host_index: usize, container_id: String, container_name: String },
    DockerStop { host_index: usize, container_id: String, container_name: String },
    DockerStart { host_index: usize, container_id: String, container_name: String },
    DockerRemove { host_index: usize, container_id: String, container_name: String, image_name: String, remove_volumes: bool, remove_image: bool },
    RunScript { host_index: usize, script_path: String },
    EnableSudo { host_index: usize },
}

impl DockerPendingAction {
    pub fn description(&self) -> String {
        match self {
            DockerPendingAction::DockerPull { container_name, .. } => {
                format!("Pull latest image for '{}'?", container_name)
            }
            DockerPendingAction::DockerRestart { container_name, .. } => {
                format!("Restart container '{}'?", container_name)
            }
            DockerPendingAction::DockerStop { container_name, .. } => {
                format!("Stop container '{}'?", container_name)
            }
            DockerPendingAction::DockerStart { container_name, .. } => {
                format!("Start container '{}'?", container_name)
            }
            DockerPendingAction::DockerRemove { container_name, remove_volumes, remove_image, .. } => {
                let mut desc = format!("Remove container '{}'?", container_name);
                if *remove_volumes {
                    desc.push_str(" + volumes");
                }
                if *remove_image {
                    desc.push_str(" + image");
                }
                desc.push_str(" (Cannot be undone!)");
                desc
            }
            DockerPendingAction::RunScript { script_path, .. } => {
                format!("Run deployment script '{}'?", script_path)
            }
            DockerPendingAction::EnableSudo { .. } => {
                "Use sudo for Docker commands? (Required if Docker runs as root)".to_string()
            }
        }
    }
}

