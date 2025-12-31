use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{Host, HostField};

/// Render the host editor view
pub fn render_editor_view(
    frame: &mut Frame,
    editing_host: &Host,
    focused_field: &HostField,
    field_buffer: &str,
    editing_mode: bool,
    area: Rect,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Form
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title with mode indicator
    let mode_text = if editing_mode {
        "Edit Host - EDITING MODE"
    } else {
        "Edit Host - NAVIGATION MODE"
    };
    let title = Paragraph::new(mode_text)
        .style(Style::default().fg(if editing_mode { Color::Green } else { Color::Cyan }).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Form
    render_form(frame, editing_host, focused_field, field_buffer, editing_mode, chunks[1]);

    // Instructions based on mode
    let instructions_text = if editing_mode {
        "Type to edit │ Enter: Save field │ Tab: Save & next field │ Esc: Cancel edit │ Ctrl+S: SAVE ALL"
    } else {
        "j/k/↑/↓: Navigate │ Enter: Edit field │ Tab: Next field │ Ctrl+S: SAVE │ Esc: Cancel"
    };
    let instructions = Paragraph::new(instructions_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(if editing_mode { Color::Green } else { Color::Cyan }))
        .block(Block::default().borders(Borders::ALL).title(" Remember to press Ctrl+S to save! "));
    frame.render_widget(instructions, chunks[2]);
}

fn render_form(frame: &mut Frame, host: &Host, focused_field: &HostField, field_buffer: &str, editing_mode: bool, area: Rect) {
    let form_chunks = Layout::vertical([
        Constraint::Length(3), // Host alias
        Constraint::Length(3), // Hostname
        Constraint::Length(3), // User
        Constraint::Length(3), // Port
        Constraint::Length(3), // Identity files
        Constraint::Length(3), // Proxy jump
        Constraint::Length(3), // SSH Flags
        Constraint::Length(3), // Shell
        Constraint::Length(3), // Tags
        Constraint::Length(4), // Note
    ])
    .split(area);

    // Helper to render a field
    let render_field = |frame: &mut Frame,
                        field: HostField,
                        label: &str,
                        value: &str,
                        area: Rect| {
        let is_focused = focused_field == &field;
        let is_editing = is_focused && editing_mode;

        // Text style
        let style = if is_editing {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Border style
        let border_style = if is_editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        // Title with mode indicator
        let title = if is_editing {
            format!(" {} [EDITING] ", label)
        } else if is_focused {
            format!(" {} [Press Enter to edit] ", label)
        } else {
            format!(" {} ", label)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let text = if is_focused && value.is_empty() {
            Span::styled("_", Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(value, style)
        };

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    };

    // Helper to get display value for a field
    let get_display_value = |field: HostField| {
        if focused_field == &field {
            field_buffer.to_string()
        } else {
            match field {
                HostField::HostAlias => host.host.clone(),
                HostField::Hostname => host.hostname.clone(),
                HostField::User => host.user.clone().unwrap_or_default(),
                HostField::Port => host.port.map(|p| p.to_string()).unwrap_or_default(),
                HostField::ProxyJump => host.proxy_jump.clone().unwrap_or_default(),
                HostField::Note => host.note.clone().unwrap_or_default(),
                HostField::IdentityFiles => {
                    host.identity_file
                        .as_ref()
                        .map(|keys| keys.join(", "))
                        .unwrap_or_else(|| "Press Enter to select...".to_string())
                }
                HostField::SshFlags => {
                    if host.ssh_flags.is_empty() {
                        "Press Enter to select flags...".to_string()
                    } else {
                        host.ssh_flags.join(" ")
                    }
                }
                HostField::Shell => {
                    host.shell
                        .clone()
                        .unwrap_or_else(|| "Press Enter to select shell...".to_string())
                }
                HostField::Tags => {
                    if host.tags.is_empty() {
                        "Press Enter to edit...".to_string()
                    } else {
                        host.tags.join(", ")
                    }
                }
            }
        }
    };

    // Render each field
    render_field(
        frame,
        HostField::HostAlias,
        "Host (alias)",
        &get_display_value(HostField::HostAlias),
        form_chunks[0],
    );

    render_field(
        frame,
        HostField::Hostname,
        "Hostname (IP)",
        &get_display_value(HostField::Hostname),
        form_chunks[1],
    );

    render_field(
        frame,
        HostField::User,
        "User",
        &get_display_value(HostField::User),
        form_chunks[2],
    );

    render_field(
        frame,
        HostField::Port,
        "Port (default: 22)",
        &get_display_value(HostField::Port),
        form_chunks[3],
    );

    render_field(
        frame,
        HostField::IdentityFiles,
        "SSH Keys (Enter to select)",
        &get_display_value(HostField::IdentityFiles),
        form_chunks[4],
    );

    render_field(
        frame,
        HostField::ProxyJump,
        "Jump Host",
        &get_display_value(HostField::ProxyJump),
        form_chunks[5],
    );

    render_field(
        frame,
        HostField::SshFlags,
        "SSH Flags (Enter to select)",
        &get_display_value(HostField::SshFlags),
        form_chunks[6],
    );

    render_field(
        frame,
        HostField::Shell,
        "Shell (Enter to select)",
        &get_display_value(HostField::Shell),
        form_chunks[7],
    );

    render_field(
        frame,
        HostField::Tags,
        "Tags (Enter to edit)",
        &get_display_value(HostField::Tags),
        form_chunks[8],
    );

    render_field(
        frame,
        HostField::Note,
        "Note",
        &get_display_value(HostField::Note),
        form_chunks[9],
    );
}

/// Render SSH key selection view
pub fn render_key_selection_view(
    frame: &mut Frame,
    available_keys: &[String],
    selected_keys: &[String],
    selected_index: usize,
    area: Rect,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Key list
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Select SSH Keys")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Key list
    let items: Vec<ListItem> = available_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let is_selected = selected_keys.contains(key);
            let is_highlighted = i == selected_index;
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let style = if is_highlighted {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            ListItem::new(format!("{} {}", checkbox, key)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Available SSH Keys "),
    );

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("j/k/↑/↓: Navigate │ Space/Enter: Toggle │ Esc: Back")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}

/// Render tag editing view
pub fn render_tag_edit_view(
    frame: &mut Frame,
    host_tags: &[String],
    all_available_tags: &[String],
    tag_input: &str,
    selected_index: usize,
    input_mode: bool,
    area: Rect,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Length(3),  // Input field
        Constraint::Min(0),     // Tag list
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title with mode indicator
    let mode_text = if input_mode {
        "Assign Tags - CREATE NEW TAG (adds to global pool)"
    } else {
        "Assign Tags - SELECT/DESELECT TAGS FOR THIS HOST"
    };
    let title = Paragraph::new(mode_text)
        .style(Style::default().fg(if input_mode { Color::Green } else { Color::Cyan }).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Input field for creating new tags
    let input_text = if input_mode {
        if tag_input.is_empty() {
            "Type new tag name...".to_string()
        } else {
            tag_input.to_string()
        }
    } else {
        "Press 'a' or 'n' to create a new tag for global pool".to_string()
    };

    let input_border_style = if input_mode {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input_widget = Paragraph::new(input_text)
        .style(
            Style::default()
                .fg(if input_mode {
                    if tag_input.is_empty() {
                        Color::DarkGray
                    } else {
                        Color::Yellow
                    }
                } else {
                    Color::DarkGray
                })
                .add_modifier(if input_mode && !tag_input.is_empty() {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(input_border_style)
                .title(if input_mode { " Create New Global Tag (active) " } else { " Create New Global Tag " }),
        );
    frame.render_widget(input_widget, chunks[1]);

    // Tag list showing ALL available tags with checkboxes
    let items: Vec<ListItem> = all_available_tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let is_selected = host_tags.contains(tag);
            let is_highlighted = i == selected_index && !input_mode;
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let style = if is_highlighted {
                Style::default()
                    .fg(if is_selected { Color::Green } else { Color::Yellow })
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else if input_mode {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(format!("{} {}", checkbox, tag)).style(style)
        })
        .collect();

    let list_border_style = if input_mode {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let selected_count = host_tags.len();
    let total_count = all_available_tags.len();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(list_border_style)
            .title(if input_mode {
                format!(" Available Tags ({} selected) ", selected_count)
            } else {
                format!(" Available Tags ({}/{} selected - Space/Enter to toggle) ", selected_count, total_count)
            }),
    );

    frame.render_widget(list, chunks[2]);

    // Instructions based on mode
    let instructions_text = if input_mode {
        "Type new tag name │ Enter: Add to global pool (won't auto-assign) │ Esc: Cancel"
    } else {
        "Space/Enter: Toggle tag for this host │ j/k/↑/↓: Navigate │ a/n: Create new tag │ Esc: Done"
    };

    let instructions = Paragraph::new(instructions_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(if input_mode { Color::Green } else { Color::Cyan }))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[3]);
}

/// Render SSH flags selection view
pub fn render_ssh_flags_selection_view(
    frame: &mut Frame,
    selected_flags: &[String],
    selected_index: usize,
    area: Rect,
) {
    use crate::models::get_ssh_flag_options;

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Flag list
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Select SSH Flags")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Flag list
    let available_flags = get_ssh_flag_options();
    let items: Vec<ListItem> = available_flags
        .iter()
        .enumerate()
        .map(|(i, flag_option)| {
            let is_selected = selected_flags.contains(&flag_option.flag.to_string());
            let is_highlighted = i == selected_index;
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let style = if is_highlighted {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let text = format!("{} {} - {}", checkbox, flag_option.flag, flag_option.description);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Available SSH Flags "),
    );

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("j/k/↑/↓: Navigate │ Space/Enter: Toggle │ Esc: Back")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}

/// Render shell selection view
pub fn render_shell_selection_view(
    frame: &mut Frame,
    current_shell: Option<&String>,
    selected_index: usize,
    area: Rect,
) {
    use crate::models::get_shell_options;

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Title
        Constraint::Min(0),     // Shell list
        Constraint::Length(3),  // Instructions
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Select Shell")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Shell list
    let available_shells = get_shell_options();
    let items: Vec<ListItem> = available_shells
        .iter()
        .enumerate()
        .map(|(i, shell_option)| {
            let is_selected = current_shell.map_or(false, |s| s == shell_option.name);
            let is_highlighted = i == selected_index;
            let checkbox = if is_selected { "[●]" } else { "[ ]" };

            let style = if is_highlighted {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let text = format!("{} {} - {}", checkbox, shell_option.name, shell_option.description);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Available Shells "),
    );

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("j/k/↑/↓: Navigate │ Space/Enter: Select/Deselect │ Esc: Back")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}
