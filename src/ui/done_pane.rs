use crate::app::AppState;
use crate::ui::styles::{border_style, default_style, done_style, title_style};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the done tasks pane
pub fn render_done_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .done_today
        .iter()
        .map(|item| {
            let elapsed_str = item.track.elapsed_formatted();
            let estimate_str = item.track.estimate_formatted();

            let line = Line::from(vec![
                Span::styled("✓ ", done_style()),
                Span::styled(&item.title, default_style()),
                Span::raw("  "),
                Span::styled(
                    format!("({} / {})", elapsed_str, estimate_str),
                    done_style(),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let count = app.done_today.len();
    let title = if count == 0 {
        " Done Today (0) ".to_string()
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
