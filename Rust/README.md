# IronJump — Rust port

A Rust port of the original iPhone game with a Macroquad desktop backend. The gameplay follows
`SharedClasses` and the campaign flow in `IronJump/Classes/EAGLView.m`. Rendering
uses the original iPhone atlas, bitmap font, and menu artwork.

## Run

From the repository root:

```sh
cargo run --manifest-path Rust/Cargo.toml
```

A graphical desktop with an OpenGL-capable driver is required. Gameplay fills the window, keeping its shorter dimension at 320 logical units.
Wider or taller screens reveal more of the level with uniformly scaled sprites.
Resizing recenters the player without resetting the game. The original 480 × 320
menu artwork remains centered with letterboxing.

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

- `src/lib.rs`: backend-independent library; depends only on `glam` and `roxmltree`
  when built without default features.
- `src/main.rs`: Macroquad entry point.
- `src/app.rs`: semantic actions and the fixed 60 Hz simulation clock.
- `src/input.rs`: analog steering, jump, and one-shot menu/application actions.
- `src/session.rs`: menu, stages, transitions, restart, victory, and presentation.
- `src/progress.rs`: progress data and its text format; no filesystem access.
- `src/math.rs`: vectors and collision rectangles independent of the renderer.
- `src/viewport.rs`: responsive gameplay, fitted menus, and pointer coordinate mapping.
- `src/drawing.rs`: renderer trait, sprite identities, colors, and draw styles.
- `src/levels.rs`: embedded campaign and development levels.
- `src/game.rs`: game orchestration, completion checks, and HUD.
- `src/game/`: one module per original game object: `player.rs`, `platform.rs`,
  `movable_platform.rs`, `elevator.rs`, `diamond.rs`, `exit.rs`, `trampoline.rs`,
  `magnet.rs`, and `speed_power_up.rs`. Each owns its object's state and behavior.
- `src/game/world.rs`: object update order, world scrolling, and background drawing.
  Elevator-specific world interactions live alongside the elevator in `elevator.rs`.
- `src/game/game_object.rs`: shared object trait, update context, and world effects.
- `src/game/collision.rs`: shared collision geometry.
- `src/game/level_loader.rs`: XML validation and object construction.
- `src/backend.rs`: Macroquad renderer, render target, scaling, and sharp overlay text.
- `src/backend/assets.rs`: GPU textures, atlas regions, and fonts. The rotating ball
  uses linear filtering; tiles and the game render target use nearest filtering.
- `src/platform.rs`: desktop runner connecting the library to input, storage, and drawing.
- `src/platform/`: keyboard/touch translation, command-line options, and file-backed saves.
- `src/game/test_support.rs`: shared game fixtures, compiled only for tests.

Unit tests live at the bottom of the implementation file they cover, inside
`#[cfg(test)] mod tests` blocks. They can check private state without exposing it
as a public testing API. World tests cover interactions and update ordering;
loader tests cover validation and simulation of every bundled level. The exit
tests also cover diamond collection and completion with uncollected diamonds.

Objects use `UpdateContext` to inspect other objects without borrowing themselves
again. Effects that change other bodies are applied by `World` after the current
object borrow ends. Invisible collectibles stay in the object array, preserving
stable indices used by elevator attachments.

The game and session submit drawing through `Renderer`; their object-local `draw()`
methods never access GPU textures or global drawing functions. A different backend
implements that trait and supplies a runner that creates `FrameInput`, calls
`Session::resize` with the logical viewport size, calls `App::frame`, persists
`Progress`, and presents the frame. Sprite order, rotation in radians, font
positioning, and logical game units (32 per tile) are part of the drawing contract.
The viewport may change independently of the simulation; level loads retain it.
Physical controls and their displayed hints belong to the platform adapter. On resume, a runner resets the app clock and its elapsed-time
measurement so suspended time does not advance the simulation.

Macroquad is an optional dependency behind the default `macroquad-backend` feature.
To build and test the library without a graphics backend or display:

```sh
cargo test --offline --manifest-path Rust/Cargo.toml --no-default-features
```

This separation prepares for another backend or Android adapter; Android packaging
and accelerometer support are not implemented yet.

This ports the iPhone game runtime to the existing desktop Rust application.
Cocoa level-editor tools, editor replay recording, and native iOS application
packaging are outside this runtime.

## Development checks

```sh
cargo fmt --manifest-path Rust/Cargo.toml --check
cargo clippy --offline --manifest-path Rust/Cargo.toml --all-targets -- -D warnings
cargo test --offline --manifest-path Rust/Cargo.toml
cargo test --offline --manifest-path Rust/Cargo.toml --no-default-features
cargo build --offline --locked --manifest-path Rust/Cargo.toml
python3 tools/convert_levels.py --check
```

Tests cover all object types, collision sequencing, moving platforms, power-up
expiry, progression, persistence, viewport mapping, and frame-rate-independent
simulation. A recording renderer checks drawing without GPU resources, including
layer order, collected-object visibility, menu/victory presentation, and drawing
every converted level after simulation.
Run one group by its module name, for example:

```sh
cargo test --offline --manifest-path Rust/Cargo.toml game::player::tests
```
