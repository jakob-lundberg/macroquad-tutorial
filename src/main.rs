use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use macroquad_particles::{self as particles, AtlasConfig, Emitter, EmitterConfig};
use std::fs;

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

enum Turning {
    Not,
    Left,
    Right,
}

struct Game {
    game_state: GameState,

    circle: Shape,
    squares: Vec<Shape>,
    bullets: Vec<Shape>,
    explosions: Vec<(Emitter, Vec2)>,

    last_fired: f64,
    turning: Turning,
    turning_time: f32,

    score: u32,
    highscore: u32,

    direction_modifier: f32,
    render_target: RenderTarget,
    material: Material,

    ship_texture: Texture2D,
    ship_sprite: AnimatedSprite,
    bullet_texture: Texture2D,
    bullet_sprite: AnimatedSprite,
    enemy_small_texture: Texture2D,
    enemy_small_sprite: AnimatedSprite,
    enemy_medium_texture: Texture2D,
    enemy_medium_sprite: AnimatedSprite,
    enemy_large_texture: Texture2D,
    enemy_large_sprite: AnimatedSprite,
}

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
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
    const SCOLL_SPEED: f32 = 0.05;
    const LARGE_TURNING_TIME: f32 = 0.2;
    const FIRE_INTERVAL: f64 = 0.3;

    set_pc_assets_folder("assets");

    rand::srand(miniquad::date::now() as u64);

    let ship_texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load ship");
    ship_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D = load_texture("laser-bolts.png")
        .await
        .expect("Couldn't load bullet");
    bullet_texture.set_filter(FilterMode::Nearest);

    let explosion_texture: Texture2D = load_texture("explosion.png")
        .await
        .expect("Couldn't load explosion");
    explosion_texture.set_filter(FilterMode::Nearest);

    let enemy_small_texture: Texture2D = load_texture("enemy-small.png")
        .await
        .expect("Couldn't load small enemy");
    enemy_small_texture.set_filter(FilterMode::Nearest);

    let enemy_medium_texture: Texture2D = load_texture("enemy-medium.png")
        .await
        .expect("Couldn't load medium enemy");
    enemy_medium_texture.set_filter(FilterMode::Nearest);

    let enemy_large_texture: Texture2D = load_texture("enemy-big.png")
        .await
        .expect("Couldn't load large enemy");
    enemy_large_texture.set_filter(FilterMode::Nearest);

    build_textures_atlas();

    let enemy_small_sprite = AnimatedSprite::new(
        17,
        16,
        &[Animation {
            name: "enemy_small".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let enemy_medium_sprite = AnimatedSprite::new(
        32,
        16,
        &[Animation {
            name: "enemy_medium".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let enemy_large_sprite = AnimatedSprite::new(
        32,
        32,
        &[Animation {
            name: "enemy_large".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "bolt".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    bullet_sprite.set_animation(1);
    let ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left_little".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right_little".to_string(),
                row: 3,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );

    let mut game = Game {
        game_state: GameState::MainMenu,

        circle: Shape {
            size: 32.0,
            speed: MOVEMENT_SPEED,
            x: screen_height() / 2.0,
            y: screen_width() / 2.0,
            collided: false,
        },
        squares: vec![],
        bullets: vec![],
        explosions: vec![],
        last_fired: 0.0,
        turning: Turning::Not,
        turning_time: 0.0,
        score: 0,
        highscore: fs::read_to_string("highscore.dat")
            .map_or(Ok(0), |i| i.parse::<u32>())
            .unwrap_or(0),
        direction_modifier: 0.0,
        render_target: render_target(320, 150),
        material: load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("direction_modifier", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .unwrap(),
        ship_sprite: ship_sprite,
        ship_texture: ship_texture,
        bullet_sprite: bullet_sprite,
        bullet_texture: bullet_texture,
        enemy_small_texture: enemy_small_texture,
        enemy_small_sprite: enemy_small_sprite,
        enemy_medium_texture: enemy_medium_texture,
        enemy_medium_sprite: enemy_medium_sprite,
        enemy_large_texture: enemy_large_texture,
        enemy_large_sprite: enemy_large_sprite,
    };
    game.render_target.texture.set_filter(FilterMode::Nearest);

    loop {
        let delta_time = get_frame_time();
        draw_background(&game);
        match game.game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    game.squares.clear();
                    game.bullets.clear();
                    game.explosions.clear();
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
                game.ship_sprite.set_animation(0);
                if is_key_down(KeyCode::F) {
                    game.circle.x += MOVEMENT_SPEED * delta_time;
                    game.direction_modifier += SCOLL_SPEED * delta_time;
                    if let Turning::Right = game.turning {
                        game.turning_time += delta_time;
                    } else {
                        game.turning_time = 0.0;
                    }
                    game.turning = Turning::Right;
                    if game.turning_time > LARGE_TURNING_TIME {
                        game.ship_sprite.set_animation(4);
                    } else {
                        game.ship_sprite.set_animation(3);
                    }
                } else if is_key_down(KeyCode::S) {
                    game.circle.x -= MOVEMENT_SPEED * delta_time;
                    game.direction_modifier -= SCOLL_SPEED * delta_time;
                    if let Turning::Left = game.turning {
                        game.turning_time += delta_time;
                    } else {
                        game.turning_time = 0.0;
                    }
                    game.turning = Turning::Left;
                    if game.turning_time > LARGE_TURNING_TIME {
                        game.ship_sprite.set_animation(2);
                    } else {
                        game.ship_sprite.set_animation(1);
                    }
                } else {
                    game.turning = Turning::Not;
                    game.turning_time = 0.0;
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

                if is_key_down(KeyCode::Space) && get_time() - game.last_fired > FIRE_INTERVAL {
                    game.bullets.push(Shape {
                        size: 32.0,
                        speed: game.circle.speed * 3.0,
                        x: game.circle.x,
                        y: game.circle.y - 24.0,
                        collided: false,
                    });

                    game.last_fired = get_time();
                }

                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    game.squares.push(Shape {
                        size: size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        collided: false,
                    });
                }

                for square in &mut game.squares {
                    square.y += square.speed * delta_time;
                }

                for bullet in &mut game.bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                game.ship_sprite.update();
                game.bullet_sprite.update();
                game.enemy_small_sprite.update();
                game.enemy_medium_sprite.update();
                game.enemy_large_sprite.update();

                game.squares
                    .retain(|square| square.y < screen_height() + square.size);
                game.bullets
                    .retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

                game.squares.retain(|square| !square.collided);
                game.bullets.retain(|bullet| !bullet.collided);
                game.explosions
                    .retain(|(explosion, _)| explosion.config.emitting);

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
                            game.explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: (square.size.round() as u32 * 1).max(30),
                                    texture: Some(explosion_texture.clone()),
                                    ..particle_explosion(square.size)
                                }),
                                vec2(square.x, square.y),
                            ));
                        }
                    }
                }
                draw_playing_field(&mut game);
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game.game_state = GameState::Playing;
                }

                draw_playing_field(&mut game);
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

fn draw_background(game: &Game) -> () {
    clear_background(BROWN);
    game.material
        .set_uniform("iResolution", (screen_width(), screen_height()));
    game.material
        .set_uniform("direction_modifier", game.direction_modifier);
    gl_use_material(&game.material);
    draw_texture_ex(
        &game.render_target.texture,
        0.,
        0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        },
    );
    gl_use_default_material();
}

fn draw_playing_field(game: &mut Game) -> () {
    let enemy_small_frame = game.enemy_small_sprite.frame();
    let enemy_medium_frame = game.enemy_medium_sprite.frame();
    let enemy_large_frame = game.enemy_large_sprite.frame();
    for square in &game.squares {
        let texture = match square.size {
            x if (..30.0).contains(&x) => &game.enemy_small_texture,
            x if (30.0..50.0).contains(&x) => &game.enemy_medium_texture,
            x if (50.0..).contains(&x) => &game.enemy_large_texture,
            _ => &game.enemy_small_texture,
        };
        let source = match square.size {
            x if (..30.0).contains(&x) => &enemy_small_frame,
            x if (30.0..50.0).contains(&x) => &enemy_medium_frame,
            x if (50.0..).contains(&x) => &enemy_large_frame,
            _ => &enemy_small_frame,
        };
        draw_texture_ex(
            texture,
            square.x - square.size / 2.0,
            square.y - square.size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: square.size,
                    y: square.size,
                }),
                source: Some(source.source_rect),
                ..Default::default()
            },
        );
    }
    let bullet_frame = game.bullet_sprite.frame();
    for bullet in &game.bullets {
        draw_texture_ex(
            &game.bullet_texture,
            bullet.x - bullet.size / 2.0,
            bullet.y - bullet.size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: bullet.size,
                    y: bullet.size,
                }),
                source: Some(bullet_frame.source_rect),
                ..Default::default()
            },
        );
    }

    for (explosion, coords) in game.explosions.iter_mut() {
        explosion.draw(*coords);
    }

    let ship_frame = game.ship_sprite.frame();
    draw_texture_ex(
        &game.ship_texture,
        game.circle.x - ship_frame.dest_size.x,
        game.circle.y - ship_frame.dest_size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(ship_frame.dest_size * 2.0),
            source: Some(ship_frame.source_rect),
            ..Default::default()
        },
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

fn particle_explosion(size: f32) -> particles::EmitterConfig {
    particles::EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.8 * size / 100.0,
        lifetime_randomness: 0.3,
        explosiveness: 0.65 * size / 100.0,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 400.0,
        initial_velocity_randomness: 0.8,
        size: 32.0 * size / 100.0,
        size_randomness: 0.3,
        atlas: Some(AtlasConfig::new(5, 1, 0..)),
        ..Default::default()
    }
}
