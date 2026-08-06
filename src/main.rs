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
mod normal_mode;
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

            if app.mode == Mode::Insert {
                ui::render_insert_modal(f, app);
            }
        })?;

        // 2. Non-blocking event check
        if event::poll(Duration::from_micros(10))? {
            match event::read()? {
                // ------------------------------------------------------------------
                // Mouse Events
                // ------------------------------------------------------------------
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        // Left click selection / double-click insert
                        MouseEventKind::Down(MouseButton::Left) => {
                            if app.mode == Mode::Normal || app.mode == Mode::Visual {
                                app.handle_mouse_left_click(mouse_event.column, mouse_event.row);
                            }
                        }

                        // Mouse wheel scroll UP / LEFT
                        MouseEventKind::ScrollUp => {
                            if app.mode != Mode::Insert {
                                if mouse_event.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.move_cursor(Direction::Horizontal(-1));
                                } else {
                                    app.move_cursor(Direction::Vertical(-1));
                                }
                            }
                        }

                        // Mouse wheel scroll DOWN / RIGHT
                        MouseEventKind::ScrollDown => {
                            if app.mode != Mode::Insert {
                                if mouse_event.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.move_cursor(Direction::Horizontal(1));
                                } else {
                                    app.move_cursor(Direction::Vertical(1));
                                }
                            }
                        }

                        _ => {}
                    }
                }

                // ------------------------------------------------------------------
                // Key Events
                // ------------------------------------------------------------------
                Event::Key(key) => match app.mode {
                    // ---- Insert mode: all input goes to the cell buffer ----------
                    Mode::Insert => {
                        app.handle_insert_input(key);
                    }

                    // ---- Visual mode: arrows navigate; any char opens Insert -------
                    Mode::Visual => {
                        match (key.modifiers, key.code) {
                            // Exit Visual mode
                            (_, KeyCode::Esc) => app.exit_visual_mode(),

                            // Navigation (arrows only — hjkl intentionally excluded)
                            (KeyModifiers::CONTROL, KeyCode::Home) => app.move_to_start(),
                            (KeyModifiers::SHIFT, KeyCode::Home) => app.move_to_start_of_col(),
                            (_, KeyCode::Home) => app.move_to_start_line(),

                            (_, KeyCode::Up) => app.move_cursor(Direction::Vertical(-1)),
                            (_, KeyCode::Down) | (_, KeyCode::Enter) => {
                                app.move_cursor(Direction::Vertical(1))
                            }
                            (_, KeyCode::Left) => app.move_cursor(Direction::Horizontal(-1)),
                            (_, KeyCode::Right) => app.move_cursor(Direction::Horizontal(1)),

                            // Cell deletion still available in Visual mode
                            (_, KeyCode::Delete) | (_, KeyCode::Backspace) => app.delete_cell(),

                            // Any printable character → Insert mode seeded with that character
                            (_, KeyCode::Char(c)) => app.enter_insert_mode(Some(c.to_string())),

                            _ => {}
                        }
                    }

                    // ---- Normal mode: VIM-style command state machine --------
                    Mode::Normal => {
                        match (key.modifiers, key.code) {
                            // Non-char keys that bypass the state machine
                            // (reset any pending count/operator first).

                            // Cancel / reset pending command
                            (_, KeyCode::Esc) => app.reset_normal_cmd(),

                            // Arrow keys: translate to hjkl and process through normal mode
                            (_, KeyCode::Up) => app.handle_normal_char('k'),
                            (_, KeyCode::Down) => app.handle_normal_char('j'),
                            (_, KeyCode::Left) => app.handle_normal_char('h'),
                            (_, KeyCode::Right) => app.handle_normal_char('l'),

                            // Enter → just like 'j' (down) in normal mode
                            (_, KeyCode::Enter) => app.handle_normal_char('j'),

                            // Home key family
                            (KeyModifiers::CONTROL, KeyCode::Home) => {
                                app.reset_normal_cmd();
                                app.move_to_start();
                            }
                            (KeyModifiers::SHIFT, KeyCode::Home) => {
                                app.reset_normal_cmd();
                                app.move_to_start_of_col();
                            }
                            (_, KeyCode::Home) => {
                                app.reset_normal_cmd();
                                app.move_to_start_line();
                            }

                            // F2 → Visual mode
                            (_, KeyCode::F(2)) => {
                                app.reset_normal_cmd();
                                app.enter_visual_mode();
                            }

                            // Delete / Backspace → clear current cell
                            (_, KeyCode::Delete) | (_, KeyCode::Backspace) => {
                                app.reset_normal_cmd();
                                app.delete_cell();
                            }

                            // All printable characters → VIM state machine
                            (_, KeyCode::Char(c)) => {
                                app.handle_normal_char(c);
                            }

                            _ => {}
                        }
                    }
                },

                // Catch-all for Resize, FocusGained, FocusLost, Paste, etc.
                _ => {}
            }
        }
    }
}
