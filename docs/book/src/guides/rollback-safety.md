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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// GOOD - Deterministic
let spawn_x = (random() % 320) as f32;
let damage = 10 + (random() % 5) as i32;

// BAD - Non-deterministic
let spawn_x = system_time_nanos() % 320;  // Different on each client!
let damage = 10 + (thread_rng().next_u32() % 5); // Different seeds!
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// GOOD - Deterministic
float spawn_x = (float)(random_u32() % 320);
int32_t damage = 10 + (int32_t)(random_u32() % 5);

// BAD - Non-deterministic
float spawn_x = (float)(time(NULL) % 320);  // Different on each client!
int32_t damage = 10 + (rand() % 5); // Different seeds!
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// GOOD - Deterministic
const spawn_x: f32 = @floatFromInt(random_u32() % 320);
const damage: i32 = 10 + @intCast(random_u32() % 5);

// BAD - Non-deterministic
const spawn_x: f32 = @floatFromInt(std.time.nanoTimestamp() % 320);  // Different on each client!
const damage: i32 = 10 + @intCast(rand.next() % 5); // Different seeds!
```
{{#endtab}}

{{#endtabs}}

The `random()` function returns values from a synchronized seed, ensuring all clients get the same sequence.

---

### 2. Keep State in Static Variables

All game state must live in WASM linear memory (global statics):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// GOOD - State in WASM memory (snapshotted)
static mut PLAYER_X: f32 = 0.0;
static mut ENEMIES: [Enemy; 10] = [Enemy::new(); 10];

// BAD - State outside WASM memory
// (external systems, thread-locals, etc. are not snapshotted)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// GOOD - State in WASM memory (snapshotted)
static float player_x = 0.0f;
static Enemy enemies[10] = {0};

// BAD - State outside WASM memory
// (external systems, thread-locals, etc. are not snapshotted)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// GOOD - State in WASM memory (snapshotted)
var player_x: f32 = 0.0;
var enemies: [10]Enemy = [_]Enemy{Enemy{}} ** 10;

// BAD - State outside WASM memory
// (external systems, thread-locals, etc. are not snapshotted)
```
{{#endtab}}

{{#endtabs}}

---

### 3. Same Inputs = Same State

Your `update()` must produce identical results given identical inputs:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    // All calculations based only on:
    // - Current state (in WASM memory)
    // - Player inputs (from GGRS)
    // - delta_time() / elapsed_time() / tick_count() (synchronized)
    // - random_u32() (synchronized)

    float dt = delta_time();
    for (uint32_t p = 0; p < player_count(); p++) {
        if (button_pressed(p, EWZX_BUTTON_A) != 0) {
            player_jump(&players[p]);
        }
        players[p].x += left_stick_x(p) * SPEED * dt;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // All calculations based only on:
    // - Current state (in WASM memory)
    // - Player inputs (from GGRS)
    // - delta_time() / elapsed_time() / tick_count() (synchronized)
    // - random_u32() (synchronized)

    const dt = delta_time();
    var p: u32 = 0;
    while (p < player_count()) : (p += 1) {
        if (button_pressed(p, Button.a) != 0) {
            players[p].jump();
        }
        players[p].x += left_stick_x(p) * SPEED * dt;
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### 4. Render is Skipped During Rollback

`render()` is **not** called during rollback replay. Don't put game logic in `render()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
// GOOD - Logic in update()
EWZX_EXPORT void update(void) {
    animation_frame = (tick_count() / 6) % 4;
}

EWZX_EXPORT void render(void) {
    // Just draw, no state changes
    draw_sprite_region(..., (float)animation_frame * 32.0f, ...);
}

// BAD - Logic in render()
EWZX_EXPORT void render(void) {
    animation_frame++;  // Skipped during rollback = desynced!
    draw_sprite_region(...);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// GOOD - Logic in update()
export fn update() void {
    animation_frame = (tick_count() / 6) % 4;
}

export fn render() void {
    // Just draw, no state changes
    draw_sprite_region(..., @as(f32, @floatFromInt(animation_frame)) * 32.0, ...);
}

// BAD - Logic in render()
export fn render() void {
    animation_frame += 1;  // Skipped during rollback = desynced!
    draw_sprite_region(...);
}
```
{{#endtab}}

{{#endtabs}}

---

## Common Pitfalls

### Floating Point Non-Determinism

Floating point operations can vary across CPUs. Emberware handles most cases, but be careful with:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Potentially problematic
let angle = (y / x).atan();  // atan can differ slightly

// Safer alternatives
// - Use integer math where possible
// - Use lookup tables for trig
// - Accept small visual differences (for rendering only)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Potentially problematic
float angle = atanf(y / x);  // atan can differ slightly

// Safer alternatives
// - Use integer math where possible
// - Use lookup tables for trig
// - Accept small visual differences (for rendering only)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Potentially problematic
const angle = std.math.atan(y / x);  // atan can differ slightly

// Safer alternatives
// - Use integer math where possible
// - Use lookup tables for trig
// - Accept small visual differences (for rendering only)
```
{{#endtab}}

{{#endtabs}}

### Order-Dependent Iteration

HashMap iteration order is non-deterministic:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
// BAD - Non-deterministic order
// (using a hash map or unordered container)

// GOOD - Fixed order
for (size_t i = 0; i < enemy_count; i++) {
    enemy_update(&enemies[i]);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// BAD - Non-deterministic order
// (using a hash map)

// GOOD - Fixed order
var i: usize = 0;
while (i < enemies.len) : (i += 1) {
    enemies[i].update();
}
```
{{#endtab}}

{{#endtabs}}

### External State

Never read from external sources in `update()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// BAD
let now = SystemTime::now();  // Different on each client
let file = read_file("data.txt");  // Files can differ
let response = http_get("api.com");  // Network varies

// GOOD - All data from ROM or synchronized state
let data = rom_data(b"level".as_ptr(), 5, ...);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// BAD
time_t now = time(NULL);  // Different on each client
// Reading files or network requests  // Can differ

// GOOD - All data from ROM or synchronized state
uint32_t data = rom_data((uint32_t)"level", 5, ...);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// BAD
const now = std.time.timestamp();  // Different on each client
// Reading files or network requests  // Can differ

// GOOD - All data from ROM or synchronized state
const data = rom_data("level".ptr, 5, ...);
```
{{#endtab}}

{{#endtabs}}

---

## Audio and Visual Effects

Audio and particles are often non-critical for gameplay:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    // Core gameplay - must be deterministic
    if (player_hit_enemy()) {
        enemy_health -= DAMAGE;

        // Audio/VFX triggers are fine here
        // (they'll replay during rollback, but that's OK)
        play_sound(HIT_SFX, 1.0f, 0.0f);
    }
}

EWZX_EXPORT void render(void) {
    // Visual-only effects
    spawn_particles(player_x, player_y);  // Not critical
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Core gameplay - must be deterministic
    if (player_hit_enemy()) {
        enemy_health -= DAMAGE;

        // Audio/VFX triggers are fine here
        // (they'll replay during rollback, but that's OK)
        play_sound(HIT_SFX, 1.0, 0.0);
    }
}

export fn render() void {
    // Visual-only effects
    spawn_particles(player_x, player_y);  // Not critical
}
```
{{#endtab}}

{{#endtabs}}

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // After each update, log state hash
    let hash = calculate_state_hash();
    log_fmt(b"Tick {} hash: {}", tick_count(), hash);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update(void) {
    // After each update, log state hash
    uint32_t hash = calculate_state_hash();
    log_fmt((uint32_t)"Tick %u hash: %u", tick_count(), hash);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // After each update, log state hash
    const hash = calculate_state_hash();
    log_fmt("Tick {} hash: {}", .{tick_count(), hash});
}
```
{{#endtab}}

{{#endtabs}}

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
typedef struct {
    float x;
    float y;
    int32_t health;
    uint8_t ai_state;
    uint32_t ai_timer;
} Enemy;

static Enemy enemies[10] = {0};
static size_t enemy_count = 0;

void enemy_update(Enemy* enemy, float player_x, float player_y) {
    switch (enemy->ai_state) {
        case 0:
            // Idle - random chance to start patrol
            if (random_u32() % 100 < 5) {  // 5% chance per tick
                enemy->ai_state = 1;
                enemy->ai_timer = 60 + (random_u32() % 60);  // 1-2 seconds
            }
            break;
        case 1:
            // Patrol - move toward random target
            enemy->ai_timer--;
            if (enemy->ai_timer == 0) {
                enemy->ai_state = 0;
            }
            // Movement...
            break;
    }
}

EWZX_EXPORT void update(void) {
    float px = player_x;
    float py = player_y;

    // Fixed iteration order
    for (size_t i = 0; i < enemy_count; i++) {
        enemy_update(&enemies[i], px, py);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const Enemy = struct {
    x: f32 = 0.0,
    y: f32 = 0.0,
    health: i32 = 100,
    ai_state: u8 = 0,
    ai_timer: u32 = 0,

    pub fn update(self: *Enemy, player_x: f32, player_y: f32) void {
        switch (self.ai_state) {
            0 => {
                // Idle - random chance to start patrol
                if (random_u32() % 100 < 5) {  // 5% chance per tick
                    self.ai_state = 1;
                    self.ai_timer = 60 + (random_u32() % 60);  // 1-2 seconds
                }
            },
            1 => {
                // Patrol - move toward random target
                self.ai_timer -= 1;
                if (self.ai_timer == 0) {
                    self.ai_state = 0;
                }
                // Movement...
            },
            else => {},
        }
    }
};

var enemies: [10]Enemy = [_]Enemy{Enemy{}} ** 10;
var enemy_count: usize = 0;

export fn update() void {
    const px = player_x;
    const py = player_y;

    // Fixed iteration order
    var i: usize = 0;
    while (i < enemy_count) : (i += 1) {
        enemies[i].update(px, py);
    }
}
```
{{#endtab}}

{{#endtabs}}

This AI is deterministic because:
- `random()` is synchronized
- Array iteration has fixed order
- All state is in WASM memory
- No external dependencies
