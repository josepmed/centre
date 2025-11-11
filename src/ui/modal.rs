use crate::app::AppState;
use crate::domain::{GlobalMode, UiMode};
use crate::ui::{
    layout::create_modal_area,
    styles::{modal_bg_style, modal_title_style},
};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the day changed modal (forces restart)
pub fn render_day_changed_modal(f: &mut Frame, app: &AppState, area: Rect) {
    if app.ui_mode == UiMode::DayChanged {
        let modal_area = create_modal_area(area);

        // Clear the area behind the modal
        f.render_widget(Clear, modal_area);

        let mut lines = Vec::new();

        // Message
        lines.push(Line::raw(""));
        lines.push(Line::raw("  A new day has begun!"));
        lines.push(Line::raw(""));
        lines.push(Line::raw("  The date has changed since you started the app."));
        lines.push(Line::raw("  Please close and restart Centre to continue."));
        lines.push(Line::raw(""));
        lines.push(Line::raw("  Your work has been saved."));
        lines.push(Line::raw(""));

        // Options
        lines.push(Line::from(vec![
            Span::styled("  [q]", modal_title_style()),
            Span::raw(" Close Centre  "),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " \u{1F305} Day Changed ",
                        modal_title_style(),
                    ))
                    .style(modal_bg_style()),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, modal_area);
    }
}

/// Render the estimate-hit modal
pub fn render_modal(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(modal) = &app.modal {
        let modal_area = create_modal_area(area);

        // Clear the area behind the modal
        f.render_widget(Clear, modal_area);

        let mut lines = Vec::new();

        // Message
        lines.push(Line::raw(""));
        lines.push(Line::raw(&modal.message));
        lines.push(Line::raw(""));
        lines.push(Line::raw("What would you like to do?"));
        lines.push(Line::raw(""));

        // Options
        lines.push(Line::from(vec![
            Span::styled("[d]", modal_title_style()),
            Span::raw(" Done  "),
            Span::styled("[e]", modal_title_style()),
            Span::raw(" Extend  "),
        ]));
        lines.push(Line::from(vec![
            Span::styled("[s]", modal_title_style()),
            Span::raw(" Pause  "),
            Span::styled("[t]", modal_title_style()),
            Span::raw(" Tomorrow  "),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " ⏱ Estimate Reached ",
                        modal_title_style(),
                    ))
                    .style(modal_bg_style()),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, modal_area);
    }
}

/// Render the idle check modal
pub fn render_idle_check_modal(f: &mut Frame, app: &AppState, area: Rect) {
    if app.ui_mode == UiMode::IdleCheck {
        let modal_area = create_modal_area(area);

        // Clear the area behind the modal
        f.render_widget(Clear, modal_area);

        let mut lines = Vec::new();

        // Calculate time remaining
        let time_left = if let Some(deadline) = app.idle_check_deadline {
            let now = std::time::Instant::now();
            if deadline > now {
                let secs = (deadline - now).as_secs();
                format!("{} minutes {} seconds", secs / 60, secs % 60)
            } else {
                String::from("0 seconds")
            }
        } else {
            String::from("30 minutes")
        };

        // Message
        lines.push(Line::raw(""));
        lines.push(Line::raw("Are you still working on your tasks?"));
        lines.push(Line::raw(""));
        lines.push(Line::raw(format!("Time remaining: {}", time_left)));
        lines.push(Line::raw(""));
        lines.push(Line::raw("If you don't confirm, all running tasks will be paused."));
        lines.push(Line::raw(""));

        // Options
        lines.push(Line::from(vec![
            Span::styled("[y]", modal_title_style()),
            Span::raw(" Yes, I'm still working  "),
        ]));
        lines.push(Line::from(vec![
            Span::styled("[n]", modal_title_style()),
            Span::raw(" No, pause everything  "),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " ⏰ Idle Check ",
                        modal_title_style(),
                    ))
                    .style(modal_bg_style()),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, modal_area);
    }
}

/// Render the mode selector modal
pub fn render_mode_selector(f: &mut Frame, app: &AppState, area: Rect) {
    if app.ui_mode == UiMode::ModeSelector {
        let modal_area = create_modal_area(area);

        // Clear the area behind the modal
        f.render_widget(Clear, modal_area);

        let mut lines = Vec::new();

        // Title and instructions
        lines.push(Line::raw(""));
        lines.push(Line::raw("  Select your current status:"));
        lines.push(Line::raw(""));

        // Mode options with keys
        let modes = GlobalMode::all();
        let keys = ['1', '2', '3', '4', '5', '6'];

        for (idx, mode) in modes.iter().enumerate() {
            let key = keys[idx];
            let is_current = *mode == app.global_mode;

            let line = if is_current {
                Line::from(vec![
                    Span::styled(format!("  [{}] ", key), modal_title_style()),
                    Span::raw(format!("{} ", mode.symbol())),
                    Span::styled(mode.name(), modal_title_style()),
                    Span::raw(" ← Current"),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!("  [{}] ", key), modal_title_style()),
                    Span::raw(format!("{} {}", mode.symbol(), mode.name())),
                ])
            };
            lines.push(line);
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  [Esc]", modal_title_style()),
            Span::raw(" Cancel"),
        ]));
        lines.push(Line::raw(""));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " Set Status ",
                        modal_title_style(),
                    ))
                    .style(modal_bg_style()),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, modal_area);
    }
}
