# Your First Game

Let's create a simple game that draws a colored square and responds to input. This will introduce you to the core concepts of Emberware game development.

## Create the Project

```bash
cargo new --lib my-first-game
cd my-first-game
```

## Configure Cargo.toml

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "my-first-game"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
```

Key settings:
- `crate-type = ["cdylib"]` - Builds a C-compatible dynamic library (required for WASM)
- `opt-level = "s"` - Optimize for small binary size
- `lto = true` - Link-time optimization for even smaller binaries

## Write Your Game

Replace `src/lib.rs` with the following code:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Panic handler required for no_std
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// FFI imports from the Emberware runtime
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn button_pressed(player: u32, button: u32) -> u32;
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Game state - stored in static variables for rollback safety
static mut SQUARE_Y: f32 = 200.0;

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Set the background color (dark blue)
        set_clear_color(0x1a1a2eFF);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Move square with D-pad
        if button_pressed(0, BUTTON_UP) != 0 {
            SQUARE_Y -= 10.0;
        }
        if button_pressed(0, BUTTON_DOWN) != 0 {
            SQUARE_Y += 10.0;
        }

        // Reset position with A button
        if button_pressed(0, BUTTON_A) != 0 {
            SQUARE_Y = 200.0;
        }

        // Keep square on screen
        SQUARE_Y = SQUARE_Y.clamp(20.0, 450.0);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw title text
        let title = b"Hello Emberware!";
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            80.0,
            50.0,
            32.0,
            0xFFFFFFFF,
        );

        // Draw the moving square
        draw_rect(200.0, SQUARE_Y, 80.0, 80.0, 0xFF6B6BFF);

        // Draw instructions
        let hint = b"D-pad: Move   A: Reset";
        draw_text(
            hint.as_ptr(),
            hint.len() as u32,
            60.0,
            500.0,
            18.0,
            0x888888FF,
        );
    }
}
```

## Understanding the Code

### No Standard Library

```rust
#![no_std]
#![no_main]
```

Emberware games run in a minimal WebAssembly environment without the Rust standard library. This keeps binaries small and avoids OS dependencies.

### FFI Imports

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    // ...
}
```

Functions are imported from the Emberware runtime. See the [Cheat Sheet](../cheat-sheet.md) for all available functions.

### Static Game State

```rust
static mut SQUARE_Y: f32 = 200.0;
```

All game state lives in `static mut` variables. This is intentional - the Emberware runtime automatically snapshots all WASM memory for rollback netcode. No manual state serialization needed!

### Colors

Colors are 32-bit RGBA values in hexadecimal:
- `0xFFFFFFFF` = White (R=255, G=255, B=255, A=255)
- `0xFF6B6BFF` = Salmon red (R=255, G=107, B=107, A=255)
- `0x1a1a2eFF` = Dark blue (R=26, G=26, B=46, A=255)

## Build and Run

### Build the WASM file:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/my_first_game.wasm`

### Run in the Emberware player:

```bash
ember run target/wasm32-unknown-unknown/release/my_first_game.wasm
```

Or load the `.wasm` file directly in the Emberware Library application.

## What You've Learned

- Setting up a Rust project for WASM
- The `#![no_std]` environment
- Importing FFI functions from the runtime
- The three lifecycle functions: `init()`, `update()`, `render()`
- Drawing 2D graphics with `draw_rect()` and `draw_text()`
- Handling input with `button_pressed()`
- Using static variables for game state

---

**Next:** [Understanding the Game Loop](./game-loop.md)
