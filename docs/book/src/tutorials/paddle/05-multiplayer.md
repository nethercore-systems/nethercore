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
```rust
static mut PLAYER_X: f32 = 0.0;
static mut SCORE: u32 = 0;
```

❌ **Bad** - State outside WASM (if this were possible):
```rust
// Don't try to use external state!
// (Rust's no_std prevents most of this anyway)
```

### Rule 2: Deterministic Update

Given the same inputs, `update()` must produce the same results.

✅ **Good** - Use `random()` for randomness:
```rust
let rand = random();  // Deterministic, seeded by runtime
```

❌ **Bad** - Use system time (if this were possible):
```rust
// let time = get_system_time();  // Non-deterministic!
```

### Rule 3: No State Changes in Render

The `render()` function is **skipped during rollback**. Never modify game state there.

✅ **Good**:
```rust
fn render() {
    // Only READ state
    draw_rect(BALL_X, BALL_Y, ...);
}
```

❌ **Bad**:
```rust
fn render() {
    ANIMATION_FRAME += 1;  // This won't replay during rollback!
}
```

## Our Paddle Game Follows the Rules

Let's verify our code is rollback-safe:

### ✅ All State in Statics
```rust
static mut PADDLE1_Y: f32 = 0.0;
static mut PADDLE2_Y: f32 = 0.0;
static mut BALL_X: f32 = 0.0;
static mut BALL_Y: f32 = 0.0;
static mut BALL_VX: f32 = 0.0;
static mut BALL_VY: f32 = 0.0;
```

### ✅ Deterministic Randomness
```rust
fn reset_ball(direction: i32) {
    let rand = random() % 100;  // Uses runtime's seeded RNG
    // ...
}
```

### ✅ Update Reads Input, Render Just Draws
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
