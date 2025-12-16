# Rollback Safety Guide

Writing deterministic code for Emberware's rollback netcode.

## How Rollback Works

Emberware uses GGRS for deterministic rollback netcode:

1. **Every tick**, your `update()` receives inputs from all players
2. **GGRS synchronizes** inputs across the network
3. **On misprediction**, the game state is restored from a snapshot and replayed

For this to work, your `update()` must be **deterministic**: same inputs → same state.

---

## The Golden Rules

### 1. Use `random()` for All Randomness

```rust
// GOOD - Deterministic
let spawn_x = (random() % 320) as f32;
let damage = 10 + (random() % 5) as i32;

// BAD - Non-deterministic
let spawn_x = system_time_nanos() % 320;  // Different on each client!
let damage = 10 + (thread_rng().next_u32() % 5); // Different seeds!
```

The `random()` function returns values from a synchronized seed, ensuring all clients get the same sequence.

---

### 2. Keep State in Static Variables

All game state must live in WASM linear memory (global statics):

```rust
// GOOD - State in WASM memory (snapshotted)
static mut PLAYER_X: f32 = 0.0;
static mut ENEMIES: [Enemy; 10] = [Enemy::new(); 10];

// BAD - State outside WASM memory
// (external systems, thread-locals, etc. are not snapshotted)
```

---

### 3. Same Inputs = Same State

Your `update()` must produce identical results given identical inputs:

```rust
fn update() {
    // All calculations based only on:
    // - Current state (in WASM memory)
    // - Player inputs (from GGRS)
    // - delta_time() / elapsed_time() / tick_count() (synchronized)
    // - random() (synchronized)

    let dt = delta_time();
    for p in 0..player_count() {
        if button_pressed(p, BUTTON_A) != 0 {
            players[p].jump();
        }
        players[p].x += left_stick_x(p) * SPEED * dt;
    }
}
```

---

### 4. Render is Skipped During Rollback

`render()` is **not** called during rollback replay. Don't put game logic in `render()`:

```rust
// GOOD - Logic in update()
fn update() {
    ANIMATION_FRAME = (tick_count() as u32 / 6) % 4;
}

fn render() {
    // Just draw, no state changes
    draw_sprite_region(..., ANIMATION_FRAME as f32 * 32.0, ...);
}

// BAD - Logic in render()
fn render() {
    ANIMATION_FRAME += 1;  // Skipped during rollback = desynced!
    draw_sprite_region(...);
}
```

---

## Common Pitfalls

### Floating Point Non-Determinism

Floating point operations can vary across CPUs. Emberware handles most cases, but be careful with:

```rust
// Potentially problematic
let angle = (y / x).atan();  // atan can differ slightly

// Safer alternatives
// - Use integer math where possible
// - Use lookup tables for trig
// - Accept small visual differences (for rendering only)
```

### Order-Dependent Iteration

HashMap iteration order is non-deterministic:

```rust
// BAD - Non-deterministic order
for (id, enemy) in enemies.iter() {
    enemy.update();  // Order matters for collisions!
}

// GOOD - Fixed order
for i in 0..enemies.len() {
    enemies[i].update();
}
```

### External State

Never read from external sources in `update()`:

```rust
// BAD
let now = SystemTime::now();  // Different on each client
let file = read_file("data.txt");  // Files can differ
let response = http_get("api.com");  // Network varies

// GOOD - All data from ROM or synchronized state
let data = rom_data(b"level".as_ptr(), 5, ...);
```

---

## Audio and Visual Effects

Audio and particles are often non-critical for gameplay:

```rust
fn update() {
    // Core gameplay - must be deterministic
    if player_hit_enemy() {
        ENEMY_HEALTH -= DAMAGE;

        // Audio/VFX triggers are fine here
        // (they'll replay during rollback, but that's OK)
        play_sound(HIT_SFX, 1.0, 0.0);
    }
}

fn render() {
    // Visual-only effects
    spawn_particles(PLAYER_X, PLAYER_Y);  // Not critical
}
```

---

## Memory Snapshotting

Emberware automatically snapshots your WASM linear memory:

```
What's Snapshotted (RAM):        What's NOT Snapshotted:
├── Static variables             ├── GPU textures (VRAM)
├── Heap allocations             ├── Audio buffers
├── Stack (function locals)      ├── Mesh data
└── Resource handles (u32s)      └── Resource data
```

**Tip:** Keep your game state small for faster snapshots. Only handles (u32) live in RAM; actual texture/mesh/audio data stays in host memory.

---

## Testing Determinism

### Local Testing

Run the same inputs twice and compare state:

```rust
fn update() {
    // After each update, log state hash
    let hash = calculate_state_hash();
    log_fmt(b"Tick {} hash: {}", tick_count(), hash);
}
```

### Multiplayer Testing

1. Start a local game with 2 players
2. Give identical inputs
3. Verify states match

---

## Debug Checklist

If you see desync:

1. **Check `random()` usage** - All randomness from `random()`?
2. **Check iteration order** - Using fixed-order arrays?
3. **Check floating point** - Sensitive calculations reproducible?
4. **Check `render()` logic** - Any state changes in render?
5. **Check external reads** - System time, files, network?
6. **Check audio timing** - Audio triggering consistent?

---

## Example: Deterministic Enemy AI

```rust
static mut ENEMIES: [Enemy; 10] = [Enemy::new(); 10];
static mut ENEMY_COUNT: usize = 0;

#[derive(Clone, Copy)]
struct Enemy {
    x: f32,
    y: f32,
    health: i32,
    ai_state: u8,
    ai_timer: u32,
}

impl Enemy {
    const fn new() -> Self {
        Self { x: 0.0, y: 0.0, health: 100, ai_state: 0, ai_timer: 0 }
    }

    fn update(&mut self, player_x: f32, player_y: f32) {
        match self.ai_state {
            0 => {
                // Idle - random chance to start patrol
                if random() % 100 < 5 {  // 5% chance per tick
                    self.ai_state = 1;
                    self.ai_timer = 60 + (random() % 60);  // 1-2 seconds
                }
            }
            1 => {
                // Patrol - move toward random target
                self.ai_timer -= 1;
                if self.ai_timer == 0 {
                    self.ai_state = 0;
                }
                // Movement...
            }
            _ => {}
        }
    }
}

fn update() {
    unsafe {
        let px = PLAYER_X;
        let py = PLAYER_Y;

        // Fixed iteration order
        for i in 0..ENEMY_COUNT {
            ENEMIES[i].update(px, py);
        }
    }
}
```

This AI is deterministic because:
- `random()` is synchronized
- Array iteration has fixed order
- All state is in WASM memory
- No external dependencies
