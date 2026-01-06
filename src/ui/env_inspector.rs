use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header with search
        Constraint::Min(0),     // Env var list
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_env_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (container_name, search_query, total_count) = match &app.mode {
        AppMode::EnvInspector { container_index, search_query, container_vars, .. } => {
            let name = app.containers.get(*container_index)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            (name.to_string(), search_query.clone(), container_vars.len())
        }
        _ => (String::new(), String::new(), 0),
    };

    let search_display = if search_query.is_empty() {
        Span::styled("(type to filter)", styles::style_muted())
    } else {
        Span::styled(format!("Filter: {}", search_query), styles::style_accent())
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Env Inspector ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(&container_name, styles::style_running()),
        Span::styled(format!(" ({} vars) ", total_count), styles::style_muted()),
        Span::styled("│ ", styles::style_muted()),
        search_display,
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_env_list(frame: &mut Frame, app: &App, area: Rect) {
    let (container_vars, script_vars, selected_index, search_query) = match &app.mode {
        AppMode::EnvInspector { container_vars, script_vars, selected_index, search_query, .. } => {
            (container_vars, script_vars, *selected_index, search_query)
        }
        _ => return,
    };

    // Filter vars by search query
    let filtered_vars: Vec<&(String, String)> = if search_query.is_empty() {
        container_vars.iter().collect()
    } else {
        let query = search_query.to_lowercase();
        container_vars
            .iter()
            .filter(|(k, v)| k.to_lowercase().contains(&query) || v.to_lowercase().contains(&query))
            .collect()
    };

    if filtered_vars.is_empty() {
        let msg = if search_query.is_empty() {
            "No environment variables found"
        } else {
            "No matching variables"
        };
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(msg, styles::style_muted())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Container Environment "));
        frame.render_widget(empty, area);
        return;
    }

    // Calculate visible rows (area height - 2 for borders - 1 for header)
    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_vars = filtered_vars.len();

    // ALWAYS compute scroll_offset from selected_index to guarantee visibility
    // Keep selection in the middle of the visible area when possible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected_index < visible_rows / 2 {
        // Near the top - no scrolling needed
        0
    } else if selected_index >= total_vars.saturating_sub(visible_rows / 2) {
        // Near the bottom - scroll to show bottom items
        total_vars.saturating_sub(visible_rows)
    } else {
        // In the middle - center the selection
        selected_index.saturating_sub(visible_rows / 2)
    };

    // Create a set of script var keys for comparison
    let script_keys: std::collections::HashSet<&str> = script_vars.iter().map(|(k, _)| k.as_str()).collect();

    let header_cells = ["", "Key", "Value", "In Script"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    // Slice to only visible rows based on scroll offset
    let end_index = (scroll_offset + visible_rows).min(total_vars);
    let visible_vars = &filtered_vars[scroll_offset..end_index];

    let rows: Vec<Row> = visible_vars
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected_index { "▸" } else { " " };

            // Check if this var exists in the script
            let in_script = script_keys.contains(key.as_str());
            let script_indicator = if in_script {
                Cell::from("✓").style(styles::style_running())
            } else {
                Cell::from("-").style(styles::style_muted())
            };

            // Truncate long values
            let value_display = if value.len() > 50 {
                format!("{}...", &value[..47])
            } else {
                value.clone()
            };

            let style = if actual_index == selected_index {
                styles::style_selected()
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(key.clone()).style(styles::style_accent()),
                Cell::from(value_display),
                script_indicator,
            ])
            .style(style)
        })
        .collect();

    // Show scroll position in title
    let scroll_info = if total_vars > visible_rows {
        format!(" Container Environment ({}-{} of {}) ",
            scroll_offset + 1,
            end_index,
            total_vars
        )
    } else {
        format!(" Container Environment ({}/{}) ",
            filtered_vars.len(),
            container_vars.len()
        )
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(25),
            Constraint::Percentage(60),
            Constraint::Percentage(13),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(scroll_info));

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = "[j/k] Navigate  [g/G] Top/Bottom  [type] Filter  [Esc] Back";

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
