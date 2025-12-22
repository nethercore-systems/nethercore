# System Functions

Core system functions for time, logging, randomness, and session management.

## Time Functions

### delta_time

Returns the time elapsed since the last tick in seconds.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn delta_time() -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float delta_time(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn delta_time() f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Time in seconds since last tick (typically 1/60 = 0.0167 at 60fps)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Frame-rate independent movement
    position.x += velocity.x * delta_time();
    position.y += velocity.y * delta_time();
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Frame-rate independent movement */
    position_x += velocity_x * delta_time();
    position_y += velocity_y * delta_time();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Frame-rate independent movement
    position_x += velocity_x * delta_time();
    position_y += velocity_y * delta_time();
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [elapsed_time](#elapsed_time), [tick_count](#tick_count)

---

### elapsed_time

Returns total elapsed time since game start in seconds.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn elapsed_time() -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float elapsed_time(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn elapsed_time() f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Total seconds since `init()` was called

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Pulsing effect
    let pulse = (elapsed_time() * 2.0).sin() * 0.5 + 0.5;
    set_color(rgba(255, 255, 255, (pulse * 255.0) as u8));
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    /* Pulsing effect */
    float pulse = sinf(elapsed_time() * 2.0f) * 0.5f + 0.5f;
    set_color(nczx_rgba(255, 255, 255, (uint8_t)(pulse * 255.0f)));
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Pulsing effect
    const pulse = @sin(elapsed_time() * 2.0) * 0.5 + 0.5;
    set_color(rgba(255, 255, 255, @intFromFloat(pulse * 255.0)));
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [delta_time](#delta_time), [tick_count](#tick_count)

---

### tick_count

Returns the current tick number (frame count).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn tick_count() -> u64
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint64_t tick_count(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn tick_count() u64;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Number of ticks since game start

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Every second at 60fps
    if tick_count() % 60 == 0 {
        spawn_enemy();
    }

    // Every other tick
    if tick_count() % 2 == 0 {
        animate_water();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Every second at 60fps */
    if (tick_count() % 60 == 0) {
        spawn_enemy();
    }

    /* Every other tick */
    if (tick_count() % 2 == 0) {
        animate_water();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Every second at 60fps
    if (tick_count() % 60 == 0) {
        spawn_enemy();
    }

    // Every other tick
    if (tick_count() % 2 == 0) {
        animate_water();
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [delta_time](#delta_time), [elapsed_time](#elapsed_time)

---

## Logging

### log

Outputs a message to the console for debugging.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn log(ptr: *const u8, len: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void log_msg(const uint8_t* ptr, uint32_t len);
// Helper macro: NCZX_LOG("message")
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn log_msg(ptr: [*]const u8, len: u32) void;
// Helper: zx.log("message")
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ptr | `*const u8` | Pointer to UTF-8 string data |
| len | `u32` | Length of the string in bytes |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    let msg = b"Game initialized!";
    log(msg.as_ptr(), msg.len() as u32);
}

fn update() {
    if player_died {
        let msg = b"Player died";
        log(msg.as_ptr(), msg.len() as u32);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    NCZX_LOG("Game initialized!");
}

NCZX_EXPORT void update(void) {
    if (player_died) {
        NCZX_LOG("Player died");
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const zx = @import("zx.zig");

export fn init() void {
    zx.log("Game initialized!");
}

export fn update() void {
    if (player_died) {
        zx.log("Player died");
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Control Flow

### quit

Exits the game and returns to the Nethercore library.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn quit()
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void quit(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn quit() void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Quit on Start + Select held for 60 frames
    if buttons_held(0) & ((1 << BUTTON_START) | (1 << BUTTON_SELECT)) != 0 {
        quit_timer += 1;
        if quit_timer >= 60 {
            quit();
        }
    } else {
        quit_timer = 0;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Quit on Start + Select held for 60 frames */
    if (buttons_held(0) & ((1 << NCZX_BUTTON_START) | (1 << NCZX_BUTTON_SELECT))) {
        quit_timer++;
        if (quit_timer >= 60) {
            quit();
        }
    } else {
        quit_timer = 0;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Quit on Start + Select held for 60 frames
    if (buttons_held(0) & ((1 << Button.start) | (1 << Button.select)) != 0) {
        quit_timer += 1;
        if (quit_timer >= 60) {
            quit();
        }
    } else {
        quit_timer = 0;
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Randomness

### random

Returns a deterministic random number from the host's seeded RNG.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn random() -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t random_u32(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn random_u32() u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** A random `u32` value (0 to 4,294,967,295)

**Constraints:** Must use this for all randomness to maintain rollback determinism.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Random integer in range [0, 320)
    let spawn_x = (random() % 320) as f32;

    // Random float 0.0 to 1.0
    let rf = (random() as f32) / (u32::MAX as f32);

    // Random bool
    let coin_flip = random() & 1 == 0;

    // Random float in range [min, max]
    let min = 10.0;
    let max = 50.0;
    let rf = (random() as f32) / (u32::MAX as f32);
    let value = min + rf * (max - min);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Random integer in range [0, 320) */
    float spawn_x = (float)(random_u32() % 320);

    /* Random float 0.0 to 1.0 */
    float rf = (float)random_u32() / (float)UINT32_MAX;

    /* Random bool */
    int coin_flip = (random_u32() & 1) == 0;

    /* Random float in range [min, max] */
    float min = 10.0f;
    float max = 50.0f;
    float value = min + rf * (max - min);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Random integer in range [0, 320)
    const spawn_x: f32 = @floatFromInt(random_u32() % 320);

    // Random float 0.0 to 1.0
    const rf: f32 = @as(f32, @floatFromInt(random_u32())) / @as(f32, @floatFromInt(@as(u32, 0xFFFFFFFF)));

    // Random bool
    const coin_flip = (random_u32() & 1) == 0;

    // Random float in range [min, max]
    const min: f32 = 10.0;
    const max: f32 = 50.0;
    const value = min + rf * (max - min);
}
```
{{#endtab}}

{{#endtabs}}

**Warning:** Never use external random sources (system time, etc.) — this breaks rollback determinism.

---

## Session Functions

### player_count

Returns the number of players in the current session.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn player_count() -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t player_count(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn player_count() u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Number of players (1-4)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Process all players
    for p in 0..player_count() {
        process_player_input(p);
        update_player_state(p);
    }
}

fn render() {
    // Draw viewport split for multiplayer
    match player_count() {
        1 => draw_fullscreen_viewport(0),
        2 => {
            draw_half_viewport(0, 0);   // Left half
            draw_half_viewport(1, 1);   // Right half
        }
        _ => draw_quad_viewports(),
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Process all players */
    for (uint32_t p = 0; p < player_count(); p++) {
        process_player_input(p);
        update_player_state(p);
    }
}

NCZX_EXPORT void render(void) {
    /* Draw viewport split for multiplayer */
    switch (player_count()) {
        case 1: draw_fullscreen_viewport(0); break;
        case 2:
            draw_half_viewport(0, 0);   /* Left half */
            draw_half_viewport(1, 1);   /* Right half */
            break;
        default: draw_quad_viewports(); break;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Process all players
    var p: u32 = 0;
    while (p < player_count()) : (p += 1) {
        process_player_input(p);
        update_player_state(p);
    }
}

export fn render() void {
    // Draw viewport split for multiplayer
    switch (player_count()) {
        1 => draw_fullscreen_viewport(0),
        2 => {
            draw_half_viewport(0, 0);   // Left half
            draw_half_viewport(1, 1);   // Right half
        },
        else => draw_quad_viewports(),
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [local_player_mask](#local_player_mask)

---

### local_player_mask

Returns a bitmask indicating which players are local to this client.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn local_player_mask() -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t local_player_mask(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn local_player_mask() u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Bitmask where bit N is set if player N is local

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    let mask = local_player_mask();

    // Check if specific player is local
    let p0_local = (mask & 1) != 0;  // Player 0
    let p1_local = (mask & 2) != 0;  // Player 1
    let p2_local = (mask & 4) != 0;  // Player 2
    let p3_local = (mask & 8) != 0;  // Player 3

    // Only show local player's UI
    for p in 0..player_count() {
        if (mask & (1 << p)) != 0 {
            draw_player_ui(p);
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    uint32_t mask = local_player_mask();

    /* Check if specific player is local */
    int p0_local = (mask & 1) != 0;  /* Player 0 */
    int p1_local = (mask & 2) != 0;  /* Player 1 */
    int p2_local = (mask & 4) != 0;  /* Player 2 */
    int p3_local = (mask & 8) != 0;  /* Player 3 */

    /* Only show local player's UI */
    for (uint32_t p = 0; p < player_count(); p++) {
        if (mask & (1 << p)) {
            draw_player_ui(p);
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const mask = local_player_mask();

    // Check if specific player is local
    const p0_local = (mask & 1) != 0;  // Player 0
    const p1_local = (mask & 2) != 0;  // Player 1
    const p2_local = (mask & 4) != 0;  // Player 2
    const p3_local = (mask & 8) != 0;  // Player 3

    // Only show local player's UI
    var p: u32 = 0;
    while (p < player_count()) : (p += 1) {
        if (mask & (@as(u32, 1) << @intCast(p)) != 0) {
            draw_player_ui(p);
        }
    }
}
```
{{#endtab}}

{{#endtabs}}

### Multiplayer Model

Nethercore supports up to 4 players in any combination:
- 4 local players (couch co-op)
- 1 local + 3 remote (online)
- 2 local + 2 remote (mixed)

All inputs are synchronized via GGRS rollback netcode. Your `update()` processes all players uniformly — the host handles synchronization automatically.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // This code works for any local/remote mix
    for p in 0..player_count() {
        let input = get_player_input(p);
        update_player(p, input);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* This code works for any local/remote mix */
    for (uint32_t p = 0; p < player_count(); p++) {
        Input input = get_player_input(p);
        update_player(p, input);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // This code works for any local/remote mix
    var p: u32 = 0;
    while (p < player_count()) : (p += 1) {
        const input = get_player_input(p);
        update_player(p, input);
    }
}
```
{{#endtab}}

{{#endtabs}}
