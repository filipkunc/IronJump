//! Optional buffered capture. A separate thread writes chunks outside the render loop.
use ironjump::{progress::Progress, replay::Frame};
use std::{
    io::Write,
    path::Path,
    sync::mpsc::{self, Sender},
    time::Instant,
};

pub struct Trace {
    sender: Sender<String>,
    buffer: String,
    frames: usize,
}
impl Trace {
    pub fn new(path: &Path, progress: Progress) -> std::io::Result<Self> {
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "IronJumpTrace 1")?;
        write!(file, "{}", progress.encode())?;
        let (sender, receiver) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            for chunk in receiver {
                if let Err(error) = file.write_all(chunk.as_bytes()) {
                    eprintln!("Could not write replay trace: {error}");
                    break;
                }
            }
        });
        Ok(Self {
            sender,
            buffer: String::with_capacity(32768),
            frames: 0,
        })
    }
    pub fn record(&mut self, frame: &Frame, ticks: usize, update: Instant, draw: Instant) {
        use std::fmt::Write;
        let update_ms = draw.duration_since(update).as_secs_f64() * 1000.0;
        let draw_ms = draw.elapsed().as_secs_f64() * 1000.0;
        writeln!(
            self.buffer,
            "{} | {} {:.6} {:.6}",
            frame.encode(),
            ticks,
            update_ms,
            draw_ms
        )
        .unwrap();
        self.frames += 1;
        if self.frames >= 60 {
            self.flush();
        }
    }
    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let buffer = std::mem::replace(&mut self.buffer, String::with_capacity(32768));
            let _ = self.sender.send(buffer);
        }
        self.frames = 0;
    }
}
impl Drop for Trace {
    fn drop(&mut self) {
        self.flush();
    }
}
