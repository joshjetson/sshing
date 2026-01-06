use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Paused,
    Restarting,
    Exited(i32),
    Dead,
    Unknown(String),
}

impl ContainerStatus {
    pub fn from_docker_status(status: &str) -> Self {
        let status_lower = status.to_lowercase();
        if status_lower.starts_with("up") {
            ContainerStatus::Running
        } else if status_lower.starts_with("exited") {
            // Parse exit code if present: "Exited (0)"
            if let Some(code) = status_lower
                .split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .and_then(|s| s.parse().ok())
            {
                ContainerStatus::Exited(code)
            } else {
                ContainerStatus::Stopped
            }
        } else if status_lower.contains("paused") {
            ContainerStatus::Paused
        } else if status_lower.contains("restarting") {
            ContainerStatus::Restarting
        } else if status_lower.contains("dead") {
            ContainerStatus::Dead
        } else {
            ContainerStatus::Unknown(status.to_string())
        }
    }

    #[allow(dead_code)]
    pub fn display(&self) -> &str {
        match self {
            ContainerStatus::Running => "Up",
            ContainerStatus::Stopped => "Stopped",
            ContainerStatus::Paused => "Paused",
            ContainerStatus::Restarting => "Restarting",
            ContainerStatus::Exited(_) => "Exited",
            ContainerStatus::Dead => "Dead",
            ContainerStatus::Unknown(_) => "Unknown",
        }
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        matches!(self, ContainerStatus::Running)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

impl PortMapping {
    pub fn display(&self) -> String {
        format!("{}:{}", self.host_port, self.container_port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub ports: Vec<PortMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    pub server_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    pub networks: Vec<String>,
}

impl Container {
    pub fn has_script(&self) -> bool {
        self.script_path.is_some()
    }

    pub fn ports_display(&self) -> String {
        if self.ports.is_empty() {
            "-".to_string()
        } else {
            self.ports
                .iter()
                .map(|p| p.display())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    pub fn short_image(&self) -> String {
        // Extract just the image name from full path
        self.image
            .split('/')
            .last()
            .unwrap_or(&self.image)
            .split(':')
            .next()
            .unwrap_or(&self.image)
            .to_string()
    }

    #[allow(dead_code)]
    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.image.to_lowercase().contains(&query)
            || self.id.to_lowercase().starts_with(&query)
    }
}
