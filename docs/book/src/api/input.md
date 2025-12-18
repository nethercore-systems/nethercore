# Input Functions

Controller input handling for buttons, analog sticks, and triggers.

## Controller Layout

Emberware ZX uses a modern PS2/Xbox-style controller:

```
         [LB]                    [RB]
         [LT]                    [RT]
        +-----------------------------+
       |  [^]              [Y]        |
       | [<][>]    [=][=]  [X] [B]    |
       |  [v]              [A]        |
       |       [SELECT] [START]       |
       |        [L3]     [R3]         |
        +-----------------------------+
           Left      Right
           Stick     Stick
```

- **D-Pad:** 4 directions (digital)
- **Face buttons:** A, B, X, Y (digital)
- **Shoulder bumpers:** LB, RB (digital)
- **Triggers:** LT, RT (analog 0.0-1.0)
- **Sticks:** Left + Right (analog -1.0 to 1.0, clickable L3/R3)
- **Menu:** Start, Select (digital)

## Button Constants

```rust
// D-Pad
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;

// Face buttons
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;

// Shoulder bumpers
const BUTTON_LB: u32 = 8;
const BUTTON_RB: u32 = 9;

// Stick clicks
const BUTTON_L3: u32 = 10;
const BUTTON_R3: u32 = 11;

// Menu
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```

---

## Individual Button Queries

### button_held

Check if a button is currently held down.

**Signature:**
```rust
fn button_held(player: u32, button: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if held, `0` otherwise

**Example:**
```rust
fn update() {
    // Continuous movement while held
    if button_held(0, BUTTON_RIGHT) != 0 {
        player.x += MOVE_SPEED * delta_time();
    }
    if button_held(0, BUTTON_LEFT) != 0 {
        player.x -= MOVE_SPEED * delta_time();
    }
}
```

**See Also:** [button_pressed](#button_pressed), [button_released](#button_released)

---

### button_pressed

Check if a button was just pressed this tick (edge detection).

**Signature:**
```rust
fn button_pressed(player: u32, button: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if just pressed this tick, `0` otherwise

**Example:**
```rust
fn update() {
    // Jump only triggers once per press
    if button_pressed(0, BUTTON_A) != 0 && player.on_ground {
        player.velocity_y = JUMP_VELOCITY;
        play_sound(jump_sfx, 1.0, 0.0);
    }

    // Cycle weapons
    if button_pressed(0, BUTTON_RB) != 0 {
        current_weapon = (current_weapon + 1) % NUM_WEAPONS;
    }
}
```

**See Also:** [button_held](#button_held), [button_released](#button_released)

---

### button_released

Check if a button was just released this tick.

**Signature:**
```rust
fn button_released(player: u32, button: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if just released this tick, `0` otherwise

**Example:**
```rust
fn update() {
    // Variable jump height (release early = smaller jump)
    if button_released(0, BUTTON_A) != 0 && player.velocity_y < 0.0 {
        player.velocity_y *= 0.5; // Cut upward velocity
    }

    // Charged attack
    if button_released(0, BUTTON_X) != 0 {
        let power = charge_time.min(MAX_CHARGE);
        fire_charged_attack(power);
        charge_time = 0.0;
    }
}
```

**See Also:** [button_held](#button_held), [button_pressed](#button_pressed)

---

## Bulk Button Queries

For better performance when checking multiple buttons, use bulk queries to reduce FFI overhead.

### buttons_held

Get a bitmask of all currently held buttons.

**Signature:**
```rust
fn buttons_held(player: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Bitmask where bit N is set if button N is held

**Example:**
```rust
fn update() {
    let held = buttons_held(0);

    // Check multiple buttons efficiently
    if held & (1 << BUTTON_A) != 0 { /* A held */ }
    if held & (1 << BUTTON_B) != 0 { /* B held */ }

    // Check for combo (A + B held together)
    let combo = (1 << BUTTON_A) | (1 << BUTTON_B);
    if held & combo == combo {
        perform_combo_attack();
    }
}
```

**See Also:** [buttons_pressed](#buttons_pressed), [buttons_released](#buttons_released)

---

### buttons_pressed

Get a bitmask of all buttons pressed this tick.

**Signature:**
```rust
fn buttons_pressed(player: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Bitmask where bit N is set if button N was just pressed

**Example:**
```rust
fn update() {
    let pressed = buttons_pressed(0);

    // Check if any face button pressed
    let face_buttons = (1 << BUTTON_A) | (1 << BUTTON_B) |
                       (1 << BUTTON_X) | (1 << BUTTON_Y);
    if pressed & face_buttons != 0 {
        // Handle menu selection
    }
}
```

---

### buttons_released

Get a bitmask of all buttons released this tick.

**Signature:**
```rust
fn buttons_released(player: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Bitmask where bit N is set if button N was just released

---

## Analog Sticks

### left_stick_x

Get the left stick horizontal axis.

**Signature:**
```rust
fn left_stick_x(player: u32) -> f32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Value from `-1.0` (left) to `1.0` (right), `0.0` at center

**Example:**
```rust
fn update() {
    let stick_x = left_stick_x(0);

    // Apply deadzone
    let deadzone = 0.15;
    if stick_x.abs() > deadzone {
        player.x += stick_x * MOVE_SPEED * delta_time();
    }
}
```

---

### left_stick_y

Get the left stick vertical axis.

**Signature:**
```rust
fn left_stick_y(player: u32) -> f32
```

**Returns:** Value from `-1.0` (down) to `1.0` (up), `0.0` at center

---

### right_stick_x

Get the right stick horizontal axis.

**Signature:**
```rust
fn right_stick_x(player: u32) -> f32
```

**Returns:** Value from `-1.0` (left) to `1.0` (right)

**Example:**
```rust
fn update() {
    // Camera control with right stick
    camera_yaw += right_stick_x(0) * CAMERA_SPEED * delta_time();
}
```

---

### right_stick_y

Get the right stick vertical axis.

**Signature:**
```rust
fn right_stick_y(player: u32) -> f32
```

**Returns:** Value from `-1.0` (down) to `1.0` (up)

---

### left_stick

Get both left stick axes in a single FFI call (more efficient).

**Signature:**
```rust
fn left_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| out_x | `*mut f32` | Pointer to write X value |
| out_y | `*mut f32` | Pointer to write Y value |

**Example:**
```rust
fn update() {
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    left_stick(0, &mut x, &mut y);

    // Calculate magnitude for circular deadzone
    let mag = (x * x + y * y).sqrt();
    if mag > 0.15 {
        let nx = x / mag;
        let ny = y / mag;
        player.x += nx * MOVE_SPEED * delta_time();
        player.y += ny * MOVE_SPEED * delta_time();
    }
}
```

---

### right_stick

Get both right stick axes in a single FFI call.

**Signature:**
```rust
fn right_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
```

---

## Analog Triggers

### trigger_left

Get the left trigger (LT) value.

**Signature:**
```rust
fn trigger_left(player: u32) -> f32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Value from `0.0` (released) to `1.0` (fully pressed)

**Example:**
```rust
fn update() {
    let lt = trigger_left(0);

    // Brake with analog pressure
    if lt > 0.1 {
        vehicle.speed *= 1.0 - (lt * BRAKE_FORCE * delta_time());
    }
}
```

---

### trigger_right

Get the right trigger (RT) value.

**Signature:**
```rust
fn trigger_right(player: u32) -> f32
```

**Returns:** Value from `0.0` (released) to `1.0` (fully pressed)

**Example:**
```rust
fn update() {
    let rt = trigger_right(0);

    // Accelerate with analog pressure
    if rt > 0.1 {
        vehicle.speed += rt * ACCEL_FORCE * delta_time();
    }

    // Aiming zoom
    let zoom = 1.0 + rt * 2.0; // 1x to 3x zoom
    camera_fov(60.0 / zoom);
}
```

---

## Complete Input Example

```rust
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const MOVE_SPEED: f32 = 100.0;
const DEADZONE: f32 = 0.15;

static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Y: f32 = 0.0;
static mut ON_GROUND: bool = true;
static mut VEL_Y: f32 = 0.0;

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();

        // Movement with left stick
        let mut sx: f32 = 0.0;
        let mut sy: f32 = 0.0;
        left_stick(0, &mut sx, &mut sy);

        if sx.abs() > DEADZONE {
            PLAYER_X += sx * MOVE_SPEED * dt;
        }

        // Jump with A button
        if button_pressed(0, BUTTON_A) != 0 && ON_GROUND {
            VEL_Y = -300.0;
            ON_GROUND = false;
        }

        // Gravity
        VEL_Y += 800.0 * dt;
        PLAYER_Y += VEL_Y * dt;

        // Ground collision
        if PLAYER_Y >= 200.0 {
            PLAYER_Y = 200.0;
            VEL_Y = 0.0;
            ON_GROUND = true;
        }
    }
}
```
