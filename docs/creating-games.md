# How do I create a new Arcadium game?

## Overview

An Arcadium game is a Rust library compiled to WebAssembly using `wasm32-unknown-unknown` target.\
A game package contains two files: `game.wasm` (the compiled game) and `manifest.toml` (game metadata and Arcadium configuration).
Once you have the package, the .adm file can be installed directly from Arcadium and launched from the game list.

```text
my-game.adm/
├── manifest.toml
└── game.wasm
```

## Prerequisites

Install Rust, the WASM target, and a ZIP utility. The examples below use `zip` on Linux:

```bash
rustup target add wasm32-unknown-unknown
```

## 1. Create the game crate

Start with this layout:

```text
my-game/
├── Cargo.toml
├── manifest.toml
└── src/
    └── lib.rs
```

## 2. Configure the WASM target

Use this `Cargo.toml`, following the bundled games:

```toml
[package]
name = "my-game"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
crate-type = ["cdylib"]

[workspace]

[profile.release]
opt-level = "s"
lto = true
strip = true
panic = "abort"
```

!NOTE: The empty `[workspace]` keeps the crate independent when created inside another Rust workspace.

## 3. Game lifecycle

Export these C ABI functions. Arcadium calls `arcadium_init` once at launch (return nonzero for success), `arcadium_update` once per frame, and `arcadium_shutdown` when the session closes.

```rust
#[unsafe(no_mangle)]
pub extern "C" fn arcadium_init() -> i32 { 1 }

#[unsafe(no_mangle)]
pub extern "C" fn arcadium_update(_delta_seconds: f32) {
    if unsafe { screen_width() > 0 && screen_height() > 0 } {
        unsafe { draw_cell(0, 0, '@' as i32, 0xFF7338, -1) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn arcadium_shutdown() {}
```

Combine this with the import block below in `src/lib.rs` for a minimal guest that draws `@` in the top-left corner.

## 4. Host API

Import host functions from the `arcadium` module:

```rust
#[link(wasm_import_module = "arcadium")]
unsafe extern "C" {
    fn screen_width() -> i32;
    fn screen_height() -> i32;
    fn draw_cell(x: i32, y: i32, character: i32, foreground: i32, background: i32);
    fn key_pressed(key: i32) -> i32;
}
```

Call imported functions inside `unsafe` blocks. The current imports are:

| Function | Purpose |
| --- | --- |
| `screen_width() -> i32`, `screen_height() -> i32` | Current drawable size |
| `draw_char(x: i32, y: i32, character: i32)` | Draw a character with default colors |
| `draw_cell(x: i32, y: i32, character: i32, foreground: i32, background: i32)` | Draw a colored character |
| `key_pressed(key: i32) -> i32` | Read this frame's key input |
| `load_score() -> i64` | Read this game's saved score |
| `save_score(score: i64) -> i32` | Save a nonnegative score; returns 1 on success |
| `request_exit()` | Return to the library after the update |

For a character, pass its Unicode scalar value, such as `'@' as i32`. Single-column printable characters are safest for terminal layouts.

## 5. Input

`key_pressed(key) == 1` when Arcadium received a press or repeat for that key during the current frame. The IDs are:

| Key | ID | Key | ID |
| --- | ---: | --- | ---: |
| UP | 0 | DOWN | 1 |
| LEFT | 2 | RIGHT | 3 |
| W | 4 | A | 5 |
| S | 6 | D | 7 |
| SPACE | 8 | ENTER | 9 |

Letter input is case insensitive. ESC belongs to Arcadium and returns to the library; games do not handle it.

## 6. Persistent state

The WASM instance stays alive for the game session, so state can persist between `arcadium_update` calls. The bundled games use thread-local `Cell` or `RefCell` for this:

```rust
use std::cell::Cell;

thread_local! {
    static POSITION: Cell<i32> = const { Cell::new(0) };
}

// In arcadium_update:
POSITION.with(|position| position.set(position.get() + 1));
```

Reset state in `arcadium_init` if it should start fresh on launch.

## 7. Rendering

`screen_width` and `screen_height` give the current drawable area inside Arcadium's border. The host clears the guest framebuffer each frame, so draw the complete visible frame during `arcadium_update`.

`draw_cell` uses `0xRRGGBB` for foreground and background colors; `-1` selects the terminal default, as in the lifecycle example. Coordinates outside the framebuffer are ignored.

## 8. Delta time

`arcadium_update(delta_seconds: f32)` receives elapsed seconds since the previous frame. Real-time games can move with `position += velocity * delta_seconds;` and should clamp unusually large deltas after stalls. Turn-based games may ignore delta time.

## 9. Scores and exiting

`load_score()` and `save_score(score)` provide a persistent score for each game ID. Load on startup and save when a high score changes, instead of every frame. Scores are nonnegative and fit in `u32`, though the import takes `i64`.

Call `request_exit()` for an in-game Back, Quit, or Return to Library action. Arcadium returns to the library after that update. ESC works independently as the host exit key.

## 10. manifest.toml

Create this file beside `Cargo.toml`:

```toml
[game]
id = "com.example.my-game"
name = "My Game"
author = "Your Name"
version = "0.1.0"
description = "A short description."

[arcadium]
sdk = 1
entry = "game.wasm"
```

The `id` is a globally unique logical game ID. `sdk` is the Arcadium SDK version, currently `1`. `entry` names the WASM file inside the ADM.

## 11. Build the game

From `my-game/`, run:

```bash
cargo build --release --target wasm32-unknown-unknown
```

The result is normally `target/wasm32-unknown-unknown/release/my_game.wasm`: Cargo replaces the package name's hyphen with an underscore in the library filename.

## 12. Package the ADM

Copy the build output to `game.wasm`, then place it and the manifest at the ZIP root:

```bash
cp target/wasm32-unknown-unknown/release/my_game.wasm game.wasm
zip my-game.adm manifest.toml game.wasm
```

The ADM needs at least those two root-level files. See `games/*/build.sh` for complete bundled packaging examples.

## 13. Install and test

1. Launch Arcadium.
2. Press **I** or select **Install new game**.
3. Choose `my-game.adm`.
4. Arcadium validates and installs it; the game appears immediately in the library.
5. Select the game and press **ENTER** to play. Press **ESC** to return.

## Reference games

- `games/pong/` demonstrates real-time movement, delta time, and local multiplayer or AI.
- `games/tictactoe/` demonstrates turn-based state, cursor input, and Minimax AI.
- `games/snake/` demonstrates grid simulation, dynamic state, timed movement, and score persistence.

`games/common/` is shared code used internally by bundled examples. External games should remain self-contained and should not depend on it.
