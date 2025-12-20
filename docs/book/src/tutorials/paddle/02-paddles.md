# Part 2: Paddle Movement

Now let's make the paddles respond to player input.

## What You'll Learn

- Reading input with `button_held()` and `left_stick_y()`
- Clamping values to keep paddles on screen
- The difference between `button_pressed()` and `button_held()`

## Add Input FFI Functions

Update your FFI imports:

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

## Add Constants for Input

```rust
// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;

// Movement speed
const PADDLE_SPEED: f32 = 8.0;
```

## Implement Paddle Movement

Update the `update()` function:

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

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

Both paddles should now respond to input. Use:
- **Player 1:** Left stick or D-pad on controller 1
- **Player 2:** Left stick or D-pad on controller 2 (if connected)

If you only have one controller, player 2's paddle won't move yet - we'll add AI in Part 4.

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

---

**Next:** [Part 3: Ball Physics](./03-ball.md) - Make the ball move and bounce.
