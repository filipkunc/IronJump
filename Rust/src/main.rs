mod assets;
mod game;
mod levels;
mod session;

use assets::Assets;
use game::Game;
use levels::Level;
use macroquad::prelude::*;
use session::{Mode, Progress, Session};
use std::path::{Path, PathBuf};

fn window_conf() -> Conf {
    Conf {
        window_title: "IronJump".to_owned(),
        window_width: 960,
        window_height: 640,
        window_resizable: true,
        high_dpi: true,
        platform: macroquad::miniquad::conf::Platform {
            linux_backend: macroquad::miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn custom_level(path: &Path) -> Result<Level, String> {
    let xml =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Game::new()
        .load(&xml)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Level {
        name: path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        xml,
        custom: true,
    })
}

fn options() -> Result<(Vec<Level>, bool, PathBuf), String> {
    let mut additional = Vec::new();
    let mut start = false;
    let mut progress = PathBuf::from("ironjump-progress.txt");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--level" => {
                let path = args.next().ok_or("--level requires an .xlevel path")?;
                additional.push(custom_level(Path::new(&path))?);
                start = true;
            }
            "--levels-dir" => {
                let path = args.next().ok_or("--levels-dir requires a directory")?;
                let mut files = std::fs::read_dir(path)
                    .map_err(|error| error.to_string())?
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| error.to_string())?;
                files.sort();
                for file in files {
                    if file
                        .extension()
                        .is_some_and(|extension| extension == "xlevel")
                    {
                        additional.push(custom_level(&file)?);
                    }
                }
            }
            "--progress" => {
                progress = PathBuf::from(args.next().ok_or("--progress requires a file path")?)
            }
            _ => {
                return Err(format!(
                    "Unknown argument {arg}. Use --level FILE, --levels-dir DIR, or --progress FILE."
                ));
            }
        }
    }
    Ok((additional, start, progress))
}

fn viewport() -> Rect {
    let scale = (screen_width() / 480.0)
        .min(screen_height() / 320.0)
        .max(0.001);
    let size = vec2(480.0, 320.0) * scale;
    Rect::new(
        (screen_width() - size.x) * 0.5,
        (screen_height() - size.y) * 0.5,
        size.x,
        size.y,
    )
}

fn game_input(view: Rect) -> Vec2 {
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    let jump = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space);
    let mut input = vec2(
        if left {
            -1.0
        } else if right {
            1.0
        } else {
            0.0
        },
        f32::from(jump),
    );
    // Touch controls replace the iPhone accelerometer on touch-capable desktops.
    for touch in touches() {
        if touch.phase == TouchPhase::Ended
            || touch.phase == TouchPhase::Cancelled
            || !view.contains(touch.position)
        {
            continue;
        }
        let x = (touch.position.x - view.x) * 480.0 / view.w;
        if x > 320.0 {
            input.y = 1.0;
        } else {
            input.x = if x < 160.0 { -1.0 } else { 1.0 };
        }
    }
    input
}

#[macroquad::main(window_conf)]
async fn main() {
    let (additional, start_custom, progress_path) = match options() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };
    let progress = Progress::load(&progress_path).unwrap_or_else(|error| {
        eprintln!("Could not load progress: {error}");
        Progress::default()
    });
    let assets = Assets::new();
    let mut session = Session::new(progress, additional);
    if start_custom {
        session.select(session.levels.len() - 1);
        session.start();
    }

    let target = render_target(480, 320);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 480.0, 320.0));
    camera.render_target = Some(target.clone());
    const STEP: f64 = 1.0 / 60.0;
    let mut accumulator = 0.0_f64;

    loop {
        let view = viewport();
        if is_key_pressed(KeyCode::Q) {
            break;
        }
        if is_key_pressed(KeyCode::Escape) {
            if session.mode == Mode::Menu {
                break;
            }
            session.menu();
            accumulator = 0.0;
        }
        if session.mode == Mode::Menu {
            if is_key_pressed(KeyCode::Left) {
                session.change_page(-1);
            }
            if is_key_pressed(KeyCode::Right) {
                session.change_page(1);
            }
            if is_key_pressed(KeyCode::Up) {
                session.select(session.progress.selected.saturating_sub(1));
            }
            if is_key_pressed(KeyCode::Down) {
                session.select(session.progress.selected + 1);
            }
            if is_key_pressed(KeyCode::Enter) {
                session.start();
            }
        } else if session.mode == Mode::Victory {
            if is_key_pressed(KeyCode::Enter) {
                session.menu();
            }
        } else if is_key_pressed(KeyCode::R) {
            session.start();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let point = Vec2::from(mouse_position());
            if view.contains(point) {
                session.click((point - view.point()) * vec2(480.0 / view.w, 320.0 / view.h));
            }
        }
        let input = game_input(view);
        accumulator += f64::from(get_frame_time()).min(0.25);
        while accumulator >= STEP {
            session.update(input);
            accumulator -= STEP;
        }
        if session.dirty {
            if let Err(error) = session.progress.save(&progress_path) {
                eprintln!("Could not save progress: {error}");
            }
            session.dirty = false;
        }

        set_camera(&camera);
        clear_background(Color::from_rgba(101, 97, 85, 255));
        session.draw(&assets);
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &target.texture,
            view.x,
            view.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(view.size()),
                flip_y: true,
                ..Default::default()
            },
        );
        next_frame().await;
    }
}
