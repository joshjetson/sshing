use regex::Regex;

use crate::models::{DeploymentScript, EnvVar, PortMapping, VolumeMount};

/// Parse a deployment script to extract configuration
pub fn parse_script(content: &str, path: &str, client_name: &str) -> DeploymentScript {
    let mut script = DeploymentScript::new(path.to_string(), client_name.to_string());
    script.raw_content = content.to_string();

    // Extract NAME variable first, then fall back to --name flag
    if let Some(name) = extract_variable(content, "NAME") {
        script.container_name = name;
    } else if let Some(name) = extract_container_name_flag(content) {
        script.container_name = name;
    }

    // Extract REPO variable, or try to get image from docker command
    if let Some(repo) = extract_variable(content, "REPO") {
        script.repo = repo;
    } else if let Some(image) = extract_docker_image(content) {
        script.repo = image;
    }

    // Extract environment variables
    script.env_vars = extract_env_vars(content);

    // Extract volume mounts
    script.volumes = extract_volumes(content);

    // Extract port mappings
    script.ports = extract_ports(content);

    // Extract network
    script.network = extract_network(content);

    // Extract restart policy
    script.restart_policy = extract_restart_policy(content);

    script
}

/// Extract a shell variable assignment: NAME='value' or NAME="value"
fn extract_variable(content: &str, var_name: &str) -> Option<String> {
    let patterns = [
        format!(r#"{}='([^']*)'"#, var_name),
        format!(r#"{}="([^"]*)""#, var_name),
        format!(r#"{}=(\S+)"#, var_name),
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(value) = caps.get(1) {
                    return Some(value.as_str().to_string());
                }
            }
        }
    }
    None
}

/// Extract environment variables from -e flags
fn extract_env_vars(content: &str) -> Vec<EnvVar> {
    let mut env_vars = Vec::new();

    let patterns = [
        r#"-e\s+([A-Za-z_][A-Za-z0-9_]*)="([^"]*)""#,
        r#"-e\s+([A-Za-z_][A-Za-z0-9_]*)='([^']*)'"#,
        r#"-e\s*([A-Za-z_][A-Za-z0-9_]*)=(\S+)"#,
        r#"-e([A-Za-z_][A-Za-z0-9_]*)=(\S+)"#,
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(content) {
                if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                    let key_str = key.as_str().to_string();
                    let value_str = value.as_str().to_string();

                    // Avoid duplicates
                    if !env_vars.iter().any(|e: &EnvVar| e.key == key_str) {
                        env_vars.push(EnvVar::new(key_str, value_str));
                    }
                }
            }
        }
    }

    env_vars
}

/// Extract volume mounts from -v flags
fn extract_volumes(content: &str) -> Vec<VolumeMount> {
    let mut volumes = Vec::new();

    let re = Regex::new(r#"-v\s+([^:\s]+):([^:\s]+)(:ro)?"#).unwrap();

    for caps in re.captures_iter(content) {
        if let (Some(host), Some(container)) = (caps.get(1), caps.get(2)) {
            let read_only = caps.get(3).is_some();
            volumes.push(VolumeMount {
                host_path: host.as_str().to_string(),
                container_path: container.as_str().to_string(),
                read_only,
            });
        }
    }

    volumes
}

/// Extract port mappings from -p flags
fn extract_ports(content: &str) -> Vec<PortMapping> {
    let mut ports = Vec::new();

    let re = Regex::new(r#"-p\s+(\d+):(\d+)(/\w+)?"#).unwrap();

    for caps in re.captures_iter(content) {
        if let (Some(host), Some(container)) = (caps.get(1), caps.get(2)) {
            if let (Ok(host_port), Ok(container_port)) = (
                host.as_str().parse::<u16>(),
                container.as_str().parse::<u16>(),
            ) {
                let protocol = caps
                    .get(3)
                    .map(|p| p.as_str().trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "tcp".to_string());

                ports.push(PortMapping {
                    host_port,
                    container_port,
                    protocol,
                });
            }
        }
    }

    ports
}

/// Extract network from --net or --network flag
fn extract_network(content: &str) -> Option<String> {
    let patterns = [
        r#"--net=(\S+)"#,
        r#"--network=(\S+)"#,
        r#"--net\s+(\S+)"#,
        r#"--network\s+(\S+)"#,
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(network) = caps.get(1) {
                    return Some(network.as_str().to_string());
                }
            }
        }
    }
    None
}

/// Extract restart policy from --restart flag
fn extract_restart_policy(content: &str) -> Option<String> {
    let patterns = [
        r#"--restart=(\S+)"#,
        r#"--restart\s+(\S+)"#,
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }
    None
}

/// Extract container name from --name flag (for scripts without NAME variable)
fn extract_container_name_flag(content: &str) -> Option<String> {
    let patterns = [
        r#"--name=([^\s\\]+)"#,
        r#"--name\s+([^\s\\]+)"#,
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().trim_matches('"').trim_matches('\'');
                    // Skip if it's a variable reference like $NAME
                    if !name.starts_with('$') {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract docker image from the end of docker run/create command
fn extract_docker_image(content: &str) -> Option<String> {
    // Look for the image at the end of docker run or docker create command
    // The image is typically the last argument before any command to run
    let patterns = [
        // Match image after all flags, before newline or end
        r#"(?:docker\s+(?:run|create)[^\n]*?)\s+([a-zA-Z0-9._/-]+(?::[a-zA-Z0-9._-]+)?)\s*(?:\n|$)"#,
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    let image = m.as_str();
                    // Skip if it looks like a flag or variable
                    if !image.starts_with('-') && !image.starts_with('$') {
                        return Some(image.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Generate script content from a DeploymentScript (only used for new scripts)
pub fn generate_script(script: &DeploymentScript) -> String {
    let mut lines = Vec::new();

    lines.push("#!/usr/bin/env bash".to_string());
    lines.push(String::new());
    lines.push("# Configuration".to_string());
    lines.push(format!("NAME='{}'", script.container_name));
    lines.push(format!("REPO=\"{}\"", script.repo));
    lines.push(String::new());

    lines.push("docker pull $REPO".to_string());
    lines.push("docker stop $NAME".to_string());
    lines.push("docker rm $NAME".to_string());
    lines.push(String::new());

    // Build docker create command
    let mut create_parts = vec!["docker create".to_string()];

    if let Some(ref network) = script.network {
        create_parts.push(format!("  --net={}", network));
    }

    create_parts.push("  --name $NAME".to_string());
    create_parts.push("  --restart=unless-stopped".to_string());

    // Add ports
    for port in &script.ports {
        create_parts.push(format!("  -p {}:{}", port.host_port, port.container_port));
    }

    // Add volumes
    for vol in &script.volumes {
        let ro = if vol.read_only { ":ro" } else { "" };
        create_parts.push(format!("  -v {}:{}{}", vol.host_path, vol.container_path, ro));
    }

    // Add environment variables
    for env in &script.env_vars {
        create_parts.push(format!("  -e {}=\"{}\"", env.key, env.value));
    }

    create_parts.push("  $REPO".to_string());

    // Join with line continuations
    let create_cmd = create_parts.join(" \\\n");
    lines.push(create_cmd);
    lines.push(String::new());

    lines.push("docker start $NAME".to_string());

    lines.join("\n")
}

/// Apply changes from a DeploymentScript back to its raw_content in-place
/// This preserves the original script structure while updating env vars
pub fn apply_script_changes(script: &DeploymentScript, original_env_vars: &[EnvVar]) -> String {
    let mut content = script.raw_content.clone();

    // Find which env vars were added, modified, or removed
    let current_keys: std::collections::HashSet<_> = script.env_vars.iter().map(|e| &e.key).collect();
    let original_keys: std::collections::HashSet<_> = original_env_vars.iter().map(|e| &e.key).collect();

    // Remove deleted env vars first
    for orig in original_env_vars {
        if !current_keys.contains(&orig.key) {
            content = remove_env_var(&content, &orig.key);
        }
    }

    // Update existing and add new env vars
    for env in &script.env_vars {
        if original_keys.contains(&env.key) {
            // Update existing
            content = update_env_var(&content, &env.key, &env.value);
        } else {
            // Add new
            content = add_env_var(&content, &env.key, &env.value);
        }
    }

    content
}

/// Update an existing environment variable in the script content
fn update_env_var(content: &str, key: &str, new_value: &str) -> String {
    // Match various forms of -e KEY=VALUE
    let patterns = [
        format!(r#"(-e\s+{}=)"([^"]*)""#, regex::escape(key)),
        format!(r#"(-e\s+{}=)'([^']*)'"#, regex::escape(key)),
        format!(r#"(-e\s+{}=)([^\s\\]+)"#, regex::escape(key)),
        format!(r#"(-e{}=)"([^"]*)""#, regex::escape(key)),
        format!(r#"(-e{}=)'([^']*)'"#, regex::escape(key)),
        format!(r#"(-e{}=)([^\s\\]+)"#, regex::escape(key)),
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(content) {
                // Escape special characters in replacement value
                let escaped_value = new_value.replace('\\', "\\\\").replace('$', "\\$");
                let replacement = format!("${{1}}\"{}\"", escaped_value);
                return re.replace(content, replacement.as_str()).to_string();
            }
        }
    }

    content.to_string()
}

/// Remove an environment variable from the script content
fn remove_env_var(content: &str, key: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check if this line contains the env var we want to remove
        let env_pattern = format!(r#"-e\s*{}="#, regex::escape(key));
        if let Ok(re) = Regex::new(&env_pattern) {
            if re.is_match(trimmed) {
                // Skip this line
                // If previous line ends with \, we need to handle continuation
                if !result_lines.is_empty() {
                    let last_idx = result_lines.len() - 1;
                    let last_line = &result_lines[last_idx];

                    // Check if next line exists and is a continuation
                    let has_more_after = i + 1 < lines.len() && !lines[i + 1].trim().is_empty()
                        && (lines[i + 1].trim().starts_with('-')
                            || lines[i + 1].trim().starts_with('$')
                            || lines[i + 1].trim().starts_with('"'));

                    if last_line.trim_end().ends_with('\\') && !has_more_after {
                        // Remove the trailing \ from previous line
                        result_lines[last_idx] = last_line.trim_end().trim_end_matches('\\').trim_end().to_string();
                    }
                }
                i += 1;
                continue;
            }
        }

        result_lines.push(line.to_string());
        i += 1;
    }

    result_lines.join("\n")
}

/// Add a new environment variable to the script content
fn add_env_var(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut inserted = false;

    // Find the best place to insert:
    // 1. After the last existing -e flag
    // 2. Before the image/repo reference at the end of docker command
    // 3. After -p or -v flags

    let mut last_env_line_idx: Option<usize> = None;
    let mut last_flag_line_idx: Option<usize> = None;
    let mut docker_cmd_end_idx: Option<usize> = None;

    // Find positions
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("-e ") || trimmed.starts_with("-e") {
            last_env_line_idx = Some(i);
        }
        if trimmed.starts_with("-p ") || trimmed.starts_with("-p")
            || trimmed.starts_with("-v ") || trimmed.starts_with("-v")
            || trimmed.starts_with("--")
        {
            last_flag_line_idx = Some(i);
        }
        // Find line with $REPO or image name at end of docker command
        if (trimmed.starts_with('$') || trimmed.contains("$REPO"))
            && !trimmed.contains('=')
            && i > 0
            && lines[i - 1].trim_end().ends_with('\\')
        {
            docker_cmd_end_idx = Some(i);
        }
    }

    // Determine insertion point
    let insert_after = last_env_line_idx
        .or(last_flag_line_idx)
        .or(docker_cmd_end_idx.map(|i| i.saturating_sub(1)));

    // Detect indentation from existing -e lines or other flag lines
    let indent = if let Some(idx) = last_env_line_idx {
        let line = lines[idx];
        let trimmed = line.trim_start();
        &line[..line.len() - trimmed.len()]
    } else if let Some(idx) = last_flag_line_idx {
        let line = lines[idx];
        let trimmed = line.trim_start();
        &line[..line.len() - trimmed.len()]
    } else {
        "  "
    };

    // Escape the value properly for shell
    let escaped_value = escape_shell_value(value);
    let new_line = format!("{}-e {}={}", indent, key, escaped_value);

    for (i, line) in lines.iter().enumerate() {
        if let Some(insert_idx) = insert_after {
            if i == insert_idx && !inserted {
                // Make sure current line has continuation
                let mut current_line = line.to_string();
                if !current_line.trim_end().ends_with('\\') {
                    current_line = format!("{} \\", current_line.trim_end());
                }
                result_lines.push(current_line);

                // Add the new env var with continuation if needed
                let needs_continuation = i + 1 < lines.len()
                    && !lines[i + 1].trim().is_empty()
                    && !lines[i + 1].trim().starts_with('#');

                if needs_continuation {
                    result_lines.push(format!("{} \\", new_line));
                } else {
                    result_lines.push(new_line.clone());
                }
                inserted = true;
                continue;
            }
        }
        result_lines.push(line.to_string());
    }

    // If we couldn't find a good place, append before docker start
    if !inserted {
        // Find docker start line and insert before it
        let mut final_lines: Vec<String> = Vec::new();
        for line in &result_lines {
            if line.trim().starts_with("docker start") && !inserted {
                final_lines.push(format!("  -e {}={} \\", key, escape_shell_value(value)));
                inserted = true;
            }
            final_lines.push(line.clone());
        }
        if inserted {
            return final_lines.join("\n");
        }
    }

    result_lines.join("\n")
}

/// Escape a value for safe use in shell scripts
fn escape_shell_value(value: &str) -> String {
    // If value contains special characters, wrap in double quotes
    if value.contains(' ')
        || value.contains('$')
        || value.contains('\\')
        || value.contains('"')
        || value.contains('\'')
        || value.contains('!')
        || value.contains('`')
        || value.is_empty()
    {
        // Escape internal double quotes and backslashes
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        // Simple value can be quoted for consistency
        format!("\"{}\"", value)
    }
}
