mod assets;
mod game;

use assets::Assets;
use game::Game;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "IronJump - Rust starter".to_owned(),
        window_width: 480,
        window_height: 320,
        window_resizable: true,
        high_dpi: true,
        platform: macroquad::miniquad::conf::Platform {
            linux_backend: macroquad::miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Textures must be created after Macroquad initializes the graphics context.
    let assets = Assets::new();
    let mut game = Game::new();
    game.load(&assets.level);

    const STEP: f64 = 1.0 / 60.0;
    let mut accumulator = 0.0_f64;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Limit catch-up after a pause or a long stall.
        accumulator += f64::from(get_frame_time()).min(0.25);

        while accumulator >= STEP {
            game.update();
            accumulator -= STEP;
        }

        clear_background(Color::from_rgba(22, 27, 37, 255));
        game.draw(&assets);

        // Present the frame and let the window/input event loop run.
        next_frame().await;
    }
}
