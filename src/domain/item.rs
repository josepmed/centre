use super::enums::{RunStatus, ScheduleDay};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

/// State transition event for tracking task history
#[derive(Debug, Clone)]
pub struct StateEvent {
    /// When the transition occurred
    pub timestamp: DateTime<Local>,
    /// Previous status (None for initial creation)
    pub from_status: Option<RunStatus>,
    /// New status after transition
    pub to_status: RunStatus,
}

impl StateEvent {
    pub fn new(from_status: Option<RunStatus>, to_status: RunStatus) -> Self {
        Self {
            timestamp: Local::now(),
            from_status,
            to_status,
        }
    }
}

/// Time tracking information for a task or subtask
#[derive(Debug, Clone)]
pub struct TimeTracking {
    /// Estimated duration for completion
    pub estimate: Duration,
    /// Total elapsed time accumulated
    pub elapsed: Duration,
    /// When the timer was started (not persisted)
    pub started_at: Option<Instant>,
}

impl TimeTracking {
    pub fn new(estimate: Duration) -> Self {
        Self {
            estimate,
            elapsed: Duration::zero(),
            started_at: None,
        }
    }

    /// Start the timer
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }

    /// Pause the timer and accumulate elapsed time
    pub fn pause(&mut self) {
        if let Some(started) = self.started_at.take() {
            let duration = started.elapsed();
            self.elapsed = self.elapsed + Duration::from_std(duration).unwrap_or(Duration::zero());
        }
    }

    /// Update elapsed time for a running timer (called on tick)
    pub fn tick(&mut self) {
        if let Some(started) = self.started_at {
            let duration = started.elapsed();
            let accumulated = Duration::from_std(duration).unwrap_or(Duration::zero());
            // Store the new baseline
            self.elapsed = self.elapsed + accumulated;
            self.started_at = Some(Instant::now());
        }
    }

    /// Check if elapsed time has reached or exceeded estimate
    pub fn is_over_estimate(&self) -> bool {
        self.elapsed >= self.estimate
    }

    /// Get the ratio of elapsed to estimate (0.0 to 1.0+)
    pub fn progress_ratio(&self) -> f64 {
        let estimate_secs = self.estimate.num_seconds() as f64;
        if estimate_secs == 0.0 {
            return 1.0;
        }
        let elapsed_secs = self.elapsed.num_seconds() as f64;
        elapsed_secs / estimate_secs
    }

    /// Get elapsed time in hours as a float
    pub fn elapsed_hours(&self) -> f64 {
        self.elapsed.num_seconds() as f64 / 3600.0
    }

    /// Get estimate time in hours as a float
    pub fn estimate_hours(&self) -> f64 {
        self.estimate.num_seconds() as f64 / 3600.0
    }

    /// Format elapsed time as "Xh Ym" (e.g., "1h 30m", "45m", "2h 5m")
    pub fn elapsed_formatted(&self) -> String {
        format_duration(self.elapsed)
    }

    /// Format estimate time as "Xh Ym" (e.g., "1h 30m", "45m", "2h 5m")
    pub fn estimate_formatted(&self) -> String {
        format_duration(self.estimate)
    }

    /// Create from hours (for parsing)
    pub fn from_hours(estimate_hours: f64, elapsed_hours: f64) -> Self {
        let estimate = Duration::seconds((estimate_hours * 3600.0) as i64);
        let elapsed = Duration::seconds((elapsed_hours * 3600.0) as i64);
        Self {
            estimate,
            elapsed,
            started_at: None,
        }
    }
}

/// Format a duration as "Xh Ym" (omits 0 values)
fn format_duration(duration: Duration) -> String {
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

/// A task or subtask item
#[derive(Debug, Clone)]
pub struct Item {
    /// Unique ID for internal references (not persisted in markdown)
    pub id: Uuid,
    /// Task title
    pub title: String,
    /// Multi-line notes
    pub notes: String,
    /// Time tracking info
    pub track: TimeTracking,
    /// Current status
    pub status: RunStatus,
    /// Schedule bucket
    pub schedule: ScheduleDay,
    /// Whether subtasks are expanded (for parent tasks)
    pub expanded: bool,
    /// Subtasks (one level deep)
    pub subtasks: Vec<Item>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// When the task was created
    pub created_at: DateTime<Local>,
    /// When the task was completed (if done)
    pub completed_at: Option<DateTime<Local>>,
    /// History of state transitions
    pub state_history: Vec<StateEvent>,
}

impl Item {
    pub fn new(title: String, estimate: Duration, schedule: ScheduleDay) -> Self {
        let created_at = Local::now();
        let initial_event = StateEvent::new(None, RunStatus::Idle);

        Self {
            id: Uuid::new_v4(),
            title,
            notes: String::new(),
            track: TimeTracking::new(estimate),
            status: RunStatus::Idle,
            schedule,
            expanded: true,
            subtasks: Vec::new(),
            tags: Vec::new(),
            created_at,
            completed_at: None,
            state_history: vec![initial_event],
        }
    }

    /// Start running this item
    pub fn start(&mut self) {
        if self.status != RunStatus::Running {
            let prev_status = self.status;
            self.status = RunStatus::Running;
            self.track.start();
            self.state_history.push(StateEvent::new(Some(prev_status), RunStatus::Running));
        }
    }

    /// Pause this item
    pub fn pause(&mut self) {
        if self.status == RunStatus::Running {
            let prev_status = self.status;
            self.status = RunStatus::Paused;
            self.track.pause();
            self.state_history.push(StateEvent::new(Some(prev_status), RunStatus::Paused));
        }
    }

    /// Set this item to idle (stops timer if running)
    pub fn set_idle(&mut self) {
        if self.status == RunStatus::Running || self.status == RunStatus::Paused {
            if self.status == RunStatus::Running {
                self.track.pause();
            }
            let prev_status = self.status;
            self.status = RunStatus::Idle;
            self.state_history.push(StateEvent::new(Some(prev_status), RunStatus::Idle));
        }
    }

    /// Toggle between running and paused
    pub fn toggle_run_pause(&mut self) {
        match self.status {
            RunStatus::Idle | RunStatus::Paused => self.start(),
            RunStatus::Running => self.pause(),
            _ => {}
        }
    }

    /// Mark as done
    pub fn mark_done(&mut self) {
        if self.status == RunStatus::Running {
            self.track.pause();
        }
        let prev_status = self.status;
        self.status = RunStatus::Done;
        self.completed_at = Some(Local::now());
        self.state_history.push(StateEvent::new(Some(prev_status), RunStatus::Done));
    }

    /// Postpone to tomorrow - pauses if running and sets to Idle
    pub fn postpone(&mut self) {
        if self.status == RunStatus::Running {
            self.track.pause();
        }
        if self.status != RunStatus::Idle {
            let prev_status = self.status;
            self.status = RunStatus::Idle;
            self.state_history.push(StateEvent::new(Some(prev_status), RunStatus::Idle));
        }
    }

    /// Update elapsed time if running (called on tick)
    pub fn tick(&mut self) {
        if self.status == RunStatus::Running {
            self.track.tick();
        }
        // Tick all subtasks too
        for subtask in &mut self.subtasks {
            subtask.tick();
        }
    }

    /// Increase estimate by a duration
    pub fn increase_estimate(&mut self, amount: Duration) {
        self.track.estimate = self.track.estimate + amount;
    }

    /// Decrease estimate by a duration (minimum 0)
    pub fn decrease_estimate(&mut self, amount: Duration) {
        self.track.estimate = std::cmp::max(Duration::zero(), self.track.estimate - amount);
    }

    /// Check if this item has hit its estimate
    pub fn is_over_estimate(&self) -> bool {
        self.status == RunStatus::Running && self.track.is_over_estimate()
    }

    /// Add a subtask
    pub fn add_subtask(&mut self, subtask: Item) {
        self.subtasks.push(subtask);
    }

    /// Check if this item has any running subtasks
    pub fn has_running_subtasks(&self) -> bool {
        self.subtasks.iter().any(|st| st.status == RunStatus::Running)
    }

    /// Get total estimate from all subtasks
    pub fn subtask_total_estimate(&self) -> Duration {
        self.subtasks
            .iter()
            .map(|st| st.track.estimate)
            .fold(Duration::zero(), |acc, est| acc + est)
    }

    /// Get total elapsed time from all subtasks
    pub fn subtask_total_elapsed(&self) -> Duration {
        self.subtasks
            .iter()
            .map(|st| st.track.elapsed)
            .fold(Duration::zero(), |acc, elapsed| acc + elapsed)
    }

    /// Coerce all running items to paused (for startup)
    pub fn coerce_running_to_paused(&mut self) {
        if self.status == RunStatus::Running {
            self.status = RunStatus::Paused;
        }
        for subtask in &mut self.subtasks {
            subtask.coerce_running_to_paused();
        }
    }

    /// Sync elapsed time from state history
    /// This should be called after loading from disk to ensure elapsed matches actual history
    pub fn sync_elapsed_from_history(&mut self) {
        // Get running time from history
        let (running, paused, idle) = self.time_in_each_state();

        // Update elapsed to match actual running time
        self.track.elapsed = running;

        // Recursively sync subtasks
        for subtask in &mut self.subtasks {
            subtask.sync_elapsed_from_history();
        }
    }

    /// Regenerate UUIDs (for items loaded from disk)
    pub fn regenerate_ids(&mut self) {
        self.id = Uuid::new_v4();
        for subtask in &mut self.subtasks {
            subtask.regenerate_ids();
        }
    }

    /// Calculate total calendar time (created to completed)
    pub fn calendar_time(&self) -> Option<Duration> {
        self.completed_at.map(|completed| completed.signed_duration_since(self.created_at))
    }

    /// Calculate time spent in each state from state history
    /// Returns (running_time, paused_time, idle_time)
    pub fn time_in_each_state(&self) -> (Duration, Duration, Duration) {
        let mut running_time = Duration::zero();
        let mut paused_time = Duration::zero();
        let mut idle_time = Duration::zero();

        // If no history, return zeros
        if self.state_history.is_empty() {
            return (running_time, paused_time, idle_time);
        }

        // Iterate through state transitions and calculate time in each state
        for i in 0..self.state_history.len() {
            let current_event = &self.state_history[i];

            // Determine when this state ended (next transition or now)
            let state_end = if i + 1 < self.state_history.len() {
                self.state_history[i + 1].timestamp
            } else {
                // For the last (current) state, use now or completed_at
                self.completed_at.unwrap_or_else(chrono::Local::now)
            };

            // Calculate duration in this state
            let duration = state_end.signed_duration_since(current_event.timestamp);

            // Add to appropriate counter based on the state we transitioned TO
            match current_event.to_status {
                RunStatus::Running => running_time = running_time + duration,
                RunStatus::Paused => paused_time = paused_time + duration,
                RunStatus::Idle => idle_time = idle_time + duration,
                RunStatus::Done | RunStatus::Postponed => {
                    // Don't count time in Done/Postponed states
                }
            }
        }

        (running_time, paused_time, idle_time)
    }

    /// Calculate total time spent in RUNNING state based on state history
    pub fn running_time(&self) -> Duration {
        let mut total = Duration::zero();
        let mut last_running_start: Option<DateTime<Local>> = None;

        for i in 0..self.state_history.len() {
            let event = &self.state_history[i];

            if event.to_status == RunStatus::Running {
                last_running_start = Some(event.timestamp);
            } else if let Some(start) = last_running_start {
                // Transitioned away from RUNNING
                total = total + event.timestamp.signed_duration_since(start);
                last_running_start = None;
            }
        }

        // If still running, add time until now (or completed_at)
        if let Some(start) = last_running_start {
            let end = self.completed_at.unwrap_or_else(|| Local::now());
            total = total + end.signed_duration_since(start);
        }

        total
    }

    /// Count number of interruptions (transitions from RUNNING to PAUSED)
    pub fn interruption_count(&self) -> usize {
        self.state_history
            .iter()
            .filter(|event| {
                event.from_status == Some(RunStatus::Running) && event.to_status == RunStatus::Paused
            })
            .count()
    }

    /// Count number of work sessions (transitions to RUNNING)
    pub fn session_count(&self) -> usize {
        self.state_history
            .iter()
            .filter(|event| event.to_status == RunStatus::Running)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_tracking_new() {
        let estimate = Duration::hours(2);
        let track = TimeTracking::new(estimate);
        assert_eq!(track.estimate, estimate);
        assert_eq!(track.elapsed, Duration::zero());
        assert!(track.started_at.is_none());
    }

    #[test]
    fn test_time_tracking_from_hours() {
        let track = TimeTracking::from_hours(2.5, 1.25);
        assert_eq!(track.estimate_hours(), 2.5);
        assert_eq!(track.elapsed_hours(), 1.25);
    }

    #[test]
    fn test_time_tracking_formatted() {
        // Test various time formats
        let track1 = TimeTracking::from_hours(1.5, 0.75);
        assert_eq!(track1.estimate_formatted(), "1h 30m");
        assert_eq!(track1.elapsed_formatted(), "45m");

        let track2 = TimeTracking::from_hours(2.0, 1.0);
        assert_eq!(track2.estimate_formatted(), "2h");
        assert_eq!(track2.elapsed_formatted(), "1h");

        let track3 = TimeTracking::from_hours(0.5, 0.25);
        assert_eq!(track3.estimate_formatted(), "30m");
        assert_eq!(track3.elapsed_formatted(), "15m");

        // Test with exact minute values (5/60 = 0.08333...)
        let track4 = TimeTracking {
            estimate: Duration::minutes(125), // 2h 5m
            elapsed: Duration::minutes(65),   // 1h 5m
            started_at: None,
        };
        assert_eq!(track4.estimate_formatted(), "2h 5m");
        assert_eq!(track4.elapsed_formatted(), "1h 5m");
    }

    #[test]
    fn test_time_tracking_progress_ratio() {
        let mut track = TimeTracking::from_hours(2.0, 1.0);
        assert_eq!(track.progress_ratio(), 0.5);

        track.elapsed = Duration::hours(2);
        assert_eq!(track.progress_ratio(), 1.0);

        track.elapsed = Duration::hours(3);
        assert_eq!(track.progress_ratio(), 1.5);
    }

    #[test]
    fn test_time_tracking_is_over_estimate() {
        let mut track = TimeTracking::from_hours(2.0, 1.0);
        assert!(!track.is_over_estimate());

        track.elapsed = Duration::hours(2);
        assert!(track.is_over_estimate());

        track.elapsed = Duration::hours(3);
        assert!(track.is_over_estimate());
    }

    #[test]
    fn test_item_new() {
        let item = Item::new("Test task".to_string(), Duration::hours(1), ScheduleDay::Today);
        assert_eq!(item.title, "Test task");
        assert_eq!(item.status, RunStatus::Idle);
        assert_eq!(item.schedule, ScheduleDay::Today);
        assert!(item.expanded);
        assert!(item.subtasks.is_empty());
    }

    #[test]
    fn test_item_toggle_run_pause() {
        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);

        // Idle -> Running
        item.toggle_run_pause();
        assert_eq!(item.status, RunStatus::Running);
        assert!(item.track.started_at.is_some());

        // Running -> Paused
        item.toggle_run_pause();
        assert_eq!(item.status, RunStatus::Paused);
        assert!(item.track.started_at.is_none());

        // Paused -> Running
        item.toggle_run_pause();
        assert_eq!(item.status, RunStatus::Running);
    }

    #[test]
    fn test_item_mark_done() {
        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);
        item.start();
        item.mark_done();
        assert_eq!(item.status, RunStatus::Done);
        assert!(item.track.started_at.is_none());
    }

    #[test]
    fn test_item_postpone() {
        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);
        item.start();
        item.postpone();
        assert_eq!(item.status, RunStatus::Idle); // Postpone sets to Idle
        assert!(item.track.started_at.is_none()); // Pauses the timer
    }

    #[test]
    fn test_item_estimate_adjustment() {
        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);

        item.increase_estimate(Duration::minutes(30));
        assert_eq!(item.track.estimate, Duration::minutes(90));

        item.decrease_estimate(Duration::minutes(30));
        assert_eq!(item.track.estimate, Duration::hours(1));

        // Test minimum (shouldn't go negative)
        item.decrease_estimate(Duration::hours(2));
        assert_eq!(item.track.estimate, Duration::zero());
    }

    #[test]
    fn test_item_coerce_running_to_paused() {
        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);
        item.start();

        let mut subtask = Item::new("Subtask".to_string(), Duration::minutes(30), ScheduleDay::Today);
        subtask.start();
        item.add_subtask(subtask);

        item.coerce_running_to_paused();

        assert_eq!(item.status, RunStatus::Paused);
        assert_eq!(item.subtasks[0].status, RunStatus::Paused);
    }

    #[test]
    fn test_sync_elapsed_from_history() {
        use chrono::Local;

        let mut item = Item::new("Test".to_string(), Duration::hours(1), ScheduleDay::Today);

        // Manually create state history that simulates 30 minutes of running time
        let now = Local::now();
        let start1 = now - chrono::Duration::minutes(40);
        let end1 = now - chrono::Duration::minutes(30); // 10 min running
        let start2 = now - chrono::Duration::minutes(20);
        let end2 = now - chrono::Duration::minutes(0); // 20 min running
                                                        // Total: 30 minutes

        // Add state transitions
        item.state_history.push(StateEvent {
            timestamp: start1,
            from_status: Some(RunStatus::Idle),
            to_status: RunStatus::Running,
        });
        item.state_history.push(StateEvent {
            timestamp: end1,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });
        item.state_history.push(StateEvent {
            timestamp: start2,
            from_status: Some(RunStatus::Paused),
            to_status: RunStatus::Running,
        });
        item.state_history.push(StateEvent {
            timestamp: end2,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });

        // Initially elapsed is zero (simulating loaded from disk)
        item.track.elapsed = Duration::zero();

        // Sync elapsed from history
        item.sync_elapsed_from_history();

        // Verify elapsed matches the running time from history (~30 minutes)
        let expected = Duration::minutes(30);
        let tolerance = Duration::seconds(1); // Allow 1 second tolerance
        assert!(
            (item.track.elapsed - expected).num_seconds().abs() < tolerance.num_seconds(),
            "Expected elapsed to be ~30 minutes, got {} minutes",
            item.track.elapsed.num_minutes()
        );
    }

    #[test]
    fn test_sync_elapsed_from_history_with_subtasks() {
        use chrono::Local;

        let mut item = Item::new("Parent".to_string(), Duration::hours(2), ScheduleDay::Today);
        let mut subtask = Item::new("Child".to_string(), Duration::hours(1), ScheduleDay::Today);

        // Parent: 20 minutes of running time
        let now = Local::now();
        let start = now - chrono::Duration::minutes(20);
        item.state_history.push(StateEvent {
            timestamp: start,
            from_status: Some(RunStatus::Idle),
            to_status: RunStatus::Running,
        });
        item.state_history.push(StateEvent {
            timestamp: now,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });

        // Subtask: 10 minutes of running time
        let sub_start = now - chrono::Duration::minutes(10);
        subtask.state_history.push(StateEvent {
            timestamp: sub_start,
            from_status: Some(RunStatus::Idle),
            to_status: RunStatus::Running,
        });
        subtask.state_history.push(StateEvent {
            timestamp: now,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });

        item.add_subtask(subtask);

        // Set elapsed to incorrect values
        item.track.elapsed = Duration::minutes(5);
        item.subtasks[0].track.elapsed = Duration::minutes(2);

        // Sync from history
        item.sync_elapsed_from_history();

        // Verify both parent and subtask are synced
        let tolerance = Duration::seconds(1);
        assert!(
            (item.track.elapsed - Duration::minutes(20)).num_seconds().abs() < tolerance.num_seconds(),
            "Expected parent elapsed to be ~20 minutes, got {} minutes",
            item.track.elapsed.num_minutes()
        );
        assert!(
            (item.subtasks[0].track.elapsed - Duration::minutes(10)).num_seconds().abs()
                < tolerance.num_seconds(),
            "Expected subtask elapsed to be ~10 minutes, got {} minutes",
            item.subtasks[0].track.elapsed.num_minutes()
        );
    }
}
