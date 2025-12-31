use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the help overlay
pub fn render_help_view(frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Help content
        Constraint::Length(3),  // Close instruction
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Help")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Help content
    let help_text = vec![
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  j / ↓         - Move down"),
        Line::from("  k / ↑         - Move up"),
        Line::from("  g             - Jump to first host"),
        Line::from("  G             - Jump to last host"),
        Line::from("  Ctrl+d        - Page down"),
        Line::from("  Ctrl+u        - Page up"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space / Enter - Connect to selected host"),
        Line::from("  n             - Create new host"),
        Line::from("  e             - Edit selected host"),
        Line::from("  d             - Delete selected host"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Filtering & Sorting:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  /             - Search hosts"),
        Line::from("  t             - Filter by tags"),
        Line::from("  s             - Cycle sort order (Name/Hostname/Last Used/User)"),
        Line::from("  Esc           - Clear filters"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Other:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ?             - Show this help"),
        Line::from("  q             - Quit"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Keybindings "))
        .alignment(Alignment::Left);

    frame.render_widget(help_paragraph, chunks[1]);

    // Close instruction
    let close = Paragraph::new("Press any key to close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(close, chunks[2]);
}
