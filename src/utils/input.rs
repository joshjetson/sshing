use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::models::{AppMode, HostField, ScriptSection};

/// Handle keyboard input based on current app mode (with timeout for non-blocking)
pub fn handle_input(app: &mut App) -> Result<()> {
    // Use poll with timeout so we don't block - allows SSH commands to be processed
    if !event::poll(std::time::Duration::from_millis(100))? {
        return Ok(());
    }

    if let Event::Key(key) = event::read()? {
        // Clear messages on any key press
        app.clear_messages();

        match &app.mode {
            AppMode::Table => handle_table_input(app, key)?,
            AppMode::EditHost { .. } => handle_edit_input(app, key)?,
            AppMode::Search { .. } => handle_search_input(app, key)?,
            AppMode::TagFilter { .. } => handle_tag_filter_input(app, key)?,
            AppMode::SelectKeys { .. } => handle_key_selection_input(app, key)?,
            AppMode::EditTags { .. } => handle_tag_edit_input(app, key)?,
            AppMode::Help => app.return_to_table(),
            AppMode::ConfirmDelete { host_index } => {
                let index = *host_index;
                handle_delete_confirm_input(app, key, index)?;
            }
            AppMode::SelectSshFlags { .. } => handle_ssh_flags_selection_input(app, key)?,
            AppMode::SelectShell { .. } => handle_shell_selection_input(app, key)?,
            AppMode::Rsync { .. } => handle_rsync_input(app, key)?,
            AppMode::RsyncFileBrowser { .. } => handle_rsync_file_browser_input(app, key)?,

            // Docker modes
            AppMode::ContainerList { .. } => handle_container_list_input(app, key)?,
            AppMode::ConfirmDockerAction { .. } => handle_docker_confirm_input(app, key)?,
            AppMode::LogsViewer { .. } => handle_logs_input(app, key)?,
            AppMode::StatsViewer { .. } => handle_stats_input(app, key)?,
            AppMode::ProcessViewer { .. } => handle_process_input(app, key)?,
            AppMode::InspectViewer { .. } => handle_inspect_input(app, key)?,
            AppMode::EnvInspector { .. } => handle_env_inspector_input(app, key)?,
            AppMode::ScriptViewer { .. } => handle_script_viewer_input(app, key)?,
            AppMode::ScriptEdit { .. } => handle_script_edit_input(app, key)?,
            AppMode::EnvVarEditor { .. } => handle_env_var_editor_input(app, key)?,
            AppMode::FileBrowser { .. } => handle_file_browser_input(app, key)?,
        }
    }

    Ok(())
}

/// Handle input in table view
fn handle_table_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.page_down(10)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.page_up(10),

        // Actions
        KeyCode::Char(' ') | KeyCode::Enter => {
            app.connect_to_selected()?;
        }
        KeyCode::Char('r') => {
            if app.rsync_available {
                app.start_rsync();
            } else {
                app.set_error("rsync is not installed on this system");
            }
        }
        KeyCode::Char('n') => app.start_new_host(),
        KeyCode::Char('e') => app.start_edit_host(),
        KeyCode::Char('D') => app.start_delete_host(),
        KeyCode::Char('d') => app.start_docker_mode(),

        // Filters
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Char('t') => app.start_tag_filter(),

        // Sort
        KeyCode::Char('s') => app.cycle_sort(),

        // Help
        KeyCode::Char('?') => app.show_help(),

        // Quit
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),

        _ => {}
    }

    Ok(())
}

/// Handle input in edit host view
fn handle_edit_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::EditHost {
        host_index,
        editing_host,
        focused_field,
        field_buffer,
        editing_mode,
    } = &mut app.mode
    {
        let original_index = *host_index;
        let current_field = *focused_field;

        if *editing_mode {
            // EDITING MODE: typing into the current field
            match key.code {
                // Exit editing mode and save
                KeyCode::Enter => {
                    apply_field_buffer(editing_host, &current_field, field_buffer);

                    // Special fields open their editors
                    match current_field {
                        HostField::IdentityFiles => {
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_key_selection(idx, host, current_field);
                        }
                        HostField::SshFlags => {
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_ssh_flags_selection(idx, host, current_field);
                        }
                        HostField::Shell => {
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_shell_selection(idx, host, current_field);
                        }
                        HostField::Tags => {
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_tag_editing(idx, host, current_field);
                        }
                        _ => {
                            // Exit editing mode
                            *editing_mode = false;
                        }
                    }
                }

                // Exit editing mode without saving
                KeyCode::Esc => {
                    // Restore field buffer from host
                    *field_buffer = get_field_value_for_editing(editing_host, &current_field);
                    *editing_mode = false;
                }

                // Tab: save, exit editing, move to next field
                KeyCode::Tab => {
                    apply_field_buffer(editing_host, &current_field, field_buffer);
                    *editing_mode = false;
                    let next_field = focused_field.next();
                    *focused_field = next_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &next_field);
                }

                // Shift+Tab: save, exit editing, move to previous field
                KeyCode::BackTab => {
                    apply_field_buffer(editing_host, &current_field, field_buffer);
                    *editing_mode = false;
                    let prev_field = focused_field.previous();
                    *focused_field = prev_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &prev_field);
                }

                // Type characters
                KeyCode::Char(c) => {
                    field_buffer.push(c);
                }

                // Backspace
                KeyCode::Backspace => {
                    field_buffer.pop();
                }

                _ => {}
            }
        } else {
            // NAVIGATION MODE: moving between fields
            match key.code {
                // Navigate fields
                KeyCode::Char('j') | KeyCode::Down => {
                    let next_field = focused_field.next();
                    *focused_field = next_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &next_field);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let prev_field = focused_field.previous();
                    *focused_field = prev_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &prev_field);
                }

                // Tab navigation
                KeyCode::Tab => {
                    let next_field = focused_field.next();
                    *focused_field = next_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &next_field);
                }
                KeyCode::BackTab => {
                    let prev_field = focused_field.previous();
                    *focused_field = prev_field;
                    *field_buffer = get_field_value_for_editing(editing_host, &prev_field);
                }

                // Enter: activate editing mode or open special editors
                KeyCode::Enter => {
                    match current_field {
                        HostField::IdentityFiles => {
                            // Apply buffer before opening special editor
                            apply_field_buffer(editing_host, &current_field, field_buffer);
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_key_selection(idx, host, current_field);
                        }
                        HostField::SshFlags => {
                            // Apply buffer before opening special editor
                            apply_field_buffer(editing_host, &current_field, field_buffer);
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_ssh_flags_selection(idx, host, current_field);
                        }
                        HostField::Shell => {
                            // Apply buffer before opening special editor
                            apply_field_buffer(editing_host, &current_field, field_buffer);
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_shell_selection(idx, host, current_field);
                        }
                        HostField::Tags => {
                            // Apply buffer before opening special editor
                            apply_field_buffer(editing_host, &current_field, field_buffer);
                            let host = editing_host.clone();
                            let idx = original_index;
                            app.start_tag_editing(idx, host, current_field);
                        }
                        _ => {
                            // Enter editing mode for regular fields
                            *editing_mode = true;
                        }
                    }
                }

                // Save entire form
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    apply_field_buffer(editing_host, &current_field, field_buffer);
                    let host_to_save = editing_host.clone();
                    app.save_edited_host(host_to_save, original_index)?;
                }

                // Cancel and return to table
                KeyCode::Esc => {
                    app.return_to_table();
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/// Handle input in search mode
fn handle_search_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::Search { query } = &mut app.mode {
        match key.code {
            KeyCode::Enter => {
                let final_query = query.clone();
                app.apply_search(final_query);
            }
            KeyCode::Esc => {
                app.return_to_table();
            }
            KeyCode::Char(c) => {
                query.push(c);
            }
            KeyCode::Backspace => {
                query.pop();
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle input in tag filter mode
fn handle_tag_filter_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::TagFilter { selected_tags } = &mut app.mode {
        match key.code {
            KeyCode::Enter => {
                let final_tags = selected_tags.clone();
                app.apply_tag_filter(final_tags);
            }
            KeyCode::Esc => {
                app.return_to_table();
            }
            // TODO: Implement tag selection with arrow keys and space
            _ => {}
        }
    }

    Ok(())
}

/// Handle input in delete confirmation dialog
fn handle_delete_confirm_input(app: &mut App, key: KeyEvent, host_index: usize) -> Result<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.delete_host(host_index)?;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.return_to_table();
        }
        _ => {}
    }

    Ok(())
}

/// Handle input in SSH key selection mode
fn handle_key_selection_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::SelectKeys {
        host_index,
        editing_host,
        available_keys,
        selected_key_index,
        return_field,
    } = &mut app.mode
    {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if *selected_key_index > 0 {
                    *selected_key_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if *selected_key_index < available_keys.len().saturating_sub(1) {
                    *selected_key_index += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Toggle key selection
                if *selected_key_index < available_keys.len() {
                    let key = available_keys[*selected_key_index].clone();
                    let identity_files = editing_host.identity_file.get_or_insert_with(Vec::new);

                    if identity_files.contains(&key) {
                        identity_files.retain(|k| k != &key);
                    } else {
                        identity_files.push(key);
                    }
                }
            }
            KeyCode::Esc => {
                // Return to edit mode
                let idx = *host_index;
                let host = editing_host.clone();
                let field = *return_field;
                app.return_to_edit(idx, host, field);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle input in tag editing mode
fn handle_tag_edit_input(app: &mut App, key: KeyEvent) -> Result<()> {
    // Get all available tags from the app before borrowing mode
    let all_available_tags = app.all_tags();

    // Variable to store new tag to add to global pool
    let mut new_global_tag: Option<String> = None;

    if let AppMode::EditTags {
        host_index,
        editing_host,
        tag_input,
        selected_tag_index,
        return_field,
        input_mode,
    } = &mut app.mode
    {
        if *input_mode {
            // INPUT MODE: creating a new tag for the global pool
            match key.code {
                KeyCode::Enter => {
                    // Add new tag to the global pool (NOT to this host)
                    if !tag_input.is_empty() {
                        let new_tag = tag_input.trim().to_string();
                        if !all_available_tags.contains(&new_tag) {
                            new_global_tag = Some(new_tag);
                        }
                        tag_input.clear();
                    }
                    *input_mode = false; // Return to selection mode
                }
                KeyCode::Esc => {
                    // Cancel input and return to selection mode
                    tag_input.clear();
                    *input_mode = false;
                }
                KeyCode::Char(c) => {
                    // Type into input field
                    tag_input.push(c);
                }
                KeyCode::Backspace => {
                    tag_input.pop();
                }
                _ => {}
            }
        } else {
            // SELECTION MODE: toggling tags on/off for this host
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if !all_available_tags.is_empty() && *selected_tag_index > 0 {
                        *selected_tag_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *selected_tag_index < all_available_tags.len().saturating_sub(1) {
                        *selected_tag_index += 1;
                    }
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    // Toggle tag on/off for this host
                    if *selected_tag_index < all_available_tags.len() {
                        let tag = all_available_tags[*selected_tag_index].clone();
                        if editing_host.tags.contains(&tag) {
                            // Remove tag from this host
                            editing_host.tags.retain(|t| t != &tag);
                        } else {
                            // Add tag to this host
                            editing_host.tags.push(tag);
                        }
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('n') | KeyCode::Char('i') => {
                    // Enter input mode to create a new tag
                    *input_mode = true;
                }
                KeyCode::Esc => {
                    // Return to edit mode
                    let idx = *host_index;
                    let host = editing_host.clone();
                    let field = *return_field;
                    app.return_to_edit(idx, host, field);
                }
                _ => {}
            }
        }
    }

    // Add new tag to global pool if one was created
    if let Some(tag) = new_global_tag {
        app.add_global_tag(tag)?;
    }

    Ok(())
}

/// Get the current value of a field for editing
fn get_field_value_for_editing(host: &crate::models::Host, field: &HostField) -> String {
    match field {
        HostField::HostAlias => host.host.clone(),
        HostField::Hostname => host.hostname.clone(),
        HostField::User => host.user.clone().unwrap_or_default(),
        HostField::Port => host.port.map(|p| p.to_string()).unwrap_or_default(),
        HostField::ProxyJump => host.proxy_jump.clone().unwrap_or_default(),
        HostField::Note => host.note.clone().unwrap_or_default(),
        HostField::IdentityFiles | HostField::SshFlags | HostField::Shell | HostField::Tags => {
            String::new() // These use special editors
        }
    }
}

/// Apply the field buffer to the host
fn apply_field_buffer(host: &mut crate::models::Host, field: &HostField, buffer: &str) {
    match field {
        HostField::HostAlias => {
            host.host = buffer.to_string();
        }
        HostField::Hostname => {
            host.hostname = buffer.to_string();
        }
        HostField::User => {
            host.user = if buffer.is_empty() {
                None
            } else {
                Some(buffer.to_string())
            };
        }
        HostField::Port => {
            host.port = buffer.parse().ok();
        }
        HostField::ProxyJump => {
            host.proxy_jump = if buffer.is_empty() {
                None
            } else {
                Some(buffer.to_string())
            };
        }
        HostField::Note => {
            host.note = if buffer.is_empty() {
                None
            } else {
                Some(buffer.to_string())
            };
        }
        HostField::IdentityFiles | HostField::SshFlags | HostField::Shell | HostField::Tags => {
            // These are handled by special editors
        }
    }
}

/// Handle input in SSH flags selection mode
fn handle_ssh_flags_selection_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crate::models::get_ssh_flag_options;

    let available_flags = get_ssh_flag_options();

    if let AppMode::SelectSshFlags {
        host_index,
        editing_host,
        selected_flag_index,
        return_field,
    } = &mut app.mode
    {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if *selected_flag_index > 0 {
                    *selected_flag_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if *selected_flag_index < available_flags.len().saturating_sub(1) {
                    *selected_flag_index += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Toggle flag selection
                if *selected_flag_index < available_flags.len() {
                    let flag = available_flags[*selected_flag_index].flag.to_string();

                    if editing_host.ssh_flags.contains(&flag) {
                        editing_host.ssh_flags.retain(|f| f != &flag);
                    } else {
                        editing_host.ssh_flags.push(flag);
                    }
                }
            }
            KeyCode::Esc => {
                // Return to edit mode
                let idx = *host_index;
                let host = editing_host.clone();
                let field = *return_field;
                app.return_to_edit(idx, host, field);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle input in shell selection mode
fn handle_shell_selection_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crate::models::get_shell_options;

    let available_shells = get_shell_options();

    if let AppMode::SelectShell {
        host_index,
        editing_host,
        selected_shell_index,
        return_field,
    } = &mut app.mode
    {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if *selected_shell_index > 0 {
                    *selected_shell_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if *selected_shell_index < available_shells.len().saturating_sub(1) {
                    *selected_shell_index += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Select shell (single selection)
                if *selected_shell_index < available_shells.len() {
                    let shell = available_shells[*selected_shell_index].name.to_string();

                    // Toggle - if already selected, deselect it
                    if editing_host.shell.as_ref() == Some(&shell) {
                        editing_host.shell = None;
                    } else {
                        editing_host.shell = Some(shell);
                    }
                }
            }
            KeyCode::Esc => {
                // Return to edit mode
                let idx = *host_index;
                let host = editing_host.clone();
                let field = *return_field;
                app.return_to_edit(idx, host, field);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle input in rsync mode
/// Autocomplete a path by finding the longest common prefix of matching entries
fn autocomplete_path(partial_path: &str) -> Option<String> {
    use std::path::Path;
    use std::fs;

    // Handle empty path
    if partial_path.is_empty() {
        return None;
    }

    let path = Path::new(partial_path);
    let (dir, prefix) = if partial_path.ends_with('/') {
        (path.to_path_buf(), String::new())
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => (p.to_path_buf(), path.file_name()?.to_string_lossy().to_string()),
            _ => (Path::new(".").to_path_buf(), partial_path.to_string()),
        }
    };

    // List entries in the directory
    let entries: Vec<String> = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return None,
    };

    if entries.is_empty() {
        return None;
    }

    // Find longest common prefix
    let mut common = entries[0].clone();
    for entry in &entries[1..] {
        while !entry.starts_with(&common) {
            common.pop();
        }
    }

    // Return the completed path
    if partial_path.ends_with('/') {
        Some(format!("{}{}", partial_path, common))
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => {
                let parent_str = p.display().to_string();
                if parent_str == "/" {
                    Some(format!("/{}", common))
                } else {
                    Some(format!("{}/{}", parent_str, common))
                }
            }
            _ => Some(common),
        }
    }
}

/// Autocomplete a remote path using SSH to list remote directory
fn autocomplete_path_remote(host: &crate::models::Host, partial_path: &str) -> Option<String> {
    use std::process::Command;

    // Handle empty path
    if partial_path.is_empty() {
        return None;
    }

    // Split path into directory and prefix
    let (dir, prefix) = if partial_path.ends_with('/') {
        (partial_path.to_string(), String::new())
    } else {
        match partial_path.rfind('/') {
            Some(idx) => {
                let dir = &partial_path[..=idx];
                let prefix = &partial_path[idx + 1..];
                (dir.to_string(), prefix.to_string())
            }
            None => {
                // No directory separator, list current directory
                ("./".to_string(), partial_path.to_string())
            }
        }
    };

    // Build SSH command
    let mut cmd = Command::new("ssh");

    // Add SSH options
    if let Some(ref user) = host.user {
        cmd.arg(format!("{}@{}", user, host.hostname));
    } else {
        cmd.arg(&host.hostname);
    }

    if let Some(port) = host.port {
        cmd.arg("-p").arg(port.to_string());
    }

    if let Some(ref identity_files) = host.identity_file {
        for file in identity_files {
            cmd.arg("-i").arg(file);
        }
    }

    cmd.arg("-o").arg("StrictHostKeyChecking=no");

    // Execute `ls -1` on the remote directory
    cmd.arg(format!("ls -1 '{}'", dir));

    // Execute SSH and capture output
    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                return None;
            }

            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse ls output - each line is a filename
            let entries: Vec<String> = output_str
                .lines()
                .filter_map(|line| {
                    let name = line.trim();
                    if !name.is_empty() && name.starts_with(&prefix) {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if entries.is_empty() {
                return None;
            }

            // Find longest common prefix
            let mut common = entries[0].clone();
            for entry in &entries[1..] {
                while !entry.starts_with(&common) {
                    common.pop();
                }
            }

            // Return the completed path
            if partial_path.ends_with('/') {
                Some(format!("{}{}", partial_path, common))
            } else {
                Some(format!("{}{}", dir, common))
            }
        }
        Err(_) => {
            // If SSH fails, just return None and let user type manually
            None
        }
    }
}

/// Handle input in rsync mode
fn handle_rsync_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::Rsync {
        editing_mode,
        focused_field,
        source_path,
        dest_path,
        sync_to_host,
        editing_host,
        compress,
        ..
    } = &mut app.mode
    {
        if *editing_mode {
            // In editing mode - typing into a field
            match key.code {
                KeyCode::Char(c) => {
                    match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => source_path.push(c),
                        crate::models::app_state::RsyncField::DestPath => dest_path.push(c),
                    }
                }
                KeyCode::Backspace => {
                    match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => {
                            source_path.pop();
                        }
                        crate::models::app_state::RsyncField::DestPath => {
                            dest_path.pop();
                        }
                    }
                }
                KeyCode::Tab => {
                    // Autocomplete in path fields (dynamically choose local or remote)
                    match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => {
                            // Source is remote if sync_to_host is false, local if true
                            if *sync_to_host {
                                // Source is local
                                if let Some(completed) = autocomplete_path(source_path) {
                                    *source_path = completed;
                                }
                            } else {
                                // Source is remote
                                if let Some(completed) = autocomplete_path_remote(editing_host, source_path) {
                                    *source_path = completed;
                                }
                            }
                        }
                        crate::models::app_state::RsyncField::DestPath => {
                            // Dest is remote if sync_to_host is true, local if false
                            if *sync_to_host {
                                // Dest is remote
                                if let Some(completed) = autocomplete_path_remote(editing_host, dest_path) {
                                    *dest_path = completed;
                                }
                            } else {
                                // Dest is local
                                if let Some(completed) = autocomplete_path(dest_path) {
                                    *dest_path = completed;
                                }
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    // Move to next field and exit edit mode
                    *editing_mode = false;
                    match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => {
                            *focused_field = crate::models::app_state::RsyncField::DestPath;
                        }
                        crate::models::app_state::RsyncField::DestPath => {
                            *focused_field = crate::models::app_state::RsyncField::SourcePath;
                        }
                    }
                }
                KeyCode::Esc => {
                    // Exit edit mode without saving
                    *editing_mode = false;
                }
                _ => {}
            }
        } else {
            // Not in editing mode - navigate between fields or toggle settings
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    *focused_field = match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => crate::models::app_state::RsyncField::DestPath,
                        crate::models::app_state::RsyncField::DestPath => crate::models::app_state::RsyncField::SourcePath,
                    };
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    *focused_field = match focused_field {
                        crate::models::app_state::RsyncField::SourcePath => crate::models::app_state::RsyncField::DestPath,
                        crate::models::app_state::RsyncField::DestPath => crate::models::app_state::RsyncField::SourcePath,
                    };
                }
                KeyCode::Char('i') | KeyCode::Enter => {
                    // Enter edit mode
                    *editing_mode = true;
                }
                KeyCode::Char('r') => {
                    // Toggle direction
                    *sync_to_host = !*sync_to_host;
                }
                KeyCode::Char('z') => {
                    // Toggle compression flag
                    *compress = !*compress;
                }
                KeyCode::Char('b') => {
                    // Browse for file/directory
                    app.start_rsync_browse();
                }
                KeyCode::Char(' ') => {
                    // Execute rsync - check if paths are filled
                    if source_path.is_empty() || dest_path.is_empty() {
                        app.set_error("Both source and destination paths required");
                    } else {
                        // Set pending rsync (main loop will handle execution)
                        app.pending_rsync = Some((
                            editing_host.clone(),
                            source_path.clone(),
                            dest_path.clone(),
                            *sync_to_host,
                            *compress,
                        ));
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    // Return to table
                    app.return_to_table();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Handle input in rsync file browser mode
fn handle_rsync_file_browser_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::RsyncFileBrowser {
        current_path,
        entries,
        selected_index,
        loading,
        ..
    } = &mut app.mode
    {
        if *loading {
            // Only allow escape while loading
            if key.code == KeyCode::Esc {
                app.rsync_cancel_browse();
            }
            return Ok(());
        }

        let current_path = current_path.clone();
        let entries_len = entries.len();

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if *selected_index < entries_len.saturating_sub(1) {
                    *selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                *selected_index = 0;
            }
            KeyCode::Char('G') => {
                *selected_index = entries_len.saturating_sub(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *selected_index = (*selected_index + 10).min(entries_len.saturating_sub(1));
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *selected_index = selected_index.saturating_sub(10);
            }
            KeyCode::Enter => {
                // Navigate into directory or select file
                if *selected_index < entries.len() {
                    let entry = &entries[*selected_index];
                    if entry.is_dir {
                        let new_path = if entry.name == ".." {
                            // Go up one directory
                            if let Some(parent) = std::path::Path::new(&current_path).parent() {
                                parent.to_string_lossy().to_string()
                            } else {
                                "/".to_string()
                            }
                        } else {
                            // Enter subdirectory
                            if current_path.ends_with('/') {
                                format!("{}{}", current_path, entry.name)
                            } else {
                                format!("{}/{}", current_path, entry.name)
                            }
                        };
                        app.rsync_navigate_to(new_path);
                    } else {
                        // Select the file
                        app.rsync_select_current_path();
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Select current directory (useful for selecting folders as source/dest)
                app.rsync_select_current_path();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.rsync_cancel_browse();
            }
            _ => {}
        }
    }
    Ok(())
}

// ==================== Docker Input Handlers ====================

/// Handle input in container list view
fn handle_container_list_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.docker_select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.docker_select_previous(),
        KeyCode::Char('g') => app.docker_select_first(),
        KeyCode::Char('G') => app.docker_select_last(),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => app.docker_page_down(),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.docker_page_up(),

        // Container operations
        KeyCode::Char('p') => app.docker_pull(),
        KeyCode::Char('r') => app.docker_restart(),
        KeyCode::Char('s') => app.docker_stop(),
        KeyCode::Char('S') => app.docker_start(),
        KeyCode::Char('d') => app.docker_remove(false, false),
        KeyCode::Char('X') => app.docker_remove(true, true),

        // View operations
        KeyCode::Char('l') => app.view_logs(),
        KeyCode::Char('E') => app.inspect_env(),
        KeyCode::Char('D') => app.view_stats(),
        KeyCode::Char('T') => app.view_processes(),
        KeyCode::Char('I') => app.view_inspect(),

        // Script operations
        KeyCode::Char('n') => app.create_script(),  // NEW: Create new script
        KeyCode::Char('e') => app.edit_script(),
        KeyCode::Char('v') => app.view_script(),
        KeyCode::Char('x') => app.run_script(),
        KeyCode::Char('b') => app.browse_for_script(),

        // Refresh
        KeyCode::Char('R') => app.refresh_containers(),

        // Back
        KeyCode::Esc | KeyCode::Char('q') => app.docker_go_back(),

        _ => {}
    }
    Ok(())
}

/// Handle input in docker confirmation dialog
fn handle_docker_confirm_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::ConfirmDockerAction { action, return_mode } = &app.mode {
        let action = action.clone();
        let return_mode = *return_mode.clone();

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.execute_docker_action(action);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.cancel_docker_action(return_mode);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in logs viewer
fn handle_logs_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::LogsViewer { host_index, scroll_offset, log_buffer, follow_mode, .. } = &mut app.mode {
        let host_index = *host_index;
        match key.code {
            KeyCode::Char('m') => {
                // Load more logs
                app.load_more_logs();
            }
            KeyCode::Char('f') => {
                // Toggle follow mode
                *follow_mode = !*follow_mode;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                *follow_mode = false;
                if *scroll_offset < log_buffer.len().saturating_sub(1) {
                    *scroll_offset += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *follow_mode = false;
                *scroll_offset = scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                *follow_mode = false;
                *scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                // Go to bottom (enable follow)
                *follow_mode = true;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *follow_mode = false;
                *scroll_offset = (*scroll_offset + 10).min(log_buffer.len().saturating_sub(1));
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *follow_mode = false;
                *scroll_offset = scroll_offset.saturating_sub(10);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in stats viewer
fn handle_stats_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::StatsViewer { host_index, container_index, .. } = &app.mode {
        let host_index = *host_index;
        let container_index = *container_index;
        match key.code {
            KeyCode::Char('r') => {
                // Refresh stats - go back and re-enter
                app.mode = AppMode::ContainerList { host_index };
                app.docker_selected_index = container_index;
                app.view_stats();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in process viewer
fn handle_process_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::ProcessViewer { host_index, container_index, processes, selected_index, .. } = &mut app.mode {
        let host_index = *host_index;
        let container_index = *container_index;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if *selected_index < processes.len().saturating_sub(1) {
                    *selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Char('g') => *selected_index = 0,
            KeyCode::Char('G') => {
                *selected_index = processes.len().saturating_sub(1);
            }
            KeyCode::Char('r') => {
                // Refresh processes - go back and re-enter
                app.mode = AppMode::ContainerList { host_index };
                app.docker_selected_index = container_index;
                app.view_processes();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in inspect viewer
fn handle_inspect_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::InspectViewer { host_index, container_index, .. } = &app.mode {
        let host_index = *host_index;
        let container_index = *container_index;
        match key.code {
            KeyCode::Char('r') => {
                // Refresh inspect info - go back and re-enter
                app.mode = AppMode::ContainerList { host_index };
                app.docker_selected_index = container_index;
                app.view_inspect();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in environment inspector
fn handle_env_inspector_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::EnvInspector { host_index, container_vars, selected_index, scroll_offset, search_query, .. } = &mut app.mode {
        let host_index = *host_index;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if *selected_index < container_vars.len().saturating_sub(1) {
                    *selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = (*scroll_offset + 10).min(container_vars.len().saturating_sub(1));
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = scroll_offset.saturating_sub(10);
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                search_query.push(c);
            }
            KeyCode::Backspace => {
                search_query.pop();
            }
            KeyCode::Esc => {
                if !search_query.is_empty() {
                    search_query.clear();
                } else {
                    app.mode = AppMode::ContainerList { host_index };
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in script viewer
fn handle_script_viewer_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::ScriptViewer { host_index, scroll_offset, script_content, .. } = &mut app.mode {
        let host_index = *host_index;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if *scroll_offset < script_content.len().saturating_sub(1) {
                    *scroll_offset += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *scroll_offset = scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('g') => *scroll_offset = 0,
            KeyCode::Char('G') => {
                *scroll_offset = script_content.len().saturating_sub(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = (*scroll_offset + 10).min(script_content.len().saturating_sub(1));
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = scroll_offset.saturating_sub(10);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in script editor
fn handle_script_edit_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::ScriptEdit { host_index, focused_section, selected_index, editing_script, .. } = &app.mode {
        let host_index = *host_index;
        let focused_section = *focused_section;
        let selected_index = *selected_index;
        let env_vars_len = editing_script.env_vars.len();
        let volumes_len = editing_script.volumes.len();
        let ports_len = editing_script.ports.len();

        match key.code {
            // Navigation between sections
            KeyCode::Tab => {
                if let AppMode::ScriptEdit { focused_section, selected_index, .. } = &mut app.mode {
                    *focused_section = focused_section.next();
                    *selected_index = 0;
                }
            }
            KeyCode::BackTab => {
                if let AppMode::ScriptEdit { focused_section, selected_index, .. } = &mut app.mode {
                    *focused_section = focused_section.previous();
                    *selected_index = 0;
                }
            }

            // Navigation within section
            KeyCode::Char('j') | KeyCode::Down => {
                if let AppMode::ScriptEdit { focused_section, selected_index, editing_script, .. } = &mut app.mode {
                    let max = match focused_section {
                        ScriptSection::EnvVars => editing_script.env_vars.len(),
                        ScriptSection::Volumes => editing_script.volumes.len(),
                        ScriptSection::Ports => editing_script.ports.len(),
                        ScriptSection::Network => 1,
                    };
                    if *selected_index < max.saturating_sub(1) {
                        *selected_index += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let AppMode::ScriptEdit { selected_index, .. } = &mut app.mode {
                    *selected_index = selected_index.saturating_sub(1);
                }
            }

            // Add new item
            KeyCode::Char('a') => {
                match focused_section {
                    ScriptSection::EnvVars => app.start_add_env_var(),
                    _ => {} // TODO: Add volume/port editors
                }
            }

            // Edit existing item
            KeyCode::Enter => {
                match focused_section {
                    ScriptSection::EnvVars if env_vars_len > 0 => {
                        app.start_edit_env_var();
                    }
                    _ => {}
                }
            }

            // Delete item
            KeyCode::Char('d') => {
                match focused_section {
                    ScriptSection::EnvVars if env_vars_len > 0 => {
                        app.remove_env_var_from_current_script(selected_index);
                    }
                    ScriptSection::Volumes if volumes_len > 0 => {
                        if let AppMode::ScriptEdit { editing_script, selected_index, .. } = &mut app.mode {
                            if *selected_index < editing_script.volumes.len() {
                                editing_script.volumes.remove(*selected_index);
                                *selected_index = (*selected_index).min(editing_script.volumes.len().saturating_sub(1));
                            }
                        }
                    }
                    ScriptSection::Ports if ports_len > 0 => {
                        if let AppMode::ScriptEdit { editing_script, selected_index, .. } = &mut app.mode {
                            if *selected_index < editing_script.ports.len() {
                                editing_script.ports.remove(*selected_index);
                                *selected_index = (*selected_index).min(editing_script.ports.len().saturating_sub(1));
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Save script (Ctrl+S)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.save_current_script();
            }

            // Cancel and go back
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }

            _ => {}
        }
    }
    Ok(())
}

/// Handle input in environment variable editor
fn handle_env_var_editor_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::EnvVarEditor { key_buffer, value_buffer, editing_key, .. } = &mut app.mode {
        match key.code {
            KeyCode::Tab => {
                *editing_key = !*editing_key;
            }
            KeyCode::Enter => {
                // Save the env var
                app.save_env_var();
            }
            KeyCode::Esc => {
                // Cancel and return to script edit
                app.cancel_env_var_edit();
            }
            KeyCode::Char(c) => {
                if *editing_key {
                    // Key field: only allow alphanumeric and underscore, convert to uppercase
                    if c.is_alphanumeric() || c == '_' {
                        key_buffer.push(c.to_ascii_uppercase());
                    }
                } else {
                    // Value field: allow any character
                    value_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if *editing_key {
                    key_buffer.pop();
                } else {
                    value_buffer.pop();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle input in file browser
fn handle_file_browser_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let AppMode::FileBrowser { host_index, container_index, entries, selected_index, current_path, loading, .. } = &mut app.mode {
        let host_index = *host_index;
        let container_index = *container_index;

        if *loading {
            // Don't process input while loading
            match key.code {
                KeyCode::Esc => {
                    app.mode = AppMode::ContainerList { host_index };
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if *selected_index < entries.len().saturating_sub(1) {
                    *selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Char('g') => *selected_index = 0,
            KeyCode::Char('G') => {
                *selected_index = entries.len().saturating_sub(1);
            }
            KeyCode::Enter => {
                if *selected_index < entries.len() {
                    let entry = &entries[*selected_index];
                    if entry.is_dir {
                        // Navigate into directory
                        let new_path = if entry.name == ".." {
                            // Go up
                            let path = std::path::Path::new(&current_path);
                            path.parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| current_path.clone())
                        } else {
                            format!("{}/{}", current_path.trim_end_matches('/'), entry.name)
                        };

                        if let Some(host) = app.hosts.get(host_index).cloned() {
                            // Match dockering's ls -la format
                            let base_cmd = format!("ls -la {} 2>/dev/null | tail -n +2", new_path);
                            let cmd = if app.use_sudo {
                                format!("sudo -i {}", base_cmd)
                            } else {
                                base_cmd
                            };
                            app.pending_ssh_command = Some(crate::app::PendingSshCommand {
                                host,
                                command: cmd,
                                command_type: crate::app::SshCommandType::ListDirectory { path: new_path.clone() },
                            });
                            *current_path = new_path;
                            *loading = true;
                            *selected_index = 0;
                        }
                    } else if entry.is_script {
                        // Select this script for the container
                        let script_path = format!("{}/{}", current_path.trim_end_matches('/'), entry.name);
                        if let Some(host) = app.hosts.get(host_index).cloned() {
                            let base_cmd = format!("cat {}", script_path);
                            let cmd = if app.use_sudo {
                                format!("sudo -i {}", base_cmd)
                            } else {
                                base_cmd
                            };
                            app.pending_ssh_command = Some(crate::app::PendingSshCommand {
                                host,
                                command: cmd,
                                command_type: crate::app::SshCommandType::ReadScriptForContainer {
                                    script_path,
                                    container_index,
                                },
                            });
                        }
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::ContainerList { host_index };
            }
            _ => {}
        }
    }
    Ok(())
}
