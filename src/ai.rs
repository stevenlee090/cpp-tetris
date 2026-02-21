use crate::game::{GameState, FIELD_HEIGHT, FIELD_WIDTH, TETROMINOES};

/// Returns the (rotation, x) pair that maximises the heuristic score for the
/// current piece.  Called once per piece spawn when AI mode is active.
pub fn compute_best_move(game: &GameState) -> (usize, i32) {
    let mut best_score = f64::NEG_INFINITY;
    let mut best_rotation = game.current_rotation;
    let mut best_x = game.current_x;

    for rotation in 0..4usize {
        for x in -2..(FIELD_WIDTH as i32) {
            // Skip if the piece can't be placed at this column at spawn row
            if !game.does_piece_fit(game.current_piece, rotation, x, 0) {
                continue;
            }

            // Find the final drop y from the top
            let mut drop_y = 0i32;
            while game.does_piece_fit(game.current_piece, rotation, x, drop_y + 1) {
                drop_y += 1;
            }

            let (locked_field, lines) =
                simulate_lock(&game.field, game.current_piece, rotation, x, drop_y);
            let score = score_field(&locked_field, lines);

            if score > best_score {
                best_score = score;
                best_rotation = rotation;
                best_x = x;
            }
        }
    }

    (best_rotation, best_x)
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

/// Removes complete interior rows (values 8 from pending clears count as
/// filled) and returns how many were removed.
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
// Heuristic scoring  (Yiyuan Lee weights)
// ---------------------------------------------------------------------------

fn score_field(field: &[[u8; FIELD_WIDTH]; FIELD_HEIGHT], lines_cleared: u32) -> f64 {
    let heights = column_heights(field);
    let agg_height: i32 = heights.iter().sum();
    let holes = count_holes(field, &heights) as i32;
    let bump = bumpiness(&heights);

    -0.510066 * agg_height as f64
        + 0.760666 * lines_cleared as f64
        - 0.356630 * holes as f64
        - 0.184483 * bump
}

/// Height of each interior column (index 0 = column 1 in the field).
/// Height is the number of rows from the topmost filled cell down to
/// (not including) the bottom border.
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

/// Sum of absolute differences between adjacent column heights.
fn bumpiness(heights: &[i32]) -> f64 {
    heights
        .windows(2)
        .map(|w| (w[0] - w[1]).abs() as f64)
        .sum()
}
