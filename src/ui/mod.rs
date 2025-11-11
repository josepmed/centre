pub mod animation_pane;
pub mod details_pane;
pub mod done_pane;
pub mod garden_pane;
pub mod input_form;
pub mod journal_pane;
pub mod keybindings;
pub mod layout;
pub mod list_pane;
pub mod modal;
pub mod styles;

use crate::app::AppState;
use crate::domain::UiMode;
use animation_pane::render_animation_pane;
use details_pane::render_details_pane;
use done_pane::render_done_pane;
use garden_pane::render_garden_pane;
use input_form::render_input_form;
use journal_pane::render_journal_pane;
use keybindings::render_keybindings;
use layout::create_layout;
use list_pane::render_list_pane;
use modal::{render_day_changed_modal, render_idle_check_modal, render_modal, render_mode_selector};
use ratatui::Frame;

/// Main render function - draws the entire UI
pub fn render(f: &mut Frame, app: &mut AppState) {
    let size = f.size();
    let layout = create_layout(size, app.show_done);

    // Render keybindings bar
    render_keybindings(f, layout.keybindings_area);

    // Render panes
    render_list_pane(f, app, layout.list_area);
    render_details_pane(f, app, layout.details_area);
    render_garden_pane(f, app, layout.garden_area);
    render_journal_pane(f, app, layout.journal_area);

    // Render done pane if showing
    if let Some(done_area) = layout.done_area {
        render_done_pane(f, app, done_area);
    }

    // Render animation pane if showing
    if let Some(animation_area) = layout.animation_area {
        render_animation_pane(f, app, animation_area);
    }

    // Render day changed modal (takes precedence)
    if app.ui_mode == UiMode::DayChanged {
        render_day_changed_modal(f, app, size);
        return; // Don't render other modals
    }

    // Render modal if active
    if app.modal.is_some() {
        render_modal(f, app, size);
    }

    // Render idle check modal if active
    if app.ui_mode == UiMode::IdleCheck {
        render_idle_check_modal(f, app, size);
    }

    // Render input form if active
    if app.input_form.is_some() {
        render_input_form(f, app, size);
    }

    // Render mode selector if active
    if app.ui_mode == UiMode::ModeSelector {
        render_mode_selector(f, app, size);
    }
}
