//! Desktop runner. OS input, filesystem access, and Macroquad stay outside the core.
mod input;
mod options;
mod storage;

use crate::backend::MacroquadRenderer;
use ironjump::{
    app::App,
    session::{Mode, Session},
    viewport::Viewport,
};
use macroquad::prelude as mq;
use storage::FileProgressStore;

pub fn window_conf() -> mq::Conf {
    mq::Conf {
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

pub async fn run() {
    let (additional, start_custom, progress_path) = match options::options() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };
    let storage = FileProgressStore::new(progress_path);
    let progress = storage.load().unwrap_or_else(|error| {
        eprintln!("Could not load progress: {error}");
        Default::default()
    });
    let mut session = Session::new(progress, additional);
    if start_custom {
        session.select(session.levels.len() - 1);
        session.start();
    }
    let mut app = App::new(session);
    let mut renderer = MacroquadRenderer::new();
    loop {
        let window_size = mq::vec2(mq::screen_width(), mq::screen_height());
        let game_view = Viewport::new(window_size);
        app.session.resize(game_view.logical_size);
        let view = if app.session.mode == Mode::Menu {
            Viewport::menu(window_size)
        } else {
            game_view
        };
        if !app.frame(&input::poll(view), f64::from(mq::get_frame_time())) {
            break;
        }
        if app.session.dirty {
            if let Err(error) = storage.save(app.session.progress) {
                eprintln!("Could not save progress: {error}");
            }
            app.session.dirty = false;
        }
        // Actions may have switched between the fitted menu and full-screen game.
        let view = if app.session.mode == Mode::Menu {
            Viewport::menu(window_size)
        } else {
            game_view
        };
        renderer.begin_frame(view);
        app.session.draw(&mut renderer, input::RETURN_TO_MENU_HINT);
        renderer.present(view);
        if app.session.mode == Mode::Menu {
            renderer.controls(view, input::CONTROL_HINT);
        }
        mq::next_frame().await;
    }
}
