use std::collections::VecDeque;
use std::time::{Duration, Instant};

use macroquad::prelude::*;

const PILE_HEIGHT: usize = 40;
const PILE_WIDTH: usize = 10;
const GRID_HEIGHT: i32 = 20;
const BLOCK_SIZE: f32 = 25.;
const ENGINE_FPS: i32 = 60;
const FRAME_TIME: u128 = 1_000_000_000 / ENGINE_FPS as u128;
const ENGINE_DAS: i32 = 6;
const ENGINE_ARR: i32 = 1;

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

enum DasDirection {
    Left,
    Right,
}

struct Engine {
    pile: [[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT],
    active_piece: ActivePiece,
    residue_time: u128,
    frame_actions: PlayerActions,
    gravity_time: i32,
    das: Option<DasDirection>,
    move_left: bool,
    move_right: bool,
    softdrop: bool,
    das_time: i32,
}

impl Engine {
    fn new() -> Engine {
        Engine {
            pile: [[None; PILE_WIDTH]; PILE_HEIGHT],
            active_piece: ActivePiece::spawn(Piece::T),
            residue_time: 0,
            gravity_time: 0,
            frame_actions: Default::default(),
            das: None,
            move_left: false,
            move_right: false,
            softdrop: false,
            das_time: 0,
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
        if self.das.is_some() {
            self.das_time += 1;
        }

        if self.das_time >= ENGINE_DAS {
            self.das_time -= ENGINE_ARR;

            let mut branched_piece = self.active_piece.clone();
            branched_piece.x += match self.das {
                Some(DasDirection::Left) => -1,
                Some(DasDirection::Right) => 1,
                None => unreachable!(),
            };
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        let fa = &mut self.frame_actions;

        if fa.end_move_left {
            fa.end_move_left = false;
            self.move_left = false;
            if self.move_right {
                self.das = Some(DasDirection::Right);
            } else {
                self.das = None;
            }
            self.das_time = 0;
        }

        if fa.end_move_right {
            fa.end_move_right = false;
            self.move_right = false;
            if self.move_left {
                self.das = Some(DasDirection::Left);
            } else {
                self.das = None;
            }
            self.das_time = 0;
        }

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
            self.move_left = true;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.x -= 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
            self.das = Some(DasDirection::Left);
            self.das_time = 0;
        }

        if fa.begin_move_right {
            fa.begin_move_right = false;
            self.move_right = true;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.x += 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
            self.das = Some(DasDirection::Right);
            self.das_time = 0;
        }

        if fa.harddrop {
            fa.harddrop = false;
            loop {
                let mut branched_piece = self.active_piece.clone();
                branched_piece.y -= 1;
                branched_piece.update_blocks();
                if !check_collision(&self.pile, &branched_piece.blocks) {
                    self.active_piece = branched_piece;
                } else {
                    break;
                }
            }
            for (x, y) in self.active_piece.blocks {
                self.pile[y as usize][x as usize] = Some(self.active_piece.kind)
            }
            self.active_piece = ActivePiece::spawn(Piece::T);
        }

        if fa.begin_softdrop {
            fa.begin_softdrop = false;
            self.softdrop = true;
            self.gravity_time = 0;
            let mut branched_piece = self.active_piece.clone();
            branched_piece.y -= 1;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                self.active_piece = branched_piece;
            }
        }

        if fa.end_softdrop {
            fa.end_softdrop = false;
            self.softdrop = false;
        }

        self.gravity_time += 1;
        if self.gravity_time >= if self.softdrop { 5 } else { ENGINE_FPS } {
            self.gravity_time = 0;
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

#[derive(Debug, Clone)]
struct Timer<T>(VecDeque<(u32, T)>);

impl<T> Timer<T> {
    fn new() -> Timer<T> {
        Timer(VecDeque::new())
    }

    fn next_in(&self) -> Option<u32> {
        self.0.front().map(|&(deadline, _)| deadline)
    }

    fn update(&mut self, elapsed: u32) {
        let mut remaining = elapsed;
        for (delta, _) in &mut self.0 {
            let temp = *delta;
            *delta = delta.saturating_sub(elapsed);
            remaining = remaining.saturating_sub(temp);

            if remaining == 0 {
                break;
            }
        }
    }

    fn add(&mut self, deadline: u32, event: T) {
        let mut sum = 0;
        let mut insertion_point = 0;

        for (index, &(delta, _)) in self.0.iter().enumerate() {
            if sum + delta > deadline {
                break;
            }
            insertion_point = index + 1;
            sum += delta;
        }

        let insertion_delta = deadline - sum;

        if let Some((delta, _)) = &mut self.0.get_mut(insertion_point) {
            *delta = delta.saturating_sub(insertion_delta);
        }

        self.0.insert(insertion_point, (insertion_delta, event));
    }

    fn remove(&mut self, event: T) -> Option<u32>
    where
        T: PartialEq,
    {
        let mut sum = 0;
        let mut target = None;

        for (index, &(delta, ref element)) in self.0.iter().enumerate() {
            sum += delta;
            if *element == event {
                target = Some(index);
                break;
            }
        }

        let index = target?;
        let (delta, _) = self.0.remove(index)?;

        if let Some((next_delta, _)) = self.0.get_mut(index) {
            *next_delta += delta;
        }

        Some(sum)
    }

    fn poll(&mut self) -> Option<T> {
        let &(delta, _) = self.0.front()?;

        if delta == 0 {
            self.0.pop_front().map(|(_, event)| event)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_preserves_fifo_order() {
        let mut timer = Timer::new();

        timer.add(0, "testing");
        timer.add(0, "one two three");

        assert_eq!(timer.poll(), Some("testing"));
        assert_eq!(timer.poll(), Some("one two three"));
        assert_eq!(timer.poll(), None);
    }

    #[test]
    fn timer_fires_in_order() {
        let mut timer = Timer::new();

        timer.add(4, "test");
        timer.add(3, "a");
        timer.add(2, "is");
        timer.add(1, "this");

        timer.update(4);

        assert_eq!(timer.poll(), Some("this"));
        assert_eq!(timer.poll(), Some("is"));
        assert_eq!(timer.poll(), Some("a"));
        assert_eq!(timer.poll(), Some("test"));
        assert_eq!(timer.poll(), None);
    }

    #[test]
    fn timer_fires_in_time() {
        let mut timer = Timer::new();

        timer.add(40, 40);
        timer.add(20, 20);
        timer.add(0, 0);
        timer.add(30, 30);
        timer.add(10, 10);

        for i in 0..=41 {
            if let Some(value) = timer.poll() {
                assert_eq!(value, i);
            }
            timer.update(1);
        }
    }

    #[test]
    fn timer_next_in() {
        let mut timer = Timer::new();

        timer.add(0, "hi");
        timer.add(20, "!");
        timer.add(10, "there");

        assert_eq!(timer.next_in(), Some(0));
        assert_eq!(timer.poll(), Some("hi"));

        assert_eq!(timer.next_in(), Some(10));

        timer.update(10);
        assert_eq!(timer.next_in(), Some(0));
        assert_eq!(timer.poll(), Some("there"));

        timer.update(3);
        assert_eq!(timer.next_in(), Some(7));
    }

    #[test]
    fn timer_remove() {
        let mut timer = Timer::new();

        timer.add(100, "boom!");
        timer.update(50);
        assert_eq!(timer.remove("boom!"), Some(50));
        assert_eq!(timer.next_in(), None);
    }

    #[test]
    fn timer_fires_after_remove() {
        let mut timer = Timer::new();

        timer.add(30, 30);
        timer.add(20, 20);
        timer.add(40, 40);
        timer.add(10, 10);
        timer.add(50, 50);

        assert_eq!(timer.remove(50), Some(50));
        assert_eq!(timer.remove(10), Some(10));

        for i in 0..=41 {
            if let Some(value) = timer.poll() {
                assert_eq!(value, i);
            }
            timer.update(1);
        }
    }
}
