use serde::{Deserialize, Serialize};

/// Schedule bucket for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleDay {
    Today,
    Tomorrow,
}

/// Runtime status of a task or subtask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Idle,
    Running,
    Paused,
    Done,
    Postponed,
}

impl RunStatus {
    /// Parse status from markdown tag like "[RUNNING]"
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag.to_uppercase().as_str() {
            "IDLE" => Some(Self::Idle),
            "RUNNING" => Some(Self::Running),
            "PAUSED" => Some(Self::Paused),
            "DONE" => Some(Self::Done),
            "POSTPONED" => Some(Self::Postponed),
            _ => None,
        }
    }

    /// Convert status to markdown tag
    pub fn to_tag(&self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Running => "RUNNING",
            Self::Paused => "PAUSED",
            Self::Done => "DONE",
            Self::Postponed => "POSTPONED",
        }
    }

    /// Check if status is valid for today.md/tomorrow.md (excludes DONE/POSTPONED)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Idle | Self::Running | Self::Paused)
    }
}

/// UI mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Normal,
    EditingNotes,
    EditingEstimate,
    Modal,
    AddingTask,
    AddingSubtask,
    EditingTask, // Editing an existing task/subtask
    IdleCheck,
    EditingJournal,
    DayChanged, // Shown when midnight has passed, forces restart
    ModeSelector, // Shown when user presses 'm' to select global mode
}

/// Global activity state for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalState {
    /// At least one task is RUNNING
    Running,
    /// No tasks RUNNING, but at least one is PAUSED
    Paused,
    /// All tasks are IDLE (or no tasks)
    Idle,
}

/// Global context mode representing the user's current life state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalMode {
    /// Active focus time. Default mode. Timers run normally.
    Working,
    /// Short break. All tasks paused.
    Break,
    /// Lunch break. All tasks paused.
    Lunch,
    /// Gym/exercise time. All tasks paused.
    Gym,
    /// Dinner time. All tasks paused.
    Dinner,
    /// Personal errands. All tasks paused.
    Personal,
    /// Sleep/night mode. All tasks paused.
    Sleep,
}

impl GlobalMode {
    /// Get the emoji symbol for this mode
    pub fn symbol(&self) -> &'static str {
        match self {
            GlobalMode::Working => "💼",
            GlobalMode::Break => "☁️",
            GlobalMode::Lunch => "🍽",
            GlobalMode::Gym => "🏋️",
            GlobalMode::Dinner => "🍲",
            GlobalMode::Personal => "🏡",
            GlobalMode::Sleep => "🌙",
        }
    }

    /// Get the display name for this mode
    pub fn name(&self) -> &'static str {
        match self {
            GlobalMode::Working => "Working",
            GlobalMode::Break => "Break",
            GlobalMode::Lunch => "Lunch",
            GlobalMode::Gym => "Gym",
            GlobalMode::Dinner => "Dinner",
            GlobalMode::Personal => "Personal",
            GlobalMode::Sleep => "Sleep",
        }
    }

    /// Get the contextual phrase for this mode (for Focus Garden)
    pub fn contextual_phrase(&self) -> &'static str {
        match self {
            GlobalMode::Working => "",
            GlobalMode::Break => "Breathe and reset ☁️",
            GlobalMode::Lunch => "Nourish before you bloom again 🍽",
            GlobalMode::Gym => "Strength feeds focus 🏋️",
            GlobalMode::Dinner => "Evening nourishment 🍲",
            GlobalMode::Personal => "Tending your own garden 🏡",
            GlobalMode::Sleep => "Rest — tomorrow's seeds await 🌙",
        }
    }

    /// Check if this mode should pause timers
    pub fn should_pause_timers(&self) -> bool {
        !matches!(self, GlobalMode::Working)
    }

    /// Get all modes as a list
    pub fn all() -> &'static [GlobalMode] {
        &[
            GlobalMode::Working,
            GlobalMode::Break,
            GlobalMode::Lunch,
            GlobalMode::Gym,
            GlobalMode::Dinner,
            GlobalMode::Personal,
            GlobalMode::Sleep,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_status_from_tag() {
        assert_eq!(RunStatus::from_tag("IDLE"), Some(RunStatus::Idle));
        assert_eq!(RunStatus::from_tag("RUNNING"), Some(RunStatus::Running));
        assert_eq!(RunStatus::from_tag("PAUSED"), Some(RunStatus::Paused));
        assert_eq!(RunStatus::from_tag("running"), Some(RunStatus::Running));
        assert_eq!(RunStatus::from_tag("INVALID"), None);
    }

    #[test]
    fn test_run_status_to_tag() {
        assert_eq!(RunStatus::Idle.to_tag(), "IDLE");
        assert_eq!(RunStatus::Running.to_tag(), "RUNNING");
        assert_eq!(RunStatus::Paused.to_tag(), "PAUSED");
    }

    #[test]
    fn test_run_status_is_active() {
        assert!(RunStatus::Idle.is_active());
        assert!(RunStatus::Running.is_active());
        assert!(RunStatus::Paused.is_active());
        assert!(!RunStatus::Done.is_active());
        assert!(!RunStatus::Postponed.is_active());
    }
}
