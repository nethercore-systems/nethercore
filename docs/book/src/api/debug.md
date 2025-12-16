# Debug Functions

Runtime value inspection via the F3 debug panel.

## Overview

The debug system allows you to register game variables for live editing and monitoring. Press **F3** to open the debug panel during development.

**Features:**
- Live value editing (sliders, color pickers)
- Read-only watches
- Grouped organization
- Frame control (pause, step, time scale)
- Zero overhead in release builds

---

## Value Registration

Register editable values in `init()`. The debug panel will show controls for these values.

### debug_register_i32

Registers an editable 32-bit signed integer.

**Signature:**
```rust
fn debug_register_i32(name_ptr: *const u8, name_len: u32, ptr: *const i32)
```

**Example:**
```rust
static mut ENEMY_COUNT: i32 = 5;

fn init() {
    unsafe {
        debug_register_i32(b"Enemy Count".as_ptr(), 11, &ENEMY_COUNT);
    }
}
```

---

### debug_register_f32

Registers an editable 32-bit float.

**Signature:**
```rust
fn debug_register_f32(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```

**Example:**
```rust
static mut GRAVITY: f32 = 9.8;
static mut JUMP_FORCE: f32 = 15.0;

fn init() {
    unsafe {
        debug_register_f32(b"Gravity".as_ptr(), 7, &GRAVITY);
        debug_register_f32(b"Jump Force".as_ptr(), 10, &JUMP_FORCE);
    }
}
```

---

### debug_register_bool

Registers an editable boolean (checkbox).

**Signature:**
```rust
fn debug_register_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```

**Example:**
```rust
static mut GOD_MODE: u8 = 0; // 0 = false, 1 = true

fn init() {
    unsafe {
        debug_register_bool(b"God Mode".as_ptr(), 8, &GOD_MODE);
    }
}
```

---

### debug_register_u8 / u16 / u32

Registers unsigned integers.

```rust
fn debug_register_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_register_u16(name_ptr: *const u8, name_len: u32, ptr: *const u16)
fn debug_register_u32(name_ptr: *const u8, name_len: u32, ptr: *const u32)
```

---

### debug_register_i8 / i16

Registers signed integers.

```rust
fn debug_register_i8(name_ptr: *const u8, name_len: u32, ptr: *const i8)
fn debug_register_i16(name_ptr: *const u8, name_len: u32, ptr: *const i16)
```

---

## Range-Constrained Registration

### debug_register_i32_range

Registers an integer with min/max bounds (slider).

**Signature:**
```rust
fn debug_register_i32_range(
    name_ptr: *const u8, name_len: u32,
    ptr: *const i32,
    min: i32, max: i32
)
```

**Example:**
```rust
static mut DIFFICULTY: i32 = 2;

fn init() {
    unsafe {
        debug_register_i32_range(b"Difficulty".as_ptr(), 10, &DIFFICULTY, 1, 5);
    }
}
```

---

### debug_register_f32_range

Registers a float with min/max bounds.

**Signature:**
```rust
fn debug_register_f32_range(
    name_ptr: *const u8, name_len: u32,
    ptr: *const f32,
    min: f32, max: f32
)
```

**Example:**
```rust
static mut PLAYER_SPEED: f32 = 5.0;

fn init() {
    unsafe {
        debug_register_f32_range(b"Speed".as_ptr(), 5, &PLAYER_SPEED, 0.0, 20.0);
    }
}
```

---

### debug_register_u8_range / u16_range / i16_range

```rust
fn debug_register_u8_range(name_ptr: *const u8, name_len: u32, ptr: *const u8, min: u32, max: u32)
fn debug_register_u16_range(name_ptr: *const u8, name_len: u32, ptr: *const u16, min: u32, max: u32)
fn debug_register_i16_range(name_ptr: *const u8, name_len: u32, ptr: *const i16, min: i32, max: i32)
```

---

## Compound Types

### debug_register_vec2

Registers a 2D vector (two f32s).

**Signature:**
```rust
fn debug_register_vec2(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```

**Example:**
```rust
static mut PLAYER_POS: [f32; 2] = [0.0, 0.0];

fn init() {
    unsafe {
        debug_register_vec2(b"Player Pos".as_ptr(), 10, PLAYER_POS.as_ptr());
    }
}
```

---

### debug_register_vec3

Registers a 3D vector (three f32s).

**Signature:**
```rust
fn debug_register_vec3(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```

---

### debug_register_rect

Registers a rectangle (x, y, width, height as four f32s).

**Signature:**
```rust
fn debug_register_rect(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```

---

### debug_register_color

Registers a color (RGBA as four u8s).

**Signature:**
```rust
fn debug_register_color(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```

**Example:**
```rust
static mut TINT_COLOR: [u8; 4] = [255, 255, 255, 255];

fn init() {
    unsafe {
        debug_register_color(b"Tint".as_ptr(), 4, TINT_COLOR.as_ptr());
    }
}
```

---

## Fixed-Point Registration

For games using fixed-point math.

```rust
fn debug_register_fixed_i16_q8(name_ptr: *const u8, name_len: u32, ptr: *const i16)
fn debug_register_fixed_i32_q8(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_register_fixed_i32_q16(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_register_fixed_i32_q24(name_ptr: *const u8, name_len: u32, ptr: *const i32)
```

---

## Watch Functions (Read-Only)

Watches display values without allowing editing.

```rust
fn debug_watch_i8(name_ptr: *const u8, name_len: u32, ptr: *const i8)
fn debug_watch_i16(name_ptr: *const u8, name_len: u32, ptr: *const i16)
fn debug_watch_i32(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_watch_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_watch_u16(name_ptr: *const u8, name_len: u32, ptr: *const u16)
fn debug_watch_u32(name_ptr: *const u8, name_len: u32, ptr: *const u32)
fn debug_watch_f32(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_watch_vec2(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_vec3(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_rect(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_color(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```

**Example:**
```rust
static mut FRAME_COUNT: u32 = 0;
static mut FPS: f32 = 0.0;

fn init() {
    unsafe {
        debug_watch_u32(b"Frame".as_ptr(), 5, &FRAME_COUNT);
        debug_watch_f32(b"FPS".as_ptr(), 3, &FPS);
    }
}
```

---

## Grouping

### debug_group_begin

Starts a collapsible group in the debug panel.

**Signature:**
```rust
fn debug_group_begin(name_ptr: *const u8, name_len: u32)
```

---

### debug_group_end

Ends the current group.

**Signature:**
```rust
fn debug_group_end()
```

**Example:**
```rust
fn init() {
    unsafe {
        debug_group_begin(b"Player".as_ptr(), 6);
        debug_register_vec3(b"Position".as_ptr(), 8, PLAYER_POS.as_ptr());
        debug_register_f32(b"Health".as_ptr(), 6, &PLAYER_HEALTH);
        debug_register_f32(b"Speed".as_ptr(), 5, &PLAYER_SPEED);
        debug_group_end();

        debug_group_begin(b"Physics".as_ptr(), 7);
        debug_register_f32(b"Gravity".as_ptr(), 7, &GRAVITY);
        debug_register_f32(b"Friction".as_ptr(), 8, &FRICTION);
        debug_group_end();
    }
}
```

---

## Frame Control

### debug_is_paused

Check if game is paused via debug panel.

**Signature:**
```rust
fn debug_is_paused() -> i32
```

**Returns:** 1 if paused, 0 otherwise

---

### debug_get_time_scale

Get the current time scale.

**Signature:**
```rust
fn debug_get_time_scale() -> f32
```

**Returns:** Time scale (1.0 = normal, 0.5 = half speed, 2.0 = double)

**Example:**
```rust
fn update() {
    unsafe {
        if debug_is_paused() != 0 {
            return; // Skip update when paused
        }

        let dt = delta_time() * debug_get_time_scale();
        // Use scaled delta time
    }
}
```

---

## Debug Keyboard Shortcuts

| Key | Action |
|-----|--------|
| F3 | Toggle debug panel |
| F5 | Pause/unpause |
| F6 | Step one frame (while paused) |
| F7 | Decrease time scale |
| F8 | Increase time scale |

---

## Complete Example

```rust
// Game state
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Y: f32 = 0.0;
static mut PLAYER_VEL_X: f32 = 0.0;
static mut PLAYER_VEL_Y: f32 = 0.0;
static mut PLAYER_HEALTH: f32 = 100.0;

// Tuning parameters
static mut MOVE_SPEED: f32 = 5.0;
static mut JUMP_FORCE: f32 = 12.0;
static mut GRAVITY: f32 = 25.0;
static mut FRICTION: f32 = 0.9;

// Debug
static mut GOD_MODE: u8 = 0;
static mut SHOW_HITBOXES: u8 = 0;
static mut ENEMY_COUNT: i32 = 5;

fn init() {
    unsafe {
        // Player group
        debug_group_begin(b"Player".as_ptr(), 6);
        debug_watch_f32(b"X".as_ptr(), 1, &PLAYER_X);
        debug_watch_f32(b"Y".as_ptr(), 1, &PLAYER_Y);
        debug_watch_f32(b"Vel X".as_ptr(), 5, &PLAYER_VEL_X);
        debug_watch_f32(b"Vel Y".as_ptr(), 5, &PLAYER_VEL_Y);
        debug_register_f32_range(b"Health".as_ptr(), 6, &PLAYER_HEALTH, 0.0, 100.0);
        debug_group_end();

        // Physics group
        debug_group_begin(b"Physics".as_ptr(), 7);
        debug_register_f32_range(b"Move Speed".as_ptr(), 10, &MOVE_SPEED, 1.0, 20.0);
        debug_register_f32_range(b"Jump Force".as_ptr(), 10, &JUMP_FORCE, 5.0, 30.0);
        debug_register_f32_range(b"Gravity".as_ptr(), 7, &GRAVITY, 10.0, 50.0);
        debug_register_f32_range(b"Friction".as_ptr(), 8, &FRICTION, 0.5, 1.0);
        debug_group_end();

        // Debug options
        debug_group_begin(b"Debug".as_ptr(), 5);
        debug_register_bool(b"God Mode".as_ptr(), 8, &GOD_MODE);
        debug_register_bool(b"Show Hitboxes".as_ptr(), 13, &SHOW_HITBOXES);
        debug_register_i32_range(b"Enemy Count".as_ptr(), 11, &ENEMY_COUNT, 0, 20);
        debug_group_end();
    }
}

fn update() {
    unsafe {
        // Respect debug pause
        if debug_is_paused() != 0 {
            return;
        }

        let dt = delta_time() * debug_get_time_scale();

        // Use tunable values
        PLAYER_VEL_Y += GRAVITY * dt;
        PLAYER_VEL_X *= FRICTION;

        if button_held(0, BUTTON_RIGHT) != 0 {
            PLAYER_VEL_X = MOVE_SPEED;
        }
        if button_held(0, BUTTON_LEFT) != 0 {
            PLAYER_VEL_X = -MOVE_SPEED;
        }
        if button_pressed(0, BUTTON_A) != 0 {
            PLAYER_VEL_Y = -JUMP_FORCE;
        }

        PLAYER_X += PLAYER_VEL_X * dt;
        PLAYER_Y += PLAYER_VEL_Y * dt;
    }
}
```

**See Also:** [System Functions](./system.md)
