# Understanding the Game Loop

Every Emberware game implements three core functions that the runtime calls at specific times. Understanding this lifecycle is key to building robust, multiplayer-ready games.

## The Three Functions

### `init()` - Called Once at Startup

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    // Load resources
    // Configure graphics settings
    // Initialize game state
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void init(void) {
    /* Load resources */
    /* Configure graphics settings */
    /* Initialize game state */
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    // Load resources
    // Configure graphics settings
    // Initialize game state
}
```
{{#endtab}}

{{#endtabs}}

**Purpose:** Set up your game. This runs once when the game starts.

**Common uses:**
- Set resolution and tick rate
- Configure render mode
- Load textures from ROM or create procedural ones
- Initialize game state to starting values
- Set clear color

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_resolution(1);        // 540p
        set_tick_rate(2);         // 60 FPS
        set_clear_color(0x000000FF);
        render_mode(2);           // PBR lighting

        // Load a texture
        PLAYER_TEXTURE = load_texture(8, 8, PIXELS.as_ptr());
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t player_texture = 0;

EWZX_EXPORT void init(void) {
    set_resolution(1);        /* 540p */
    set_tick_rate(2);         /* 60 FPS */
    set_clear_color(0x000000FF);
    render_mode(EWZX_RENDER_PBR);

    /* Load a texture */
    player_texture = load_texture(8, 8, pixels);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var player_texture: u32 = 0;

export fn init() void {
    set_resolution(1);        // 540p
    set_tick_rate(2);         // 60 FPS
    set_clear_color(0x000000FF);
    render_mode(2);           // PBR lighting

    // Load a texture
    player_texture = load_texture(8, 8, &pixels);
}
```
{{#endtab}}

{{#endtabs}}

### `update()` - Called Every Tick

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    // Read input
    // Update game logic
    // Handle physics
    // Check collisions
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    /* Read input */
    /* Update game logic */
    /* Handle physics */
    /* Check collisions */
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Read input
    // Update game logic
    // Handle physics
    // Check collisions
}
```
{{#endtab}}

{{#endtabs}}

**Purpose:** Update your game state. This runs at a fixed rate (default 60 times per second).

**Critical for multiplayer:** The `update()` function must be **deterministic**. Given the same inputs, it must produce exactly the same results every time. This is how rollback netcode works.

**Rules for deterministic code:**
- Use `random()` (or `random_u32()` in C) for randomness (seeded by the runtime)
- Don't use system time or external state
- All game logic goes here, not in `render()`

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();

        // Read input
        let move_x = left_stick_x(0);
        let jump = button_pressed(0, BUTTON_A) != 0;

        // Update physics
        PLAYER_VY -= GRAVITY;
        PLAYER_X += move_x * SPEED * dt;
        PLAYER_Y += PLAYER_VY * dt;

        // Handle jump
        if jump && ON_GROUND {
            PLAYER_VY = JUMP_FORCE;
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    float dt = delta_time();

    /* Read input */
    float move_x = left_stick_x(0);
    int jump = button_pressed(0, EWZX_BUTTON_A) != 0;

    /* Update physics */
    player_vy -= GRAVITY;
    player_x += move_x * SPEED * dt;
    player_y += player_vy * dt;

    /* Handle jump */
    if (jump && on_ground) {
        player_vy = JUMP_FORCE;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const dt = delta_time();

    // Read input
    const move_x = left_stick_x(0);
    const jump = button_pressed(0, BUTTON_A) != 0;

    // Update physics
    player_vy -= GRAVITY;
    player_x += move_x * SPEED * dt;
    player_y += player_vy * dt;

    // Handle jump
    if (jump and on_ground) {
        player_vy = JUMP_FORCE;
    }
}
```
{{#endtab}}

{{#endtabs}}

### `render()` - Called Every Frame

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn render() {
    // Set up camera
    // Draw game objects
    // Draw UI
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Set up camera */
    /* Draw game objects */
    /* Draw UI */
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Set up camera
    // Draw game objects
    // Draw UI
}
```
{{#endtab}}

{{#endtabs}}

**Purpose:** Draw your game. This runs every frame (may be more often than `update()` for smooth visuals).

**Important:**
- This function is **skipped during rollback**
- Don't modify game state here
- Use state from `update()` to determine what to draw

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

        // Draw player
        push_identity();
        push_translate(PLAYER_X, PLAYER_Y, 0.0);
        texture_bind(PLAYER_TEXTURE);
        draw_mesh(PLAYER_MESH);

        // Draw UI
        let score = b"Score: ";
        draw_text(score.as_ptr(), score.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Set camera */
    camera_set(0.0f, 5.0f, 10.0f, 0.0f, 0.0f, 0.0f);

    /* Draw player */
    push_identity();
    push_translate(player_x, player_y, 0.0f);
    texture_bind(player_texture);
    draw_mesh(player_mesh);

    /* Draw UI */
    EWZX_DRAW_TEXT("Score: ", 10.0f, 10.0f, 20.0f, EWZX_WHITE);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Set camera
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // Draw player
    push_identity();
    push_translate(player_x, player_y, 0.0);
    texture_bind(player_texture);
    draw_mesh(player_mesh);

    // Draw UI
    const score = "Score: ";
    draw_text(score.ptr, score.len, 10.0, 10.0, 20.0, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#endtabs}}

## Tick Rate vs Frame Rate

| Concept | Default | Purpose |
|---------|---------|---------|
| **Tick Rate** | 60 Hz | How often `update()` runs. Fixed for determinism. |
| **Frame Rate** | Variable | How often `render()` runs. Matches display refresh. |

You can change the tick rate in `init()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
set_tick_rate(0);  // 24 ticks per second (cinematic)
set_tick_rate(1);  // 30 ticks per second
set_tick_rate(2);  // 60 ticks per second (default)
set_tick_rate(3);  // 120 ticks per second (fighting games)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
set_tick_rate(EWZX_TICK_RATE_24);   /* 24 ticks per second (cinematic) */
set_tick_rate(EWZX_TICK_RATE_30);   /* 30 ticks per second */
set_tick_rate(EWZX_TICK_RATE_60);   /* 60 ticks per second (default) */
set_tick_rate(EWZX_TICK_RATE_120);  /* 120 ticks per second (fighting games) */
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
set_tick_rate(0);  // 24 ticks per second (cinematic)
set_tick_rate(1);  // 30 ticks per second
set_tick_rate(2);  // 60 ticks per second (default)
set_tick_rate(3);  // 120 ticks per second (fighting games)
```
{{#endtab}}

{{#endtabs}}

## The Rollback System

Emberware's killer feature is automatic rollback netcode. Here's how it works:

1. **Snapshot:** The runtime snapshots all WASM memory after each `update()`
2. **Predict:** When waiting for remote player input, the game predicts and continues
3. **Rollback:** When real input arrives, the game rolls back and replays
4. **Skip render:** During rollback replay, `render()` is not called

**Why this matters:**
- All your game state must be in WASM memory (static/global variables)
- `update()` must be deterministic
- `render()` should only read state, never modify it

```
init()          ← Run once
    │
    ▼
┌─────────────────┐
│   update() ←────┼── Runs at fixed tick rate
│   (snapshot)    │   Rollback replays from here
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   render() ←────┼── Runs every frame
│   (skipped      │   Skipped during rollback
│    on rollback) │
└────────┬────────┘
         │
         └── Loop back to update()
```

## Helpful Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `delta_time()` | `f32`/`float` | Seconds since last tick (fixed) |
| `elapsed_time()` | `f32`/`float` | Total seconds since game start |
| `tick_count()` | `u64`/`uint64_t` | Number of ticks since start |
| `random()` / `random_u32()` | `u32`/`uint32_t` | Deterministic random number |

## Common Patterns

### Game State Machine

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    Paused,
    GameOver,
}

static mut STATE: GameState = GameState::Title;

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        match STATE {
            GameState::Title => update_title(),
            GameState::Playing => update_gameplay(),
            GameState::Paused => update_pause(),
            GameState::GameOver => update_game_over(),
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
typedef enum {
    STATE_TITLE,
    STATE_PLAYING,
    STATE_PAUSED,
    STATE_GAME_OVER
} GameState;

static GameState state = STATE_TITLE;

EWZX_EXPORT void update(void) {
    switch (state) {
        case STATE_TITLE:    update_title();     break;
        case STATE_PLAYING:  update_gameplay();  break;
        case STATE_PAUSED:   update_pause();     break;
        case STATE_GAME_OVER: update_game_over(); break;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const GameState = enum {
    title,
    playing,
    paused,
    game_over,
};

var state: GameState = .title;

export fn update() void {
    switch (state) {
        .title => update_title(),
        .playing => update_gameplay(),
        .paused => update_pause(),
        .game_over => update_game_over(),
    }
}
```
{{#endtab}}

{{#endtabs}}

### Delta Time for Smooth Movement

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();

        // Movement is frame-rate independent
        PLAYER_X += SPEED * dt;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    float dt = delta_time();

    /* Movement is frame-rate independent */
    player_x += SPEED * dt;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const dt = delta_time();

    // Movement is frame-rate independent
    player_x += SPEED * dt;
}
```
{{#endtab}}

{{#endtabs}}

---

**You're ready to build real games!**

Continue to the [Build Paddle](../tutorials/paddle/index.md) tutorial to create your first complete game, or explore the [API Reference](../cheat-sheet.md) to see all available functions.
