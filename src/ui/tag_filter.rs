use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the tag filter view
pub fn render_tag_filter_view(
    frame: &mut Frame,
    all_tags: &[String],
    selected_tags: &[String],
    area: Rect,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Tag list
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Filter by Tags")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Tag list
    let items: Vec<ListItem> = all_tags
        .iter()
        .map(|tag| {
            let is_selected = selected_tags.contains(tag);
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(format!("{} {}", checkbox, tag)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Tags (Space to toggle) "),
        );

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("Space: Toggle │ Enter: Apply │ Esc: Cancel")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}
