# Emberware Classic — Console Specification

Emberware Classic is a beginner-friendly 2D fantasy console inspired by SNES/Genesis era hardware. Designed for students, junior developers, and anyone learning 2D game development.

## Console Specs

| Spec | Value |
|------|-------|
| **Aesthetic** | SNES/Genesis (4th gen pixel art) |
| **Resolution** | 8 options: 4× 16:9, 4× 4:3 (see below) |
| **Scaling** | Pixel-perfect to 1080p |
| **Color depth** | RGBA8 |
| **Tick rate** | 30, 60 (default) fps |
| **RAM** | 1MB |
| **VRAM** | 1MB |
| **CPU budget** | 4ms per tick (at 60fps) |
| **ROM size** | 4MB max (uncompressed) |
| **Netcode** | Deterministic rollback via GGRS |
| **Max players** | 4 (any mix of local + remote) |

### Resolution Options

**16:9** (pixel-perfect to 1920×1080):

| Resolution | Scale | Output |
|------------|-------|--------|
| 320×180 | 6× | 1920×1080 |
| 384×216 | 5× | 1920×1080 |
| 480×270 | 4× | 1920×1080 |
| 640×360 | 3× | 1920×1080 |

**4:3** (pixel-perfect to 1440×1080, pillarboxed on 16:9 displays):

| Resolution | Scale | Output |
|------------|-------|--------|
| 240×180 | 6× | 1440×1080 |
| 288×216 | 5× | 1440×1080 |
| 360×270 | 4× | 1440×1080 |
| 480×360 | 3× | 1440×1080 |

### Configuration (init-only)

These settings **must be called in `init()`** — they cannot be changed at runtime.

```rust
fn set_resolution(res: u32)                 // 0-7, see resolution options below
fn set_tick_rate(fps: u32)                  // 24, 30, 60 (default), or 120
fn set_clear_color(color: u32)              // 0xRRGGBBAA, default: 0x000000FF (black)
```

**Resolution enum values:**

| Value | Resolution | Aspect | Scale |
|-------|------------|--------|-------|
| 0 | 320×180 | 16:9 | 6× |
| 1 | 384×216 | 16:9 | 5× |
| 2 | 480×270 | 16:9 | 4× |
| 3 | 640×360 | 16:9 | 3× |
| 4 | 240×180 | 4:3 | 6× |
| 5 | 288×216 | 4:3 | 5× (default) |
| 6 | 360×270 | 4:3 | 4× |
| 7 | 480×360 | 4:3 | 3× |

If not set, defaults to resolution 5 (288×216, 4:3) @ 60fps.

---

## Controller

Emberware Classic uses a 6-button retro controller (inspired by Genesis/Saturn 6-button):

```
              [L]                    [R]
        ┌─────────────────────────────┐
       │           [^]                │
       │          [<][>]              │
       │           [v]                │
       │                              │
       │     [X] [Y] [Z]              │
       │     [A] [B] [C]              │
       │                              │
       │        [SELECT] [START]      │
        └─────────────────────────────┘
```

- **D-Pad:** 4 directions (no analog)
- **Face buttons:** A, B, C (bottom row), X, Y, Z (top row)
- **Shoulder buttons:** L, R (digital)
- **Menu:** Start, Select
- **No analog sticks or triggers**

### Button Constants

```rust
// D-Pad
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;

// Face buttons (bottom row)
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_C: u32 = 6;

// Face buttons (top row)
const BUTTON_X: u32 = 7;
const BUTTON_Y: u32 = 8;
const BUTTON_Z: u32 = 9;

// Shoulder buttons
const BUTTON_L: u32 = 10;
const BUTTON_R: u32 = 11;

// Menu
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```

---

## Input FFI

### Individual Button Queries (Convenient)

```rust
fn button_held(player: u32, button: u32) -> u32     // 1 if held, 0 otherwise
fn button_pressed(player: u32, button: u32) -> u32  // 1 if just pressed this tick
fn button_released(player: u32, button: u32) -> u32 // 1 if just released this tick
```

### Bulk Button Queries (Efficient)

```rust
fn buttons_held(player: u32) -> u32     // Bitmask of all held buttons
fn buttons_pressed(player: u32) -> u32  // Bitmask of all just pressed
fn buttons_released(player: u32) -> u32 // Bitmask of all just released
```

Use bulk queries when checking multiple buttons to reduce FFI overhead:

```rust
let held = buttons_held(0);
if held & (1 << BUTTON_A) != 0 { /* A held */ }
if held & (1 << BUTTON_B) != 0 { /* B held */ }
```

### D-Pad Helpers

```rust
fn dpad_x(player: u32) -> i32  // -1 (left), 0, or 1 (right)
fn dpad_y(player: u32) -> i32  // -1 (up), 0, or 1 (down)
```

Convenience functions for movement code.

**Note:** Emberware Classic has no analog sticks or triggers.

---

## Graphics FFI

### Frame Handling

The runtime automatically:
- Clears the screen to `set_clear_color()` before each `render()` call
- Presents the frame after `render()` returns

No manual `frame_begin()`/`frame_end()` calls needed.

### Textures

Games embed assets via `include_bytes!()` and pass raw pixels — no file-based loading. All resources are created in `init()` and automatically cleaned up on game shutdown.

```rust
fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32
fn texture_bind(handle: u32)
```

### Sprites

**Simple (for beginners):**

```rust
fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32)
fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32)
fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32)
```

**With source region (for sprite sheets):**

```rust
fn draw_sprite_region(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    color: u32
)
```

**Full control (region + flip):**

```rust
fn draw_sprite_ex(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    flip_h: u32, flip_v: u32,       // 1 = flip, 0 = normal
    color: u32
)
```

**Legacy (flip without region):**

```rust
fn draw_sprite_flipped(
    x: f32, y: f32, w: f32, h: f32,
    flip_h: u32, flip_v: u32,
    color: u32
)
```

### Sprite Priority

```rust
fn sprite_layer(layer: u32)             // 0-3, higher = in front
```

Set which layer subsequent sprites draw to. Layers are drawn back-to-front.

### Tilemaps

Classic supports multiple tilemap layers for parallax scrolling:

```rust
fn tilemap_create(
    tile_width: u32,
    tile_height: u32,
    map_width: u32,
    map_height: u32,
    layer: u32                          // 0-3 for parallax ordering
) -> u32

fn tilemap_set_texture(map: u32, texture: u32)
fn tilemap_set_tile(map: u32, x: u32, y: u32, tile_id: u16)
fn tilemap_set_tiles(map: u32, tiles_ptr: *const u16, len: u32)
fn tilemap_scroll(map: u32, offset_x: f32, offset_y: f32)
```

Tilemaps are drawn automatically each frame in layer order (0 = back, 3 = front). Sprites can be interleaved with tilemap layers using `sprite_layer()`.

### Palette Swapping

```rust
fn palette_create(colors_ptr: *const u32, count: u32) -> u32
fn palette_bind(handle: u32)            // 0 = no palette (use texture colors)
```

When a palette is bound, texture color indices (R channel, 0-255) are remapped to palette colors. Useful for enemy color variants, damage flash, etc.

### Render State

```rust
fn blend_mode(mode: u32)                // 0 = none, 1 = alpha, 2 = additive
fn texture_filter(filter: u32)          // 0 = nearest (default), 1 = linear
```

---

## Audio FFI

> **TODO [needs clarification]:** Audio system is shelved for initial implementation.

---

## Complete Example

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "emberware")]
extern "C" {
    fn set_clear_color(color: u32);
    fn button_pressed(player: u32, button: u32) -> u32;
    fn dpad_x(player: u32) -> i32;
    fn dpad_y(player: u32) -> i32;
    fn player_count() -> u32;
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

const BUTTON_A: u32 = 4;

static mut PLAYER_X: [f32; 4] = [160.0; 4];
static mut PLAYER_Y: [f32; 4] = [112.0; 4];
static mut JUMPING: [bool; 4] = [false; 4];
static mut VELOCITY_Y: [f32; 4] = [0.0; 4];

const SPEED: f32 = 3.0;
const JUMP_FORCE: f32 = -8.0;
const GRAVITY: f32 = 0.5;
const GROUND_Y: f32 = 180.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe { set_clear_color(0x87CEEBFF); } // Sky blue
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        for p in 0..player_count() {
            let i = p as usize;

            // Horizontal movement via D-pad
            PLAYER_X[i] += (dpad_x(p) as f32) * SPEED;

            // Jump with A button
            if button_pressed(p, BUTTON_A) != 0 && !JUMPING[i] {
                VELOCITY_Y[i] = JUMP_FORCE;
                JUMPING[i] = true;
            }

            // Apply gravity
            VELOCITY_Y[i] += GRAVITY;
            PLAYER_Y[i] += VELOCITY_Y[i];

            // Ground collision
            if PLAYER_Y[i] >= GROUND_Y {
                PLAYER_Y[i] = GROUND_Y;
                VELOCITY_Y[i] = 0.0;
                JUMPING[i] = false;
            }

            // Screen bounds
            PLAYER_X[i] = PLAYER_X[i].clamp(0.0, 368.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Ground
        draw_rect(0.0, 200.0, 384.0, 16.0, 0x228B22FF);

        // Players
        let colors = [0xFF0000FF, 0x0000FFFF, 0x00FF00FF, 0xFFFF00FF];
        for p in 0..player_count() as usize {
            draw_rect(PLAYER_X[p], PLAYER_Y[p], 16.0, 20.0, colors[p]);
        }

        let title = b"Classic Platformer";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 8.0, 0x000000FF);
    }
}
```

---

## Differences from Emberware Z

| Feature | Emberware Z | Emberware Classic |
|---------|-------------|-------------------|
| Target audience | Experienced devs | Beginners, students |
| Generation | 5th (PS1/N64) | 4th (SNES/Genesis) |
| Resolution | 360p-1080p | 320×180 to 640×360 (pixel-perfect) |
| RAM | 4MB | 1MB |
| VRAM | 4MB | 1MB |
| ROM size | 8MB | 4MB |
| 3D support | Yes | No |
| Analog sticks | 2 | None |
| Analog triggers | 2 | None |
| Face buttons | 4 (A/B/X/Y) | 6 (A/B/C/X/Y/Z) |
| Tilemap layers | No | Yes (4 layers) |
| Sprite flip | No | Yes (H/V) |
| Sprite priority | No | Yes (4 layers) |
| Palette swap | No | Yes |
