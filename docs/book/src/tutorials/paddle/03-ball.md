# Part 3: Ball Physics

Time to make the ball move! We'll add velocity, wall bouncing, and paddle collision.

## What You'll Learn

- Ball velocity and movement
- Wall collision (top and bottom)
- Paddle collision with spin
- AABB collision detection

## Add Ball Velocity

Update your ball state to include velocity:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
static mut BALL_VX: f32 = 0.0;  // Horizontal velocity
static mut BALL_VY: f32 = 0.0;  // Vertical velocity
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float BALL_X = 0.0f;
static float BALL_Y = 0.0f;
static float BALL_VX = 0.0f;  // Horizontal velocity
static float BALL_VY = 0.0f;  // Vertical velocity
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var BALL_X: f32 = 0.0;
var BALL_Y: f32 = 0.0;
var BALL_VX: f32 = 0.0;  // Horizontal velocity
var BALL_VY: f32 = 0.0;  // Vertical velocity
```
{{#endtab}}

{{#endtabs}}

Add ball speed constants:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const BALL_SPEED_INITIAL: f32 = 5.0;
const BALL_SPEED_MAX: f32 = 12.0;
const BALL_SPEED_INCREMENT: f32 = 0.5;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define BALL_SPEED_INITIAL 5.0f
#define BALL_SPEED_MAX 12.0f
#define BALL_SPEED_INCREMENT 0.5f
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const BALL_SPEED_INITIAL: f32 = 5.0;
const BALL_SPEED_MAX: f32 = 12.0;
const BALL_SPEED_INCREMENT: f32 = 0.5;
```
{{#endtab}}

{{#endtabs}}

## Add Random FFI

We need randomness to vary the ball's starting angle:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...
    fn random() -> u32;
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Add to imports section
EWZX_IMPORT uint32_t random_u32(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Add to imports section
pub extern fn random_u32() u32;
```
{{#endtab}}

{{#endtabs}}

## Reset Ball Function

Create a function to reset the ball position and give it a random velocity:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn reset_ball(direction: i32) {
    unsafe {
        // Center the ball
        BALL_X = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
        BALL_Y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;

        // Random vertical angle (-0.25 to 0.25)
        let rand = random() % 100;
        let angle = ((rand as f32 / 100.0) - 0.5) * 0.5;

        // Set velocity
        BALL_VX = BALL_SPEED_INITIAL * direction as f32;
        BALL_VY = BALL_SPEED_INITIAL * angle;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void reset_ball(int32_t direction) {
    // Center the ball
    BALL_X = SCREEN_WIDTH / 2.0f - BALL_SIZE / 2.0f;
    BALL_Y = SCREEN_HEIGHT / 2.0f - BALL_SIZE / 2.0f;

    // Random vertical angle (-0.25 to 0.25)
    uint32_t rand = random_u32() % 100;
    float angle = ((rand / 100.0f) - 0.5f) * 0.5f;

    // Set velocity
    BALL_VX = BALL_SPEED_INITIAL * (float)direction;
    BALL_VY = BALL_SPEED_INITIAL * angle;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn reset_ball(direction: i32) void {
    // Center the ball
    BALL_X = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
    BALL_Y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;

    // Random vertical angle (-0.25 to 0.25)
    const rand = random_u32() % 100;
    const angle = ((@as(f32, @floatFromInt(rand)) / 100.0) - 0.5) * 0.5;

    // Set velocity
    BALL_VX = BALL_SPEED_INITIAL * @as(f32, @floatFromInt(direction));
    BALL_VY = BALL_SPEED_INITIAL * angle;
}
```
{{#endtab}}

{{#endtabs}}

Update `init()` to use it:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

        reset_ball(-1);  // Start moving toward player 1
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF);
    PADDLE1_Y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    PADDLE2_Y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;

    reset_ball(-1);  // Start moving toward player 1
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_clear_color(0x1a1a2eFF);
    PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

    reset_ball(-1);  // Start moving toward player 1
}
```
{{#endtab}}

{{#endtabs}}

## Ball Movement and Wall Bounce

Create an `update_ball()` function:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update_ball() {
    unsafe {
        // Move ball
        BALL_X += BALL_VX;
        BALL_Y += BALL_VY;

        // Bounce off top wall
        if BALL_Y <= 0.0 {
            BALL_Y = 0.0;
            BALL_VY = -BALL_VY;
        }

        // Bounce off bottom wall
        if BALL_Y >= SCREEN_HEIGHT - BALL_SIZE {
            BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
            BALL_VY = -BALL_VY;
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void update_ball(void) {
    // Move ball
    BALL_X += BALL_VX;
    BALL_Y += BALL_VY;

    // Bounce off top wall
    if (BALL_Y <= 0.0f) {
        BALL_Y = 0.0f;
        BALL_VY = -BALL_VY;
    }

    // Bounce off bottom wall
    if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
        BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
        BALL_VY = -BALL_VY;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn update_ball() void {
    // Move ball
    BALL_X += BALL_VX;
    BALL_Y += BALL_VY;

    // Bounce off top wall
    if (BALL_Y <= 0.0) {
        BALL_Y = 0.0;
        BALL_VY = -BALL_VY;
    }

    // Bounce off bottom wall
    if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
        BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
        BALL_VY = -BALL_VY;
    }
}
```
{{#endtab}}

{{#endtabs}}

Call it from `update()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        update_paddle(&mut PADDLE1_Y, 0);
        update_paddle(&mut PADDLE2_Y, 1);
        update_ball();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    update_paddle(&PADDLE1_Y, 0);
    update_paddle(&PADDLE2_Y, 1);
    update_ball();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    update_paddle(&PADDLE1_Y, 0);
    update_paddle(&PADDLE2_Y, 1);
    update_ball();
}
```
{{#endtab}}

{{#endtabs}}

## Paddle Collision

Now add collision detection with the paddles. Update `update_ball()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update_ball() {
    unsafe {
        // Move ball
        BALL_X += BALL_VX;
        BALL_Y += BALL_VY;

        // Bounce off top wall
        if BALL_Y <= 0.0 {
            BALL_Y = 0.0;
            BALL_VY = -BALL_VY;
        }

        // Bounce off bottom wall
        if BALL_Y >= SCREEN_HEIGHT - BALL_SIZE {
            BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
            BALL_VY = -BALL_VY;
        }

        // Paddle 1 (left) collision
        if BALL_VX < 0.0 {  // Ball moving left
            let paddle_x = PADDLE_MARGIN;
            let paddle_right = paddle_x + PADDLE_WIDTH;

            if BALL_X <= paddle_right
                && BALL_X + BALL_SIZE >= paddle_x
                && BALL_Y + BALL_SIZE >= PADDLE1_Y
                && BALL_Y <= PADDLE1_Y + PADDLE_HEIGHT
            {
                // Bounce
                BALL_X = paddle_right;
                BALL_VX = -BALL_VX;

                // Add spin based on where ball hit paddle
                let paddle_center = PADDLE1_Y + PADDLE_HEIGHT / 2.0;
                let ball_center = BALL_Y + BALL_SIZE / 2.0;
                let offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
                BALL_VY += offset * 2.0;

                // Speed up (makes game more exciting)
                speed_up_ball();
            }
        }

        // Paddle 2 (right) collision
        if BALL_VX > 0.0 {  // Ball moving right
            let paddle_x = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH;

            if BALL_X + BALL_SIZE >= paddle_x
                && BALL_X <= paddle_x + PADDLE_WIDTH
                && BALL_Y + BALL_SIZE >= PADDLE2_Y
                && BALL_Y <= PADDLE2_Y + PADDLE_HEIGHT
            {
                // Bounce
                BALL_X = paddle_x - BALL_SIZE;
                BALL_VX = -BALL_VX;

                // Add spin
                let paddle_center = PADDLE2_Y + PADDLE_HEIGHT / 2.0;
                let ball_center = BALL_Y + BALL_SIZE / 2.0;
                let offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
                BALL_VY += offset * 2.0;

                speed_up_ball();
            }
        }

        // Ball goes off screen (scoring - we'll handle this properly later)
        if BALL_X < -BALL_SIZE || BALL_X > SCREEN_WIDTH {
            reset_ball(if BALL_X < 0.0 { 1 } else { -1 });
        }
    }
}

fn speed_up_ball() {
    unsafe {
        let speed = libm::sqrtf(BALL_VX * BALL_VX + BALL_VY * BALL_VY);
        if speed < BALL_SPEED_MAX {
            let factor = (speed + BALL_SPEED_INCREMENT) / speed;
            BALL_VX *= factor;
            BALL_VY *= factor;
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void update_ball(void) {
    // Move ball
    BALL_X += BALL_VX;
    BALL_Y += BALL_VY;

    // Bounce off top wall
    if (BALL_Y <= 0.0f) {
        BALL_Y = 0.0f;
        BALL_VY = -BALL_VY;
    }

    // Bounce off bottom wall
    if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
        BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
        BALL_VY = -BALL_VY;
    }

    // Paddle 1 (left) collision
    if (BALL_VX < 0.0f) {  // Ball moving left
        float paddle_x = PADDLE_MARGIN;
        float paddle_right = paddle_x + PADDLE_WIDTH;

        if (BALL_X <= paddle_right
            && BALL_X + BALL_SIZE >= paddle_x
            && BALL_Y + BALL_SIZE >= PADDLE1_Y
            && BALL_Y <= PADDLE1_Y + PADDLE_HEIGHT)
        {
            // Bounce
            BALL_X = paddle_right;
            BALL_VX = -BALL_VX;

            // Add spin based on where ball hit paddle
            float paddle_center = PADDLE1_Y + PADDLE_HEIGHT / 2.0f;
            float ball_center = BALL_Y + BALL_SIZE / 2.0f;
            float offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0f);
            BALL_VY += offset * 2.0f;

            // Speed up (makes game more exciting)
            speed_up_ball();
        }
    }

    // Paddle 2 (right) collision
    if (BALL_VX > 0.0f) {  // Ball moving right
        float paddle_x = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH;

        if (BALL_X + BALL_SIZE >= paddle_x
            && BALL_X <= paddle_x + PADDLE_WIDTH
            && BALL_Y + BALL_SIZE >= PADDLE2_Y
            && BALL_Y <= PADDLE2_Y + PADDLE_HEIGHT)
        {
            // Bounce
            BALL_X = paddle_x - BALL_SIZE;
            BALL_VX = -BALL_VX;

            // Add spin
            float paddle_center = PADDLE2_Y + PADDLE_HEIGHT / 2.0f;
            float ball_center = BALL_Y + BALL_SIZE / 2.0f;
            float offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0f);
            BALL_VY += offset * 2.0f;

            speed_up_ball();
        }
    }

    // Ball goes off screen (scoring - we'll handle this properly later)
    if (BALL_X < -BALL_SIZE || BALL_X > SCREEN_WIDTH) {
        reset_ball(BALL_X < 0.0f ? 1 : -1);
    }
}

void speed_up_ball(void) {
    float speed = sqrtf(BALL_VX * BALL_VX + BALL_VY * BALL_VY);
    if (speed < BALL_SPEED_MAX) {
        float factor = (speed + BALL_SPEED_INCREMENT) / speed;
        BALL_VX *= factor;
        BALL_VY *= factor;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn update_ball() void {
    // Move ball
    BALL_X += BALL_VX;
    BALL_Y += BALL_VY;

    // Bounce off top wall
    if (BALL_Y <= 0.0) {
        BALL_Y = 0.0;
        BALL_VY = -BALL_VY;
    }

    // Bounce off bottom wall
    if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
        BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
        BALL_VY = -BALL_VY;
    }

    // Paddle 1 (left) collision
    if (BALL_VX < 0.0) {  // Ball moving left
        const paddle_x = PADDLE_MARGIN;
        const paddle_right = paddle_x + PADDLE_WIDTH;

        if (BALL_X <= paddle_right and
            BALL_X + BALL_SIZE >= paddle_x and
            BALL_Y + BALL_SIZE >= PADDLE1_Y and
            BALL_Y <= PADDLE1_Y + PADDLE_HEIGHT)
        {
            // Bounce
            BALL_X = paddle_right;
            BALL_VX = -BALL_VX;

            // Add spin based on where ball hit paddle
            const paddle_center = PADDLE1_Y + PADDLE_HEIGHT / 2.0;
            const ball_center = BALL_Y + BALL_SIZE / 2.0;
            const offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
            BALL_VY += offset * 2.0;

            // Speed up (makes game more exciting)
            speed_up_ball();
        }
    }

    // Paddle 2 (right) collision
    if (BALL_VX > 0.0) {  // Ball moving right
        const paddle_x = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH;

        if (BALL_X + BALL_SIZE >= paddle_x and
            BALL_X <= paddle_x + PADDLE_WIDTH and
            BALL_Y + BALL_SIZE >= PADDLE2_Y and
            BALL_Y <= PADDLE2_Y + PADDLE_HEIGHT)
        {
            // Bounce
            BALL_X = paddle_x - BALL_SIZE;
            BALL_VX = -BALL_VX;

            // Add spin
            const paddle_center = PADDLE2_Y + PADDLE_HEIGHT / 2.0;
            const ball_center = BALL_Y + BALL_SIZE / 2.0;
            const offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
            BALL_VY += offset * 2.0;

            speed_up_ball();
        }
    }

    // Ball goes off screen (scoring - we'll handle this properly later)
    if (BALL_X < -BALL_SIZE or BALL_X > SCREEN_WIDTH) {
        reset_ball(if (BALL_X < 0.0) 1 else -1);
    }
}

fn speed_up_ball() void {
    const speed = @sqrt(BALL_VX * BALL_VX + BALL_VY * BALL_VY);
    if (speed < BALL_SPEED_MAX) {
        const factor = (speed + BALL_SPEED_INCREMENT) / speed;
        BALL_VX *= factor;
        BALL_VY *= factor;
    }
}
```
{{#endtab}}

{{#endtabs}}

## Understanding the Collision

### AABB (Axis-Aligned Bounding Box)

The collision check uses AABB overlap testing:

```
Ball overlaps Paddle if:
  ball.left < paddle.right  AND
  ball.right > paddle.left  AND
  ball.top < paddle.bottom  AND
  ball.bottom > paddle.top
```

### Spin System

When the ball hits the paddle:
- Hit the **top** of the paddle → ball goes **up** more
- Hit the **center** → ball goes **straight**
- Hit the **bottom** → ball goes **down** more

This gives players control over the ball direction.

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

The ball should now:
- Move across the screen
- Bounce off top and bottom walls
- Bounce off paddles with spin
- Reset when it goes off screen

---

**Next:** [Part 4: AI Opponent](./04-ai.md) - Add an AI for single-player mode.
