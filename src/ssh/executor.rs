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
