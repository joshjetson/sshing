use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an SSH host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    // SSH config fields (stored in ~/.ssh/config)
    /// Host alias/name (unique identifier)
    pub host: String,

    /// IP address or domain name
    pub hostname: String,

    /// SSH username
    pub user: Option<String>,

    /// SSH port (defaults to 22 if None)
    pub port: Option<u16>,

    /// Paths to SSH identity files (private keys)
    pub identity_file: Option<Vec<String>>,

    /// Jump host configuration (ProxyJump)
    pub proxy_jump: Option<String>,

    // Extended metadata (stored in companion file)
    /// User notes about this host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Tags for grouping/filtering (e.g., "prod", "staging", "web")
    #[serde(default)]
    pub tags: Vec<String>,

    /// SSH flags to use when connecting (e.g., "-t", "-A")
    #[serde(default)]
    pub ssh_flags: Vec<String>,

    /// Shell to execute after connection (e.g., "zsh", "bash")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// Timestamp of last connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
}

impl Host {
    /// Create a new Host with required fields
    pub fn new(host: String, hostname: String) -> Self {
        Host {
            host,
            hostname,
            user: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            note: None,
            tags: Vec::new(),
            ssh_flags: Vec::new(),
            shell: None,
            last_used: None,
        }
    }

    /// Get the effective port (22 if not specified)
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or(22)
    }

    /// Check if this host has any SSH keys configured
    pub fn has_keys(&self) -> bool {
        self.identity_file
            .as_ref()
            .map(|keys| !keys.is_empty())
            .unwrap_or(false)
    }

    /// Update the last_used timestamp to now
    pub fn mark_used(&mut self) {
        self.last_used = Some(Utc::now());
    }

    /// Check if this host matches a search query
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        self.host.to_lowercase().contains(&query_lower)
            || self.hostname.to_lowercase().contains(&query_lower)
            || self.user.as_ref()
                .map(|u| u.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            || self.note.as_ref()
                .map(|n| n.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
            || self.tags.iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }

    /// Check if this host has any of the given tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        if tags.is_empty() {
            return true; // No filter means show all
        }

        tags.iter().any(|tag| self.tags.contains(tag))
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
    }
}

impl Eq for Host {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_host() {
        let host = Host::new("test".to_string(), "192.168.1.1".to_string());
        assert_eq!(host.host, "test");
        assert_eq!(host.hostname, "192.168.1.1");
        assert_eq!(host.effective_port(), 22);
    }

    #[test]
    fn test_effective_port() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        assert_eq!(host.effective_port(), 22);

        host.port = Some(2222);
        assert_eq!(host.effective_port(), 2222);
    }

    #[test]
    fn test_has_keys() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        assert!(!host.has_keys());

        host.identity_file = Some(vec!["/home/user/.ssh/id_rsa".to_string()]);
        assert!(host.has_keys());
    }

    #[test]
    fn test_matches_search() {
        let mut host = Host::new("prod-web".to_string(), "192.168.1.1".to_string());
        host.user = Some("ubuntu".to_string());
        host.tags = vec!["prod".to_string(), "web".to_string()];

        assert!(host.matches_search("prod"));
        assert!(host.matches_search("web"));
        assert!(host.matches_search("192.168"));
        assert!(host.matches_search("ubuntu"));
        assert!(!host.matches_search("staging"));
    }

    #[test]
    fn test_has_any_tag() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.tags = vec!["prod".to_string(), "web".to_string()];

        assert!(host.has_any_tag(&vec!["prod".to_string()]));
        assert!(host.has_any_tag(&vec!["web".to_string()]));
        assert!(host.has_any_tag(&vec!["prod".to_string(), "db".to_string()]));
        assert!(!host.has_any_tag(&vec!["staging".to_string()]));
        assert!(host.has_any_tag(&vec![])); // Empty filter matches all
    }
}
