use crate::app::AppState;
use crate::domain::{tree_connector, Item};
use crate::ui::styles::{border_style, default_style, done_style, title_style, tree_style};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Create a line for a done task or subtask
fn create_done_line(item: &Item, depth: usize, is_last: bool) -> Line {
    let mut spans = Vec::new();

    // Indentation and tree connector for subtasks
    if depth > 0 {
        spans.push(Span::styled("     ".to_string(), tree_style()));
        spans.push(Span::styled(
            tree_connector(is_last).to_string(),
            tree_style(),
        ));
        spans.push(Span::raw(" ".to_string()));
    }

    let elapsed_str = item.track.elapsed_formatted();
    let estimate_str = item.track.estimate_formatted();

    spans.push(Span::styled("✓ ".to_string(), done_style()));
    spans.push(Span::styled(item.title.clone(), default_style()));
    spans.push(Span::raw("  ".to_string()));
    spans.push(Span::styled(
        format!("({} / {})", elapsed_str, estimate_str),
        done_style(),
    ));

    Line::from(spans)
}

/// Render the done tasks pane
pub fn render_done_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let mut all_items: Vec<ListItem> = Vec::new();

    // Iterate through done tasks and their subtasks
    for task in &app.done_today {
        // Add the parent task
        let line = create_done_line(task, 0, false);
        all_items.push(ListItem::new(line));

        // Add subtasks if any
        let subtask_count = task.subtasks.len();
        for (idx, subtask) in task.subtasks.iter().enumerate() {
            let is_last = idx == subtask_count - 1;
            let line = create_done_line(subtask, 1, is_last);
            all_items.push(ListItem::new(line));
        }
    }

    // Calculate total before applying scroll
    let total_items: usize = all_items.len();

    // Apply scroll offset
    let items: Vec<ListItem> = all_items
        .into_iter()
        .skip(app.done_scroll_offset)
        .collect();

    let count = app.done_today.len();

    let title = if count == 0 {
        " Done Today (0) ".to_string()
    } else if app.done_scroll_offset > 0 {
        format!(" Done Today ({}) [scrolled +{}] ", count, app.done_scroll_offset)
    } else {
        format!(" Done Today ({}) ", count)
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style())
            .title(Span::styled(title, title_style())),
    );

    f.render_widget(list, area);
}
