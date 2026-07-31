use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::app::App;
use crate::domain::Spreadsheet;

pub struct CsvGrid<'a> {
    app: &'a App,
    col_width: u16,
    header_width: u16,
}

impl<'a> CsvGrid<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            app,
            col_width: 12,
            header_width: 6,
        }
    }

    /// Helper to compute visible capacity given screen dimensions
    pub fn visible_dimensions(&self, area: Rect) -> (usize, usize) {
        if area.width < self.header_width || area.height < 2 {
            return (0, 0);
        }

        let visible_cols = ((area.width - self.header_width) / self.col_width) as usize;
        let visible_rows = (area.height - 1) as usize; // reserve 1 row for top header

        (visible_rows, visible_cols)
    }
}

impl<'a> Widget for CsvGrid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < self.header_width || area.height < 2 {
            return;
        }

        let default_header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let active_header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let cell_border_style = Style::default().fg(Color::DarkGray);
        let cell_text_style = Style::default().fg(Color::Reset);
        let active_cell_style = Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let (visible_rows, visible_cols) = self.visible_dimensions(area);

        // --- 1. Draw Top-Left Corner Box ---
        let corner_str = format!("{:>width$} ", "", width = (self.header_width as usize) - 1);
        buf.set_string(area.x, area.y, &corner_str, default_header_style);

        // --- 2. Draw Column Headers (A, B, C...) ---
        let mut x_offset = area.x + self.header_width;

        for visible_col_idx in 0..visible_cols {
            let col_idx = self.app.scroll_col + visible_col_idx;
            if col_idx >= self.app.sheet.max_cols {
                break;
            }

            let col_label = Spreadsheet::col_to_letter(col_idx);
            let formatted_header = format!(
                "{:^width$}|",
                col_label,
                width = (self.col_width as usize) - 1
            );

            let header_style = if col_idx == self.app.cursor_col {
                active_header_style
            } else {
                default_header_style
            };

            buf.set_string(x_offset, area.y, &formatted_header, header_style);
            x_offset += self.col_width;
        }

        // --- 3. Draw Grid Rows & Cells ---
        let mut y_offset = area.y + 1;

        for visible_row_idx in 0..visible_rows {
            let row_idx = self.app.scroll_row + visible_row_idx;
            if row_idx >= self.app.sheet.max_rows {
                break;
            }

            // A. Draw Row Header (1, 2, 3...)
            let row_label = format!(
                "{:>width$} ",
                row_idx + 1,
                width = (self.header_width as usize) - 1
            );
            let row_header_style = if row_idx == self.app.cursor_row {
                active_header_style
            } else {
                default_header_style
            };
            buf.set_string(area.x, y_offset, &row_label, row_header_style);

            // B. Draw Row Cells
            let mut cell_x = area.x + self.header_width;
            for visible_col_idx in 0..visible_cols {
                let col_idx = self.app.scroll_col + visible_col_idx;
                if col_idx >= self.app.sheet.max_cols {
                    break;
                }

                let is_active = row_idx == self.app.cursor_row && col_idx == self.app.cursor_col;
                let cell_value = self.app.sheet.get_cell(row_idx, col_idx).unwrap_or("");

                let text_max_len = (self.col_width as usize) - 1;
                let display_text = if cell_value.len() > text_max_len {
                    format!("{}…", &cell_value[..text_max_len - 1])
                } else {
                    format!("{:<width$}", cell_value, width = text_max_len)
                };

                // Choose highlight vs standard style
                let current_cell_style = if is_active {
                    active_cell_style
                } else {
                    cell_text_style
                };

                buf.set_string(cell_x, y_offset, &display_text, current_cell_style);
                buf.set_string(
                    cell_x + (text_max_len as u16),
                    y_offset,
                    "|",
                    cell_border_style,
                );

                cell_x += self.col_width;
            }

            y_offset += 1;
        }
    }
}
