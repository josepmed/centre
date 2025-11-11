use crate::domain::{compute_totals, flatten_tasks, GlobalMode, GlobalState, Item, RunStatus, ScheduleDay, StateEvent, UiMode};
use crate::notifications;
use anyhow::Result;
use chrono::{Duration, Timelike};
use std::time::Instant;
use uuid::Uuid;

/// Modal state for estimate-hit prompt
#[derive(Debug, Clone)]
pub struct ModalState {
    pub item_id: Uuid,
    pub message: String,
}

/// Input form state for adding tasks
#[derive(Debug, Clone)]
pub struct InputFormState {
    pub title: String,
    pub notes: String,
    pub tags: String, // Comma-separated tags
    pub is_subtask: bool,
    pub editing_field: usize, // 0 = title, 1 = notes, 2 = tags
}

/// Undo action for reverting recent changes
#[derive(Debug, Clone)]
pub enum UndoAction {
    MarkedDone {
        item: Item,
        was_subtask: bool,
        parent_task_index: Option<usize>,
        subtask_index: Option<usize>,
        task_index: usize,
    },
    Deleted {
        item: Item,
        was_subtask: bool,
        parent_task_index: Option<usize>,
        subtask_index: Option<usize>,
        task_index: usize,
    },
    Archived {
        item: Item,
        was_subtask: bool,
        parent_task_index: Option<usize>,
        subtask_index: Option<usize>,
        task_index: usize,
    },
}

/// Main application state
pub struct AppState {
    pub tasks: Vec<Item>,
    pub done_today: Vec<Item>,
    pub archived_today: Vec<Item>,
    pub selected_index: usize,
    pub ui_mode: UiMode,
    pub modal: Option<ModalState>,
    pub input_form: Option<InputFormState>,
    pub last_tick: Instant,
    pub last_idle_check: Instant,
    pub idle_check_deadline: Option<Instant>,
    pub use_emoji: bool,
    pub estimate_step: Duration,
    pub needs_save: bool,
    pub show_done: bool,
    pub journal_content: String,
    pub journal_needs_save: bool,
    pub journal_cursor_pos: usize, // Cursor position in journal
    pub file_date: chrono::NaiveDate, // Track which day's file we're using
    pub undo_stack: Vec<UndoAction>, // Track recent actions for undo

    // Global activity tracking for Focus Garden
    pub app_start_time: Instant,
    pub running_time: Duration,
    pub paused_time: Duration,
    pub idle_time: Duration,
    pub last_global_state: GlobalState,
    pub last_state_change: Instant,
    pub current_session_start: Option<Instant>,
    pub completed_sessions: Vec<Duration>,
    pub last_phrase_rotation: Instant,
    pub current_phrase_index: usize,

    // Global mode tracking
    pub global_mode: GlobalMode,
    pub last_mode_change: Instant,
    pub mode_time_working: Duration,
    pub mode_time_break: Duration,
    pub mode_time_lunch: Duration,
    pub mode_time_gym: Duration,
    pub mode_time_dinner: Duration,
    pub mode_time_personal: Duration,
    pub mode_time_sleep: Duration,
    pub paused_by_mode_task_ids: Vec<Uuid>, // Tasks that were paused by mode change

    // Animation frame counter for ASCII animations (increments every tick)
    pub animation_frame: u32,

    // Scroll offset for done pane
    pub done_scroll_offset: usize,
}

impl AppState {
    pub fn new(tasks: Vec<Item>, done_today: Vec<Item>, archived_today: Vec<Item>, journal_content: String) -> Self {
        let now = Instant::now();

        // Load metadata (global mode, etc.)
        let mut metadata = Self::load_metadata_internal().unwrap_or_default();

        // Check if it's a new day and reset mode times if needed
        let current_date = chrono::Local::now().date_naive();
        let should_reset_mode_times = if let Some(last_timestamp) = &metadata.last_mode_change_timestamp {
            // Parse the last timestamp and check if it's from a different day
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last_timestamp) {
                let last_date = last_time.date_naive();
                last_date != current_date
            } else {
                false
            }
        } else {
            // No timestamp means first run or old format - don't reset
            false
        };

        // Reset mode times if it's a new day
        if should_reset_mode_times {
            metadata.mode_time_working_secs = 0;
            metadata.mode_time_break_secs = 0;
            metadata.mode_time_lunch_secs = 0;
            metadata.mode_time_gym_secs = 0;
            metadata.mode_time_dinner_secs = 0;
            metadata.mode_time_personal_secs = 0;
            metadata.mode_time_sleep_secs = 0;
        }

        // Convert paused task IDs from strings to UUIDs
        let paused_by_mode_task_ids: Vec<Uuid> = metadata
            .paused_by_mode_task_ids
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        // Load mode times from metadata (stored as seconds)
        let mode_time_working = Duration::seconds(metadata.mode_time_working_secs);
        let mode_time_break = Duration::seconds(metadata.mode_time_break_secs);
        let mode_time_lunch = Duration::seconds(metadata.mode_time_lunch_secs);
        let mode_time_gym = Duration::seconds(metadata.mode_time_gym_secs);
        let mode_time_dinner = Duration::seconds(metadata.mode_time_dinner_secs);
        let mode_time_personal = Duration::seconds(metadata.mode_time_personal_secs);
        let mode_time_sleep = Duration::seconds(metadata.mode_time_sleep_secs);

        Self {
            tasks,
            done_today,
            archived_today,
            selected_index: 0,
            ui_mode: UiMode::Normal,
            modal: None,
            input_form: None,
            last_tick: now,
            last_idle_check: now,
            idle_check_deadline: None,
            use_emoji: true,
            estimate_step: Duration::minutes(15),
            needs_save: false,
            show_done: true,
            journal_content,
            journal_needs_save: false,
            journal_cursor_pos: 0,
            file_date: chrono::Local::now().date_naive(),
            undo_stack: Vec::new(),

            // Initialize global activity tracking
            app_start_time: now,
            running_time: Duration::zero(),
            paused_time: Duration::zero(),
            idle_time: Duration::zero(),
            last_global_state: GlobalState::Idle,
            last_state_change: now,
            current_session_start: None,
            completed_sessions: Vec::new(),
            last_phrase_rotation: now,
            current_phrase_index: 0,

            // Initialize global mode tracking from loaded metadata
            global_mode: metadata.global_mode,
            last_mode_change: now,
            mode_time_working,
            mode_time_break,
            mode_time_lunch,
            mode_time_gym,
            mode_time_dinner,
            mode_time_personal,
            mode_time_sleep,
            paused_by_mode_task_ids,

            // Initialize animation frame counter
            animation_frame: 0,

            // Initialize done scroll offset
            done_scroll_offset: 0,
        }
    }

    /// Load metadata from meta.json
    fn load_metadata_internal() -> Result<crate::persistence::AppMetadata> {
        use crate::persistence::{load_metadata, meta_file};
        let meta_path = meta_file()?;
        load_metadata(meta_path)
    }

    /// Save metadata to meta.json
    pub fn save_metadata(&self) -> Result<()> {
        use crate::persistence::{save_metadata, meta_file, AppMetadata};

        // Get current mode times (includes time in current mode up to now)
        let mode_times = self.get_mode_times();

        let metadata = AppMetadata {
            global_mode: self.global_mode,
            paused_by_mode_task_ids: self
                .paused_by_mode_task_ids
                .iter()
                .map(|id| id.to_string())
                .collect(),
            // Save mode times as seconds
            mode_time_working_secs: mode_times[0].1.num_seconds(),
            mode_time_break_secs: mode_times[1].1.num_seconds(),
            mode_time_lunch_secs: mode_times[2].1.num_seconds(),
            mode_time_gym_secs: mode_times[3].1.num_seconds(),
            mode_time_dinner_secs: mode_times[4].1.num_seconds(),
            mode_time_personal_secs: mode_times[5].1.num_seconds(),
            mode_time_sleep_secs: mode_times[6].1.num_seconds(),
            last_mode_change_timestamp: Some(chrono::Local::now().to_rfc3339()),
        };

        let meta_path = meta_file()?;
        save_metadata(meta_path, &metadata)
    }

    /// Check if the current date has changed (crossed midnight)
    pub fn has_day_changed(&self) -> bool {
        let current_date = chrono::Local::now().date_naive();
        current_date != self.file_date
    }

    /// Toggle showing done tasks
    pub fn toggle_show_done(&mut self) {
        self.show_done = !self.show_done;
        // Reset scroll when toggling view
        if self.show_done {
            self.reset_done_scroll();
        }
    }

    /// Get the currently selected item (returns task_index and optional subtask_index)
    pub fn get_selected_item(&self) -> Option<(usize, Option<usize>)> {
        let flat_rows = flatten_tasks(&self.tasks);
        if self.selected_index >= flat_rows.len() {
            return None;
        }

        let row = &flat_rows[self.selected_index];
        Some((row.task_index, row.subtask_index))
    }

    /// Get a mutable reference to the selected item
    pub fn get_selected_item_mut(&mut self) -> Option<&mut Item> {
        let (task_idx, subtask_idx) = self.get_selected_item()?;

        if let Some(st_idx) = subtask_idx {
            self.tasks.get_mut(task_idx)?.subtasks.get_mut(st_idx)
        } else {
            self.tasks.get_mut(task_idx)
        }
    }

    /// Move selection up
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_selection_down(&mut self) {
        let flat_rows = flatten_tasks(&self.tasks);
        if self.selected_index + 1 < flat_rows.len() {
            self.selected_index += 1;
        }
    }

    /// Move selected task/subtask up in the list
    pub fn move_item_up(&mut self) {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            if let Some(st_idx) = subtask_idx {
                // Moving a subtask up
                if st_idx > 0 {
                    self.tasks[task_idx].subtasks.swap(st_idx, st_idx - 1);
                    self.selected_index = self.selected_index.saturating_sub(1);
                    self.needs_save = true;
                }
            } else {
                // Moving a task up
                if task_idx > 0 {
                    self.tasks.swap(task_idx, task_idx - 1);
                    // Adjust selection to follow the moved task
                    self.selected_index = self.selected_index.saturating_sub(
                        if !self.tasks[task_idx].subtasks.is_empty() && self.tasks[task_idx].expanded {
                            self.tasks[task_idx].subtasks.len() + 1
                        } else {
                            1
                        }
                    );
                    self.needs_save = true;
                }
            }
        }
    }

    /// Move selected task/subtask down in the list
    pub fn move_item_down(&mut self) {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            if let Some(st_idx) = subtask_idx {
                // Moving a subtask down
                let subtask_count = self.tasks[task_idx].subtasks.len();
                if st_idx + 1 < subtask_count {
                    self.tasks[task_idx].subtasks.swap(st_idx, st_idx + 1);
                    self.selected_index += 1;
                    self.needs_save = true;
                }
            } else {
                // Moving a task down
                if task_idx + 1 < self.tasks.len() {
                    self.tasks.swap(task_idx, task_idx + 1);
                    // Adjust selection to follow the moved task
                    self.selected_index += if !self.tasks[task_idx + 1].subtasks.is_empty() && self.tasks[task_idx + 1].expanded {
                        self.tasks[task_idx + 1].subtasks.len() + 1
                    } else {
                        1
                    };
                    self.needs_save = true;
                }
            }
        }
    }

    /// Toggle run/pause for selected item
    pub fn toggle_run_pause(&mut self) {
        // Prevent starting tasks if not in Working mode
        if self.global_mode.should_pause_timers() {
            // Only allow pausing tasks in non-working modes, not starting them
            if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
                if let Some(st_idx) = subtask_idx {
                    // Only pause if running
                    if self.tasks[task_idx].subtasks[st_idx].status == RunStatus::Running {
                        self.tasks[task_idx].subtasks[st_idx].pause();
                        self.sync_parent_status(task_idx);
                        self.needs_save = true;
                    }
                } else {
                    // Only pause if running
                    if self.tasks[task_idx].status == RunStatus::Running {
                        self.tasks[task_idx].pause();
                        self.needs_save = true;
                    }
                }
            }
            return;
        }

        // Normal toggle behavior when in Working mode
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            // Toggle the selected item
            if let Some(st_idx) = subtask_idx {
                // Toggling a subtask
                self.tasks[task_idx].subtasks[st_idx].toggle_run_pause();

                // Sync parent task status based on subtasks
                self.sync_parent_status(task_idx);
            } else {
                // Toggling a parent task
                self.tasks[task_idx].toggle_run_pause();
            }
            self.needs_save = true;
        }
    }

    /// Sync parent task status based on subtask states
    /// Note: For tasks with subtasks, the parent timer runs when any subtask is running
    fn sync_parent_status(&mut self, task_idx: usize) {
        let parent = &mut self.tasks[task_idx];

        // Only sync if parent has subtasks
        if parent.subtasks.is_empty() {
            return;
        }

        // Check if any subtask is running
        let has_running = parent.subtasks.iter().any(|st| st.status == RunStatus::Running);

        if has_running {
            // If any subtask is running, start the parent timer
            if parent.status != RunStatus::Running {
                parent.start();
            }
        } else {
            // No subtasks running - check if all are paused/idle
            let all_paused_or_idle = parent.subtasks.iter()
                .all(|st| st.status == RunStatus::Paused || st.status == RunStatus::Idle);

            if all_paused_or_idle && parent.status == RunStatus::Running {
                // All subtasks paused/idle, so pause parent
                parent.pause();
            }
        }
    }

    /// Increase estimate for selected item
    pub fn increase_estimate(&mut self) {
        let step = self.estimate_step;
        if let Some(item) = self.get_selected_item_mut() {
            item.increase_estimate(step);
            self.needs_save = true;
        }
    }

    /// Decrease estimate for selected item
    pub fn decrease_estimate(&mut self) {
        let step = self.estimate_step;
        if let Some(item) = self.get_selected_item_mut() {
            item.decrease_estimate(step);
            self.needs_save = true;
        }
    }

    /// Mark selected item as done
    pub fn mark_done(&mut self) -> Result<()> {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            // Clone the item before removing it (for undo)
            let item_to_undo = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks[st_idx].clone()
            } else {
                self.tasks[task_idx].clone()
            };

            // Remove the item
            let mut item = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks.remove(st_idx)
            } else {
                self.tasks.remove(task_idx)
            };

            // Mark as done
            item.mark_done();

            // Send notification
            notifications::notify_task_done(&item.title);

            // Save undo information before adding to done_today
            self.undo_stack.push(UndoAction::MarkedDone {
                item: item_to_undo,
                was_subtask: subtask_idx.is_some(),
                parent_task_index: if subtask_idx.is_some() { Some(task_idx) } else { None },
                subtask_index: subtask_idx,
                task_index: task_idx,
            });

            // Keep only the last 10 undo actions
            if self.undo_stack.len() > 10 {
                self.undo_stack.remove(0);
            }

            // Add to done_today list (will be in DONE section of daily file)
            self.done_today.push(item);

            // Adjust selection if needed
            let flat_rows = flatten_tasks(&self.tasks);
            if self.selected_index >= flat_rows.len() && flat_rows.len() > 0 {
                self.selected_index = flat_rows.len() - 1;
            }

            self.needs_save = true;
        }

        Ok(())
    }

    /// Undo the last action (supports undoing mark_done, delete, and archive)
    pub fn undo(&mut self) -> Result<()> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::MarkedDone {
                    item,
                    was_subtask,
                    parent_task_index,
                    subtask_index: _,
                    task_index,
                } => {
                    // Find and remove the item from done_today
                    if let Some(done_idx) = self.done_today.iter().position(|i| i.id == item.id) {
                        self.done_today.remove(done_idx);
                    }

                    // Restore the item to its original position
                    if was_subtask {
                        // Restore as subtask
                        if let Some(parent_idx) = parent_task_index {
                            if parent_idx < self.tasks.len() {
                                // Insert at the end of subtasks (original position may not be valid anymore)
                                self.tasks[parent_idx].subtasks.push(item);
                            }
                        }
                    } else {
                        // Restore as task
                        // Insert at original position if valid, otherwise at the end
                        let insert_pos = task_index.min(self.tasks.len());
                        self.tasks.insert(insert_pos, item);
                    }

                    self.needs_save = true;
                }
                UndoAction::Deleted {
                    item,
                    was_subtask,
                    parent_task_index,
                    subtask_index: _,
                    task_index,
                } => {
                    // Restore the item to its original position
                    if was_subtask {
                        // Restore as subtask
                        if let Some(parent_idx) = parent_task_index {
                            if parent_idx < self.tasks.len() {
                                // Insert at the end of subtasks (original position may not be valid anymore)
                                self.tasks[parent_idx].subtasks.push(item);
                            }
                        }
                    } else {
                        // Restore as task
                        // Insert at original position if valid, otherwise at the end
                        let insert_pos = task_index.min(self.tasks.len());
                        self.tasks.insert(insert_pos, item);
                    }

                    self.needs_save = true;
                }
                UndoAction::Archived {
                    item,
                    was_subtask,
                    parent_task_index,
                    subtask_index: _,
                    task_index,
                } => {
                    // Find and remove the item from archived_today
                    if let Some(archived_idx) = self.archived_today.iter().position(|i| i.id == item.id) {
                        self.archived_today.remove(archived_idx);
                    }

                    // Restore the item to its original position
                    if was_subtask {
                        // Restore as subtask
                        if let Some(parent_idx) = parent_task_index {
                            if parent_idx < self.tasks.len() {
                                // Insert at the end of subtasks (original position may not be valid anymore)
                                self.tasks[parent_idx].subtasks.push(item);
                            }
                        }
                    } else {
                        // Restore as task
                        // Insert at original position if valid, otherwise at the end
                        let insert_pos = task_index.min(self.tasks.len());
                        self.tasks.insert(insert_pos, item);
                    }

                    self.needs_save = true;
                }
            }
        }

        Ok(())
    }

    /// Postpone selected item to tomorrow (will create/update tomorrow's file)
    pub fn postpone_to_tomorrow(&mut self) -> Result<()> {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            let mut item = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks.remove(st_idx)
            } else {
                self.tasks.remove(task_idx)
            };

            item.postpone();

            // Load tomorrow's file (if it exists), add the item, and save
            use crate::persistence::{daily_file, parse_daily_file, serialize_daily_file, atomic_write};
            let tomorrow_date = chrono::Local::now().date_naive() + chrono::Duration::days(1);
            let tomorrow_path = daily_file(tomorrow_date)?;

            let (mut tomorrow_active, tomorrow_done, tomorrow_archived) = if tomorrow_path.exists() {
                let content = std::fs::read_to_string(&tomorrow_path)?;
                parse_daily_file(&content)?
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            };

            // Add the postponed item to tomorrow's active tasks
            tomorrow_active.push(item);

            // Save tomorrow's file
            let tomorrow_content = serialize_daily_file(&tomorrow_active, &tomorrow_done, &tomorrow_archived);
            atomic_write(&tomorrow_path, &tomorrow_content)?;

            // Adjust selection if needed
            let flat_rows = flatten_tasks(&self.tasks);
            if self.selected_index >= flat_rows.len() && flat_rows.len() > 0 {
                self.selected_index = flat_rows.len() - 1;
            }

            self.needs_save = true;
        }

        Ok(())
    }

    /// Archive selected item (moves to ARCHIVED section of daily file)
    pub fn archive_selected(&mut self) -> Result<()> {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            // Clone the item before removing it (for undo)
            let item_to_undo = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks[st_idx].clone()
            } else {
                self.tasks[task_idx].clone()
            };

            // Remove the item
            let item = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks.remove(st_idx)
            } else {
                self.tasks.remove(task_idx)
            };

            // Add to archived_today list (will be in ARCHIVED section of daily file)
            self.archived_today.push(item);

            // Save undo information
            self.undo_stack.push(UndoAction::Archived {
                item: item_to_undo,
                was_subtask: subtask_idx.is_some(),
                parent_task_index: if subtask_idx.is_some() { Some(task_idx) } else { None },
                subtask_index: subtask_idx,
                task_index: task_idx,
            });

            // Keep only the last 10 undo actions
            if self.undo_stack.len() > 10 {
                self.undo_stack.remove(0);
            }

            // Adjust selection if needed
            let flat_rows = flatten_tasks(&self.tasks);
            if self.selected_index >= flat_rows.len() && flat_rows.len() > 0 {
                self.selected_index = flat_rows.len() - 1;
            }

            self.needs_save = true;
        }

        Ok(())
    }

    /// Delete the selected task or subtask
    pub fn delete_selected(&mut self) {
        if let Some((task_idx, subtask_idx)) = self.get_selected_item() {
            // Clone the item before removing it (for undo)
            let item_to_undo = if let Some(st_idx) = subtask_idx {
                self.tasks[task_idx].subtasks[st_idx].clone()
            } else {
                // Only delete task if it has no subtasks
                if !self.tasks[task_idx].subtasks.is_empty() {
                    // Don't delete tasks with subtasks - they must delete or archive subtasks first
                    return;
                }
                self.tasks[task_idx].clone()
            };

            // Remove the item
            if let Some(st_idx) = subtask_idx {
                // Delete subtask
                self.tasks[task_idx].subtasks.remove(st_idx);
            } else {
                // Delete entire task
                self.tasks.remove(task_idx);
            }

            // Save undo information
            self.undo_stack.push(UndoAction::Deleted {
                item: item_to_undo,
                was_subtask: subtask_idx.is_some(),
                parent_task_index: if subtask_idx.is_some() { Some(task_idx) } else { None },
                subtask_index: subtask_idx,
                task_index: task_idx,
            });

            // Keep only the last 10 undo actions
            if self.undo_stack.len() > 10 {
                self.undo_stack.remove(0);
            }

            // Adjust selection if needed
            let flat_rows = flatten_tasks(&self.tasks);
            if flat_rows.is_empty() {
                self.selected_index = 0;
            } else if self.selected_index >= flat_rows.len() {
                self.selected_index = flat_rows.len() - 1;
            }

            self.needs_save = true;
        }
    }

    /// Start adding a new task (opens input form)
    pub fn start_add_task(&mut self) {
        self.input_form = Some(InputFormState {
            title: String::new(),
            notes: String::new(),
            tags: String::new(),
            is_subtask: false,
            editing_field: 0,
        });
        self.ui_mode = UiMode::AddingTask;
    }

    /// Start adding a new subtask (opens input form)
    pub fn start_add_subtask(&mut self) {
        self.input_form = Some(InputFormState {
            title: String::new(),
            notes: String::new(),
            tags: String::new(),
            is_subtask: true,
            editing_field: 0,
        });
        self.ui_mode = UiMode::AddingSubtask;
    }

    /// Toggle between editing fields in input form (title -> notes -> tags)
    pub fn input_form_toggle_field(&mut self) {
        if let Some(form) = &mut self.input_form {
            form.editing_field = (form.editing_field + 1) % 3;
        }
    }

    /// Add character to input form (current field)
    pub fn input_form_add_char(&mut self, c: char) {
        if let Some(form) = &mut self.input_form {
            match form.editing_field {
                0 => form.title.push(c),
                1 => form.notes.push(c),
                2 => form.tags.push(c),
                _ => {}
            }
        }
    }

    /// Backspace in input form (current field)
    pub fn input_form_backspace(&mut self) {
        if let Some(form) = &mut self.input_form {
            match form.editing_field {
                0 => { form.title.pop(); },
                1 => { form.notes.pop(); },
                2 => { form.tags.pop(); },
                _ => {}
            }
        }
    }

    /// Submit input form and create task/subtask
    pub fn submit_input_form(&mut self) {
        if let Some(form) = self.input_form.take() {
            if !form.title.trim().is_empty() {
                let estimate = Duration::hours(1); // Default 1 hour

                // Parse tags from comma-separated string
                let tags: Vec<String> = form.tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if form.is_subtask {
                    if let Some((task_idx, _)) = self.get_selected_item() {
                        let mut subtask = Item::new(form.title, estimate, ScheduleDay::Today);
                        subtask.notes = form.notes;
                        subtask.tags = tags;
                        self.tasks[task_idx].add_subtask(subtask);
                        self.needs_save = true;
                    }
                } else {
                    let mut task = Item::new(form.title, estimate, ScheduleDay::Today);
                    task.notes = form.notes;
                    task.tags = tags;
                    self.tasks.push(task);
                    self.needs_save = true;
                }
            }
            self.ui_mode = UiMode::Normal;
        }
    }

    /// Cancel input form
    pub fn cancel_input_form(&mut self) {
        self.input_form = None;
        self.ui_mode = UiMode::Normal;
    }

    /// Add a task directly (for testing and programmatic use)
    pub fn add_task(&mut self, title: String, estimate: Duration) {
        let task = Item::new(title, estimate, ScheduleDay::Today);
        self.tasks.push(task);
        self.needs_save = true;
    }

    /// Add a subtask directly (for testing and programmatic use)
    pub fn add_subtask(&mut self, title: String, estimate: Duration) {
        if let Some((task_idx, _)) = self.get_selected_item() {
            let subtask = Item::new(title, estimate, ScheduleDay::Today);
            self.tasks[task_idx].add_subtask(subtask);
            self.needs_save = true;
        }
    }

    /// Toggle collapse/expand for selected task
    pub fn toggle_expand(&mut self) {
        if let Some((task_idx, None)) = self.get_selected_item() {
            self.tasks[task_idx].expanded = !self.tasks[task_idx].expanded;
        }
    }

    /// Update timers for all running items
    pub fn tick(&mut self) {
        // Update task timers
        for task in &mut self.tasks {
            task.tick();
        }

        // Track global state timing
        let now = Instant::now();
        let current_state = self.get_global_state();

        // Accumulate time in previous state
        let elapsed_since_change = now.duration_since(self.last_state_change);
        let duration = Duration::from_std(elapsed_since_change).unwrap_or(Duration::zero());

        match self.last_global_state {
            GlobalState::Running => {
                self.running_time = self.running_time + duration;
            }
            GlobalState::Paused => {
                self.paused_time = self.paused_time + duration;
            }
            GlobalState::Idle => {
                self.idle_time = self.idle_time + duration;
            }
        }

        // Handle session tracking on state transitions
        if current_state != self.last_global_state {
            match (self.last_global_state, current_state) {
                // Starting a new running session
                (GlobalState::Paused | GlobalState::Idle, GlobalState::Running) => {
                    self.current_session_start = Some(now);
                }
                // Ending a running session
                (GlobalState::Running, GlobalState::Paused | GlobalState::Idle) => {
                    if let Some(session_start) = self.current_session_start.take() {
                        let session_duration = now.duration_since(session_start);
                        let duration = Duration::from_std(session_duration).unwrap_or(Duration::zero());
                        self.completed_sessions.push(duration);
                    }
                }
                _ => {}
            }

            self.last_global_state = current_state;
            self.last_state_change = now;
        }

        self.last_tick = now;

        // Increment animation frame counter (wraps at u32::MAX)
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }

    /// Check if any running items have hit their estimate
    pub fn check_estimate_hits(&mut self) {
        if self.modal.is_some() {
            return; // Already showing a modal
        }

        for task in &self.tasks {
            if task.is_over_estimate() {
                // Send notification
                notifications::notify_estimate_reached(&task.title);

                self.modal = Some(ModalState {
                    item_id: task.id,
                    message: format!(
                        "Task \"{}\" has reached its estimate ({:.1}h).",
                        task.title,
                        task.track.estimate_hours()
                    ),
                });
                self.ui_mode = UiMode::Modal;
                return;
            }

            for subtask in &task.subtasks {
                if subtask.is_over_estimate() {
                    // Send notification
                    notifications::notify_estimate_reached(&subtask.title);

                    self.modal = Some(ModalState {
                        item_id: subtask.id,
                        message: format!(
                            "Subtask \"{}\" has reached its estimate ({:.1}h).",
                            subtask.title,
                            subtask.track.estimate_hours()
                        ),
                    });
                    self.ui_mode = UiMode::Modal;
                    return;
                }
            }
        }
    }

    /// Find item by UUID (for modal actions)
    fn find_item_by_id_mut(&mut self, id: Uuid) -> Option<&mut Item> {
        for task in &mut self.tasks {
            if task.id == id {
                return Some(task);
            }
            for subtask in &mut task.subtasks {
                if subtask.id == id {
                    return Some(subtask);
                }
            }
        }
        None
    }

    /// Handle modal choice: Done
    pub fn modal_done(&mut self) -> Result<()> {
        if let Some(modal) = self.modal.take() {
            if let Some(item) = self.find_item_by_id_mut(modal.item_id) {
                item.mark_done();
            }
            self.ui_mode = UiMode::Normal;
            self.needs_save = true;

            // Actually remove the done item and log it
            // This is a bit tricky - we need to find and remove the item
            self.remove_done_items()?;
        }
        Ok(())
    }

    /// Helper to remove done items and move them to done_today list
    fn remove_done_items(&mut self) -> Result<()> {
        // Remove done tasks
        let mut i = 0;
        while i < self.tasks.len() {
            if self.tasks[i].status == RunStatus::Done {
                let item = self.tasks.remove(i);
                self.done_today.push(item);
            } else {
                // Remove done subtasks
                let mut j = 0;
                while j < self.tasks[i].subtasks.len() {
                    if self.tasks[i].subtasks[j].status == RunStatus::Done {
                        let subtask = self.tasks[i].subtasks.remove(j);
                        self.done_today.push(subtask);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }

        Ok(())
    }

    /// Handle modal choice: Extend (with custom duration)
    pub fn modal_extend(&mut self, additional: Duration) {
        if let Some(modal) = self.modal.take() {
            if let Some(item) = self.find_item_by_id_mut(modal.item_id) {
                item.increase_estimate(additional);
                // Keep it running
            }
            self.ui_mode = UiMode::Normal;
            self.needs_save = true;
        }
    }

    /// Handle modal choice: Pause
    pub fn modal_pause(&mut self) {
        if let Some(modal) = self.modal.take() {
            if let Some(item) = self.find_item_by_id_mut(modal.item_id) {
                item.pause();
            }
            self.ui_mode = UiMode::Normal;
            self.needs_save = true;
        }
    }

    /// Handle modal choice: Tomorrow
    pub fn modal_tomorrow(&mut self) {
        if let Some(modal) = self.modal.take() {
            let item_id = modal.item_id;

            // Find and remove the item, postpone to tomorrow
            for (task_idx, task) in self.tasks.iter_mut().enumerate() {
                if task.id == item_id {
                    let item = self.tasks.remove(task_idx);
                    // Temporarily store the item ID for postponing
                    if let Some((idx, _)) = self.tasks.iter().enumerate().find(|(_, t)| t.id == item.id) {
                        self.selected_index = idx;
                    }
                    self.tasks.insert(task_idx.min(self.tasks.len()), item);
                    self.ui_mode = UiMode::Normal;
                    // Postpone will be handled by the caller
                    let _ = self.postpone_to_tomorrow();
                    return;
                }

                for (st_idx, subtask) in task.subtasks.iter().enumerate() {
                    if subtask.id == item_id {
                        let item = task.subtasks.remove(st_idx);
                        task.subtasks.insert(st_idx.min(task.subtasks.len()), item);
                        self.ui_mode = UiMode::Normal;
                        let _ = self.postpone_to_tomorrow();
                        return;
                    }
                }
            }
        }
    }

    /// Save state to disk (uses new daily file format)
    pub fn save(&mut self) -> Result<()> {
        use crate::persistence::{serialize_daily_file, today_file, atomic_write};

        // Save today's daily file with ACTIVE, DONE, and ARCHIVED sections
        let daily_content = serialize_daily_file(&self.tasks, &self.done_today, &self.archived_today);
        let today_path = today_file()?;
        atomic_write(today_path, &daily_content)?;

        // Save metadata (global mode, etc.)
        self.save_metadata()?;

        self.needs_save = false;
        Ok(())
    }

    /// Save journal if needed
    pub fn save_journal(&mut self) -> Result<()> {
        if self.journal_needs_save {
            use crate::persistence::{journal_file, atomic_write};
            let journal_path = journal_file()?;
            atomic_write(journal_path, &self.journal_content)?;
            self.journal_needs_save = false;
        }
        Ok(())
    }

    /// Get total elapsed and estimate for display (including done tasks)
    /// Only counts parent tasks, not subtasks
    /// Uses state history for elapsed time to include currently running time
    pub fn get_totals(&self) -> (Duration, Duration) {
        let mut total_elapsed = Duration::zero();
        let mut total_estimate = Duration::zero();

        // Count only parent tasks (not subtasks)
        for task in self.all_todays_tasks() {
            // Use time_in_each_state() to get accurate running time including current session
            let (running, _, _) = task.time_in_each_state();
            total_elapsed = total_elapsed + running;
            total_estimate = total_estimate + task.track.estimate;
        }

        (total_elapsed, total_estimate)
    }

    /// Determine the current global activity state (only considers active tasks, not done)
    pub fn get_global_state(&self) -> GlobalState {
        let has_running = self.tasks.iter().any(|t| {
            t.status == RunStatus::Running || t.subtasks.iter().any(|s| s.status == RunStatus::Running)
        });

        if has_running {
            return GlobalState::Running;
        }

        let has_paused = self.tasks.iter().any(|t| {
            t.status == RunStatus::Paused || t.subtasks.iter().any(|s| s.status == RunStatus::Paused)
        });

        if has_paused {
            return GlobalState::Paused;
        }

        GlobalState::Idle
    }

    /// Get all tasks for today (active + done) for calculations
    fn all_todays_tasks(&self) -> impl Iterator<Item = &Item> {
        self.tasks.iter().chain(self.done_today.iter())
    }

    /// Get total time spent in RUNNING state (from history)
    /// Only counts parent tasks, not subtasks
    pub fn get_running_tasks_time(&self) -> Duration {
        let mut total = Duration::zero();

        for task in self.all_todays_tasks() {
            let (running, _, _) = task.time_in_each_state();
            total = total + running;
        }

        total
    }

    /// Get total time spent in PAUSED state (from history)
    /// Only counts parent tasks, not subtasks
    pub fn get_paused_tasks_time(&self) -> Duration {
        let mut total = Duration::zero();

        for task in self.all_todays_tasks() {
            let (_, paused, _) = task.time_in_each_state();
            total = total + paused;
        }

        total
    }

    /// Get total time spent in IDLE state (from history)
    /// Only counts parent tasks, not subtasks
    pub fn get_idle_tasks_time(&self) -> Duration {
        let mut total = Duration::zero();

        for task in self.all_todays_tasks() {
            let (_, _, idle) = task.time_in_each_state();
            total = total + idle;
        }

        total
    }

    /// Get total time spent over estimates (including done tasks)
    /// Only counts parent tasks, not subtasks
    pub fn get_over_estimate_time(&self) -> Duration {
        let mut total_overrun = Duration::zero();

        for task in self.all_todays_tasks() {
            if task.track.elapsed > task.track.estimate {
                total_overrun = total_overrun + (task.track.elapsed - task.track.estimate);
            }
        }

        total_overrun
    }

    /// Get count of tasks over estimate (including done tasks)
    /// Only counts parent tasks, not subtasks
    pub fn get_over_estimate_count(&self) -> usize {
        let mut count = 0;

        for task in self.all_todays_tasks() {
            if task.track.elapsed > task.track.estimate {
                count += 1;
            }
        }

        count
    }

    /// Get remaining time for all unfinished tasks (only active tasks)
    /// Only counts parent tasks, not subtasks
    pub fn get_remaining_time(&self) -> Duration {
        let mut remaining = Duration::zero();

        for task in &self.tasks {
            if task.status != RunStatus::Done {
                let task_remaining = task.track.estimate - task.track.elapsed;
                if task_remaining > Duration::zero() {
                    remaining = remaining + task_remaining;
                }
            }
        }

        remaining
    }

    /// Calculate efficiency percentage (running time / total app time)
    pub fn get_efficiency(&self) -> f64 {
        let total_time = self.running_time + self.paused_time + self.idle_time;
        if total_time == Duration::zero() {
            return 0.0;
        }
        (self.running_time.num_seconds() as f64 / total_time.num_seconds() as f64) * 100.0
    }

    /// Get average session duration
    pub fn get_avg_session(&self) -> Duration {
        if self.completed_sessions.is_empty() {
            return Duration::zero();
        }

        let total: i64 = self.completed_sessions.iter()
            .map(|d| d.num_seconds())
            .sum();

        Duration::seconds(total / self.completed_sessions.len() as i64)
    }

    /// Get longest session duration
    pub fn get_longest_session(&self) -> Duration {
        self.completed_sessions.iter()
            .max()
            .copied()
            .unwrap_or(Duration::zero())
    }

    /// Calculate projected finish time (simple: current time + remaining time)
    pub fn get_projected_finish(&self) -> Option<chrono::DateTime<chrono::Local>> {
        let remaining = self.get_remaining_time();
        if remaining == Duration::zero() {
            return None;
        }

        // Simple calculation: now + remaining time
        Some(chrono::Local::now() + remaining)
    }

    /// Get context-aware encouragement phrase
    pub fn get_encouragement_phrase(&mut self) -> &'static str {
        const PHRASES_IDLE: &[&str] = &[
            "Plant a new intention 🌱",
            "Choose what matters most 🌱",
            "Begin with clarity 🌱",
        ];

        const PHRASES_RUNNING: &[&str] = &[
            "Keep watering what matters 🌿",
            "Deep focus is growing 🌿",
            "Stay present with your work 🌿",
            "One mindful step at a time 🌿",
        ];

        const PHRASES_OVER_ESTIMATE: &[&str] = &[
            "Reflect before you extend 🌵",
            "Pause and reassess 🌵",
            "Notice what's taking longer 🌵",
        ];

        const PHRASES_END_OF_DAY: &[&str] = &[
            "You've grown enough for today 🌺",
            "Garden complete. 🌼",
            "Well tended 🌸",
        ];

        const PHRASES_MIDDAY_PAUSE: &[&str] = &[
            "Stretch and breathe ☁️",
            "Rest is part of growth 🌤️",
            "A gentle pause 🌥️",
        ];

        // Rotate every 30-60 minutes or on state change
        let now = Instant::now();
        let should_rotate = now.duration_since(self.last_phrase_rotation).as_secs() >= 30 * 60
            || self.get_global_state() != self.last_global_state;

        // Determine context
        let state = self.get_global_state();
        let over_estimate_count = self.get_over_estimate_count();
        let remaining = self.get_remaining_time();
        let hour = chrono::Local::now().hour();

        let phrases = if remaining == Duration::zero() {
            // All tasks complete
            PHRASES_END_OF_DAY
        } else if hour >= 17 && hour < 22 {
            // End of day
            PHRASES_END_OF_DAY
        } else if over_estimate_count > 0 {
            // Tasks over estimate
            PHRASES_OVER_ESTIMATE
        } else if state == GlobalState::Paused && hour >= 11 && hour <= 14 {
            // Midday pause
            PHRASES_MIDDAY_PAUSE
        } else if state == GlobalState::Running {
            PHRASES_RUNNING
        } else {
            // Idle or default
            PHRASES_IDLE
        };

        if should_rotate {
            self.current_phrase_index = (self.current_phrase_index + 1) % phrases.len();
            self.last_phrase_rotation = now;
        }

        phrases[self.current_phrase_index % phrases.len()]
    }

    /// Open the mode selector modal
    pub fn open_mode_selector(&mut self) {
        self.ui_mode = UiMode::ModeSelector;
    }

    /// Set the global mode and pause/resume tasks accordingly
    pub fn set_global_mode(&mut self, mode: GlobalMode) {
        let previous_mode = self.global_mode;
        self.global_mode = mode;

        // Record time spent in previous mode
        let now = Instant::now();
        let elapsed_since_change = now.duration_since(self.last_mode_change);
        let duration = Duration::from_std(elapsed_since_change).unwrap_or(Duration::zero());

        match previous_mode {
            GlobalMode::Working => self.mode_time_working = self.mode_time_working + duration,
            GlobalMode::Break => self.mode_time_break = self.mode_time_break + duration,
            GlobalMode::Lunch => self.mode_time_lunch = self.mode_time_lunch + duration,
            GlobalMode::Gym => self.mode_time_gym = self.mode_time_gym + duration,
            GlobalMode::Dinner => self.mode_time_dinner = self.mode_time_dinner + duration,
            GlobalMode::Personal => self.mode_time_personal = self.mode_time_personal + duration,
            GlobalMode::Sleep => self.mode_time_sleep = self.mode_time_sleep + duration,
        }

        self.last_mode_change = now;

        // Handle task state changes based on mode
        if mode.should_pause_timers() && !previous_mode.should_pause_timers() {
            // Switching from Working to non-working mode: Pause all running tasks
            self.pause_all_for_mode();
        } else if !mode.should_pause_timers() && previous_mode.should_pause_timers() {
            // Switching from non-working to Working mode: Resume previously paused tasks
            self.resume_all_from_mode();
        }

        self.ui_mode = UiMode::Normal;
        self.needs_save = true;
    }

    /// Pause all running tasks due to mode change
    fn pause_all_for_mode(&mut self) {
        // Store which tasks were running
        self.paused_by_mode_task_ids.clear();

        for task in &mut self.tasks {
            if task.status == RunStatus::Running {
                self.paused_by_mode_task_ids.push(task.id);
                task.pause();
            }
            for subtask in &mut task.subtasks {
                if subtask.status == RunStatus::Running {
                    self.paused_by_mode_task_ids.push(subtask.id);
                    subtask.pause();
                }
            }
        }
    }

    /// Resume tasks that were paused due to mode change
    fn resume_all_from_mode(&mut self) {
        // Resume tasks that were paused by mode change
        for task in &mut self.tasks {
            if self.paused_by_mode_task_ids.contains(&task.id) {
                task.start();
            }
            for subtask in &mut task.subtasks {
                if self.paused_by_mode_task_ids.contains(&subtask.id) {
                    subtask.start();
                }
            }
        }

        self.paused_by_mode_task_ids.clear();
    }

    /// Get time spent in each mode
    pub fn get_mode_times(&self) -> [(GlobalMode, Duration); 7] {
        // Include time in current mode
        let now = Instant::now();
        let elapsed_in_current = now.duration_since(self.last_mode_change);
        let current_duration = Duration::from_std(elapsed_in_current).unwrap_or(Duration::zero());

        let mut times = [
            (GlobalMode::Working, self.mode_time_working),
            (GlobalMode::Break, self.mode_time_break),
            (GlobalMode::Lunch, self.mode_time_lunch),
            (GlobalMode::Gym, self.mode_time_gym),
            (GlobalMode::Dinner, self.mode_time_dinner),
            (GlobalMode::Personal, self.mode_time_personal),
            (GlobalMode::Sleep, self.mode_time_sleep),
        ];

        // Add current mode time
        for (mode, time) in times.iter_mut() {
            if *mode == self.global_mode {
                *time = *time + current_duration;
            }
        }

        times
    }

    /// Check if there are any running tasks
    fn has_running_tasks(&self) -> bool {
        for task in &self.tasks {
            if task.status == RunStatus::Running {
                return true;
            }
            for subtask in &task.subtasks {
                if subtask.status == RunStatus::Running {
                    return true;
                }
            }
        }
        false
    }

    /// Check for idle time and show modal if needed
    pub fn check_idle_time(&mut self) {
        let now = Instant::now();

        // If idle check is already active and deadline has passed, auto-pause all
        if let Some(deadline) = self.idle_check_deadline {
            if now >= deadline {
                self.auto_pause_all();
                self.idle_check_deadline = None;
                self.ui_mode = UiMode::Normal;
                self.last_idle_check = now;
                return;
            }
        }

        // Check if 30 minutes have passed since last check
        if self.ui_mode == UiMode::Normal
            && self.has_running_tasks()
            && now.duration_since(self.last_idle_check).as_secs() >= 30 * 60
        {
            // Show idle check modal
            self.ui_mode = UiMode::IdleCheck;
            self.idle_check_deadline = Some(now + std::time::Duration::from_secs(30 * 60));
        }
    }

    /// Confirm user is still working (reset idle check)
    pub fn confirm_working(&mut self) {
        self.last_idle_check = Instant::now();
        self.idle_check_deadline = None;
        self.ui_mode = UiMode::Normal;
    }

    /// Auto-pause all running tasks
    pub fn auto_pause_all(&mut self) {
        for task in &mut self.tasks {
            if task.status == RunStatus::Running {
                task.pause();
            }
            for subtask in &mut task.subtasks {
                if subtask.status == RunStatus::Running {
                    subtask.pause();
                }
            }
        }
        self.needs_save = true;
    }

    /// Move all running and paused tasks to idle (for app exit)
    pub fn auto_idle_all(&mut self) {
        for task in &mut self.tasks {
            if task.status == RunStatus::Running || task.status == RunStatus::Paused {
                task.set_idle();
            }
            for subtask in &mut task.subtasks {
                if subtask.status == RunStatus::Running || subtask.status == RunStatus::Paused {
                    subtask.set_idle();
                }
            }
        }
        self.needs_save = true;
    }

    /// Scroll the done pane up
    pub fn scroll_done_up(&mut self) {
        if self.done_scroll_offset > 0 {
            self.done_scroll_offset -= 1;
        }
    }

    /// Scroll the done pane down
    pub fn scroll_done_down(&mut self) {
        // Calculate total number of lines (tasks + subtasks)
        let total_lines: usize = self.done_today.iter()
            .map(|task| 1 + task.subtasks.len())
            .sum();

        // Allow scrolling as long as there are items
        if total_lines > 0 {
            self.done_scroll_offset += 1;
        }
    }

    /// Reset done scroll offset (when switching views or adding items)
    pub fn reset_done_scroll(&mut self) {
        self.done_scroll_offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> AppState {
        let task1 = Item::new(
            "Task 1".to_string(),
            Duration::hours(1),
            ScheduleDay::Today,
        );
        let task2 = Item::new(
            "Task 2".to_string(),
            Duration::hours(2),
            ScheduleDay::Today,
        );
        AppState::new(vec![task1, task2], Vec::new(), Vec::new(), String::new())
    }

    #[test]
    fn test_app_state_new() {
        let app = create_test_app();
        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.ui_mode, UiMode::Normal);
        assert!(app.modal.is_none());
    }

    #[test]
    fn test_move_selection() {
        let mut app = create_test_app();

        app.move_selection_down();
        assert_eq!(app.selected_index, 1);

        app.move_selection_up();
        assert_eq!(app.selected_index, 0);

        // Can't go below 0
        app.move_selection_up();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_get_selected_item() {
        let app = create_test_app();
        let (task_idx, subtask_idx) = app.get_selected_item().unwrap();
        assert_eq!(task_idx, 0);
        assert!(subtask_idx.is_none());
    }

    #[test]
    fn test_toggle_run_pause() {
        let mut app = create_test_app();

        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Running);

        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Paused);
    }

    #[test]
    fn test_add_task() {
        let mut app = create_test_app();
        app.add_task("New task".to_string(), Duration::hours(1));

        assert_eq!(app.tasks.len(), 3);
        assert_eq!(app.tasks[2].title, "New task");
        assert!(app.needs_save);
    }

    #[test]
    fn test_add_subtask() {
        let mut app = create_test_app();
        app.add_subtask("New subtask".to_string(), Duration::minutes(30));

        assert_eq!(app.tasks[0].subtasks.len(), 1);
        assert_eq!(app.tasks[0].subtasks[0].title, "New subtask");
        assert!(app.needs_save);
    }

    #[test]
    fn test_archive_task() {
        let mut app = create_test_app();
        assert_eq!(app.tasks.len(), 2);

        // Archive first task
        app.archive_selected().ok();
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks[0].title, "Task 2");
        assert!(app.needs_save);
    }

    #[test]
    fn test_archive_subtask() {
        let mut app = create_test_app();
        app.add_subtask("Subtask 1".to_string(), Duration::minutes(30));
        app.add_subtask("Subtask 2".to_string(), Duration::minutes(30));
        assert_eq!(app.tasks[0].subtasks.len(), 2);

        // Move to first subtask
        app.move_selection_down();

        // Archive first subtask
        app.archive_selected().ok();
        assert_eq!(app.tasks[0].subtasks.len(), 1);
        assert_eq!(app.tasks[0].subtasks[0].title, "Subtask 2");
        assert!(app.needs_save);
    }

    #[test]
    fn test_archive_all_tasks() {
        let mut app = create_test_app();

        // Archive both tasks
        app.archive_selected().ok();
        app.archive_selected().ok();

        assert_eq!(app.tasks.len(), 0);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_estimate_adjustment() {
        let mut app = create_test_app();
        let original = app.tasks[0].track.estimate;

        app.increase_estimate();
        assert_eq!(
            app.tasks[0].track.estimate,
            original + Duration::minutes(15)
        );

        app.decrease_estimate();
        assert_eq!(app.tasks[0].track.estimate, original);
    }

    #[test]
    fn test_get_totals() {
        use chrono::Local;

        let mut app = create_test_app();

        // Add state history to simulate 30 minutes of running time for task 0
        let now = Local::now();
        let start1 = now - chrono::Duration::minutes(30);
        app.tasks[0].state_history.push(StateEvent {
            timestamp: start1,
            from_status: Some(RunStatus::Idle),
            to_status: RunStatus::Running,
        });
        app.tasks[0].state_history.push(StateEvent {
            timestamp: now,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });

        // Add state history to simulate 45 minutes of running time for task 1
        let start2 = now - chrono::Duration::minutes(45);
        app.tasks[1].state_history.push(StateEvent {
            timestamp: start2,
            from_status: Some(RunStatus::Idle),
            to_status: RunStatus::Running,
        });
        app.tasks[1].state_history.push(StateEvent {
            timestamp: now,
            from_status: Some(RunStatus::Running),
            to_status: RunStatus::Paused,
        });

        let (elapsed, estimate) = app.get_totals();
        assert_eq!(elapsed, Duration::minutes(75));
        assert_eq!(estimate, Duration::hours(3)); // 1h + 2h
    }

    #[test]
    fn test_undo_mark_done() {
        let mut app = create_test_app();
        let initial_task_count = app.tasks.len();
        let task_title = app.tasks[0].title.clone();

        // Mark first task as done
        app.mark_done().unwrap();

        // Verify task was moved to done_today
        assert_eq!(app.tasks.len(), initial_task_count - 1);
        assert_eq!(app.done_today.len(), 1);
        assert_eq!(app.done_today[0].title, task_title);

        // Verify undo stack has an entry
        assert_eq!(app.undo_stack.len(), 1);

        // Undo the action
        app.undo().unwrap();

        // Verify task was restored to active tasks
        assert_eq!(app.tasks.len(), initial_task_count);
        assert_eq!(app.done_today.len(), 0);
        assert_eq!(app.tasks[0].title, task_title);

        // Verify undo stack is empty
        assert_eq!(app.undo_stack.len(), 0);
    }

    #[test]
    fn test_undo_mark_done_subtask() {
        let mut app = create_test_app();
        app.add_subtask("Test subtask".to_string(), Duration::minutes(30));

        let initial_subtask_count = app.tasks[0].subtasks.len();
        let subtask_title = app.tasks[0].subtasks[0].title.clone();

        // Select the subtask (move down once)
        app.move_selection_down();

        // Mark subtask as done
        app.mark_done().unwrap();

        // Verify subtask was removed and added to done_today
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count - 1);
        assert_eq!(app.done_today.len(), 1);
        assert_eq!(app.done_today[0].title, subtask_title);

        // Undo the action
        app.undo().unwrap();

        // Verify subtask was restored
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count);
        assert_eq!(app.done_today.len(), 0);
        assert_eq!(app.tasks[0].subtasks[0].title, subtask_title);
    }

    #[test]
    fn test_undo_multiple_actions() {
        let mut app = create_test_app();
        app.add_task("Task 3".to_string(), Duration::hours(1));

        let task1_title = app.tasks[0].title.clone();
        let task2_title = app.tasks[1].title.clone();

        // Mark first task as done
        app.mark_done().unwrap();
        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.done_today.len(), 1);

        // Mark second task (now first) as done
        app.mark_done().unwrap();
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.done_today.len(), 2);

        // Undo last action (restore task2)
        app.undo().unwrap();
        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.done_today.len(), 1);
        assert_eq!(app.tasks[0].title, task2_title);

        // Undo second-to-last action (restore task1)
        app.undo().unwrap();
        assert_eq!(app.tasks.len(), 3);
        assert_eq!(app.done_today.len(), 0);
        assert_eq!(app.tasks[0].title, task1_title);
    }

    #[test]
    fn test_cannot_start_tasks_in_non_working_mode() {
        use crate::domain::GlobalMode;

        let mut app = create_test_app();

        // Ensure task starts as Idle
        assert_eq!(app.tasks[0].status, RunStatus::Idle);

        // Try to start task in Working mode - should work
        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Running);

        // Pause the task
        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Paused);

        // Switch to Lunch mode
        app.set_global_mode(GlobalMode::Lunch);

        // Try to start task in Lunch mode - should NOT work
        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Paused); // Should remain paused

        // Switch back to Working mode
        app.set_global_mode(GlobalMode::Working);

        // Now it should work
        app.toggle_run_pause();
        assert_eq!(app.tasks[0].status, RunStatus::Running);
    }

    #[test]
    fn test_undo_delete() {
        let mut app = create_test_app();
        let initial_task_count = app.tasks.len();
        let task_title = app.tasks[0].title.clone();

        // Delete first task
        app.delete_selected();

        // Verify task was deleted
        assert_eq!(app.tasks.len(), initial_task_count - 1);

        // Verify undo stack has an entry
        assert_eq!(app.undo_stack.len(), 1);

        // Undo the action
        app.undo().unwrap();

        // Verify task was restored
        assert_eq!(app.tasks.len(), initial_task_count);
        assert_eq!(app.tasks[0].title, task_title);

        // Verify undo stack is empty
        assert_eq!(app.undo_stack.len(), 0);
    }

    #[test]
    fn test_undo_delete_subtask() {
        let mut app = create_test_app();
        app.add_subtask("Test subtask".to_string(), Duration::minutes(30));

        let initial_subtask_count = app.tasks[0].subtasks.len();
        let subtask_title = app.tasks[0].subtasks[0].title.clone();

        // Select the subtask (move down once)
        app.move_selection_down();

        // Delete subtask
        app.delete_selected();

        // Verify subtask was deleted
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count - 1);

        // Undo the action
        app.undo().unwrap();

        // Verify subtask was restored
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count);
        assert_eq!(app.tasks[0].subtasks[0].title, subtask_title);
    }

    #[test]
    fn test_undo_archive() {
        let mut app = create_test_app();
        let initial_task_count = app.tasks.len();
        let task_title = app.tasks[0].title.clone();

        // Archive first task
        app.archive_selected().unwrap();

        // Verify task was archived
        assert_eq!(app.tasks.len(), initial_task_count - 1);
        assert_eq!(app.archived_today.len(), 1);
        assert_eq!(app.archived_today[0].title, task_title);

        // Verify undo stack has an entry
        assert_eq!(app.undo_stack.len(), 1);

        // Undo the action
        app.undo().unwrap();

        // Verify task was restored to active tasks
        assert_eq!(app.tasks.len(), initial_task_count);
        assert_eq!(app.archived_today.len(), 0);
        assert_eq!(app.tasks[0].title, task_title);

        // Verify undo stack is empty
        assert_eq!(app.undo_stack.len(), 0);
    }

    #[test]
    fn test_undo_archive_subtask() {
        let mut app = create_test_app();
        app.add_subtask("Test subtask".to_string(), Duration::minutes(30));

        let initial_subtask_count = app.tasks[0].subtasks.len();
        let subtask_title = app.tasks[0].subtasks[0].title.clone();

        // Select the subtask (move down once)
        app.move_selection_down();

        // Archive subtask
        app.archive_selected().unwrap();

        // Verify subtask was archived
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count - 1);
        assert_eq!(app.archived_today.len(), 1);
        assert_eq!(app.archived_today[0].title, subtask_title);

        // Undo the action
        app.undo().unwrap();

        // Verify subtask was restored
        assert_eq!(app.tasks[0].subtasks.len(), initial_subtask_count);
        assert_eq!(app.archived_today.len(), 0);
        assert_eq!(app.tasks[0].subtasks[0].title, subtask_title);
    }

    #[test]
    fn test_undo_mixed_actions() {
        let mut app = create_test_app();
        app.add_task("Task 3".to_string(), Duration::hours(1));

        // Archive first task
        app.archive_selected().unwrap();
        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.archived_today.len(), 1);

        // Mark second task (now first) as done
        app.mark_done().unwrap();
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.done_today.len(), 1);

        // Delete remaining task
        app.delete_selected();
        assert_eq!(app.tasks.len(), 0);

        // Undo delete
        app.undo().unwrap();
        assert_eq!(app.tasks.len(), 1);

        // Undo mark done
        app.undo().unwrap();
        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.done_today.len(), 0);

        // Undo archive
        app.undo().unwrap();
        assert_eq!(app.tasks.len(), 3);
        assert_eq!(app.archived_today.len(), 0);
    }
}
