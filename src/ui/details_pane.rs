use crate::app::AppState;
use crate::domain::{flatten_tasks, RunStatus};
use crate::ui::styles::{border_style, default_style, running_style, title_style};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the details pane for the selected item
pub fn render_details_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let flat_rows = flatten_tasks(&app.tasks);

    if app.selected_index >= flat_rows.len() || app.tasks.is_empty() {
        let empty = Paragraph::new("No task selected").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style())
                .title(Span::styled(" Details ", title_style())),
        );
        f.render_widget(empty, area);
        return;
    }

    let row = &flat_rows[app.selected_index];
    let item = if let Some(st_idx) = row.subtask_index {
        &app.tasks[row.task_index].subtasks[st_idx]
    } else {
        &app.tasks[row.task_index]
    };

    let mut lines = Vec::new();

    // Title
    lines.push(Line::from(vec![
        Span::styled("Title: ", title_style()),
        Span::raw(&item.title),
    ]));
    lines.push(Line::raw(""));

    // Estimate
    lines.push(Line::from(vec![
        Span::styled("Est:     ", title_style()),
        Span::raw(item.track.estimate_formatted()),
    ]));

    // Elapsed
    let elapsed_style = if item.status == RunStatus::Running {
        running_style()
    } else {
        default_style()
    };
    lines.push(Line::from(vec![
        Span::styled("Elapsed: ", title_style()),
        Span::styled(
            item.track.elapsed_formatted(),
            elapsed_style,
        ),
    ]));

    // Progress
    let progress = (item.track.progress_ratio() * 100.0).min(999.9);
    lines.push(Line::from(vec![
        Span::styled("Progress: ", title_style()),
        Span::raw(format!("{:.0}%", progress)),
    ]));

    // Status
    lines.push(Line::from(vec![
        Span::styled("Status: ", title_style()),
        Span::raw(item.status.to_tag()),
    ]));
    lines.push(Line::raw(""));

    // Notes
    if !item.notes.trim().is_empty() {
        lines.push(Line::from(Span::styled("Notes:", title_style())));
        for note_line in item.notes.lines() {
            lines.push(Line::raw(format!("  {}", note_line)));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Notes: (empty)",
            default_style(),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style())
                .title(Span::styled(" Details ", title_style())),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
