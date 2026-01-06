use crate::models::{DeploymentScript, Project};
use super::script_parser::parse_script;

/// List all project directories in the clients path
pub fn list_projects_command(clients_path: &str) -> String {
    format!(
        "find {} -maxdepth 1 -mindepth 1 -type d -exec basename {{}} \\; 2>/dev/null | sort",
        expand_path(clients_path)
    )
}

/// Find all docker-related shell scripts in a project directory
pub fn find_scripts_command(project_path: &str) -> String {
    format!(
        r#"find {} -type f \( -name "start*.sh" -o -name "deploy*.sh" -o -name "run*.sh" -o -name "docker*.sh" \) \
        ! -path "*/node_modules/*" \
        ! -path "*/.git/*" \
        ! -path "*/vendor/*" \
        2>/dev/null"#,
        project_path
    )
}

/// Read a script file content
pub fn read_script_command(script_path: &str) -> String {
    format!("cat {}", script_path)
}

/// Parse project listing output into Project structs
pub fn parse_project_listing(output: &str, clients_path: &str) -> Vec<Project> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|name| {
            let name = name.trim().to_string();
            let path = format!("{}/{}", expand_path(clients_path), name);
            Project::new(name, path)
        })
        .collect()
}

/// Parse script paths output and create script entries
pub fn parse_script_paths(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

/// Create a DeploymentScript from path and content
pub fn create_script_from_content(
    path: &str,
    content: &str,
    project_name: &str,
) -> Option<DeploymentScript> {
    // Only create if it looks like a docker script
    if !content.contains("docker") {
        return None;
    }

    let script = parse_script(content, path, project_name);

    // Only return if we found a container name
    if script.container_name.is_empty() {
        return None;
    }

    Some(script)
}

/// Write script content to file on remote server
pub fn write_script_command(script_path: &str, content: &str) -> String {
    let escaped = content.replace('\'', "'\\''");
    format!(
        "cat > '{}' << 'DOCKERING_SCRIPT_EOF'\n{}\nDOCKERING_SCRIPT_EOF && chmod +x '{}'",
        script_path, escaped, script_path
    )
}

/// Run a deployment script
pub fn run_script_command(script_path: &str) -> String {
    format!("cd $(dirname '{}') && bash '{}'", script_path, script_path)
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        path.replacen("~", "$HOME", 1)
    } else {
        path.to_string()
    }
}
