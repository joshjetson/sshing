use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::DockerPendingAction;

/// Render a docker confirmation dialog
pub fn render_docker_confirm(frame: &mut Frame, action: &DockerPendingAction, area: Rect) {
    // Center the dialog
    let dialog_width = 60.min(area.width - 4);
    let dialog_height = 7;

    let x = (area.width - dialog_width) / 2;
    let y = (area.height - dialog_height) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    let description = action.description();

    // Determine if this is a destructive action
    let is_destructive = matches!(action, DockerPendingAction::DockerRemove { .. });

    let border_style = if is_destructive {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let chunks = Layout::vertical([
        Constraint::Length(3), // Description
        Constraint::Length(2), // Buttons
    ])
    .split(Rect::new(dialog_area.x + 1, dialog_area.y + 1, dialog_area.width - 2, dialog_area.height - 2));

    let title = if is_destructive { " Warning " } else { " Confirm " };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(block, dialog_area);

    // Description
    let desc_paragraph = Paragraph::new(description)
        .alignment(Alignment::Center);
    frame.render_widget(desc_paragraph, chunks[0]);

    // Buttons
    let buttons = Line::from(vec![
        Span::styled("[Y]es", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("[N]o", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ]);

    let buttons_paragraph = Paragraph::new(buttons)
        .alignment(Alignment::Center);
    frame.render_widget(buttons_paragraph, chunks[1]);
}
