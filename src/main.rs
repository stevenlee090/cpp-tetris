mod ai;
mod audio;
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
    let mut last_piece_count = game.piece_count;
    let mut audio = audio::AudioManager::new(); // None if no audio device

    loop {
        // Draw frame
        terminal.draw(|f| render_ui(f, &game))?;

        // Drain and play any sounds queued by the game logic
        if let Some(ref mut mgr) = audio {
            for event in game.pending_sounds.drain(..) {
                mgr.play_event(&event);
            }
        } else {
            game.pending_sounds.clear();
        }

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
                        KeyCode::Char('m') => {
                            if let Some(ref mut mgr) = audio {
                                mgr.toggle_music();
                            }
                        }
                        KeyCode::Char('a') if !game.game_over => {
                            game.ai_mode = !game.ai_mode;
                            if game.ai_mode {
                                let (rot, x) = ai::compute_best_move(&game);
                                game.set_ai_target(rot, x);
                                last_piece_count = game.piece_count;
                            }
                        }
                        // Movement keys — only when AI is off
                        KeyCode::Left if !game.game_over && !game.ai_mode => {
                            game.move_left();
                        }
                        KeyCode::Right if !game.game_over && !game.ai_mode => {
                            game.move_right();
                        }
                        KeyCode::Down if !game.game_over && !game.ai_mode => {
                            game.move_down();
                        }
                        KeyCode::Up if !game.game_over && !game.ai_mode => {
                            game.rotate_piece();
                        }
                        KeyCode::Char(' ') if !game.game_over && !game.ai_mode => {
                            game.hard_drop();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Timeout → game tick
            if game.ai_mode {
                // Recompute target when a new piece has spawned and the board
                // is settled (no pending line-clear animation).
                if game.piece_count != last_piece_count && game.lines_to_clear.is_empty() {
                    last_piece_count = game.piece_count;
                    let (rot, x) = ai::compute_best_move(&game);
                    game.set_ai_target(rot, x);
                }
                game.ai_step();
            } else {
                game.tick();
            }
        }
    }

    Ok(())
}
