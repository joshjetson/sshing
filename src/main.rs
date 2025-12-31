mod app;
mod models;
mod ssh;
mod ui;
mod utils;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use app::App;
use models::AppMode;
use ssh::executor::connect_to_host;
use ui::{
    render_delete_confirmation, render_editor_view, render_help_view, render_key_selection_view,
    render_search_overlay, render_shell_selection_view, render_ssh_flags_selection_view,
    render_table_view, render_tag_edit_view, render_tag_filter_view,
};
use utils::handle_input;

fn main() -> Result<()> {
    // Initialize the application
    let mut app = App::new()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print any errors
    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Run the main application loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();

            match &app.mode {
                AppMode::Table => {
                    render_table_view(frame, app, area);
                }
                AppMode::EditHost {
                    editing_host,
                    focused_field,
                    field_buffer,
                    editing_mode,
                    ..
                } => {
                    render_editor_view(frame, editing_host, focused_field, field_buffer, *editing_mode, area);
                }
                AppMode::SelectKeys {
                    editing_host,
                    available_keys,
                    selected_key_index,
                    ..
                } => {
                    let selected_keys = editing_host
                        .identity_file
                        .as_deref()
                        .unwrap_or(&[]);
                    render_key_selection_view(frame, available_keys, selected_keys, *selected_key_index, area);
                }
                AppMode::EditTags {
                    editing_host,
                    tag_input,
                    selected_tag_index,
                    input_mode,
                    ..
                } => {
                    // Get global tags from app
                    let all_tags = app.all_tags();
                    render_tag_edit_view(frame, &editing_host.tags, &all_tags, tag_input, *selected_tag_index, *input_mode, area);
                }
                AppMode::SelectSshFlags {
                    editing_host,
                    selected_flag_index,
                    ..
                } => {
                    render_ssh_flags_selection_view(frame, &editing_host.ssh_flags, *selected_flag_index, area);
                }
                AppMode::SelectShell {
                    editing_host,
                    selected_shell_index,
                    ..
                } => {
                    render_shell_selection_view(frame, editing_host.shell.as_ref(), *selected_shell_index, area);
                }
                AppMode::Search { query } => {
                    // Render table with search overlay
                    render_table_view(frame, app, area);
                    render_search_overlay(frame, query, area);
                }
                AppMode::TagFilter { selected_tags } => {
                    let all_tags = app.all_tags();
                    render_tag_filter_view(frame, &all_tags, selected_tags, area);
                }
                AppMode::Help => {
                    render_help_view(frame, area);
                }
                AppMode::ConfirmDelete { host_index } => {
                    // Render table with confirmation dialog
                    render_table_view(frame, app, area);

                    // Show confirmation dialog
                    if *host_index < app.hosts.len() {
                        let host = &app.hosts[*host_index];
                        render_delete_confirmation(frame, host, area);
                    }
                }
            }
        })?;

        // Handle input
        handle_input(app)?;

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Check if there's a pending SSH connection
        if let Some(host) = app.pending_connection.clone() {
            // Cleanup terminal before SSH
            disable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(
                stdout,
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            // Execute SSH connection
            let result = connect_to_host(&host);

            // Restore terminal after SSH
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
            enable_raw_mode()?;

            // Update app with connection result
            match result {
                Ok(_) => app.complete_connection(true, None),
                Err(e) => app.complete_connection(false, Some(format!("SSH error: {}", e))),
            }

            // Force a redraw
            terminal.clear()?;
        }
    }

    Ok(())
}
