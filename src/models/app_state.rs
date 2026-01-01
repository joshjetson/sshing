use crate::models::Host;

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
    },
}

/// Fields in rsync mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsyncField {
    SourcePath,
    DestPath,
    Direction,
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
}

impl SortBy {
    /// Get the next sort option (for cycling)
    pub fn next(&self) -> SortBy {
        match self {
            SortBy::Name => SortBy::Hostname,
            SortBy::Hostname => SortBy::LastUsed,
            SortBy::LastUsed => SortBy::User,
            SortBy::User => SortBy::Name,
        }
    }

    /// Get the display label for this sort option
    pub fn label(&self) -> &'static str {
        match self {
            SortBy::Name => "Name",
            SortBy::Hostname => "Hostname",
            SortBy::LastUsed => "Last Used",
            SortBy::User => "User",
        }
    }
}

