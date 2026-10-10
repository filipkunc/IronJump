# IronJump — Rust port

A desktop port of the original iPhone game using Macroquad. The gameplay follows
`SharedClasses` and the campaign flow in `IronJump/Classes/EAGLView.m`. Rendering
uses the original iPhone atlas, bitmap font, and menu artwork.

## Run

From the repository root:

```sh
cargo run --manifest-path Rust/Cargo.toml
```

A graphical desktop with an OpenGL-capable driver is required. The game renders
at 480 × 320 and scales with letterboxing when the window is resized.

## Controls

- **Left / Right** or **A / D**: move.
- **Up**, **W**, or **Space**: jump. Holding jump repeats jumps when grounded.
- **R**: restart the current level.
- **Escape**: return to the level menu; Escape in the menu quits.
- **Q**: quit.
- In the menu, click a level and **Play**, or press **Enter** to start the selected
  level. Left/Right changes stage; Up/Down selects an available adjacent level.
- On a touch-capable display, hold the left/middle third to move left/right and
  the right third to jump. Menu buttons also respond to touch through mouse
  emulation. These controls replace the original iPhone accelerometer input.

## Gameplay

All nine archived object types are supported: player, platforms, movable
platforms, elevators, diamonds, exits, trampolines, magnets, and speed boosts.
The 30 campaign levels follow the original iPhone order in stages of up to 11
levels. Five additional archived development levels appear in a custom stage.

Touching an exit advances after 31 simulation ticks and unlocks the next campaign
level. Diamonds are counted but are not required to use an exit, matching the
original. Falling below all fixed platforms restarts after 31 ticks. Completing
the final campaign or custom level displays the victory screen.

Speed boosts last six seconds and respawn. Trampolines launch both the player
and movable platforms; magnets attract the player from below. Elevator motion,
attachments, carrying, and collision correction are coordinated by `World`.
Initially attached crates are moved once, avoiding the original double-movement
path through both attachment and movable carrying logic.

The simulation runs at 60 Hz. Non-movable objects update first, then movable
platforms, then the player, preserving the original gameplay order.

## Progress and custom levels

Progress is saved in `ironjump-progress.txt` in the working directory. It stores
the selected level and highest unlocked campaign level. To choose another file:

```sh
cargo run --manifest-path Rust/Cargo.toml -- --progress /path/to/progress.txt
```

Open an XML level directly, or add a folder of XML levels to the custom stages:

```sh
cargo run --manifest-path Rust/Cargo.toml -- --level IronJump/XLevels/Converted/example.xlevel
cargo run --manifest-path Rust/Cargo.toml -- --levels-dir /path/to/custom-levels
```

Custom files are validated before play. A level must have one player, known
object types, finite coordinates, and positive tile dimensions. Campaign levels
and artwork are embedded in the executable, so they need no runtime asset paths.

All 35 original binary `.level` archives are converted into
`IronJump/XLevels/Converted`. Regenerate or verify them using Python 3.10+:

```sh
python3 tools/convert_levels.py
python3 tools/convert_levels.py --check
```

For an additional binary archive, use the converter before loading it in Rust:

```sh
python3 tools/convert_levels.py /path/to/custom.level --output-dir /path/to/xml
```

## Code layout

- `src/main.rs`: window, desktop/touch input, fixed simulation steps, and scaling.
- `src/session.rs`: menu, stages, progress, transitions, restart, and victory.
- `src/levels.rs`: embedded campaign and development levels.
- `src/game.rs`: player physics, world, collision geometry, XML loading, and HUD.
- `src/game/elevator.rs`: elevator route state and coordinated world interactions.
- `src/game/objects.rs`: collectibles, exit, trampoline, and magnet behavior.
- `src/assets.rs`: iPhone atlas regions, sprite drawing, font, and menu textures.

Objects use `UpdateContext` to inspect other objects without borrowing themselves
again. Effects that change other bodies are applied by `World` after the current
object borrow ends. Invisible collectibles stay in the object array, preserving
stable indices used by elevator attachments.

This ports the iPhone game runtime to the existing desktop Rust application.
Cocoa level-editor tools, editor replay recording, and native iOS application
packaging are outside this runtime. The old `src/demo.rs` is not part of the build.

## Development checks

```sh
cargo fmt --manifest-path Rust/Cargo.toml --check
cargo clippy --offline --manifest-path Rust/Cargo.toml --all-targets -- -D warnings
cargo test --offline --manifest-path Rust/Cargo.toml
cargo build --offline --locked --manifest-path Rust/Cargo.toml
python3 tools/convert_levels.py --check
```

Tests cover all object types, collision sequencing, moving platforms, power-up
expiry, progression, persistence, and loading/simulating every converted level.
