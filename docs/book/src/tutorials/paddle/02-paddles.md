# Part 2: Paddle Movement

Now let's make the paddles respond to player input.

## What You'll Learn

- Reading input with `button_held()` and `left_stick_y()`
- Clamping values to keep paddles on screen
- The difference between `button_pressed()` and `button_held()`

## Add Input FFI Functions

Update your FFI imports:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);

    // Input functions
    fn left_stick_y(player: u32) -> f32;
    fn button_held(player: u32, button: u32) -> u32;
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void set_clear_color(uint32_t color);
EWZX_IMPORT void draw_rect(float x, float y, float w, float h, uint32_t color);

// Input functions
EWZX_IMPORT float left_stick_y(uint32_t player);
EWZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_clear_color(color: u32) void;
pub extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;

// Input functions
pub extern fn left_stick_y(player: u32) f32;
pub extern fn button_held(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

## Add Constants for Input

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;

// Movement speed
const PADDLE_SPEED: f32 = 8.0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Button constants
#define EWZX_BUTTON_UP 0
#define EWZX_BUTTON_DOWN 1

// Movement speed
#define PADDLE_SPEED 8.0f
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Button constants
const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
};

// Movement speed
const PADDLE_SPEED: f32 = 8.0;
```
{{#endtab}}

{{#endtabs}}

## Implement Paddle Movement

Update the `update()` function:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Player 1 (left paddle)
        update_paddle(&mut PADDLE1_Y, 0);

        // Player 2 (right paddle) - we'll add AI later
        update_paddle(&mut PADDLE2_Y, 1);
    }
}

fn update_paddle(paddle_y: &mut f32, player: u32) {
    unsafe {
        // Read analog stick (Y axis is inverted: up is negative)
        let stick_y = left_stick_y(player);

        // Read D-pad buttons
        let up = button_held(player, BUTTON_UP) != 0;
        let down = button_held(player, BUTTON_DOWN) != 0;

        // Calculate movement
        let mut movement = -stick_y * PADDLE_SPEED;  // Invert stick

        if up {
            movement -= PADDLE_SPEED;
        }
        if down {
            movement += PADDLE_SPEED;
        }

        // Apply movement
        *paddle_y += movement;

        // Clamp to screen bounds
        *paddle_y = clamp(*paddle_y, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
    }
}

// Helper function
fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void update_paddle(float *paddle_y, uint32_t player) {
    // Read analog stick (Y axis is inverted: up is negative)
    float stick_y = left_stick_y(player);

    // Read D-pad buttons
    bool up = button_held(player, EWZX_BUTTON_UP) != 0;
    bool down = button_held(player, EWZX_BUTTON_DOWN) != 0;

    // Calculate movement
    float movement = -stick_y * PADDLE_SPEED;  // Invert stick

    if (up) {
        movement -= PADDLE_SPEED;
    }
    if (down) {
        movement += PADDLE_SPEED;
    }

    // Apply movement
    *paddle_y += movement;

    // Clamp to screen bounds
    *paddle_y = clamp(*paddle_y, 0.0f, SCREEN_HEIGHT - PADDLE_HEIGHT);
}

EWZX_EXPORT void update() {
    // Player 1 (left paddle)
    update_paddle(&paddle1_y, 0);

    // Player 2 (right paddle) - we'll add AI later
    update_paddle(&paddle2_y, 1);
}

// Helper function
float clamp(float v, float min, float max) {
    if (v < min) return min;
    if (v > max) return max;
    return v;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn update_paddle(paddle_y: *f32, player: u32) void {
    // Read analog stick (Y axis is inverted: up is negative)
    const stick_y = left_stick_y(player);

    // Read D-pad buttons
    const up = button_held(player, Button.up) != 0;
    const down = button_held(player, Button.down) != 0;

    // Calculate movement
    var movement = -stick_y * PADDLE_SPEED;  // Invert stick

    if (up) {
        movement -= PADDLE_SPEED;
    }
    if (down) {
        movement += PADDLE_SPEED;
    }

    // Apply movement
    paddle_y.* += movement;

    // Clamp to screen bounds
    paddle_y.* = clamp(paddle_y.*, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
}

export fn update() void {
    // Player 1 (left paddle)
    update_paddle(&paddle1_y, 0);

    // Player 2 (right paddle) - we'll add AI later
    update_paddle(&paddle2_y, 1);
}

// Helper function
fn clamp(v: f32, min: f32, max: f32) f32 {
    if (v < min) return min;
    if (v > max) return max;
    return v;
}
```
{{#endtab}}

{{#endtabs}}

## Understanding Input

### Analog Stick

`left_stick_y(player)` returns a value from -1.0 to 1.0:
- **-1.0** = stick pushed fully up
- **0.0** = stick at center
- **1.0** = stick pushed fully down

We invert this because screen Y coordinates increase downward.

### D-Pad Buttons

There are two ways to read buttons:

| Function | Behavior |
|----------|----------|
| `button_pressed(player, button)` | Returns `1` only on the frame the button is first pressed |
| `button_held(player, button)` | Returns `1` every frame the button is held down |

For continuous movement like paddles, use `button_held()`.

### Button Constants

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;
const BUTTON_LB: u32 = 8;
const BUTTON_RB: u32 = 9;
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define EWZX_BUTTON_UP 0
#define EWZX_BUTTON_DOWN 1
#define EWZX_BUTTON_LEFT 2
#define EWZX_BUTTON_RIGHT 3
#define EWZX_BUTTON_A 4
#define EWZX_BUTTON_B 5
#define EWZX_BUTTON_X 6
#define EWZX_BUTTON_Y 7
#define EWZX_BUTTON_LB 8
#define EWZX_BUTTON_RB 9
#define EWZX_BUTTON_START 12
#define EWZX_BUTTON_SELECT 13
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;
    pub const a: u32 = 4;
    pub const b: u32 = 5;
    pub const x: u32 = 6;
    pub const y: u32 = 7;
    pub const lb: u32 = 8;
    pub const rb: u32 = 9;
    pub const start: u32 = 12;
    pub const select: u32 = 13;
};
```
{{#endtab}}

{{#endtabs}}

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

Both paddles should now respond to input. Use:
- **Player 1:** Left stick or D-pad on controller 1
- **Player 2:** Left stick or D-pad on controller 2 (if connected)

If you only have one controller, player 2's paddle won't move yet - we'll add AI in Part 4.

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
    fn left_stick_y(player: u32) -> f32;
    fn button_held(player: u32, button: u32) -> u32;
}

const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;
const PADDLE_SPEED: f32 = 8.0;
const BALL_SIZE: f32 = 15.0;

const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;

const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;
const COLOR_PLAYER2: u32 = 0xff6b6bFF;

static mut PADDLE1_Y: f32 = 0.0;
static mut PADDLE2_Y: f32 = 0.0;
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

fn update_paddle(paddle_y: &mut f32, player: u32) {
    unsafe {
        let stick_y = left_stick_y(player);
        let up = button_held(player, BUTTON_UP) != 0;
        let down = button_held(player, BUTTON_DOWN) != 0;

        let mut movement = -stick_y * PADDLE_SPEED;
        if up { movement -= PADDLE_SPEED; }
        if down { movement += PADDLE_SPEED; }

        *paddle_y += movement;
        *paddle_y = clamp(*paddle_y, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
    }
}

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
pub extern "C" fn update() {
    unsafe {
        update_paddle(&mut PADDLE1_Y, 0);
        update_paddle(&mut PADDLE2_Y, 1);
    }
}

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
#include <stdbool.h>

EWZX_IMPORT void set_clear_color(uint32_t color);
EWZX_IMPORT void draw_rect(float x, float y, float w, float h, uint32_t color);
EWZX_IMPORT float left_stick_y(uint32_t player);
EWZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);

#define SCREEN_WIDTH 960.0f
#define SCREEN_HEIGHT 540.0f
#define PADDLE_WIDTH 15.0f
#define PADDLE_HEIGHT 80.0f
#define PADDLE_MARGIN 30.0f
#define PADDLE_SPEED 8.0f
#define BALL_SIZE 15.0f

#define EWZX_BUTTON_UP 0
#define EWZX_BUTTON_DOWN 1

#define COLOR_WHITE 0xFFFFFFFF
#define COLOR_GRAY 0x666666FF
#define COLOR_PLAYER1 0x4a9fffFF
#define COLOR_PLAYER2 0xff6b6bFF

static float paddle1_y = 0.0f;
static float paddle2_y = 0.0f;
static float ball_x = 0.0f;
static float ball_y = 0.0f;

float clamp(float v, float min, float max) {
    if (v < min) return min;
    if (v > max) return max;
    return v;
}

void update_paddle(float *paddle_y, uint32_t player) {
    float stick_y = left_stick_y(player);
    bool up = button_held(player, EWZX_BUTTON_UP) != 0;
    bool down = button_held(player, EWZX_BUTTON_DOWN) != 0;

    float movement = -stick_y * PADDLE_SPEED;
    if (up) movement -= PADDLE_SPEED;
    if (down) movement += PADDLE_SPEED;

    *paddle_y += movement;
    *paddle_y = clamp(*paddle_y, 0.0f, SCREEN_HEIGHT - PADDLE_HEIGHT);
}

EWZX_EXPORT void init() {
    set_clear_color(0x1a1a2eFF);
    paddle1_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    paddle2_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    ball_x = SCREEN_WIDTH / 2.0f - BALL_SIZE / 2.0f;
    ball_y = SCREEN_HEIGHT / 2.0f - BALL_SIZE / 2.0f;
}

EWZX_EXPORT void update() {
    update_paddle(&paddle1_y, 0);
    update_paddle(&paddle2_y, 1);
}

EWZX_EXPORT void render() {
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
const std = @import("std");

pub extern fn set_clear_color(color: u32) void;
pub extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;
pub extern fn left_stick_y(player: u32) f32;
pub extern fn button_held(player: u32, button: u32) u32;

const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;
const PADDLE_SPEED: f32 = 8.0;
const BALL_SIZE: f32 = 15.0;

const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
};

const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF;
const COLOR_PLAYER2: u32 = 0xff6b6bFF;

var paddle1_y: f32 = 0.0;
var paddle2_y: f32 = 0.0;
var ball_x: f32 = 0.0;
var ball_y: f32 = 0.0;

fn clamp(v: f32, min: f32, max: f32) f32 {
    if (v < min) return min;
    if (v > max) return max;
    return v;
}

fn update_paddle(paddle_y: *f32, player: u32) void {
    const stick_y = left_stick_y(player);
    const up = button_held(player, Button.up) != 0;
    const down = button_held(player, Button.down) != 0;

    var movement = -stick_y * PADDLE_SPEED;
    if (up) movement -= PADDLE_SPEED;
    if (down) movement += PADDLE_SPEED;

    paddle_y.* += movement;
    paddle_y.* = clamp(paddle_y.*, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
}

export fn init() void {
    set_clear_color(0x1a1a2eFF);
    paddle1_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    paddle2_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    ball_x = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
    ball_y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
}

export fn update() void {
    update_paddle(&paddle1_y, 0);
    update_paddle(&paddle2_y, 1);
}

export fn render() void {
    // Center line
    const dash_height: f32 = 20.0;
    const dash_gap: f32 = 15.0;
    const dash_width: f32 = 4.0;
    const center_x: f32 = SCREEN_WIDTH / 2.0 - dash_width / 2.0;
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

**Next:** [Part 3: Ball Physics](./03-ball.md) - Make the ball move and bounce.
