use super::files::{daily_file, previous_day_file, read_file, today_file};
use super::parser::{parse_daily_file, parse_markdown};
use crate::domain::{Item, ScheduleDay};
use anyhow::Result;
use std::path::Path;

/// Load and migrate tasks on startup
///
/// New behavior with daily files:
/// 1. Check if today's file exists
/// 2. If it exists, load ACTIVE, DONE, and ARCHIVED sections
/// 3. If it doesn't exist, create new file by copying incomplete tasks from previous day
/// 4. Coerce all RUNNING items to PAUSED (prevent orphaned timers)
///
/// Returns: (active_tasks, done_tasks, archived_tasks)
pub fn load_and_migrate() -> Result<(Vec<Item>, Vec<Item>, Vec<Item>)> {
    let today_path = today_file()?;

    if today_path.exists() {
        // Load today's file
        let today_content = read_file(&today_path)?;
        let (mut active_items, done_items, archived_items) = if !today_content.is_empty() {
            parse_daily_file(&today_content)?
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        // Sync elapsed time from history and coerce running items to paused
        for item in &mut active_items {
            item.sync_elapsed_from_history();
            item.coerce_running_to_paused();
        }

        Ok((active_items, done_items, archived_items))
    } else {
        // Today's file doesn't exist - check if previous day's file exists
        let previous_path = previous_day_file()?;

        if previous_path.exists() {
            // Generate report for the previous day before migrating
            let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
            if let Err(e) = crate::report::generate_report(Some(yesterday), None) {
                eprintln!("Warning: Failed to generate report for {}: {}", yesterday, e);
            }

            // Load previous day's incomplete tasks
            let previous_content = read_file(&previous_path)?;
            let (mut previous_active, _, _) = if !previous_content.is_empty() {
                parse_daily_file(&previous_content)?
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            };

            // Copy incomplete tasks to today (sync elapsed and coerce to paused)
            for item in &mut previous_active {
                item.schedule = ScheduleDay::Today;
                item.sync_elapsed_from_history();
                item.coerce_running_to_paused();
            }

            // Return copied tasks, empty done and archived
            Ok((previous_active, Vec::new(), Vec::new()))
        } else {
            // No previous file - start fresh
            Ok((Vec::new(), Vec::new(), Vec::new()))
        }
    }
}

/// Legacy migration for old format (today.md, tomorrow.md, done.log.md)
/// This can be used to migrate from the old format to the new daily file format
pub fn migrate_legacy_format() -> Result<()> {
    use super::files::{done_log_file, tomorrow_file, truncate_file};
    use super::parser::parse_done_log_today;
    use super::serializer::serialize_daily_file;
    use super::files::atomic_write;

    // Check if legacy files exist
    let old_today_path = Path::new("today.md");
    let old_tomorrow_path = tomorrow_file()?;
    let old_done_log_path = done_log_file()?;

    if !old_today_path.exists() && !old_tomorrow_path.exists() && !old_done_log_path.exists() {
        return Ok(()); // No legacy files to migrate
    }

    // Load legacy files
    let today_content = if old_today_path.exists() {
        std::fs::read_to_string(old_today_path)?
    } else {
        String::new()
    };

    let tomorrow_content = if old_tomorrow_path.exists() {
        read_file(&old_tomorrow_path)?
    } else {
        String::new()
    };

    let done_log_content = if old_done_log_path.exists() {
        read_file(&old_done_log_path)?
    } else {
        String::new()
    };

    // Parse legacy files
    let mut today_items = if !today_content.is_empty() {
        parse_markdown(&today_content, ScheduleDay::Today)?
    } else {
        Vec::new()
    };

    let mut tomorrow_items = if !tomorrow_content.is_empty() {
        parse_markdown(&tomorrow_content, ScheduleDay::Tomorrow)?
    } else {
        Vec::new()
    };

    let done_items = if !done_log_content.is_empty() {
        parse_done_log_today(&done_log_content)?
    } else {
        Vec::new()
    };

    // Merge today and tomorrow (promote tomorrow to today)
    for item in &mut tomorrow_items {
        item.schedule = ScheduleDay::Today;
    }
    today_items.extend(tomorrow_items);

    // Sync elapsed from history and coerce running to paused
    for item in &mut today_items {
        item.sync_elapsed_from_history();
        item.coerce_running_to_paused();
    }

    // Save to new daily file format
    let daily_content = serialize_daily_file(&today_items, &done_items, &Vec::new());
    let today_path = today_file()?;
    atomic_write(today_path, &daily_content)?;

    // Clear legacy files
    if old_today_path.exists() {
        std::fs::remove_file(old_today_path)?;
    }
    if old_tomorrow_path.exists() {
        truncate_file(&old_tomorrow_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RunStatus;
    use chrono::Duration;

    #[test]
    fn test_coerce_running_to_paused() {
        let mut item = Item::new(
            "Test".to_string(),
            Duration::hours(1),
            ScheduleDay::Today,
        );
        item.status = RunStatus::Running;
        item.track.start();

        let mut subtask = Item::new(
            "Subtask".to_string(),
            Duration::minutes(30),
            ScheduleDay::Today,
        );
        subtask.status = RunStatus::Running;
        subtask.track.start();
        item.add_subtask(subtask);

        item.coerce_running_to_paused();

        assert_eq!(item.status, RunStatus::Paused);
        assert_eq!(item.subtasks[0].status, RunStatus::Paused);
    }

    #[test]
    fn test_schedule_promotion() {
        let mut item = Item::new(
            "Tomorrow task".to_string(),
            Duration::hours(1),
            ScheduleDay::Tomorrow,
        );

        item.schedule = ScheduleDay::Today;
        assert_eq!(item.schedule, ScheduleDay::Today);
    }
}
