# Understanding the Game Loop

Every Emberware game implements three core functions that the runtime calls at specific times. Understanding this lifecycle is key to building robust, multiplayer-ready games.

## The Three Functions

### `init()` - Called Once at Startup

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Load resources
    // Configure graphics settings
    // Initialize game state
}
```

**Purpose:** Set up your game. This runs once when the game starts.

**Common uses:**
- Set resolution and tick rate
- Configure render mode
- Load textures from ROM or create procedural ones
- Initialize game state to starting values
- Set clear color

**Example:**
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

### `update()` - Called Every Tick

```rust
#[no_mangle]
pub extern "C" fn update() {
    // Read input
    // Update game logic
    // Handle physics
    // Check collisions
}
```

**Purpose:** Update your game state. This runs at a fixed rate (default 60 times per second).

**Critical for multiplayer:** The `update()` function must be **deterministic**. Given the same inputs, it must produce exactly the same results every time. This is how rollback netcode works.

**Rules for deterministic code:**
- Use `random()` for randomness (seeded by the runtime)
- Don't use system time or external state
- All game logic goes here, not in `render()`

**Example:**
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

### `render()` - Called Every Frame

```rust
#[no_mangle]
pub extern "C" fn render() {
    // Set up camera
    // Draw game objects
    // Draw UI
}
```

**Purpose:** Draw your game. This runs every frame (may be more often than `update()` for smooth visuals).

**Important:**
- This function is **skipped during rollback**
- Don't modify game state here
- Use state from `update()` to determine what to draw

**Example:**
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
        draw_text(b"Score: ".as_ptr(), 7, 10.0, 10.0, 20.0, 0xFFFFFFFF);
    }
}
```

## Tick Rate vs Frame Rate

| Concept | Default | Purpose |
|---------|---------|---------|
| **Tick Rate** | 60 Hz | How often `update()` runs. Fixed for determinism. |
| **Frame Rate** | Variable | How often `render()` runs. Matches display refresh. |

You can change the tick rate in `init()`:

```rust
set_tick_rate(0);  // 24 ticks per second (cinematic)
set_tick_rate(1);  // 30 ticks per second
set_tick_rate(2);  // 60 ticks per second (default)
set_tick_rate(3);  // 120 ticks per second (fighting games)
```

## The Rollback System

Emberware's killer feature is automatic rollback netcode. Here's how it works:

1. **Snapshot:** The runtime snapshots all WASM memory after each `update()`
2. **Predict:** When waiting for remote player input, the game predicts and continues
3. **Rollback:** When real input arrives, the game rolls back and replays
4. **Skip render:** During rollback replay, `render()` is not called

**Why this matters:**
- All your game state must be in WASM memory (static variables)
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
| `delta_time()` | `f32` | Seconds since last tick |
| `elapsed_time()` | `f32` | Total seconds since game start |
| `tick_count()` | `u64` | Number of ticks since start |
| `random()` | `u32` | Deterministic random number |

## Common Patterns

### Game State Machine

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

### Delta Time for Smooth Movement

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

---

**You're ready to build real games!**

Continue to the [Build Paddle](../tutorials/paddle/index.md) tutorial to create your first complete game, or explore the [API Reference](../cheat-sheet.md) to see all available functions.
