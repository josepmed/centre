use crate::app::AppState;
use crate::domain::item::TimeTracking;
use crate::ui::styles::{border_style, default_style, gauge_style, title_style};
use chrono::Duration;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Format duration as "Xh Ym" or "Xm" for short durations
fn format_duration(duration: Duration) -> String {
    let total_mins = duration.num_minutes();
    if total_mins < 60 {
        format!("{}m", total_mins)
    } else {
        let hours = total_mins / 60;
        let mins = total_mins % 60;
        if mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, mins)
        }
    }
}

/// Format time as "HH:MM"
fn format_time(dt: chrono::DateTime<chrono::Local>) -> String {
    dt.format("%H:%M").to_string()
}

/// Render the Focus Garden pane with quantitative time dashboard
pub fn render_garden_pane(f: &mut Frame, app: &mut AppState, area: Rect) {
    let (total_elapsed, total_estimate) = app.get_totals();

    // Calculate growth percentage
    let growth_pct = if total_estimate > Duration::zero() {
        (total_elapsed.num_seconds() as f64 / total_estimate.num_seconds() as f64 * 100.0)
    } else {
        0.0
    };

    // Get metrics
    let running_time = app.get_running_tasks_time();
    let paused_time = app.get_paused_tasks_time();
    let idle_time = app.get_idle_tasks_time();

    let over_estimate_time = app.get_over_estimate_time();
    let over_estimate_count = app.get_over_estimate_count();
    let efficiency = app.get_efficiency();

    let remaining = app.get_remaining_time();
    let projected_finish = app.get_projected_finish();

    // Get encouragement phrase
    let phrase = app.get_encouragement_phrase();

    // Calculate percentages for running/paused based on estimate
    let running_pct = if total_estimate > Duration::zero() {
        (running_time.num_seconds() as f64 / total_estimate.num_seconds() as f64 * 100.0)
    } else {
        0.0
    };
    let paused_pct = if total_estimate > Duration::zero() {
        (paused_time.num_seconds() as f64 / total_estimate.num_seconds() as f64 * 100.0)
    } else {
        0.0
    };

    let over_estimate_pct = if total_elapsed > Duration::zero() {
        (over_estimate_time.num_seconds() as f64 / total_elapsed.num_seconds() as f64 * 100.0)
    } else {
        0.0
    };

    let remaining_pct = if total_estimate > Duration::zero() {
        (remaining.num_seconds() as f64 / total_estimate.num_seconds() as f64 * 100.0)
    } else {
        0.0
    };

    // Create gauge for growth
    let gauge_capped = growth_pct.min(100.0);
    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(gauge_style())
        .percent(gauge_capped as u16)
        .label("");

    // Build lines of text
    let mut lines = Vec::new();

    // Line 1: Growth gauge (will be rendered separately)
    // Line 2: Growth text
    lines.push(Line::from(vec![
        Span::styled("Growth: ", title_style()),
        Span::raw(format!(
            "{} / {} ({:.0}%)",
            format_duration(total_elapsed),
            format_duration(total_estimate),
            growth_pct
        )),
    ]));

    // Line 3: Running / Paused distribution
    lines.push(Line::from(vec![
        Span::styled("Running: ", title_style()),
        Span::raw(format!("{} ({:.0}%)   ", format_duration(running_time), running_pct)),
        Span::styled("Paused: ", title_style()),
        Span::raw(format!("{} ({:.0}%)", format_duration(paused_time), paused_pct)),
    ]));

    // Line 4: Daily Idle Time (wall-clock time in IDLE state)
    lines.push(Line::from(vec![
        Span::styled("Daily Idle Time: ", title_style()),
        Span::raw(format_duration(idle_time)),
    ]));

    // Line 5: Over-estimate and Efficiency
    let over_estimate_text = if over_estimate_count > 0 {
        format!(
            "{} ({:.0}%) across {} task{}  ",
            format_duration(over_estimate_time),
            over_estimate_pct,
            over_estimate_count,
            if over_estimate_count == 1 { "" } else { "s" }
        )
    } else {
        "None  ".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled("Over-estimate: ", title_style()),
        Span::raw(over_estimate_text),
        Span::styled("Efficiency: ", title_style()),
        Span::raw(format!("{:.0}%", efficiency)),
    ]));

    // Line 6: Remaining time with finish time (using projected if available, else simple)
    let finish_time_text = if let Some(finish_time) = projected_finish {
        format!("→ {}", format_time(finish_time))
    } else {
        let now = chrono::Local::now();
        let simple_finish = now + remaining;
        format!("→ {}", format_time(simple_finish))
    };

    lines.push(Line::from(vec![
        Span::styled("Remaining: ", title_style()),
        Span::raw(format!(
            "{} ({:.0}%) ",
            format_duration(remaining),
            remaining_pct
        )),
        Span::raw(finish_time_text),
    ]));

    // Line 7: Streak (placeholder for future implementation)
    lines.push(Line::from(vec![
        Span::styled("Streak: ", title_style()),
        Span::raw("-- (placeholder for future)"),
    ]));

    // Line 8: Mode time tracking (show all modes with any time)
    let mode_times = app.get_mode_times();
    let total_time: Duration = mode_times.iter().map(|(_, time)| *time).sum();

    // Only show mode times if we have at least 1 minute accumulated
    if total_time.num_minutes() > 0 {
        let mut mode_spans = Vec::new();
        let mut first = true;

        for (mode, time) in &mode_times {
            // Only show modes with at least 1 minute
            if time.num_minutes() > 0 {
                if !first {
                    mode_spans.push(Span::raw(" | "));
                }
                first = false;

                mode_spans.push(Span::raw(format!(
                    "{} {} {}",
                    mode.symbol(),
                    mode.name(),
                    format_duration(*time)
                )));
            }
        }

        if !mode_spans.is_empty() {
            lines.push(Line::from(mode_spans));
        }
    }

    // Create block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style())
        .title(Span::styled(" Focus Garden 🌼 ", title_style()));

    // Layout: gauge at top, text lines in middle, phrase at bottom
    use ratatui::layout::{Constraint, Direction, Layout, Alignment};
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Gauge
            Constraint::Length(1), // Spacing
            Constraint::Min(8),    // Text lines (now 8 lines to include mode times)
            Constraint::Length(1), // Spacing before phrase
            Constraint::Length(1), // Encouragement phrase
        ])
        .split(block.inner(area));

    // Render main content
    f.render_widget(block, area);
    f.render_widget(gauge, chunks[0]);
    f.render_widget(Paragraph::new(lines), chunks[2]);

    // Render centered phrase at bottom - use contextual phrase for non-working modes
    let display_phrase = if app.global_mode.should_pause_timers() {
        app.global_mode.contextual_phrase()
    } else {
        phrase
    };

    let phrase_line = if !display_phrase.is_empty() {
        Line::from(vec![Span::raw(format!("\"{}\"", display_phrase))])
    } else {
        Line::from(vec![Span::raw(phrase)])
    };
    let phrase_paragraph = Paragraph::new(phrase_line).alignment(Alignment::Center);
    f.render_widget(phrase_paragraph, chunks[4]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::minutes(30)), "30m");
        assert_eq!(format_duration(Duration::minutes(60)), "1h");
        assert_eq!(format_duration(Duration::minutes(90)), "1h 30m");
        assert_eq!(format_duration(Duration::minutes(125)), "2h 5m");
    }
}
