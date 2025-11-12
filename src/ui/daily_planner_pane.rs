use crate::app::AppState;
use crate::domain::RunStatus;
use crate::ui::styles::{idle_style, paused_style, running_style};
use chrono::{Duration, Local, NaiveTime, Timelike};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;

/// Time slice in minutes (15-minute intervals)
const SLICE_MINUTES: i64 = 15;

/// Start hour for planner (9am)
const START_HOUR: u32 = 9;

/// End hour for planner (12am = midnight = 24)
const END_HOUR: u32 = 24;

/// Task block scheduled for the planner
#[derive(Debug, Clone)]
struct TaskBlock {
    task_id: Uuid,
    title: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    status: RunStatus,
    is_selected: bool,
    color: Color,
}

/// A time slice with tasks occupying it
#[derive(Debug, Clone)]
struct TimeSlice {
    time: NaiveTime,
    tasks: Vec<TaskBlock>,
}

/// Render the daily planner pane
pub fn render_daily_planner_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let now = Local::now();
    let current_time = now.time();

    // Calculate time grid from now until midnight
    let slices = build_time_grid(current_time);

    // Schedule all active tasks into time blocks
    let task_blocks = schedule_tasks(app, current_time);

    // Render the planner (no title line - it's in the Block border)
    let mut all_lines = Vec::new();

    let content_width = area.width.saturating_sub(9) as usize; // 9 for "HH:MM | " + "|"

    // Build all lines first (we'll slice later for scrolling)
    for (_idx, slice) in slices.iter().enumerate() {
        let slice_mins = slice.hour() as i64 * 60 + slice.minute() as i64;
        let next_slice_mins = slice_mins + SLICE_MINUTES;

        // Check if this is the current time slot
        let current_mins = current_time.hour() as i64 * 60 + current_time.minute() as i64;
        let is_current_slot = current_mins >= slice_mins && current_mins < next_slice_mins;

        // Find ALL blocks that occupy this time slot
        let mut slot_blocks: Vec<&TaskBlock> = Vec::new();
        for block in task_blocks.iter() {
            let block_start = block.start_time.hour() as i64 * 60 + block.start_time.minute() as i64;
            let block_end = block.end_time.hour() as i64 * 60 + block.end_time.minute() as i64;

            // Check if this slot overlaps with the block
            if block_start < next_slice_mins && block_end > slice_mins {
                slot_blocks.push(block);
            }
        }

        let time_label = format!("{:02}:{:02}", slice.hour(), slice.minute());
        let time_style = if is_current_slot {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        if slot_blocks.is_empty() {
            // No blocks in this slot - single empty line
            all_lines.push(Line::from(Span::styled(
                format!("{} │{}│", time_label, " ".repeat(content_width)),
                time_style,
            )));
        } else {
            // First line: time label with first task (left-aligned)
            let first_task = &slot_blocks[0];
            let task_style = get_task_style(first_task.status, is_current_slot);
            let task_text = truncate_string(&first_task.title, content_width);
            let padding = content_width.saturating_sub(task_text.len());
            let right_padding = format!("{}│", " ".repeat(padding.saturating_add(1)));

            all_lines.push(Line::from(vec![
                Span::styled(time_label.clone(), time_style),
                Span::raw(" │ "),
                Span::styled(task_text, task_style),
                Span::raw(right_padding),
            ]));

            // Additional lines: one per additional task (left-aligned with indent)
            for task in slot_blocks.iter().skip(1) {
                let task_style = get_task_style(task.status, is_current_slot);
                let task_text = truncate_string(&task.title, content_width);
                let padding = content_width.saturating_sub(task_text.len());
                let right_padding = format!("{}│", " ".repeat(padding.saturating_add(1)));

                all_lines.push(Line::from(vec![
                    Span::raw("      │ "),
                    Span::styled(task_text, task_style),
                    Span::raw(right_padding),
                ]));
            }
        }
    }

    // Add legend at bottom
    if !task_blocks.is_empty() {
        all_lines.push(Line::from(""));
        let legend = build_legend(&task_blocks);
        all_lines.push(Line::from(legend));
    }

    // Apply scroll offset - show only the visible portion
    let available_height = area.height.saturating_sub(2) as usize; // Subtract border
    let scroll_offset = app.planner_scroll_offset.min(all_lines.len().saturating_sub(available_height));
    let visible_lines: Vec<Line> = all_lines
        .into_iter()
        .skip(scroll_offset)
        .take(available_height)
        .collect();

    let paragraph = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL).title("Daily Planner 🕒"));

    f.render_widget(paragraph, area);
}

/// Build time grid from 9am to midnight in 15-minute intervals
fn build_time_grid(_current_time: NaiveTime) -> Vec<NaiveTime> {
    let mut slices = Vec::new();

    // Create 30-minute slices from START_HOUR to END_HOUR
    let start_minutes = (START_HOUR * 60) as i64;
    let end_minutes = (END_HOUR * 60) as i64;

    let mut current_minutes = start_minutes;
    while current_minutes < end_minutes {
        let hours = (current_minutes / 60) as u32;
        let mins = (current_minutes % 60) as u32;
        if let Some(time) = NaiveTime::from_hms_opt(hours, mins, 0) {
            slices.push(time);
        }
        current_minutes += SLICE_MINUTES;
    }

    slices
}

/// Schedule tasks into time blocks based on their remaining time estimates
fn schedule_tasks(app: &AppState, current_time: NaiveTime) -> Vec<TaskBlock> {
    let mut blocks = Vec::new();

    // Start time accumulator
    let mut accumulated_minutes = current_time.hour() as i64 * 60 + current_time.minute() as i64;

    // Process all tasks and subtasks
    for (task_idx, task) in app.tasks.iter().enumerate() {
        let is_task_selected = task_idx == app.selected_index;

        // If task has no subtasks, schedule it
        if task.subtasks.is_empty() {
            // Only schedule tasks that have remaining time
            let remaining = task.track.estimate - task.track.elapsed;
            if remaining > Duration::zero() {
                let remaining_minutes = remaining.num_minutes();
                if remaining_minutes > 0 {
                    // Calculate start and end times
                    let start_hours = (accumulated_minutes / 60) as u32;
                    let start_mins = (accumulated_minutes % 60) as u32;
                    let start_time = NaiveTime::from_hms_opt(start_hours.min(23), start_mins.min(59), 0)
                        .unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 0).unwrap());

                    accumulated_minutes += remaining_minutes;

                    // Cap at midnight
                    if accumulated_minutes >= 24 * 60 {
                        accumulated_minutes = 24 * 60 - 1;
                    }

                    let end_hours = (accumulated_minutes / 60) as u32;
                    let end_mins = (accumulated_minutes % 60) as u32;
                    let end_time = NaiveTime::from_hms_opt(end_hours.min(23), end_mins.min(59), 0)
                        .unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 0).unwrap());

                    blocks.push(TaskBlock {
                        task_id: task.id,
                        title: task.title.trim().to_string(),
                        start_time,
                        end_time,
                        status: task.status,
                        is_selected: is_task_selected,
                        color: task_color_from_id(&task.id),
                    });
                }
            }
        } else {
            // Schedule all subtasks with parent task prefix
            let parent_title = task.title.trim();
            for subtask in &task.subtasks {
                // Only schedule subtasks that have remaining time
                let remaining = subtask.track.estimate - subtask.track.elapsed;
                if remaining > Duration::zero() {
                    let remaining_minutes = remaining.num_minutes();
                    if remaining_minutes > 0 {
                        // Calculate start and end times
                        let start_hours = (accumulated_minutes / 60) as u32;
                        let start_mins = (accumulated_minutes % 60) as u32;
                        let start_time = NaiveTime::from_hms_opt(start_hours.min(23), start_mins.min(59), 0)
                            .unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 0).unwrap());

                        accumulated_minutes += remaining_minutes;

                        // Cap at midnight
                        if accumulated_minutes >= 24 * 60 {
                            accumulated_minutes = 24 * 60 - 1;
                        }

                        let end_hours = (accumulated_minutes / 60) as u32;
                        let end_mins = (accumulated_minutes % 60) as u32;
                        let end_time = NaiveTime::from_hms_opt(end_hours.min(23), end_mins.min(59), 0)
                            .unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 0).unwrap());

                        // Prefix subtask title with parent task name
                        let subtask_title = format!("{} > {}", parent_title, subtask.title.trim());

                        blocks.push(TaskBlock {
                            task_id: subtask.id,
                            title: subtask_title,
                            start_time,
                            end_time,
                            status: subtask.status,
                            is_selected: is_task_selected,
                            color: task_color_from_id(&subtask.id),
                        });
                    }
                }
            }
        }
    }

    blocks
}


/// Check if a time slice should show the NOW line
/// NOW line appears at the hour that contains the current time
fn is_near_current_time(slice_time: &NaiveTime, current_time: NaiveTime) -> bool {
    // Show NOW line at the current hour
    slice_time.hour() == current_time.hour()
}

/// Get the style for a task based on its status
fn get_task_style(status: RunStatus, is_current_slot: bool) -> Style {
    let base_style = match status {
        RunStatus::Running => running_style(),
        RunStatus::Paused => paused_style(),
        RunStatus::Idle => idle_style(),
        RunStatus::Done => Style::default().fg(Color::Green),
        RunStatus::Postponed => Style::default().fg(Color::DarkGray),
    };

    // If it's the current time slot, make it bold
    if is_current_slot {
        base_style.add_modifier(Modifier::BOLD)
    } else {
        base_style
    }
}

/// Build legend showing task names with their colors
fn build_legend(blocks: &[TaskBlock]) -> Span {
    let mut legend = String::new();

    // Collect unique task titles (use a Vec to maintain order, deduplicate by ID)
    let mut seen_ids = std::collections::HashSet::new();
    let mut unique_tasks = Vec::new();

    for block in blocks {
        if !seen_ids.contains(&block.task_id) {
            seen_ids.insert(block.task_id);
            unique_tasks.push(block.title.as_str());
        }
    }

    for (idx, title) in unique_tasks.iter().enumerate() {
        if idx > 0 {
            legend.push_str(" · ");
        }
        legend.push_str(&format!("□ {}", truncate_string(title, 15)));

        // Limit to 3 tasks in legend
        if idx >= 2 {
            if unique_tasks.len() > 3 {
                legend.push_str(&format!(" · +{} more", unique_tasks.len() - 3));
            }
            break;
        }
    }

    Span::styled(legend, Style::default().fg(Color::DarkGray))
}

/// Generate a stable color for a task based on its ID
fn task_color_from_id(id: &Uuid) -> Color {
    let bytes = id.as_bytes();
    let hash = bytes.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32));

    // Color palette (terminal-safe colors)
    let palette = [
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::LightCyan,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
    ];

    palette[(hash as usize) % palette.len()]
}

/// Truncate string to fit width
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Helper trait to dim colors
trait DimColor {
    fn dim(self) -> Self;
}

impl DimColor for Color {
    fn dim(self) -> Self {
        match self {
            Color::Cyan => Color::DarkGray,
            Color::Green => Color::DarkGray,
            Color::Yellow => Color::DarkGray,
            Color::Blue => Color::DarkGray,
            Color::Magenta => Color::DarkGray,
            Color::LightCyan => Color::Cyan,
            Color::LightGreen => Color::Green,
            Color::LightYellow => Color::Yellow,
            Color::LightBlue => Color::Blue,
            Color::LightMagenta => Color::Magenta,
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Item;

    #[test]
    fn test_build_time_grid() {
        let current = NaiveTime::from_hms_opt(14, 7, 0).unwrap();
        let grid = build_time_grid(current);

        // Should show 9am to 11:45pm in 15-minute intervals (60 slots)
        assert_eq!(grid.len(), 60);
        assert_eq!(grid[0].hour(), 9);
        assert_eq!(grid[0].minute(), 0);
        assert_eq!(grid[1].hour(), 9);
        assert_eq!(grid[1].minute(), 15);
        assert_eq!(grid[2].hour(), 9);
        assert_eq!(grid[2].minute(), 30);
        assert_eq!(grid[59].hour(), 23);
        assert_eq!(grid[59].minute(), 45);
    }

    #[test]
    fn test_schedule_tasks_basic() {
        use crate::domain::ScheduleDay;

        // Create a mock AppState with test tasks
        let mut app = crate::app::AppState::new(
            vec![],
            vec![],
            vec![],
            "".to_string(),
        );

        // Add test tasks
        let mut task1 = Item::new("Task 1".to_string(), Duration::minutes(30), ScheduleDay::Today);
        task1.track.elapsed = Duration::minutes(21); // 9 minutes remaining

        let mut task2 = Item::new("Task 2".to_string(), Duration::minutes(15), ScheduleDay::Today);
        task2.track.elapsed = Duration::minutes(0); // 15 minutes remaining

        let mut task3 = Item::new("Task 3".to_string(), Duration::hours(3), ScheduleDay::Today);
        task3.track.elapsed = Duration::minutes(0); // 3 hours remaining

        app.tasks = vec![task1, task2, task3];

        let current_time = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
        let blocks = schedule_tasks(&app, current_time);

        println!("\n=== Test Schedule Output ===");
        println!("Current time: {:02}:{:02}", current_time.hour(), current_time.minute());
        println!("Number of blocks scheduled: {}", blocks.len());

        for (i, block) in blocks.iter().enumerate() {
            println!("\nBlock {}: {}", i + 1, block.title);
            println!("  Start: {:02}:{:02}", block.start_time.hour(), block.start_time.minute());
            println!("  End:   {:02}:{:02}", block.end_time.hour(), block.end_time.minute());
            println!("  Duration: {} minutes",
                (block.end_time.hour() as i64 * 60 + block.end_time.minute() as i64) -
                (block.start_time.hour() as i64 * 60 + block.start_time.minute() as i64)
            );
        }
        println!("=== End Test Output ===\n");

        // Should schedule all 3 tasks
        assert_eq!(blocks.len(), 3, "Should schedule all 3 tasks");

        // Task 1: starts at 15:00, 9 minutes remaining
        assert_eq!(blocks[0].title, "Task 1");
        assert_eq!(blocks[0].start_time.hour(), 15);
        assert_eq!(blocks[0].start_time.minute(), 0);

        // Task 2: starts after Task 1
        assert_eq!(blocks[1].title, "Task 2");

        // Task 3: starts after Task 2
        assert_eq!(blocks[2].title, "Task 3");
    }

    #[test]
    fn test_block_overlap_detection() {
        let current_time = NaiveTime::from_hms_opt(15, 0, 0).unwrap();

        // Create a test block from 15:30 to 16:45
        let block = TaskBlock {
            task_id: uuid::Uuid::new_v4(),
            title: "Test Task".to_string(),
            start_time: NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(16, 45, 0).unwrap(),
            status: RunStatus::Idle,
            is_selected: false,
            color: Color::Green,
        };

        // Test various hour slots
        let test_cases = vec![
            (14, false, "14:00 - before block"),
            (15, true, "15:00 - overlaps with block (15:30 start)"),
            (16, true, "16:00 - inside block"),
            (17, false, "17:00 - after block (ends at 16:45)"),
            (18, false, "18:00 - after block"),
        ];

        println!("\n=== Block Overlap Test ===");
        println!("Block: {:02}:{:02} - {:02}:{:02}",
            block.start_time.hour(), block.start_time.minute(),
            block.end_time.hour(), block.end_time.minute()
        );

        for (hour, expected_overlap, desc) in test_cases {
            let slice_hour = hour as i64 * 60;
            let next_hour = slice_hour + 60;
            let block_start = block.start_time.hour() as i64 * 60 + block.start_time.minute() as i64;
            let block_end = block.end_time.hour() as i64 * 60 + block.end_time.minute() as i64;

            let overlaps = block_start < next_hour && block_end > slice_hour;

            println!("\nHour {:02}:00 - {:02}:00", hour, hour + 1);
            println!("  Slice: {} - {}", slice_hour, next_hour);
            println!("  Block: {} - {}", block_start, block_end);
            println!("  Overlaps: {} (expected: {})", overlaps, expected_overlap);
            println!("  {}", desc);

            assert_eq!(overlaps, expected_overlap, "{}", desc);
        }
        println!("=== End Overlap Test ===\n");
    }


    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello World", 20), "Hello World");
        assert_eq!(truncate_string("Hello World", 8), "Hello...");
        assert_eq!(truncate_string("Hello World", 5), "He...");
    }
}
