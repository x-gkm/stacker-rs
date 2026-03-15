#![no_std]

use heapless::Deque;

use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};

pub const PILE_HEIGHT: usize = 40;
pub const PILE_WIDTH: usize = 10;
pub const GRID_HEIGHT: i32 = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GameConfig {
    das: u32,
    arr: u32,
    are: u32,
    gravity: u32,
    softdrop: u32,
    clear_delay: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl PieceKind {
    pub fn blocks(self, orientation: Orientation) -> [(i32, i32); 4] {
        match (self, orientation) {
            (PieceKind::I, Orientation::N) => [(0, 0), (-1, 0), (1, 0), (2, 0)],
            (PieceKind::I, Orientation::E) => [(0, 0), (0, -2), (0, -1), (0, 1)],
            (PieceKind::I, Orientation::S) => [(0, 0), (-2, 0), (-1, 0), (1, 0)],
            (PieceKind::I, Orientation::W) => [(0, 0), (0, -1), (0, 1), (0, 2)],

            (PieceKind::J, Orientation::N) => [(0, 0), (-1, 0), (-1, 1), (1, 0)],
            (PieceKind::J, Orientation::E) => [(0, 0), (0, 1), (1, 1), (0, -1)],
            (PieceKind::J, Orientation::S) => [(0, 0), (-1, 0), (1, -1), (1, 0)],
            (PieceKind::J, Orientation::W) => [(0, 0), (0, 1), (-1, -1), (0, -1)],

            (PieceKind::L, Orientation::N) => [(0, 0), (-1, 0), (1, 1), (1, 0)],
            (PieceKind::L, Orientation::E) => [(0, 0), (0, 1), (1, -1), (0, -1)],
            (PieceKind::L, Orientation::S) => [(0, 0), (-1, 0), (-1, -1), (1, 0)],
            (PieceKind::L, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (0, -1)],

            (PieceKind::O, Orientation::N) => [(0, 0), (0, 1), (1, 1), (1, 0)],
            (PieceKind::O, Orientation::E) => [(0, 0), (0, -1), (1, -1), (1, 0)],
            (PieceKind::O, Orientation::S) => [(0, 0), (0, -1), (-1, -1), (-1, 0)],
            (PieceKind::O, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (-1, 0)],

            (PieceKind::S, Orientation::N) => [(0, 0), (1, 1), (0, 1), (-1, 0)],
            (PieceKind::S, Orientation::E) => [(0, 0), (1, -1), (1, 0), (0, 1)],
            (PieceKind::S, Orientation::S) => [(0, 0), (1, 0), (0, -1), (-1, -1)],
            (PieceKind::S, Orientation::W) => [(0, 0), (0, -1), (-1, 0), (-1, 1)],

            (PieceKind::T, Orientation::N) => [(0, 0), (-1, 0), (0, 1), (1, 0)],
            (PieceKind::T, Orientation::E) => [(0, 0), (0, 1), (1, 0), (0, -1)],
            (PieceKind::T, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, 0)],
            (PieceKind::T, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (0, -1)],

            (PieceKind::Z, Orientation::N) => [(0, 0), (-1, 1), (0, 1), (1, 0)],
            (PieceKind::Z, Orientation::E) => [(0, 0), (1, 1), (1, 0), (0, -1)],
            (PieceKind::Z, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, -1)],
            (PieceKind::Z, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (-1, -1)],
        }
    }
}

pub type Cell = Option<PieceKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Flip,
    Hold,
    Rotate(Direction),
    Move(Direction),
    Harddrop,
    Softdrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input {
    Begin(Action),
    End(Action),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MovementState {
    das: Option<Direction>,
    move_left: bool,
    move_right: bool,
    soft_dropping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NextQueue {
    pieces: Deque<PieceKind, 13>,
    rng: ChaChaRng,
}

impl NextQueue {
    fn new(seed: u64) -> NextQueue {
        let mut result = NextQueue {
            pieces: Deque::new(),
            rng: ChaChaRng::seed_from_u64(seed),
        };

        result.add_bag();

        result
    }
    fn pull(&mut self) -> PieceKind {
        let result = self.pieces.pop_front().unwrap();
        if self.pieces.len() < 5 {
            self.add_bag();
        }
        result
    }
    fn add_bag(&mut self) {
        use PieceKind::*;
        let mut bag = [I, J, L, O, S, T, Z];
        bag.shuffle(&mut self.rng);
        self.pieces.extend(bag);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldPiece {
    Empty,
    Locked(PieceKind),
    Unlocked(PieceKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Engine {
    pile: Pile,
    active_piece: Option<Piece>,
    ghost_piece: Option<Piece>,
    hold: HoldPiece,
    next_queue: NextQueue,
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
            pile: Pile::new(),
            active_piece: None,
            ghost_piece: None,
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
            if !self.pile.check_collision(&branched_piece.blocks) {
                *active_piece = branched_piece;
                break;
            }
        }

        self.ghost_piece = Some(self.pile.calculate_ghost(active_piece));
    }

    fn harddrop(&mut self) {
        let Some(ref ghost_piece) = self.ghost_piece else {
            return;
        };

        for (x, y) in ghost_piece.blocks {
            self.pile.0[y as usize][x as usize] = Some(ghost_piece.kind)
        }
        let line_clear = self.pile.any_lines_to_clear();

        if let HoldPiece::Locked(piece) = self.hold {
            self.hold = HoldPiece::Unlocked(piece);
        }

        self.fall_timer.stop();
        self.active_piece = None;
        self.ghost_piece = None;
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
        if !self.pile.check_collision(&branched_piece.blocks) {
            *active_piece = branched_piece;
        }
        self.ghost_piece = Some(self.pile.calculate_ghost(active_piece))
    }

    fn fall(&mut self) {
        let Some(ref active_piece) = self.active_piece else {
            return;
        };

        let mut branched_piece = active_piece.clone();
        branched_piece.y -= 1;
        branched_piece.update_blocks();
        if !self.pile.check_collision(&branched_piece.blocks) {
            self.active_piece = Some(branched_piece);
        }
    }

    fn set_fall_timer(&mut self) {
        self.fall_timer.set(if self.movement.soft_dropping {
            self.config.softdrop
        } else {
            self.config.gravity
        });
    }

    fn spawn(&mut self, piece: PieceKind) {
        self.active_piece = Some(Piece::spawn(piece));
        self.ghost_piece = Some(self.pile.calculate_ghost(self.active_piece.as_ref().unwrap()));
        self.fall();
        self.set_fall_timer();
    }

    pub fn update(&mut self, frame_inputs: &[Input]) {
        for input in frame_inputs {
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
            self.pile.line_clear();
        }
        if self.spawn_timer.tick() {
            let piece = self.next_queue.pull();
            self.spawn(piece);
        }
        if self.fall_timer.tick() {
            self.fall();
            self.set_fall_timer();
        }
        if self.das_timer.tick() {
            self.do_move(self.movement.das.unwrap());
            self.das_timer.set(self.config.arr);
        }
    }

    pub fn active_piece(&self) -> &Option<Piece> {
        &self.active_piece
    }

    pub fn ghost_piece(&self) -> &Option<Piece> {
        &self.ghost_piece
    }

    pub fn hold(&self) -> &HoldPiece {
        &self.hold
    }

    pub fn next_queue(&self) -> impl Iterator<Item = PieceKind> {
        self.next_queue.pieces.iter().take(5).copied()
    }

    pub fn pile(&self) -> &[[Cell; PILE_WIDTH]; PILE_HEIGHT] {
        &self.pile.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Pile(#[serde(with = "serde_big_array::BigArray")] [[Cell; PILE_WIDTH]; PILE_HEIGHT]);

impl Pile {
    fn new() -> Pile {
        Pile([[None; PILE_WIDTH]; PILE_HEIGHT])
    }

    fn any_lines_to_clear(&self) -> bool {
        for row in self.0 {
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

    fn line_clear(&mut self) {
        for row in (0..PILE_HEIGHT).rev() {
            let mut full = true;
            for cell in &self.0[row] {
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
                    self.0[ripple][cell] = self.0[ripple + 1][cell];
                }
            }

            for cell in 0..PILE_WIDTH {
                self.0[PILE_HEIGHT - 1][cell] = None;
            }
        }
    }

    fn check_collision(&self, blocks: &[(i32, i32)]) -> bool {
        for &(x, y) in blocks {
            if x < 0 || x >= PILE_WIDTH as i32 || y < 0 || y >= PILE_HEIGHT as i32 {
                return true;
            }

            if self.0[y as usize][x as usize].is_some() {
                return true;
            }
        }

        false
    }

    fn calculate_ghost(&self, piece: &Piece) -> Piece {
        let mut ghost_piece = piece.clone();
        loop {
            let mut branched_piece = ghost_piece.clone();
            branched_piece.y -= 1;
            branched_piece.update_blocks();
            if !self.check_collision(&branched_piece.blocks) {
                ghost_piece = branched_piece;
            } else {
                break;
            }
        }
        ghost_piece
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub orientation: Orientation,
    pub x: i32,
    pub y: i32,
    pub blocks: [(i32, i32); 4],
}

impl Piece {
    fn spawn(kind: PieceKind) -> Piece {
        let x = PILE_WIDTH as i32 / 2 - 1;
        let y = GRID_HEIGHT + 2;
        let orientation = Orientation::N;

        let mut result = Piece {
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

fn kick_offset(piece: PieceKind, from: Orientation, to: Orientation, n: i32) -> (i32, i32) {
    let (x1, y1) = kick_offset_part(piece, from, n);
    let (x2, y2) = kick_offset_part(piece, to, n);

    (x1 - x2, y1 - y2)
}

fn kick_offset_part(piece: PieceKind, orientation: Orientation, n: i32) -> (i32, i32) {
    if piece == PieceKind::O {
        return match orientation {
            Orientation::N => (0, 0),
            Orientation::E => (0, -1),
            Orientation::S => (-1, -1),
            Orientation::W => (-1, 0),
        };
    }

    let offsets = match (piece, orientation) {
        (PieceKind::I, Orientation::N) => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        (PieceKind::I, Orientation::E) => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        (PieceKind::I, Orientation::S) => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        (PieceKind::I, Orientation::W) => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
        (_, Orientation::N) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::E) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (_, Orientation::S) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::W) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    };

    offsets[n as usize]
}
