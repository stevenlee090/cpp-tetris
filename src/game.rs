use rand::Rng;

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

        let mut gs = GameState {
            field,
            current_piece,
            current_rotation: 0,
            current_x: (FIELD_WIDTH as i32 / 2) - 2,
            current_y: 0,
            next_piece,
            score: 0,
            piece_count: 0,
            speed: 20,
            speed_counter: 0,
            lines_to_clear: Vec::new(),
            game_over: false,
            paused: false,
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
