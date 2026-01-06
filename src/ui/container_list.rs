use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::models::ContainerStatus;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Container list
        Constraint::Length(4),  // Footer/status
    ])
    .split(area);

    render_header(frame, app, chunks[0]);

    if app.containers.is_empty() {
        render_empty_state(frame, app, chunks[1]);
    } else {
        render_container_table(frame, app, chunks[1]);
    }

    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let server_name = app
        .get_current_docker_host()
        .map(|s| s.host.as_str())
        .unwrap_or("Unknown");

    let scripts_count = app.scripts.len();
    let running_count = app.containers.iter().filter(|c| c.status == ContainerStatus::Running).count();

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Docker: ", styles::style_default()),
        Span::styled(server_name, styles::style_header()),
        Span::styled(" │ ", styles::style_muted()),
        Span::styled(format!("{} containers", app.containers.len()), styles::style_default()),
        Span::styled(format!(" ({} running)", running_count), styles::style_running()),
        Span::styled(" │ ", styles::style_muted()),
        Span::styled(format!("{} scripts", scripts_count), styles::style_accent()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_empty_state(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let message = if app.status_message.as_ref().map_or(false, |s| s.contains("...")) {
        // Still loading
        vec![
            Line::from(""),
            Line::from(Span::styled("Loading...", styles::style_status())),
            Line::from(""),
            Line::from(Span::styled("Fetching containers and discovering scripts", styles::style_muted())),
        ]
    } else {
        // No containers found
        vec![
            Line::from(""),
            Line::from(Span::styled("No containers found", styles::style_header())),
            Line::from(""),
            Line::from("This server has no Docker containers."),
            Line::from(""),
            Line::from(Span::styled("Press [Esc] to go back", styles::style_muted())),
        ]
    };

    let paragraph = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_container_table(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Calculate visible rows
    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_containers = app.containers.len();
    let selected = app.docker_selected_index;

    // Compute scroll_offset to keep selection visible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected < visible_rows / 2 {
        0
    } else if selected >= total_containers.saturating_sub(visible_rows / 2) {
        total_containers.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };

    let header_cells = ["", "Name", "Status", "Image", "Ports", "Script"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    // Only render visible rows
    let end_index = (scroll_offset + visible_rows).min(total_containers);
    let visible_containers = &app.containers[scroll_offset..end_index];

    let rows: Vec<Row> = visible_containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == app.docker_selected_index { "▸" } else { " " };

            let status_style = match container.status {
                ContainerStatus::Running => styles::style_running(),
                ContainerStatus::Stopped | ContainerStatus::Exited(_) => styles::style_stopped(),
                ContainerStatus::Paused => styles::style_paused(),
                _ => styles::style_muted(),
            };

            let status_indicator = match container.status {
                ContainerStatus::Running => "● Up",
                ContainerStatus::Stopped => "○ Down",
                ContainerStatus::Exited(code) => {
                    if code == 0 {
                        "○ Exited"
                    } else {
                        "✗ Failed"
                    }
                }
                ContainerStatus::Paused => "◐ Paused",
                ContainerStatus::Restarting => "↻ Restart",
                _ => "? Unknown",
            };

            // Show script path hint if available
            let script_cell = if container.has_script() {
                Cell::from("✓ has script").style(styles::style_running())
            } else {
                Cell::from("✗ no script").style(styles::style_muted())
            };

            let row_style = if actual_index == app.docker_selected_index {
                styles::style_selected()
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(container.name.clone()),
                Cell::from(status_indicator).style(status_style),
                Cell::from(container.short_image()),
                Cell::from(container.ports_display()),
                script_cell,
            ])
            .style(row_style)
        })
        .collect();

    // Show scroll position in title if needed
    let title = if total_containers > visible_rows {
        format!(" Containers ({}-{} of {}) ", scroll_offset + 1, end_index, total_containers)
    } else {
        format!(" Containers ({}) ", total_containers)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(18),
            Constraint::Percentage(12),
            Constraint::Percentage(28),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Context-aware help based on selected container
    let has_script = app.containers
        .get(app.docker_selected_index)
        .map_or(false, |c| c.has_script());

    let help = if has_script {
        "[e]dit [v]iew [x]Run [b]Replace  [l]ogs [E]nv [D]stats [T]op [I]nfo [p]ull [r]estart [s]top [S]tart"
    } else {
        "[b]rowse [n]ew  [l]ogs [E]nv [D]stats [T]op [I]nfo [d]el [X]Purge [p]ull [r]estart [s]top [S]tart"
    };

    // Show error/status on first line, help on second line
    let content = if let Some(ref err) = app.error_message {
        vec![
            Line::from(Span::styled(err.clone(), styles::style_error())),
            Line::from(Span::styled(help, styles::style_muted())),
        ]
    } else if let Some(ref status) = app.status_message {
        vec![
            Line::from(Span::styled(status.clone(), styles::style_status())),
            Line::from(Span::styled(help, styles::style_muted())),
        ]
    } else {
        vec![Line::from(Span::styled(help, styles::style_muted()))]
    };

    let footer = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
