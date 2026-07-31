use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

// ============================================================================
// 1. Data Model
// ============================================================================

pub struct Spreadsheet {
    /// 2D grid containing cell string values
    pub data: Vec<Vec<String>>,
    /// Fixed row/column bounds for display
    pub max_rows: usize,
    pub max_cols: usize,
}

impl Spreadsheet {
    /// Load a CSV file and pad/trim it to a fixed dimension
    pub fn load_from_csv<P: AsRef<Path>>(
        path: P,
        max_rows: usize,
        max_cols: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false) // Treat row 0 as regular data
            .from_reader(file);

        let mut data = Vec::new();

        for record_result in rdr.records().take(max_rows) {
            let record = record_result?;
            let mut row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

            // Ensure row reaches max_cols
            row.resize(max_cols, String::new());
            data.push(row);
        }

        // Fill remaining rows if CSV had fewer than max_rows
        while data.len() < max_rows {
            data.push(vec![String::new(); max_cols]);
        }

        Ok(Self {
            data,
            max_rows,
            max_cols,
        })
    }
}

// ============================================================================
// 2. Custom Grid Widget
// ============================================================================

pub struct CsvGrid<'a> {
    sheet: &'a Spreadsheet,
    col_width: u16,
    header_width: u16,
}

impl<'a> CsvGrid<'a> {
    pub fn new(sheet: &'a Spreadsheet) -> Self {
        Self {
            sheet,
            col_width: 12,   // Fixed width per cell column
            header_width: 5, // Width for row label column ("1 ", "2 ", etc.)
        }
    }

    /// Helper to convert column index to Excel-style letters (0 -> "A", 25 -> "Z", 26 -> "AA")
    fn col_to_letter(mut col: usize) -> String {
        let mut result = String::new();
        loop {
            let rem = (col % 26) as u8;
            result.insert(0, (b'A' + rem) as char);
            if col < 26 {
                break;
            }
            col = (col / 26) - 1;
        }
        result
    }
}

impl<'a> Widget for CsvGrid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < self.header_width || area.height < 2 {
            return; // Not enough screen space to draw
        }

        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let cell_border_style = Style::default().fg(Color::DarkGray);
        let cell_text_style = Style::default().fg(Color::Reset);

        // --- 1. Draw Top Left Corner Blank Box ---
        let corner_str = format!("{:>width$} ", "", width = (self.header_width as usize) - 1);
        buf.set_string(area.x, area.y, &corner_str, header_style);

        // --- 2. Draw Column Headers (A, B, C...) ---
        let mut x_offset = area.x + self.header_width;
        let mut visible_cols = 0;

        for col_idx in 0..self.sheet.max_cols {
            if x_offset + self.col_width > area.x + area.width {
                break; // Boundary check: column doesn't fit on screen
            }

            let col_label = Self::col_to_letter(col_idx);
            let formatted_header = format!(
                "{:^width$}|",
                col_label,
                width = (self.col_width as usize) - 1
            );

            buf.set_string(x_offset, area.y, &formatted_header, header_style);
            x_offset += self.col_width;
            visible_cols += 1;
        }

        // --- 3. Draw Rows & Cells ---
        let mut y_offset = area.y + 1;

        for row_idx in 0..self.sheet.max_rows {
            if y_offset >= area.y + area.height {
                break; // Boundary check: row doesn't fit on screen
            }

            // A. Draw Row Header (1, 2, 3...)
            let row_label = format!(
                "{:>width$} ",
                row_idx + 1,
                width = (self.header_width as usize) - 1
            );
            buf.set_string(area.x, y_offset, &row_label, header_style);

            // B. Draw Cells in this Row
            let mut cell_x = area.x + self.header_width;
            for col_idx in 0..visible_cols {
                let cell_value = &self.sheet.data[row_idx][col_idx];

                // Truncate or pad string to fit inside fixed cell width
                let text_max_len = (self.col_width as usize) - 1;
                let display_text = if cell_value.len() > text_max_len {
                    format!("{}…", &cell_value[..text_max_len - 1])
                } else {
                    format!("{:<width$}", cell_value, width = text_max_len)
                };

                // Render cell contents
                buf.set_string(cell_x, y_offset, &display_text, cell_text_style);

                // Render right cell separator border "|"
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

// ============================================================================
// 3. Application Lifecycle
// ============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    // A. Setup Terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // B. Load Data (Fixed grid size: 50 rows x 20 columns)
    let csv_path = "data/smoketest.csv";
    let spreadsheet = Spreadsheet::load_from_csv(csv_path, 50, 20).unwrap_or_else(|_| {
        // Fallback dummy data if file is missing
        let mut fallback = Spreadsheet {
            data: vec![vec![String::from("No CSV"); 20]; 50],
            max_rows: 50,
            max_cols: 20,
        };
        fallback.data[0][0] = String::from("Missing File");
        fallback.data[0][1] = String::from("data/smoketest.csv");
        fallback
    });

    // C. Render Loop
    loop {
        terminal.draw(|frame| {
            let grid = CsvGrid::new(&spreadsheet);
            frame.render_widget(grid, frame.area());
        })?;

        // D. Key Event (Quit on 'q' or Esc)
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    // E. Restore Terminal State
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
