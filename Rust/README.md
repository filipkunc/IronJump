# IronJump Rust starter

A small desktop starting point using [Macroquad](https://macroquad.rs/), which
provides a window, input, GPU-backed 2D drawing, and text via Miniquad. It uses
the OpenGL-style stack you asked for, with no custom shaders or renderer setup
needed. Audio is disabled. There is no game logic, physics, or level loading yet.

## Run

Install a current stable Rust toolchain, then from the repository root:

```sh
cargo run --manifest-path Rust/Cargo.toml
```

Or run `cargo run` inside `Rust/`. The first build downloads dependencies. A
graphical desktop with an OpenGL-capable driver is required to open the window.
The starter targets desktop use; mobile/browser packaging is not set up here.

You should see a movable ball, a diamond, and text. Use arrows or WASD to move
the ball in four directions; Escape or closing the window quits. Resize the
window to see drawing bounds adjust.

## Where to start

- `src/main.rs`: window settings and the input/update/draw frame loop.
- `src/assets.rs`: texture ownership and one-time loading.
- `src/demo.rs`: a small state struct with `update(dt)` and `draw(&assets)`;
  replace this with your own port as you go.

The PNGs are embedded from `Canvas/Images` at compile time, so launching the
binary does not depend on the current directory or on loose asset files.
Changing an embedded image requires rebuilding. Create textures after the
graphics context starts and reuse them across frames.

## Drawing images and text

Inside `draw`, an image at its original size is just:

```rust
draw_texture(&assets.ball, 100.0, 200.0, WHITE);
```

`draw_texture_ex` adds scaling, rotation, flipping, and atlas source rectangles:

```rust
draw_texture_ex(
    &assets.ball,
    100.0,
    200.0,
    WHITE,
    DrawTextureParams {
        dest_size: Some(vec2(64.0, 64.0)),
        // source: Some(Rect::new(x, y, width, height)), // atlas pixels
        rotation: 0.0, // radians
        ..Default::default()
    },
);
```

Coordinates start at the top left, with positive y downward. Image coordinates
refer to the top-left corner; text y refers to the baseline. Drawing uses the
current window coordinate space, not a fixed game resolution. Later drawing
appears on top of earlier drawing. `WHITE` preserves an image's original color;
another color tints it. The starter uses nearest-neighbor texture filtering;
switch to `FilterMode::Linear` in `assets.rs` for smooth scaling.

Text works immediately with the bundled default font:

```rust
draw_text("Hello, Rust!", 24.0, 48.0, 32.0, WHITE);
```

For your own TTF font, add a font file, load it once in `main` after graphics
initialization, and pass it when drawing:

```rust,ignore
let font = load_ttf_font_from_bytes(include_bytes!("../assets/my-font.ttf"))
    .expect("could not load font");
draw_text_ex("Hello!", 24.0, 48.0, TextParams {
    font: Some(&font),
    font_size: 32,
    color: WHITE,
    ..Default::default()
});
```

`dt` is in seconds. The demo caps long frames and moves at a constant speed,
including diagonally. Choose your own physics/timing model when porting the game.
The `async` entry point and `next_frame().await` yield to Macroquad's frame loop;
no separate async runtime or background thread is needed.

## Development checks

```sh
cargo fmt --manifest-path Rust/Cargo.toml --check
cargo clippy --manifest-path Rust/Cargo.toml --all-targets -- -D warnings
cargo build --manifest-path Rust/Cargo.toml --locked
```

API references: [textures](https://docs.rs/macroquad/0.4.16/macroquad/texture/index.html),
[text](https://docs.rs/macroquad/0.4.16/macroquad/text/index.html),
[input](https://docs.rs/macroquad/0.4.16/macroquad/input/index.html).
