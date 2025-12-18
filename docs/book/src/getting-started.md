# Getting Started

Create your first Emberware ZX game in minutes.

## Prerequisites

- [Rust](https://rustup.rs/) installed
- WASM target: `rustup target add wasm32-unknown-unknown`

## Create a Project

```bash
cargo new --lib my-game
cd my-game
```

## Configure Cargo.toml

```toml
[package]
name = "my-game"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
```

## Write Your Game

Replace `src/lib.rs`:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// FFI imports
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
    fn camera_set(x: f32, y: f32, z: f32, tx: f32, ty: f32, tz: f32);
    fn cube(sx: f32, sy: f32, sz: f32) -> u32;
    fn draw_mesh(handle: u32);
    fn push_identity();
    fn push_rotate_y(deg: f32);
    fn set_color(color: u32);
    fn elapsed_time() -> f32;
    fn delta_time() -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;
}

// Game state
static mut CUBE: u32 = 0;
static mut ANGLE: f32 = 0.0;
static mut PLAYER_X: f32 = 0.0;

const BUTTON_A: u32 = 4;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(0); // Unlit

        CUBE = cube(0.5, 0.5, 0.5);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();

        // Rotate cube
        ANGLE += 90.0 * dt;

        // Move with left stick
        PLAYER_X += left_stick_x(0) * 5.0 * dt;

        // Reset on A button
        if button_pressed(0, BUTTON_A) != 0 {
            PLAYER_X = 0.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        camera_set(0.0, 2.0, 5.0, 0.0, 0.0, 0.0);

        push_identity();
        push_rotate_y(ANGLE);
        set_color(0x4488FFFF);
        draw_mesh(CUBE);
    }
}
```

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/my_game.wasm`

## Run

Load the WASM file in the Emberware library or use `ember run`:

```bash
ember run target/wasm32-unknown-unknown/release/my_game.wasm
```

---

## Next Steps

### Add a Texture

```rust
static mut TEXTURE: u32 = 0;

// Checkerboard pixels (8x8)
const PIXELS: [u8; 8 * 8 * 4] = { /* ... */ };

fn init() {
    unsafe {
        TEXTURE = load_texture(8, 8, PIXELS.as_ptr());
    }
}

fn render() {
    unsafe {
        texture_bind(TEXTURE);
        draw_mesh(CUBE);
    }
}
```

### Add Lighting

```rust
fn init() {
    render_mode(2); // PBR lighting
    sky_set_colors(0xB2D8F2FF, 0x3366B2FF);
    sky_set_sun(0.5, 0.7, 0.5, 0xFFF2E6FF, 0.95);
}

fn render() {
    draw_sky();

    light_set(0, 0.5, -0.7, 0.5);
    light_enable(0);

    material_metallic(0.0);
    material_roughness(0.5);
    draw_mesh(CUBE);
}
```

### Add Sound

```rust
static mut JUMP_SFX: u32 = 0;

fn init() {
    JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
}

fn update() {
    if button_pressed(0, BUTTON_A) != 0 {
        play_sound(JUMP_SFX, 1.0, 0.0);
    }
}
```

### Use ROM Assets

Create `ember.toml`:

```toml
[game]
id = "my-game"
title = "My Game"
author = "Your Name"
version = "1.0.0"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.meshes]]
id = "level"
path = "assets/level.ewzmesh"
```

Build with:
```bash
ember build
ember pack
```

---

## Project Structure

```
my-game/
├── Cargo.toml
├── ember.toml          # Asset manifest
├── src/
│   └── lib.rs          # Game code
└── assets/
    ├── textures/
    ├── meshes/
    └── sounds/
```

---

## Quick Reference

### Game Lifecycle

| Function | Called | Purpose |
|----------|--------|---------|
| `init()` | Once at startup | Load resources, configure console |
| `update()` | Every tick | Game logic (deterministic!) |
| `render()` | Every frame | Drawing (skipped during rollback) |

### Essential Functions

```rust
// Configuration (init-only)
set_resolution(1);        // 0=360p, 1=540p, 2=720p, 3=1080p
set_tick_rate(2);         // 0=24, 1=30, 2=60, 3=120 fps
set_clear_color(0x000000FF);
render_mode(0);           // 0-3

// Time
delta_time() -> f32       // Seconds since last tick
elapsed_time() -> f32     // Total seconds
tick_count() -> u64       // Frame number

// Input
button_pressed(player, button) -> u32
button_held(player, button) -> u32
left_stick_x(player) -> f32
left_stick_y(player) -> f32

// Camera
camera_set(x, y, z, target_x, target_y, target_z)
camera_fov(degrees)

// Transforms
push_identity()
push_translate(x, y, z)
push_rotate_y(degrees)
push_scale_uniform(s)

// Drawing
draw_mesh(handle)
draw_sprite(x, y, w, h, color)
draw_text(ptr, len, x, y, size, color)
```

### Button Constants

```rust
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

---

## Learn More

- [API Reference](./api/system.md) - All functions documented
- [Cheat Sheet](./cheat-sheet.md) - Quick reference
- [Render Modes](./guides/render-modes.md) - Lighting guide
- [Rollback Safety](./guides/rollback-safety.md) - Netcode guide
