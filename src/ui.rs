use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::game::{GameState, FIELD_HEIGHT, FIELD_WIDTH, TETROMINOES};

/// Map piece index (1-7) to a color. 0 = empty, 9 = border, 8 = cleared flash.
fn piece_color(val: u8) -> Color {
    match val {
        1 => Color::Cyan,    // I
        2 => Color::Green,   // S
        3 => Color::Red,     // Z
        4 => Color::Yellow,  // O
        5 => Color::Magenta, // T
        6 => Color::White,   // L
        7 => Color::Blue,    // J
        8 => Color::White,   // cleared line flash
        9 => Color::White,   // border
        _ => Color::DarkGray,
    }
}

struct BoardWidget<'a> {
    game: &'a GameState,
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Build a display buffer: copy field, then draw current piece on top
        let mut display = self.game.field;

        if !self.game.game_over {
            let tetromino = TETROMINOES[self.game.current_piece];
            for px in 0..4usize {
                for py in 0..4usize {
                    let pi = GameState::rotate(px, py, self.game.current_rotation);
                    if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                        let fx = self.game.current_x + px as i32;
                        let fy = self.game.current_y + py as i32;
                        if fx >= 0
                            && fx < FIELD_WIDTH as i32
                            && fy >= 0
                            && fy < FIELD_HEIGHT as i32
                        {
                            display[fy as usize][fx as usize] =
                                (self.game.current_piece + 1) as u8;
                        }
                    }
                }
            }
        }

        // Render cell-by-cell; each cell is 2 chars wide ("██")
        for row in 0..FIELD_HEIGHT {
            for col in 0..FIELD_WIDTH {
                let cell_x = area.x + (col as u16) * 2;
                let cell_y = area.y + row as u16;

                if cell_x + 1 >= area.x + area.width || cell_y >= area.y + area.height {
                    continue;
                }

                let val = display[row][col];
                let color = piece_color(val);
                let bg = if val == 0 { Color::Reset } else { color };
                let fg = if val == 0 { Color::DarkGray } else { color };
                let ch = if val == 0 { '·' } else { '█' };

                let style = Style::default().fg(fg).bg(bg);
                buf[(cell_x, cell_y)].set_char(ch).set_style(style);
                buf[(cell_x + 1, cell_y)].set_char(ch).set_style(style);
            }
        }
    }
}

fn render_next_piece(game: &GameState) -> Vec<Line<'static>> {
    let tetromino = TETROMINOES[game.next_piece];
    let color = piece_color((game.next_piece + 1) as u8);
    let mut lines = Vec::new();

    for py in 0..4usize {
        let mut spans = Vec::new();
        for px in 0..4usize {
            let pi = py * 4 + px;
            let ch = tetromino.chars().nth(pi).unwrap_or('.');
            if ch == 'X' {
                spans.push(Span::styled("██", Style::default().fg(color)));
            } else {
                spans.push(Span::raw("  "));
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}

pub fn render_ui(f: &mut Frame, game: &GameState) {
    let size = f.area();

    // Board area: FIELD_WIDTH * 2 chars wide + 2 for borders
    let board_width = (FIELD_WIDTH as u16) * 2 + 2;
    let sidebar_width = size.width.saturating_sub(board_width);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(board_width), Constraint::Min(sidebar_width)])
        .split(size);

    // --- Board ---
    let board_block = Block::default()
        .borders(Borders::ALL)
        .title(" TETRIS ");
    let inner_board = board_block.inner(chunks[0]);
    f.render_widget(board_block, chunks[0]);
    f.render_widget(BoardWidget { game }, inner_board);

    // --- Sidebar ---
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Score
            Constraint::Length(6),  // Next piece
            Constraint::Min(0),     // Controls
        ])
        .split(chunks[1]);

    // Score
    let score_text = Text::from(vec![Line::from(format!("Score: {}", game.score))]);
    let score_widget = Paragraph::new(score_text)
        .block(Block::default().borders(Borders::ALL).title(" Score "));
    f.render_widget(score_widget, sidebar_chunks[0]);

    // Next piece preview
    let mut next_lines = vec![Line::from("Next:")];
    next_lines.extend(render_next_piece(game));
    let next_widget = Paragraph::new(Text::from(next_lines))
        .block(Block::default().borders(Borders::ALL).title(" Next "));
    f.render_widget(next_widget, sidebar_chunks[1]);

    // Controls
    let controls_text = Text::from(vec![
        Line::from("Controls:"),
        Line::from(""),
        Line::from("← →  Move"),
        Line::from("↑    Rotate"),
        Line::from("↓    Soft drop"),
        Line::from("Spc  Hard drop"),
        Line::from("p    Pause"),
        Line::from("q    Quit"),
    ]);
    let controls_widget = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(controls_widget, sidebar_chunks[2]);

    // Game over overlay
    if game.game_over {
        render_game_over(f, size, game.score);
    }

    // Paused overlay
    if game.paused && !game.game_over {
        render_paused(f, size);
    }
}

fn render_game_over(f: &mut Frame, area: Rect, score: u32) {
    let popup_width = 30u16;
    let popup_height = 7u16;
    let popup_x = area.x + area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.y + area.height.saturating_sub(popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width.min(area.width), popup_height.min(area.height));

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  G A M E  O V E R  ",
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(format!("  Final Score: {}", score)),
        Line::from(""),
        Line::from("  Press q or Enter to exit"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Game Over ")
        .style(Style::default().bg(Color::Black));

    let widget = Paragraph::new(text).block(block);
    f.render_widget(widget, popup_area);
}

fn render_paused(f: &mut Frame, area: Rect) {
    let popup_width = 24u16;
    let popup_height = 5u16;
    let popup_x = area.x + area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.y + area.height.saturating_sub(popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width.min(area.width), popup_height.min(area.height));

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "    P A U S E D    ",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("  Press p to resume"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Paused ")
        .style(Style::default().bg(Color::Black));

    let widget = Paragraph::new(text).block(block);
    f.render_widget(widget, popup_area);
}
