use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::PortMapping;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
}

impl EnvVar {
    pub fn new(key: String, value: String) -> Self {
        let is_secret = Self::detect_secret(&key);
        Self { key, value, is_secret }
    }

    fn detect_secret(key: &str) -> bool {
        let key_upper = key.to_uppercase();
        key_upper.contains("PASSWORD")
            || key_upper.contains("SECRET")
            || key_upper.contains("TOKEN")
            || key_upper.contains("KEY")
            || key_upper.contains("CREDENTIAL")
            || key_upper.contains("PRIVATE")
    }

    #[allow(dead_code)]
    pub fn display_value(&self) -> String {
        if self.is_secret {
            "••••••••••••".to_string()
        } else {
            self.value.clone()
        }
    }

    #[allow(dead_code)]
    pub fn display_value_truncated(&self, max_len: usize) -> String {
        let value = self.display_value();
        if value.len() > max_len {
            format!("{}...", &value[..max_len - 3])
        } else {
            value
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    #[serde(default)]
    pub read_only: bool,
}

impl VolumeMount {
    #[allow(dead_code)]
    pub fn display(&self) -> String {
        let ro = if self.read_only { ":ro" } else { "" };
        format!("{} -> {}{}", self.host_path, self.container_path, ro)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentScript {
    pub path: String,
    pub client_name: String,
    pub container_name: String,
    pub repo: String,
    pub env_vars: Vec<EnvVar>,
    pub volumes: Vec<VolumeMount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub ports: Vec<PortMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_policy: Option<String>,
    pub raw_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
}

impl DeploymentScript {
    pub fn new(path: String, client_name: String) -> Self {
        Self {
            path,
            client_name,
            container_name: String::new(),
            repo: String::new(),
            env_vars: Vec::new(),
            volumes: Vec::new(),
            network: None,
            ports: Vec::new(),
            restart_policy: None,
            raw_content: String::new(),
            last_modified: None,
        }
    }

    #[allow(dead_code)]
    pub fn add_env_var(&mut self, key: String, value: String) {
        // Check if key already exists and update it
        if let Some(existing) = self.env_vars.iter_mut().find(|e| e.key == key) {
            existing.value = value;
        } else {
            self.env_vars.push(EnvVar::new(key, value));
        }
    }

    #[allow(dead_code)]
    pub fn remove_env_var(&mut self, key: &str) {
        self.env_vars.retain(|e| e.key != key);
    }

    #[allow(dead_code)]
    pub fn get_env_var(&self, key: &str) -> Option<&EnvVar> {
        self.env_vars.iter().find(|e| e.key == key)
    }
}

/// Container resource statistics
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerStats {
    pub cpu_percent: String,
    pub memory_usage: String,
    pub memory_limit: String,
    pub memory_percent: String,
    pub net_io: String,
    pub block_io: String,
    pub pids: String,
}

/// Process information from docker top
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu: String,
    pub mem: String,
    pub command: String,
}

/// Container inspection summary
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
    pub started: String,
    pub ip_address: String,
    pub networks: Vec<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub restart_policy: String,
    pub health_status: Option<String>,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_script: bool,
}

impl FileEntry {
    pub fn new(name: String, is_dir: bool) -> Self {
        let is_script = !is_dir && (name.ends_with(".sh") || name.starts_with("start"));
        Self { name, is_dir, is_script }
    }

    pub fn parent() -> Self {
        Self {
            name: "..".to_string(),
            is_dir: true,
            is_script: false,
        }
    }
}

/// Represents a project folder on the remote server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub scripts: Vec<DeploymentScript>,
}

impl Project {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            scripts: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn find_script_for_container(&self, container_name: &str) -> Option<&DeploymentScript> {
        self.scripts.iter().find(|s| s.container_name == container_name)
    }

    #[allow(dead_code)]
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }
}
