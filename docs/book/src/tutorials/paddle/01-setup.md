# Part 1: Setup & Drawing

In this part, you'll set up your Paddle project and draw the basic game elements: the court and paddles.

## What You'll Learn

- Creating a new Emberware game project
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

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// FFI imports from the Emberware runtime
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

## Define Constants

Add these constants after the FFI imports:

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

## Draw the Court

Let's draw a dashed center line. Update `render()`:

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

## Draw the Paddles

Add paddle state and drawing:

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

Update `render()` to draw the paddles:

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

## Draw the Ball

Add ball state:

```rust
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
```

Initialize it in `init()`:

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

Draw it in `render()` (add after paddles):

```rust
        // Draw ball
        draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
```

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

You should see:
- Dark blue background
- Dashed white center line
- Blue paddle on the left
- Red paddle on the right
- White ball in the center

## Complete Code So Far

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

---

**Next:** [Part 2: Paddle Movement](./02-paddles.md) — Make the paddles respond to input.
