use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

/// Render the main table view
pub fn render_table_view(frame: &mut Frame, app: &App, area: Rect) {
    // Split the area into header, table, and footer
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Table
        Constraint::Length(3), // Footer
    ])
    .split(area);

    // Render header
    render_header(frame, app, chunks[0]);

    // Render table
    render_host_table(frame, app, chunks[1]);

    // Render footer
    render_footer(frame, app, chunks[2]);
}

/// Render the header with search bar and tag filters
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut header_text = Vec::new();

    // Sort indicator
    header_text.push(Span::styled(
        format!("Sort: {} ", app.sort_by.label()),
        Style::default().fg(Color::Magenta),
    ));

    // Search indicator
    if !app.search_query.is_empty() {
        header_text.push(Span::raw("│ "));
        header_text.push(Span::styled(
            format!("Search: {} ", app.search_query),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Tag filters
    if !app.active_tag_filters.is_empty() {
        header_text.push(Span::raw("│ Tags: "));
        for (i, tag) in app.active_tag_filters.iter().enumerate() {
            if i > 0 {
                header_text.push(Span::raw(", "));
            }
            header_text.push(Span::styled(
                format!("[{}]", tag),
                Style::default().fg(Color::Cyan),
            ));
        }
    }

    // All tags with counts (only if no filters active)
    if app.active_tag_filters.is_empty() && app.search_query.is_empty() {
        let all_tags = app.all_tags();
        if !all_tags.is_empty() {
            header_text.push(Span::raw("  │ Tags: "));
            for (i, tag) in all_tags.iter().take(5).enumerate() {
                if i > 0 {
                    header_text.push(Span::raw(", "));
                }
                let count = app.hosts.iter().filter(|h| h.tags.contains(tag)).count();
                header_text.push(Span::styled(
                    format!("[{}:{}]", tag, count),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            if all_tags.len() > 5 {
                header_text.push(Span::raw("..."));
            }
        }
    }

    let header = Paragraph::new(Line::from(header_text))
        .block(Block::default().borders(Borders::ALL).title(" sshing "));

    frame.render_widget(header, area);
}

/// Get color for a tag based on its name
fn get_tag_color(tag: &str) -> Color {
    match tag.to_lowercase().as_str() {
        "prod" | "production" => Color::Red,
        "staging" | "stage" => Color::Yellow,
        "dev" | "development" => Color::Green,
        "test" | "testing" => Color::Cyan,
        "db" | "database" => Color::Magenta,
        "web" | "frontend" => Color::Blue,
        "api" | "backend" => Color::LightBlue,
        "critical" | "important" => Color::LightRed,
        _ => Color::DarkGray,
    }
}

/// Render the main host table
fn render_host_table(frame: &mut Frame, app: &App, area: Rect) {
    let filtered_hosts = app.filtered_hosts();

    // Calculate visible rows (subtract 3 for borders and header)
    let visible_rows = area.height.saturating_sub(4) as usize;
    let total_hosts = filtered_hosts.len();
    let selected = app.selected_index;

    // Compute scroll_offset to keep selection visible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected < visible_rows / 2 {
        0
    } else if selected >= total_hosts.saturating_sub(visible_rows / 2) {
        total_hosts.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };

    // Create table headers
    let header_cells = ["Host", "Hostname", "User", "Port", "Keys", "Tags", "Note"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    // Only render visible rows
    let end_index = (scroll_offset + visible_rows).min(total_hosts);
    let visible_hosts = if total_hosts > 0 {
        &filtered_hosts[scroll_offset..end_index]
    } else {
        &filtered_hosts[..]
    };

    // Create table rows
    let rows: Vec<Row> = visible_hosts
        .iter()
        .enumerate()
        .map(|(i, host)| {
            let actual_index = scroll_offset + i;
            let is_selected = actual_index == app.selected_index;

            // Get primary tag color for the row
            let primary_color = host
                .tags
                .first()
                .map(|tag| get_tag_color(tag))
                .unwrap_or(Color::White);

            let base_style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Build colored tags display
            let tags_display = if host.tags.is_empty() {
                "-".to_string()
            } else {
                host.tags.join(", ")
            };

            let cells = vec![
                Cell::from(host.host.clone()),
                Cell::from(host.hostname.clone()),
                Cell::from(host.user.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(host.effective_port().to_string()),
                Cell::from(if host.has_keys() { "✓" } else { "-" }),
                Cell::from(tags_display).style(
                    if is_selected {
                        base_style
                    } else {
                        Style::default().fg(primary_color)
                    }
                ),
                Cell::from(host.note.clone().unwrap_or_default()),
            ];

            Row::new(cells).style(base_style).height(1)
        })
        .collect();

    // Show scroll position in title if needed
    let title = if total_hosts > visible_rows && visible_rows > 0 {
        format!(
            " Hosts ({}-{} of {}/{}) ",
            scroll_offset + 1,
            end_index,
            total_hosts,
            app.hosts.len()
        )
    } else {
        format!(" Hosts ({}/{}) ", filtered_hosts.len(), app.hosts.len())
    };

    // Create the table
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15), // Host
            Constraint::Percentage(20), // Hostname
            Constraint::Percentage(10), // User
            Constraint::Percentage(5),  // Port
            Constraint::Percentage(5),  // Keys
            Constraint::Percentage(15), // Tags
            Constraint::Percentage(30), // Note
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .column_spacing(1);

    frame.render_widget(table, area);
}

/// Render the footer with keybindings help
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keybindings = vec![
        ("Space", "Connect"),
        ("d", "Docker"),
        if app.rsync_available { ("r", "Rsync") } else { ("r", "Rsync (disabled)") },
        ("n", "New"),
        ("e", "Edit"),
        ("D", "Delete"),
        ("/", "Search"),
        ("t", "Tags"),
        ("s", "Sort"),
        ("?", "Help"),
        ("q", "Quit"),
    ];

    let mut footer_spans = Vec::new();
    for (i, (key, desc)) in keybindings.iter().enumerate() {
        if i > 0 {
            footer_spans.push(Span::raw(" │ "));
        }

        // Grey out rsync keybinding if not available
        let key_style = if !app.rsync_available && *key == "r" {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };

        footer_spans.push(Span::styled(*key, key_style));
        footer_spans.push(Span::raw(":"));
        footer_spans.push(Span::raw(*desc));
    }

    let mut footer = Paragraph::new(Line::from(footer_spans))
        .block(Block::default().borders(Borders::ALL));

    // Show status or error messages
    if let Some(ref msg) = app.status_message {
        footer = Paragraph::new(msg.clone())
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Status "));
    } else if let Some(ref msg) = app.error_message {
        footer = Paragraph::new(msg.clone())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title(" Error "));
    }

    frame.render_widget(footer, area);
}
