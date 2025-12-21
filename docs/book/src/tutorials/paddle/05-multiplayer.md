# Part 5: Multiplayer

This is where Emberware's magic happens. Our Paddle game already supports online multiplayer - and we didn't write any networking code!

## What You'll Learn

- How Emberware's rollback netcode works
- Why all game state must be in static variables
- Rules for deterministic code
- What happens during a rollback

## The Magic

Here's a surprising fact: **your Paddle game already works online**.

When two players connect over the internet:
1. Player 1's inputs are sent to Player 2's game
2. Player 2's inputs are sent to Player 1's game
3. Both games run the same `update()` function with the same inputs
4. Both games show the same result

**You didn't write a single line of networking code.**

## How It Works

### Rollback Netcode

Traditional netcode waits for the other player's input before advancing. This causes lag.

Emberware uses **rollback netcode**:

1. **Predict**: Don't have remote input? Guess it (usually "same as last frame")
2. **Continue**: Run the game with the prediction
3. **Correct**: When real input arrives, if it differs from prediction:
   - Roll back to the snapshot
   - Replay with correct input
   - Catch up to present

### Automatic Snapshots

Every frame, Emberware snapshots your entire WASM memory:

```
Frame 1: [snapshot] → update() → render()
Frame 2: [snapshot] → update() → render()
Frame 3: [snapshot] → update() → render()
         ↑
         If rollback needed, restore this and replay
```

This is why all game state must be in `static mut` variables - they live in WASM memory and get snapshotted automatically.

## The Rules for Rollback-Safe Code

### Rule 1: All State in WASM Memory

✅ **Good** - State in static variables:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PLAYER_X: f32 = 0.0;
static mut SCORE: u32 = 0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float player_x = 0.0f;
static uint32_t score = 0;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var player_x: f32 = 0.0;
var score: u32 = 0;
```
{{#endtab}}

{{#endtabs}}

❌ **Bad** - State outside WASM (if this were possible):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Don't try to use external state!
// (Rust's no_std prevents most of this anyway)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Don't try to use external state!
// All state must be in static variables
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Don't try to use external state!
// All state must be in global variables
```
{{#endtab}}

{{#endtabs}}

### Rule 2: Deterministic Update

Given the same inputs, `update()` must produce the same results.

✅ **Good** - Use `random()` for randomness:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
let rand = random();  // Deterministic, seeded by runtime
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t rand = random_u32();  // Deterministic, seeded by runtime
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const rand = random_u32();  // Deterministic, seeded by runtime
```
{{#endtab}}

{{#endtabs}}

❌ **Bad** - Use system time (if this were possible):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// let time = get_system_time();  // Non-deterministic!
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// uint64_t time = get_system_time();  // Non-deterministic!
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// const time = get_system_time();  // Non-deterministic!
```
{{#endtab}}

{{#endtabs}}

### Rule 3: No State Changes in Render

The `render()` function is **skipped during rollback**. Never modify game state there.

✅ **Good**:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Only READ state
    draw_rect(BALL_X, BALL_Y, ...);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Only READ state
    draw_rect(ball_x, ball_y, ...);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Only READ state
    draw_rect(ball_x, ball_y, ...);
}
```
{{#endtab}}

{{#endtabs}}

❌ **Bad**:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    ANIMATION_FRAME += 1;  // This won't replay during rollback!
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    animation_frame += 1;  // This won't replay during rollback!
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    animation_frame += 1;  // This won't replay during rollback!
}
```
{{#endtab}}

{{#endtabs}}

## Our Paddle Game Follows the Rules

Let's verify our code is rollback-safe:

### ✅ All State in Statics

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PADDLE1_Y: f32 = 0.0;
static mut PADDLE2_Y: f32 = 0.0;
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
static mut BALL_VX: f32 = 0.0;
static mut BALL_VY: f32 = 0.0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float paddle1_y = 0.0f;
static float paddle2_y = 0.0f;
static float ball_x = 0.0f;
static float ball_y = 0.0f;
static float ball_vx = 0.0f;
static float ball_vy = 0.0f;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var paddle1_y: f32 = 0.0;
var paddle2_y: f32 = 0.0;
var ball_x: f32 = 0.0;
var ball_y: f32 = 0.0;
var ball_vx: f32 = 0.0;
var ball_vy: f32 = 0.0;
```
{{#endtab}}

{{#endtabs}}

### ✅ Deterministic Randomness

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn reset_ball(direction: i32) {
    let rand = random() % 100;  // Uses runtime's seeded RNG
    // ...
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void reset_ball(int32_t direction) {
    uint32_t rand = random_u32() % 100;  // Uses runtime's seeded RNG
    // ...
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn reset_ball(direction: i32) void {
    const rand = random_u32() % 100;  // Uses runtime's seeded RNG
    // ...
}
```
{{#endtab}}

{{#endtabs}}

### ✅ Update Reads Input, Render Just Draws

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Read input
    let stick_y = left_stick_y(player);
    // Modify state
    PADDLE1_Y += movement;
}

fn render() {
    // Only draw, never modify state
    draw_rect(PADDLE_MARGIN, PADDLE1_Y, ...);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    // Read input
    int8_t stick_y = left_stick_y(player);
    // Modify state
    paddle1_y += movement;
}

EWZX_EXPORT void render(void) {
    // Only draw, never modify state
    draw_rect(PADDLE_MARGIN, paddle1_y, ...);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Read input
    const stick_y = left_stick_y(player);
    // Modify state
    paddle1_y += movement;
}

export fn render() void {
    // Only draw, never modify state
    draw_rect(PADDLE_MARGIN, paddle1_y, ...);
}
```
{{#endtab}}

{{#endtabs}}

## Testing Multiplayer Locally

To test multiplayer on your local machine:

1. Start the game
2. Connect a second controller
3. Both players can play!

The `player_count()` function automatically detects connected players.

## Testing Online Multiplayer

Online play is handled by the Emberware runtime:

1. Player 1 hosts a game
2. Player 2 joins via game code or direct connect
3. The runtime handles all networking
4. Your game code doesn't change at all!

## What Rollback Looks Like

During normal play:
```
You press A → Your game shows jump immediately
               (predicting remote player holds same buttons)

50ms later → Remote input arrives, matches prediction
             Nothing changes, smooth gameplay!
```

When prediction is wrong:
```
You press A → Your game shows jump immediately
               (predicting remote player holds same buttons)

50ms later → Remote input arrives: they pressed B!
             Game rolls back to frame N-3
             Replays frames N-3, N-2, N-1 with correct input
             Catches up to present frame N
             Visual "correction" happens in ~1-2 frames
```

With good connections, predictions are usually correct and rollbacks are rare.

## Summary

| Traditional Netcode | Emberware Rollback |
|--------------------|--------------------|
| Wait for input → lag | Predict input → smooth |
| Manual state sync | Automatic snapshots |
| You write network code | You write game code |
| State can be anywhere | State must be in WASM |

The key insight: **Emberware handles multiplayer complexity so you can focus on making your game fun.**

---

**Next:** [Part 6: Scoring & Win States](./06-scoring.md) - Add scoring and game flow.
