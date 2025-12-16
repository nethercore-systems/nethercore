# System Functions

Core system functions for time, logging, randomness, and session management.

## Time Functions

### delta_time

Returns the time elapsed since the last tick in seconds.

**Signature:**
```rust
fn delta_time() -> f32
```

**Returns:** Time in seconds since last tick (typically 1/60 = 0.0167 at 60fps)

**Example:**
```rust
fn update() {
    // Frame-rate independent movement
    position.x += velocity.x * delta_time();
    position.y += velocity.y * delta_time();
}
```

**See Also:** [elapsed_time](#elapsed_time), [tick_count](#tick_count)

---

### elapsed_time

Returns total elapsed time since game start in seconds.

**Signature:**
```rust
fn elapsed_time() -> f32
```

**Returns:** Total seconds since `init()` was called

**Example:**
```rust
fn render() {
    // Pulsing effect
    let pulse = (elapsed_time() * 2.0).sin() * 0.5 + 0.5;
    set_color(rgba(255, 255, 255, (pulse * 255.0) as u8));
}
```

**See Also:** [delta_time](#delta_time), [tick_count](#tick_count)

---

### tick_count

Returns the current tick number (frame count).

**Signature:**
```rust
fn tick_count() -> u64
```

**Returns:** Number of ticks since game start

**Example:**
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

**See Also:** [delta_time](#delta_time), [elapsed_time](#elapsed_time)

---

## Logging

### log

Outputs a message to the console for debugging.

**Signature:**
```rust
fn log(ptr: *const u8, len: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ptr | `*const u8` | Pointer to UTF-8 string data |
| len | `u32` | Length of the string in bytes |

**Example:**
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

---

## Control Flow

### quit

Exits the game and returns to the Emberware library.

**Signature:**
```rust
fn quit()
```

**Example:**
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

---

## Randomness

### random

Returns a deterministic random number from the host's seeded RNG.

**Signature:**
```rust
fn random() -> u32
```

**Returns:** A random `u32` value (0 to 4,294,967,295)

**Constraints:** Must use this for all randomness to maintain rollback determinism.

**Example:**
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

**Warning:** Never use external random sources (system time, etc.) — this breaks rollback determinism.

---

## Session Functions

### player_count

Returns the number of players in the current session.

**Signature:**
```rust
fn player_count() -> u32
```

**Returns:** Number of players (1-4)

**Example:**
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

**See Also:** [local_player_mask](#local_player_mask)

---

### local_player_mask

Returns a bitmask indicating which players are local to this client.

**Signature:**
```rust
fn local_player_mask() -> u32
```

**Returns:** Bitmask where bit N is set if player N is local

**Example:**
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

### Multiplayer Model

Emberware supports up to 4 players in any combination:
- 4 local players (couch co-op)
- 1 local + 3 remote (online)
- 2 local + 2 remote (mixed)

All inputs are synchronized via GGRS rollback netcode. Your `update()` processes all players uniformly — the host handles synchronization automatically.

```rust
fn update() {
    // This code works for any local/remote mix
    for p in 0..player_count() {
        let input = get_player_input(p);
        update_player(p, input);
    }
}
```
