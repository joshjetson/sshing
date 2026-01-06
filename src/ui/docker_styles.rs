use ratatui::style::{Color, Modifier, Style};

// Colors - match dockering exactly
pub const COLOR_SELECTED: Color = Color::Cyan;
pub const COLOR_HEADER: Color = Color::Yellow;
pub const COLOR_RUNNING: Color = Color::Green;
pub const COLOR_STOPPED: Color = Color::Red;
pub const COLOR_PAUSED: Color = Color::Yellow;
pub const COLOR_ERROR: Color = Color::Red;
pub const COLOR_STATUS: Color = Color::Green;
pub const COLOR_MUTED: Color = Color::DarkGray;
pub const COLOR_ACCENT: Color = Color::Magenta;

// Styles
pub fn style_default() -> Style {
    Style::default()
}

pub fn style_header() -> Style {
    Style::default()
        .fg(COLOR_HEADER)
        .add_modifier(Modifier::BOLD)
}

pub fn style_selected() -> Style {
    Style::default()
        .fg(COLOR_SELECTED)
        .add_modifier(Modifier::BOLD)
}

pub fn style_running() -> Style {
    Style::default().fg(COLOR_RUNNING)
}

pub fn style_stopped() -> Style {
    Style::default().fg(COLOR_STOPPED)
}

pub fn style_paused() -> Style {
    Style::default().fg(COLOR_PAUSED)
}

pub fn style_muted() -> Style {
    Style::default().fg(COLOR_MUTED)
}

pub fn style_status() -> Style {
    Style::default().fg(COLOR_STATUS)
}

pub fn style_error() -> Style {
    Style::default().fg(COLOR_ERROR)
}

pub fn style_accent() -> Style {
    Style::default().fg(COLOR_ACCENT)
}

#[allow(dead_code)]
pub fn style_edit_mode() -> Style {
    Style::default()
        .fg(COLOR_RUNNING)
        .add_modifier(Modifier::BOLD)
}

pub fn style_editing() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(COLOR_SELECTED)
}

#[allow(dead_code)]
pub fn style_nav_mode() -> Style {
    Style::default().fg(COLOR_SELECTED)
}
