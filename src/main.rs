use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, error::Error, io};

use std::time::Duration;

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
    execute!(stdout, EnableMouseCapture)?;

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
    execute!(io::stdout(), DisableMouseCapture)?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Application Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        // 1. Render UI continuously at high refresh rate
        terminal.draw(|f| {
            let size = f.area();
            let grid = CsvGrid::new(app);
            f.render_widget(grid, size);

            if app.mode == Mode::Edit {
                ui::render_edit_modal(f, app);
            }
        })?;

        // 2. Non-blocking event check
        if event::poll(Duration::from_micros(10))? {
            match event::read()? {
                // Unified Mouse Event Handler
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        // Left click selection / double-click edit
                        MouseEventKind::Down(MouseButton::Left) => {
                            if app.mode == Mode::Normal {
                                app.handle_mouse_left_click(mouse_event.column, mouse_event.row);
                            }
                        }

                        // Mouse wheel scroll UP / LEFT
                        MouseEventKind::ScrollUp => {
                            if app.mode == Mode::Normal {
                                if mouse_event.modifiers.contains(KeyModifiers::CONTROL) {
                                    // CONTROL + Wheel Up -> Scroll Left
                                    app.move_cursor(Direction::Horizontal(-1));
                                } else {
                                    // Standard Wheel Up -> Scroll Up
                                    app.move_cursor(Direction::Vertical(-1));
                                }
                            }
                        }

                        // Mouse wheel scroll DOWN / RIGHT
                        MouseEventKind::ScrollDown => {
                            if app.mode == Mode::Normal {
                                if mouse_event.modifiers.contains(KeyModifiers::CONTROL) {
                                    // CONTROL + Wheel Down -> Scroll Right
                                    app.move_cursor(Direction::Horizontal(1));
                                } else {
                                    // Standard Wheel Down -> Scroll Down
                                    app.move_cursor(Direction::Vertical(1));
                                }
                            }
                        }

                        _ => {}
                    }
                }

                // Unified Key Event Handler
                Event::Key(key) => {
                    if app.mode == Mode::Edit {
                        app.handle_edit_input(key);
                    } else {
                        match (key.modifiers, key.code) {
                            // Home Key Combinations
                            (KeyModifiers::CONTROL, KeyCode::Home) => app.move_to_start(),
                            (KeyModifiers::SHIFT, KeyCode::Home) => app.move_to_start_of_col(),
                            (_, KeyCode::Home) => app.move_to_start_line(),

                            // General Actions
                            (_, KeyCode::Char('q')) | (_, KeyCode::Char('Q')) => {
                                app.should_quit = true
                            }
                            (_, KeyCode::Char('s')) | (_, KeyCode::Char('S')) => {
                                app.save_spreadsheet()
                            }
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
                            | (_, KeyCode::Char('9')) => {
                                app.enter_edit_mode(Some(key.code.to_string()))
                            }

                            // Navigation
                            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                                app.move_cursor(Direction::Vertical(-1));
                            }
                            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                                app.move_cursor(Direction::Vertical(1));
                            }
                            (_, KeyCode::Left) | (_, KeyCode::Char('h')) => {
                                app.move_cursor(Direction::Horizontal(-1));
                            }
                            (_, KeyCode::Right) | (_, KeyCode::Char('l')) => {
                                app.move_cursor(Direction::Horizontal(1));
                            }

                            _ => {}
                        }
                    }
                }

                // Catch-all for Resize, FocusGained, FocusLost, Paste, etc.
                _ => {}
            }
        }
    }
}
