use super::item::Item;
use chrono::Duration;

/// A flattened row for rendering the task list
#[derive(Debug, Clone)]
pub struct FlatRow {
    /// Index in the flattened list
    pub index: usize,
    /// Depth in the tree (0 = parent, 1 = subtask)
    pub depth: usize,
    /// Whether this is the last subtask of its parent
    pub is_last: bool,
    /// Reference to the item (stored as index into tasks array)
    pub task_index: usize,
    /// Subtask index (None for parent tasks)
    pub subtask_index: Option<usize>,
}

/// Flatten a hierarchical task list into a linear list for rendering
pub fn flatten_tasks(tasks: &[Item]) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    let mut flat_index = 0;

    for (task_idx, task) in tasks.iter().enumerate() {
        // Add parent task
        rows.push(FlatRow {
            index: flat_index,
            depth: 0,
            is_last: false,
            task_index: task_idx,
            subtask_index: None,
        });
        flat_index += 1;

        // Add subtasks if expanded
        if task.expanded && !task.subtasks.is_empty() {
            let subtask_count = task.subtasks.len();
            for (st_idx, _subtask) in task.subtasks.iter().enumerate() {
                rows.push(FlatRow {
                    index: flat_index,
                    depth: 1,
                    is_last: st_idx == subtask_count - 1,
                    task_index: task_idx,
                    subtask_index: Some(st_idx),
                });
                flat_index += 1;
            }
        }
    }

    rows
}

/// Compute total elapsed and estimate for today's tasks
/// Only counts leaf items (tasks without subtasks + all subtasks) to avoid double-counting
pub fn compute_totals(tasks: &[Item]) -> (Duration, Duration) {
    let mut total_elapsed = Duration::zero();
    let mut total_estimate = Duration::zero();

    for task in tasks {
        // Only count the parent task if it has no subtasks (it's a leaf)
        if task.subtasks.is_empty() {
            total_elapsed = total_elapsed + task.track.elapsed;
            total_estimate = total_estimate + task.track.estimate;
        }

        // Always count subtasks (they're always leaves)
        for subtask in &task.subtasks {
            total_elapsed = total_elapsed + subtask.track.elapsed;
            total_estimate = total_estimate + subtask.track.estimate;
        }
    }

    (total_elapsed, total_estimate)
}

/// Choose plant glyph based on progress ratio
pub fn plant_glyph(ratio: f64, use_emoji: bool) -> &'static str {
    if use_emoji {
        if ratio < 0.25 {
            "🌱" // Sprout
        } else if ratio < 1.0 {
            "🌿" // Leaf
        } else {
            "🌵" // Cactus (over estimate)
        }
    } else {
        // ASCII fallback
        if ratio < 0.25 {
            "*" // Sprout
        } else if ratio < 1.0 {
            "+" // Leaf
        } else {
            "!" // Overrun
        }
    }
}

/// Choose garden plant state based on progress percentage
pub fn garden_plant_state(percent: f64, use_emoji: bool) -> &'static str {
    if use_emoji {
        if percent < 25.0 {
            "🌱"
        } else if percent < 100.0 {
            "🌿"
        } else {
            "🌵"
        }
    } else {
        if percent < 25.0 {
            "*"
        } else if percent < 100.0 {
            "+"
        } else {
            "!"
        }
    }
}

/// Get status badge text
pub fn status_badge(item: &Item) -> &'static str {
    use super::enums::RunStatus;

    match item.status {
        RunStatus::Running => "⏱ RUNNING",
        RunStatus::Paused => "⏸ PAUSED",
        RunStatus::Idle => "🌿 IDLE",
        RunStatus::Done => "✓ DONE",
        RunStatus::Postponed => "→ TOMORROW",
    }
}

/// Get tree connector for subtasks
pub fn tree_connector(is_last: bool) -> &'static str {
    if is_last {
        "└─"
    } else {
        "├─"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::enums::{RunStatus, ScheduleDay};

    fn create_test_item(title: &str) -> Item {
        Item::new(title.to_string(), Duration::hours(1), ScheduleDay::Today)
    }

    #[test]
    fn test_flatten_tasks_simple() {
        let tasks = vec![
            create_test_item("Task 1"),
            create_test_item("Task 2"),
        ];

        let rows = flatten_tasks(&tasks);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[0].task_index, 0);
        assert_eq!(rows[1].depth, 0);
        assert_eq!(rows[1].task_index, 1);
    }

    #[test]
    fn test_flatten_tasks_with_subtasks() {
        let mut task = create_test_item("Parent");
        task.add_subtask(create_test_item("Subtask 1"));
        task.add_subtask(create_test_item("Subtask 2"));

        let tasks = vec![task];
        let rows = flatten_tasks(&tasks);

        assert_eq!(rows.len(), 3); // 1 parent + 2 subtasks
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 1);
        assert!(!rows[1].is_last);
        assert!(rows[2].is_last);
    }

    #[test]
    fn test_flatten_tasks_collapsed() {
        let mut task = create_test_item("Parent");
        task.add_subtask(create_test_item("Subtask 1"));
        task.expanded = false;

        let tasks = vec![task];
        let rows = flatten_tasks(&tasks);

        assert_eq!(rows.len(), 1); // Only parent, subtasks hidden
    }

    #[test]
    fn test_compute_totals() {
        let mut task1 = create_test_item("Task 1");
        task1.track.elapsed = Duration::minutes(30);

        let mut task2 = create_test_item("Task 2");
        task2.track.elapsed = Duration::minutes(45);

        let mut subtask = create_test_item("Subtask");
        subtask.track.elapsed = Duration::minutes(15);
        task2.add_subtask(subtask);

        let tasks = vec![task1, task2];
        let (elapsed, estimate) = compute_totals(&tasks);

        // Only counts leaves: task1 (30min) + subtask (15min)
        // task2 is skipped because it has subtasks
        assert_eq!(elapsed, Duration::minutes(45)); // 30 + 15 (not 45 from task2)
        assert_eq!(estimate, Duration::hours(2)); // task1 + subtask (not task2)
    }

    #[test]
    fn test_plant_glyph_emoji() {
        assert_eq!(plant_glyph(0.1, true), "🌱");
        assert_eq!(plant_glyph(0.5, true), "🌿");
        assert_eq!(plant_glyph(1.2, true), "🌵");
    }

    #[test]
    fn test_plant_glyph_ascii() {
        assert_eq!(plant_glyph(0.1, false), "*");
        assert_eq!(plant_glyph(0.5, false), "+");
        assert_eq!(plant_glyph(1.2, false), "!");
    }

    #[test]
    fn test_tree_connector() {
        assert_eq!(tree_connector(false), "├─");
        assert_eq!(tree_connector(true), "└─");
    }

    #[test]
    fn test_status_badge() {
        let mut item = create_test_item("Test");

        item.status = RunStatus::Running;
        assert_eq!(status_badge(&item), "⏱ RUNNING");

        item.status = RunStatus::Paused;
        assert_eq!(status_badge(&item), "⏸ PAUSED");

        item.status = RunStatus::Idle;
        assert_eq!(status_badge(&item), "🌿 IDLE");
    }
}
