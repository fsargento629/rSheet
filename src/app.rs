use crate::domain::Spreadsheet;
use crate::normal_mode::{ClipboardContent, Motion, NormalCommand, NormalModeState};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Default navigation mode. hjkl / arrow keys move the cursor.
    /// 'a'/F2 → Visual, 'i' → Insert (blank), '=' → Insert ('='), Enter → Insert (current value).
    Normal,
    /// Focused navigation mode. Arrow keys move the cursor; hjkl do nothing.
    /// Any printable character press enters Insert mode for the current cell.
    /// ESC returns to Normal.
    Visual,
    /// Cell editing mode. Shows the modal input overlay.
    /// Enter commits the buffer; ESC discards and returns to Visual.
    Insert,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Vertical(isize),   // Positive = Down, Negative = Up
    Horizontal(isize), // Positive = Right, Negative = Left
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
    /// Normal-mode command state machine (tracks pending counts/operators).
    pub normal_cmd: NormalModeState,
    /// Clipboard for yank/paste operations.
    pub clipboard: Option<ClipboardContent>,

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
            status_message: String::from(
                "NORMAL -- 'a'/F2: visual mode | 'i': insert | '=': formula | 's': save | 'q': quit",
            ),
            should_quit: false,
            normal_cmd: NormalModeState::new(),
            clipboard: None,
            last_click_time: None,
            last_clicked_cell: None,
        }
    }

    // -------------------------------------------------------------------------
    // Mode transitions
    // -------------------------------------------------------------------------

    /// Switch from Normal → Visual mode. No cell buffer is prepared.
    pub fn enter_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        self.status_message =
            String::from("VISUAL -- Arrows: navigate | Type to insert | Esc: back to normal");
    }

    /// Switch from Visual → Normal mode.
    pub fn exit_visual_mode(&mut self) {
        self.mode = Mode::Normal;
        self.status_message = String::from(
            "NORMAL -- 'a'/F2: visual mode | 'i': insert | '=': formula | 's': save | 'q': quit",
        );
    }

    /// Switch to Insert mode, optionally with an initial buffer value.
    /// - `None`  → loads the current cell's raw content for in-place editing.
    /// - `Some(s)` → seeds the buffer with `s` (e.g. `"="` for formulas, `""` for blank insert).
    pub fn enter_insert_mode(&mut self, initial_value: Option<String>) {
        self.mode = Mode::Insert;

        self.edit_buffer = match initial_value {
            Some(val) => val,
            None => self
                .sheet
                .get_cell(self.cursor_row, self.cursor_col)
                .map(|c| c.raw.clone())
                .unwrap_or_default(),
        };

        self.edit_cursor_idx = self.edit_buffer.chars().count();
        self.status_message = String::from("INSERT -- Enter: commit | Esc: discard");
    }

    /// Commit or discard the current insert buffer, then return to Visual mode.
    pub fn exit_insert_mode(&mut self, commit: bool) {
        if commit {
            self.sheet
                .set_cell(self.cursor_row, self.cursor_col, self.edit_buffer.clone());
            self.status_message = String::from("Cell updated");
        } else {
            self.status_message = String::from("Insert canceled");
        }
        self.mode = Mode::Visual;
        self.edit_buffer.clear();
        self.edit_cursor_idx = 0;
    }

    // -------------------------------------------------------------------------
    // Cell operations
    // -------------------------------------------------------------------------

    pub fn delete_cell(&mut self) {
        self.sheet
            .set_cell(self.cursor_row, self.cursor_col, String::new());
        self.status_message = String::from("Cell deleted");
    }

    /// Handle keystrokes while in Insert mode (the modal input overlay).
    pub fn handle_insert_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::F(2) => {
                self.exit_insert_mode(true);
            }
            KeyCode::Esc => {
                self.exit_insert_mode(false);
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

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Vertical(delta) => {
                if delta >= 0 {
                    let amount = delta as usize;
                    self.cursor_row = (self.cursor_row + amount).min(self.sheet.max_rows - 1);
                } else {
                    let amount = delta.unsigned_abs();
                    self.cursor_row = self.cursor_row.saturating_sub(amount);
                }
            }
            Direction::Horizontal(delta) => {
                if delta >= 0 {
                    let amount = delta as usize;
                    self.cursor_col = (self.cursor_col + amount).min(self.sheet.max_cols - 1);
                } else {
                    let amount = delta.unsigned_abs();
                    self.cursor_col = self.cursor_col.saturating_sub(amount);
                }
            }
        }
        self.adjust_viewport();
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

    /// Move the cursor to the last column.
    pub fn move_to_end_of_row(&mut self) {
        self.cursor_col = self.sheet.max_cols - 1;
        self.adjust_viewport();
    }

    // -------------------------------------------------------------------------
    // Normal-mode command handling
    // -------------------------------------------------------------------------

    /// Reset the normal-mode state machine and refresh the status bar.
    pub fn reset_normal_cmd(&mut self) {
        self.normal_cmd.reset();
        self.update_normal_status();
    }

    /// Update the status message to reflect the current normal-mode state.
    pub fn update_normal_status(&mut self) {
        let pending = self.normal_cmd.pending_keys();
        if pending.is_empty() {
            self.status_message = String::from(
                "NORMAL -- hjkl/arrows: nav | d: delete | y: yank | p: paste | 'a': visual | 'q': quit",
            );
        } else {
            self.status_message = format!(
                "NORMAL [{}] -- Esc: cancel | d/y + motion | dd/yy: row | p/P: paste",
                pending
            );
        }
    }

    /// Feed one character key to the normal-mode state machine and execute any
    /// resulting command.
    pub fn handle_normal_char(&mut self, ch: char) {
        if let Some(cmd) = self.normal_cmd.process_char(ch) {
            self.execute_normal_command(cmd);
        } else {
            self.update_normal_status();
        }
    }

    /// Execute a fully-resolved [`NormalCommand`].
    pub fn execute_normal_command(&mut self, cmd: NormalCommand) {
        match cmd {
            NormalCommand::Move { motion, count } => {
                self.execute_motion(motion, count);
            }
            NormalCommand::Delete { motion, count } => {
                self.execute_delete(motion, count);
            }
            NormalCommand::DeleteRow { count } => {
                let row_end = (self.cursor_row + count - 1).min(self.sheet.max_rows - 1);
                // Yank before delete so dd implicitly fills the clipboard.
                let rows = self.collect_rows(self.cursor_row, row_end);
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.sheet.delete_rows(self.cursor_row, row_end);
                self.cursor_row = self.cursor_row.min(self.sheet.max_rows - 1);
                self.adjust_viewport();
                self.status_message = format!("Deleted {} row(s)", count);
            }
            NormalCommand::Yank { motion, count } => {
                self.execute_yank(motion, count);
            }
            NormalCommand::YankRow { count } => {
                let row_end = (self.cursor_row + count - 1).min(self.sheet.max_rows - 1);
                let rows = self.collect_rows(self.cursor_row, row_end);
                let n = rows.len();
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.status_message = format!("Yanked {} row(s)", n);
            }
            NormalCommand::Paste { before, count } => {
                self.execute_paste(before, count);
            }
            NormalCommand::EnterVisualMode => self.enter_visual_mode(),
            NormalCommand::EnterInsertMode(s) => self.enter_insert_mode(s),
            NormalCommand::Save => self.save_spreadsheet(),
            NormalCommand::Quit => self.should_quit = true,
            NormalCommand::DeleteCell => self.delete_cell(),
            NormalCommand::Reset => {}
        }
        self.update_normal_status();
    }

    // -------------------------------------------------------------------------
    // Motion execution
    // -------------------------------------------------------------------------

    fn execute_motion(&mut self, motion: Motion, count: usize) {
        match motion {
            Motion::Left => self.move_cursor(Direction::Horizontal(-(count as isize))),
            Motion::Right => self.move_cursor(Direction::Horizontal(count as isize)),
            Motion::Up => self.move_cursor(Direction::Vertical(-(count as isize))),
            Motion::Down => self.move_cursor(Direction::Vertical(count as isize)),
            Motion::StartOfRow => self.move_to_start_line(),
            Motion::EndOfRow => self.move_to_end_of_row(),
            Motion::WordForward => self.move_cursor(Direction::Horizontal(count as isize)),
            Motion::WordBack => self.move_cursor(Direction::Horizontal(-(count as isize))),
        }
    }

    // -------------------------------------------------------------------------
    // Delete execution
    // -------------------------------------------------------------------------

    fn execute_delete(&mut self, motion: Motion, count: usize) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let max_col = self.sheet.max_cols - 1;
        let max_row = self.sheet.max_rows - 1;

        match motion {
            // Delete `count` cells to the LEFT of cursor (not including cursor).
            Motion::Left | Motion::WordBack => {
                if col == 0 {
                    return;
                }
                let col_end = col - 1;
                let col_start = col.saturating_sub(count);
                // Yank deleted range into clipboard.
                let cells = self.collect_cells_in_row(row, col_start, col_end);
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.sheet.delete_cells_in_row(row, col_start, col_end);
                // Cursor stays at same index; content under it shifts.
                self.cursor_col = col_start.min(self.sheet.max_cols - 1);
                self.adjust_viewport();
                self.status_message = format!("Deleted {} cell(s) left", col_end - col_start + 1);
            }
            // Delete `count` cells to the RIGHT of cursor (not including cursor).
            Motion::Right | Motion::WordForward => {
                if col >= max_col {
                    return;
                }
                let col_start = col + 1;
                let col_end = (col + count).min(max_col);
                let cells = self.collect_cells_in_row(row, col_start, col_end);
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.sheet.delete_cells_in_row(row, col_start, col_end);
                // Cursor stays at col (same position, same content).
                self.adjust_viewport();
                self.status_message = format!("Deleted {} cell(s) right", col_end - col_start + 1);
            }
            // Delete rows from cursor down to cursor+count (inclusive).
            Motion::Down => {
                let row_end = (row + count).min(max_row);
                let rows = self.collect_rows(row, row_end);
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.sheet.delete_rows(row, row_end);
                self.cursor_row = row.min(self.sheet.max_rows - 1);
                self.adjust_viewport();
                self.status_message = format!("Deleted {} row(s)", row_end - row + 1);
            }
            // Delete rows from cursor-count up to cursor (inclusive).
            Motion::Up => {
                let row_start = row.saturating_sub(count);
                let rows = self.collect_rows(row_start, row);
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.sheet.delete_rows(row_start, row);
                self.cursor_row = row_start.min(self.sheet.max_rows - 1);
                self.adjust_viewport();
                self.status_message = format!("Deleted {} row(s)", row - row_start + 1);
            }
            // Delete from cursor to end of row (cursor included).
            Motion::EndOfRow => {
                let cells = self.collect_cells_in_row(row, col, max_col);
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.sheet.delete_cells_in_row(row, col, max_col);
                self.adjust_viewport();
                self.status_message = String::from("Deleted to end of row");
            }
            // Delete from start of row to cursor-1 (cursor NOT included).
            Motion::StartOfRow => {
                if col == 0 {
                    return;
                }
                let cells = self.collect_cells_in_row(row, 0, col - 1);
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.sheet.delete_cells_in_row(row, 0, col - 1);
                self.cursor_col = 0;
                self.adjust_viewport();
                self.status_message = String::from("Deleted to start of row");
            }
        }
    }

    // -------------------------------------------------------------------------
    // Yank execution
    // -------------------------------------------------------------------------

    fn execute_yank(&mut self, motion: Motion, count: usize) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let max_col = self.sheet.max_cols - 1;
        let max_row = self.sheet.max_rows - 1;

        match motion {
            Motion::Left | Motion::WordBack => {
                if col == 0 {
                    return;
                }
                let col_end = col - 1;
                let col_start = col.saturating_sub(count);
                let cells = self.collect_cells_in_row(row, col_start, col_end);
                let n = cells.len();
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.status_message = format!("Yanked {} cell(s)", n);
            }
            Motion::Right | Motion::WordForward => {
                if col >= max_col {
                    return;
                }
                let col_start = col + 1;
                let col_end = (col + count).min(max_col);
                let cells = self.collect_cells_in_row(row, col_start, col_end);
                let n = cells.len();
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.status_message = format!("Yanked {} cell(s)", n);
            }
            Motion::Down => {
                let row_end = (row + count).min(max_row);
                let rows = self.collect_rows(row, row_end);
                let n = rows.len();
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.status_message = format!("Yanked {} row(s)", n);
            }
            Motion::Up => {
                let row_start = row.saturating_sub(count);
                let rows = self.collect_rows(row_start, row);
                let n = rows.len();
                self.clipboard = Some(ClipboardContent::Rows(rows));
                self.status_message = format!("Yanked {} row(s)", n);
            }
            Motion::EndOfRow => {
                let cells = self.collect_cells_in_row(row, col, max_col);
                let n = cells.len();
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.status_message = format!("Yanked {} cell(s) to end of row", n);
            }
            Motion::StartOfRow => {
                if col == 0 {
                    return;
                }
                let cells = self.collect_cells_in_row(row, 0, col - 1);
                let n = cells.len();
                self.clipboard = Some(ClipboardContent::Cells(cells));
                self.status_message = format!("Yanked {} cell(s) to start of row", n);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Paste execution
    // -------------------------------------------------------------------------

    fn execute_paste(&mut self, before: bool, count: usize) {
        let clipboard = match self.clipboard.clone() {
            Some(c) => c,
            None => {
                self.status_message = String::from("Nothing to paste");
                return;
            }
        };

        match clipboard {
            ClipboardContent::Cells(cells) => {
                // Build the repeated payload.
                let payload: Vec<String> = cells
                    .iter()
                    .cloned()
                    .cycle()
                    .take(cells.len() * count)
                    .collect();
                let col = if before {
                    self.cursor_col
                } else {
                    self.cursor_col + 1
                };
                self.sheet
                    .insert_cells_in_row(self.cursor_row, col, &payload);
                self.adjust_viewport();
                self.status_message = format!("Pasted {} cell(s)", payload.len());
            }
            ClipboardContent::Rows(rows) => {
                let row = if before {
                    self.cursor_row
                } else {
                    self.cursor_row + 1
                };
                // Repeat `count` times.
                let payload: Vec<Vec<String>> = rows
                    .iter()
                    .cloned()
                    .cycle()
                    .take(rows.len() * count)
                    .collect();
                let n = payload.len();
                self.sheet.insert_rows(row, &payload);
                self.adjust_viewport();
                self.status_message = format!("Pasted {} row(s)", n);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Clipboard collection helpers
    // -------------------------------------------------------------------------

    /// Collect raw cell values for columns `[col_start ..= col_end]` of `row`.
    fn collect_cells_in_row(&self, row: usize, col_start: usize, col_end: usize) -> Vec<String> {
        (col_start..=col_end)
            .filter_map(|c| self.sheet.get_cell(row, c).map(|cell| cell.raw.clone()))
            .collect()
    }

    /// Collect full rows (as `Vec<String>` of raw values) for `[row_start ..= row_end]`.
    fn collect_rows(&self, row_start: usize, row_end: usize) -> Vec<Vec<String>> {
        (row_start..=row_end)
            .map(|r| {
                (0..self.sheet.max_cols)
                    .filter_map(|c| self.sheet.get_cell(r, c).map(|cell| cell.raw.clone()))
                    .collect()
            })
            .collect()
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
                    // Trigger insert mode on double-click
                    self.enter_insert_mode(None);
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

    pub fn adjust_viewport(&mut self) {
        let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
        let config = self.grid_config;

        let visible_cols = if term_width > config.header_offset_x {
            ((term_width - config.header_offset_x) / config.cell_width) as usize
        } else {
            0
        };

        // Subtract 1 from height to account for the status bar at the bottom!
        let grid_height = term_height.saturating_sub(1);

        let visible_rows = if grid_height > config.header_offset_y {
            ((grid_height - config.header_offset_y) / config.cell_height) as usize
        } else {
            0
        };

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
    fn test_enter_and_cancel_insert_mode() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        // 'i' from Normal → Insert with blank buffer
        app.enter_insert_mode(Some(String::new()));
        assert_eq!(app.mode, Mode::Insert);

        // Type "123"
        app.handle_insert_input(make_key_event(KeyCode::Char('1')));
        app.handle_insert_input(make_key_event(KeyCode::Char('2')));
        app.handle_insert_input(make_key_event(KeyCode::Char('3')));
        assert_eq!(app.edit_buffer, "123");

        // Cancel with Esc → back to Visual mode
        app.handle_insert_input(make_key_event(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Visual);
        assert_eq!(app.sheet.get_cell(0, 0).map(|c| c.raw.as_str()), Some(""));
    }

    #[test]
    fn test_commit_insert_mode() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        app.enter_insert_mode(Some(String::new()));
        app.handle_insert_input(make_key_event(KeyCode::Char('4')));
        app.handle_insert_input(make_key_event(KeyCode::Char('2')));

        // Commit with Enter → back to Visual mode
        app.handle_insert_input(make_key_event(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Visual);
        assert_eq!(app.sheet.get_cell(0, 0).map(|c| c.raw.as_str()), Some("42"));
    }

    #[test]
    fn test_visual_mode_transition() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        assert_eq!(app.mode, Mode::Normal);
        app.enter_visual_mode();
        assert_eq!(app.mode, Mode::Visual);
        app.exit_visual_mode();
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_formula_shortcut_enters_insert_with_equals() {
        let sheet = Spreadsheet::new(10, 10);
        let mut app = App::new(sheet);

        app.enter_insert_mode(Some("=".to_string()));
        assert_eq!(app.mode, Mode::Insert);
        assert_eq!(app.edit_buffer, "=");
        assert_eq!(app.edit_cursor_idx, 1);
    }
}
