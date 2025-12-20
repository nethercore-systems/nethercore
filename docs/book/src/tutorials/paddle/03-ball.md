# Part 3: Ball Physics

Time to make the ball move! We'll add velocity, wall bouncing, and paddle collision.

## What You'll Learn

- Ball velocity and movement
- Wall collision (top and bottom)
- Paddle collision with spin
- AABB collision detection

## Add Ball Velocity

Update your ball state to include velocity:

```rust
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
static mut BALL_VX: f32 = 0.0;  // Horizontal velocity
static mut BALL_VY: f32 = 0.0;  // Vertical velocity
```

Add ball speed constants:

```rust
const BALL_SPEED_INITIAL: f32 = 5.0;
const BALL_SPEED_MAX: f32 = 12.0;
const BALL_SPEED_INCREMENT: f32 = 0.5;
```

## Add Random FFI

We need randomness to vary the ball's starting angle:

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...
    fn random() -> u32;
}
```

## Reset Ball Function

Create a function to reset the ball position and give it a random velocity:

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

Update `init()` to use it:

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

## Ball Movement and Wall Bounce

Create an `update_ball()` function:

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

Call it from `update()`:

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

## Paddle Collision

Now add collision detection with the paddles. Update `update_ball()`:

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
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

The ball should now:
- Move across the screen
- Bounce off top and bottom walls
- Bounce off paddles with spin
- Reset when it goes off screen

---

**Next:** [Part 4: AI Opponent](./04-ai.md) - Add an AI for single-player mode.
