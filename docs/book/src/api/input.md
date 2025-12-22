# Input Functions

Controller input handling for buttons, analog sticks, and triggers.

## Controller Layout

Nethercore ZX uses a modern PS2/Xbox-style controller:

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
// D-Pad
#define NCZX_BUTTON_UP      0
#define NCZX_BUTTON_DOWN    1
#define NCZX_BUTTON_LEFT    2
#define NCZX_BUTTON_RIGHT   3

// Face buttons
#define NCZX_BUTTON_A       4
#define NCZX_BUTTON_B       5
#define NCZX_BUTTON_X       6
#define NCZX_BUTTON_Y       7

// Shoulder bumpers
#define NCZX_BUTTON_L1      8
#define NCZX_BUTTON_R1      9

// Stick clicks
#define NCZX_BUTTON_L3      10
#define NCZX_BUTTON_R3      11

// Menu
#define NCZX_BUTTON_START   12
#define NCZX_BUTTON_SELECT  13
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const Button = struct {
    // D-Pad
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;

    // Face buttons
    pub const a: u32 = 4;
    pub const b: u32 = 5;
    pub const x: u32 = 6;
    pub const y: u32 = 7;

    // Shoulder bumpers
    pub const l1: u32 = 8;
    pub const r1: u32 = 9;

    // Stick clicks
    pub const l3: u32 = 10;
    pub const r3: u32 = 11;

    // Menu
    pub const start: u32 = 12;
    pub const select: u32 = 13;
};
```
{{#endtab}}

{{#endtabs}}

---

## Individual Button Queries

### button_held

Check if a button is currently held down.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn button_held(player: u32, button: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn button_held(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if held, `0` otherwise

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Continuous movement while held */
    if (button_held(0, NCZX_BUTTON_RIGHT)) {
        player_x += MOVE_SPEED * delta_time();
    }
    if (button_held(0, NCZX_BUTTON_LEFT)) {
        player_x -= MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Continuous movement while held
    if (button_held(0, Button.right) != 0) {
        player_x += MOVE_SPEED * delta_time();
    }
    if (button_held(0, Button.left) != 0) {
        player_x -= MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [button_pressed](#button_pressed), [button_released](#button_released)

---

### button_pressed

Check if a button was just pressed this tick (edge detection).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn button_pressed(player: u32, button: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t button_pressed(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn button_pressed(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if just pressed this tick, `0` otherwise

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Jump only triggers once per press */
    if (button_pressed(0, NCZX_BUTTON_A) && on_ground) {
        velocity_y = JUMP_VELOCITY;
        play_sound(jump_sfx, 1.0f, 0.0f);
    }

    /* Cycle weapons */
    if (button_pressed(0, NCZX_BUTTON_R1)) {
        current_weapon = (current_weapon + 1) % NUM_WEAPONS;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Jump only triggers once per press
    if (button_pressed(0, Button.a) != 0 and on_ground) {
        velocity_y = JUMP_VELOCITY;
        play_sound(jump_sfx, 1.0, 0.0);
    }

    // Cycle weapons
    if (button_pressed(0, Button.r1) != 0) {
        current_weapon = (current_weapon + 1) % NUM_WEAPONS;
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [button_held](#button_held), [button_released](#button_released)

---

### button_released

Check if a button was just released this tick.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn button_released(player: u32, button: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t button_released(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn button_released(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| button | `u32` | Button constant (0-13) |

**Returns:** `1` if just released this tick, `0` otherwise

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Variable jump height (release early = smaller jump) */
    if (button_released(0, NCZX_BUTTON_A) && velocity_y < 0.0f) {
        velocity_y *= 0.5f; /* Cut upward velocity */
    }

    /* Charged attack */
    if (button_released(0, NCZX_BUTTON_X)) {
        float power = nczx_minf(charge_time, MAX_CHARGE);
        fire_charged_attack(power);
        charge_time = 0.0f;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Variable jump height (release early = smaller jump)
    if (button_released(0, Button.a) != 0 and velocity_y < 0.0) {
        velocity_y *= 0.5; // Cut upward velocity
    }

    // Charged attack
    if (button_released(0, Button.x) != 0) {
        const power = @min(charge_time, MAX_CHARGE);
        fire_charged_attack(power);
        charge_time = 0.0;
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [button_held](#button_held), [button_pressed](#button_pressed)

---

## Bulk Button Queries

For better performance when checking multiple buttons, use bulk queries to reduce FFI overhead.

### buttons_held

Get a bitmask of all currently held buttons.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn buttons_held(player: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t buttons_held(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn buttons_held(player: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Bitmask where bit N is set if button N is held

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    uint32_t held = buttons_held(0);

    /* Check multiple buttons efficiently */
    if (held & (1 << NCZX_BUTTON_A)) { /* A held */ }
    if (held & (1 << NCZX_BUTTON_B)) { /* B held */ }

    /* Check for combo (A + B held together) */
    uint32_t combo = (1 << NCZX_BUTTON_A) | (1 << NCZX_BUTTON_B);
    if ((held & combo) == combo) {
        perform_combo_attack();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const held = buttons_held(0);

    // Check multiple buttons efficiently
    if (held & (1 << Button.a) != 0) { /* A held */ }
    if (held & (1 << Button.b) != 0) { /* B held */ }

    // Check for combo (A + B held together)
    const combo = (1 << Button.a) | (1 << Button.b);
    if (held & combo == combo) {
        perform_combo_attack();
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [buttons_pressed](#buttons_pressed), [buttons_released](#buttons_released)

---

### buttons_pressed

Get a bitmask of all buttons pressed this tick.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn buttons_pressed(player: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t buttons_pressed(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn buttons_pressed(player: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Bitmask where bit N is set if button N was just pressed

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    uint32_t pressed = buttons_pressed(0);

    /* Check if any face button pressed */
    uint32_t face_buttons = (1 << NCZX_BUTTON_A) | (1 << NCZX_BUTTON_B) |
                            (1 << NCZX_BUTTON_X) | (1 << NCZX_BUTTON_Y);
    if (pressed & face_buttons) {
        /* Handle menu selection */
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const pressed = buttons_pressed(0);

    // Check if any face button pressed
    const face_buttons = (1 << Button.a) | (1 << Button.b) |
                         (1 << Button.x) | (1 << Button.y);
    if (pressed & face_buttons != 0) {
        // Handle menu selection
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### buttons_released

Get a bitmask of all buttons released this tick.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn buttons_released(player: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t buttons_released(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn buttons_released(player: u32) u32;
```
{{#endtab}}

{{#endtabs}}

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn left_stick_x(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float left_stick_x(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn left_stick_x(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Value from `-1.0` (left) to `1.0` (right), `0.0` at center

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    float stick_x = left_stick_x(0);

    /* Apply deadzone */
    float deadzone = 0.15f;
    if (nczx_absf(stick_x) > deadzone) {
        player_x += stick_x * MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const stick_x = left_stick_x(0);

    // Apply deadzone
    const deadzone = 0.15;
    if (@abs(stick_x) > deadzone) {
        player_x += stick_x * MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### left_stick_y

Get the left stick vertical axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn left_stick_y(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float left_stick_y(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn left_stick_y(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Value from `-1.0` (down) to `1.0` (up), `0.0` at center

---

### right_stick_x

Get the right stick horizontal axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn right_stick_x(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float right_stick_x(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn right_stick_x(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Value from `-1.0` (left) to `1.0` (right)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    // Camera control with right stick
    camera_yaw += right_stick_x(0) * CAMERA_SPEED * delta_time();
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    /* Camera control with right stick */
    camera_yaw += right_stick_x(0) * CAMERA_SPEED * delta_time();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Camera control with right stick
    camera_yaw += right_stick_x(0) * CAMERA_SPEED * delta_time();
}
```
{{#endtab}}

{{#endtabs}}

---

### right_stick_y

Get the right stick vertical axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn right_stick_y(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float right_stick_y(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn right_stick_y(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Value from `-1.0` (down) to `1.0` (up)

---

### left_stick

Get both left stick axes in a single FFI call (more efficient).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn left_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void left_stick(uint32_t player, float* out_x, float* out_y);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn left_stick(player: u32, out_x: *f32, out_y: *f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |
| out_x | `*mut f32` | Pointer to write X value |
| out_y | `*mut f32` | Pointer to write Y value |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    float x = 0.0f;
    float y = 0.0f;
    left_stick(0, &x, &y);

    /* Calculate magnitude for circular deadzone */
    float mag = sqrtf(x * x + y * y);
    if (mag > 0.15f) {
        float nx = x / mag;
        float ny = y / mag;
        player_x += nx * MOVE_SPEED * delta_time();
        player_y += ny * MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    var x: f32 = 0.0;
    var y: f32 = 0.0;
    left_stick(0, &x, &y);

    // Calculate magnitude for circular deadzone
    const mag = @sqrt(x * x + y * y);
    if (mag > 0.15) {
        const nx = x / mag;
        const ny = y / mag;
        player_x += nx * MOVE_SPEED * delta_time();
        player_y += ny * MOVE_SPEED * delta_time();
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### right_stick

Get both right stick axes in a single FFI call.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn right_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void right_stick(uint32_t player, float* out_x, float* out_y);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn right_stick(player: u32, out_x: *f32, out_y: *f32) void;
```
{{#endtab}}

{{#endtabs}}

---

## Analog Triggers

### trigger_left

Get the left trigger (LT) value.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn trigger_left(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float trigger_left(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn trigger_left(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| player | `u32` | Player index (0-3) |

**Returns:** Value from `0.0` (released) to `1.0` (fully pressed)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    let lt = trigger_left(0);

    // Brake with analog pressure
    if lt > 0.1 {
        vehicle.speed *= 1.0 - (lt * BRAKE_FORCE * delta_time());
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    float lt = trigger_left(0);

    /* Brake with analog pressure */
    if (lt > 0.1f) {
        vehicle_speed *= 1.0f - (lt * BRAKE_FORCE * delta_time());
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const lt = trigger_left(0);

    // Brake with analog pressure
    if (lt > 0.1) {
        vehicle_speed *= 1.0 - (lt * BRAKE_FORCE * delta_time());
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### trigger_right

Get the right trigger (RT) value.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn trigger_right(player: u32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float trigger_right(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn trigger_right(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Value from `0.0` (released) to `1.0` (fully pressed)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    float rt = trigger_right(0);

    /* Accelerate with analog pressure */
    if (rt > 0.1f) {
        vehicle_speed += rt * ACCEL_FORCE * delta_time();
    }

    /* Aiming zoom */
    float zoom = 1.0f + rt * 2.0f; /* 1x to 3x zoom */
    camera_fov(60.0f / zoom);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    const rt = trigger_right(0);

    // Accelerate with analog pressure
    if (rt > 0.1) {
        vehicle_speed += rt * ACCEL_FORCE * delta_time();
    }

    // Aiming zoom
    const zoom = 1.0 + rt * 2.0; // 1x to 3x zoom
    camera_fov(60.0 / zoom);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Input Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define MOVE_SPEED 100.0f
#define DEADZONE 0.15f

static float player_x = 0.0f;
static float player_y = 0.0f;
static int on_ground = 1;
static float vel_y = 0.0f;

NCZX_EXPORT void update(void) {
    float dt = delta_time();

    /* Movement with left stick */
    float sx = 0.0f;
    float sy = 0.0f;
    left_stick(0, &sx, &sy);

    if (nczx_absf(sx) > DEADZONE) {
        player_x += sx * MOVE_SPEED * dt;
    }

    /* Jump with A button */
    if (button_pressed(0, NCZX_BUTTON_A) && on_ground) {
        vel_y = -300.0f;
        on_ground = 0;
    }

    /* Gravity */
    vel_y += 800.0f * dt;
    player_y += vel_y * dt;

    /* Ground collision */
    if (player_y >= 200.0f) {
        player_y = 200.0f;
        vel_y = 0.0f;
        on_ground = 1;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const MOVE_SPEED: f32 = 100.0;
const DEADZONE: f32 = 0.15;

var player_x: f32 = 0.0;
var player_y: f32 = 0.0;
var on_ground: bool = true;
var vel_y: f32 = 0.0;

export fn update() void {
    const dt = delta_time();

    // Movement with left stick
    var sx: f32 = 0.0;
    var sy: f32 = 0.0;
    left_stick(0, &sx, &sy);

    if (@abs(sx) > DEADZONE) {
        player_x += sx * MOVE_SPEED * dt;
    }

    // Jump with A button
    if (button_pressed(0, Button.a) != 0 and on_ground) {
        vel_y = -300.0;
        on_ground = false;
    }

    // Gravity
    vel_y += 800.0 * dt;
    player_y += vel_y * dt;

    // Ground collision
    if (player_y >= 200.0) {
        player_y = 200.0;
        vel_y = 0.0;
        on_ground = true;
    }
}
```
{{#endtab}}

{{#endtabs}}
