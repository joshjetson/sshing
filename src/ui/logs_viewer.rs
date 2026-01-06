use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let (container_name, log_buffer, follow_mode, scroll_offset, tail_count) = match &app.mode {
        AppMode::LogsViewer {
            container_index,
            log_buffer,
            follow_mode,
            scroll_offset,
            tail_count,
            ..
        } => {
            let name = app
                .containers
                .get(*container_index)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            (name, log_buffer, *follow_mode, *scroll_offset, *tail_count)
        }
        _ => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Logs
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    // Header
    let follow_indicator = if follow_mode { " (following)" } else { "" };
    let tail_info = format!(" (last {} lines)", tail_count);
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Logs: ", styles::style_muted()),
        Span::styled(container_name, styles::style_header()),
        Span::styled(&tail_info, styles::style_muted()),
        Span::styled(follow_indicator, styles::style_accent()),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Logs content
    let logs_height = chunks[1].height.saturating_sub(2) as usize;
    let total_lines = log_buffer.len();

    let visible_lines: Vec<Line> = if total_lines <= logs_height {
        log_buffer.iter().map(|l| Line::from(l.as_str())).collect()
    } else {
        let start = if follow_mode {
            total_lines.saturating_sub(logs_height)
        } else {
            scroll_offset.min(total_lines.saturating_sub(logs_height))
        };
        let end = (start + logs_height).min(total_lines);
        log_buffer[start..end]
            .iter()
            .map(|l| colorize_log_line(l))
            .collect()
    };

    let logs = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(logs, chunks[1]);

    // Footer
    let scroll_info = format!(
        " Lines {}-{} of {} ",
        scroll_offset + 1,
        (scroll_offset + logs_height).min(total_lines),
        total_lines
    );

    // Show different help based on whether we can load more
    let can_load_more = tail_count < 50000;
    let help_text = if can_load_more {
        "[m] More  [f] Follow  [g/G] Top/Bottom  [j/k] Scroll  [Esc] Back"
    } else {
        "[f] Follow  [g/G] Top/Bottom  [j/k] Scroll  [Esc] Back"
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(help_text, styles::style_muted()),
        Span::styled(scroll_info, styles::style_accent()),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn colorize_log_line(line: &str) -> Line {
    let line_lower = line.to_lowercase();

    let style = if line_lower.contains("error") || line_lower.contains("exception") {
        styles::style_error()
    } else if line_lower.contains("warn") {
        styles::style_paused()
    } else if line_lower.contains("info") {
        styles::style_running()
    } else if line_lower.contains("debug") {
        styles::style_muted()
    } else {
        styles::style_default()
    };

    Line::from(Span::styled(line, style))
}
