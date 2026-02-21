use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use tui_piechart::{PieChart, PieSlice};

use crate::game::{BoardStats, GameState, FIELD_HEIGHT, FIELD_WIDTH, TETROMINOES};

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
        // Build a display buffer: copy field
        let mut display = self.game.field;
        // 0=empty, 1-7=locked piece, 8=cleared, 9=border
        // 10-16 = ghost piece (piece_index+10), rendered as outline

        if !self.game.game_over {
            let tetromino = TETROMINOES[self.game.current_piece];
            let ghost_y = self.game.ghost_drop_y();

            // Draw ghost first (underneath active piece)
            for px in 0..4usize {
                for py in 0..4usize {
                    let pi = GameState::rotate(px, py, self.game.current_rotation);
                    if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                        let fx = self.game.current_x + px as i32;
                        let fy = ghost_y + py as i32;
                        if fx >= 0 && fx < FIELD_WIDTH as i32 && fy >= 0 && fy < FIELD_HEIGHT as i32 {
                            // Only draw ghost where the field is empty
                            if display[fy as usize][fx as usize] == 0 {
                                display[fy as usize][fx as usize] =
                                    (self.game.current_piece + 10) as u8; // ghost marker
                            }
                        }
                    }
                }
            }

            // Draw active piece on top
            for px in 0..4usize {
                for py in 0..4usize {
                    let pi = GameState::rotate(px, py, self.game.current_rotation);
                    if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                        let fx = self.game.current_x + px as i32;
                        let fy = self.game.current_y + py as i32;
                        if fx >= 0 && fx < FIELD_WIDTH as i32 && fy >= 0 && fy < FIELD_HEIGHT as i32 {
                            display[fy as usize][fx as usize] =
                                (self.game.current_piece + 1) as u8;
                        }
                    }
                }
            }
        }

        // Render cell-by-cell; each cell is 2 chars wide
        for row in 0..FIELD_HEIGHT {
            for col in 0..FIELD_WIDTH {
                let cell_x = area.x + (col as u16) * 2;
                let cell_y = area.y + row as u16;

                if cell_x + 1 >= area.x + area.width || cell_y >= area.y + area.height {
                    continue;
                }

                let val = display[row][col];
                let (fg, bg, ch) = if val >= 10 {
                    // Ghost piece: dim outline using piece color, no background fill
                    let color = piece_color(val - 9);
                    (color, Color::Reset, '░')
                } else if val == 0 {
                    (Color::DarkGray, Color::Reset, '·')
                } else {
                    let color = piece_color(val);
                    (color, color, '█')
                };

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

// ---------------------------------------------------------------------------
// Analytics column
// ---------------------------------------------------------------------------

/// Return green / yellow / red depending on whether val is below the two
/// thresholds (lower is better).
fn traffic_light(val: i32, good: i32, warn: i32) -> Color {
    if val <= good {
        Color::Green
    } else if val <= warn {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// A horizontal bar made of block chars, `width` cells wide.
fn filled_bar(count: u32, max: u32, width: usize, fill: &str, empty: &str) -> String {
    let n = if max > 0 {
        ((count as usize) * width / (max as usize)).min(width)
    } else {
        0
    };
    format!("{}{}", fill.repeat(n), empty.repeat(width - n))
}

fn render_analytics(f: &mut Frame, game: &GameState, area: Rect) {
    let stats = game.board_stats();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Board Health
            Constraint::Length(7), // Lines
            Constraint::Length(5), // Efficiency
            Constraint::Length(3), // Trend sparkline
            Constraint::Min(5),    // Pie chart (uses remaining space)
        ])
        .split(area);

    render_board_health(f, &stats, chunks[0]);
    render_lines(f, game, chunks[1]);
    render_efficiency(f, game, chunks[2]);
    render_trend(f, game, chunks[3]);
    render_clears_pie(f, game, chunks[4]);
}

fn render_board_health(f: &mut Frame, stats: &BoardStats, area: Rect) {
    let ht_color = traffic_light(stats.max_height, 8, 12);
    let holes_color = traffic_light(stats.holes as i32, 2, 5);
    let bumpy_color = traffic_light(stats.bumpiness, 5, 10);

    // 12-block bar relative to max playfield height (16)
    let bar_fill = ((stats.max_height as usize) * 12 / 16).min(12);
    let bar = format!("{}{}", "█".repeat(bar_fill), "░".repeat(12 - bar_fill));

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw(" Ht "),
            Span::styled(format!("{:2} ", stats.max_height), Style::default().fg(ht_color).add_modifier(Modifier::BOLD)),
            Span::styled(bar, Style::default().fg(ht_color)),
        ]),
        Line::from(vec![
            Span::raw(" Holes  "),
            Span::styled(format!("{:>15}", stats.holes), Style::default().fg(holes_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(" Bumpy  "),
            Span::styled(format!("{:>15}", stats.bumpiness), Style::default().fg(bumpy_color).add_modifier(Modifier::BOLD)),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Board Health ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_lines(f: &mut Frame, game: &GameState, area: Rect) {
    let max_type = [game.singles, game.doubles, game.triples, game.tetrises]
        .iter()
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);

    let total_clears = game.singles + game.doubles + game.triples + game.tetrises;
    let tetris_pct = if total_clears > 0 {
        game.tetrises * 100 / total_clears
    } else {
        0
    };
    let tetris_color = if tetris_pct >= 20 {
        Color::Cyan
    } else if tetris_pct >= 10 {
        Color::Yellow
    } else {
        Color::Red
    };

    let make_row = |label: &'static str, count: u32, color: Color| -> Line<'static> {
        let bar = filled_bar(count, max_type, 8, "▓", "░");
        Line::from(vec![
            Span::raw(format!(" {} ", label)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!("{:4}", count), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])
    };

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw(" Total  "),
            Span::styled(
                format!("{:>15}", game.lines_cleared),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        make_row("1L", game.singles, Color::White),
        make_row("2L", game.doubles, Color::Yellow),
        make_row("3L", game.triples, Color::Green),
        make_row("4L", game.tetrises, Color::Cyan),
        Line::from(vec![
            Span::raw(" Tetris%"),
            Span::styled(
                format!("{:>14}%", tetris_pct),
                Style::default().fg(tetris_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Lines ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_efficiency(f: &mut Frame, game: &GameState, area: Rect) {
    let score_per_pc = if game.piece_count > 0 {
        game.score / game.piece_count
    } else {
        0
    };
    let lines_per_pc = if game.piece_count > 0 {
        game.lines_cleared as f32 / game.piece_count as f32
    } else {
        0.0
    };
    let elapsed = game.start_time.elapsed().as_secs_f32();
    let pcs_per_sec = if elapsed > 0.5 {
        game.piece_count as f32 / elapsed
    } else {
        0.0
    };

    let lpp_color = if lines_per_pc >= 0.6 {
        Color::Cyan
    } else if lines_per_pc >= 0.3 {
        Color::Yellow
    } else {
        Color::Red
    };

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw(" Score/pc"),
            Span::styled(
                format!("{:>14}", score_per_pc),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Lines/pc"),
            Span::styled(
                format!("{:>13.2}", lines_per_pc),
                Style::default().fg(lpp_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Pcs/sec "),
            Span::styled(
                format!("{:>13.1}", pcs_per_sec),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Efficiency ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_trend(f: &mut Frame, game: &GameState, area: Rect) {
    let bar_chars = ['·', '▂', '▄', '▆', '█'];
    let bar_colors = [Color::DarkGray, Color::White, Color::Yellow, Color::Green, Color::Cyan];

    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for &n in &game.lines_history {
        let idx = (n as usize).min(4);
        spans.push(Span::styled(
            bar_chars[idx].to_string(),
            Style::default().fg(bar_colors[idx]),
        ));
    }
    // pad to fill width
    let filled = game.lines_history.len() + 1;
    let inner_w = area.width.saturating_sub(2) as usize;
    if filled < inner_w {
        spans.push(Span::styled(
            "·".repeat(inner_w - filled),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let text = Text::from(vec![Line::from(spans)]);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Trend (last 20 pieces) ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Green));
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_clears_pie(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Clear Mix ",
            Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::LightBlue));

    let total = game.singles + game.doubles + game.triples + game.tetrises;
    if total == 0 {
        // No data yet — show placeholder
        let text = Text::from(vec![Line::from(Span::styled(
            " No clears yet",
            Style::default().fg(Color::DarkGray),
        ))]);
        f.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let slices = vec![
        PieSlice::new("1L", game.singles as f64, Color::White),
        PieSlice::new("2L", game.doubles as f64, Color::Yellow),
        PieSlice::new("3L", game.triples as f64, Color::Green),
        PieSlice::new("4L", game.tetrises as f64, Color::Cyan),
    ];

    let chart = PieChart::new(slices)
        .block(block)
        .show_legend(true)
        .show_percentages(true)
        .high_resolution(true);
    f.render_widget(chart, area);
}

pub fn render_ui(f: &mut Frame, game: &GameState) {
    let size = f.area();

    // Column widths
    let analytics_width: u16 = 46;
    let board_width = (FIELD_WIDTH as u16) * 2 + 2;
    let sidebar_width: u16 = 22;
    let total_width = analytics_width + board_width + sidebar_width;

    // Center the game horizontally by adding equal padding on both sides
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(total_width.min(size.width)),
            Constraint::Min(0),
        ])
        .split(size);
    let center = h_chunks[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(analytics_width),
            Constraint::Length(board_width),
            Constraint::Length(sidebar_width),
        ])
        .split(center);

    // --- Analytics ---
    render_analytics(f, game, chunks[0]);

    // --- Board ---
    let board_block = Block::default()
        .borders(Borders::ALL)
        .title(" TETRIS ");
    let inner_board = board_block.inner(chunks[1]);
    f.render_widget(board_block, chunks[1]);
    f.render_widget(BoardWidget { game }, inner_board);

    // --- Sidebar ---
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Score
            Constraint::Length(3),  // AI status
            Constraint::Length(6),  // Next piece
            Constraint::Min(0),     // Controls
        ])
        .split(chunks[2]);

    // Score
    let score_text = Text::from(vec![Line::from(format!("Score: {}", game.score))]);
    let score_widget = Paragraph::new(score_text)
        .block(Block::default().borders(Borders::ALL).title(" Score "));
    f.render_widget(score_widget, sidebar_chunks[0]);

    // AI status badge
    let (ai_label, ai_style) = if game.ai_mode {
        (
            "▶ AI: ON ",
            Style::default().fg(Color::Green),
        )
    } else {
        (
            "  AI: OFF",
            Style::default().fg(Color::DarkGray),
        )
    };
    let ai_text = Text::from(vec![Line::from(Span::styled(ai_label, ai_style))]);
    let ai_widget = Paragraph::new(ai_text)
        .block(Block::default().borders(Borders::ALL).title(" AI "));
    f.render_widget(ai_widget, sidebar_chunks[1]);

    // Next piece preview
    let mut next_lines = vec![Line::from("Next:")];
    next_lines.extend(render_next_piece(game));
    let next_widget = Paragraph::new(Text::from(next_lines))
        .block(Block::default().borders(Borders::ALL).title(" Next "));
    f.render_widget(next_widget, sidebar_chunks[2]);

    // Controls
    let controls_text = Text::from(vec![
        Line::from("Controls:"),
        Line::from(""),
        Line::from("← →  Move"),
        Line::from("↑    Rotate"),
        Line::from("↓    Soft drop"),
        Line::from("Spc  Hard drop"),
        Line::from("p    Pause"),
        Line::from("a    AI mode"),
        Line::from("q    Quit"),
    ]);
    let controls_widget = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(controls_widget, sidebar_chunks[3]);

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
