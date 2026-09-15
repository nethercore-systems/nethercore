# Part 5: Multiplayer

This is where Nethercore's magic happens. Our Paddle game already supports online multiplayer - and we didn't write any networking code!

## What You'll Learn

- How Nethercore's rollback netcode works
- Why persistent simulation state must live in snapshotted WASM linear memory
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

Nethercore uses **rollback netcode**:

1. **Predict**: Don't have remote input? Guess it (usually "same as last frame")
2. **Continue**: Run the game with the prediction
3. **Correct**: When real input arrives, if it differs from prediction:
   - Roll back to the snapshot
   - Replay with correct input
   - Catch up to present

### Automatic Snapshots

Every frame, Nethercore snapshots your entire WASM memory:

```
Frame 1: [snapshot] → update() → render()
Frame 2: [snapshot] → update() → render()
Frame 3: [snapshot] → update() → render()
         ↑
         If rollback needed, restore this and replay
```

Ordinary Rust statics reside in linear memory and are covered by these snapshots.
They are not the only valid storage: heap-backed state in that memory is covered
too. Do not assume arbitrary WASM globals or external host state are snapshotted;
keep all persistent simulation state on the covered path.

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
let rand = random_u32();  // Deterministic, seeded by runtime
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
NCZX_EXPORT void render(void) {
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
NCZX_EXPORT void render(void) {
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
    let rand = random_u32() % 100;  // Uses runtime's seeded RNG
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
NCZX_EXPORT void update(void) {
    // Read input
    float stick_y = left_stick_y(player);
    // Modify state
    paddle1_y += movement;
}

NCZX_EXPORT void render(void) {
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

## Test the local paths

For a maintained example, start in `examples/7-games/netplay-demo` in the
checkout and run `nether build`. The resulting cartridge is `netplay-demo.nczx`.
It permits up to four players; the commands below exercise two. Put the matching
`nether` and `nethercore-zx` executables together on PATH. For your own Paddle
project, substitute its cartridge and explicitly configure `[game].max_players`
and `[netplay].enabled`.

Keep peer-local information such as `local_player_mask()` out of shared,
snapshotted WASM state. Query it for presentation or local persistence instead.
Update shared values (including saved-stat checksums) on every peer before
branching into local-only disk operations.

### Two controllers on one machine

```bash
nether build
nether run --no-build --players 2
```

Map the second input source to player 2 in player settings. `player_count()`
reports the configured session, not the number of plugged-in controllers.
This checks local controls; it does **not** exercise network transport.

### Rollback determinism (one process)

```bash
nether run --no-build --sync-test --check-distance 2 --players 2 --exit-after-frames 120
```

Require successful frame completion and no sync-test mismatch. A sync test is
not evidence of a live connection between peers.

### Two real local peers

```bash
nether run --no-build --p2p-test --exit-after-frames 120
```

This opens two players on localhost using ports 7777 and 7778. Both must reach
frame completion without disconnection/desync errors. Keep those ports free.
To control the processes separately, use the standalone player in two terminals
with the same packed cartridge:

```bash
nethercore-zx netplay-demo.nczx --p2p --bind 7777 --peer 7778 --local-player 0 --exit-after-frames 120
nethercore-zx netplay-demo.nczx --p2p --bind 7778 --peer 7777 --local-player 1 --exit-after-frames 120
```

For host/join on the same machine, use two terminals and distinct bind ports:

```bash
nethercore-zx netplay-demo.nczx --host 7777 --players 2 --exit-after-frames 120
nethercore-zx netplay-demo.nczx --join 127.0.0.1:7777 --bind 7778 --players 2 --exit-after-frames 120
```

For a LAN test, replace `127.0.0.1` with the host's LAN address and permit the
selected UDP ports through each machine's firewall. Internet NAT traversal,
hosted lobbies and accounts are separate from this local walkthrough. A second
window opening alone is not a successful multiplayer test: require both
processes to finish their frames without desync/disconnection errors. These
connection checks are not evidence of cross-machine save synchronization.

From the checkout root, the retained native regression helper can enforce this
without silently accepting an early process exit (use a new output directory):

```bash
python scripts/check_local_netplay.py path/to/nethercore-zx.exe path/to/netplay-demo.nczx --output path/to/new-check
```

Add `--host-join` to exercise that path. It needs a real desktop/GPU; it is not a
headless substitute for the normal player.

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

| Traditional Netcode | Nethercore Rollback |
|--------------------|--------------------|
| Wait for input → lag | Predict input → smooth |
| Manual state sync | Automatic snapshots |
| You write network code | You write game code |
| State can be anywhere | State must be in WASM |

The key insight: **Nethercore handles multiplayer complexity so you can focus on making your game fun.**

---

**Next:** [Part 6: Scoring & Win States](./06-scoring.md) - Add scoring and game flow.
