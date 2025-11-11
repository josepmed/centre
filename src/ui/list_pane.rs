use crate::app::AppState;
use crate::domain::{flatten_tasks, plant_glyph, status_badge, tree_connector, Item, RunStatus, TimeTracking};
use crate::ui::styles::{
    border_style, default_style, idle_style, over_estimate_style, paused_style, running_style,
    selected_style, title_style, tree_style, tag_style,
};
use chrono::{DateTime, Local, Timelike};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Helper to format duration for display
fn format_duration(duration: chrono::Duration) -> String {
    let total_minutes = duration.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 && minutes > 0 {
        format!("{}h {}m", hours, minutes)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", minutes)
    }
}

/// Calculate ETAs for all tasks and subtasks sequentially
fn calculate_etas(tasks: &[Item]) -> HashMap<Uuid, DateTime<Local>> {
    let mut etas = HashMap::new();
    let mut accumulated_time = chrono::Duration::zero();
    let now = Local::now();

    for task in tasks {
        // Calculate remaining time for task
        let remaining = task.track.estimate - task.track.elapsed;

        if task.status == RunStatus::Running {
            // If running, ETA is now + remaining
            let eta = snap_to_5min(now + remaining);
            etas.insert(task.id, eta);
        } else {
            // If idle/paused, accumulate previous times
            let eta = snap_to_5min(now + accumulated_time + remaining);
            etas.insert(task.id, eta);
        }

        // Process subtasks
        if !task.subtasks.is_empty() {
            let mut subtask_accumulated = chrono::Duration::zero();

            for subtask in &task.subtasks {
                let subtask_remaining = subtask.track.estimate - subtask.track.elapsed;

                if subtask.status == RunStatus::Running {
                    // Subtask running: ETA is now + remaining
                    let eta = snap_to_5min(now + subtask_remaining);
                    etas.insert(subtask.id, eta);
                } else {
                    // Subtask idle/paused: accumulate within task
                    let eta = snap_to_5min(now + subtask_accumulated + subtask_remaining);
                    etas.insert(subtask.id, eta);
                }

                // Add to subtask accumulation
                subtask_accumulated = subtask_accumulated + subtask_remaining;
            }

            // Use subtask total for parent accumulation
            accumulated_time = accumulated_time + subtask_accumulated;
        } else {
            // No subtasks, accumulate task time
            if task.status != RunStatus::Running {
                accumulated_time = accumulated_time + remaining;
            }
        }
    }

    etas
}

/// Snap time to nearest 5-minute increment
fn snap_to_5min(time: DateTime<Local>) -> DateTime<Local> {
    let minutes = time.minute();
    let snapped_minutes = ((minutes + 2) / 5) * 5; // Round to nearest 5

    time.with_minute(snapped_minutes % 60)
        .and_then(|t| {
            if snapped_minutes >= 60 {
                t.with_hour((time.hour() + 1) % 24)
            } else {
                Some(t)
            }
        })
        .unwrap_or(time)
}

/// Get phase emoji based on hour of day
fn phase_emoji(time: DateTime<Local>) -> &'static str {
    let hour = time.hour();
    match hour {
        5..=11 => "🌅",   // Morning
        12..=17 => "🌞",  // Afternoon
        18..=20 => "🌇",  // Evening
        21..=23 | 0..=4 => "🌙", // Night
        _ => "🌞",
    }
}

/// Format time as HH:MM
fn format_time(time: DateTime<Local>) -> String {
    time.format("%H:%M").to_string()
}

/// Render the "Today's Focus" list pane
pub fn render_list_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let flat_rows = flatten_tasks(&app.tasks);

    // Calculate ETAs for all tasks/subtasks
    let etas = calculate_etas(&app.tasks);

    let items: Vec<ListItem> = flat_rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let item = if let Some(st_idx) = row.subtask_index {
                &app.tasks[row.task_index].subtasks[st_idx]
            } else {
                &app.tasks[row.task_index]
            };

            let eta = etas.get(&item.id).copied();
            let line = create_task_line(item, row.depth, row.is_last, app.use_emoji, eta);
            let style = if idx == app.selected_index {
                selected_style()
            } else {
                default_style()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let date = Local::now().format("%a %b %d");
    let title = format!(" Today's Centre 🌱 ({}) — {} {} ", date, app.global_mode.symbol(), app.global_mode.name());

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style())
            .title(Span::styled(title, title_style())),
    );

    f.render_widget(list, area);
}

/// Create a single line for a task/subtask
/// Format: [🌿] Write proposal  ⏱ 1.3h / 2.0h (RUNNING) [TAGS]   ⇢ 🕒 12:45 🌞
fn create_task_line(item: &Item, depth: usize, is_last: bool, use_emoji: bool, eta: Option<DateTime<Local>>) -> Line<'static> {
    let mut spans = Vec::new();

    // Indentation and tree connector for subtasks
    if depth > 0 {
        spans.push(Span::styled("     ".to_string(), tree_style()));
        spans.push(Span::styled(
            tree_connector(is_last).to_string(),
            tree_style(),
        ));
        spans.push(Span::raw(" ".to_string()));
    }

    // Plant glyph
    let ratio = item.track.progress_ratio();
    let plant = plant_glyph(ratio, use_emoji);
    spans.push(Span::raw(format!("[{}] ", plant)));

    // Title
    spans.push(Span::raw(item.title.clone()));

    // Padding
    spans.push(Span::raw("  ".to_string()));

    // Time info with clock emoji
    let time_str = format!(
        "⏱ {} / {} ",
        item.track.elapsed_formatted(),
        item.track.estimate_formatted()
    );
    spans.push(Span::raw(time_str));

    // Status badge
    let badge = status_badge(item);
    let badge_style = match item.status {
        RunStatus::Running => running_style(),
        RunStatus::Paused => paused_style(),
        _ => idle_style(),
    };
    spans.push(Span::styled(badge.to_string(), badge_style));

    // ETA with phase emoji if available
    if let Some(eta_time) = eta {
        let phase = phase_emoji(eta_time);
        spans.push(Span::raw(format!(" • {} {} ", format_time(eta_time), phase)));
    }

    // Tags (if any)
    if !item.tags.is_empty() {
        spans.push(Span::raw(" ".to_string()));
        for (i, tag) in item.tags.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" ".to_string()));
            }
            spans.push(Span::styled(format!("[{}]", tag), tag_style()));
        }
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ScheduleDay;
    use chrono::Duration;

    #[test]
    fn test_create_task_line() {
        let item = Item::new(
            "Test task".to_string(),
            Duration::hours(2),
            ScheduleDay::Today,
        );
        let line = create_task_line(&item, 0, false, true, None);

        // Check that line contains expected components
        let line_str = format!("{:?}", line);
        assert!(line_str.contains("Test task"));
    }

    #[test]
    fn test_create_subtask_line() {
        let item = Item::new(
            "Subtask".to_string(),
            Duration::hours(1),
            ScheduleDay::Today,
        );
        let line = create_task_line(&item, 1, true, true, None);

        // Subtask should have indentation
        let line_str = format!("{:?}", line);
        assert!(line_str.contains("Subtask"));
    }
}
