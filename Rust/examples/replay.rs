//! Headless reproduction using recorded elapsed times and semantic inputs.
use ironjump::{
    app::App,
    progress::Progress,
    replay::Frame,
    session::{Mode, Session},
};
use std::collections::BTreeMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("Usage: replay TRACE")?;
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some("IronJumpTrace 1") {
        return Err("Unknown trace version".into());
    }
    let progress = Progress::parse(lines.next().ok_or("Missing initial progress")?)?;
    let mut app = App::new(Session::new(progress, Vec::new()));
    let mut frames = 0;
    let mut ticks = 0;
    let mut levels: BTreeMap<String, Vec<[f64; 4]>> = BTreeMap::new();
    let start = std::time::Instant::now();
    for line in lines {
        let frame = Frame::parse(line)?;
        if frame.reset_clock {
            app.reset_clock();
        }
        app.session.resize(frame.viewport);
        let running = app.frame(&frame.input, frame.elapsed);
        if let Some(timing) = line.split('|').nth(1) {
            let mut timing = timing.split_whitespace();
            let recorded: usize = timing.next().ok_or("Missing ticks")?.parse()?;
            if recorded != app.ticks_last_frame {
                return Err(format!("Tick mismatch at frame {frames}").into());
            }
            if app.session.mode == Mode::Playing {
                let update: f64 = timing.next().ok_or("Missing update timing")?.parse()?;
                let draw: f64 = timing.next().ok_or("Missing draw timing")?.parse()?;
                let level = app.session.levels[app.session.progress.selected]
                    .name
                    .clone();
                levels.entry(level).or_default().push([
                    frame.elapsed * 1000.0,
                    recorded as f64,
                    update,
                    draw,
                ]);
            }
        }
        frames += 1;
        ticks += app.ticks_last_frame;
        if !running {
            break;
        }
    }
    println!(
        "Replayed {frames} frames / {ticks} ticks in {:?}; mode={:?}, selected={}, unlocked={}",
        start.elapsed(),
        app.session.mode,
        app.session.progress.selected,
        app.session.progress.unlocked
    );
    for (level, samples) in levels {
        let n = samples.len();
        let zero = samples.iter().filter(|s| s[1] == 0.0).count();
        let multi = samples.iter().filter(|s| s[1] > 1.0).count();
        let late = samples.iter().filter(|s| s[0] > 25.0).count();
        let worst_repeats = samples
            .windows(60)
            .map(|window| window.iter().filter(|s| s[1] == 0.0).count())
            .max()
            .unwrap_or(zero);
        println!(
            "{level}: frames={n} zero_ticks={zero} multi_ticks={multi} intervals_over_25ms={late}"
        );
        println!("  Most repeated simulation poses within 60 rendered frames: {worst_repeats}");
        for (axis, name) in [(0, "interval"), (2, "update"), (3, "draw")] {
            let mut values: Vec<_> = samples.iter().map(|s| s[axis]).collect();
            values.sort_by(f64::total_cmp);
            println!(
                "  {name} ms: median={:.3} p95={:.3} max={:.3}",
                values[n / 2],
                values[(n - 1) * 95 / 100],
                values[n - 1]
            );
        }
    }
    Ok(())
}
