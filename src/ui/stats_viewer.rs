use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Gauge},
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Stats display
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_stats(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let container_name = match &app.mode {
        AppMode::StatsViewer { container_index, .. } => {
            app.containers.get(*container_index)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown")
        }
        _ => "Unknown",
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Container Stats ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(container_name, styles::style_running()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let stats = match &app.mode {
        AppMode::StatsViewer { stats, .. } => stats,
        _ => return,
    };

    // Create a centered layout for stats
    let chunks = Layout::vertical([
        Constraint::Length(1),  // Spacer
        Constraint::Length(3),  // CPU
        Constraint::Length(1),  // Spacer
        Constraint::Length(3),  // Memory
        Constraint::Length(1),  // Spacer
        Constraint::Length(5),  // I/O stats
        Constraint::Min(0),     // Rest
    ])
    .split(area);

    // CPU Usage
    let cpu_percent = stats.cpu_percent.trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU Usage "))
        .gauge_style(styles::style_running())
        .percent(cpu_percent.min(100.0) as u16)
        .label(format!("{}", stats.cpu_percent));
    frame.render_widget(cpu_gauge, chunks[1]);

    // Memory Usage
    let mem_percent = stats.memory_percent.trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
    let mem_label = format!("{} / {} ({})", stats.memory_usage, stats.memory_limit, stats.memory_percent);
    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory Usage "))
        .gauge_style(styles::style_accent())
        .percent(mem_percent.min(100.0) as u16)
        .label(mem_label);
    frame.render_widget(mem_gauge, chunks[3]);

    // I/O and PIDs
    let io_info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Network I/O: ", styles::style_muted()),
            Span::styled(&stats.net_io, styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  Block I/O:   ", styles::style_muted()),
            Span::styled(&stats.block_io, styles::style_default()),
        ]),
        Line::from(vec![
            Span::styled("  PIDs:        ", styles::style_muted()),
            Span::styled(&stats.pids, styles::style_default()),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" I/O & Processes "));
    frame.render_widget(io_info, chunks[5]);
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
