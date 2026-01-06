use ratatui::{
    layout::{Constraint, Layout, Rect, Alignment},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header with path
        Constraint::Min(0),     // File listing
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_file_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (current_path, container_name) = match &app.mode {
        AppMode::FileBrowser { current_path, container_index, .. } => {
            let name = app.containers.get(*container_index)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            (current_path.clone(), name.to_string())
        }
        _ => (String::new(), String::new()),
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Browse for script ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled("Container: ", styles::style_muted()),
        Span::styled(&container_name, styles::style_accent()),
        Span::styled(" │ ", styles::style_muted()),
        Span::styled(&current_path, styles::style_default()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let (entries, selected_index, loading) = match &app.mode {
        AppMode::FileBrowser { entries, selected_index, loading, .. } => {
            (entries, *selected_index, *loading)
        }
        _ => return,
    };

    if loading {
        let loading_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("Loading...", styles::style_status())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Files "))
        .alignment(Alignment::Center);
        frame.render_widget(loading_msg, area);
        return;
    }

    if entries.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("Directory is empty", styles::style_muted())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Files "))
        .alignment(Alignment::Center);
        frame.render_widget(empty_msg, area);
        return;
    }

    // Calculate visible rows (area height - 2 for borders)
    let visible_rows = area.height.saturating_sub(2) as usize;
    let total_entries = entries.len();

    // Compute scroll_offset to keep selection visible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected_index < visible_rows / 2 {
        0
    } else if selected_index >= total_entries.saturating_sub(visible_rows / 2) {
        total_entries.saturating_sub(visible_rows)
    } else {
        selected_index.saturating_sub(visible_rows / 2)
    };

    // Only render visible entries
    let end_index = (scroll_offset + visible_rows).min(total_entries);
    let visible_entries = &entries[scroll_offset..end_index];

    let items: Vec<ListItem> = visible_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected_index { "▸ " } else { "  " };

            let (icon, style) = if entry.is_dir {
                ("📁 ", styles::style_accent())
            } else if entry.is_script {
                ("📜 ", styles::style_running())
            } else {
                ("   ", styles::style_muted())
            };

            let line_style = if actual_index == selected_index {
                styles::style_selected()
            } else {
                style
            };

            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(icon, style),
                Span::styled(&entry.name, line_style),
            ]))
        })
        .collect();

    // Show scroll position in title if needed
    let title = if total_entries > visible_rows {
        format!(" Files ({}-{} of {}) ", scroll_offset + 1, end_index, total_entries)
    } else {
        format!(" Files ({}) ", total_entries)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = "[Enter] Open/Select  [Esc] Cancel  [j/k] Navigate";

    let (message, style) = if let Some(ref err) = app.error_message {
        (err.clone(), styles::style_error())
    } else if let Some(ref status) = app.status_message {
        (status.clone(), styles::style_status())
    } else {
        (help.to_string(), styles::style_muted())
    };

    let footer = Paragraph::new(Line::from(Span::styled(message, style)))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
