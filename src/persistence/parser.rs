use crate::domain::{Item, RunStatus, ScheduleDay, StateEvent};
use anyhow::{Context, Result};
use chrono::{Duration, DateTime, Local, TimeZone};

/// Section type for daily files
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    Active,
    Done,
    Archived,
}

/// Parse a daily markdown file into separate lists for ACTIVE, DONE, and ARCHIVED items
pub fn parse_daily_file(content: &str) -> Result<(Vec<Item>, Vec<Item>, Vec<Item>)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut active_items = Vec::new();
    let mut done_items = Vec::new();
    let mut archived_items = Vec::new();
    let mut current_section = Section::Active; // Default section
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check for section headers
        if line == "## ACTIVE" {
            current_section = Section::Active;
            i += 1;
            continue;
        } else if line == "## DONE" {
            current_section = Section::Done;
            i += 1;
            continue;
        } else if line == "## ARCHIVED" {
            current_section = Section::Archived;
            i += 1;
            continue;
        }

        // Skip empty lines and main header
        if line.is_empty() || line.starts_with("# ") {
            i += 1;
            continue;
        }

        // Parse task (starts with "- [STATUS]")
        if line.starts_with("- [") {
            match parse_item(&lines, &mut i, ScheduleDay::Today, 0) {
                Ok(item) => {
                    match current_section {
                        Section::Active => active_items.push(item),
                        Section::Done => done_items.push(item),
                        Section::Archived => archived_items.push(item),
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse item at line {}: {}", i + 1, e);
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    Ok((active_items, done_items, archived_items))
}

/// Parse a markdown file (legacy format for today.md or tomorrow.md) into a list of items
pub fn parse_markdown(content: &str, schedule: ScheduleDay) -> Result<Vec<Item>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and headers
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Parse task (starts with "- [STATUS]")
        if line.starts_with("- [") {
            match parse_item(&lines, &mut i, schedule, 0) {
                Ok(item) => items.push(item),
                Err(e) => {
                    eprintln!("Warning: Failed to parse item at line {}: {}", i + 1, e);
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(items)
}

/// Parse a single item (task or subtask) starting at the given line index
fn parse_item(
    lines: &[&str],
    index: &mut usize,
    schedule: ScheduleDay,
    _depth: usize,
) -> Result<Item> {
    let line = lines[*index].trim();

    // Parse status and title from "- [STATUS] Title"
    let (status, title) = parse_task_line(line)?;

    *index += 1;

    // Parse fields: est, elapsed, notes, tags, created, completed, history, subtasks
    let mut estimate = Duration::zero();
    let mut elapsed = Duration::zero();
    let mut notes = String::new();
    let mut tags = Vec::new();
    let mut created_at: Option<DateTime<Local>> = None;
    let mut completed_at: Option<DateTime<Local>> = None;
    let mut state_history = Vec::new();
    let mut subtasks = Vec::new();

    while *index < lines.len() {
        let current_line = lines[*index];

        // Check if we've reached a section header
        let trimmed = current_line.trim();
        if trimmed == "## ACTIVE" || trimmed == "## DONE" || trimmed == "## ARCHIVED" {
            break;
        }

        // Check if we've reached the next task
        if trimmed.starts_with("- [") && !current_line.trim_start().starts_with(" ") {
            break;
        }

        if trimmed.starts_with("est:") {
            estimate = parse_duration(trimmed.trim_start_matches("est:").trim())?;
            *index += 1;
        } else if trimmed.starts_with("elapsed:") {
            elapsed = parse_duration(trimmed.trim_start_matches("elapsed:").trim())?;
            *index += 1;
        } else if trimmed.starts_with("notes:") {
            *index += 1;
            notes = parse_notes(lines, index);
        } else if trimmed.starts_with("tags:") {
            let tags_str = trimmed.trim_start_matches("tags:").trim();
            tags = parse_tags(tags_str);
            *index += 1;
        } else if trimmed.starts_with("created:") {
            let timestamp_str = trimmed.trim_start_matches("created:").trim();
            created_at = DateTime::parse_from_rfc3339(timestamp_str)
                .ok()
                .map(|dt| dt.with_timezone(&Local));
            *index += 1;
        } else if trimmed.starts_with("completed:") {
            let timestamp_str = trimmed.trim_start_matches("completed:").trim();
            completed_at = DateTime::parse_from_rfc3339(timestamp_str)
                .ok()
                .map(|dt| dt.with_timezone(&Local));
            *index += 1;
        } else if trimmed.starts_with("history:") {
            *index += 1;
            state_history = parse_state_history(lines, index)?;
        } else if trimmed.starts_with("subtasks:") {
            *index += 1;
            subtasks = parse_subtasks(lines, index, schedule)?;
        } else if trimmed.is_empty() {
            *index += 1;
        } else {
            // Unknown field, skip
            *index += 1;
        }
    }

    let mut item = Item::new(title, estimate, schedule);
    item.status = status;
    item.track.elapsed = elapsed;
    item.notes = notes;
    item.tags = tags;

    // Override created_at if parsed, otherwise keep the one from new()
    if let Some(created) = created_at {
        item.created_at = created;
    }
    item.completed_at = completed_at;

    // Override state_history if parsed, otherwise keep initial event from new()
    if !state_history.is_empty() {
        item.state_history = state_history;
    }

    item.subtasks = subtasks;
    item.regenerate_ids();

    Ok(item)
}

/// Parse task line: "- [STATUS] Title" -> (status, title)
fn parse_task_line(line: &str) -> Result<(RunStatus, String)> {
    let line = line.trim_start_matches('-').trim();

    // Extract status from [STATUS]
    let status_end = line
        .find(']')
        .context("Invalid task line: missing ] for status")?;
    let status_str = &line[1..status_end];
    let status = RunStatus::from_tag(status_str)
        .with_context(|| format!("Invalid status tag: {}", status_str))?;

    // Extract title (everything after the status)
    let title = line[status_end + 1..].trim().to_string();

    if title.is_empty() {
        anyhow::bail!("Task has empty title");
    }

    Ok((status, title))
}

/// Parse duration from string like "1.5h" or "2.0h"
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim().trim_end_matches('h');
    let hours: f64 = s
        .parse()
        .with_context(|| format!("Invalid duration: {}", s))?;
    Ok(Duration::seconds((hours * 3600.0) as i64))
}

/// Parse multi-line notes after "notes: |"
fn parse_notes(lines: &[&str], index: &mut usize) -> String {
    let mut notes = Vec::new();

    while *index < lines.len() {
        let line = lines[*index];

        // Check if this line starts a new field or task
        let trimmed = line.trim();
        if trimmed.starts_with("est:")
            || trimmed.starts_with("elapsed:")
            || trimmed.starts_with("tags:")
            || trimmed.starts_with("created:")
            || trimmed.starts_with("completed:")
            || trimmed.starts_with("history:")
            || trimmed.starts_with("subtasks:")
            || trimmed.starts_with("- [")
        {
            break;
        }

        // Collect indented lines as notes
        if line.starts_with("    ") || line.trim().is_empty() {
            notes.push(line.trim_start_matches("    "));
            *index += 1;
        } else {
            break;
        }
    }

    // Trim leading/trailing empty lines
    while notes.first().map_or(false, |s| s.trim().is_empty()) {
        notes.remove(0);
    }
    while notes.last().map_or(false, |s| s.trim().is_empty()) {
        notes.pop();
    }

    notes.join("\n")
}

/// Parse tags from comma-separated string like "tag1, tag2, tag3"
fn parse_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

/// Parse state history after "history:"
/// Expected format:
///   - 2025-11-10T09:00:00Z: IDLE
///   - 2025-11-10T09:15:00Z: IDLE -> RUNNING
fn parse_state_history(lines: &[&str], index: &mut usize) -> Result<Vec<StateEvent>> {
    let mut history = Vec::new();

    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim();

        // Check if this line starts a new field or task
        if trimmed.starts_with("est:")
            || trimmed.starts_with("elapsed:")
            || trimmed.starts_with("notes:")
            || trimmed.starts_with("tags:")
            || trimmed.starts_with("created:")
            || trimmed.starts_with("completed:")
            || trimmed.starts_with("subtasks:")
            || trimmed.starts_with("- [")
        {
            break;
        }

        // Parse history entry (indented "- timestamp: status" or "- timestamp: from -> to")
        if line.starts_with("    - ") {
            let entry = line.trim_start_matches("    - ");
            if let Some((timestamp_str, status_str)) = entry.split_once(": ") {
                // Try parsing as RFC3339 first (for today.md/tomorrow.md format)
                let timestamp_local = if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                    timestamp.with_timezone(&Local)
                } else {
                    // Try parsing as simple format "YYYY-MM-DD HH:MM:SS" (for done.log.md format)
                    match chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
                        Ok(naive_dt) => Local.from_local_datetime(&naive_dt).single().unwrap_or_else(|| Local::now()),
                        Err(_) => {
                            *index += 1;
                            continue;
                        }
                    }
                };

                // Check if it's a transition (has ->)
                if let Some((from_str, to_str)) = status_str.split_once(" -> ") {
                    // Transition: from -> to
                    if let (Some(from_status), Some(to_status)) = (
                        RunStatus::from_tag(from_str.trim()),
                        RunStatus::from_tag(to_str.trim()),
                    ) {
                        history.push(StateEvent {
                            timestamp: timestamp_local,
                            from_status: Some(from_status),
                            to_status,
                        });
                    }
                } else {
                    // Initial state (no ->)
                    if let Some(to_status) = RunStatus::from_tag(status_str.trim()) {
                        history.push(StateEvent {
                            timestamp: timestamp_local,
                            from_status: None,
                            to_status,
                        });
                    }
                }
            }
            *index += 1;
        } else if trimmed.is_empty() {
            *index += 1;
        } else {
            break;
        }
    }

    Ok(history)
}

/// Parse subtasks list after "subtasks:"
fn parse_subtasks(
    lines: &[&str],
    index: &mut usize,
    schedule: ScheduleDay,
) -> Result<Vec<Item>> {
    let mut subtasks = Vec::new();

    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim();

        // Check if this is a subtask line (indented "- [STATUS] Title")
        if line.starts_with("    - [") {
            match parse_item(lines, index, schedule, 1) {
                Ok(subtask) => subtasks.push(subtask),
                Err(e) => {
                    eprintln!("Warning: Failed to parse subtask at line {}: {}", *index + 1, e);
                    *index += 1;
                }
            }
        } else if trimmed.is_empty() {
            *index += 1;
        } else if trimmed.starts_with("- [") {
            // Reached next parent task
            break;
        } else if !line.starts_with("    ") && !trimmed.is_empty() {
            // Reached next field or section
            break;
        } else {
            *index += 1;
        }
    }

    Ok(subtasks)
}

/// Parse done.log.md and return tasks completed today
pub fn parse_done_log_today(content: &str) -> Result<Vec<Item>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut done_items = Vec::new();
    let today = Local::now().date_naive();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for timestamp header (e.g., "## 2025-11-10T14:22:08")
        if line.starts_with("## ") {
            let timestamp_str = line.trim_start_matches("## ");

            // Try to parse the timestamp
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                let entry_date = timestamp.date_naive();

                // Only parse entries from today
                if entry_date == today {
                    if let Ok(item) = parse_done_entry(&lines, &mut i) {
                        done_items.push(item);
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(done_items)
}

/// Parse a single done log entry
fn parse_done_entry(lines: &[&str], index: &mut usize) -> Result<Item> {
    let mut title = String::new();
    let mut elapsed_hours = 0.0;
    let mut estimate_hours = 0.0;
    let mut notes = String::new();
    let mut state_history = Vec::new();
    let mut in_notes = false;

    *index += 1; // Skip the timestamp header

    while *index < lines.len() {
        let line = lines[*index];

        // Stop at next entry
        if line.starts_with("## ") {
            break;
        }

        // Parse fields
        if line.starts_with("Task: \"") {
            title = line
                .trim_start_matches("Task: \"")
                .trim_end_matches('"')
                .to_string();
            in_notes = false;
        } else if line.starts_with("Elapsed: ") {
            let val = line.trim_start_matches("Elapsed: ").trim_end_matches('h');
            elapsed_hours = val.parse().unwrap_or(0.0);
            in_notes = false;
        } else if line.starts_with("Estimate at finish: ") {
            let val = line
                .trim_start_matches("Estimate at finish: ")
                .trim_end_matches('h');
            estimate_hours = val.parse().unwrap_or(0.0);
            in_notes = false;
        } else if line.starts_with("Estimate: ") {
            // Also support "Estimate:" without "at finish"
            let val = line.trim_start_matches("Estimate: ").trim_end_matches('h');
            estimate_hours = val.parse().unwrap_or(0.0);
            in_notes = false;
        } else if line.starts_with("History:") {
            in_notes = false;
            *index += 1;
            state_history = parse_state_history(lines, index)?;
            continue; // parse_state_history advances index, so don't increment again
        } else if line.starts_with("Notes:") {
            in_notes = true;
        } else if in_notes && !line.trim().is_empty() {
            if !notes.is_empty() {
                notes.push('\n');
            }
            notes.push_str(line);
        }

        *index += 1;
    }

    let mut item = Item::new(
        title,
        Duration::seconds((estimate_hours * 3600.0) as i64),
        ScheduleDay::Today,
    );
    item.track.elapsed = Duration::seconds((elapsed_hours * 3600.0) as i64);
    item.notes = notes;
    item.state_history = state_history;
    item.mark_done();

    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_line() {
        let (status, title) = parse_task_line("- [RUNNING] Write tests").unwrap();
        assert_eq!(status, RunStatus::Running);
        assert_eq!(title, "Write tests");

        let (status, title) = parse_task_line("- [IDLE] Review code").unwrap();
        assert_eq!(status, RunStatus::Idle);
        assert_eq!(title, "Review code");
    }

    #[test]
    fn test_parse_duration() {
        let dur = parse_duration("1.5h").unwrap();
        assert_eq!(dur, Duration::seconds(5400)); // 1.5 * 3600

        let dur = parse_duration("2.0h").unwrap();
        assert_eq!(dur, Duration::hours(2));
    }

    #[test]
    fn test_parse_simple_task() {
        let content = r#"# Today (2025-11-10)

- [RUNNING] Write project proposal
  est: 2.0h
  elapsed: 1.3h
  notes: |
    finalize argument for timeline
"#;

        let items = parse_markdown(content, ScheduleDay::Today).unwrap();
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.title, "Write project proposal");
        assert_eq!(item.status, RunStatus::Running);
        assert_eq!(item.track.estimate, Duration::hours(2));
        assert_eq!(item.track.elapsed_hours(), 1.3);
        assert_eq!(item.notes.trim(), "finalize argument for timeline");
        assert_eq!(item.subtasks.len(), 0);
    }

    #[test]
    fn test_parse_task_with_subtasks() {
        let content = r#"# Today (2025-11-10)

- [RUNNING] Write project proposal
  est: 2.0h
  elapsed: 1.3h
  notes: |
    finalize argument for timeline
  subtasks:
    - [PAUSED] Outline sections
      est: 1.0h
      elapsed: 0.7h
      notes: |
        bullet the main points
    - [RUNNING] Draft intro
      est: 1.0h
      elapsed: 0.6h
      notes: |
        tone: concise, confident
"#;

        let items = parse_markdown(content, ScheduleDay::Today).unwrap();
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.subtasks.len(), 2);

        let subtask1 = &item.subtasks[0];
        assert_eq!(subtask1.title, "Outline sections");
        assert_eq!(subtask1.status, RunStatus::Paused);

        let subtask2 = &item.subtasks[1];
        assert_eq!(subtask2.title, "Draft intro");
        assert_eq!(subtask2.status, RunStatus::Running);
    }

    #[test]
    fn test_parse_multiple_tasks() {
        let content = r#"# Today (2025-11-10)

- [RUNNING] Task 1
  est: 1.0h
  elapsed: 0.5h
  notes: |
    notes for task 1

- [IDLE] Task 2
  est: 2.0h
  elapsed: 0.0h
  notes: |
    notes for task 2
"#;

        let items = parse_markdown(content, ScheduleDay::Today).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Task 1");
        assert_eq!(items[1].title, "Task 2");
    }

    #[test]
    fn test_parse_empty_file() {
        let content = "";
        let items = parse_markdown(content, ScheduleDay::Today).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_parse_daily_file_sections() {
        let content = r#"# 2025-11-11

## ACTIVE

- [IDLE] Active task
  est: 1.00h
  elapsed: 0.00h
  created: 2025-11-11T10:00:00+01:00
  history:
    - 2025-11-11T10:00:00+01:00: IDLE

## DONE

- [DONE] Done task
  est: 1.00h
  elapsed: 0.50h
  completed: 2025-11-11T11:00:00+01:00
  created: 2025-11-11T10:00:00+01:00
  history:
    - 2025-11-11T10:00:00+01:00: IDLE
    - 2025-11-11T11:00:00+01:00: IDLE -> DONE

## ARCHIVED

- [IDLE] Archived task
  est: 1.00h
  elapsed: 0.00h
  created: 2025-11-11T10:00:00+01:00
  history:
    - 2025-11-11T10:00:00+01:00: IDLE
"#;

        let (active, done, archived) = parse_daily_file(content).unwrap();

        println!("Active: {}", active.len());
        println!("Done: {}", done.len());
        println!("Archived: {}", archived.len());

        assert_eq!(active.len(), 1, "Should have 1 active task");
        assert_eq!(done.len(), 1, "Should have 1 done task");
        assert_eq!(archived.len(), 1, "Should have 1 archived task");

        assert_eq!(active[0].title, "Active task");
        assert_eq!(done[0].title, "Done task");
        assert_eq!(archived[0].title, "Archived task");

        assert_eq!(active[0].status, RunStatus::Idle);
        assert_eq!(done[0].status, RunStatus::Done);
        assert_eq!(archived[0].status, RunStatus::Idle);
    }
}
