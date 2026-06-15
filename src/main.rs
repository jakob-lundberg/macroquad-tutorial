use macroquad::{prelude::*, rand::ChooseRandom};
use std::fs;

enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

struct Game {
    game_state: GameState,

    circle: Shape,
    squares: Vec<Shape>,
    bullets: Vec<Shape>,
    last_fired: f64,
    score: u32,
    highscore: u32,
}

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

    let mut game = Game {
        game_state: GameState::MainMenu,

        circle: Shape {
            size: 32.0,
            speed: MOVEMENT_SPEED,
            x: screen_height() / 2.0,
            y: screen_width() / 2.0,
            color: YELLOW,
            collided: false,
        },
        squares: vec![],
        bullets: vec![],
        last_fired: 0.0,
        score: 0,
        highscore: fs::read_to_string("highscore.dat")
            .map_or(Ok(0), |i| i.parse::<u32>())
            .unwrap_or(0),
    };

    loop {
        let delta_time = get_frame_time();
        clear_background(BROWN);
        match game.game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    game.squares.clear();
                    game.bullets.clear();
                    game.circle.x = screen_width() / 2.0;
                    game.circle.y = screen_height() / 2.0;
                    game.score = 0;
                    game.game_state = GameState::Playing;
                }

                let text = "Space Blasters";
                let text_dimensions = measure_text(text, None, 100, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 4.0,
                    100.0,
                    WHITE,
                );

                let text = "Press Space to start";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            GameState::Playing => {
                if is_key_down(KeyCode::F) {
                    game.circle.x += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::S) {
                    game.circle.x -= MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::D) {
                    game.circle.y += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::E) {
                    game.circle.y -= MOVEMENT_SPEED * delta_time;
                }

                if is_key_pressed(KeyCode::P) {
                    game.game_state = GameState::Paused;
                }

                game.circle.x = clamp(game.circle.x, 0.0, screen_width());
                game.circle.y = clamp(game.circle.y, 0.0, screen_height());

                if is_key_down(KeyCode::Space) && get_time() - game.last_fired > 1.0 {
                    game.bullets.push(Shape {
                        size: 5.0,
                        speed: game.circle.speed * 2.0,
                        x: game.circle.x,
                        y: game.circle.y,
                        color: BLACK,
                        collided: false,
                    });

                    game.last_fired = get_time();
                }

                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    let color = colors.choose().unwrap();
                    game.squares.push(Shape {
                        size: size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        color: *color,
                        collided: false,
                    });
                }

                for square in &mut game.squares {
                    square.y += square.speed * delta_time;
                }

                for bullet in &mut game.bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                game.squares
                    .retain(|square| square.y < screen_height() + square.size);
                game.bullets
                    .retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

                game.squares.retain(|square| !square.collided);
                game.bullets.retain(|bullet| !bullet.collided);

                if game
                    .squares
                    .iter()
                    .any(|square| game.circle.collides_with(square))
                {
                    if game.score == game.highscore {
                        fs::write("highscore.dat", game.highscore.to_string()).ok();
                    }
                    game.game_state = GameState::GameOver;
                }
                for square in &mut game.squares {
                    for bullet in &mut game.bullets {
                        if bullet.collides_with(square) {
                            bullet.collided = true;
                            square.collided = true;
                            game.score += square.size.round() as u32;
                            game.highscore = game.highscore.max(game.score);
                        }
                    }
                }
                draw_playing_field(&game);
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game.game_state = GameState::Playing;
                }

                draw_playing_field(&game);
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            GameState::GameOver => {
                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    RED,
                );

                if is_key_pressed(KeyCode::Space) {
                    game.squares.clear();
                    game.bullets.clear();
                    game.circle.x = screen_width() / 2.0;
                    game.circle.y = screen_height() / 2.0;
                    game.game_state = GameState::MainMenu;
                    game.score = 0;
                }
            }
        };

        next_frame().await;
    }
}

fn draw_playing_field(game: &Game) -> () {
    for square in &game.squares {
        draw_rectangle(
            square.x - square.size / 2.0,
            square.y - square.size / 2.0,
            square.size,
            square.size,
            square.color,
        );
    }
    for bullet in &game.bullets {
        draw_circle_lines(bullet.x, bullet.y, bullet.size / 2.0, 1.0, bullet.color);
    }
    draw_circle(
        game.circle.x,
        game.circle.y,
        game.circle.size / 2.0,
        game.circle.color,
    );

    draw_text(format!("Score: {}", game.score), 10.0, 35.0, 25.0, WHITE);
    let highscore_text = format!("High score: {}", game.highscore);
    let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
    draw_text(
        highscore_text.as_str(),
        screen_width() - text_dimensions.width - 10.0,
        35.0,
        25.0,
        WHITE,
    );
}
