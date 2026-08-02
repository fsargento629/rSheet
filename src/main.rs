use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::{env, error::Error, io};

mod app;
mod domain;
mod ui;

use app::{App, Direction, Mode};
use domain::Spreadsheet;
use ui::CsvGrid;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Determine target file: CLI arg if provided, otherwise "data/smoketest.csv"
    let file_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("data/smoketest.csv");

    // Initialize spreadsheet with fallback to empty sheet
    let sheet = Spreadsheet::load_from_csv(file_path, 100, 26)
        .unwrap_or_else(|_| Spreadsheet::new(100, 26));

    let mut app = App::new(sheet);

    // Run the main event loop
    let res = run_app(&mut terminal, &mut app);

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Application Error: {:?}", err);
    }

    Ok(())
}

fn get_terminal_rect(
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<Rect, Box<dyn Error>> {
    let size = terminal.size()?;
    Ok(Rect::new(0, 0, size.width, size.height))
}

fn get_visible_dims(
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(usize, usize), Box<dyn Error>> {
    let rect = get_terminal_rect(terminal)?;
    let visible_cols = if rect.width > 6 {
        ((rect.width - 6) / 12) as usize
    } else {
        0
    };
    let visible_rows = if rect.height > 2 {
        (rect.height - 2) as usize
    } else {
        0
    };
    Ok((visible_rows, visible_cols))
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| {
            let size = f.area();
            let grid = CsvGrid::new(app);
            f.render_widget(grid, size);

            if app.mode == Mode::Edit {
                ui::render_edit_modal(f, app);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if app.mode == Mode::Edit {
                app.handle_edit_input(key);
            } else {
                let rect = get_terminal_rect(terminal)?;
                match (key.modifiers, key.code) {
                    // Home Key Combinations
                    (KeyModifiers::CONTROL, KeyCode::Home) => app.move_to_start(),
                    (KeyModifiers::SHIFT, KeyCode::Home) => app.move_to_start_of_col(),
                    (_, KeyCode::Home) => app.move_to_start_line(),

                    // General Actions
                    (_, KeyCode::Char('q')) | (_, KeyCode::Char('Q')) => app.should_quit = true,
                    (_, KeyCode::Char('s')) | (_, KeyCode::Char('S')) => app.save_spreadsheet(),
                    (_, KeyCode::Delete) | (_, KeyCode::Backspace) => app.delete_cell(),

                    // Edit Mode Entry
                    (_, KeyCode::Char('a')) | (_, KeyCode::F(2)) | (_, KeyCode::Enter) => {
                        app.enter_edit_mode(None)
                    }
                    (_, KeyCode::Char('=')) => app.enter_edit_mode(Some('='.to_string())),
                    (_, KeyCode::Char('0'))
                    | (_, KeyCode::Char('1'))
                    | (_, KeyCode::Char('2'))
                    | (_, KeyCode::Char('3'))
                    | (_, KeyCode::Char('4'))
                    | (_, KeyCode::Char('5'))
                    | (_, KeyCode::Char('6'))
                    | (_, KeyCode::Char('7'))
                    | (_, KeyCode::Char('8'))
                    | (_, KeyCode::Char('9')) => app.enter_edit_mode(Some(key.code.to_string())),

                    // Navigation
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                        let (rows, cols) = get_visible_dims(terminal)?;
                        let _ = CsvGrid::new(app).visible_dimensions(rect);
                        app.move_cursor(Direction::Up, rows, cols);
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                        let (rows, cols) = get_visible_dims(terminal)?;
                        let _ = CsvGrid::new(app).visible_dimensions(rect);
                        app.move_cursor(Direction::Down, rows, cols);
                    }
                    (_, KeyCode::Left) | (_, KeyCode::Char('h')) => {
                        let (rows, cols) = get_visible_dims(terminal)?;
                        let _ = CsvGrid::new(app).visible_dimensions(rect);
                        app.move_cursor(Direction::Left, rows, cols);
                    }
                    (_, KeyCode::Right) | (_, KeyCode::Char('l')) => {
                        let (rows, cols) = get_visible_dims(terminal)?;
                        let _ = CsvGrid::new(app).visible_dimensions(rect);
                        app.move_cursor(Direction::Right, rows, cols);
                    }

                    _ => {}
                }
            }
        }
    }
}
