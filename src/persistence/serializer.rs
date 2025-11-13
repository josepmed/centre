use crate::domain::{Item, RunStatus, ScheduleDay};
use chrono::Local;

/// Serialize active, done, and archived items into daily file format (YYYY-MM-DD.md)
pub fn serialize_daily_file(
    active_items: &[Item],
    done_items: &[Item],
    archived_items: &[Item],
) -> String {
    serialize_daily_file_with_date(active_items, done_items, archived_items, Local::now().date_naive())
}

/// Serialize active, done, and archived items into daily file format with a specific date
pub fn serialize_daily_file_with_date(
    active_items: &[Item],
    done_items: &[Item],
    archived_items: &[Item],
    date: chrono::NaiveDate,
) -> String {
    let mut output = String::new();

    // Add main header with date
    output.push_str(&format!("# {}\n\n", date.format("%Y-%m-%d")));

    // ACTIVE section
    output.push_str("## ACTIVE\n\n");
    for item in active_items {
        if item.status.is_active() {
            output.push_str(&serialize_item(item, 0, false));
            output.push('\n');
        }
    }

    // DONE section
    if !done_items.is_empty() {
        output.push_str("## DONE\n\n");
        for item in done_items {
            output.push_str(&serialize_item(item, 0, true));
            output.push('\n');
        }
    }

    // ARCHIVED section
    if !archived_items.is_empty() {
        output.push_str("## ARCHIVED\n\n");
        for item in archived_items {
            output.push_str(&serialize_item(item, 0, false));
            output.push('\n');
        }
    }

    output
}

/// Serialize items to markdown format (legacy format for today.md or tomorrow.md)
pub fn serialize_to_markdown(items: &[Item], schedule: ScheduleDay) -> String {
    let mut output = String::new();

    // Add header
    let date = Local::now().format("%Y-%m-%d");
    let header = match schedule {
        ScheduleDay::Today => format!("# Today ({})\n\n", date),
        ScheduleDay::Tomorrow => format!("# Tomorrow ({})\n\n", date),
    };
    output.push_str(&header);

    // Serialize each task
    for item in items {
        // Only include active items (not DONE or POSTPONED)
        if item.status.is_active() {
            output.push_str(&serialize_item(item, 0, false));
            output.push('\n');
        }
    }

    output
}

/// Serialize a single item (task or subtask)
/// `include_analytics` adds analytics data for done items
fn serialize_item(item: &Item, depth: usize, include_analytics: bool) -> String {
    let mut output = String::new();
    let indent = "    ".repeat(depth);

    // Task line: "- [STATUS] Title"
    output.push_str(&format!(
        "{}- [{}] {}\n",
        indent,
        item.status.to_tag(),
        item.title
    ));

    // Estimate
    output.push_str(&format!(
        "{}  est: {:.2}h\n",
        indent,
        item.track.estimate_hours()
    ));

    // Elapsed
    output.push_str(&format!(
        "{}  elapsed: {:.2}h\n",
        indent,
        item.track.elapsed_hours()
    ));

    // Completed timestamp (if done)
    if let Some(completed) = item.completed_at {
        output.push_str(&format!("{}  completed: {}\n", indent, completed.to_rfc3339()));
    }

    // Tags (if any)
    if !item.tags.is_empty() {
        output.push_str(&format!("{}  tags: {}\n", indent, item.tags.join(", ")));
    }

    // Notes (if not empty)
    if !item.notes.trim().is_empty() {
        output.push_str(&format!("{}  notes: |\n", indent));
        for line in item.notes.lines() {
            output.push_str(&format!("{}    {}\n", indent, line));
        }
    }

    // Analytics (for done items)
    if include_analytics && item.status == RunStatus::Done {
        output.push_str(&format!("{}  Analytics:\n", indent));

        if let Some(calendar) = item.calendar_time() {
            let calendar_hours = calendar.num_minutes() as f64 / 60.0;
            output.push_str(&format!("{}    Calendar Time: {:.2}h\n", indent, calendar_hours));
        }

        let running = item.running_time();
        let running_hours = running.num_minutes() as f64 / 60.0;
        output.push_str(&format!("{}    Active Time: {:.2}h\n", indent, running_hours));

        let interruptions = item.interruption_count();
        output.push_str(&format!("{}    Interruptions: {}\n", indent, interruptions));

        let sessions = item.session_count();
        output.push_str(&format!("{}    Sessions: {}\n", indent, sessions));
    }

    // Created timestamp
    output.push_str(&format!("{}  created: {}\n", indent, item.created_at.to_rfc3339()));

    // State history (if any beyond initial event)
    if !item.state_history.is_empty() {
        output.push_str(&format!("{}  history:\n", indent));
        for event in &item.state_history {
            let event_str = if let Some(from_status) = event.from_status {
                format!("{} -> {}", from_status.to_tag(), event.to_status.to_tag())
            } else {
                event.to_status.to_tag().to_string()
            };
            output.push_str(&format!("{}    - {}: {}\n", indent, event.timestamp.to_rfc3339(), event_str));
        }
    }

    // Subtasks (if any)
    if !item.subtasks.is_empty() {
        output.push_str(&format!("{}  subtasks:\n", indent));
        for subtask in &item.subtasks {
            // Include subtasks based on status
            if include_analytics || subtask.status.is_active() || subtask.status == RunStatus::Done {
                output.push_str(&serialize_item(subtask, depth + 1, include_analytics));
            }
        }
    }

    output
}

/// Serialize a done entry for the done.log.md
pub fn serialize_done_entry(item: &Item) -> String {
    let timestamp = Local::now().to_rfc3339();
    let mut output = String::new();

    output.push_str(&format!("## {}\n", timestamp));
    output.push_str(&format!("Task: \"{}\"\n", item.title));

    // Basic metrics
    output.push_str(&format!("Elapsed: {:.2}h\n", item.track.elapsed_hours()));
    output.push_str(&format!(
        "Estimate: {:.2}h\n",
        item.track.estimate_hours()
    ));
    output.push_str("Status: Done\n");

    // Analytics metrics
    if let Some(calendar) = item.calendar_time() {
        let calendar_hours = calendar.num_minutes() as f64 / 60.0;
        output.push_str(&format!("Calendar Time: {:.2}h\n", calendar_hours));
    }

    let running = item.running_time();
    let running_hours = running.num_minutes() as f64 / 60.0;
    output.push_str(&format!("Active Time: {:.2}h\n", running_hours));

    let interruptions = item.interruption_count();
    output.push_str(&format!("Interruptions: {}\n", interruptions));

    let sessions = item.session_count();
    output.push_str(&format!("Sessions: {}\n", sessions));

    if !item.tags.is_empty() {
        output.push_str(&format!("Tags: {}\n", item.tags.join(", ")));
    }

    // State history
    if !item.state_history.is_empty() {
        output.push_str("History:\n");
        for event in &item.state_history {
            let event_str = if let Some(from_status) = event.from_status {
                format!("{} -> {}", from_status.to_tag(), event.to_status.to_tag())
            } else {
                event.to_status.to_tag().to_string()
            };
            output.push_str(&format!("  - {}: {}\n", event.timestamp.format("%Y-%m-%d %H:%M:%S"), event_str));
        }
    }

    if !item.notes.trim().is_empty() {
        output.push_str("Notes:\n");
        output.push_str(&item.notes);
        output.push('\n');
    }

    output.push('\n');
    output
}

/// Serialize an archive entry for the archive.md
pub fn serialize_archive_entry(item: &Item) -> String {
    let timestamp = Local::now().to_rfc3339();
    let mut output = String::new();

    output.push_str(&format!("## {}\n", timestamp));
    output.push_str(&format!("Task: \"{}\"\n", item.title));
    output.push_str(&format!("Elapsed: {:.2}h\n", item.track.elapsed_hours()));
    output.push_str(&format!("Estimate: {:.2}h\n", item.track.estimate_hours()));
    output.push_str(&format!("Status: {:?}\n", item.status));

    if !item.tags.is_empty() {
        output.push_str(&format!("Tags: {}\n", item.tags.join(", ")));
    }

    if !item.notes.trim().is_empty() {
        output.push_str("Notes:\n");
        output.push_str(&item.notes);
        output.push('\n');
    }

    // Include subtasks if any
    if !item.subtasks.is_empty() {
        output.push_str("Subtasks:\n");
        for subtask in &item.subtasks {
            output.push_str(&format!("  - {}\n", subtask.title));
            output.push_str(&format!("    Elapsed: {:.2}h\n", subtask.track.elapsed_hours()));
            output.push_str(&format!("    Estimate: {:.2}h\n", subtask.track.estimate_hours()));
        }
    }

    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RunStatus, TimeTracking};
    use chrono::Duration;

    fn create_test_item(title: &str, status: RunStatus) -> Item {
        let mut item = Item::new(
            title.to_string(),
            Duration::hours(2),
            ScheduleDay::Today,
        );
        item.status = status;
        item.track.elapsed = Duration::minutes(78); // 1.3 hours
        item.notes = "Test notes".to_string();
        item
    }

    #[test]
    fn test_serialize_simple_task() {
        let item = create_test_item("Write tests", RunStatus::Running);
        let items = vec![item];

        let output = serialize_to_markdown(&items, ScheduleDay::Today);

        assert!(output.contains("# Today"));
        assert!(output.contains("- [RUNNING] Write tests"));
        assert!(output.contains("est: 2.00h"));
        assert!(output.contains("elapsed: 1.30h"));
        assert!(output.contains("notes: |"));
        assert!(output.contains("Test notes"));
    }

    #[test]
    fn test_serialize_with_subtasks() {
        let mut item = create_test_item("Parent task", RunStatus::Running);
        let subtask1 = create_test_item("Subtask 1", RunStatus::Paused);
        let subtask2 = create_test_item("Subtask 2", RunStatus::Idle);

        item.add_subtask(subtask1);
        item.add_subtask(subtask2);

        let items = vec![item];
        let output = serialize_to_markdown(&items, ScheduleDay::Today);

        assert!(output.contains("- [RUNNING] Parent task"));
        assert!(output.contains("subtasks:"));
        assert!(output.contains("- [PAUSED] Subtask 1"));
        assert!(output.contains("- [IDLE] Subtask 2"));
    }

    #[test]
    fn test_serialize_excludes_done() {
        let item1 = create_test_item("Active task", RunStatus::Running);
        let item2 = create_test_item("Done task", RunStatus::Done);

        let items = vec![item1, item2];
        let output = serialize_to_markdown(&items, ScheduleDay::Today);

        assert!(output.contains("Active task"));
        assert!(!output.contains("Done task"));
    }

    #[test]
    fn test_serialize_done_entry() {
        let item = create_test_item("Completed task", RunStatus::Done);
        let output = serialize_done_entry(&item);

        assert!(output.contains("##"));
        assert!(output.contains("Task: \"Completed task\""));
        assert!(output.contains("Elapsed: 1.30h"));
        assert!(output.contains("Estimate: 2.00h"));
        assert!(output.contains("Status: Done"));
        assert!(output.contains("Active Time:"));
        assert!(output.contains("Interruptions:"));
        assert!(output.contains("Sessions:"));
        assert!(output.contains("History:"));
        assert!(output.contains("Notes:"));
        assert!(output.contains("Test notes"));
    }

    #[test]
    fn test_serialize_empty_notes() {
        let mut item = create_test_item("Task", RunStatus::Idle);
        item.notes = String::new();

        let items = vec![item];
        let output = serialize_to_markdown(&items, ScheduleDay::Today);

        // Should not include notes section if notes are empty
        assert!(!output.contains("notes: |"));
    }

    #[test]
    fn test_serialize_daily_file_with_done_items() {
        let active_item = create_test_item("Active task", RunStatus::Idle);
        let done_item = create_test_item("Done task", RunStatus::Done);
        let archived_item = create_test_item("Archived task", RunStatus::Idle);

        let active_items = vec![active_item];
        let done_items = vec![done_item];
        let archived_items = vec![archived_item];

        let output = serialize_daily_file(&active_items, &done_items, &archived_items);

        println!("Output:\n{}", output);

        // Check all sections are present
        assert!(output.contains("## ACTIVE"));
        assert!(output.contains("## DONE"));
        assert!(output.contains("## ARCHIVED"));

        // Check items are in correct sections
        assert!(output.contains("Active task"));
        assert!(output.contains("Done task"));
        assert!(output.contains("Archived task"));
    }
}
