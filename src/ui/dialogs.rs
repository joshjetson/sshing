use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::Host;

/// Render a delete confirmation dialog
pub fn render_delete_confirmation(frame: &mut Frame, host: &Host, area: Rect) {
    // Create a centered dialog box
    let dialog_area = centered_rect(60, 30, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Message
        Constraint::Length(3),  // Actions
    ])
    .split(dialog_area);

    // Title
    let title = Paragraph::new("Confirm Delete")
        .style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
    frame.render_widget(title, chunks[0]);

    // Message
    let message = format!(
        "Are you sure you want to delete host '{}'?\n\nHostname: {}\nUser: {}\n\nThis action cannot be undone.",
        host.host,
        host.hostname,
        host.user.clone().unwrap_or_else(|| "-".to_string())
    );
    let msg_widget = Paragraph::new(message)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(msg_widget, chunks[1]);

    // Actions
    let actions = Paragraph::new("Y: Yes, delete  │  N/Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(actions, chunks[2]);
}

/// Render a search input overlay
pub fn render_search_overlay(frame: &mut Frame, query: &str, area: Rect) {
    // Create a search bar at the top
    let search_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: 3,
    };

    // Clear the area behind the search box
    frame.render_widget(Clear, search_area);

    let search_text = if query.is_empty() {
        "Type to search...".to_string()
    } else {
        query.to_string()
    };

    let search_widget = Paragraph::new(search_text)
        .style(
            Style::default()
                .fg(if query.is_empty() {
                    Color::DarkGray
                } else {
                    Color::Yellow
                })
                .add_modifier(if query.is_empty() {
                    Modifier::empty()
                } else {
                    Modifier::BOLD
                }),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(search_widget, search_area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
