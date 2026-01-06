use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Info display
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_info(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let container_name = match &app.mode {
        AppMode::InspectViewer { info, .. } => &info.name,
        _ => "Unknown",
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Container Details ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(container_name, styles::style_running()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_info(frame: &mut Frame, app: &App, area: Rect) {
    let info = match &app.mode {
        AppMode::InspectViewer { info, .. } => info,
        _ => return,
    };

    // Split into two columns
    let columns = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    // Left column: Basic info
    let basic_info = vec![
        Line::from(vec![
            Span::styled("  ID:          ", styles::style_muted()),
            Span::styled(&info.id, styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  Image:       ", styles::style_muted()),
            Span::styled(&info.image, styles::style_accent()),
        ]),
        Line::from(vec![
            Span::styled("  Status:      ", styles::style_muted()),
            Span::styled(&info.status, if info.status.contains("Up") { styles::style_running() } else { styles::style_stopped() }),
        ]),
        Line::from(vec![
            Span::styled("  Created:     ", styles::style_muted()),
            Span::styled(format_timestamp(&info.created), styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  Started:     ", styles::style_muted()),
            Span::styled(format_timestamp(&info.started), styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  IP Address:  ", styles::style_muted()),
            Span::styled(&info.ip_address, styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  Restart:     ", styles::style_muted()),
            Span::styled(&info.restart_policy, styles::style_default()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Networks:", styles::style_header())),
    ];

    let mut left_lines = basic_info;

    if info.networks.is_empty() {
        left_lines.push(Line::from(Span::styled("    (none)", styles::style_muted())));
    } else {
        for net in &info.networks {
            left_lines.push(Line::from(vec![
                Span::styled("    • ", styles::style_muted()),
                Span::styled(net, styles::style_default()),
            ]));
        }
    }

    // Health status if available
    if let Some(ref health) = info.health_status {
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(vec![
            Span::styled("  Health:      ", styles::style_muted()),
            Span::styled(health, match health.as_str() {
                "healthy" => styles::style_running(),
                "unhealthy" => styles::style_error(),
                _ => styles::style_accent(),
            }),
        ]));
    }

    let left_panel = Paragraph::new(left_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(left_panel, columns[0]);

    // Right column: Ports and Volumes
    let right_chunks = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .split(columns[1]);

    // Ports section
    let mut port_lines = vec![];
    if info.ports.is_empty() {
        port_lines.push(Line::from(Span::styled("  (no ports exposed)", styles::style_muted())));
    } else {
        for port in &info.ports {
            port_lines.push(Line::from(vec![
                Span::styled("  • ", styles::style_muted()),
                Span::styled(port, styles::style_accent()),
            ]));
        }
    }

    let ports_panel = Paragraph::new(port_lines)
        .block(Block::default().borders(Borders::ALL).title(" Ports "));
    frame.render_widget(ports_panel, right_chunks[0]);

    // Volumes section
    let mut volume_lines = vec![];
    if info.volumes.is_empty() {
        volume_lines.push(Line::from(Span::styled("  (no volumes mounted)", styles::style_muted())));
    } else {
        for vol in &info.volumes {
            // Truncate long paths
            let display = if vol.len() > 45 {
                format!("{}...", &vol[..42])
            } else {
                vol.clone()
            };
            volume_lines.push(Line::from(vec![
                Span::styled("  • ", styles::style_muted()),
                Span::styled(display, styles::style_default()),
            ]));
        }
    }

    let volumes_panel = Paragraph::new(volume_lines)
        .block(Block::default().borders(Borders::ALL).title(" Volumes "));
    frame.render_widget(volumes_panel, right_chunks[1]);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = "[r] Refresh  [Esc] Back";

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

/// Format ISO timestamp to a more readable format
fn format_timestamp(ts: &str) -> String {
    // Input: "2024-01-15T10:30:00.123456Z"
    // Output: "2024-01-15 10:30:00"
    if ts.len() >= 19 {
        ts[..19].replace('T', " ")
    } else {
        ts.to_string()
    }
}
