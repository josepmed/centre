use crate::domain::Item;
use chrono::Duration;
use std::collections::{HashMap, HashSet};

/// Global statistics for all tasks
#[derive(Debug)]
pub struct GlobalStats {
    pub total_tasks: usize,
    pub active_count: usize,
    pub done_count: usize,
    pub archived_count: usize,
    pub total_elapsed: Duration,
    pub total_estimate: Duration,
    pub running_time: Duration,
    pub paused_time: Duration,
    pub idle_time: Duration,
    pub total_sessions: usize,
    pub total_interruptions: usize,
    pub avg_session: Duration,
    pub longest_session: Duration,
}

/// Estimation accuracy statistics
#[derive(Debug)]
pub struct EstimationStats {
    pub over_estimate_count: usize,
    pub over_estimate_time: Duration,
    pub under_estimate_count: usize,
    pub under_estimate_time: Duration,
    pub perfect_count: usize,
    pub avg_accuracy_percent: f64,
}

/// Task completion statistics
#[derive(Debug)]
pub struct CompletionStats {
    pub completed_count: usize,
    pub avg_completion_time: Duration,
    pub fastest_task: Option<(String, Duration)>,
    pub longest_task: Option<(String, Duration)>,
}

/// Per-tag statistics
#[derive(Debug)]
pub struct TagStats {
    pub task_count: usize,
    pub done_count: usize,
    pub active_count: usize,
    pub elapsed: Duration,
    pub estimate: Duration,
    pub accuracy_percent: f64,
    pub avg_session: Duration,
}

/// Calculate global statistics across all tasks
pub fn calculate_global_stats(
    active: &[Item],
    done: &[Item],
    archived: &[Item],
) -> GlobalStats {
    let all_tasks: Vec<&Item> = active.iter().chain(done.iter()).chain(archived.iter()).collect();

    let total_tasks = all_tasks.len();
    let active_count = active.len();
    let done_count = done.len();
    let archived_count = archived.len();

    let mut total_elapsed = Duration::zero();
    let mut total_estimate = Duration::zero();
    let mut running_time = Duration::zero();
    let mut paused_time = Duration::zero();
    let mut idle_time = Duration::zero();
    let mut total_sessions = 0;
    let mut total_interruptions = 0;
    let mut all_sessions = Vec::new();

    for task in &all_tasks {
        total_elapsed = total_elapsed + task.track.elapsed;
        total_estimate = total_estimate + task.track.estimate;

        let (task_running, task_paused, task_idle) = task.time_in_each_state();
        running_time = running_time + task_running;
        paused_time = paused_time + task_paused;
        idle_time = idle_time + task_idle;

        total_sessions += task.session_count();
        total_interruptions += task.interruption_count();

        // Calculate individual session durations
        let session_count = task.session_count();
        if session_count > 0 {
            let avg_task_session = task_running.num_milliseconds() / session_count as i64;
            all_sessions.push(Duration::milliseconds(avg_task_session));
        }
    }

    let avg_session = if !all_sessions.is_empty() {
        let total_ms: i64 = all_sessions.iter().map(|d| d.num_milliseconds()).sum();
        Duration::milliseconds(total_ms / all_sessions.len() as i64)
    } else {
        Duration::zero()
    };

    let longest_session = all_sessions.into_iter().max().unwrap_or(Duration::zero());

    GlobalStats {
        total_tasks,
        active_count,
        done_count,
        archived_count,
        total_elapsed,
        total_estimate,
        running_time,
        paused_time,
        idle_time,
        total_sessions,
        total_interruptions,
        avg_session,
        longest_session,
    }
}

/// Calculate estimation accuracy statistics
pub fn calculate_estimation_stats(items: &[Item]) -> EstimationStats {
    let mut over_estimate_count = 0;
    let mut over_estimate_time = Duration::zero();
    let mut under_estimate_count = 0;
    let mut under_estimate_time = Duration::zero();
    let mut perfect_count = 0;
    let mut accuracy_sum = 0.0;
    let mut accuracy_count = 0;

    for task in items {
        let elapsed = task.track.elapsed;
        let estimate = task.track.estimate;

        if elapsed > estimate {
            over_estimate_count += 1;
            over_estimate_time = over_estimate_time + (elapsed - estimate);
        } else if elapsed < estimate {
            under_estimate_count += 1;
            under_estimate_time = under_estimate_time + (estimate - elapsed);
        } else {
            perfect_count += 1;
        }

        // Calculate accuracy percentage (0-100%, where 100% = perfect estimate)
        if estimate > Duration::zero() {
            let ratio = elapsed.num_seconds() as f64 / estimate.num_seconds() as f64;
            let accuracy = if ratio > 1.0 {
                100.0 / ratio // Over-estimate: penalize
            } else {
                ratio * 100.0 // Under-estimate: also shows efficiency
            };
            accuracy_sum += accuracy;
            accuracy_count += 1;
        }
    }

    let avg_accuracy_percent = if accuracy_count > 0 {
        accuracy_sum / accuracy_count as f64
    } else {
        0.0
    };

    EstimationStats {
        over_estimate_count,
        over_estimate_time,
        under_estimate_count,
        under_estimate_time,
        perfect_count,
        avg_accuracy_percent,
    }
}

/// Calculate task completion statistics
pub fn calculate_completion_stats(done_tasks: &[Item]) -> CompletionStats {
    let completed_count = done_tasks.len();

    let mut total_time = Duration::zero();
    let mut fastest: Option<(String, Duration)> = None;
    let mut longest: Option<(String, Duration)> = None;

    for task in done_tasks {
        let elapsed = task.track.elapsed;
        total_time = total_time + elapsed;

        // Track fastest
        if let Some((_, fastest_time)) = &fastest {
            if elapsed < *fastest_time {
                fastest = Some((task.title.clone(), elapsed));
            }
        } else {
            fastest = Some((task.title.clone(), elapsed));
        }

        // Track longest
        if let Some((_, longest_time)) = &longest {
            if elapsed > *longest_time {
                longest = Some((task.title.clone(), elapsed));
            }
        } else {
            longest = Some((task.title.clone(), elapsed));
        }
    }

    let avg_completion_time = if completed_count > 0 {
        Duration::milliseconds(total_time.num_milliseconds() / completed_count as i64)
    } else {
        Duration::zero()
    };

    CompletionStats {
        completed_count,
        avg_completion_time,
        fastest_task: fastest,
        longest_task: longest,
    }
}

/// Calculate per-tag statistics
pub fn calculate_tag_stats(
    active: &[Item],
    done: &[Item],
    archived: &[Item],
) -> HashMap<String, TagStats> {
    let mut tag_map: HashMap<String, TagStats> = HashMap::new();

    let all_tasks: Vec<&Item> = active.iter().chain(done.iter()).chain(archived.iter()).collect();

    // Create ID sets for efficient lookup
    let done_ids: std::collections::HashSet<_> = done.iter().map(|t| t.id).collect();
    let active_ids: std::collections::HashSet<_> = active.iter().map(|t| t.id).collect();

    for task in &all_tasks {
        for tag in &task.tags {
            let entry = tag_map.entry(tag.clone()).or_insert(TagStats {
                task_count: 0,
                done_count: 0,
                active_count: 0,
                elapsed: Duration::zero(),
                estimate: Duration::zero(),
                accuracy_percent: 0.0,
                avg_session: Duration::zero(),
            });

            entry.task_count += 1;
            entry.elapsed = entry.elapsed + task.track.elapsed;
            entry.estimate = entry.estimate + task.track.estimate;

            if done_ids.contains(&task.id) {
                entry.done_count += 1;
            } else if active_ids.contains(&task.id) {
                entry.active_count += 1;
            }
        }
    }

    // Calculate accuracy and avg session for each tag
    for (tag, stats) in tag_map.iter_mut() {
        if stats.estimate > Duration::zero() {
            let ratio = stats.elapsed.num_seconds() as f64 / stats.estimate.num_seconds() as f64;
            stats.accuracy_percent = if ratio > 1.0 {
                100.0 / ratio
            } else {
                ratio * 100.0
            };
        }

        // Calculate average session for this tag
        let tag_tasks: Vec<&Item> = all_tasks.iter().filter(|t| t.tags.contains(tag)).copied().collect();
        let mut total_sessions = 0;
        let mut total_running = Duration::zero();

        for task in tag_tasks {
            let sessions = task.session_count();
            total_sessions += sessions;
            let (running, _, _) = task.time_in_each_state();
            total_running = total_running + running;
        }

        if total_sessions > 0 {
            stats.avg_session = Duration::milliseconds(total_running.num_milliseconds() / total_sessions as i64);
        }
    }

    tag_map
}
