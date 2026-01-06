# Nethercore Developer Guide

This guide walks you through creating your first Nethercore game, explains best practices for rollback-safe code, and provides debugging tips.

**Prerequisites:**
- Rust (rustup installed)
- Basic Rust knowledge

---

## Getting Started

### 1. Install the WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

### 2. Create a New Project

```bash
cargo new --lib my-game
cd my-game
```

### 3. Configure Cargo.toml

```toml
[package]
name = "my-game"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"    # Optimize for size
lto = true         # Link-time optimization
```

### 4. Write Your Game

Replace `src/lib.rs` with:

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

// Import FFI functions from the host
#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn button_held(player: u32, button: u32) -> u32;
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Game state
static mut PLAYER_X: f32 = 160.0;
static mut PLAYER_Y: f32 = 120.0;

// Called once at startup
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF); // Dark blue background
    }
}

// Called every tick (deterministic!)
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        const SPEED: f32 = 3.0;

        // D-pad movement
        if button_held(0, 0) != 0 { PLAYER_Y -= SPEED; } // Up
        if button_held(0, 1) != 0 { PLAYER_Y += SPEED; } // Down
        if button_held(0, 2) != 0 { PLAYER_X -= SPEED; } // Left
        if button_held(0, 3) != 0 { PLAYER_X += SPEED; } // Right

        // Keep player on screen
        PLAYER_X = PLAYER_X.clamp(0.0, 300.0);
        PLAYER_Y = PLAYER_Y.clamp(0.0, 220.0);
    }
}

// Called every frame
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw player
        draw_rect(PLAYER_X, PLAYER_Y, 20.0, 20.0, 0x4a9fffFF);

        // Draw title
        let text = b"My First Game";
        draw_text(text.as_ptr(), text.len() as u32, 10.0, 10.0, 12.0, 0xFFFFFFFF);
    }
}
```

### 5. Build Your Game

```bash
cargo build --target wasm32-unknown-unknown --release
```

Your game is at `target/wasm32-unknown-unknown/release/my_game.wasm`.

### 6. Run in Nethercore

Place the `.wasm` file in your Nethercore library or upload it to [nethercore.systems](https://nethercore.systems).

---

## Game Architecture

### The Three Functions

Every Nethercore game exports three functions:

| Function | When Called | Purpose |
|----------|-------------|---------|
| `init()` | Once at startup | Set configuration, load resources |
| `update()` | Every tick (60x/sec default) | Game logic, physics, input |
| `render()` | Every frame | Drawing only |

**Key insight:** `update()` runs at a fixed rate for determinism, while `render()` can run at the display's refresh rate.

### Init-Only Configuration

Some settings must be called in `init()` and cannot change at runtime:

```rust
fn init() {
    unsafe {
        set_resolution(2);        // 720p (0=360p, 1=540p, 2=720p, 3=1080p)
        set_tick_rate(60);        // 60 updates per second
        set_clear_color(0x000000FF);
        render_mode(0);           // Lambert rendering
    }
}
```

If you call these outside `init()`, they'll be ignored with a warning.

---

## Best Practices for Rollback-Safe Code

Nethercore uses GGRS rollback netcode. For multiplayer to work correctly, your `update()` must be **deterministic**—identical inputs produce identical results.

### DO: Use the Host RNG

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn random() -> u32;
}

fn spawn_enemy() {
    let x = (random() % 320) as f32;
    let y = (random() % 240) as f32;
    // ...
}
```

### DON'T: Use External Random Sources

```rust
// BAD - will cause desyncs!
use some_rng_crate::Rng;
let x = rng.gen::<f32>() * 320.0;
```

### DO: Keep All State in WASM Memory

All game state in WASM linear memory is automatically saved and restored during rollback. No manual serialization needed!

```rust
static mut GAME_STATE: GameState = GameState::new();

fn update() {
    unsafe {
        // Modify GAME_STATE
        // All changes are automatically captured for rollback
    }
}

// No save_state/load_state needed - the host snapshots entire WASM memory!
```

**How it works:**
- The host automatically snapshots your entire WASM linear memory during rollback
- All static variables, heap allocations (if used), and stack state are preserved
- Resources (textures, meshes, sounds) stay in GPU/host memory - only their handles (IDs) in WASM memory are snapshotted
- This works transparently - you never need to write serialization code

### DON'T: Use Floating Point Carefully

Some float operations can produce different results across platforms. For critical game logic:

```rust
// Prefer integer math for positions/collisions
static mut PLAYER_X_FIXED: i32 = 160 * 256; // 8.8 fixed point

fn update() {
    unsafe {
        PLAYER_X_FIXED += velocity_x_fixed;
    }
}

fn render() {
    unsafe {
        let x = (PLAYER_X_FIXED / 256) as f32;
        draw_rect(x, y, 20.0, 20.0, color);
    }
}
```

### DO: Keep State Small

Smaller state = faster save/load during rollback:

```rust
// Good: Minimal state
struct Player {
    x: i16,
    y: i16,
    health: u8,
    flags: u8,  // Pack booleans into flags
}

// Avoid: Bloated state
struct Player {
    x: f64,
    y: f64,
    history: Vec<Position>,  // No allocations!
}
```

### Rollback Checklist

- [ ] `update()` uses only `random()` for RNG
- [ ] No heap allocations (no `Vec`, `String`, `Box`) - recommended for simplicity
- [ ] No reading from external sources (time, files, network)
- [ ] All game state lives in WASM memory (static variables, globals)
- [ ] Fixed-point math for determinism (optional but safer)

**Note:** Heap allocations (Vec, Box, etc.) ARE automatically captured by memory snapshotting, but avoiding them keeps your code simpler and state size predictable.

---

## Asset Pipeline

### Embedding Assets

Assets are embedded at compile time—no file loading at runtime:

```rust
// Embed raw bytes
static SPRITE_DATA: &[u8] = include_bytes!("../assets/player.raw");

// For textures: RGBA8 format, width×height×4 bytes
fn init() {
    unsafe {
        let handle = load_texture(32, 32, SPRITE_DATA.as_ptr());
        texture_bind(handle);
    }
}
```

### Image Formats

Nethercore expects raw RGBA8 pixel data. Convert your images:

**Using ImageMagick:**
```bash
convert sprite.png -depth 8 rgba:sprite.raw
```

**Using a build script (build.rs):**
```rust
use std::fs;
use image::GenericImageView;

fn main() {
    let img = image::open("assets/sprite.png").unwrap();
    let rgba = img.to_rgba8();
    fs::write("assets/sprite.raw", rgba.as_raw()).unwrap();
    println!("cargo:rerun-if-changed=assets/sprite.png");
}
```

**Inline PNG decoding (no_std compatible):**
```rust
// Use a no_std PNG decoder like `minipng` or embed pre-decoded data
// Pre-decoded is simpler and faster at runtime
```

### Texture Guidelines

| Guideline | Reason |
|-----------|--------|
| Power-of-2 dimensions | Better GPU compatibility |
| 256×256 or smaller | Retro aesthetic, VRAM budget |
| Use texture atlases | Fewer bind calls = better perf |
| Nearest-neighbor filtering | Sharp pixels |

### Audio (Coming Soon)

Audio system is pending implementation. For now, games are silent.

---

## Debugging Tips

### Using log()

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn log(ptr: *const u8, len: u32);
}

fn debug_log(msg: &str) {
    unsafe { log(msg.as_ptr(), msg.len() as u32); }
}

fn update() {
    debug_log("update called");
}
```

### Runtime Stats Panel

Press **F3** during gameplay to toggle the Runtime Stats Panel showing:
- FPS and frame time
- VRAM usage
- Network stats (when in multiplayer)

### Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| Magenta checkerboard | Missing/invalid texture | Check `load_texture` returns non-zero |
| No lighting | Missing normals | Use vertex format with `FORMAT_NORMAL` |
| Config ignored | Called outside `init()` | Move to `init()` |
| Transform overflow | `push()` without `pop()` | Balance push/pop calls |
| Desync in multiplayer | Non-deterministic `update()` | Use `random()`, avoid floats |

### WASM Size Optimization

```toml
[profile.release]
opt-level = "z"    # Optimize for size (smaller than "s")
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Typical game size: 10-50KB (without assets).

---

## Example Projects

Study these examples in the `examples/` directory:

| Example | Demonstrates |
|---------|--------------|
| `hello-world` | Basic 2D drawing, input |
| `triangle` | Immediate mode 3D |
| `textured-quad` | Texture loading |
| `cube` | Retained meshes, lighting |
| `billboard` | 3D sprites |
| `lighting` | PBR rendering |
| `skinned-mesh` | GPU skeletal animation |
| `platformer` | Full mini-game |

Each example is a standalone project you can build and run.

---

## Quick Reference

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
const BUTTON_L3: u32 = 10;
const BUTTON_R3: u32 = 11;
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```

### Vertex Formats

```rust
const FORMAT_POS: u32 = 0;           // pos(3)
const FORMAT_UV: u32 = 1;            // pos(3) + uv(2)
const FORMAT_COLOR: u32 = 2;         // pos(3) + color(3)
const FORMAT_UV_COLOR: u32 = 3;      // pos(3) + uv(2) + color(3)
const FORMAT_NORMAL: u32 = 4;        // pos(3) + normal(3)
const FORMAT_UV_NORMAL: u32 = 5;     // pos(3) + uv(2) + normal(3)
const FORMAT_COLOR_NORMAL: u32 = 6;  // pos(3) + color(3) + normal(3)
const FORMAT_UV_COLOR_NORMAL: u32 = 7;
const FORMAT_SKINNED: u32 = 8;       // Add to any format for bone weights
```

### Color Format

Colors are 32-bit RGBA: `0xRRGGBBAA`

```rust
const RED: u32 = 0xFF0000FF;
const GREEN: u32 = 0x00FF00FF;
const BLUE: u32 = 0x0000FFFF;
const WHITE: u32 = 0xFFFFFFFF;
const TRANSPARENT: u32 = 0x00000000;
```

---

## Next Steps

1. **Read the API reference**: [FFI Reference](../architecture/ffi.md) and [Nethercore ZX](../architecture/nethercore-zx.md)
2. **Study the examples**: Each demonstrates specific features
3. **Build something small**: A simple game teaches more than reading docs
4. **Join the community**: Share your creations at [nethercore.systems](https://nethercore.systems)
