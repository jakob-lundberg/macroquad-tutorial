use macroquad::{prelude::*, rand::ChooseRandom};

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
}

#[macroquad::main("My Game")]
async fn main() {
    const MOVEMENT_SPEED: f32 = 200.0;
    let colors = vec![GREEN, RED, PURPLE, BLACK];

    rand::srand(miniquad::date::now() as u64);

    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;

    let mut squares = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_height() / 2.0,
        y: screen_width() / 2.0,
        color: YELLOW,
    };
    loop {
        let delta_time = get_frame_time();
        clear_background(BLUE);

        if is_key_down(KeyCode::Right) {
            x += MOVEMENT_SPEED * delta_time;
        }
        if is_key_down(KeyCode::Left) {
            x -= MOVEMENT_SPEED * delta_time;
        }
        if is_key_down(KeyCode::Down) {
            y += MOVEMENT_SPEED * delta_time;
        }
        if is_key_down(KeyCode::Up) {
            y -= MOVEMENT_SPEED * delta_time;
        }

        if rand::gen_range(0, 99) >= 95 {
            let size = rand::gen_range(16.0, 64.0);
            let color = colors.choose().unwrap();
            squares.push(Shape {
                size: size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                y: -size,
                color: *color,
            });
        }

        for square in &mut squares {
            square.y += square.speed * delta_time;
        }
        squares.retain(|square| square.y < screen_height() + square.size);

        x = clamp(x, 0.0, screen_width());
        y = clamp(y, 0.0, screen_height());

        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                square.color,
            );
        }
        draw_circle(x, y, 16.0, YELLOW);
        next_frame().await;
    }
}
