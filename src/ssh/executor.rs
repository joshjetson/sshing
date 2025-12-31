use anyhow::{Context, Result};
use std::process::Command;

use crate::models::Host;

/// Connect to a host via SSH
pub fn connect_to_host(host: &Host) -> Result<()> {
    let mut cmd = Command::new("ssh");

    // Add user if specified
    if let Some(ref user) = host.user {
        cmd.arg("-l").arg(user);
    }

    // Add port if specified
    if let Some(port) = host.port {
        cmd.arg("-p").arg(port.to_string());
    }

    // Add identity files if specified
    if let Some(ref identity_files) = host.identity_file {
        for file in identity_files {
            cmd.arg("-i").arg(file);
        }
    }

    // Add ProxyJump if specified
    if let Some(ref proxy_jump) = host.proxy_jump {
        cmd.arg("-J").arg(proxy_jump);
    }

    // Add SSH flags (e.g., -t, -A, -X, etc.)
    for flag in &host.ssh_flags {
        cmd.arg(flag);
    }

    // Add the hostname
    cmd.arg(&host.hostname);

    // If a shell is specified, execute it
    if let Some(ref shell) = host.shell {
        cmd.arg(shell);
    }

    // Execute SSH - this will take over the terminal
    let status = cmd
        .status()
        .context("Failed to execute SSH command")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "SSH connection failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

/// Build SSH command string for display purposes
pub fn build_ssh_command(host: &Host) -> String {
    let mut parts = vec!["ssh".to_string()];

    if let Some(ref user) = host.user {
        parts.push(format!("-l {}", user));
    }

    if let Some(port) = host.port {
        parts.push(format!("-p {}", port));
    }

    if let Some(ref identity_files) = host.identity_file {
        for file in identity_files {
            parts.push(format!("-i {}", file));
        }
    }

    if let Some(ref proxy_jump) = host.proxy_jump {
        parts.push(format!("-J {}", proxy_jump));
    }

    parts.push(host.hostname.clone());

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ssh_command_simple() {
        let host = Host::new("test".to_string(), "192.168.1.1".to_string());
        let cmd = build_ssh_command(&host);
        assert_eq!(cmd, "ssh 192.168.1.1");
    }

    #[test]
    fn test_build_ssh_command_with_user() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.user = Some("ubuntu".to_string());
        let cmd = build_ssh_command(&host);
        assert_eq!(cmd, "ssh -l ubuntu 192.168.1.1");
    }

    #[test]
    fn test_build_ssh_command_with_port() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.port = Some(2222);
        let cmd = build_ssh_command(&host);
        assert_eq!(cmd, "ssh -p 2222 192.168.1.1");
    }

    #[test]
    fn test_build_ssh_command_full() {
        let mut host = Host::new("test".to_string(), "192.168.1.1".to_string());
        host.user = Some("ubuntu".to_string());
        host.port = Some(2222);
        host.identity_file = Some(vec!["~/.ssh/id_rsa".to_string()]);
        host.proxy_jump = Some("bastion".to_string());

        let cmd = build_ssh_command(&host);
        assert!(cmd.contains("ssh"));
        assert!(cmd.contains("-l ubuntu"));
        assert!(cmd.contains("-p 2222"));
        assert!(cmd.contains("-i ~/.ssh/id_rsa"));
        assert!(cmd.contains("-J bastion"));
        assert!(cmd.contains("192.168.1.1"));
    }
}
