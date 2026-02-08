use std::time::Instant;

use macroquad::prelude::*;
use stacker_engine::{Action, Direction, Engine, GRID_HEIGHT, Input, Orientation, PILE_WIDTH, Piece};

const BLOCK_SIZE: f32 = 25.;

#[macroquad::main("stacker")]
async fn main() {
    let mut engine = Engine::new();
    let mut prev_time = Instant::now();

    loop {
        let time = Instant::now();
        let delta = time - prev_time;

        if is_key_pressed(KeyCode::A) {
            engine.process_input(Input::Begin(Action::Hold));
        }

        if is_key_released(KeyCode::A) {
            engine.process_input(Input::End(Action::Hold));
        }

        if is_key_pressed(KeyCode::S) {
            engine.process_input(Input::Begin(Action::Flip));
        }

        if is_key_released(KeyCode::S) {
            engine.process_input(Input::End(Action::Flip));
        }

        if is_key_pressed(KeyCode::D) {
            engine.process_input(Input::Begin(Action::Rotate(Direction::Left)));
        }

        if is_key_released(KeyCode::D) {
            engine.process_input(Input::End(Action::Rotate(Direction::Left)));
        }

        if is_key_pressed(KeyCode::F) {
            engine.process_input(Input::Begin(Action::Rotate(Direction::Right)));
        }

        if is_key_released(KeyCode::F) {
            engine.process_input(Input::End(Action::Rotate(Direction::Right)));
        }

        if is_key_pressed(KeyCode::Space) {
            engine.process_input(Input::Begin(Action::Harddrop));
        }

        if is_key_released(KeyCode::Space) {
            engine.process_input(Input::End(Action::Harddrop));
        }

        if is_key_pressed(KeyCode::J) {
            engine.process_input(Input::Begin(Action::Move(Direction::Left)));
        }

        if is_key_released(KeyCode::J) {
            engine.process_input(Input::End(Action::Move(Direction::Left)));
        }

        if is_key_pressed(KeyCode::K) {
            engine.process_input(Input::Begin(Action::Softdrop));
        }

        if is_key_released(KeyCode::K) {
            engine.process_input(Input::End(Action::Softdrop));
        }

        if is_key_pressed(KeyCode::L) {
            engine.process_input(Input::Begin(Action::Move(Direction::Right)));
        }

        if is_key_released(KeyCode::L) {
            engine.process_input(Input::End(Action::Move(Direction::Right)));
        }

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
