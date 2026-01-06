use regex::Regex;

use crate::models::{DeploymentScript, EnvVar, PortMapping, VolumeMount};

/// Parse a deployment script to extract configuration
pub fn parse_script(content: &str, path: &str, client_name: &str) -> DeploymentScript {
    let mut script = DeploymentScript::new(path.to_string(), client_name.to_string());
    script.raw_content = content.to_string();

    // Extract NAME variable
    if let Some(name) = extract_variable(content, "NAME") {
        script.container_name = name;
    }

    // Extract REPO variable
    if let Some(repo) = extract_variable(content, "REPO") {
        script.repo = repo;
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
    let re = Regex::new(r#"--restart=(\S+)"#).unwrap();
    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Generate script content from a DeploymentScript
pub fn generate_script(script: &DeploymentScript) -> String {
    let mut lines = Vec::new();

    lines.push("#! /usr/bin/env bash".to_string());
    lines.push(String::new());
    lines.push("#Configuration".to_string());
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
        create_parts.push(format!("--net={}", network));
    }

    create_parts.push("--name $NAME --restart=unless-stopped".to_string());

    // Add environment variables
    for env in &script.env_vars {
        create_parts.push(format!("-e {}=\"{}\"", env.key, env.value));
    }

    // Add ports
    for port in &script.ports {
        create_parts.push(format!("-p {}:{}", port.host_port, port.container_port));
    }

    // Add volumes
    for vol in &script.volumes {
        let ro = if vol.read_only { ":ro" } else { "" };
        create_parts.push(format!("-v {}:{}{}", vol.host_path, vol.container_path, ro));
    }

    create_parts.push(" $REPO".to_string());

    // Join with line continuations
    let create_cmd = create_parts.join(" \\\n");
    lines.push(create_cmd);
    lines.push(String::new());

    lines.push("docker start $NAME".to_string());

    lines.join("\n")
}
