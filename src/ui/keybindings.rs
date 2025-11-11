use crate::ui::styles::hint_style;
use ratatui::{layout::Rect, text::{Line, Span}, widgets::Paragraph, Frame};

/// Render the keybindings hint bar
pub fn render_keybindings(f: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::raw(" ↑/↓ select   "),
        Span::raw("Shift+↑/↓ reorder   "),
        Span::raw("Enter start/stop   "),
        Span::raw("+ / - est   "),
        Span::raw("d done   "),
        Span::raw("u undo   "),
        Span::raw("p tomorrow   "),
        Span::raw("x/r archive   "),
        Span::raw("a add   "),
        Span::raw("A subtask   "),
        Span::raw("j journal   "),
        Span::raw("m mode   "),
        Span::raw("c done-view   "),
        Span::raw("q quit"),
    ]);

    let paragraph = Paragraph::new(hints).style(hint_style());
    f.render_widget(paragraph, area);
}
