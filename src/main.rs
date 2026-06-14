use macroquad::{prelude::*, rand::ChooseRandom};

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
    collided: bool,
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.circle().overlaps_rect(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
    fn circle(&self) -> Circle {
        Circle {
            x: self.x,
            y: self.y,
            r: self.size / 2.0,
        }
    }
}

#[macroquad::main("My Game")]
async fn main() {
    const MOVEMENT_SPEED: f32 = 200.0;
    let colors = vec![GREEN, PINK, GRAY, PURPLE, BLACK];

    rand::srand(miniquad::date::now() as u64);

    let mut game_over = false;
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];

    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_height() / 2.0,
        y: screen_width() / 2.0,
        color: YELLOW,
        collided: false,
    };
    loop {
        let delta_time = get_frame_time();
        clear_background(BROWN);

        if !game_over {
            if is_key_down(KeyCode::F) {
                circle.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::S) {
                circle.x -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::D) {
                circle.y += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::E) {
                circle.y -= MOVEMENT_SPEED * delta_time;
            }

            circle.x = clamp(circle.x, 0.0, screen_width());
            circle.y = clamp(circle.y, 0.0, screen_height());

            if is_key_pressed(KeyCode::Space) {
                bullets.push(Shape {
                    size: 5.0,
                    speed: circle.speed * 2.0,
                    x: circle.x,
                    y: circle.y,
                    color: BLACK,
                    collided: false,
                });
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
                    collided: false,
                });
            }

            for square in &mut squares {
                square.y += square.speed * delta_time;
            }

            for bullet in &mut bullets {
                bullet.y -= bullet.speed * delta_time;
            }

            squares.retain(|square| square.y < screen_height() + square.size);
            bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

            squares.retain(|square| !square.collided);
            bullets.retain(|bullet| !bullet.collided);

            if squares.iter().any(|square| circle.collides_with(square)) {
                game_over = true;
            }
            for square in &mut squares {
                for bullet in &mut bullets {
                    if bullet.collides_with(square) {
                        bullet.collided = true;
                        square.collided = true;
                    }
                }
            }
        }

        if game_over && is_key_pressed(KeyCode::Space) {
            squares.clear();
            bullets.clear();
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            game_over = false;
        }

        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                square.color,
            );
        }
        for bullet in &bullets {
            draw_circle(bullet.x, bullet.y, bullet.size / 2.0, bullet.color);
        }
        draw_circle(circle.x, circle.y, circle.size / 2.0, circle.color);

        if game_over {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(
                text,
                screen_width() / 2.0 - text_dimensions.width / 2.0,
                screen_height() / 2.0,
                50.0,
                RED,
            );
        }

        next_frame().await;
    }
}
