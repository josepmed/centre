use crate::domain::{GlobalMode, Item};
use crate::persistence::{daily_file, load_metadata, meta_file, parse_daily_file, read_file};
use crate::report::stats::{
    calculate_completion_stats, calculate_estimation_stats, calculate_global_stats,
    calculate_tag_stats,
};
use anyhow::Result;
use chrono::{Duration, Local, NaiveDate};
use std::fs;
use std::path::PathBuf;

/// Format duration as "Xh Ym" or "Xm" for display
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

/// Format percentage with 1 decimal place
fn format_percent(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Generate a daily report for the specified date
pub fn generate_report(date: Option<NaiveDate>, output_path: Option<PathBuf>) -> Result<PathBuf> {
    // Determine date (default to today)
    let report_date = date.unwrap_or_else(|| Local::now().date_naive());

    // Load daily file
    let daily_path = daily_file(report_date)?;
    let content = read_file(&daily_path)?;
    let (active, done, archived) = parse_daily_file(&content)?;

    // Load metadata for mode times
    let metadata = load_metadata(meta_file()?).unwrap_or_default();
    let mode_times = [
        (GlobalMode::Working, Duration::seconds(metadata.mode_time_working_secs)),
        (GlobalMode::Lunch, Duration::seconds(metadata.mode_time_lunch_secs)),
        (GlobalMode::Gym, Duration::seconds(metadata.mode_time_gym_secs)),
        (GlobalMode::Dinner, Duration::seconds(metadata.mode_time_dinner_secs)),
        (GlobalMode::Personal, Duration::seconds(metadata.mode_time_personal_secs)),
        (GlobalMode::Sleep, Duration::seconds(metadata.mode_time_sleep_secs)),
    ];

    // Calculate all statistics
    let global = calculate_global_stats(&active, &done, &archived);
    let estimation = calculate_estimation_stats(&done);
    let completion = calculate_completion_stats(&done);
    let tag_stats = calculate_tag_stats(&active, &done, &archived);

    // Generate markdown report
    let mut report = String::new();

    // Header
    report.push_str(&format!("# Daily Report - {}\n\n", report_date));

    // Summary Section
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Tasks:** {} (Active: {}, Done: {}, Archived: {})\n",
        global.total_tasks, global.active_count, global.done_count, global.archived_count));

    let completion_pct = if global.total_estimate > Duration::zero() {
        (global.total_elapsed.num_seconds() as f64 / global.total_estimate.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Total Time:** {} / {} estimated ({})\n",
        format_duration(global.total_elapsed),
        format_duration(global.total_estimate),
        format_percent(completion_pct)));

    let efficiency = if global.total_elapsed > Duration::zero() {
        (global.running_time.num_seconds() as f64 / global.total_elapsed.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Efficiency:** {}\n", format_percent(efficiency)));
    report.push_str(&format!("- **Completion Rate:** {}/{} tasks done\n\n",
        global.done_count, global.total_tasks));

    // Context Modes Section
    let total_mode_time: Duration = mode_times.iter().map(|(_, time)| *time).sum();
    if total_mode_time.num_minutes() > 0 {
        report.push_str("## Context Modes\n\n");

        for (mode, time) in &mode_times {
            if time.num_minutes() > 0 {
                let percentage = if total_mode_time > Duration::zero() {
                    (time.num_seconds() as f64 / total_mode_time.num_seconds() as f64) * 100.0
                } else {
                    0.0
                };

                report.push_str(&format!("- {} **{}:** {} ({})\n",
                    mode.symbol(),
                    mode.name(),
                    format_duration(*time),
                    format_percent(percentage)));
            }
        }
        report.push_str("\n");
    }

    // Time & Productivity Section
    report.push_str("## Time & Productivity\n\n");

    let running_pct = if global.total_elapsed > Duration::zero() {
        (global.running_time.num_seconds() as f64 / global.total_elapsed.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Running Time:** {} ({})\n",
        format_duration(global.running_time), format_percent(running_pct)));

    let paused_pct = if global.total_elapsed > Duration::zero() {
        (global.paused_time.num_seconds() as f64 / global.total_elapsed.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Paused Time:** {} ({})\n",
        format_duration(global.paused_time), format_percent(paused_pct)));

    let idle_pct = if global.total_elapsed > Duration::zero() {
        (global.idle_time.num_seconds() as f64 / global.total_elapsed.num_seconds() as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Idle Time:** {} ({})\n",
        format_duration(global.idle_time), format_percent(idle_pct)));

    report.push_str(&format!("- **Average Session:** {}\n", format_duration(global.avg_session)));
    report.push_str(&format!("- **Longest Session:** {}\n", format_duration(global.longest_session)));
    report.push_str(&format!("- **Total Sessions:** {}\n", global.total_sessions));
    report.push_str(&format!("- **Interruptions:** {}\n\n", global.total_interruptions));

    // Estimation Accuracy Section
    report.push_str("## Estimation Accuracy\n\n");

    let over_pct = if global.done_count > 0 {
        (estimation.over_estimate_count as f64 / global.done_count as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Tasks Over Estimate:** {} ({} of completed)\n",
        estimation.over_estimate_count, format_percent(over_pct)));
    report.push_str(&format!("- **Time Over Estimate:** {} total\n",
        format_duration(estimation.over_estimate_time)));

    let under_pct = if global.done_count > 0 {
        (estimation.under_estimate_count as f64 / global.done_count as f64) * 100.0
    } else {
        0.0
    };
    report.push_str(&format!("- **Tasks Under Estimate:** {} ({} of completed)\n",
        estimation.under_estimate_count, format_percent(under_pct)));
    report.push_str(&format!("- **Time Under Estimate:** {} saved\n",
        format_duration(estimation.under_estimate_time)));
    report.push_str(&format!("- **Perfect Estimates:** {}\n", estimation.perfect_count));
    report.push_str(&format!("- **Average Accuracy:** {}\n\n",
        format_percent(estimation.avg_accuracy_percent)));

    // Task Completion Section
    report.push_str("## Task Completion\n\n");
    report.push_str(&format!("- **Completed Today:** {} tasks\n", completion.completed_count));
    report.push_str(&format!("- **Average Time to Complete:** {}\n",
        format_duration(completion.avg_completion_time)));

    if let Some((title, time)) = &completion.fastest_task {
        report.push_str(&format!("- **Fastest Task:** \"{}\" ({})\n", title, format_duration(*time)));
    }
    if let Some((title, time)) = &completion.longest_task {
        report.push_str(&format!("- **Longest Task:** \"{}\" ({})\n", title, format_duration(*time)));
    }
    report.push_str("\n");

    // Tag Analysis Section
    if !tag_stats.is_empty() {
        report.push_str("## Tag Analysis\n\n");

        let mut tags: Vec<_> = tag_stats.iter().collect();
        tags.sort_by(|a, b| b.1.elapsed.cmp(&a.1.elapsed)); // Sort by time spent

        for (tag, stats) in tags {
            report.push_str(&format!("### #{}\n\n", tag));
            report.push_str(&format!("- **Tasks:** {} (Done: {}, Active: {})\n",
                stats.task_count, stats.done_count, stats.active_count));
            report.push_str(&format!("- **Time:** {} / {} estimated\n",
                format_duration(stats.elapsed), format_duration(stats.estimate)));
            report.push_str(&format!("- **Estimation Accuracy:** {}\n",
                format_percent(stats.accuracy_percent)));
            report.push_str(&format!("- **Average Session:** {}\n\n",
                format_duration(stats.avg_session)));
        }
    }

    // Tasks Breakdown Section
    report.push_str("## Tasks Breakdown\n\n");

    // Done Tasks
    if !done.is_empty() {
        report.push_str("### Done Tasks\n\n");
        for task in &done {
            let tags_str = if !task.tags.is_empty() {
                format!(" ({})", task.tags.join(", "))
            } else {
                String::new()
            };

            let ratio = if task.track.estimate > Duration::zero() {
                (task.track.elapsed.num_seconds() as f64 / task.track.estimate.num_seconds() as f64) * 100.0
            } else {
                0.0
            };

            report.push_str(&format!("- [x] **{}**{}\n", task.title, tags_str));
            report.push_str(&format!("  - Time: {} / {} estimated ({})\n",
                format_duration(task.track.elapsed),
                format_duration(task.track.estimate),
                format_percent(ratio)));
            report.push_str(&format!("  - Sessions: {} | Interruptions: {}\n",
                task.session_count(), task.interruption_count()));
            if let Some(calendar_time) = task.calendar_time() {
                report.push_str(&format!("  - Calendar Time: {}\n",
                    format_duration(calendar_time)));
            }

            // Include subtasks if any
            if !task.subtasks.is_empty() {
                for subtask in &task.subtasks {
                    report.push_str(&format!("    - [x] {} ({} / {})\n",
                        subtask.title,
                        format_duration(subtask.track.elapsed),
                        format_duration(subtask.track.estimate)));
                }
            }
            report.push_str("\n");
        }
    }

    // Active Tasks
    if !active.is_empty() {
        report.push_str("### Active Tasks\n\n");
        for task in &active {
            let tags_str = if !task.tags.is_empty() {
                format!(" ({})", task.tags.join(", "))
            } else {
                String::new()
            };

            let ratio = if task.track.estimate > Duration::zero() {
                (task.track.elapsed.num_seconds() as f64 / task.track.estimate.num_seconds() as f64) * 100.0
            } else {
                0.0
            };

            let status_icon = match task.status {
                crate::domain::RunStatus::Running => "▶",
                crate::domain::RunStatus::Paused => "⏸",
                _ => " ",
            };

            report.push_str(&format!("- [{}] **{}**{}\n", status_icon, task.title, tags_str));
            report.push_str(&format!("  - Time: {} / {} estimated ({})\n",
                format_duration(task.track.elapsed),
                format_duration(task.track.estimate),
                format_percent(ratio)));
            report.push_str(&format!("  - Sessions: {} | Interruptions: {}\n",
                task.session_count(), task.interruption_count()));

            // Include subtasks if any
            if !task.subtasks.is_empty() {
                for subtask in &task.subtasks {
                    let sub_status = match subtask.status {
                        crate::domain::RunStatus::Running => "▶",
                        crate::domain::RunStatus::Paused => "⏸",
                        _ => " ",
                    };
                    report.push_str(&format!("    - [{}] {} ({} / {})\n",
                        sub_status,
                        subtask.title,
                        format_duration(subtask.track.elapsed),
                        format_duration(subtask.track.estimate)));
                }
            }
            report.push_str("\n");
        }
    }

    // Archived Tasks
    if !archived.is_empty() {
        report.push_str("### Archived Tasks\n\n");
        for task in &archived {
            let tags_str = if !task.tags.is_empty() {
                format!(" ({})", task.tags.join(", "))
            } else {
                String::new()
            };

            report.push_str(&format!("- [~] **{}**{}\n", task.title, tags_str));
            report.push_str(&format!("  - Time: {} / {} estimated\n",
                format_duration(task.track.elapsed),
                format_duration(task.track.estimate)));
            report.push_str("\n");
        }
    }

    // Determine output path
    let output = if let Some(path) = output_path {
        path
    } else {
        crate::persistence::ensure_centre_dir()?.join(format!("report-{}.md", report_date))
    };

    // Write report to file
    fs::write(&output, report)?;

    Ok(output)
}
