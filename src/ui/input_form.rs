use crate::app::AppState;
use crate::ui::{
    layout::create_modal_area,
    styles::{modal_bg_style, modal_title_style},
};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the input form for adding tasks/subtasks
pub fn render_input_form(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(form) = &app.input_form {
        let modal_area = create_modal_area(area);

        // Clear the area behind the form
        f.render_widget(Clear, modal_area);

        let mut lines = Vec::new();

        // Title
        let title_text = if form.is_subtask {
            " Add Subtask "
        } else {
            " Add Task "
        };

        // Title field
        lines.push(Line::raw(""));
        let title_label = if form.editing_field == 0 {
            "Title: (editing)"
        } else {
            "Title:"
        };
        lines.push(Line::raw(title_label));

        let title_line = Line::from(vec![
            Span::raw("> "),
            Span::styled(&form.title, modal_title_style()),
            if form.editing_field == 0 {
                Span::styled("█", modal_title_style()) // Cursor
            } else {
                Span::raw("")
            },
        ]);
        lines.push(title_line);
        lines.push(Line::raw(""));

        // Notes field
        let notes_label = if form.editing_field == 1 {
            "Notes: (editing)"
        } else {
            "Notes:"
        };
        lines.push(Line::raw(notes_label));

        let notes_line = Line::from(vec![
            Span::raw("> "),
            Span::styled(&form.notes, modal_title_style()),
            if form.editing_field == 1 {
                Span::styled("█", modal_title_style()) // Cursor
            } else {
                Span::raw("")
            },
        ]);
        lines.push(notes_line);
        lines.push(Line::raw(""));

        // Tags field
        let tags_label = if form.editing_field == 2 {
            "Tags (comma-separated): (editing)"
        } else {
            "Tags (comma-separated):"
        };
        lines.push(Line::raw(tags_label));

        let tags_line = Line::from(vec![
            Span::raw("> "),
            Span::styled(&form.tags, modal_title_style()),
            if form.editing_field == 2 {
                Span::styled("█", modal_title_style()) // Cursor
            } else {
                Span::raw("")
            },
        ]);
        lines.push(tags_line);
        lines.push(Line::raw(""));

        // Instructions
        lines.push(Line::raw("Tab to switch fields  ·  Enter to submit  ·  Esc to cancel"));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("(Default estimate: "),
            Span::styled("1h", modal_title_style()),
            Span::raw(")"),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(title_text, modal_title_style()))
                    .style(modal_bg_style()),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, modal_area);
    }
}
