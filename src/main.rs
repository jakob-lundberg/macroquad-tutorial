use macroquad::prelude::*;

const MOVEMENT_SPEED: f32 = 200.0;

#[macroquad::main("My Game")]
async fn main() {
    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;
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
        draw_circle(x, y, 16.0, YELLOW);
        next_frame().await;
    }
}
