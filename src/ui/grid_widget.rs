use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::app::{App, GridConfig, Mode};
use crate::domain::Spreadsheet;

pub struct CsvGrid<'a> {
    app: &'a App,
    config: GridConfig, // Store it directly on the widget
}

impl<'a> CsvGrid<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            app,
            config: app.grid_config, // Populate it once when the widget is created
        }
    }

    pub fn visible_dimensions(&self, area: Rect) -> (usize, usize) {
        if area.width < self.config.header_offset_x || area.height < self.config.header_offset_y {
            return (0, 0);
        }

        let visible_cols =
            ((area.width - self.config.header_offset_x) / self.config.cell_width) as usize;
        let visible_rows =
            ((area.height - self.config.header_offset_y) / self.config.cell_height) as usize;

        (visible_rows, visible_cols)
    }
}

impl<'a> Widget for CsvGrid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < self.config.header_offset_x || area.height < self.config.header_offset_y {
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

        // --- Per-mode cursor highlight styles ---
        // Normal:  Blue  — default navigation
        // Edit:    Magenta — focused navigation, ready to insert
        // Insert:  Green  — active cell editing (modal is open)
        let normal_cursor_style = Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let edit_cursor_style = Style::default()
            .bg(Color::Magenta)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let insert_cursor_style = Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let (visible_rows, visible_cols) = self.visible_dimensions(area);

        // --- 1. Corner Box ---
        let corner_str = format!(
            "{:>width$} ",
            "",
            width = (self.config.header_offset_x as usize) - 1
        );
        buf.set_string(area.x, area.y, &corner_str, default_header_style);

        // --- 2. Column Headers ---
        let mut x_offset = area.x + self.config.header_offset_x;
        for visible_col_idx in 0..visible_cols {
            let col_idx = self.app.scroll_col + visible_col_idx;
            if col_idx >= self.app.sheet.max_cols {
                break;
            }

            let col_label = Spreadsheet::col_to_letter(col_idx);
            let formatted_header = format!(
                "{:^width$}|",
                col_label,
                width = (self.config.cell_width as usize) - 1
            );

            let header_style = if col_idx == self.app.cursor_col {
                active_header_style
            } else {
                default_header_style
            };

            buf.set_string(x_offset, area.y, &formatted_header, header_style);
            x_offset += self.config.cell_width;
        }

        // --- 3. Grid Rows & Cells ---
        let mut y_offset = area.y + 1;
        for visible_row_idx in 0..visible_rows {
            let row_idx = self.app.scroll_row + visible_row_idx;
            if row_idx >= self.app.sheet.max_rows {
                break;
            }

            // Row Header
            let row_label = format!(
                "{:>width$} ",
                row_idx + 1,
                width = (self.config.header_offset_x as usize) - 1
            );
            let row_header_style = if row_idx == self.app.cursor_row {
                active_header_style
            } else {
                default_header_style
            };
            buf.set_string(area.x, y_offset, &row_label, row_header_style);

            // Cells
            let mut cell_x = area.x + self.config.header_offset_x;
            for visible_col_idx in 0..visible_cols {
                let col_idx = self.app.scroll_col + visible_col_idx;
                if col_idx >= self.app.sheet.max_cols {
                    break;
                }

                let is_active = row_idx == self.app.cursor_row && col_idx == self.app.cursor_col;
                let text_max_len = (self.config.cell_width as usize) - 1;

                let display_text = if let Some(cell) = self.app.sheet.get_cell(row_idx, col_idx) {
                    let val = cell.display_text();
                    if val.len() > text_max_len {
                        format!("{}…", &val[..text_max_len - 1])
                    } else {
                        format!("{:<width$}", val, width = text_max_len)
                    }
                } else {
                    format!("{:<width$}", "", width = text_max_len)
                };

                let current_cell_style = if is_active {
                    match self.app.mode {
                        Mode::Normal => normal_cursor_style,
                        Mode::Edit => edit_cursor_style,
                        Mode::Insert => insert_cursor_style,
                    }
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
                cell_x += self.config.cell_width;
            }

            y_offset += self.config.cell_height;
        }

        // --- 4. Status Bar ---
        let status_y = area.y + area.height - 1;
        let coord_str = format!(
            " [{}{}] ",
            Spreadsheet::col_to_letter(self.app.cursor_col),
            self.app.cursor_row + 1
        );

        let formula_preview = if let Some(cell) = self
            .app
            .sheet
            .get_cell(self.app.cursor_row, self.app.cursor_col)
        {
            if cell.raw.starts_with('=') {
                format!(" Formula: '{}' ", cell.raw)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Mode label and status bar colour vary per mode for instant visual feedback.
        let (mode_str, status_style) = match self.app.mode {
            Mode::Normal => (
                " NORMAL ",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Mode::Edit => (
                " EDIT ",
                Style::default().bg(Color::Magenta).fg(Color::White),
            ),
            Mode::Insert => (
                " INSERT ",
                Style::default().bg(Color::Green).fg(Color::Black),
            ),
        };

        let status_line = format!(
            "{:<width$}",
            format!(
                "{}{}{}{}",
                mode_str, coord_str, formula_preview, self.app.status_message
            ),
            width = area.width as usize
        );

        buf.set_string(area.x, status_y, &status_line, status_style);
    }
}
