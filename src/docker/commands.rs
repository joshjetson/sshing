/// Build docker commands for remote execution

pub fn docker_ps_command(all: bool) -> String {
    let all_flag = if all { "-a " } else { "" };
    format!(
        "docker ps {}--format '{{{{.ID}}}}|{{{{.Names}}}}|{{{{.Image}}}}|{{{{.Status}}}}|{{{{.Ports}}}}'",
        all_flag
    )
}

pub fn docker_pull_command(image: &str) -> String {
    format!("docker pull {}", image)
}

pub fn docker_start_command(container: &str) -> String {
    format!("docker start {}", container)
}

pub fn docker_stop_command(container: &str) -> String {
    format!("docker stop {}", container)
}

pub fn docker_restart_command(container: &str) -> String {
    format!("docker restart {}", container)
}

pub fn docker_rm_command(container: &str) -> String {
    format!("docker rm {}", container)
}

pub fn docker_rm_with_volumes_command(container: &str) -> String {
    format!("docker rm -v {}", container)
}

pub fn docker_rmi_command(image: &str) -> String {
    format!("docker rmi {}", image)
}

pub fn docker_logs_command(container: &str, tail: Option<usize>, follow: bool) -> String {
    let mut cmd = "docker logs".to_string();

    if let Some(n) = tail {
        cmd.push_str(&format!(" --tail {}", n));
    }

    if follow {
        cmd.push_str(" -f");
    }

    // Docker logs outputs to stderr, so redirect to stdout for capture
    cmd.push_str(&format!(" {} 2>&1", container));
    cmd
}

pub fn docker_exec_env_command(container: &str) -> String {
    format!("docker exec {} env", container)
}

pub fn docker_stats_command(container: &str) -> String {
    // Match dockering's custom format exactly
    format!("docker stats --no-stream --format '{{{{.CPUPerc}}}}|{{{{.MemUsage}}}}|{{{{.MemPerc}}}}|{{{{.NetIO}}}}|{{{{.BlockIO}}}}|{{{{.PIDs}}}}' {}", container)
}

pub fn docker_top_command(container: &str) -> String {
    // Match dockering's format: -o pid,user,%cpu,%mem,comm
    format!("docker top {} -o pid,user,%cpu,%mem,comm", container)
}

pub fn docker_inspect_command(container: &str) -> String {
    format!("docker inspect {}", container)
}

/// List directory contents for file browser
/// Match dockering's format: ls -la with tail to skip header
pub fn list_directory_command(path: &str) -> String {
    format!("ls -la {} 2>/dev/null | tail -n +2", path)
}
