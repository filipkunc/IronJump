//! Platform runner. OS input, filesystem access, and Macroquad stay outside the core.
#[cfg(target_os = "android")]
mod android;
#[cfg(feature = "render-benchmark")]
mod benchmark;
mod input;
#[cfg(any(target_os = "android", test))]
mod motion;
#[cfg(not(target_os = "android"))]
mod options;
#[cfg(feature = "frame-profile")]
mod profile;
mod storage;
#[cfg(feature = "frame-profile")]
mod trace;

use crate::backend::MacroquadRenderer;
use ironjump::{
    app::App,
    session::{Mode, Session},
    viewport::Viewport,
};
use macroquad::prelude as mq;
use storage::FileProgressStore;

pub fn window_conf() -> macroquad::conf::Conf {
    let miniquad_conf = mq::Conf {
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
    };
    macroquad::conf::Conf {
        miniquad_conf,
        // Match vertex and index budgets: four vertices and six indices per quad.
        draw_call_vertex_capacity: 10000,
        draw_call_index_capacity: 15000,
        ..Default::default()
    }
}

pub async fn run() {
    #[cfg(feature = "render-benchmark")]
    benchmark::run().await;
    #[cfg(not(target_os = "android"))]
    let options = options::options();
    #[cfg(target_os = "android")]
    let options = android::progress_path().map(|path| (Vec::new(), false, path));
    let (additional, start_custom, progress_path) = match options {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };
    #[cfg(all(feature = "frame-profile", not(target_os = "android")))]
    let trace_path = progress_path.with_file_name("ironjump-trace.txt");
    #[cfg(all(feature = "frame-profile", target_os = "android"))]
    let trace_path = android::trace_path().expect("Android trace directory");
    let storage = FileProgressStore::new(progress_path);
    let progress = storage.load().unwrap_or_else(|error| {
        eprintln!("Could not load progress: {error}");
        Default::default()
    });
    #[cfg(feature = "frame-profile")]
    let mut trace = if additional.is_empty() && !start_custom {
        trace::Trace::new(&trace_path, progress)
            .map_err(|error| eprintln!("Could not record trace: {error}"))
            .ok()
    } else {
        None
    };
    let mut session = Session::new(progress, additional);
    if start_custom {
        session.select(session.levels.len() - 1);
        session.start();
    }
    let mut app = App::new(session);
    let mut renderer = MacroquadRenderer::new();
    let mut previous_frame = std::time::Instant::now();
    #[cfg(feature = "frame-profile")]
    let mut profile = profile::Profile::default();
    loop {
        let now = std::time::Instant::now();
        #[allow(unused_mut)]
        let mut elapsed = now.duration_since(previous_frame).as_secs_f64();
        previous_frame = now;
        #[allow(unused_mut, unused_variables)]
        let mut reset_clock = false;
        #[cfg(target_os = "android")]
        {
            if !android::resumed() {
                #[cfg(feature = "frame-profile")]
                if let Some(trace) = &mut trace {
                    trace.flush();
                }
                mq::next_frame().await;
                continue;
            }
            if android::take_clock_reset() {
                app.reset_clock();
                elapsed = 0.0;
                reset_clock = true;
            }
        }
        let window_size = mq::vec2(mq::screen_width(), mq::screen_height());
        let game_view = Viewport::new(window_size);
        app.session.resize(game_view.logical_size);
        let view = if app.session.mode == Mode::Menu {
            Viewport::menu(window_size)
        } else {
            game_view
        };
        #[cfg(feature = "frame-profile")]
        let update_start = std::time::Instant::now();
        let frame = ironjump::replay::Frame {
            elapsed,
            viewport: game_view.logical_size,
            reset_clock,
            input: input::poll(view),
        };
        if !app.frame(&frame.input, elapsed) {
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
        #[cfg(feature = "frame-profile")]
        let draw_start = std::time::Instant::now();
        renderer.begin_frame(view);
        app.session.draw_interpolated(
            &mut renderer,
            input::RETURN_TO_MENU_HINT,
            app.interpolation_alpha(),
        );
        renderer.present(view);
        if app.session.mode == Mode::Menu {
            renderer.controls(view, input::CONTROL_HINT);
        }
        #[cfg(feature = "frame-profile")]
        profile.record(elapsed, update_start, draw_start, app.ticks_last_frame);
        #[cfg(feature = "frame-profile")]
        if let Some(trace) = &mut trace {
            trace.record(&frame, app.ticks_last_frame, update_start, draw_start);
        }
        mq::next_frame().await;
    }
}
