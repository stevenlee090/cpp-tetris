use crate::game::{GameState, FIELD_HEIGHT, FIELD_WIDTH, TETROMINOES};

/// Returns the (rotation, x) pair that maximises the heuristic score for the
/// current piece, using one-piece lookahead with the next piece.
pub fn compute_best_move(game: &GameState) -> (usize, i32) {
    let mut best_score = f64::NEG_INFINITY;
    let mut best_rotation = game.current_rotation;
    let mut best_x = game.current_x;

    for rotation in 0..4usize {
        for x in -2..(FIELD_WIDTH as i32) {
            if !piece_fits_field(&game.field, game.current_piece, rotation, x, 0) {
                continue;
            }

            let mut drop_y = 0i32;
            while piece_fits_field(&game.field, game.current_piece, rotation, x, drop_y + 1) {
                drop_y += 1;
            }

            let (locked_field, lines) =
                simulate_lock(&game.field, game.current_piece, rotation, x, drop_y);

            // One-piece lookahead: best score achievable with the next piece
            let next_best = best_placement_score(&locked_field, game.next_piece);
            let score = score_field(&locked_field, lines) + 0.5 * next_best;

            if score > best_score {
                best_score = score;
                best_rotation = rotation;
                best_x = x;
            }
        }
    }

    (best_rotation, best_x)
}

/// Best score achievable by placing `piece` on `field` in any rotation/column.
fn best_placement_score(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT], piece: usize) -> f64 {
    let mut best = f64::NEG_INFINITY;
    for rotation in 0..4usize {
        for x in -2..(FIELD_WIDTH as i32) {
            if !piece_fits_field(field, piece, rotation, x, 0) {
                continue;
            }
            let mut drop_y = 0i32;
            while piece_fits_field(field, piece, rotation, x, drop_y + 1) {
                drop_y += 1;
            }
            let (locked, lines) = simulate_lock(field, piece, rotation, x, drop_y);
            let s = score_field(&locked, lines);
            if s > best {
                best = s;
            }
        }
    }
    if best == f64::NEG_INFINITY { 0.0 } else { best }
}

/// Check whether `piece` at (rotation, pos_x, pos_y) fits in an arbitrary field
/// (no GameState required, so it can be used on simulated boards).
fn piece_fits_field(
    field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT],
    piece: usize,
    rotation: usize,
    pos_x: i32,
    pos_y: i32,
) -> bool {
    let tetromino = TETROMINOES[piece];
    for px in 0..4usize {
        for py in 0..4usize {
            let pi = GameState::rotate(px, py, rotation);
            let fi_x = pos_x + px as i32;
            let fi_y = pos_y + py as i32;
            if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                if fi_x < 0
                    || fi_x >= FIELD_WIDTH as i32
                    || fi_y < 0
                    || fi_y >= FIELD_HEIGHT as i32
                {
                    return false;
                }
                if field[fi_y as usize][fi_x as usize] != 0 {
                    return false;
                }
            }
        }
    }
    true
}

/// Clones the field, writes the piece at (rotation, x, y), clears complete
/// lines, and returns the resulting field together with the number of lines
/// cleared.
fn simulate_lock(
    field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT],
    piece: usize,
    rotation: usize,
    x: i32,
    y: i32,
) -> ([[u8; FIELD_WIDTH]; FIELD_HEIGHT], u32) {
    let mut f = *field;
    let tetromino = TETROMINOES[piece];

    for px in 0..4usize {
        for py in 0..4usize {
            let pi = GameState::rotate(px, py, rotation);
            if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                let fx = x + px as i32;
                let fy = y + py as i32;
                if fx >= 0 && fx < FIELD_WIDTH as i32 && fy >= 0 && fy < FIELD_HEIGHT as i32 {
                    f[fy as usize][fx as usize] = (piece + 1) as u8;
                }
            }
        }
    }

    let lines = clear_lines_sim(&mut f);
    (f, lines)
}

/// Removes complete interior rows (values != 0) and returns how many were removed.
fn clear_lines_sim(field: &mut [[u8; FIELD_WIDTH]; FIELD_HEIGHT]) -> u32 {
    let mut lines = 0u32;
    let mut row = (FIELD_HEIGHT - 2) as i32; // last playfield row (exclude border)
    while row >= 0 {
        let complete = (1..FIELD_WIDTH - 1).all(|c| field[row as usize][c] != 0);
        if complete {
            // Shift everything above this row down by one
            for r in (1..=row as usize).rev() {
                for c in 1..(FIELD_WIDTH - 1) {
                    field[r][c] = field[r - 1][c];
                }
            }
            for c in 1..(FIELD_WIDTH - 1) {
                field[0][c] = 0;
            }
            lines += 1;
            // Re-check the same row index (it now contains the row that was above)
        } else {
            row -= 1;
        }
    }
    lines
}

// ---------------------------------------------------------------------------
// Heuristic scoring
// ---------------------------------------------------------------------------

fn score_field(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT], lines_cleared: u32) -> f64 {
    let heights = column_heights(field);
    let agg_height: i32 = heights.iter().sum();
    let max_height = heights.iter().copied().max().unwrap_or(0);
    let holes = count_holes(field, &heights) as i32;
    let covered = count_covered_holes(field, &heights) as i32;
    let bump = bumpiness(&heights);

    // Steep extra penalty when the stack enters the danger zone (> 12 rows).
    // Each additional row above 12 costs 3× extra to strongly discourage
    // letting the board climb near the top.
    let danger = if max_height > 12 {
        (max_height - 12) as f64 * 3.0
    } else {
        0.0
    };

    -0.510066 * agg_height as f64
        + 0.760666 * lines_cleared as f64
        - 0.75    * holes as f64    // was -0.356630; holes are catastrophic
        - 0.35    * covered as f64  // extra penalty for deeply buried holes
        - 0.356630 * bump           // was -0.184483; high bumpiness blocks future pieces
        - danger
}

/// Height of each interior column (index 0 = column 1 in the field).
fn column_heights(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT]) -> Vec<i32> {
    let num_cols = FIELD_WIDTH - 2; // exclude left/right border columns
    let mut heights = vec![0i32; num_cols];

    for (i, col) in (1..FIELD_WIDTH - 1).enumerate() {
        for row in 0..(FIELD_HEIGHT - 1) {
            if field[row][col] != 0 {
                heights[i] = (FIELD_HEIGHT - 1 - row) as i32;
                break;
            }
        }
    }
    heights
}

/// Count cells that are empty but have a filled cell somewhere above them.
fn count_holes(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT], heights: &[i32]) -> u32 {
    let mut holes = 0u32;
    for (i, &h) in heights.iter().enumerate() {
        if h == 0 {
            continue;
        }
        let col = i + 1; // actual field column
        let top_row = (FIELD_HEIGHT as i32 - 1 - h) as usize;
        for row in (top_row + 1)..(FIELD_HEIGHT - 1) {
            if field[row][col] == 0 {
                holes += 1;
            }
        }
    }
    holes
}

/// Total burial depth: for each hole, count how many filled cells sit above it
/// in the same column.  A hole buried under 3 blocks is far harder to clear
/// than one buried under 1, so this adds a proportional penalty on top of the
/// plain hole count.
fn count_covered_holes(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT], heights: &[i32]) -> u32 {
    let mut total = 0u32;
    for (i, &h) in heights.iter().enumerate() {
        if h == 0 {
            continue;
        }
        let col = i + 1;
        let top_row = (FIELD_HEIGHT as i32 - 1 - h) as usize;
        let mut cover = 0u32;
        for row in top_row..(FIELD_HEIGHT - 1) {
            if field[row][col] != 0 {
                cover += 1;
            } else {
                // This cell is a hole; add the number of blocks overhead
                total += cover;
            }
        }
    }
    total
}

/// Sum of absolute differences between adjacent column heights.
fn bumpiness(heights: &[i32]) -> f64 {
    heights
        .windows(2)
        .map(|w| (w[0] - w[1]).abs() as f64)
        .sum()
}
