# Your First Game

Let's create a simple game that draws a colored square and responds to input. This will introduce you to the core concepts of Nethercore game development.

## Create the Project

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```bash
cargo new --lib my-first-game
cd my-first-game
```
Copy the SDK's entire `include/zx` directory to `src/zx` (not just `mod.rs`).
On Windows, with `$env:NETHERCORE_HOME` set as in [Prerequisites](./prerequisites.md):

```powershell
Copy-Item -Recurse "$env:NETHERCORE_HOME\include\zx" src\zx
```
{{#endtab}}

{{#tab name="C/C++"}}
```bash
mkdir my-first-game
cd my-first-game
```

Copy `zx.h` from the Nethercore repository to your project folder.
{{#endtab}}

{{#tab name="Zig"}}
```bash
mkdir my-first-game
cd my-first-game
```

Copy `zx.h` from the Nethercore repository to your project folder.
{{#endtab}}

{{#endtabs}}

## Configure Your Build

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
panic = "abort"
```

Key settings:
- `crate-type = ["cdylib"]` - Builds a C-compatible dynamic library (required for WASM)
- `opt-level = "s"` - Optimize for small binary size
- `lto = true` - Link-time optimization for even smaller binaries
{{#endtab}}

{{#tab name="C/C++"}}
Create a `Makefile`:

```makefile
# Path to wasi-sdk (adjust for your system)
WASI_SDK_PATH ?= /opt/wasi-sdk

CC = $(WASI_SDK_PATH)/bin/clang
CFLAGS = -O2 -Wall -Wextra

# Export the three required functions
LDFLAGS = -Wl,--no-entry \
          -Wl,--export=init \
          -Wl,--export=update \
          -Wl,--export=render \
          -Wl,--allow-undefined

all: game.wasm

game.wasm: game.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $^

clean:
	rm -f game.wasm
```

Key settings:
- `--no-entry` - No main() function, we use init/update/render
- `--export=...` - Make our functions visible to the runtime
- `--allow-undefined` - FFI functions are provided by the runtime
{{#endtab}}

{{#tab name="Zig"}}
Create a `build.zig`:

```zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    const exe = b.addExecutable(.{
        .name = "game",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = .ReleaseSmall,
    });

    // Export the required functions
    exe.entry = .disabled;
    exe.rdynamic = true;

    b.installArtifact(exe);
}
```

Key settings:
- `.wasm32` + `.freestanding` - Compile to bare WASM
- `.ReleaseSmall` - Optimize for size
- `.rdynamic = true` - Export public functions
{{#endtab}}

{{#endtabs}}

## Write Your Game

{{#tabs global="lang"}}

{{#tab name="Rust"}}
Replace `src/lib.rs` with:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Panic handler required for no_std
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Use the version-matched SDK, not hand-maintained import signatures.
mod zx;
use zx::*;

// This Rust static resides in the snapshotted WASM linear memory
static mut SQUARE_Y: f32 = 200.0;

// Use the version-matched SDK's button indices.
use zx::button::{UP as BUTTON_UP, DOWN as BUTTON_DOWN, A as BUTTON_A};

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
        set_color(0xFFFFFFFF);
        let title = b"Hello Nethercore!";
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            80.0, 50.0, 32.0,
        );

        // Draw the moving square
        set_color(0xFF6B6BFF);
        draw_rect(200.0, SQUARE_Y, 80.0, 80.0);

        // Draw instructions
        set_color(0x888888FF);
        let hint = b"D-pad: Move   A: Reset";
        draw_text(
            hint.as_ptr(),
            hint.len() as u32,
            60.0, 500.0, 18.0,
        );
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
Create `game.c`:

```c
#include "zx.h"

/* Game state - stored in static variables for rollback safety */
static float square_y = 200.0f;

NCZX_EXPORT void init(void) {
    /* Set the background color (dark blue) */
    set_clear_color(0x1a1a2eFF);
}

NCZX_EXPORT void update(void) {
    /* Move square with D-pad */
    if (button_pressed(0, NCZX_BUTTON_UP)) {
        square_y -= 10.0f;
    }
    if (button_pressed(0, NCZX_BUTTON_DOWN)) {
        square_y += 10.0f;
    }

    /* Reset position with A button */
    if (button_pressed(0, NCZX_BUTTON_A)) {
        square_y = 200.0f;
    }

    /* Keep square on screen */
    square_y = nczx_clampf(square_y, 20.0f, 450.0f);
}

NCZX_EXPORT void render(void) {
    /* Draw title text */
    set_color(NCZX_COLOR_WHITE);
    NCZX_DRAW_TEXT("Hello Nethercore!", 80.0f, 50.0f, 32.0f);

    /* Draw the moving square */
    set_color(0xFF6B6BFF);
    draw_rect(200.0f, square_y, 80.0f, 80.0f);

    /* Draw instructions */
    set_color(0x888888FF);
    NCZX_DRAW_TEXT("D-pad: Move   A: Reset", 60.0f, 500.0f, 18.0f);
}
```

The header provides:
- `NCZX_EXPORT` - Marks functions for WASM export
- `NCZX_BUTTON_*` - Button constants
- `NCZX_DRAW_TEXT()` - Helper macro for string literals
- `nczx_clampf()` - Clamp float between min and max
{{#endtab}}

{{#tab name="Zig"}}
Create `src/main.zig`:

```zig
// FFI imports from the Nethercore runtime
extern fn set_clear_color(color: u32) void;
extern fn button_pressed(player: u32, button: u32) u32;
extern fn set_color(color: u32) void;
extern fn draw_rect(x: f32, y: f32, w: f32, h: f32) void;
extern fn draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32) void;

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

// Game state - stored in static variables for rollback safety
var square_y: f32 = 200.0;

export fn init() void {
    // Set the background color (dark blue)
    set_clear_color(0x1a1a2eFF);
}

export fn update() void {
    // Move square with D-pad
    if (button_pressed(0, BUTTON_UP) != 0) {
        square_y -= 10.0;
    }
    if (button_pressed(0, BUTTON_DOWN) != 0) {
        square_y += 10.0;
    }

    // Reset position with A button
    if (button_pressed(0, BUTTON_A) != 0) {
        square_y = 200.0;
    }

    // Keep square on screen
    square_y = @max(20.0, @min(square_y, 450.0));
}

export fn render() void {
    // Draw title text
    const title = "Hello Nethercore!";
    set_color(0xFFFFFFFF);
    draw_text(title.ptr, title.len, 80.0, 50.0, 32.0);

    // Draw the moving square
    set_color(0xFF6B6BFF);
    draw_rect(200.0, square_y, 80.0, 80.0);

    // Draw instructions
    const hint = "D-pad: Move   A: Reset";
    set_color(0x888888FF);
    draw_text(hint.ptr, hint.len, 60.0, 500.0, 18.0);
}
```

Zig's `export fn` automatically exports functions from the WASM module.
{{#endtab}}

{{#endtabs}}

## Understanding the Code

### No Standard Library

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#![no_std]
#![no_main]
```

Nethercore games run in a minimal WebAssembly environment without the Rust standard library. This keeps binaries small and avoids OS dependencies.
{{#endtab}}

{{#tab name="C/C++"}}
We compile with `--no-entry` and don't link libc. The header provides everything you need. For advanced use cases, you can optionally link wasi-libc.
{{#endtab}}

{{#tab name="Zig"}}
Zig compiles to freestanding WASM by default with `.os_tag = .freestanding`. The standard library's OS-specific parts are unavailable, but math and memory functions work.
{{#endtab}}

{{#endtabs}}

### FFI Imports

Functions are imported from the Nethercore runtime. See the [Cheat Sheet](../cheat-sheet.md) for all available functions.

### Static Game State

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut SQUARE_Y: f32 = 200.0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float square_y = 200.0f;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var square_y: f32 = 200.0;
```
{{#endtab}}

{{#endtabs}}

This example's mutable state resides in WASM linear memory, which the runtime snapshots for rollback. Ordinary Rust statics can live there; `static mut` itself does not defeat rollback. This is not a promise to capture arbitrary WASM globals, external state, or render-only host objects. Keep simulation changes in `update()`, not `render()`.

### Colors

Colors are 32-bit RGBA values in hexadecimal:
- `0xFFFFFFFF` = White (R=255, G=255, B=255, A=255)
- `0xFF6B6BFF` = Salmon red (R=255, G=107, B=107, A=255)
- `0x1a1a2eFF` = Dark blue (R=26, G=26, B=46, A=255)

## Build and Run

{{#tabs global="lang"}}

{{#tab name="Rust"}}
### Build the WASM file:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/my_first_game.wasm`

### Run in the Nethercore player:

```bash
nethercore-zx target/wasm32-unknown-unknown/release/my_first_game.wasm
```
{{#endtab}}

{{#tab name="C/C++"}}
### Build the WASM file:

```bash
make
```

Output: `game.wasm`

### Run in the Nethercore player:

```bash
nethercore-zx game.wasm
```
{{#endtab}}

{{#tab name="Zig"}}
### Build the WASM file:

```bash
zig build
```

Output: `zig-out/bin/game.wasm`

### Run in the Nethercore player:

```bash
nethercore-zx zig-out/bin/game.wasm
```
{{#endtab}}

{{#endtabs}}

Or load the `.wasm` file directly in the Nethercore Library application.

### Pack, edit and share the Rust game

In the project directory, create `nether.toml`:

```toml
[game]
id = "my-first-game"
title = "My First Game"
author = "Your Name"
version = "0.1.0"
max_players = 1
tick_rate = 60
render_mode = 0
compress_textures = true

[netplay]
enabled = false

[build]
wasm = "target/wasm32-unknown-unknown/release/my_first_game.wasm"
```

```powershell
nether pack
nether run --no-build
```

`nether run` takes a **project**, not a positional WASM filename. The standalone
player takes a positional `PATH` for an existing WASM or cartridge. Close the player window
before editing. Change `SQUARE_Y`'s initial value to `300.0`, then rebuild and run:

```powershell
nether build
nether run --no-build
```

Share the generated `my-first-game.nczx` beside `nether.toml`, not just its WASM. Your recipient
extracts the complete player bundle, changes to a directory outside the source
checkout, and launches it with an absolute cartridge path:

```powershell
& "$env:NETHERCORE_HOME\nethercore-zx.exe" "C:\Games\my-first-game.nczx"
```

Choose the actual paths on your machine. No source checkout, game compiler,
account, or hosted service is required by that player invocation.

## What You've Learned

- Setting up a project for WASM compilation
- The minimal/freestanding environment
- Importing FFI functions from the runtime
- The three lifecycle functions: `init()`, `update()`, `render()`
- Drawing 2D graphics with `draw_rect()` and `draw_text()`
- Handling input with `button_pressed()`
- Using static variables for game state

---

**Next:** [Understanding the Game Loop](./game-loop.md)
