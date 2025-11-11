use crate::app::AppState;
use crate::domain::UiMode;
use crate::ui::styles::{border_style, selected_style};
use chrono::Local;
use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the journal pane
pub fn render_journal_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let is_editing = app.ui_mode == UiMode::EditingJournal;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let title = if is_editing {
        format!(" 📓 Journal ({}) - [Editing] ", today)
    } else {
        format!(" 📓 Journal ({}) ", today)
    };

    let style = if is_editing {
        selected_style()
    } else {
        border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(style);

    let lines: Vec<Line> = app
        .journal_content
        .lines()
        .map(|line| Line::raw(line.to_string()))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    // Show cursor when editing
    if is_editing {
        // Calculate cursor position (row, col)
        let text_before_cursor = &app.journal_content[..app.journal_cursor_pos];
        let line_number = text_before_cursor.matches('\n').count();
        let column = text_before_cursor
            .lines()
            .last()
            .map(|l| l.len())
            .unwrap_or(text_before_cursor.len());

        // Account for border (1 char) and position within the text area
        let cursor_x = area.x + 1 + column as u16;
        let cursor_y = area.y + 1 + line_number as u16;

        // Only show cursor if it's within bounds
        if cursor_x < area.x + area.width - 1 && cursor_y < area.y + area.height - 1 {
            f.set_cursor(cursor_x, cursor_y);
        }
    }
}
