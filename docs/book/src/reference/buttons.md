# Button Constants

Quick reference for all button constants used with `button_pressed()` and `button_held()`.

## Standard Layout

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

## Controller Mapping

| Emberware | Xbox | PlayStation | Nintendo |
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

```rust
// Returns 1 on the frame button is first pressed, 0 otherwise
fn button_pressed(player: u32, button: u32) -> u32;

// Returns 1 every frame the button is held, 0 otherwise
fn button_held(player: u32, button: u32) -> u32;

// Returns 1 on the frame button is released, 0 otherwise
fn button_released(player: u32, button: u32) -> u32;
```

### Usage Examples

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

## Analog Input

For smooth movement, use the analog sticks:

```rust
fn left_stick_x(player: u32) -> f32;   // -1.0 to 1.0
fn left_stick_y(player: u32) -> f32;   // -1.0 (up) to 1.0 (down)
fn right_stick_x(player: u32) -> f32;
fn right_stick_y(player: u32) -> f32;
fn left_trigger(player: u32) -> f32;   // 0.0 to 1.0
fn right_trigger(player: u32) -> f32;
```

## Multiple Players

All input functions take a `player` parameter (0-3):

```rust
// Player 1 (index 0)
let p1_x = left_stick_x(0);

// Player 2 (index 1)
let p2_x = left_stick_x(1);

// Check how many players are connected
let count = player_count();
```

## Copy-Paste Template

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
