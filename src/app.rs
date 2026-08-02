use crate::domain::Spreadsheet;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

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

#[derive(Debug, Clone, Copy)]
pub struct GridConfig {
    pub header_offset_x: u16, // Width reserved for row numbers (e.g., "  1 | ")
    pub header_offset_y: u16, // Height reserved for column letters
    pub cell_width: u16,      // Fixed width of each spreadsheet column
    pub cell_height: u16,     // Height of each row
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            header_offset_x: 6, // 6 characters wide for the left header
            header_offset_y: 1, // 1 line for the top header
            cell_width: 12,     // 12 characters per cell
            cell_height: 1,     // 1 line per cell
        }
    }
}

pub struct App {
    pub sheet: Spreadsheet,
    pub mode: Mode,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub grid_config: GridConfig,

    pub edit_buffer: String,
    pub edit_cursor_idx: usize,

    pub status_message: String,
    pub should_quit: bool,

    // For tracking mouse click events and cell selection
    pub last_click_time: Option<Instant>,
    pub last_clicked_cell: Option<(usize, usize)>,
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
            grid_config: GridConfig::default(),
            edit_buffer: String::new(),
            edit_cursor_idx: 0,
            status_message: String::from("Press 'a' or F2 to edit | 's' to save | 'q' to quit"),
            should_quit: false,
            last_click_time: None,
            last_clicked_cell: None,
        }
    }

    pub fn enter_edit_mode(&mut self, initial_value: Option<String>) {
        self.mode = Mode::Edit;

        self.edit_buffer = match initial_value {
            Some(val) => val,
            None => self
                .sheet
                .get_cell(self.cursor_row, self.cursor_col)
                .map(|c| c.raw.clone())
                .unwrap_or_default(),
        };

        self.edit_cursor_idx = self.edit_buffer.chars().count();
        self.status_message = String::from("EDIT MODE -- Press Enter to commit | Esc to cancel");
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

    pub fn delete_cell(&mut self) {
        self.sheet
            .set_cell(self.cursor_row, self.cursor_col, String::new());
        self.status_message = String::from("Cell deleted");
    }

    pub fn handle_edit_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::F(2) => {
                self.exit_edit_mode(true);
            }
            KeyCode::Esc => {
                self.exit_edit_mode(false);
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
            KeyCode::Home => {
                self.edit_cursor_idx = 0;
            }
            KeyCode::End => {
                self.edit_cursor_idx = self.edit_buffer.chars().count();
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

    pub fn move_to_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.scroll_col = 0;
    }

    pub fn move_to_start_line(&mut self) {
        self.cursor_col = 0;
        self.scroll_col = 0;
    }

    pub fn move_to_start_of_col(&mut self) {
        self.cursor_row = 0;
        self.scroll_row = 0;
    }

    pub fn handle_mouse_left_click(&mut self, mouse_x: u16, mouse_y: u16) {
        let config = self.grid_config;

        // Ensure the click is within the actual spreadsheet grid, not the headers
        if mouse_x >= config.header_offset_x && mouse_y >= config.header_offset_y {
            let relative_x = mouse_x - config.header_offset_x;
            let relative_y = mouse_y - config.header_offset_y;

            // Calculate which visible cell was clicked
            let visible_col = (relative_x / config.cell_width) as usize;
            let visible_row = (relative_y / config.cell_height) as usize;

            // Add scroll offsets to convert relative screen space to absolute sheet coordinates
            let target_col = self.scroll_col + visible_col;
            let target_row = self.scroll_row + visible_row;

            // Validate against total sheet bounds (max_rows / max_cols)
            if target_row < self.sheet.max_rows && target_col < self.sheet.max_cols {
                let now = Instant::now();
                let double_click_threshold = std::time::Duration::from_millis(300);

                // Check if this click matches the previous target cell and occurred within double_click_threshold
                let is_double_click = match (self.last_click_time, self.last_clicked_cell) {
                    (Some(last_time), Some((last_row, last_col))) => {
                        now.duration_since(last_time) <= double_click_threshold
                            && last_row == target_row
                            && last_col == target_col
                    }
                    _ => false,
                };

                // Move selection to target cell regardless
                self.cursor_row = target_row;
                self.cursor_col = target_col;

                if is_double_click {
                    // Trigger edit mode on double-click
                    self.enter_edit_mode(None);
                    // Reset click tracking so a 3rd fast click doesn't trigger another toggle
                    self.last_click_time = None;
                    self.last_clicked_cell = None;
                } else {
                    // Save state for potential second click
                    self.last_click_time = Some(now);
                    self.last_clicked_cell = Some((target_row, target_col));
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_enter_and_cancel_edit_mode() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        app.enter_edit_mode(None);
        assert_eq!(app.mode, Mode::Edit);

        // Type "123"
        app.handle_edit_input(make_key_event(KeyCode::Char('1')));
        app.handle_edit_input(make_key_event(KeyCode::Char('2')));
        app.handle_edit_input(make_key_event(KeyCode::Char('3')));
        assert_eq!(app.edit_buffer, "123");

        // Cancel with Esc
        app.handle_edit_input(make_key_event(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.sheet.get_cell(0, 0).map(|c| c.raw.as_str()), Some(""));
    }

    #[test]
    fn test_commit_edit_mode() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        app.enter_edit_mode(None);
        app.handle_edit_input(make_key_event(KeyCode::Char('4')));
        app.handle_edit_input(make_key_event(KeyCode::Char('2')));

        // Commit with Enter
        app.handle_edit_input(make_key_event(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.sheet.get_cell(0, 0).map(|c| c.raw.as_str()), Some("42"));
    }
}
