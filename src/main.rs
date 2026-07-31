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

use app::{App, Direction};
use domain::Spreadsheet;
use ui::CsvGrid;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // 2. Load Spreadsheet Engine
    let csv_path = "data/smoketest.csv";
    let spreadsheet = Spreadsheet::load_from_csv(csv_path, 100, 50).unwrap_or_else(|_| {
        let mut fallback = Spreadsheet::new(100, 50);
        fallback.data[0][0] = String::from("Missing");
        fallback.data[0][1] = String::from("data/smoketest.csv");
        fallback
    });

    let mut app = App::new(spreadsheet);

    // 3. Application Main Loop
    while !app.should_quit {
        terminal.draw(|frame| {
            let area = frame.area();
            let grid = CsvGrid::new(&app);

            // Compute visible dimensions so navigation scrolling behaves precisely
            let (vis_rows, vis_cols) = grid.visible_dimensions(area);

            frame.render_widget(grid, area);

            // Handle Input Events inside draw pass context for layout dimensions
            if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    match key.code {
                        // Quit bindings
                        KeyCode::Char('q') => app.should_quit = true,

                        // Vim Navigation
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
                    }
                }
            }
        })?;
    }

    // 4. Teardown & Restore Terminal State
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
