use std::{collections::VecDeque, time::Duration};

use macroquad::rand::ChooseRandom;

const PILE_HEIGHT: usize = 40;
pub const PILE_WIDTH: usize = 10;
pub const GRID_HEIGHT: i32 = 20;

struct GameConfig {
    das: u32,
    arr: u32,
    are: u32,
    gravity: u32,
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

#[derive(Debug, Clone)]
pub enum Input {
    Flip,
    Hold,
    Rotate(Direction),
    Harddrop,
    BeginMove(Direction),
    EndMove(Direction),
    BeginSoftdrop,
    EndSoftdrop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(PartialEq)]
enum GameEvent {
    Gravity,
    Softdrop,
    Rotate(i32),
    Harddrop,
    Move(Direction),
    Das,
    Spawn,
    ClearLines,
}

struct DasState {
    direction: Option<Direction>,
    move_left: bool,
    move_right: bool,
}

pub struct NextQueue {
    pieces: VecDeque<Piece>,
}

impl NextQueue {
    fn new() -> NextQueue {
        let mut result = NextQueue {
            pieces: VecDeque::new(),
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
        bag.shuffle();
        self.pieces.extend(bag);
    }
}

pub struct Engine {
    pub pile: [[Option<Piece>; PILE_WIDTH]; PILE_HEIGHT],
    pub active_piece: Option<ActivePiece>,
    das: DasState,
    next_queue: NextQueue,
    timer: Timer<GameEvent>,
    config: GameConfig,
}

impl Engine {
    pub fn new() -> Engine {
        let mut timer = Timer::new();

        timer.add(0, GameEvent::Spawn);

        let pile = [[None; PILE_WIDTH]; PILE_HEIGHT];

        Engine {
            pile,
            active_piece: None,
            das: DasState {
                direction: None,
                move_left: false,
                move_right: false,
            },
            next_queue: NextQueue::new(),
            timer,
            config: GameConfig {
                das: 100,
                arr: 15,
                are: 100,
                gravity: 1000,
                clear_delay: 100,
            },
        }
    }

    pub fn process_input(&mut self, input: Input) {
        use Direction::*;
        use Input::*;
        match input {
            Flip => {
                self.timer.add(0, GameEvent::Rotate(2));
            }
            Hold => {}
            Rotate(Left) => {
                self.timer.add(0, GameEvent::Rotate(3));
            }
            Rotate(Right) => {
                self.timer.add(0, GameEvent::Rotate(1));
            }
            Harddrop => {
                self.timer.add(0, GameEvent::Harddrop);
            }
            BeginMove(Left) => {
                self.das.move_left = true;
                self.timer.add(0, GameEvent::Move(Direction::Left));
                self.timer.remove(GameEvent::Das);
                self.timer.add(self.config.das, GameEvent::Das);
                self.das.direction = Some(Direction::Left);
            }
            BeginMove(Right) => {
                self.das.move_right = true;
                self.timer.add(0, GameEvent::Move(Direction::Right));
                self.timer.remove(GameEvent::Das);
                self.timer.add(self.config.das, GameEvent::Das);
                self.das.direction = Some(Direction::Right);
            }
            EndMove(Left) => {
                self.das.move_left = false;
                self.timer.remove(GameEvent::Das);
                if self.das.move_right {
                    self.das.direction = Some(Direction::Right);
                    self.timer.add(self.config.das, GameEvent::Das);
                } else {
                    self.das.direction = None;
                }
            }
            EndMove(Right) => {
                self.das.move_right = false;
                self.timer.remove(GameEvent::Das);
                if self.das.move_left {
                    self.das.direction = Some(Direction::Left);
                    self.timer.add(self.config.das, GameEvent::Das);
                } else {
                    self.das.direction = None;
                }
            }
            BeginSoftdrop => {
                self.timer.remove(GameEvent::Gravity);
                self.timer.add(0, GameEvent::Softdrop);
            }
            EndSoftdrop => {
                self.timer.remove(GameEvent::Softdrop);
                self.timer.add(self.config.gravity, GameEvent::Gravity);
            }
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.timer.update(delta.as_millis() as u32);

        while let Some(event) = self.timer.poll() {
            match event {
                GameEvent::Spawn => {
                    self.timer.add(self.config.gravity, GameEvent::Gravity);
                    self.active_piece = Some(ActivePiece::spawn(self.next_queue.pull()));
                    self.active_piece.as_mut().unwrap().update_ghost(&self.pile);
                }
                GameEvent::Gravity => {
                    let mut branched_piece = self.active_piece.clone().unwrap();
                    branched_piece.y -= 1;
                    branched_piece.update_blocks();
                    if !check_collision(&self.pile, &branched_piece.blocks) {
                        self.active_piece = Some(branched_piece);
                    }

                    self.timer.add(self.config.gravity, GameEvent::Gravity);
                }
                GameEvent::Softdrop => {
                    let Some(ref active_piece) = self.active_piece else {
                        continue;
                    };

                    let mut branched_piece = active_piece.clone();
                    branched_piece.y -= 1;
                    branched_piece.update_blocks();
                    if !check_collision(&self.pile, &branched_piece.blocks) {
                        self.active_piece = Some(branched_piece);
                    }

                    self.timer.add(5, GameEvent::Softdrop);
                }
                GameEvent::Das => {
                    self.timer
                        .add(0, GameEvent::Move(self.das.direction.unwrap()));
                    self.timer.add(self.config.arr, GameEvent::Das);
                }
                GameEvent::Rotate(n) => {
                    let Some(ref mut active_piece) = self.active_piece else {
                        continue;
                    };

                    let mut branched_piece = active_piece.clone();
                    branched_piece.orientation.rotate_cw(n);

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
                GameEvent::Harddrop => {
                    let Some(ref active_piece) = self.active_piece else {
                        continue;
                    };

                    for (x, y) in active_piece.ghost_blocks {
                        self.pile[y as usize][x as usize] = Some(active_piece.kind)
                    }
                    let line_clear = any_lines_to_clear(&self.pile);

                    self.timer.remove(GameEvent::Gravity);
                    self.active_piece = None;
                    if line_clear {
                        self.timer.add(self.config.clear_delay, GameEvent::ClearLines);
                        self.timer.add(self.config.clear_delay, GameEvent::Spawn);
                    } else {
                        self.timer.add(self.config.are, GameEvent::Spawn);
                    }
                }
                GameEvent::Move(direction) => {
                    let Some(ref mut active_piece) = self.active_piece else {
                        continue;
                    };

                    let mut branched_piece = active_piece.clone();
                    branched_piece.x += match direction {
                        Direction::Right => 1,
                        Direction::Left => -1,
                    };
                    branched_piece.update_blocks();
                    if !check_collision(&self.pile, &branched_piece.blocks) {
                        *active_piece = branched_piece;
                    }
                    active_piece.update_ghost(&self.pile);
                }
                GameEvent::ClearLines => {
                    line_clear(&mut self.pile);
                }
            }
        }
    }

    pub fn next_queue(&self) -> impl Iterator<Item = Piece> {
        self.next_queue.pieces.iter().take(5).copied()
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
