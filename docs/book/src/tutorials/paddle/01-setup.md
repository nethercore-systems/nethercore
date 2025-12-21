# Part 1: Setup & Drawing

In this part, you'll set up your Paddle project and draw the basic game elements: the court and paddles.

## What You'll Learn

- Creating a new Nethercore game project
- Importing FFI functions
- Drawing rectangles with `draw_rect()`
- Using colors in RGBA hex format

## Create the Project

```bash
cargo new --lib paddle
cd paddle
```

## Configure Cargo.toml

```toml
[package]
name = "paddle"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
libm = "0.2"

[profile.release]
opt-level = "s"
lto = true
```

We include `libm` for math functions like `sqrt()` that we'll need later.

## Write the Basic Structure

Create `src/lib.rs`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// FFI imports from the Nethercore runtime
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1a1a2eFF);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    // Game logic will go here
}

#[no_mangle]
pub extern "C" fn render() {
    // Drawing will go here
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <stdint.h>

// FFI imports from the Nethercore runtime
EWZX_IMPORT void set_clear_color(uint32_t color);
EWZX_IMPORT void draw_rect(float x, float y, float w, float h, uint32_t color);

EWZX_EXPORT void init(void) {
    // Dark background
    set_clear_color(0x1a1a2eFF);
}

EWZX_EXPORT void update(void) {
    // Game logic will go here
}

EWZX_EXPORT void render(void) {
    // Drawing will go here
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// FFI imports from the Nethercore runtime
extern fn set_clear_color(color: u32) void;
extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;

export fn init() void {
    // Dark background
    set_clear_color(0x1a1a2eFF);
}

export fn update() void {
    // Game logic will go here
}

export fn render() void {
    // Drawing will go here
}
```
{{#endtab}}

{{#endtabs}}

## Define Constants

Add these constants after the FFI imports:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Screen dimensions (540p default resolution)
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Paddle dimensions
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;  // Distance from edge

// Ball size
const BALL_SIZE: f32 = 15.0;

// Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;  // Blue
const COLOR_PLAYER2: u32 = 0xff6b6bFF;  // Red
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Screen dimensions (540p default resolution)
#define SCREEN_WIDTH 960.0f
#define SCREEN_HEIGHT 540.0f

// Paddle dimensions
#define PADDLE_WIDTH 15.0f
#define PADDLE_HEIGHT 80.0f
#define PADDLE_MARGIN 30.0f  // Distance from edge

// Ball size
#define BALL_SIZE 15.0f

// Colors
#define COLOR_WHITE 0xFFFFFFFF
#define COLOR_GRAY 0x666666FF
#define COLOR_PLAYER1 0x4a9fffFF  // Blue
#define COLOR_PLAYER2 0xff6b6bFF  // Red
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Screen dimensions (540p default resolution)
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Paddle dimensions
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;  // Distance from edge

// Ball size
const BALL_SIZE: f32 = 15.0;

// Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;  // Blue
const COLOR_PLAYER2: u32 = 0xff6b6bFF;  // Red
```
{{#endtab}}

{{#endtabs}}

## Draw the Court

Let's draw a dashed center line. Update `render()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw center line (dashed)
        let dash_height = 20.0;
        let dash_gap = 15.0;
        let dash_width = 4.0;
        let center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;

        let mut y = 10.0;
        while y < SCREEN_HEIGHT - 10.0 {
            draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
            y += dash_height + dash_gap;
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Draw center line (dashed)
    float dash_height = 20.0f;
    float dash_gap = 15.0f;
    float dash_width = 4.0f;
    float center_x = SCREEN_WIDTH / 2.0f - dash_width / 2.0f;

    float y = 10.0f;
    while (y < SCREEN_HEIGHT - 10.0f) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw center line (dashed)
    const dash_height = 20.0;
    const dash_gap = 15.0;
    const dash_width = 4.0;
    const center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;

    var y: f32 = 10.0;
    while (y < SCREEN_HEIGHT - 10.0) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }
}
```
{{#endtab}}

{{#endtabs}}

## Draw the Paddles

Add paddle state and drawing:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Add after constants
static mut PADDLE1_Y: f32 = 0.0;
static mut PADDLE2_Y: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Center paddles vertically
        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Add after constants
static float paddle1_y = 0.0f;
static float paddle2_y = 0.0f;

EWZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF);

    // Center paddles vertically
    paddle1_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    paddle2_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Add after constants
var paddle1_y: f32 = 0.0;
var paddle2_y: f32 = 0.0;

export fn init() void {
    set_clear_color(0x1a1a2eFF);

    // Center paddles vertically
    paddle1_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    paddle2_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
}
```
{{#endtab}}

{{#endtabs}}

Update `render()` to draw the paddles:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw center line (dashed)
        let dash_height = 20.0;
        let dash_gap = 15.0;
        let dash_width = 4.0;
        let center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;

        let mut y = 10.0;
        while y < SCREEN_HEIGHT - 10.0 {
            draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
            y += dash_height + dash_gap;
        }

        // Draw paddle 1 (left, blue)
        draw_rect(
            PADDLE_MARGIN,
            PADDLE1_Y,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
            COLOR_PLAYER1,
        );

        // Draw paddle 2 (right, red)
        draw_rect(
            SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH,
            PADDLE2_Y,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
            COLOR_PLAYER2,
        );
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Draw center line (dashed)
    float dash_height = 20.0f;
    float dash_gap = 15.0f;
    float dash_width = 4.0f;
    float center_x = SCREEN_WIDTH / 2.0f - dash_width / 2.0f;

    float y = 10.0f;
    while (y < SCREEN_HEIGHT - 10.0f) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }

    // Draw paddle 1 (left, blue)
    draw_rect(
        PADDLE_MARGIN,
        paddle1_y,
        PADDLE_WIDTH,
        PADDLE_HEIGHT,
        COLOR_PLAYER1
    );

    // Draw paddle 2 (right, red)
    draw_rect(
        SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH,
        paddle2_y,
        PADDLE_WIDTH,
        PADDLE_HEIGHT,
        COLOR_PLAYER2
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw center line (dashed)
    const dash_height = 20.0;
    const dash_gap = 15.0;
    const dash_width = 4.0;
    const center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;

    var y: f32 = 10.0;
    while (y < SCREEN_HEIGHT - 10.0) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }

    // Draw paddle 1 (left, blue)
    draw_rect(
        PADDLE_MARGIN,
        paddle1_y,
        PADDLE_WIDTH,
        PADDLE_HEIGHT,
        COLOR_PLAYER1,
    );

    // Draw paddle 2 (right, red)
    draw_rect(
        SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH,
        paddle2_y,
        PADDLE_WIDTH,
        PADDLE_HEIGHT,
        COLOR_PLAYER2,
    );
}
```
{{#endtab}}

{{#endtabs}}

## Draw the Ball

Add ball state:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float ball_x = 0.0f;
static float ball_y = 0.0f;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var ball_x: f32 = 0.0;
var ball_y: f32 = 0.0;
```
{{#endtab}}

{{#endtabs}}

Initialize it in `init()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

        // Center the ball
        BALL_X = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
        BALL_Y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF);

    paddle1_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    paddle2_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;

    // Center the ball
    ball_x = SCREEN_WIDTH / 2.0f - BALL_SIZE / 2.0f;
    ball_y = SCREEN_HEIGHT / 2.0f - BALL_SIZE / 2.0f;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_clear_color(0x1a1a2eFF);

    paddle1_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    paddle2_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

    // Center the ball
    ball_x = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
    ball_y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
}
```
{{#endtab}}

{{#endtabs}}

Draw it in `render()` (add after paddles):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
        // Draw ball
        draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
    // Draw ball
    draw_rect(ball_x, ball_y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
    // Draw ball
    draw_rect(ball_x, ball_y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
```
{{#endtab}}

{{#endtabs}}

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

You should see:
- Dark blue background
- Dashed white center line
- Blue paddle on the left
- Red paddle on the right
- White ball in the center

## Complete Code So Far

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;
const BALL_SIZE: f32 = 15.0;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;
const COLOR_PLAYER2: u32 = 0xff6b6bFF;

static mut PADDLE1_Y: f32 = 0.0;
static mut PADDLE2_Y: f32 = 0.0;
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        BALL_X = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
        BALL_Y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
    }
}

#[no_mangle]
pub extern "C" fn update() {}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Center line
        let dash_height = 20.0;
        let dash_gap = 15.0;
        let dash_width = 4.0;
        let center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;
        let mut y = 10.0;
        while y < SCREEN_HEIGHT - 10.0 {
            draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
            y += dash_height + dash_gap;
        }

        // Paddles
        draw_rect(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
        draw_rect(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, PADDLE2_Y,
                  PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER2);

        // Ball
        draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <stdint.h>

// FFI imports
EWZX_IMPORT void set_clear_color(uint32_t color);
EWZX_IMPORT void draw_rect(float x, float y, float w, float h, uint32_t color);

// Constants
#define SCREEN_WIDTH 960.0f
#define SCREEN_HEIGHT 540.0f
#define PADDLE_WIDTH 15.0f
#define PADDLE_HEIGHT 80.0f
#define PADDLE_MARGIN 30.0f
#define BALL_SIZE 15.0f
#define COLOR_WHITE 0xFFFFFFFF
#define COLOR_GRAY 0x666666FF
#define COLOR_PLAYER1 0x4a9fffFF
#define COLOR_PLAYER2 0xff6b6bFF

// State
static float paddle1_y = 0.0f;
static float paddle2_y = 0.0f;
static float ball_x = 0.0f;
static float ball_y = 0.0f;

EWZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF);
    paddle1_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    paddle2_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    ball_x = SCREEN_WIDTH / 2.0f - BALL_SIZE / 2.0f;
    ball_y = SCREEN_HEIGHT / 2.0f - BALL_SIZE / 2.0f;
}

EWZX_EXPORT void update(void) {}

EWZX_EXPORT void render(void) {
    // Center line
    float dash_height = 20.0f;
    float dash_gap = 15.0f;
    float dash_width = 4.0f;
    float center_x = SCREEN_WIDTH / 2.0f - dash_width / 2.0f;
    float y = 10.0f;
    while (y < SCREEN_HEIGHT - 10.0f) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }

    // Paddles
    draw_rect(PADDLE_MARGIN, paddle1_y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
    draw_rect(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, paddle2_y,
              PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER2);

    // Ball
    draw_rect(ball_x, ball_y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// FFI imports
extern fn set_clear_color(color: u32) void;
extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;

// Constants
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;
const BALL_SIZE: f32 = 15.0;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;
const COLOR_PLAYER2: u32 = 0xff6b6bFF;

// State
var paddle1_y: f32 = 0.0;
var paddle2_y: f32 = 0.0;
var ball_x: f32 = 0.0;
var ball_y: f32 = 0.0;

export fn init() void {
    set_clear_color(0x1a1a2eFF);
    paddle1_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    paddle2_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    ball_x = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
    ball_y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
}

export fn update() void {}

export fn render() void {
    // Center line
    const dash_height = 20.0;
    const dash_gap = 15.0;
    const dash_width = 4.0;
    const center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;
    var y: f32 = 10.0;
    while (y < SCREEN_HEIGHT - 10.0) {
        draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
        y += dash_height + dash_gap;
    }

    // Paddles
    draw_rect(PADDLE_MARGIN, paddle1_y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
    draw_rect(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, paddle2_y,
              PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER2);

    // Ball
    draw_rect(ball_x, ball_y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
}
```
{{#endtab}}

{{#endtabs}}

---

**Next:** [Part 2: Paddle Movement](./02-paddles.md) — Make the paddles respond to input.
