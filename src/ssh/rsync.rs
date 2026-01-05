use anyhow::Result;
use std::process::Command;

use crate::models::Host;

/// Execute rsync with SSH connection
pub fn execute_rsync(
    host: &Host,
    source: &str,
    dest: &str,
    to_host: bool,
    _password: Option<String>,
    compress: bool,
) -> Result<(bool, String)> {
    let mut cmd = Command::new("rsync");

    // Build SSH options
    let mut ssh_options = String::new();

    // Add user if specified
    if let Some(ref user) = host.user {
        ssh_options.push_str(&format!("-l {} ", user));
    }

    // Add port if specified
    if let Some(port) = host.port {
        ssh_options.push_str(&format!("-p {} ", port));
    }

    // Add identity files if specified
    if let Some(ref identity_files) = host.identity_file {
        for file in identity_files {
            ssh_options.push_str(&format!("-i {} ", file));
        }
    }

    // Set StrictHostKeyChecking=no to avoid prompts on new hosts
    ssh_options.push_str("-o StrictHostKeyChecking=no ");

    let ssh_arg = format!("ssh {}", ssh_options.trim());

    // Add -e ssh option
    if !ssh_options.trim().is_empty() {
        cmd.arg("-e").arg(&ssh_arg);
    }

    // Add -a (archive) flag by default
    cmd.arg("-a");

    // Add -z (compress) flag if requested
    if compress {
        cmd.arg("-z");
    }

    // Add source and destination
    if to_host {
        // Sending to host: local source to remote dest
        cmd.arg(source);
        cmd.arg(format!("{}:{}", host.hostname, dest));
    } else {
        // Receiving from host: remote source to local dest
        cmd.arg(format!("{}:{}", host.hostname, source));
        cmd.arg(dest);
    }

    // Execute rsync
    let output = cmd.output()?;

    let success = output.status.success();
    let output_str = if success {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok((success, output_str))
}
