//! Opt-in device benchmark; never changes saved progress or uses sensor input.
use crate::backend::MacroquadRenderer;
use ironjump::{game::Game, input::GameInput, levels, viewport::Viewport};
use macroquad::{prelude::*, telemetry};
use std::time::Instant;

pub async fn run() {
    let mut renderer = MacroquadRenderer::new();
    for level in levels::bundled().into_iter().filter(|level| {
        ["tutorial", "trampoline", "speed4", "labyrinth"].contains(&level.name.as_str())
    }) {
        let view = Viewport::new(vec2(screen_width(), screen_height()));
        let mut game = Game::with_viewport(view.logical_size);
        game.load(&level.xml).unwrap();
        for _ in 0..60 {
            game.update(GameInput::NONE);
        }
        let mut intervals = Vec::new();
        let mut submit_times = Vec::new();
        for frame in 0..180 {
            // Capture one frame's actual GPU batches. Keep capture overhead out
            // of the timing samples (telemetry draws a thumbnail for each batch).
            if frame == 20 {
                telemetry::capture_frame();
            }
            if frame == 23 {
                let calls = telemetry::drawcalls();
                macroquad::logging::info!(
                    "IronJumpBench {} draws={} quads={}",
                    level.name,
                    calls.len(),
                    calls
                        .iter()
                        .map(|call| call.indices_count / 6)
                        .sum::<usize>()
                );
            }
            let start = Instant::now();
            renderer.begin_frame(view);
            game.draw(&mut renderer, level.custom);
            renderer.present(view);
            if frame >= 40 {
                intervals.push(get_frame_time() * 1000.0);
                submit_times.push(start.elapsed().as_secs_f32() * 1000.0);
            }
            next_frame().await;
        }
        intervals.sort_by(f32::total_cmp);
        submit_times.sort_by(f32::total_cmp);
        macroquad::logging::info!(
            "IronJumpBench {} frame_ms median={:.2} p95={:.2} submit_ms median={:.3} p95={:.3}",
            level.name,
            intervals[70],
            intervals[133],
            submit_times[70],
            submit_times[133]
        );
    }
    macroquad::logging::info!("IronJumpBench complete");
}
