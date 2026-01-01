use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::app_state::RsyncField;

/// Render the rsync mode view
pub fn render_rsync_view(frame: &mut Frame, app: &App, area: Rect) {
    if let crate::models::AppMode::Rsync {
        editing_host,
        source_path,
        dest_path,
        sync_to_host,
        focused_field,
        editing_mode,
        ..
    } = &app.mode
    {
        // Split layout: title, fields, help footer
        let chunks = Layout::vertical([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Form fields
            Constraint::Length(5),  // Help/status
        ])
        .split(area);

        // Title
        let title = Paragraph::new("Rsync File Synchronization")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Form fields area
        let field_chunks = Layout::vertical([
            Constraint::Length(3), // Source path
            Constraint::Length(3), // Dest path
            Constraint::Length(3), // Direction
            Constraint::Min(0),    // Spacer
        ])
        .split(chunks[1]);

        // Source path field
        let source_style = if *focused_field == RsyncField::SourcePath {
            if *editing_mode {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };

        let source_label = Span::styled("Source Path: ", Style::default().fg(Color::White));
        let source_value = Span::raw(source_path.clone());
        let source_hint = if *focused_field == RsyncField::SourcePath && *editing_mode {
            Span::styled(" (editing)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        };
        let source_input = Line::from(vec![source_label, source_value, source_hint]);

        let source_widget = Paragraph::new(source_input)
            .style(source_style)
            .block(Block::default().borders(Borders::ALL).title(" Source "));
        frame.render_widget(source_widget, field_chunks[0]);

        // Destination path field
        let dest_style = if *focused_field == RsyncField::DestPath {
            if *editing_mode {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };

        let dest_label = Span::styled("Dest Path: ", Style::default().fg(Color::White));
        let dest_value = Span::raw(dest_path.clone());
        let dest_hint = if *focused_field == RsyncField::DestPath && *editing_mode {
            Span::styled(" (editing)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        };
        let dest_input = Line::from(vec![dest_label, dest_value, dest_hint]);

        let dest_widget = Paragraph::new(dest_input)
            .style(dest_style)
            .block(Block::default().borders(Borders::ALL).title(" Destination "));
        frame.render_widget(dest_widget, field_chunks[1]);

        // Direction field
        let direction_style = if *focused_field == RsyncField::Direction {
            if *editing_mode {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };

        let direction_label = Span::styled("Direction: ", Style::default().fg(Color::White));
        let direction_value = if *sync_to_host {
            Span::styled("→ To Host (t)", Style::default().fg(Color::Cyan))
        } else {
            Span::styled("← From Host (f)", Style::default().fg(Color::Magenta))
        };
        let direction_input = Line::from(vec![direction_label, direction_value]);

        let direction_widget = Paragraph::new(direction_input)
            .style(direction_style)
            .block(Block::default().borders(Borders::ALL).title(" Sync Direction "));
        frame.render_widget(direction_widget, field_chunks[2]);

        // Help footer
        let help_text = if *editing_mode {
            vec![
                Line::from(Span::styled(
                    "i/Enter: Edit  │  Tab: Next Field  │  Backspace: Delete  │  Esc: Cancel",
                    Style::default().fg(Color::Gray),
                )),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    format!(
                        "Host: {}  │  User: {}",
                        editing_host.hostname,
                        editing_host.user.as_deref().unwrap_or("(none)")
                    ),
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    "k/↑: Up  │  j/↓: Down  │  i/Enter: Edit  │  Esc/q: Back",
                    Style::default().fg(Color::Gray),
                )),
            ]
        };

        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_widget, chunks[2]);
    }
}
