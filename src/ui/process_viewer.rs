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
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Process list
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_process_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (container_name, process_count) = match &app.mode {
        AppMode::ProcessViewer { container_index, processes, .. } => {
            let name = app.containers.get(*container_index)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            (name.to_string(), processes.len())
        }
        _ => (String::new(), 0),
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Running Processes ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(&container_name, styles::style_running()),
        Span::styled(format!(" ({} processes)", process_count), styles::style_muted()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_process_list(frame: &mut Frame, app: &App, area: Rect) {
    let (processes, selected_index) = match &app.mode {
        AppMode::ProcessViewer { processes, selected_index, .. } => {
            (processes, *selected_index)
        }
        _ => return,
    };

    if processes.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No processes found", styles::style_muted())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Processes "));
        frame.render_widget(empty, area);
        return;
    }

    // Calculate visible rows (area height - 2 for borders - 1 for header)
    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_processes = processes.len();

    // Compute scroll_offset to keep selection visible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected_index < visible_rows / 2 {
        0
    } else if selected_index >= total_processes.saturating_sub(visible_rows / 2) {
        total_processes.saturating_sub(visible_rows)
    } else {
        selected_index.saturating_sub(visible_rows / 2)
    };

    let header_cells = ["", "PID", "User", "CPU %", "Mem %", "Command"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    // Only render visible rows
    let end_index = (scroll_offset + visible_rows).min(total_processes);
    let visible_processes = &processes[scroll_offset..end_index];

    let rows: Vec<Row> = visible_processes
        .iter()
        .enumerate()
        .map(|(i, proc)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected_index { "▸" } else { " " };

            // Color CPU based on usage
            let cpu_val: f64 = proc.cpu.parse().unwrap_or(0.0);
            let cpu_style = if cpu_val > 80.0 {
                styles::style_error()
            } else if cpu_val > 50.0 {
                styles::style_accent()
            } else {
                styles::style_default()
            };

            // Color memory based on usage
            let mem_val: f64 = proc.mem.parse().unwrap_or(0.0);
            let mem_style = if mem_val > 80.0 {
                styles::style_error()
            } else if mem_val > 50.0 {
                styles::style_accent()
            } else {
                styles::style_default()
            };

            let style = if actual_index == selected_index {
                styles::style_selected()
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(proc.pid.clone()).style(styles::style_muted()),
                Cell::from(proc.user.clone()),
                Cell::from(format!("{}%", proc.cpu)).style(cpu_style),
                Cell::from(format!("{}%", proc.mem)).style(mem_style),
                Cell::from(proc.command.clone()),
            ])
            .style(style)
        })
        .collect();

    // Show scroll position in title if needed
    let title = if total_processes > visible_rows {
        format!(" Processes ({}-{} of {}) ", scroll_offset + 1, end_index, total_processes)
    } else {
        format!(" Processes ({}) ", total_processes)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = "[j/k] Navigate  [r] Refresh  [Esc] Back";

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
