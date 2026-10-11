//! Opt-in live gameplay timings; no per-frame logging or GPU readback.
use std::time::Instant;

#[derive(Default)]
pub struct Profile {
    frames: Vec<[f64; 4]>,
    zero_ticks: usize,
    multi_ticks: usize,
}
impl Profile {
    pub fn record(&mut self, elapsed: f64, update: Instant, draw: Instant, ticks: usize) {
        self.frames.push([
            elapsed * 1000.0,
            update.elapsed().as_secs_f64() * 1000.0,
            draw.duration_since(update).as_secs_f64() * 1000.0,
            draw.elapsed().as_secs_f64() * 1000.0,
        ]);
        self.zero_ticks += usize::from(ticks == 0);
        self.multi_ticks += usize::from(ticks > 1);
        if self.frames.len() < 300 {
            return;
        }
        let late = self.frames.iter().filter(|f| f[0] > 25.0).count();
        let mut p95 = [0.0; 4];
        let mut max = [0.0; 4];
        for axis in 0..4 {
            self.frames.sort_by(|a, b| a[axis].total_cmp(&b[axis]));
            p95[axis] = self.frames[284][axis];
            max[axis] = self.frames[299][axis];
        }
        macroquad::logging::info!(
            "IronJumpFrame frames=300 late={} zero_ticks={} multi_ticks={} p95_ms interval/work/update/draw={:.2?} max_ms={:.2?}",
            late,
            self.zero_ticks,
            self.multi_ticks,
            p95,
            max
        );
        self.frames.clear();
        self.zero_ticks = 0;
        self.multi_ticks = 0;
    }
}
