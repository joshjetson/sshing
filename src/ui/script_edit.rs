use ratatui::{
    layout::{Constraint, Layout, Rect, Alignment},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear},
    Frame,
};

use crate::app::App;
use crate::models::{AppMode, ScriptSection};
use super::docker_styles as styles;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Main content
        Constraint::Length(3),  // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_editor(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (script_name, container_name) = match &app.mode {
        AppMode::ScriptEdit { editing_script, .. } => {
            (editing_script.path.clone(), editing_script.container_name.clone())
        }
        _ => (String::new(), String::new()),
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Script Editor ", styles::style_header()),
        Span::styled("│ ", styles::style_muted()),
        Span::styled(&container_name, styles::style_accent()),
        Span::styled(" │ ", styles::style_muted()),
        Span::styled(&script_name, styles::style_muted()),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let (editing_script, focused_section, selected_index, editing_mode) = match &app.mode {
        AppMode::ScriptEdit { editing_script, focused_section, selected_index, editing_mode, .. } => {
            (editing_script, *focused_section, *selected_index, *editing_mode)
        }
        _ => return,
    };

    // Split into sections: tabs at top, content below
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Section tabs
        Constraint::Min(0),     // Section content
    ])
    .split(area);

    // Render section tabs
    let tabs = vec![
        ("Env Vars", ScriptSection::EnvVars, editing_script.env_vars.len()),
        ("Ports", ScriptSection::Ports, editing_script.ports.len()),
        ("Volumes", ScriptSection::Volumes, editing_script.volumes.len()),
        ("Network", ScriptSection::Network, 1),
    ];

    let tab_spans: Vec<Span> = tabs
        .iter()
        .flat_map(|(name, section, count)| {
            let style = if *section == focused_section {
                styles::style_selected()
            } else {
                styles::style_muted()
            };
            vec![
                Span::styled(format!(" {} ({}) ", name, count), style),
                Span::styled("│", styles::style_muted()),
            ]
        })
        .collect();

    let tabs_line = Paragraph::new(Line::from(tab_spans))
        .block(Block::default().borders(Borders::ALL).title(" Sections [Tab] to switch "));
    frame.render_widget(tabs_line, chunks[0]);

    // Render section content
    match focused_section {
        ScriptSection::EnvVars => render_env_vars(frame, editing_script, selected_index, editing_mode, chunks[1]),
        ScriptSection::Ports => render_ports(frame, editing_script, selected_index, chunks[1]),
        ScriptSection::Volumes => render_volumes(frame, editing_script, selected_index, chunks[1]),
        ScriptSection::Network => render_network(frame, editing_script, chunks[1]),
    }
}

fn render_env_vars(
    frame: &mut Frame,
    script: &crate::models::DeploymentScript,
    selected: usize,
    editing: bool,
    area: Rect,
) {
    if script.env_vars.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No environment variables", styles::style_muted())),
            Line::from(""),
            Line::from(Span::styled("Press [a] to add one", styles::style_accent())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Environment Variables "))
        .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    // Calculate visible rows (area height - 2 for borders - 1 for header)
    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_vars = script.env_vars.len();

    // Compute scroll_offset from selected index to keep selection visible
    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected < visible_rows / 2 {
        0
    } else if selected >= total_vars.saturating_sub(visible_rows / 2) {
        total_vars.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };

    let header_cells = ["", "Key", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    // Only render visible rows
    let end_index = (scroll_offset + visible_rows).min(total_vars);
    let visible_vars = &script.env_vars[scroll_offset..end_index];

    let rows: Vec<Row> = visible_vars
        .iter()
        .enumerate()
        .map(|(i, env)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected { "▸" } else { " " };
            let value_display = if env.is_secret {
                "••••••••••••".to_string()
            } else {
                env.value.clone()
            };

            let style = if actual_index == selected {
                if editing {
                    styles::style_editing()
                } else {
                    styles::style_selected()
                }
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(env.key.clone()),
                Cell::from(value_display),
            ])
            .style(style)
        })
        .collect();

    // Show scroll position in title if scrolling
    let title = if total_vars > visible_rows {
        format!(" Environment Variables ({}-{} of {}) ",
            scroll_offset + 1,
            end_index,
            total_vars
        )
    } else {
        format!(" Environment Variables ({}) ", total_vars)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(30),
            Constraint::Percentage(68),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(table, area);
}

fn render_ports(
    frame: &mut Frame,
    script: &crate::models::DeploymentScript,
    selected: usize,
    area: Rect,
) {
    if script.ports.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No port mappings", styles::style_muted())),
            Line::from(""),
            Line::from(Span::styled("Press [a] to add one", styles::style_accent())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Port Mappings "))
        .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_ports = script.ports.len();

    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected < visible_rows / 2 {
        0
    } else if selected >= total_ports.saturating_sub(visible_rows / 2) {
        total_ports.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };

    let header_cells = ["", "Host Port", "Container Port", "Protocol"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    let end_index = (scroll_offset + visible_rows).min(total_ports);
    let visible_ports = &script.ports[scroll_offset..end_index];

    let rows: Vec<Row> = visible_ports
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected { "▸" } else { " " };
            let style = if actual_index == selected {
                styles::style_selected()
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(port.host_port.to_string()),
                Cell::from(port.container_port.to_string()),
                Cell::from(port.protocol.clone()),
            ])
            .style(style)
        })
        .collect();

    let title = if total_ports > visible_rows {
        format!(" Port Mappings ({}-{} of {}) ",
            scroll_offset + 1,
            end_index,
            total_ports
        )
    } else {
        format!(" Port Mappings ({}) ", total_ports)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(33),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_volumes(
    frame: &mut Frame,
    script: &crate::models::DeploymentScript,
    selected: usize,
    area: Rect,
) {
    if script.volumes.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No volume mounts", styles::style_muted())),
            Line::from(""),
            Line::from(Span::styled("Press [a] to add one", styles::style_accent())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Volume Mounts "))
        .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let visible_rows = area.height.saturating_sub(3) as usize;
    let total_volumes = script.volumes.len();

    let scroll_offset = if visible_rows == 0 {
        0
    } else if selected < visible_rows / 2 {
        0
    } else if selected >= total_volumes.saturating_sub(visible_rows / 2) {
        total_volumes.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };

    let header_cells = ["", "Host Path", "Container Path", "RO"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::style_header()));
    let header = Row::new(header_cells).height(1);

    let end_index = (scroll_offset + visible_rows).min(total_volumes);
    let visible_volumes = &script.volumes[scroll_offset..end_index];

    let rows: Vec<Row> = visible_volumes
        .iter()
        .enumerate()
        .map(|(i, vol)| {
            let actual_index = scroll_offset + i;
            let marker = if actual_index == selected { "▸" } else { " " };
            let ro = if vol.read_only { "yes" } else { "no" };
            let style = if actual_index == selected {
                styles::style_selected()
            } else {
                styles::style_default()
            };

            Row::new(vec![
                Cell::from(marker),
                Cell::from(vol.host_path.clone()),
                Cell::from(vol.container_path.clone()),
                Cell::from(ro),
            ])
            .style(style)
        })
        .collect();

    let title = if total_volumes > visible_rows {
        format!(" Volume Mounts ({}-{} of {}) ",
            scroll_offset + 1,
            end_index,
            total_volumes
        )
    } else {
        format!(" Volume Mounts ({}) ", total_volumes)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(40),
            Constraint::Percentage(45),
            Constraint::Percentage(13),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_network(frame: &mut Frame, script: &crate::models::DeploymentScript, area: Rect) {
    let network_display = script.network.clone().unwrap_or_else(|| "default".to_string());

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Network: ", styles::style_header()),
            Span::styled(&network_display, styles::style_accent()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Repo: ", styles::style_header()),
            Span::styled(&script.repo, styles::style_default()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Container Name: ", styles::style_header()),
            Span::styled(&script.container_name, styles::style_default()),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Network & Settings "));

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let (focused_section, editing_mode) = match &app.mode {
        AppMode::ScriptEdit { focused_section, editing_mode, .. } => (*focused_section, *editing_mode),
        _ => return,
    };

    let help = if editing_mode {
        "[Esc] Cancel  [Enter] Save field"
    } else {
        match focused_section {
            ScriptSection::EnvVars => "[a]dd [Enter]Edit [d]elete [Tab]Switch section [Ctrl+S]Save [Esc]Back",
            ScriptSection::Ports => "[a]dd [d]elete [Tab]Switch section [Ctrl+S]Save [Esc]Back",
            ScriptSection::Volumes => "[a]dd [d]elete [Tab]Switch section [Ctrl+S]Save [Esc]Back",
            ScriptSection::Network => "[Tab]Switch section [Ctrl+S]Save [Esc]Back",
        }
    };

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

/// Render an input dialog overlay for adding/editing env vars
pub fn render_env_var_dialog(frame: &mut Frame, app: &App) {
    if let AppMode::EnvVarEditor { key_buffer, value_buffer, editing_key, is_new, .. } = &app.mode {
        let area = frame.area();

        // Center the dialog
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 10;
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear the area first
        frame.render_widget(Clear, dialog_area);

        let title = if *is_new { " Add Environment Variable " } else { " Edit Environment Variable " };

        let key_style = if *editing_key { styles::style_editing() } else { styles::style_default() };
        let value_style = if !*editing_key { styles::style_editing() } else { styles::style_default() };

        let content = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Key:   ", styles::style_header()),
                Span::styled(key_buffer.as_str(), key_style),
                if *editing_key { Span::styled("▏", styles::style_accent()) } else { Span::raw("") },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Value: ", styles::style_header()),
                Span::styled(value_buffer.as_str(), value_style),
                if !*editing_key { Span::styled("▏", styles::style_accent()) } else { Span::raw("") },
            ]),
            Line::from(""),
            Line::from(Span::styled("  [Tab] Switch field  [Enter] Save  [Esc] Cancel", styles::style_muted())),
        ];

        let dialog = Paragraph::new(content)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(styles::style_accent()));

        frame.render_widget(dialog, dialog_area);
    }
}
