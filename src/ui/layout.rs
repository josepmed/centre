use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout structure
pub struct MainLayout {
    pub list_area: Rect,
    pub planner_area: Option<Rect>,
    pub details_area: Rect,
    pub garden_area: Rect,
    pub journal_area: Rect,
    pub done_area: Option<Rect>,
    pub animation_area: Option<Rect>,
    pub keybindings_area: Rect,
}

/// Create the main layout
/// - Top bar: keybindings (1 row)
/// - Main area: Split horizontally
///   - When planner shown: List (70%) | Planner (30%)
///   - When planner hidden: List (70%) | Details (30%)
/// - Bottom area: Done pane (if showing) above Garden pane
pub fn create_layout(area: Rect, show_done: bool, show_planner: bool) -> MainLayout {
    // Split into top bar and main content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Keybindings bar
            Constraint::Min(0),    // Main content
        ])
        .split(area);

    let keybindings_area = main_chunks[0];
    let content_area = main_chunks[1];

    if show_done {
        // Split content vertically: top section and bottom section
        let vertical_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Top section (list + details)
                Constraint::Percentage(25), // Done pane
                Constraint::Percentage(25), // Bottom section (garden + journal)
            ])
            .split(content_area);

        // Split top section horizontally: list on left, planner on right (when shown), or list + details (when planner hidden)
        let (list_area, details_area, planner_area) = if show_planner {
            // When planner is shown: List (70%) | Planner (30%), no details
            let top_horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // List pane
                    Constraint::Percentage(30), // Planner pane
                ])
                .split(vertical_split[0]);
            // Create a zero-sized rect for details (hidden)
            let hidden_details = ratatui::layout::Rect::new(0, 0, 0, 0);
            (top_horizontal[0], hidden_details, Some(top_horizontal[1]))
        } else {
            // When planner is hidden: List (70%) | Details (30%)
            let top_horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // List pane
                    Constraint::Percentage(30), // Details pane
                ])
                .split(vertical_split[0]);
            (top_horizontal[0], top_horizontal[1], None)
        };

        // Split done pane row horizontally: done on left (80%), animation on right (20%)
        let done_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80), // Done pane
                Constraint::Percentage(20), // Animation pane
            ])
            .split(vertical_split[1]);

        // Split bottom section horizontally: garden on left, journal on right
        let bottom_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Garden pane
                Constraint::Percentage(50), // Journal pane
            ])
            .split(vertical_split[2]);

        MainLayout {
            list_area,
            planner_area,
            details_area,
            done_area: Some(done_horizontal[0]),
            animation_area: Some(done_horizontal[1]),
            garden_area: bottom_horizontal[0],
            journal_area: bottom_horizontal[1],
            keybindings_area,
        }
    } else {
        // Split content vertically: top section and bottom section
        let vertical_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(75), // Top section (list + details)
                Constraint::Percentage(25), // Bottom section (garden + journal)
            ])
            .split(content_area);

        // Split top section horizontally: list on left, planner on right (when shown), or list + details (when planner hidden)
        let (list_area, details_area, planner_area) = if show_planner {
            // When planner is shown: List (70%) | Planner (30%), no details
            let horizontal_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // List pane
                    Constraint::Percentage(30), // Planner pane
                ])
                .split(vertical_split[0]);
            // Create a zero-sized rect for details (hidden)
            let hidden_details = ratatui::layout::Rect::new(0, 0, 0, 0);
            (horizontal_split[0], hidden_details, Some(horizontal_split[1]))
        } else {
            // When planner is hidden: List (70%) | Details (30%)
            let horizontal_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // List pane
                    Constraint::Percentage(30), // Details pane
                ])
                .split(vertical_split[0]);
            (horizontal_split[0], horizontal_split[1], None)
        };

        // Split bottom section horizontally: garden on left, journal on right
        let bottom_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Garden pane
                Constraint::Percentage(50), // Journal pane
            ])
            .split(vertical_split[1]);

        MainLayout {
            list_area,
            planner_area,
            details_area,
            done_area: None,
            animation_area: None,
            garden_area: bottom_horizontal[0],
            journal_area: bottom_horizontal[1],
            keybindings_area,
        }
    }
}

/// Create centered modal area (for estimate-hit modal)
pub fn create_modal_area(area: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(16),
            Constraint::Percentage(25),
        ])
        .split(area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical_chunks[1]);

    horizontal_chunks[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_layout() {
        let area = Rect::new(0, 0, 100, 50);
        let layout = create_layout(area, false, true);

        assert!(layout.list_area.height > 0);
        assert!(layout.details_area.height > 0);
        assert!(layout.garden_area.height > 0);
        assert!(layout.journal_area.height > 0);
        assert!(layout.done_area.is_none());
        assert!(layout.planner_area.is_some());
        assert_eq!(layout.keybindings_area.height, 1);

        let layout_with_done = create_layout(area, true, true);
        assert!(layout_with_done.done_area.is_some());
        assert!(layout_with_done.planner_area.is_some());
        assert!(layout_with_done.journal_area.height > 0);

        let layout_no_planner = create_layout(area, false, false);
        assert!(layout_no_planner.planner_area.is_none());
    }

    #[test]
    fn test_create_modal_area() {
        let area = Rect::new(0, 0, 100, 50);
        let modal = create_modal_area(area);

        assert!(modal.width < area.width);
        assert!(modal.height < area.height);
        assert_eq!(modal.height, 16);
    }
}
