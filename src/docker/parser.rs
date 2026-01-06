use crate::models::{Container, ContainerStatus, PortMapping};

/// Parse output from `docker ps --format '{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}'`
pub fn parse_docker_ps(output: &str, server_name: &str) -> Vec<Container> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_container_line(line, server_name))
        .collect()
}

fn parse_container_line(line: &str, server_name: &str) -> Option<Container> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 4 {
        return None;
    }

    let id = parts[0].to_string();
    let name = parts[1].to_string();
    let image = parts[2].to_string();
    let status = ContainerStatus::from_docker_status(parts[3]);
    let ports = if parts.len() > 4 {
        parse_ports(parts[4])
    } else {
        Vec::new()
    };

    Some(Container {
        id,
        name,
        image,
        status,
        ports,
        created: None,
        server_name: server_name.to_string(),
        script_path: None,
        networks: Vec::new(),
    })
}

/// Parse port mappings from docker ps output
/// Format: "0.0.0.0:8096->8080/tcp, :::8096->8080/tcp"
fn parse_ports(ports_str: &str) -> Vec<PortMapping> {
    let mut result = Vec::new();

    for port_entry in ports_str.split(", ") {
        if let Some(mapping) = parse_single_port(port_entry) {
            // Avoid duplicates (IPv4 and IPv6 often listed separately)
            if !result.iter().any(|p: &PortMapping| {
                p.host_port == mapping.host_port && p.container_port == mapping.container_port
            }) {
                result.push(mapping);
            }
        }
    }

    result
}

fn parse_single_port(port_str: &str) -> Option<PortMapping> {
    // Format: "0.0.0.0:8096->8080/tcp" or ":::8096->8080/tcp" or just "8080/tcp"

    // Check if it's a mapping (contains ->)
    if let Some(arrow_pos) = port_str.find("->") {
        let left = &port_str[..arrow_pos];
        let right = &port_str[arrow_pos + 2..];

        // Extract host port (after last :)
        let host_port: u16 = left.rsplit(':').next()?.parse().ok()?;

        // Extract container port and protocol
        let (container_port, protocol) = parse_port_protocol(right)?;

        Some(PortMapping {
            host_port,
            container_port,
            protocol,
        })
    } else {
        // Just an exposed port without mapping
        let (container_port, protocol) = parse_port_protocol(port_str)?;
        Some(PortMapping {
            host_port: container_port,
            container_port,
            protocol,
        })
    }
}

fn parse_port_protocol(s: &str) -> Option<(u16, String)> {
    let parts: Vec<&str> = s.split('/').collect();
    let port: u16 = parts[0].parse().ok()?;
    let protocol = parts.get(1).unwrap_or(&"tcp").to_string();
    Some((port, protocol))
}

/// Parse docker stats output
/// Format from dockering: CPU%|MemUsage|MemPerc|NetIO|BlockIO|PIDs (pipe-separated)
pub fn parse_docker_stats(output: &str) -> crate::models::ContainerStats {
    let line = output.lines().next().unwrap_or("");
    let parts: Vec<&str> = line.split('|').collect();

    crate::models::ContainerStats {
        cpu_percent: parts.get(0).unwrap_or(&"--").to_string(),
        memory_usage: parts.get(1).map(|s| s.split('/').next().unwrap_or(s).trim().to_string()).unwrap_or_else(|| "--".to_string()),
        memory_limit: parts.get(1).map(|s| s.split('/').nth(1).unwrap_or("--").trim().to_string()).unwrap_or_else(|| "--".to_string()),
        memory_percent: parts.get(2).unwrap_or(&"--").to_string(),
        net_io: parts.get(3).unwrap_or(&"--").to_string(),
        block_io: parts.get(4).unwrap_or(&"--").to_string(),
        pids: parts.get(5).unwrap_or(&"--").to_string(),
    }
}

/// Parse docker top output
/// Format from dockering: PID USER %CPU %MEM COMMAND (from -o pid,user,%cpu,%mem,comm)
pub fn parse_docker_top(output: &str) -> Vec<crate::models::ProcessInfo> {
    let mut processes = Vec::new();
    let mut lines = output.lines();

    // Skip header line
    lines.next();

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            processes.push(crate::models::ProcessInfo {
                pid: parts[0].to_string(),
                user: parts[1].to_string(),
                cpu: parts.get(2).unwrap_or(&"0").to_string(),
                mem: parts.get(3).unwrap_or(&"0").to_string(),
                command: parts[4..].join(" "),
            });
        }
    }

    processes
}

/// Parse docker inspect output (JSON) into ContainerInfo
pub fn parse_docker_inspect(output: &str) -> crate::models::ContainerInfo {
    let mut info = crate::models::ContainerInfo::default();

    // Simple parsing - look for key fields in the JSON
    // This is a simplified version; a full implementation would use serde_json
    for line in output.lines() {
        let line = line.trim();
        if line.contains("\"Id\":") {
            info.id = extract_json_value(line);
        } else if line.contains("\"Name\":") {
            info.name = extract_json_value(line).trim_start_matches('/').to_string();
        } else if line.contains("\"Image\":") && info.image.is_empty() {
            info.image = extract_json_value(line);
        } else if line.contains("\"Status\":") && info.status.is_empty() {
            info.status = extract_json_value(line);
        } else if line.contains("\"Created\":") && info.created.is_empty() {
            info.created = extract_json_value(line);
        } else if line.contains("\"StartedAt\":") {
            info.started = extract_json_value(line);
        } else if line.contains("\"IPAddress\":") && info.ip_address.is_empty() {
            let ip = extract_json_value(line);
            if !ip.is_empty() {
                info.ip_address = ip;
            }
        }
    }

    info
}

fn extract_json_value(line: &str) -> String {
    line.split(':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches(|c| c == '"' || c == ',' || c == ' ')
        .to_string()
}

/// Parse directory listing for file browser
/// Format from dockering: ls -la output (without header, via tail -n +2)
/// Example line: drwxr-xr-x  2 user group  4096 Jan  1 12:00 dirname
pub fn parse_directory_listing(output: &str, _path: &str) -> Vec<crate::models::FileEntry> {
    let mut entries = vec![crate::models::FileEntry::parent()];

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        // First char indicates type: d=directory, -=file, l=link
        let perms = parts[0];
        let is_dir = perms.starts_with('d');

        // Filename is everything after the 8th field (to handle spaces in names)
        let name = parts[8..].join(" ");

        // Skip . and ..
        if name == "." || name == ".." {
            continue;
        }

        entries.push(crate::models::FileEntry::new(name, is_dir));
    }

    // Sort: directories first, then files, both alphabetically
    entries[1..].sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    entries
}
