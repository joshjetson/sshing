use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::Host;

/// Metadata for a single host (fields not in SSH config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub ssh_flags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
}

/// Container for all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Version of the metadata format
    #[serde(default = "default_version")]
    pub version: String,

    /// Global tag pool (available for all hosts)
    #[serde(default)]
    pub global_tags: Vec<String>,

    /// Map of host alias to metadata
    #[serde(default)]
    pub hosts: HashMap<String, HostMetadata>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Metadata {
    /// Create a new empty metadata container
    pub fn new() -> Self {
        Metadata {
            version: default_version(),
            global_tags: Vec::new(),
            hosts: HashMap::new(),
        }
    }

    /// Add a tag to the global tag pool
    pub fn add_global_tag(&mut self, tag: String) {
        if !self.global_tags.contains(&tag) {
            self.global_tags.push(tag);
            self.global_tags.sort();
        }
    }

    /// Get all global tags
    pub fn get_global_tags(&self) -> Vec<String> {
        self.global_tags.clone()
    }

    /// Get metadata for a host
    pub fn get(&self, host_alias: &str) -> Option<&HostMetadata> {
        self.hosts.get(host_alias)
    }

    /// Set metadata for a host
    pub fn set(&mut self, host_alias: String, metadata: HostMetadata) {
        self.hosts.insert(host_alias, metadata);
    }

    /// Remove metadata for a host
    pub fn remove(&mut self, host_alias: &str) -> Option<HostMetadata> {
        self.hosts.remove(host_alias)
    }

    /// Update a host with its metadata
    pub fn apply_to_host(&self, host: &mut Host) {
        if let Some(metadata) = self.get(&host.host) {
            host.note = metadata.note.clone();
            host.tags = metadata.tags.clone();
            host.ssh_flags = metadata.ssh_flags.clone();
            host.shell = metadata.shell.clone();
            host.last_used = metadata.last_used;
        }
    }

    /// Extract metadata from a host
    pub fn extract_from_host(&mut self, host: &Host) {
        let metadata = HostMetadata {
            note: host.note.clone(),
            tags: host.tags.clone(),
            ssh_flags: host.ssh_flags.clone(),
            shell: host.shell.clone(),
            last_used: host.last_used,
        };

        self.set(host.host.clone(), metadata);
    }

    /// Merge metadata into a list of hosts
    pub fn merge_into_hosts(&self, hosts: &mut [Host]) {
        for host in hosts.iter_mut() {
            self.apply_to_host(host);
        }
    }

    /// Extract metadata from a list of hosts
    pub fn extract_from_hosts(&mut self, hosts: &[Host]) {
        for host in hosts {
            self.extract_from_host(host);
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Load metadata from file
pub fn load_metadata(path: &Path) -> Result<Metadata> {
    if !path.exists() {
        // Return empty metadata if file doesn't exist
        return Ok(Metadata::new());
    }

    let content = fs::read_to_string(path)
        .context("Failed to read metadata file")?;

    let metadata: Metadata = serde_json::from_str(&content)
        .context("Failed to parse metadata JSON")?;

    Ok(metadata)
}

/// Save metadata to file
pub fn save_metadata(path: &Path, metadata: &Metadata) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create metadata directory")?;
    }

    let content = serde_json::to_string_pretty(metadata)
        .context("Failed to serialize metadata")?;

    fs::write(path, content)
        .context("Failed to write metadata file")?;

    Ok(())
}

/// Get the default metadata file path (~/.ssh/sshing.json)
pub fn default_metadata_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ssh")
        .join("sshing.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_new() {
        let metadata = Metadata::new();
        assert_eq!(metadata.version, "1.0");
        assert!(metadata.hosts.is_empty());
    }

    #[test]
    fn test_metadata_get_set() {
        let mut metadata = Metadata::new();

        let host_meta = HostMetadata {
            note: Some("Test note".to_string()),
            tags: vec!["prod".to_string()],
            last_used: None,
        };

        metadata.set("test-host".to_string(), host_meta.clone());

        let retrieved = metadata.get("test-host").unwrap();
        assert_eq!(retrieved.note, Some("Test note".to_string()));
        assert_eq!(retrieved.tags, vec!["prod".to_string()]);
    }

    #[test]
    fn test_apply_to_host() {
        let mut metadata = Metadata::new();

        metadata.set(
            "test".to_string(),
            HostMetadata {
                note: Some("Production server".to_string()),
                tags: vec!["prod".to_string(), "web".to_string()],
                last_used: None,
            },
        );

        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        metadata.apply_to_host(&mut host);

        assert_eq!(host.note, Some("Production server".to_string()));
        assert_eq!(host.tags, vec!["prod".to_string(), "web".to_string()]);
    }

    #[test]
    fn test_extract_from_host() {
        let mut metadata = Metadata::new();

        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.note = Some("Test note".to_string());
        host.tags = vec!["dev".to_string()];

        metadata.extract_from_host(&host);

        let retrieved = metadata.get("test").unwrap();
        assert_eq!(retrieved.note, Some("Test note".to_string()));
        assert_eq!(retrieved.tags, vec!["dev".to_string()]);
    }

    #[test]
    fn test_serialization() {
        let mut metadata = Metadata::new();

        metadata.set(
            "test".to_string(),
            HostMetadata {
                note: Some("Test".to_string()),
                tags: vec!["prod".to_string()],
                last_used: None,
            },
        );

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: Metadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, metadata.version);
        assert_eq!(deserialized.hosts.len(), 1);
    }
}
