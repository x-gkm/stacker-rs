use std::time::{Duration, Instant};

use macroquad::prelude::*;

const PILE_HEIGHT: usize = 40;
const PILE_WIDTH: usize = 10;
const GRID_HEIGHT: i32 = 20;
const BLOCK_SIZE: f32 = 25.;
const ENGINE_FPS: i32 = 60;
const FRAME_TIME: u128 = 1_000_000_000 / ENGINE_FPS as u128;

#[derive(Debug, Copy, Clone)]
enum Piece {
    I,
    O,
    T,
    L,
    Z,
    J,
    S,
}

#[derive(Debug, Copy, Clone)]
enum Orientation {
    N,
    E,
    S,
    W,
}

impl Orientation {
    fn rotate_cw(&mut self, n: i32) {
        for _ in 0..n {
            use Orientation::*;
            *self = match self {
                N => E,
                E => S,
                S => W,
                W => N,
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct PlayerActions {
    flip: bool,
    hold: bool,
    rotate_left: bool,
    rotate_right: bool,
    harddrop: bool,
    begin_move_right: bool,
    end_move_right: bool,
    begin_move_left: bool,
    end_move_left: bool,
    begin_softdrop: bool,
    end_softdrop: bool,
}

impl PlayerActions {
    fn take_input(&mut self) {
        if is_key_pressed(KeyCode::A) {
            self.hold = true;
        }

        if is_key_pressed(KeyCode::S) {
            self.flip = true;
        }

        if is_key_pressed(KeyCode::D) {
            self.rotate_left = true;
        }

        if is_key_pressed(KeyCode::F) {
            self.rotate_right = true;
        }

        if is_key_pressed(KeyCode::Space) {
            self.harddrop = true;
        }

        if is_key_pressed(KeyCode::J) {
            self.begin_move_left = true;
        }

        if is_key_released(KeyCode::J) {
            self.end_move_left = true;
        }

        if is_key_pressed(KeyCode::K) {
            self.begin_softdrop = true;
        }

        if is_key_released(KeyCode::K) {
            self.end_softdrop = true;
        }

        if is_key_pressed(KeyCode::L) {
            self.begin_move_right = true;
        }

        if is_key_released(KeyCode::L) {
            self.end_move_right = true;
        }
    }
}

struct Engine {
    pile: [[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT],
    active_piece: ActivePiece,
    residue_time: u128,
    frame_actions: PlayerActions,
    gravity_time: i32,
}

impl Engine {
    fn new() -> Engine {
        Engine {
            pile: [[None; PILE_WIDTH]; PILE_HEIGHT],
            active_piece: ActivePiece::spawn(Piece::T),
            residue_time: 0,
            gravity_time: 0,
            frame_actions: Default::default(),
        }
    }

    fn update(&mut self, delta: Duration) {
        let nanos = delta.as_nanos() + self.residue_time;
        self.residue_time = nanos % FRAME_TIME;
        let frames = nanos / FRAME_TIME;
        for _ in 0..frames {
            self.frame();
        }
    }

    fn frame(&mut self) {
        let fa = &mut self.frame_actions;

        if fa.flip {
            fa.flip = false;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.orientation.rotate_cw(2);
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        if fa.rotate_left {
            fa.rotate_left = false;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.orientation.rotate_cw(3);
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        if fa.rotate_right {
            fa.rotate_right = false;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.orientation.rotate_cw(1);
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        if fa.begin_move_left {
            fa.begin_move_left = false;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.x -= 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        if fa.begin_move_right {
            fa.begin_move_right = false;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.x += 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        self.gravity_time += 1;
        if self.gravity_time >= ENGINE_FPS {
            self.gravity_time -= ENGINE_FPS;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.y -= 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }
    }
}

#[macroquad::main("stacker")]
async fn main() {
    let mut engine = Engine::new();
    let mut prev_time = Instant::now();

    loop {
        let time = Instant::now();
        let delta = time - prev_time;

        engine.frame_actions.take_input();

        engine.update(delta);

        clear_background(WHITE);

        let offset_x = (screen_width() - PILE_WIDTH as f32 * BLOCK_SIZE) / 2.;
        let offset_y = (screen_height() - GRID_HEIGHT as f32 * BLOCK_SIZE) / 2.;

        for (y, row) in engine.pile.iter().enumerate() {
            for (x, &block) in row.iter().enumerate() {
                let block_x = offset_x + x as f32 * BLOCK_SIZE;
                let block_y = offset_y + (GRID_HEIGHT - y as i32 - 1) as f32 * BLOCK_SIZE;

                if let Some(piece) = block {
                    draw_rectangle(block_x, block_y, BLOCK_SIZE, BLOCK_SIZE, piece.color());
                } else if y < GRID_HEIGHT as usize {
                    draw_rectangle_lines(block_x, block_y, BLOCK_SIZE, BLOCK_SIZE, 1., GRAY);
                }
            }
        }

        for (x, y) in engine.active_piece.blocks {
            let x = offset_x + x as f32 * BLOCK_SIZE;
            let y = offset_y + (GRID_HEIGHT - y - 1) as f32 * BLOCK_SIZE;

            draw_rectangle(
                x,
                y,
                BLOCK_SIZE,
                BLOCK_SIZE,
                engine.active_piece.kind.color(),
            );
        }

        prev_time = time;
        next_frame().await;
    }
}

#[derive(Debug, Clone)]
struct ActivePiece {
    kind: Piece,
    orientation: Orientation,
    x: i32,
    y: i32,
    blocks: [(i32, i32); 4],
}

impl Piece {
    fn blocks(self, orientation: Orientation) -> [(i32, i32); 4] {
        match (self, orientation) {
            (Piece::I, Orientation::N) => [(0, 0), (-1, 0), (1, 0), (2, 0)],
            (Piece::I, Orientation::E) => [(0, 0), (0, -2), (0, -1), (0, 1)],
            (Piece::I, Orientation::S) => [(0, 0), (-2, 0), (-1, 0), (1, 0)],
            (Piece::I, Orientation::W) => [(0, 0), (0, -1), (0, 1), (0, 2)],

            (Piece::O, Orientation::N) => [(0, 0), (0, 1), (1, 1), (1, 0)],
            (Piece::O, Orientation::E) => [(0, 0), (0, -1), (1, -1), (1, 0)],
            (Piece::O, Orientation::S) => [(0, 0), (0, -1), (-1, -1), (-1, 0)],
            (Piece::O, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (-1, 0)],

            (Piece::T, Orientation::N) => [(0, 0), (-1, 0), (0, 1), (1, 0)],
            (Piece::T, Orientation::E) => [(0, 0), (0, 1), (1, 0), (0, -1)],
            (Piece::T, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, 0)],
            (Piece::T, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (0, -1)],

            (Piece::L, Orientation::N) => [(0, 0), (-1, 0), (1, 1), (1, 0)],
            (Piece::L, Orientation::E) => [(0, 0), (0, 1), (1, -1), (0, -1)],
            (Piece::L, Orientation::S) => [(0, 0), (-1, 0), (-1, -1), (1, 0)],
            (Piece::L, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (0, -1)],

            (Piece::Z, Orientation::N) => [(0, 0), (-1, 1), (0, 1), (1, 0)],
            (Piece::Z, Orientation::E) => [(0, 0), (1, 1), (1, 0), (0, -1)],
            (Piece::Z, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, -1)],
            (Piece::Z, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (-1, -1)],

            (Piece::J, Orientation::N) => [(0, 0), (-1, 0), (-1, 1), (1, 0)],
            (Piece::J, Orientation::E) => [(0, 0), (0, 1), (1, 1), (0, -1)],
            (Piece::J, Orientation::S) => [(0, 0), (-1, 0), (1, -1), (1, 0)],
            (Piece::J, Orientation::W) => [(0, 0), (0, 1), (-1, -1), (0, -1)],

            (Piece::S, Orientation::N) => [(0, 0), (1, 1), (0, 1), (-1, 0)],
            (Piece::S, Orientation::E) => [(0, 0), (1, -1), (1, 0), (0, 1)],
            (Piece::S, Orientation::S) => [(0, 0), (1, 0), (0, -1), (-1, -1)],
            (Piece::S, Orientation::W) => [(0, 0), (0, -1), (-1, 0), (-1, 1)],
        }
    }

    fn color(self) -> Color {
        match self {
            Piece::I => SKYBLUE,
            Piece::O => YELLOW,
            Piece::T => PURPLE,
            Piece::L => ORANGE,
            Piece::Z => RED,
            Piece::J => BLUE,
            Piece::S => GREEN,
        }
    }
}

impl ActivePiece {
    fn spawn(kind: Piece) -> ActivePiece {
        let x = PILE_WIDTH as i32 / 2 - 1;
        let y = GRID_HEIGHT + 2;
        let orientation = Orientation::N;

        let mut result = ActivePiece {
            kind,
            orientation,
            x,
            y,
            blocks: [(0, 0); 4],
        };

        result.update_blocks();

        result
    }

    fn update_blocks(&mut self) {
        self.blocks = self
            .kind
            .blocks(self.orientation)
            .map(|(bx, by)| (self.x + bx, self.y + by));
    }
}

fn check_collision(
    pile: &[[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT],
    blocks: &[(i32, i32)],
) -> bool {
    for &(x, y) in blocks {
        if x < 0 || x >= PILE_WIDTH as i32 || y < 0 || y >= PILE_HEIGHT as i32 {
            return true;
        }

        if pile[y as usize][x as usize].is_some() {
            return true;
        }
    }

    false
}
