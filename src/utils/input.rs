use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::models::{AppMode, HostField};

/// Handle keyboard input based on current app mode
pub fn handle_input(app: &mut App) -> Result<()> {
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
        KeyCode::Char('d') => app.start_delete_host(),

        // Filters
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Char('t') => app.start_tag_filter(),
        KeyCode::Esc => {
            app.clear_search();
            app.apply_tag_filter(vec![]);
        }

        // Sort
        KeyCode::Char('s') => app.cycle_sort(),

        // Help
        KeyCode::Char('?') => app.show_help(),

        // Quit
        KeyCode::Char('q') => app.quit(),

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
