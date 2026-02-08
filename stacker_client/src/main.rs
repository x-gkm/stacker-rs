use std::time::Instant;

use macroquad::prelude::*;
use stacker_engine::{
    Action, Direction, Engine, GRID_HEIGHT, Input, Orientation, PILE_WIDTH, Piece,
};

const BLOCK_SIZE: f32 = 25.;

#[macroquad::main("stacker")]
async fn main() {
    let mut engine = Engine::new();
    let mut prev_time = Instant::now();

    loop {
        let time = Instant::now();
        let delta = time - prev_time;

        handle_input(&mut engine);

        engine.update(delta);

        clear_background(WHITE);

        let offset_x = (screen_width() - PILE_WIDTH as f32 * BLOCK_SIZE) / 2.;
        let offset_y = (screen_height() - GRID_HEIGHT as f32 * BLOCK_SIZE) / 2.;

        for (y, row) in engine.pile.iter().enumerate() {
            for (x, &block) in row.iter().enumerate() {
                let block_x = offset_x + x as f32 * BLOCK_SIZE;
                let block_y = offset_y + (GRID_HEIGHT - y as i32 - 1) as f32 * BLOCK_SIZE;

                if let Some(piece) = block {
                    draw_rectangle(block_x, block_y, BLOCK_SIZE, BLOCK_SIZE, piece_color(piece));
                } else if y < GRID_HEIGHT as usize {
                    draw_rectangle_lines(block_x, block_y, BLOCK_SIZE, BLOCK_SIZE, 1., GRAY);
                }
            }
        }

        for (index, piece) in engine.next_queue().enumerate() {
            for (x, y) in piece.blocks(Orientation::N) {
                let x = offset_x + (x + 12) as f32 * BLOCK_SIZE;
                let y =
                    offset_y + (GRID_HEIGHT - y - (4 - index) as i32 * 3 - 7) as f32 * BLOCK_SIZE;

                draw_rectangle(x, y, BLOCK_SIZE, BLOCK_SIZE, piece_color(piece));
            }
        }

        if let Some(ref active_piece) = engine.active_piece {
            for (x, y) in active_piece.ghost_blocks {
                let x = offset_x + x as f32 * BLOCK_SIZE;
                let y = offset_y + (GRID_HEIGHT - y - 1) as f32 * BLOCK_SIZE;

                draw_rectangle(
                    x,
                    y,
                    BLOCK_SIZE,
                    BLOCK_SIZE,
                    Color {
                        r: 0.,
                        g: 0.,
                        b: 0.,
                        a: 0.2,
                    },
                );
            }

            for (x, y) in active_piece.blocks {
                let x = offset_x + x as f32 * BLOCK_SIZE;
                let y = offset_y + (GRID_HEIGHT - y - 1) as f32 * BLOCK_SIZE;

                draw_rectangle(
                    x,
                    y,
                    BLOCK_SIZE,
                    BLOCK_SIZE,
                    piece_color(engine.active_piece.as_ref().unwrap().kind),
                );
            }
        }

        prev_time = time;
        next_frame().await;
    }
}

fn handle_input(engine: &mut Engine) {
    let mapping = [
        (KeyCode::A, Action::Hold),
        (KeyCode::S, Action::Flip),
        (KeyCode::D, Action::Rotate(Direction::Left)),
        (KeyCode::F, Action::Rotate(Direction::Right)),
        (KeyCode::Space, Action::Harddrop),
        (KeyCode::J, Action::Move(Direction::Left)),
        (KeyCode::K, Action::Softdrop),
        (KeyCode::L, Action::Move(Direction::Right)),
    ];

    for (key, action) in mapping {
        if is_key_pressed(key) {
            engine.process_input(Input::Begin(action));
        }

        if is_key_released(key) {
            engine.process_input(Input::End(action));
        }
    }
}

fn piece_color(piece: Piece) -> Color {
    match piece {
        Piece::I => SKYBLUE,
        Piece::O => YELLOW,
        Piece::T => PURPLE,
        Piece::L => ORANGE,
        Piece::Z => RED,
        Piece::J => BLUE,
        Piece::S => GREEN,
    }
}
