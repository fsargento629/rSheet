use crate::domain::Spreadsheet;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Edit,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct App {
    pub sheet: Spreadsheet,
    pub mode: Mode,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,

    pub edit_buffer: String,
    pub edit_cursor_idx: usize,

    pub status_message: String,
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
            edit_buffer: String::new(),
            edit_cursor_idx: 0,
            status_message: String::from("Press 'a' or F2 to edit | 's' to save | 'q' to quit"),
            should_quit: false,
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.mode = Mode::Edit;
        let current_raw = self
            .sheet
            .get_cell(self.cursor_row, self.cursor_col)
            .map(|c| c.raw.as_str())
            .unwrap_or("");
        self.edit_buffer = current_raw.to_string();
        self.edit_cursor_idx = self.edit_buffer.chars().count();
        self.status_message = String::from("EDIT MODE -- Press Esc or F2 to commit changes");
    }

    pub fn exit_edit_mode(&mut self, commit: bool) {
        if commit {
            self.sheet
                .set_cell(self.cursor_row, self.cursor_col, self.edit_buffer.clone());
            self.status_message = String::from("Cell updated");
        } else {
            self.status_message = String::from("Edit canceled");
        }
        self.mode = Mode::Normal;
        self.edit_buffer.clear();
        self.edit_cursor_idx = 0;
    }

    pub fn handle_edit_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::F(2) | KeyCode::Enter => {
                self.exit_edit_mode(true);
            }
            KeyCode::Left => {
                if self.edit_cursor_idx > 0 {
                    self.edit_cursor_idx -= 1;
                }
            }
            KeyCode::Right => {
                if self.edit_cursor_idx < self.edit_buffer.chars().count() {
                    self.edit_cursor_idx += 1;
                }
            }
            KeyCode::Char(c) => {
                let byte_idx = self.char_idx_to_byte_idx(self.edit_cursor_idx);
                self.edit_buffer.insert(byte_idx, c);
                self.edit_cursor_idx += 1;
            }
            KeyCode::Backspace => {
                if self.edit_cursor_idx > 0 {
                    self.edit_cursor_idx -= 1;
                    let byte_idx = self.char_idx_to_byte_idx(self.edit_cursor_idx);
                    self.edit_buffer.remove(byte_idx);
                }
            }
            KeyCode::Delete => {
                if self.edit_cursor_idx < self.edit_buffer.chars().count() {
                    let byte_idx = self.char_idx_to_byte_idx(self.edit_cursor_idx);
                    self.edit_buffer.remove(byte_idx);
                }
            }
            _ => {}
        }
    }

    fn char_idx_to_byte_idx(&self, char_idx: usize) -> usize {
        self.edit_buffer
            .char_indices()
            .nth(char_idx)
            .map(|(idx, _)| idx)
            .unwrap_or(self.edit_buffer.len())
    }

    pub fn save_spreadsheet(&mut self) {
        if let Some(path) = &self.sheet.loaded_path.clone() {
            match self.sheet.save_to_csv(path) {
                Ok(_) => self.status_message = format!("Saved successfully to {}", path),
                Err(e) => self.status_message = format!("Error saving: {}", e),
            }
        } else {
            self.status_message = String::from("No file path associated with spreadsheet");
        }
    }

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

    fn adjust_viewport(&mut self, visible_rows: usize, visible_cols: usize) {
        if visible_rows == 0 || visible_cols == 0 {
            return;
        }

        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }

        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col - visible_cols + 1;
        }
    }
}
