# Button Constants

Quick reference for all button constants used with `button_pressed()` and `button_held()`.

## Standard Layout

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;      // Bottom face button (Xbox A, PlayStation X)
const BUTTON_B: u32 = 5;      // Right face button (Xbox B, PlayStation O)
const BUTTON_X: u32 = 6;      // Left face button (Xbox X, PlayStation Square)
const BUTTON_Y: u32 = 7;      // Top face button (Xbox Y, PlayStation Triangle)
const BUTTON_LB: u32 = 8;     // Left bumper
const BUTTON_RB: u32 = 9;     // Right bumper
const BUTTON_LT: u32 = 10;    // Left trigger (as button)
const BUTTON_RT: u32 = 11;    // Right trigger (as button)
const BUTTON_START: u32 = 12; // Start / Options
const BUTTON_SELECT: u32 = 13; // Select / Share / Back
const BUTTON_L3: u32 = 14;    // Left stick click
const BUTTON_R3: u32 = 15;    // Right stick click
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define NCZX_BUTTON_UP 0
#define NCZX_BUTTON_DOWN 1
#define NCZX_BUTTON_LEFT 2
#define NCZX_BUTTON_RIGHT 3
#define NCZX_BUTTON_A 4       // Bottom face button (Xbox A, PlayStation X)
#define NCZX_BUTTON_B 5       // Right face button (Xbox B, PlayStation O)
#define NCZX_BUTTON_X 6       // Left face button (Xbox X, PlayStation Square)
#define NCZX_BUTTON_Y 7       // Top face button (Xbox Y, PlayStation Triangle)
#define NCZX_BUTTON_LB 8      // Left bumper
#define NCZX_BUTTON_RB 9      // Right bumper
#define NCZX_BUTTON_LT 10     // Left trigger (as button)
#define NCZX_BUTTON_RT 11     // Right trigger (as button)
#define NCZX_BUTTON_START 12  // Start / Options
#define NCZX_BUTTON_SELECT 13 // Select / Share / Back
#define NCZX_BUTTON_L3 14     // Left stick click
#define NCZX_BUTTON_R3 15     // Right stick click
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;
    pub const a: u32 = 4;      // Bottom face button (Xbox A, PlayStation X)
    pub const b: u32 = 5;      // Right face button (Xbox B, PlayStation O)
    pub const x: u32 = 6;      // Left face button (Xbox X, PlayStation Square)
    pub const y: u32 = 7;      // Top face button (Xbox Y, PlayStation Triangle)
    pub const lb: u32 = 8;     // Left bumper
    pub const rb: u32 = 9;     // Right bumper
    pub const lt: u32 = 10;    // Left trigger (as button)
    pub const rt: u32 = 11;    // Right trigger (as button)
    pub const start: u32 = 12; // Start / Options
    pub const select: u32 = 13; // Select / Share / Back
    pub const l3: u32 = 14;    // Left stick click
    pub const r3: u32 = 15;    // Right stick click
};
```
{{#endtab}}

{{#endtabs}}

## Controller Mapping

| Nethercore | Xbox | PlayStation | Nintendo |
|-----------|------|-------------|----------|
| A | A | X (Cross) | B |
| B | B | O (Circle) | A |
| X | X | Square | Y |
| Y | Y | Triangle | X |
| LB | LB | L1 | L |
| RB | RB | R1 | R |
| START | Menu | Options | + |
| SELECT | View | Share | - |

## Input Functions

### Checking Button State

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Returns 1 on the frame button is first pressed, 0 otherwise
fn button_pressed(player: u32, button: u32) -> u32;

// Returns 1 every frame the button is held, 0 otherwise
fn button_held(player: u32, button: u32) -> u32;

// Returns 1 on the frame button is released, 0 otherwise
fn button_released(player: u32, button: u32) -> u32;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Returns 1 on the frame button is first pressed, 0 otherwise
NCZX_IMPORT uint32_t button_pressed(uint32_t player, uint32_t button);

// Returns 1 every frame the button is held, 0 otherwise
NCZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);

// Returns 1 on the frame button is released, 0 otherwise
NCZX_IMPORT uint32_t button_released(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Returns 1 on the frame button is first pressed, 0 otherwise
pub extern fn button_pressed(player: u32, button: u32) u32;

// Returns 1 every frame the button is held, 0 otherwise
pub extern fn button_held(player: u32, button: u32) u32;

// Returns 1 on the frame button is released, 0 otherwise
pub extern fn button_released(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

### Usage Examples

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Jump on button press
if button_pressed(0, BUTTON_A) != 0 {
    player_jump();
}

// Continuous movement while held
if button_held(0, BUTTON_LEFT) != 0 {
    player.x -= SPEED;
}

// Trigger on release (e.g., charge attack)
if button_released(0, BUTTON_X) != 0 {
    release_charged_attack();
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Jump on button press
if (button_pressed(0, NCZX_BUTTON_A) != 0) {
    player_jump();
}

// Continuous movement while held
if (button_held(0, NCZX_BUTTON_LEFT) != 0) {
    player.x -= SPEED;
}

// Trigger on release (e.g., charge attack)
if (button_released(0, NCZX_BUTTON_X) != 0) {
    release_charged_attack();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Jump on button press
if (button_pressed(0, Button.a) != 0) {
    player_jump();
}

// Continuous movement while held
if (button_held(0, Button.left) != 0) {
    player.x -= SPEED;
}

// Trigger on release (e.g., charge attack)
if (button_released(0, Button.x) != 0) {
    release_charged_attack();
}
```
{{#endtab}}

{{#endtabs}}

## Analog Input

For smooth movement, use the analog sticks:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn left_stick_x(player: u32) -> f32;   // -1.0 to 1.0
fn left_stick_y(player: u32) -> f32;   // -1.0 (up) to 1.0 (down)
fn right_stick_x(player: u32) -> f32;
fn right_stick_y(player: u32) -> f32;
fn left_trigger(player: u32) -> f32;   // 0.0 to 1.0
fn right_trigger(player: u32) -> f32;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float left_stick_x(uint32_t player);   // -1.0 to 1.0
NCZX_IMPORT float left_stick_y(uint32_t player);   // -1.0 (up) to 1.0 (down)
NCZX_IMPORT float right_stick_x(uint32_t player);
NCZX_IMPORT float right_stick_y(uint32_t player);
NCZX_IMPORT float left_trigger(uint32_t player);   // 0.0 to 1.0
NCZX_IMPORT float right_trigger(uint32_t player);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn left_stick_x(player: u32) f32;   // -1.0 to 1.0
pub extern fn left_stick_y(player: u32) f32;   // -1.0 (up) to 1.0 (down)
pub extern fn right_stick_x(player: u32) f32;
pub extern fn right_stick_y(player: u32) f32;
pub extern fn left_trigger(player: u32) f32;   // 0.0 to 1.0
pub extern fn right_trigger(player: u32) f32;
```
{{#endtab}}

{{#endtabs}}

## Multiple Players

All input functions take a `player` parameter (0-3):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Player 1 (index 0)
let p1_x = left_stick_x(0);

// Player 2 (index 1)
let p2_x = left_stick_x(1);

// Check how many players are connected
let count = player_count();
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Player 1 (index 0)
float p1_x = left_stick_x(0);

// Player 2 (index 1)
float p2_x = left_stick_x(1);

// Check how many players are connected
uint32_t count = player_count();
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Player 1 (index 0)
const p1_x = left_stick_x(0);

// Player 2 (index 1)
const p2_x = left_stick_x(1);

// Check how many players are connected
const count = player_count();
```
{{#endtab}}

{{#endtabs}}

## Copy-Paste Template

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;
const BUTTON_LB: u32 = 8;
const BUTTON_RB: u32 = 9;
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Button constants
#define NCZX_BUTTON_UP 0
#define NCZX_BUTTON_DOWN 1
#define NCZX_BUTTON_LEFT 2
#define NCZX_BUTTON_RIGHT 3
#define NCZX_BUTTON_A 4
#define NCZX_BUTTON_B 5
#define NCZX_BUTTON_X 6
#define NCZX_BUTTON_Y 7
#define NCZX_BUTTON_LB 8
#define NCZX_BUTTON_RB 9
#define NCZX_BUTTON_START 12
#define NCZX_BUTTON_SELECT 13
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Button constants
const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;
    pub const a: u32 = 4;
    pub const b: u32 = 5;
    pub const x: u32 = 6;
    pub const y: u32 = 7;
    pub const lb: u32 = 8;
    pub const rb: u32 = 9;
    pub const start: u32 = 12;
    pub const select: u32 = 13;
};
```
{{#endtab}}

{{#endtabs}}
