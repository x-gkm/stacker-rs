use std::collections::VecDeque;

use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaChaRng;

const PILE_HEIGHT: usize = 40;
pub const PILE_WIDTH: usize = 10;
pub const GRID_HEIGHT: i32 = 20;

struct GameConfig {
    das: u32,
    arr: u32,
    are: u32,
    gravity: u32,
    softdrop: u32,
    clear_delay: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Piece {
    I,
    O,
    T,
    L,
    Z,
    J,
    S,
}

#[derive(Debug, Copy, Clone)]
pub enum Orientation {
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

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Flip,
    Hold,
    Rotate(Direction),
    Move(Direction),
    Harddrop,
    Softdrop,
}

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Begin(Action),
    End(Action),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

struct MovementState {
    das: Option<Direction>,
    move_left: bool,
    move_right: bool,
    soft_dropping: bool,
}

pub struct NextQueue {
    pub pieces: VecDeque<Piece>,
    rng: ChaChaRng,
}

impl NextQueue {
    fn new(seed: u64) -> NextQueue {
        let mut result = NextQueue {
            pieces: VecDeque::new(),
            rng: ChaChaRng::seed_from_u64(seed),
        };

        result.add_bag();

        result
    }
    fn pull(&mut self) -> Piece {
        let result = self.pieces.pop_front().unwrap();
        if self.pieces.len() < 5 {
            self.add_bag();
        }
        result
    }
    fn add_bag(&mut self) {
        use Piece::*;
        let mut bag = [I, O, T, L, Z, J, S];
        bag.shuffle(&mut self.rng);
        self.pieces.extend(bag);
    }
}

pub enum HoldPiece {
    Empty,
    Locked(Piece),
    Unlocked(Piece),
}

struct Timer(u32);

impl Timer {
    fn new() -> Timer {
        Timer(0)
    }

    fn tick(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }

        self.0 -= 1;

        if self.0 == 0 {
            return true;
        }

        false
    }

    fn set(&mut self, n: u32) {
        self.0 = n;
    }

    fn stop(&mut self) {
        self.0 = 0;
    }
}

pub struct Engine {
    pub pile: [[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT],
    pub active_piece: Option<ActivePiece>,
    pub hold: HoldPiece,
    pub next_queue: NextQueue,
    frame_inputs: Vec<Input>,
    movement: MovementState,
    config: GameConfig,
    spawn_timer: Timer,
    fall_timer: Timer,
    das_timer: Timer,
    line_clear_timer: Timer,
}

impl Engine {
    pub fn new(seed: u64) -> Engine {
        let mut spawn_timer = Timer::new();

        spawn_timer.set(60);

        Engine {
            pile: [[None; PILE_WIDTH]; PILE_HEIGHT],
            active_piece: None,
            frame_inputs: Vec::new(),
            movement: MovementState {
                das: None,
                move_left: false,
                move_right: false,
                soft_dropping: false,
            },
            next_queue: NextQueue::new(seed),
            hold: HoldPiece::Empty,
            config: GameConfig {
                das: 6,
                arr: 1,
                are: 6,
                gravity: 60,
                softdrop: 3,
                clear_delay: 6,
            },
            spawn_timer,
            fall_timer: Timer::new(),
            das_timer: Timer::new(),
            line_clear_timer: Timer::new(),
        }
    }

    pub fn queue_input(&mut self, input: Input) {
        self.frame_inputs.push(input);
    }

    fn rotate(&mut self, count: i32) {
        let Some(ref mut active_piece) = self.active_piece else {
            return;
        };

        let mut branched_piece = active_piece.clone();
        branched_piece.orientation.rotate_cw(count);

        for n in 0..5 {
            let (kick_x, kick_y) = kick_offset(
                active_piece.kind,
                active_piece.orientation,
                branched_piece.orientation,
                n,
            );
            branched_piece.x = active_piece.x + kick_x;
            branched_piece.y = active_piece.y + kick_y;
            branched_piece.update_blocks();
            if !check_collision(&self.pile, &branched_piece.blocks) {
                *active_piece = branched_piece;
                break;
            }
        }

        active_piece.update_ghost(&self.pile);
    }

    fn harddrop(&mut self) {
        let Some(ref active_piece) = self.active_piece else {
            return;
        };

        for (x, y) in active_piece.ghost_blocks {
            self.pile[y as usize][x as usize] = Some(active_piece.kind)
        }
        let line_clear = any_lines_to_clear(&self.pile);

        if let HoldPiece::Locked(piece) = self.hold {
            self.hold = HoldPiece::Unlocked(piece);
        }

        self.fall_timer.stop();
        self.active_piece = None;
        if line_clear {
            self.line_clear_timer.set(self.config.clear_delay);
            self.spawn_timer.set(self.config.clear_delay);
        } else {
            self.spawn_timer.set(self.config.are);
        }
    }

    fn do_move(&mut self, direction: Direction) {
        let Some(ref mut active_piece) = self.active_piece else {
            return;
        };

        let mut branched_piece = active_piece.clone();
        branched_piece.x += match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };
        branched_piece.update_blocks();
        if !check_collision(&self.pile, &branched_piece.blocks) {
            *active_piece = branched_piece;
        }
        active_piece.update_ghost(&self.pile);
    }

    fn fall(&mut self) {
        let Some(ref active_piece) = self.active_piece else {
            return;
        };

        let mut branched_piece = active_piece.clone();
        branched_piece.y -= 1;
        branched_piece.update_blocks();
        if !check_collision(&self.pile, &branched_piece.blocks) {
            self.active_piece = Some(branched_piece);
        }
    }

    fn handle_fall(&mut self) {
        self.fall();
        self.fall_timer.set(if self.movement.soft_dropping {
            self.config.softdrop
        } else {
            self.config.gravity
        });
    }

    fn spawn(&mut self, piece: Piece) {
        self.active_piece = Some(ActivePiece::spawn(piece));
        self.active_piece.as_mut().unwrap().update_ghost(&self.pile);
        self.handle_fall();
    }

    pub fn update(&mut self) {
        let inputs: Vec<_> = self.frame_inputs.drain(..).collect();
        for input in inputs {
            use Action::*;
            use Direction::*;
            use Input::*;
            match input {
                Begin(Rotate(Right)) => {
                    self.rotate(1);
                }
                Begin(Flip) => {
                    self.rotate(2);
                }
                Begin(Rotate(Left)) => {
                    self.rotate(3);
                }
                Begin(Hold) => {
                    let Some(ref mut active_piece) = self.active_piece else {
                        continue;
                    };

                    let piece = match self.hold {
                        HoldPiece::Unlocked(piece) => piece,
                        HoldPiece::Empty => self.next_queue.pull(),
                        HoldPiece::Locked(..) => continue,
                    };

                    self.hold = HoldPiece::Locked(active_piece.kind);
                    self.fall_timer.stop();
                    self.spawn(piece);
                }
                Begin(Harddrop) => {
                    self.harddrop();
                }
                Begin(Move(Left)) => {
                    self.movement.move_left = true;
                    self.movement.das = Some(Left);
                    self.do_move(Left);
                    self.das_timer.set(self.config.das);
                }
                Begin(Move(Right)) => {
                    self.movement.move_right = true;
                    self.movement.das = Some(Right);
                    self.do_move(Right);
                    self.das_timer.set(self.config.das);
                }
                End(Move(Left)) => {
                    self.movement.move_left = false;
                    self.das_timer.stop();
                    if self.movement.move_right {
                        self.movement.das = Some(Direction::Right);
                        self.das_timer.set(self.config.das);
                    } else {
                        self.movement.das = None;
                    }
                }
                End(Move(Right)) => {
                    self.movement.move_right = false;
                    self.das_timer.stop();
                    if self.movement.move_left {
                        self.movement.das = Some(Direction::Left);
                        self.das_timer.set(self.config.das);
                    } else {
                        self.movement.das = None;
                    }
                }
                Begin(Softdrop) => {
                    self.fall();
                    self.movement.soft_dropping = true;
                    self.fall_timer.set(self.config.softdrop);
                }
                End(Softdrop) => {
                    self.movement.soft_dropping = false;
                    self.fall_timer.set(self.config.gravity);
                }
                _ => (),
            }
        }

        // line_clear should be called before spawn so that the ghost piece isn't floating.
        if self.line_clear_timer.tick() {
            line_clear(&mut self.pile);
        }
        if self.spawn_timer.tick() {
            let piece = self.next_queue.pull();
            self.spawn(piece);
        }
        if self.fall_timer.tick() {
            self.handle_fall();
        }
        if self.das_timer.tick() {
            self.do_move(self.movement.das.unwrap());
            self.das_timer.set(self.config.arr);
        }
    }
}

fn any_lines_to_clear(pile: &[[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT]) -> bool {
    for row in pile {
        let mut full = true;
        for cell in row {
            if cell.is_none() {
                full = false;
                break;
            }
        }

        if full {
            return true;
        }
    }

    false
}

fn line_clear(pile: &mut [[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT]) {
    for row in (0..PILE_HEIGHT).rev() {
        let mut full = true;
        for cell in &pile[row] {
            if cell.is_none() {
                full = false;
                break;
            }
        }

        if !full {
            continue;
        }

        for ripple in row..PILE_HEIGHT - 1 {
            for cell in 0..PILE_WIDTH {
                pile[ripple][cell] = pile[ripple + 1][cell];
            }
        }

        for cell in 0..PILE_WIDTH {
            pile[PILE_HEIGHT - 1][cell] = None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActivePiece {
    pub kind: Piece,
    orientation: Orientation,
    x: i32,
    y: i32,
    pub blocks: [(i32, i32); 4],
    ghost_y: i32,
    pub ghost_blocks: [(i32, i32); 4],
}

fn kick_offset(piece: Piece, from: Orientation, to: Orientation, n: i32) -> (i32, i32) {
    let (x1, y1) = kick_offset_part(piece, from, n);
    let (x2, y2) = kick_offset_part(piece, to, n);

    (x1 - x2, y1 - y2)
}

fn kick_offset_part(piece: Piece, orientation: Orientation, n: i32) -> (i32, i32) {
    if let Piece::O = piece {
        return match orientation {
            Orientation::N => (0, 0),
            Orientation::E => (0, -1),
            Orientation::S => (-1, -1),
            Orientation::W => (-1, 0),
        };
    }

    let offsets = match (piece, orientation) {
        (Piece::I, Orientation::N) => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        (Piece::I, Orientation::E) => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        (Piece::I, Orientation::S) => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        (Piece::I, Orientation::W) => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
        (_, Orientation::N) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::E) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (_, Orientation::S) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::W) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    };

    offsets[n as usize]
}

impl Piece {
    pub fn blocks(self, orientation: Orientation) -> [(i32, i32); 4] {
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
            ghost_y: 0,
            ghost_blocks: [(0, 0); 4],
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

    fn update_ghost(&mut self, pile: &[[Option<Piece>; 10]; 40]) {
        let mut ghost_piece = self.clone();
        loop {
            let mut branched_piece = ghost_piece.clone();
            branched_piece.y -= 1;
            branched_piece.update_blocks();
            if !check_collision(pile, &branched_piece.blocks) {
                ghost_piece = branched_piece;
            } else {
                break;
            }
        }
        self.ghost_y = ghost_piece.y;
        self.ghost_blocks = ghost_piece.blocks;
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
