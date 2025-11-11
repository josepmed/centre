use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::domain::GlobalMode;

const FRAMES_PER_CYCLE: u32 = 12; // 12 frames at 4 FPS = 3 second loop

pub fn render_animation_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let frame_index = (app.animation_frame % FRAMES_PER_CYCLE) as usize;

    let (title, animation_lines) = get_animation_for_mode(&app.global_mode, frame_index);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Center the animation vertically and horizontally
    let animation = Paragraph::new(animation_lines)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(animation, inner);
}

fn get_animation_for_mode(mode: &GlobalMode, frame: usize) -> (String, Vec<Line<'static>>) {
    match mode {
        GlobalMode::Working => working_animation(frame),
        GlobalMode::Break => break_animation(frame),
        GlobalMode::Lunch => lunch_animation(frame),
        GlobalMode::Gym => gym_animation(frame),
        GlobalMode::Dinner => dinner_animation(frame),
        GlobalMode::Personal => personal_animation(frame),
        GlobalMode::Sleep => sleep_animation(frame),
    }
}

fn working_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Working", GlobalMode::Working.symbol());

    // Animated coffee cup with steam
    let steam_frames = [
        vec!["  ) ) )  ", "  ) ) )  "],
        vec!["  ( ( (  ", "  ( ( (  "],
        vec!["  ) ) )  ", "  ) ) )  "],
        vec!["  ( ( (  ", "  ( ( (  "],
    ];

    let steam_idx = frame / 3; // Change every 3 frames
    let steam = &steam_frames[steam_idx % steam_frames.len()];

    let lines = vec![
        Line::from(""),
        Line::from(steam[0]),
        Line::from(steam[1]),
        Line::from("  _____  "),
        Line::from(" |     | "),
        Line::from(" |     | "),
        Line::from(" |_____| "),
        Line::from("  \\___/  "),
        Line::from(""),
        Line::from(Span::styled("  Focus  ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn break_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Break", GlobalMode::Break.symbol());

    // Cloud emojis drifting horizontally at different speeds
    let cloud_positions = [
        ("      ☁️       ", "  ☁️         ", "        ☁️   "),
        ("       ☁️      ", "   ☁️        ", "         ☁️  "),
        ("        ☁️     ", "    ☁️       ", "          ☁️ "),
        ("         ☁️    ", "     ☁️      ", "           ☁️"),
        ("          ☁️   ", "      ☁️     ", "  ☁️         "),
        ("           ☁️  ", "       ☁️    ", "   ☁️        "),
        ("  ☁️           ", "        ☁️   ", "    ☁️       "),
        ("   ☁️          ", "         ☁️  ", "     ☁️      "),
        ("    ☁️         ", "          ☁️ ", "      ☁️     "),
        ("     ☁️        ", "           ☁️", "       ☁️    "),
        ("      ☁️       ", "  ☁️         ", "        ☁️   "),
        ("       ☁️      ", "   ☁️        ", "         ☁️  "),
    ];

    let pos = &cloud_positions[frame % cloud_positions.len()];

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(pos.0),
        Line::from(""),
        Line::from(pos.1),
        Line::from(""),
        Line::from(pos.2),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("  Drift ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn lunch_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Lunch", GlobalMode::Lunch.symbol());

    // Animated steaming plate
    let steam_patterns = ["~", "≈", "~", "≈"];
    let steam = steam_patterns[frame % steam_patterns.len()];

    let lines = vec![
        Line::from(""),
        Line::from(format!(" {}  {}  {} ", steam, steam, steam)),
        Line::from(format!("  {}  {}   ", steam, steam)),
        Line::from(" _______ "),
        Line::from("/       \\"),
        Line::from("|  🍽️   |"),
        Line::from("\\_______/"),
        Line::from(" _______ "),
        Line::from(""),
        Line::from(Span::styled(" Nourish ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn gym_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Gym", GlobalMode::Gym.symbol());

    // Animated dumbbell lift
    let positions = [
        ("    ___    ", "   |   |   ", "===|   |==="),
        ("   \\___/   ", "    | |    ", "====| |===="),
        ("    ___    ", "   |   |   ", "===|   |==="),
        ("  __   __  ", "  ||   ||  ", "==||   ||=="),
    ];

    let pos_idx = frame / 3; // Change every 3 frames
    let pos = &positions[pos_idx % positions.len()];

    let lines = vec![
        Line::from(""),
        Line::from(pos.0),
        Line::from(pos.1),
        Line::from(pos.2),
        Line::from(""),
        Line::from("  /|\\  "),
        Line::from("   |   "),
        Line::from("  / \\  "),
        Line::from(""),
        Line::from(Span::styled(" Strength", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn dinner_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Dinner", GlobalMode::Dinner.symbol());

    // Animated steaming bowl
    let steam_patterns = ["˚", "°", "˚", "°"];
    let steam = steam_patterns[frame % steam_patterns.len()];

    let lines = vec![
        Line::from(""),
        Line::from(format!(" {}  {}  {} ", steam, steam, steam)),
        Line::from(format!("  {}  {}   ", steam, steam)),
        Line::from("  .---.  "),
        Line::from(" /     \\ "),
        Line::from("|  🍲   |"),
        Line::from("|       |"),
        Line::from(" \\_____/ "),
        Line::from(""),
        Line::from(Span::styled(" Evening ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn personal_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Personal", GlobalMode::Personal.symbol());

    // Animated flower blooming - opens and closes gently
    let bloom_states = [
        // Closed bud
        ("       ", "   |   ", "  \\|/  ", "   |   "),
        // Starting to open
        ("       ", "  \\|/  ", "  -●-  ", "   |   "),
        // More open
        ("  \\ /  ", "  -●-  ", "  / \\  ", "   |   "),
        // Fully bloomed
        (" \\ | / ", "  \\●/  ", "  -●-  ", "   |   "),
        // Full bloom with petals
        ("\\ \\|/ /", " --●-- ", "  /|\\  ", "   |   "),
        // Stay bloomed
        ("\\ \\|/ /", " --●-- ", "  /|\\  ", "   |   "),
        ("\\ \\|/ /", " --●-- ", "  /|\\  ", "   |   "),
        // Start closing
        (" \\ | / ", "  \\●/  ", "  -●-  ", "   |   "),
        ("  \\ /  ", "  -●-  ", "  / \\  ", "   |   "),
        ("       ", "  \\|/  ", "  -●-  ", "   |   "),
        // Back to closed
        ("       ", "   |   ", "  \\|/  ", "   |   "),
        ("       ", "   |   ", "  \\|/  ", "   |   "),
    ];

    let state = &bloom_states[frame % bloom_states.len()];

    let lines = vec![
        Line::from(""),
        Line::from(state.0),
        Line::from(state.1),
        Line::from(state.2),
        Line::from(state.3),
        Line::from("   |   "),
        Line::from("   |   "),
        Line::from(""),
        Line::from("  🏡   "),
        Line::from(""),
        Line::from(Span::styled("  Bloom ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}

fn sleep_animation(frame: usize) -> (String, Vec<Line<'static>>) {
    let title = format!("{} Sleep", GlobalMode::Sleep.symbol());

    // Animated twinkling stars and moon
    let star_patterns = [
        ("*", " ", "*"),
        (" ", "*", " "),
        ("*", " ", "*"),
        (" ", "*", " "),
    ];

    let stars = &star_patterns[frame % star_patterns.len()];

    let lines = vec![
        Line::from(""),
        Line::from(format!(" {}     {} ", stars.0, stars.2)),
        Line::from(format!("    {}    ", stars.1)),
        Line::from("   .-.   "),
        Line::from("  (   )  "),
        Line::from("   '-'   "),
        Line::from(format!(" {}  🌙  {} ", stars.0, stars.2)),
        Line::from("   Zzz   "),
        Line::from(""),
        Line::from(Span::styled("  Rest   ", Style::default().add_modifier(Modifier::BOLD))),
    ];

    (title, lines)
}
