use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::models::Host;

/// Represents an SSH config file
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Path to the SSH config file
    pub path: PathBuf,
    /// Parsed hosts
    pub hosts: Vec<Host>,
}

impl SshConfig {
    /// Create a new empty SSH config
    pub fn new(path: PathBuf) -> Self {
        SshConfig {
            path,
            hosts: Vec::new(),
        }
    }

    /// Add a new host to the config
    pub fn add_host(&mut self, host: Host) {
        self.hosts.push(host);
    }

    /// Update a host in the config
    pub fn update_host(&mut self, index: usize, host: Host) -> Result<()> {
        if index < self.hosts.len() {
            self.hosts[index] = host;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Host index out of bounds"))
        }
    }

    /// Remove a host from the config
    pub fn remove_host(&mut self, index: usize) -> Result<Host> {
        if index < self.hosts.len() {
            Ok(self.hosts.remove(index))
        } else {
            Err(anyhow::anyhow!("Host index out of bounds"))
        }
    }

    /// Find a host by its alias
    pub fn find_host(&self, alias: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.host == alias)
    }

    /// Check if a host alias already exists
    pub fn host_exists(&self, alias: &str) -> bool {
        self.find_host(alias).is_some()
    }
}

/// Parse an SSH config file
pub fn parse_ssh_config(path: &Path) -> Result<SshConfig> {
    if !path.exists() {
        // Create an empty config file if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create .ssh directory")?;
        }
        fs::File::create(path)
            .context("Failed to create SSH config file")?;
        return Ok(SshConfig::new(path.to_path_buf()));
    }

    let content = fs::read_to_string(path)
        .context("Failed to read SSH config file")?;

    let hosts = parse_config_content(&content)?;

    Ok(SshConfig {
        path: path.to_path_buf(),
        hosts,
    })
}

/// Parse the content of an SSH config file
fn parse_config_content(content: &str) -> Result<Vec<Host>> {
    let mut hosts = Vec::new();
    let mut current_host: Option<Host> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split into directive and value
        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }

        let directive = parts[0].to_lowercase();
        let value = parts[1].trim();

        match directive.as_str() {
            "host" => {
                // Save previous host if exists
                if let Some(host) = current_host.take() {
                    hosts.push(host);
                }

                // Skip wildcard hosts
                if value.contains('*') || value.contains('?') {
                    continue;
                }

                // Start a new host
                current_host = Some(Host::new(value.to_string(), String::new()));
            }
            "hostname" => {
                if let Some(ref mut host) = current_host {
                    host.hostname = value.to_string();
                }
            }
            "user" => {
                if let Some(ref mut host) = current_host {
                    host.user = Some(value.to_string());
                }
            }
            "port" => {
                if let Some(ref mut host) = current_host {
                    if let Ok(port) = value.parse::<u16>() {
                        host.port = Some(port);
                    }
                }
            }
            "identityfile" => {
                if let Some(ref mut host) = current_host {
                    let identity_files = host.identity_file.get_or_insert_with(Vec::new);
                    // Expand ~ to home directory
                    let expanded = expand_tilde(value);
                    identity_files.push(expanded);
                }
            }
            "proxyjump" => {
                if let Some(ref mut host) = current_host {
                    host.proxy_jump = Some(value.to_string());
                }
            }
            _ => {
                // Ignore other directives for now
            }
        }
    }

    // Don't forget the last host
    if let Some(host) = current_host {
        hosts.push(host);
    }

    // Filter out hosts without hostname (incomplete entries)
    hosts.retain(|h| !h.hostname.is_empty());

    Ok(hosts)
}

/// Write SSH config to file
pub fn write_ssh_config(config: &SshConfig) -> Result<()> {
    let mut content = String::new();

    // Add header comment
    content.push_str("# SSH Config managed by sshing\n");
    content.push_str("# Edit with caution or use sshing to manage hosts\n\n");

    for host in &config.hosts {
        write_host_block(&mut content, host);
    }

    // Write to file with proper permissions (0600)
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config.path)
        .context("Failed to open SSH config for writing")?;

    file.write_all(content.as_bytes())
        .context("Failed to write SSH config")?;

    // Set proper permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(&config.path, permissions)
            .context("Failed to set SSH config permissions")?;
    }

    Ok(())
}

/// Write a single host block to the config content
fn write_host_block(content: &mut String, host: &Host) {
    content.push_str(&format!("Host {}\n", host.host));
    content.push_str(&format!("  HostName {}\n", host.hostname));

    if let Some(ref user) = host.user {
        content.push_str(&format!("  User {}\n", user));
    }

    if let Some(port) = host.port {
        content.push_str(&format!("  Port {}\n", port));
    }

    if let Some(ref identity_files) = host.identity_file {
        for file in identity_files {
            content.push_str(&format!("  IdentityFile {}\n", file));
        }
    }

    if let Some(ref proxy_jump) = host.proxy_jump {
        content.push_str(&format!("  ProxyJump {}\n", proxy_jump));
    }

    content.push('\n');
}

/// Expand ~ in paths to home directory
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

/// Get the default SSH config path (~/.ssh/config)
pub fn default_ssh_config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ssh")
        .join("config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config = r#"
Host test-server
  HostName 192.168.1.1
  User ubuntu
  Port 2222

Host prod-db
  HostName 10.0.0.5
  User admin
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 2);

        let host1 = &hosts[0];
        assert_eq!(host1.host, "test-server");
        assert_eq!(host1.hostname, "192.168.1.1");
        assert_eq!(host1.user, Some("ubuntu".to_string()));
        assert_eq!(host1.port, Some(2222));

        let host2 = &hosts[1];
        assert_eq!(host2.host, "prod-db");
        assert_eq!(host2.hostname, "10.0.0.5");
        assert_eq!(host2.user, Some("admin".to_string()));
        assert_eq!(host2.port, None);
    }

    #[test]
    fn test_parse_with_identity_files() {
        let config = r#"
Host github
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_rsa
  IdentityFile ~/.ssh/id_ed25519
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 1);

        let host = &hosts[0];
        assert_eq!(host.host, "github");
        assert!(host.identity_file.is_some());
        assert_eq!(host.identity_file.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_parse_with_proxy_jump() {
        let config = r#"
Host internal
  HostName 10.0.0.1
  ProxyJump bastion
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 1);

        let host = &hosts[0];
        assert_eq!(host.proxy_jump, Some("bastion".to_string()));
    }

    #[test]
    fn test_skip_wildcard_hosts() {
        let config = r#"
Host *
  ServerAliveInterval 60

Host test
  HostName 192.168.1.1
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].host, "test");
    }

    #[test]
    fn test_write_host_block() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.user = Some("ubuntu".to_string());
        host.port = Some(2222);
        host.identity_file = Some(vec!["~/.ssh/id_rsa".to_string()]);

        let mut content = String::new();
        write_host_block(&mut content, &host);

        assert!(content.contains("Host test"));
        assert!(content.contains("HostName 192.168.1.1"));
        assert!(content.contains("User ubuntu"));
        assert!(content.contains("Port 2222"));
        assert!(content.contains("IdentityFile ~/.ssh/id_rsa"));
    }
}
