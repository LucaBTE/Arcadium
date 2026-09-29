# Arcadium
Arcade cabinet in your terminal. Completely extendible.

## WASM host API

Games export `arcadium_init() -> i32` (zero means failure),
`arcadium_update(delta_seconds: f32)`, and `arcadium_shutdown()`.
The Wasmtime instance persists until exit or a runtime error, so guest memory
retains game state. Older guests with a parameterless update must be rebuilt.

Core WebAssembly imports use the `arcadium` namespace:

| Import | Signature |
| --- | --- |
| `screen_width` | `() -> i32` |
| `screen_height` | `() -> i32` |
| `draw_char` | `(x: i32, y: i32, character: i32)` |
| `draw_cell` | `(x: i32, y: i32, character: i32, foreground: i32, background: i32)` |
| `key_pressed` | `(key: i32) -> i32` |

Coordinates start at zero inside the border; the footer is excluded. Dimensions
are available at initialization and change on resize without restarting the guest.
Draw the whole frame each update: the character buffer and input flags clear at
frame start. Out-of-bounds coordinates, invalid Unicode scalars, and control
characters are ignored. Use single-column printable characters for predictable
cell placement; wide/combining character layout is not supported yet.

`draw_cell` adds foreground/background color for any game, using packed RGB
integers (`0xRRGGBB`); `-1` selects the terminal default. Invalid colors ignore
the draw. `draw_char` still draws white on the terminal default background,
so existing guests continue to work. Colors clear along with characters each
frame. Ratatui renders these cells using the existing terminal backend; no
additional graphics library is needed. RGB colors require a true-color terminal.
Games importing `draw_cell` require this updated host; older hosts cannot load
that import. The lifecycle and manifest schema are unchanged.

Key codes: `UP=0`, `DOWN=1`, `LEFT=2`, `RIGHT=3`, `W=4`, `A=5`, `S=6`,
`D=7`, `SPACE=8`, `ENTER=9`. Letter input is case insensitive.
`key_pressed` returns 1 when a press or repeat was received this frame, otherwise
0 (including unknown codes). Repeated queries do not consume input. ESC belongs
to Arcadium and shuts down the game before returning to the library.

The loop targets 60 FPS using elapsed wall-clock delta and a remaining-frame
sleep. Input is drained with non-blocking polling. There is no fixed timestep or
held-key tracking: repeat frequency depends on the terminal. Guests should account
for large deltas after stalls. Execution budgets for nonterminating guests are
not implemented; ordinary WASM traps and incompatible exports become UI errors.

## Run the host test guest

From the repository root, with the `wasm32-unknown-unknown` Rust target and Python 3:

```bash
rustup target add wasm32-unknown-unknown
bash adm-build/test_game/build.sh
mkdir -p /tmp/arcadium-test-data/arcadium/games
cp adm-build/test_game/host-test.adm /tmp/arcadium-test-data/arcadium/games/
XDG_DATA_HOME=/tmp/arcadium-test-data cargo run -p arcadium-cli
```

Select **Host API Test** and press ENTER. LEFT/RIGHT move `@`; resize the terminal
and confirm it remains vertically centered. ESC returns to the library; Q quits.
The other bundled packages remain discoverable, but old lifecycle stubs display
an ABI error until rebuilt. The sample uses delta time for movement and guest
memory for position; it is not Pong.

Validation: `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test`.

## Pong

Pong is a dependency-free, standalone guest crate (outside the host workspace).
Its dark court, cyan/magenta paddles, bright ball, score bar, and mode cards use
the shared RGB cell API. Compact layouts support small terminals; use a Unicode
monospace font and a true-color terminal for the intended appearance.
Build and package it from the repository root with Rust's
`wasm32-unknown-unknown` target and Python 3 installed:

```bash
bash games/pong/build.sh
cargo run -p arcadium-cli
```

The script builds `games/pong/target/wasm32-unknown-unknown/release/pong.wasm`
and replaces `bundled-games/pong.adm` with a ZIP containing `manifest.toml` and
`game.wasm`. Select **Pong** and press ENTER. Choose **One player** (against the
computer) or **Two players** with W/S or UP/DOWN, then press ENTER to start.
W/S move the left paddle. In two-player mode, UP/DOWN move the right paddle;
in one-player mode, the computer controls it with a limited tracking speed.
ESC returns to the library; launch Pong again to choose another mode.
Each human paddle moves two rows per input frame with a press/repeat event,
clamped to the playfield; holding a key follows the terminal's key repeat.
The ball serves toward the player who conceded. Gameplay stops below a 24×10
framebuffer and resumes when space is available; scores survive resizing.

Validate the guest separately from the host workspace:

```bash
cargo fmt --manifest-path games/pong/Cargo.toml --check
cargo check --manifest-path games/pong/Cargo.toml --target wasm32-unknown-unknown
cargo clippy --manifest-path games/pong/Cargo.toml --target wasm32-unknown-unknown -- -D warnings
cargo test --manifest-path games/pong/Cargo.toml
```

For a play check, try both modes, move the human paddles, return a ball, let a ball pass
and check the score/reset, resize the terminal (including very small sizes),
then press ESC to return to the library.
