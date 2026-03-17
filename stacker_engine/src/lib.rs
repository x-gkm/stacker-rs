#![no_std]

use heapless::Deque;

use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};

pub const PILE_HEIGHT: usize = 40;
pub const PILE_WIDTH: usize = 10;
pub const GRID_HEIGHT: i32 = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub das: u32,
    pub arr: u32,
    pub are: u32,
    pub gravity: u32,
    pub softdrop: u32,
    pub clear_delay: u32,
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

type Coords = (i32, i32);

impl PieceKind {
    pub fn blocks(self, orientation: Orientation) -> [Coords; 4] {
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
    fn rotate_cw(&self, n: i32) -> Orientation {
        let mut result = *self;
        for _ in 0..n {
            use Orientation::*;
            result = match result {
                N => E,
                E => S,
                S => W,
                W => N,
            }
        }
        result
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

impl Direction {
    fn offset(&self) -> i32 {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
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
    lowest_y: i32,
    resets: i32,
    lock_timer: Timer,
    game_over: bool,
}

impl Engine {
    pub fn new(seed: u64, config: GameConfig) -> Engine {
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
            config,
            spawn_timer,
            fall_timer: Timer::new(),
            das_timer: Timer::new(),
            line_clear_timer: Timer::new(),
            lowest_y: 0,
            resets: 0,
            lock_timer: Timer::new(),
            game_over: false,
        }
    }

    fn rotate(&mut self, count: i32) {
        let Some(ref active_piece) = self.active_piece else {
            return;
        };

        let new_orientation = active_piece.orientation.rotate_cw(count);

        for n in 0..5 {
            let (kick_x, kick_y) = kick_offset(
                active_piece.kind,
                active_piece.orientation,
                new_orientation,
                n,
            );
            if let Some(piece) = self.can_move(kick_x, kick_y, count) {
                self.set_active(Some(piece));
                break;
            }
        }
    }

    fn lock_ghost(&mut self) {
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
        self.set_active(None);
        if line_clear {
            self.line_clear_timer.set(self.config.clear_delay);
            self.spawn_timer.set(self.config.clear_delay);
        } else {
            self.spawn_timer.set(self.config.are);
        }
    }

    fn can_move(&self, dx: i32, dy: i32, rotate_cw: i32) -> Option<Piece> {
        let Some(ref active_piece) = self.active_piece else {
            return None;
        };

        let branched = active_piece.changed_by(dx, dy, rotate_cw);
        if self.pile.check_collision(&branched.blocks) {
            return None;
        }

        Some(branched)
    }

    fn do_move(&mut self, direction: Direction) {
        let Some(piece) = self.can_move(direction.offset(), 0, 0) else {
            return;
        };

        self.set_active(Some(piece));
    }

    fn fall(&mut self) {
        let Some(piece) = self.can_move(0, -1, 0) else {
            return;
        };

        self.set_active(Some(piece));
    }

    fn try_lock(&mut self) {
        let Some(piece) = self.can_move(0, -1, 0) else {
            self.lock_ghost();
            return;
        };

        self.set_active(Some(piece));
    }

    fn set_fall_timer(&mut self) {
        self.fall_timer.set(if self.movement.soft_dropping {
            self.config.softdrop
        } else {
            self.config.gravity
        });
    }

    fn spawn(&mut self, kind: PieceKind) {
        // It is very important to set resets to zero *before* calling set_active.
        self.resets = 0;
        self.set_active(Some(Piece::spawn(kind)));
        if self
            .pile
            .check_collision(&self.active_piece.as_ref().unwrap().blocks)
        {
            self.game_over = true;
            return;
        }
        self.lowest_y = self.active_piece.as_ref().unwrap().lowest_y();
        self.fall();
        self.set_fall_timer();
    }

    fn set_active(&mut self, piece: Option<Piece>) {
        let Some(piece) = piece else {
            self.active_piece = None;
            self.ghost_piece = None;
            return;
        };

        let previous_piece = self.active_piece.clone();
        let previous_lowest_y = self.lowest_y;

        self.active_piece = Some(piece);

        for dy in 0.. {
            let Some(branched) = self.can_move(0, -dy, 0) else {
                break;
            };
            self.ghost_piece = Some(branched);
        }

        self.lowest_y = self
            .lowest_y
            .min(self.active_piece.as_ref().unwrap().lowest_y());

        if self.lowest_y < previous_lowest_y {
            self.resets = 0;
        }

        if self.active_piece != previous_piece {
            if self.can_move(0, -1, 0) == None {
                self.lock_timer.set(30);
            } else {
                self.lock_timer.stop();
            }
        }

        self.resets += 1;

        if self.resets > 15 {
            self.try_lock();
        }
    }

    pub fn update(&mut self, frame_inputs: &[Input]) {
        if self.game_over {
            return;
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
            let direction = self.movement.das.unwrap();
            self.do_move(direction);
            if self.config.arr > 0 {
                self.das_timer.set(self.config.arr);
            } else {
                self.das_timer.set(1);
                loop {
                    if let Some(piece) = self.can_move(direction.offset(), 0, 0) {
                        self.set_active(Some(piece));
                    } else {
                        break;
                    }
                }
            }
        }
        if self.lock_timer.tick() {
            self.try_lock();
        }

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
                    self.lock_ghost();
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

    pub fn game_over(&self) -> bool {
        self.game_over
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

    fn check_collision(&self, blocks: &[Coords]) -> bool {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub orientation: Orientation,
    pub x: i32,
    pub y: i32,
    pub blocks: [Coords; 4],
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

    fn changed_by(&self, dx: i32, dy: i32, rotate_cw: i32) -> Piece {
        let mut branched = self.clone();

        branched.x += dx;
        branched.y += dy;
        branched.orientation = branched.orientation.rotate_cw(rotate_cw);

        branched.update_blocks();

        branched
    }

    fn lowest_y(&self) -> i32 {
        self.blocks.map(|(_, y)| y).iter().copied().min().unwrap()
    }
}

fn kick_offset(piece: PieceKind, from: Orientation, to: Orientation, n: i32) -> Coords {
    let (x1, y1) = kick_offset_part(piece, from, n);
    let (x2, y2) = kick_offset_part(piece, to, n);

    (x1 - x2, y1 - y2)
}

fn kick_offset_part(piece: PieceKind, orientation: Orientation, n: i32) -> Coords {
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
