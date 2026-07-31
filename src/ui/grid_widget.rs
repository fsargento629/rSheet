use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::app::{App, Mode};
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

    pub fn visible_dimensions(&self, area: Rect) -> (usize, usize) {
        if area.width < self.header_width || area.height < 3 {
            return (0, 0);
        }

        let visible_cols = ((area.width - self.header_width) / self.col_width) as usize;
        let visible_rows = (area.height - 2) as usize;

        (visible_rows, visible_cols)
    }
}

impl<'a> Widget for CsvGrid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < self.header_width || area.height < 3 {
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

        let normal_cursor_style = Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let edit_cursor_style = Style::default()
            .bg(Color::Magenta)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let text_cursor_char_style = Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let (visible_rows, visible_cols) = self.visible_dimensions(area);

        // --- 1. Corner Box ---
        let corner_str = format!("{:>width$} ", "", width = (self.header_width as usize) - 1);
        buf.set_string(area.x, area.y, &corner_str, default_header_style);

        // --- 2. Column Headers ---
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
                width = (self.header_width as usize) - 1
            );
            let row_header_style = if row_idx == self.app.cursor_row {
                active_header_style
            } else {
                default_header_style
            };
            buf.set_string(area.x, y_offset, &row_label, row_header_style);

            // Cells
            let mut cell_x = area.x + self.header_width;
            for visible_col_idx in 0..visible_cols {
                let col_idx = self.app.scroll_col + visible_col_idx;
                if col_idx >= self.app.sheet.max_cols {
                    break;
                }

                let is_active = row_idx == self.app.cursor_row && col_idx == self.app.cursor_col;
                let text_max_len = (self.col_width as usize) - 1;

                if is_active && self.app.mode == Mode::Edit {
                    let raw_buffer = &self.app.edit_buffer;
                    let display_text = if raw_buffer.len() > text_max_len {
                        format!("{}…", &raw_buffer[..text_max_len - 1])
                    } else {
                        format!("{:<width$}", raw_buffer, width = text_max_len)
                    };

                    buf.set_string(cell_x, y_offset, &display_text, edit_cursor_style);

                    if self.app.edit_cursor_idx < text_max_len {
                        let cursor_char = raw_buffer
                            .chars()
                            .nth(self.app.edit_cursor_idx)
                            .unwrap_or(' ');
                        buf.set_string(
                            cell_x + (self.app.edit_cursor_idx as u16),
                            y_offset,
                            cursor_char.to_string(),
                            text_cursor_char_style,
                        );
                    }
                } else {
                    let display_text = if let Some(cell) = self.app.sheet.get_cell(row_idx, col_idx)
                    {
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
                        normal_cursor_style
                    } else {
                        cell_text_style
                    };

                    buf.set_string(cell_x, y_offset, &display_text, current_cell_style);
                }

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

        let mode_str = match self.app.mode {
            Mode::Normal => " NORMAL ",
            Mode::Edit => " EDIT ",
        };

        let status_style = Style::default().bg(Color::DarkGray).fg(Color::White);
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
