mod backend;
mod platform;
use platform::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    platform::run().await;
}
