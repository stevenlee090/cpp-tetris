use rand::Rng;
use std::collections::VecDeque;
use std::time::Instant;

pub const FIELD_WIDTH: usize = 12;
pub const FIELD_HEIGHT: usize = 18;

// 7 tetrominoes as 16-char strings (4×4 grids), matching C++ shapes
pub const TETROMINOES: [&str; 7] = [
    "..X...X...X...X.", // I
    "..X..XX...X.....", // S
    ".X...XX...X.....", // Z
    ".X...XX..X......", // O (actually the S-mirror; keeping C++ layout)
    "..X..XX..X......", // T
    ".X...X...XX.....", // L
    "..X...X..XX.....", // J
];

pub struct BoardStats {
    pub max_height: i32,
    pub holes: u32,
    pub bumpiness: i32,
}

pub struct GameState {
    pub field: [[u8; FIELD_WIDTH]; FIELD_HEIGHT],
    pub current_piece: usize,
    pub current_rotation: usize,
    pub current_x: i32,
    pub current_y: i32,
    pub next_piece: usize,
    pub score: u32,
    pub piece_count: u32,
    pub speed: u32,
    pub speed_counter: u32,
    pub lines_to_clear: Vec<usize>,
    pub game_over: bool,
    pub paused: bool,
    pub ai_mode: bool,
    pub ai_target_rotation: usize,
    pub ai_target_x: i32,
    // Analytics
    pub lines_cleared: u32,
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub tetrises: u32,
    pub lines_history: VecDeque<u8>, // lines cleared per last 20 pieces
    pub start_time: Instant,
}

impl GameState {
    pub fn new() -> Self {
        let mut field = [[0u8; FIELD_WIDTH]; FIELD_HEIGHT];

        // Set borders: left/right columns = 9, bottom row = 9
        for y in 0..FIELD_HEIGHT {
            field[y][0] = 9;
            field[y][FIELD_WIDTH - 1] = 9;
        }
        for x in 0..FIELD_WIDTH {
            field[FIELD_HEIGHT - 1][x] = 9;
        }

        let mut rng = rand::thread_rng();
        let current_piece = rng.gen_range(0..7);
        let next_piece = rng.gen_range(0..7);

        let spawn_x = (FIELD_WIDTH as i32 / 2) - 2;
        let mut gs = GameState {
            field,
            current_piece,
            current_rotation: 0,
            current_x: spawn_x,
            current_y: 0,
            next_piece,
            score: 0,
            piece_count: 0,
            speed: 20,
            speed_counter: 0,
            lines_to_clear: Vec::new(),
            game_over: false,
            paused: false,
            ai_mode: false,
            ai_target_rotation: 0,
            ai_target_x: spawn_x,
            lines_cleared: 0,
            singles: 0,
            doubles: 0,
            triples: 0,
            tetrises: 0,
            lines_history: VecDeque::with_capacity(20),
            start_time: Instant::now(),
        };

        // Check if initial piece fits (it should always fit at spawn)
        if !gs.does_piece_fit(gs.current_piece, gs.current_rotation, gs.current_x, gs.current_y) {
            gs.game_over = true;
        }

        gs
    }

    /// Rotate index: convert (px, py, rotation) to tetromino string index
    pub fn rotate(px: usize, py: usize, r: usize) -> usize {
        match r % 4 {
            0 => py * 4 + px,
            1 => 12 + py - (px * 4),
            2 => 15 - (py * 4) - px,
            3 => 3 - py + (px * 4),
            _ => unreachable!(),
        }
    }

    pub fn does_piece_fit(&self, piece: usize, rotation: usize, pos_x: i32, pos_y: i32) -> bool {
        let tetromino = TETROMINOES[piece];
        for px in 0..4usize {
            for py in 0..4usize {
                let pi = Self::rotate(px, py, rotation);
                let fi_x = pos_x + px as i32;
                let fi_y = pos_y + py as i32;

                if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                    // Out of bounds check
                    if fi_x < 0 || fi_x >= FIELD_WIDTH as i32 || fi_y < 0 || fi_y >= FIELD_HEIGHT as i32 {
                        return false;
                    }
                    // Collision with field
                    if self.field[fi_y as usize][fi_x as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Called every 50ms tick
    pub fn tick(&mut self) {
        if self.game_over || self.paused {
            return;
        }

        // Process line clears (flash effect would be here; we clear immediately)
        if !self.lines_to_clear.is_empty() {
            self.clear_lines();
            return;
        }

        self.speed_counter += 1;
        if self.speed_counter >= self.speed {
            self.speed_counter = 0;
            self.force_down();
        }
    }

    fn force_down(&mut self) {
        if self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
            self.current_y += 1;
        } else {
            self.lock_piece();
        }
    }

    pub fn move_left(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x - 1, self.current_y) {
            self.current_x -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x + 1, self.current_y) {
            self.current_x += 1;
        }
    }

    /// Returns the Y position where the current piece would land (for ghost rendering).
    pub fn ghost_drop_y(&self) -> i32 {
        let mut y = self.current_y;
        while self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x, y + 1) {
            y += 1;
        }
        y
    }

    pub fn hard_drop(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        while self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
            self.current_y += 1;
        }
        self.lock_piece();
    }

    pub fn move_down(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
            self.current_y += 1;
        } else {
            self.lock_piece();
        }
    }

    pub fn rotate_piece(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        let new_rotation = (self.current_rotation + 1) % 4;
        if self.does_piece_fit(self.current_piece, new_rotation, self.current_x, self.current_y) {
            self.current_rotation = new_rotation;
        }
    }

    fn lock_piece(&mut self) {
        let tetromino = TETROMINOES[self.current_piece];
        // Write piece to field
        for px in 0..4usize {
            for py in 0..4usize {
                let pi = Self::rotate(px, py, self.current_rotation);
                if tetromino.chars().nth(pi).unwrap_or('.') == 'X' {
                    let fx = self.current_x + px as i32;
                    let fy = self.current_y + py as i32;
                    if fx >= 0 && fx < FIELD_WIDTH as i32 && fy >= 0 && fy < FIELD_HEIGHT as i32 {
                        self.field[fy as usize][fx as usize] = (self.current_piece + 1) as u8;
                    }
                }
            }
        }

        self.score += 25;
        self.piece_count += 1;

        // Speed up every 10 pieces
        if self.piece_count % 10 == 0 && self.speed > 10 {
            self.speed -= 1;
        }

        // Check for completed lines
        self.lines_to_clear.clear();
        for py in 0..4usize {
            let fy = self.current_y + py as i32;
            if fy >= 0 && fy < (FIELD_HEIGHT as i32 - 1) {
                let row = fy as usize;
                let mut line_complete = true;
                for x in 1..(FIELD_WIDTH - 1) {
                    if self.field[row][x] == 0 {
                        line_complete = false;
                        break;
                    }
                }
                if line_complete {
                    self.lines_to_clear.push(row);
                    // Mark row as cleared
                    for x in 1..(FIELD_WIDTH - 1) {
                        self.field[row][x] = 8;
                    }
                }
            }
        }

        // Record per-piece analytics
        let n = self.lines_to_clear.len() as u8;
        self.lines_cleared += n as u32;
        match n {
            1 => self.singles += 1,
            2 => self.doubles += 1,
            3 => self.triples += 1,
            4.. => self.tetrises += 1,
            _ => {}
        }
        if self.lines_history.len() >= 20 {
            self.lines_history.pop_front();
        }
        self.lines_history.push_back(n);

        // Spawn next piece
        let mut rng = rand::thread_rng();
        self.current_piece = self.next_piece;
        self.current_rotation = 0;
        self.current_x = (FIELD_WIDTH as i32 / 2) - 2;
        self.current_y = 0;
        self.next_piece = rng.gen_range(0..7);

        // Check game over
        if !self.does_piece_fit(self.current_piece, self.current_rotation, self.current_x, self.current_y) {
            self.game_over = true;
        }
    }

    /// Snapshot of board quality metrics used by the analytics panel.
    pub fn board_stats(&self) -> BoardStats {
        let num_cols = FIELD_WIDTH - 2;
        let mut heights = vec![0i32; num_cols];
        for (i, col) in (1..FIELD_WIDTH - 1).enumerate() {
            for row in 0..(FIELD_HEIGHT - 1) {
                if self.field[row][col] != 0 {
                    heights[i] = (FIELD_HEIGHT - 1 - row) as i32;
                    break;
                }
            }
        }
        let max_height = heights.iter().copied().max().unwrap_or(0);
        let mut holes = 0u32;
        for (i, &h) in heights.iter().enumerate() {
            if h == 0 {
                continue;
            }
            let col = i + 1;
            let top_row = (FIELD_HEIGHT as i32 - 1 - h) as usize;
            for row in (top_row + 1)..(FIELD_HEIGHT - 1) {
                if self.field[row][col] == 0 {
                    holes += 1;
                }
            }
        }
        let bumpiness: i32 = heights.windows(2).map(|w| (w[0] - w[1]).abs()).sum();
        BoardStats { max_height, holes, bumpiness }
    }

    /// Store the AI's chosen target placement.
    pub fn set_ai_target(&mut self, rotation: usize, x: i32) {
        self.ai_target_rotation = rotation;
        self.ai_target_x = x;
    }

    /// One 50 ms step when AI mode is active.
    /// Rotates and slides 2 steps toward the target each tick (2× speed),
    /// then hard-drops once aligned.
    pub fn ai_step(&mut self) {
        if self.game_over || self.paused {
            self.tick();
            return;
        }
        // Must drain pending line clears via tick() before touching the new
        // piece.  Without this, a second hard_drop can call lock_piece() which
        // does lines_to_clear.clear(), orphaning rows already marked as 8 in
        // the field — they'd never be removed.
        if !self.lines_to_clear.is_empty() {
            self.tick();
            return;
        }
        // 1. Rotate toward target rotation
        if self.current_rotation != self.ai_target_rotation {
            self.rotate_piece();
        }
        // 2. Slide 2 steps toward target x per tick
        for _ in 0..2 {
            if self.current_x < self.ai_target_x {
                self.move_right();
            } else if self.current_x > self.ai_target_x {
                self.move_left();
            }
        }
        // 3. Hard-drop once aligned; otherwise advance gravity normally
        if self.current_rotation == self.ai_target_rotation
            && self.current_x == self.ai_target_x
        {
            self.hard_drop();
        } else {
            self.tick();
        }
    }

    fn clear_lines(&mut self) {
        let lines = self.lines_to_clear.clone();
        let num_lines = lines.len() as u32;

        // Score for line clears: (1 << lines) * 100
        self.score += (1u32 << num_lines) * 100;

        // Remove cleared lines and shift down
        for &row in &lines {
            // Shift everything above this row down by 1
            for y in (1..=row).rev() {
                for x in 1..(FIELD_WIDTH - 1) {
                    self.field[y][x] = self.field[y - 1][x];
                }
            }
            // Clear the top row
            for x in 1..(FIELD_WIDTH - 1) {
                self.field[0][x] = 0;
            }
        }

        self.lines_to_clear.clear();
    }
}
