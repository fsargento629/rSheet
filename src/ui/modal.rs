use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::domain::Spreadsheet;

/// Calculates a centered rectangle of specified percentage width and height.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Renders the insert modal overlay when the app is in Insert mode.
pub fn render_insert_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let modal_area = centered_rect(60, 20, area);

    // 1. Clear underlying spreadsheet characters
    f.render_widget(Clear, modal_area);

    // 2. Format title with column letter and 1-based row number
    let col_letter = Spreadsheet::col_to_letter(app.cursor_col);
    let title = format!(" Edit Cell [{}{}] ", col_letter, app.cursor_row + 1);

    // 3. Build paragraph widget with borders and title/footer
    let edit_widget = Paragraph::new(app.edit_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .title_bottom(" Press [Enter] to commit | [Esc] to cancel ")
                .title_alignment(ratatui::layout::Alignment::Left),
        );

    // 4. Render modal widget over cleared area
    f.render_widget(edit_widget, modal_area);

    // 5. Set terminal cursor position inside the popup frame
    let inner_width = modal_area.width.saturating_sub(2);
    let inner_height = modal_area.height.saturating_sub(2);

    if inner_width > 0 && inner_height > 0 {
        let cursor_offset = app.edit_cursor_idx as u16;
        let row_offset = cursor_offset / inner_width;
        let col_offset = cursor_offset % inner_width;

        let cursor_x = (modal_area.x + 1 + col_offset).min(modal_area.x + modal_area.width - 2);
        let cursor_y = (modal_area.y + 1 + row_offset).min(modal_area.y + modal_area.height - 2);

        f.set_cursor_position((cursor_x, cursor_y));
    }
}
