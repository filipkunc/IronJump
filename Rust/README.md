# IronJump — Rust port

A Rust port of the original iPhone game with a Macroquad desktop/Android backend. The gameplay follows
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

## Android preview

The first Android build uses the same Macroquad renderer and game library. It
supports tilt steering, motion jumping, touch controls, Android Back, app-private
progress saves, and clock reset on resume. It opens in either landscape orientation.
The activity requests a 60 Hz display mode to match the simulation; this is an
app-local preference that the device may override.
The launcher uses the original `IronJump/Icon.png` through an Android adaptive
icon, inset to keep the ball visible under different launcher masks. The resource
is symlinked to the original artwork rather than maintaining a second copy.

On Linux, install Java, Git, Cargo, `zip`, Android SDK platform/build-tools 36,
NDK 28.2.13676358, and the Rust ARM64 Android target. Then run from the repository root:

```sh
rustup target add aarch64-linux-android
Rust/android/build.sh
adb install --no-incremental -r Rust/target/android-artifacts/release/apk/ironjump.apk
adb shell am start -n org.ironjump.game/.MainActivity
```

`ANDROID_HOME` defaults to `~/Android/Sdk`; `NDK_HOME` defaults to its
`ndk/28.2.13676358` directory. Both can be overridden. The script downloads and
builds a pinned upstream `cargo-quad-apk` under `Rust/target/android-tools`, with
our small compatibility patch for Rust 2024 exports and native library naming.
Subsequent builds reuse that tool. The APK uses the local Android debug signing
key and is intended for sideloading, not store distribution.

The Android package is `org.ironjump.game` (also used by the JNI callback names).
`android/activity.java.in` supplies activity hooks through `quad.toml`, and
`src/platform/android.rs` copies the save path and passes lifecycle/Back events
between the Android UI thread and the game thread. Accelerometer samples use a
mutex-protected adapter in `src/platform/motion.rs`, with tests beside the filter.
SDK or JNI types do not enter
the game library. A cold launch restores campaign progress and level selection;
a level in progress is retained only while the app process remains alive.

Verified on a Samsung SM-A556B running Android 16: launch, menu touch input,
wide gameplay rendering, movement, Back navigation, home/resume, and saved level
selection across a cold restart. Further
work includes tuning motion controls on real devices and broader lifecycle/device testing.

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
  emulation. Touch steering overrides tilt while held.
- On Android, tilt the phone to steer; a quick vertical motion triggers the
  original gesture jump. Touch-to-jump remains available in the right third.
  Steering retains the original 0.02g dead zone and analog strength. The jump
  detector retains the 0.05g threshold and 30 Hz filter response, adjusted for
  actual sample intervals. A detected jump is held for 100 ms so it reaches a
  simulation tick. The first sample after resume, rotation, or a gap seeds the
  filter without jumping. Readings older than 250 ms release motion input.
- Motion uses the accelerometer including gravity, remapped to screen coordinates
  in either landscape orientation. Sensors stop when the app pauses. Devices
  without an accelerometer can still use touch controls.

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
platforms, then the player, preserving the original gameplay order. Drawing
interpolates between the last two simulation poses, including scrolling objects
and ball rotation, so small frame timing variations do not show repeated poses
followed by double movement. This adds up to one simulation tick of presentation
delay. Level loads, viewport changes, and resume synchronize the poses immediately.
The platform measures frame intervals with a monotonic clock.

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
- `src/drawing/translated.rs`: presentation offsets with matching visibility bounds.
- `src/replay.rs`: portable frame-input trace encoding/decoding; no filesystem access.
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
- `src/backend/assets.rs`: shared gameplay/font atlas, sprite vertices, and visibility culling.
- `src/backend/sampling.rs`: a single sprite shader retaining linear filtering for
  the rotating ball and nearest sampling for tiles/glyphs. The game render target
  also uses nearest filtering when scaled to the window.
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

This separation also allows another drawing backend. The Android preview uses
Macroquad and keeps sensor integration in the platform adapter.

This ports the iPhone game runtime to the existing desktop Rust application.
Cocoa level-editor tools, editor replay recording, and native iOS application
packaging are outside this runtime.

## Rendering performance

Like `SharedClasses/FPGameAtlas.m`, the renderer appends visible sprite geometry
to a shared batch. Repeated platforms visit only the tile rows/columns intersecting
the current viewport; individual sprites use rotation-aware bounds. Offscreen
objects still participate in simulation. The original artwork and converted font
are packed into one GPU atlas at startup, with a padded ball region for linear
sampling. Indexed quads use four vertices and six indices; Macroquad manages the
shared streaming buffers. Unlike the original `FPGame.m` background VBO, the Rust
background currently joins this streaming batch rather than a separate static buffer.

Four fixed gameplay snapshots on the SM-A556B measured two draw calls each: one
for the scene including HUD, and one for presenting the render target. Menus and
victory overlays use additional batches. The `speed4` snapshot dropped from six
draw calls / 2,002 quads to two / 334, including the presentation quad. Its median
CPU submission time went from 1.29 ms to 0.65 ms in these runs. These are short
render-only samples, not a full gameplay or GPU timing benchmark; the display
preference also changed from the observed 120 Hz to 60 Hz between runs.

To repeat the startup benchmark on Android:

```sh
Rust/android/build.sh --features render-benchmark
adb install --no-incremental -r Rust/target/android-artifacts/release/apk/ironjump.apk
adb shell am force-stop org.ironjump.game
adb shell am start -n org.ironjump.game/.MainActivity
adb logcat -s SAPP:I '*:S'
```

`IronJumpBench` lines report actual batch/quad counts and median/p95 frame intervals
and CPU submission times for four level snapshots, then open the normal menu.
Saved progress is untouched. GPU capture frames are excluded from timing samples;
submission times exclude deferred GPU flushes. Rebuild with `Rust/android/build.sh`
without the feature to remove the startup benchmark from the APK.

For live gameplay diagnostics, build with `--features frame-profile` instead.
This logs a timing summary every 300 frames and records semantic input, elapsed
time, viewport, clock resets, and per-frame update/drawing times. A worker thread
writes buffered chunks so the game thread does not perform per-frame disk I/O.
The trace starts with current campaign progress; it supports the bundled levels.
It is overwritten on a cold launch, so retrieve it before restarting the app:

```sh
adb pull /sdcard/Android/data/org.ironjump.game/files/ironjump-trace.txt /tmp/ironjump-trace.txt
cargo run --release --manifest-path Rust/Cargo.toml --no-default-features \
  --example replay -- /tmp/ironjump-trace.txt
```

The replay runs without graphics, checks every recorded simulation tick count,
and reports the captured frame/update/draw timings by level. It never writes
progress. Return to the menu after a recording and leave the app running while
pulling the file; a partial final second can otherwise remain buffered. The
normal build has tracing disabled. On desktop the trace sits beside the selected
progress file; external custom levels disable recording because their XML is not
embedded in the trace.

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
