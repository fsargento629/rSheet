mod app;
mod domain;
mod ui;

use std::error::Error;
use std::io;
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::{App, Direction, Mode};
use domain::Spreadsheet;
use ui::CsvGrid;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // 2. Load Data
    let csv_path = "data/smoketest.csv";
    let spreadsheet = Spreadsheet::load_from_csv(csv_path, 100, 50).unwrap_or_else(|_| {
        let mut fallback = Spreadsheet::new(100, 50);
        fallback.loaded_path = Some(csv_path.to_string());
        fallback.data[0][0] = String::from("Missing");
        fallback.data[0][1] = String::from("data/smoketest.csv");
        fallback
    });

    let mut app = App::new(spreadsheet);

    // 3. App Loop
    while !app.should_quit {
        terminal.draw(|frame| {
            let area = frame.area();
            let grid = CsvGrid::new(&app);
            let (vis_rows, vis_cols) = grid.visible_dimensions(area);

            frame.render_widget(grid, area);

            if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    match app.mode {
                        Mode::Normal => match key.code {
                            // Enter Edit Mode
                            KeyCode::Char('a') | KeyCode::F(2) => app.enter_edit_mode(),

                            // Save to CSV
                            KeyCode::Char('s') => app.save_spreadsheet(),

                            // Quit
                            KeyCode::Char('q') => app.should_quit = true,

                            // Navigation
                            KeyCode::Char('h') | KeyCode::Left => {
                                app.move_cursor(Direction::Left, vis_rows, vis_cols);
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                app.move_cursor(Direction::Down, vis_rows, vis_cols);
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.move_cursor(Direction::Up, vis_rows, vis_cols);
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                app.move_cursor(Direction::Right, vis_rows, vis_cols);
                            }
                            _ => {}
                        },
                        Mode::Edit => {
                            app.handle_edit_input(key);
                        }
                    }
                }
            }
        })?;
    }

    // 4. Restore Terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
