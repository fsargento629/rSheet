use crate::domain::Spreadsheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Central application controller & state
pub struct App {
    pub sheet: Spreadsheet,
    pub mode: Mode,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new(sheet: Spreadsheet) -> Self {
        Self {
            sheet,
            mode: Mode::Normal,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            should_quit: false,
        }
    }

    /// Move cursor and auto-adjust viewport scrolling so cursor stays on screen
    pub fn move_cursor(&mut self, direction: Direction, visible_rows: usize, visible_cols: usize) {
        match direction {
            Direction::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_row + 1 < self.sheet.max_rows {
                    self.cursor_row += 1;
                }
            }
            Direction::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            Direction::Right => {
                if self.cursor_col + 1 < self.sheet.max_cols {
                    self.cursor_col += 1;
                }
            }
        }

        self.adjust_viewport(visible_rows, visible_cols);
    }

    /// Ensure the active cell remains visible inside the rendering area
    fn adjust_viewport(&mut self, visible_rows: usize, visible_cols: usize) {
        if visible_rows == 0 || visible_cols == 0 {
            return;
        }

        // Vertical auto-scroll
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }

        // Horizontal auto-scroll
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col - visible_cols + 1;
        }
    }

    pub fn current_cell_coord(&self) -> String {
        format!(
            "{}{}",
            Spreadsheet::col_to_letter(self.cursor_col),
            self.cursor_row + 1
        )
    }
}
