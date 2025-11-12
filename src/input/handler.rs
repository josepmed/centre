use crate::app::AppState;
use crate::domain::{GlobalMode, UiMode};
use anyhow::Result;
use chrono::Duration;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::env;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

/// Handle keyboard input events
pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match app.ui_mode {
        UiMode::Normal => handle_normal_mode(app, key),
        UiMode::Modal => handle_modal_mode(app, key),
        UiMode::AddingTask | UiMode::AddingSubtask | UiMode::EditingTask => handle_input_form_mode(app, key),
        UiMode::IdleCheck => handle_idle_check_mode(app, key),
        UiMode::EditingJournal => handle_journal_editing_mode(app, key),
        UiMode::ModeSelector => handle_mode_selector_mode(app, key),
        _ => Ok(false),
    }
}

/// Handle keys in normal mode
fn handle_normal_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Navigation (with Shift modifier for reordering)
        KeyCode::Up => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.move_item_up();
            } else {
                app.move_selection_up();
            }
            Ok(false)
        }
        KeyCode::Down => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.move_item_down();
            } else {
                app.move_selection_down();
            }
            Ok(false)
        }

        // Scroll done pane (when visible) using [ and ]
        KeyCode::Char('[') => {
            if app.show_done {
                app.scroll_done_up();
            }
            Ok(false)
        }
        KeyCode::Char(']') => {
            if app.show_done {
                app.scroll_done_down();
            }
            Ok(false)
        }
        KeyCode::Char('{') => {
            if app.show_done {
                // Scroll up by 5 lines
                for _ in 0..5 {
                    app.scroll_done_up();
                }
            }
            Ok(false)
        }
        KeyCode::Char('}') => {
            if app.show_done {
                // Scroll down by 5 lines
                for _ in 0..5 {
                    app.scroll_done_down();
                }
            }
            Ok(false)
        }

        // Scroll planner pane using < and >
        KeyCode::Char('<') => {
            app.scroll_planner_up();
            Ok(false)
        }
        KeyCode::Char('>') => {
            app.scroll_planner_down();
            Ok(false)
        }
        // Scroll planner pane fast using , (comma) and . (period)
        KeyCode::Char(',') => {
            // Scroll up by 5 lines
            for _ in 0..5 {
                app.scroll_planner_up();
            }
            Ok(false)
        }
        KeyCode::Char('.') => {
            // Scroll down by 5 lines
            for _ in 0..5 {
                app.scroll_planner_down();
            }
            Ok(false)
        }

        // Toggle run/pause
        KeyCode::Enter => {
            app.toggle_run_pause();
            Ok(false)
        }

        // Adjust estimate
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.increase_estimate();
            Ok(false)
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.decrease_estimate();
            Ok(false)
        }

        // Mark done
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.mark_done()?;
            Ok(false)
        }

        // Undo last action
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.undo()?;
            Ok(false)
        }

        // Postpone to tomorrow
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.postpone_to_tomorrow()?;
            Ok(false)
        }

        // Archive task/subtask
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.archive_selected()?;
            Ok(false)
        }

        // Archive task/subtask
        KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Delete => {
            app.archive_selected()?;
            Ok(false)
        }

        // Edit task/subtask (open form with existing data)
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.start_edit_task();
            Ok(false)
        }

        // Add task
        KeyCode::Char('a') => {
            app.start_add_task();
            Ok(false)
        }

        // Add subtask
        KeyCode::Char('A') => {
            app.start_add_subtask();
            Ok(false)
        }

        // Toggle expand/collapse
        KeyCode::Char(' ') => {
            app.toggle_expand();
            Ok(false)
        }

        // Toggle done tasks view
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.toggle_show_done();
            Ok(false)
        }

        // Toggle daily planner view
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.toggle_show_planner();
            Ok(false)
        }

        // Toggle journal editing
        KeyCode::Char('j') | KeyCode::Char('J') => {
            if app.ui_mode == UiMode::Normal {
                app.ui_mode = UiMode::EditingJournal;
                // Set cursor to end of content when entering edit mode
                app.journal_cursor_pos = app.journal_content.len();
            } else if app.ui_mode == UiMode::EditingJournal {
                app.ui_mode = UiMode::Normal;
            }
            Ok(false)
        }

        // Open mode selector
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.open_mode_selector();
            Ok(false)
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => Ok(true),

        // Escape (for future use)
        KeyCode::Esc => Ok(false),

        _ => Ok(false),
    }
}

/// Handle keys in modal mode
fn handle_modal_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Done
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.modal_done()?;
            Ok(false)
        }

        // Extend - show quick options
        KeyCode::Char('e') | KeyCode::Char('E') => {
            // Quick extend by 30 minutes
            app.modal_extend(Duration::minutes(30));
            Ok(false)
        }

        // Pause
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('p') | KeyCode::Char('P') => {
            app.modal_pause();
            Ok(false)
        }

        // Tomorrow
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.modal_tomorrow();
            Ok(false)
        }

        // Escape closes modal
        KeyCode::Esc => {
            app.modal = None;
            app.ui_mode = UiMode::Normal;
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle keys in idle check mode
fn handle_idle_check_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Yes, still working
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_working();
            Ok(false)
        }

        // No, pause everything
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.auto_pause_all();
            app.ui_mode = UiMode::Normal;
            app.last_idle_check = std::time::Instant::now();
            app.idle_check_deadline = None;
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle keys in input form mode (adding task/subtask)
fn handle_input_form_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Submit form
        KeyCode::Enter => {
            app.submit_input_form();
            Ok(false)
        }

        // Cancel form
        KeyCode::Esc => {
            app.cancel_input_form();
            Ok(false)
        }

        // Switch between title and notes
        KeyCode::Tab => {
            app.input_form_toggle_field();
            Ok(false)
        }

        // Backspace
        KeyCode::Backspace => {
            app.input_form_backspace();
            Ok(false)
        }

        // Add character
        KeyCode::Char(c) => {
            app.input_form_add_char(c);
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle keys in journal editing mode
fn handle_journal_editing_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Exit journal editing mode (only Esc)
        KeyCode::Esc => {
            app.ui_mode = UiMode::Normal;
            Ok(false)
        }

        // Move cursor left
        KeyCode::Left => {
            if app.journal_cursor_pos > 0 {
                app.journal_cursor_pos -= 1;
            }
            Ok(false)
        }

        // Move cursor right
        KeyCode::Right => {
            if app.journal_cursor_pos < app.journal_content.len() {
                app.journal_cursor_pos += 1;
            }
            Ok(false)
        }

        // Move cursor to start
        KeyCode::Home => {
            app.journal_cursor_pos = 0;
            Ok(false)
        }

        // Move cursor to end
        KeyCode::End => {
            app.journal_cursor_pos = app.journal_content.len();
            Ok(false)
        }

        // Add newline (both Enter and Shift+Enter)
        KeyCode::Enter => {
            app.journal_content.insert(app.journal_cursor_pos, '\n');
            app.journal_cursor_pos += 1;
            app.journal_needs_save = true;
            Ok(false)
        }

        // Backspace
        KeyCode::Backspace => {
            if app.journal_cursor_pos > 0 {
                app.journal_content.remove(app.journal_cursor_pos - 1);
                app.journal_cursor_pos -= 1;
                app.journal_needs_save = true;
            }
            Ok(false)
        }

        // Delete
        KeyCode::Delete => {
            if app.journal_cursor_pos < app.journal_content.len() {
                app.journal_content.remove(app.journal_cursor_pos);
                app.journal_needs_save = true;
            }
            Ok(false)
        }

        // Option+b (backward word) - macOS sends this for Option+Left
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Jump to previous word
            let content = &app.journal_content[..app.journal_cursor_pos];
            if let Some(pos) = content.trim_end().rfind(|c: char| c.is_whitespace()) {
                app.journal_cursor_pos = pos;
                // Skip trailing whitespace
                while app.journal_cursor_pos > 0 {
                    if let Some(ch) = app.journal_content.chars().nth(app.journal_cursor_pos - 1) {
                        if ch.is_whitespace() {
                            app.journal_cursor_pos -= 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else {
                app.journal_cursor_pos = 0;
            }
            Ok(false)
        }

        // Option+f (forward word) - macOS sends this for Option+Right
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Jump to next word
            let content = &app.journal_content[app.journal_cursor_pos..];
            if let Some(pos) = content.find(|c: char| c.is_whitespace()) {
                app.journal_cursor_pos += pos;
                // Skip whitespace to start of next word
                while app.journal_cursor_pos < app.journal_content.len() {
                    if let Some(ch) = app.journal_content.chars().nth(app.journal_cursor_pos) {
                        if ch.is_whitespace() {
                            app.journal_cursor_pos += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else {
                app.journal_cursor_pos = app.journal_content.len();
            }
            Ok(false)
        }

        // Add character (without Ctrl modifier to allow Ctrl+C/Ctrl+D to work)
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.journal_content.insert(app.journal_cursor_pos, c);
            app.journal_cursor_pos += 1;
            app.journal_needs_save = true;
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle keys in mode selector mode
fn handle_mode_selector_mode(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Number keys to select mode
        KeyCode::Char('1') => {
            app.set_global_mode(GlobalMode::Working);
            Ok(false)
        }
        KeyCode::Char('2') => {
            app.set_global_mode(GlobalMode::Break);
            Ok(false)
        }
        KeyCode::Char('3') => {
            app.set_global_mode(GlobalMode::Lunch);
            Ok(false)
        }
        KeyCode::Char('4') => {
            app.set_global_mode(GlobalMode::Gym);
            Ok(false)
        }
        KeyCode::Char('5') => {
            app.set_global_mode(GlobalMode::Dinner);
            Ok(false)
        }
        KeyCode::Char('6') => {
            app.set_global_mode(GlobalMode::Personal);
            Ok(false)
        }
        KeyCode::Char('7') => {
            app.set_global_mode(GlobalMode::Sleep);
            Ok(false)
        }

        // Cancel with Escape
        KeyCode::Esc => {
            app.ui_mode = UiMode::Normal;
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Edit notes using external $EDITOR
fn edit_notes_external(app: &mut AppState) -> Result<()> {
    if let Some(item) = app.get_selected_item_mut() {
        // Get editor from environment, default to vi
        let editor = env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

        // Create temp file with current notes
        let mut temp_file = NamedTempFile::new()?;
        std::io::Write::write_all(&mut temp_file, item.notes.as_bytes())?;

        // Persist the temp file so it doesn't get deleted
        let temp_path = temp_file.into_temp_path();

        // Spawn editor
        // Note: In the main event loop, we'll need to disable raw mode before this
        // and re-enable after
        let status = Command::new(&editor)
            .arg(&temp_path)
            .status()?;

        if status.success() {
            // Read edited notes
            let edited_notes = fs::read_to_string(&temp_path)?;
            item.notes = edited_notes;
            app.needs_save = true;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Item, ScheduleDay};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_app() -> AppState {
        let task = Item::new(
            "Test task".to_string(),
            Duration::hours(1),
            ScheduleDay::Today,
        );
        AppState::new(vec![task], Vec::new(), Vec::new(), String::new())
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_handle_navigation() {
        let mut app = create_test_app();
        app.add_task("Task 2".to_string(), Duration::hours(1));

        assert_eq!(app.selected_index, 0);

        handle_key(&mut app, key(KeyCode::Down)).unwrap();
        assert_eq!(app.selected_index, 1);

        handle_key(&mut app, key(KeyCode::Up)).unwrap();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_handle_quit() {
        let mut app = create_test_app();
        let should_quit = handle_key(&mut app, key(KeyCode::Char('q'))).unwrap();
        assert!(should_quit);
    }

    #[test]
    fn test_handle_add_task() {
        let mut app = create_test_app();
        let initial_count = app.tasks.len();

        // Press 'a' to open form
        handle_key(&mut app, key(KeyCode::Char('a'))).unwrap();
        assert_eq!(app.ui_mode, UiMode::AddingTask);
        assert!(app.input_form.is_some());

        // Type title
        handle_key(&mut app, key(KeyCode::Char('N'))).unwrap();
        handle_key(&mut app, key(KeyCode::Char('e'))).unwrap();
        handle_key(&mut app, key(KeyCode::Char('w'))).unwrap();

        // Submit with Enter
        handle_key(&mut app, key(KeyCode::Enter)).unwrap();
        assert_eq!(app.tasks.len(), initial_count + 1);
        assert_eq!(app.ui_mode, UiMode::Normal);
        assert!(app.input_form.is_none());
    }

    #[test]
    fn test_handle_archive_task() {
        let mut app = create_test_app();
        app.add_task("Task to archive".to_string(), Duration::hours(1));
        let initial_count = app.tasks.len();

        handle_key(&mut app, key(KeyCode::Char('x'))).unwrap();
        assert_eq!(app.tasks.len(), initial_count - 1);
    }

    #[test]
    fn test_handle_archive_with_delete_key() {
        let mut app = create_test_app();
        let initial_count = app.tasks.len();

        handle_key(&mut app, key(KeyCode::Delete)).unwrap();
        assert_eq!(app.tasks.len(), initial_count - 1);
    }
}
