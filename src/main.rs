mod game;
mod ui;

use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use game::GameState;
use ui::render_ui;

fn main() -> io::Result<()> {
    // --- Terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let result = run(&mut terminal);

    // --- Terminal cleanup ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut game = GameState::new();

    loop {
        // Draw frame
        terminal.draw(|f| render_ui(f, &game))?;

        // Poll for input with 50ms timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Only process key-press events (ignore release/repeat on some platforms)
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter if game.game_over => {
                            break;
                        }
                        KeyCode::Char('p') if !game.game_over => {
                            game.paused = !game.paused;
                        }
                        KeyCode::Left if !game.game_over => {
                            game.move_left();
                        }
                        KeyCode::Right if !game.game_over => {
                            game.move_right();
                        }
                        KeyCode::Down if !game.game_over => {
                            game.move_down();
                        }
                        KeyCode::Up if !game.game_over => {
                            game.rotate_piece();
                        }
                        KeyCode::Char(' ') if !game.game_over => {
                            game.hard_drop();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Timeout → game tick
            game.tick();
        }
    }

    Ok(())
}
