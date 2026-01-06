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

    let (script_path, script_content, scroll_offset) = match &app.mode {
        AppMode::ScriptViewer { script_path, script_content, scroll_offset, .. } => {
            (script_path, script_content, *scroll_offset)
        }
        _ => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Content
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Script Viewer ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(script_path, styles::style_accent()),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Content - show script with line numbers
    let content_height = chunks[1].height.saturating_sub(2) as usize;
    let total_lines = script_content.len();

    let visible_lines: Vec<Line> = script_content
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(content_height)
        .map(|(i, line)| {
            let line_num = format!("{:4} │ ", i + 1);
            Line::from(vec![
                Span::styled(line_num, styles::style_muted()),
                Span::styled(colorize_script_line(line), get_line_style(line)),
            ])
        })
        .collect();

    let content = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " {} lines ",
            total_lines
        )))
        .wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    // Footer
    let scroll_info = format!(
        " Lines {}-{} of {} ",
        scroll_offset + 1,
        (scroll_offset + content_height).min(total_lines),
        total_lines
    );

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[j/k] Scroll  [g/G] Top/Bottom  [Ctrl+d/u] Page  [q/Esc] Back", styles::style_muted()),
        Span::styled(scroll_info, styles::style_accent()),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn colorize_script_line(line: &str) -> String {
    line.to_string()
}

fn get_line_style(line: &str) -> ratatui::style::Style {
    let trimmed = line.trim();

    if trimmed.starts_with('#') {
        styles::style_muted()  // Comments
    } else if trimmed.starts_with("docker ") {
        styles::style_accent()  // Docker commands
    } else if trimmed.contains("=-e ") || trimmed.starts_with("-e ") {
        styles::style_running()  // Environment variables
    } else if trimmed.contains("=-p ") || trimmed.starts_with("-p ") {
        styles::style_paused()  // Port mappings
    } else if trimmed.contains("=-v ") || trimmed.starts_with("-v ") {
        styles::style_status()  // Volume mounts
    } else {
        styles::style_default()
    }
}
