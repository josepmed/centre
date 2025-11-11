use ratatui::style::{Color, Modifier, Style};

/// Default text style
pub fn default_style() -> Style {
    Style::default().fg(Color::White)
}

/// Selected row highlight style
pub fn selected_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::LightCyan)
        .add_modifier(Modifier::BOLD)
}

/// Running status badge style
pub fn running_style() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD)
}

/// Paused status badge style
pub fn paused_style() -> Style {
    Style::default().fg(Color::Yellow)
}

/// Idle status badge style
pub fn idle_style() -> Style {
    Style::default().fg(Color::Gray)
}

/// Over-estimate warning style
pub fn over_estimate_style() -> Style {
    Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::BOLD)
}

/// Tree connector style (for subtasks)
pub fn tree_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Title style for panes
pub fn title_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

/// Border style
pub fn border_style() -> Style {
    Style::default().fg(Color::Gray)
}

/// Modal background style
pub fn modal_bg_style() -> Style {
    Style::default().bg(Color::DarkGray).fg(Color::White)
}

/// Modal title style
pub fn modal_title_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

/// Keybinding hint style
pub fn hint_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Plant emoji color (green gradient)
pub fn plant_style() -> Style {
    Style::default().fg(Color::Green)
}

/// Garden gauge style
pub fn gauge_style() -> Style {
    Style::default().fg(Color::Green).bg(Color::DarkGray)
}

/// Error message style
pub fn error_style() -> Style {
    Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::BOLD)
}

/// Done/completed task style
pub fn done_style() -> Style {
    Style::default().fg(Color::Green)
}

/// Tag badge style
pub fn tag_style() -> Style {
    Style::default().fg(Color::Blue)
}
