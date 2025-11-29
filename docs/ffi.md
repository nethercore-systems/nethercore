# Emberware FFI Reference

Complete API reference for Emberware game developers.

## Console Specs

| Spec | Value |
|------|-------|
| Resolution | 360p, 540p (default), 720p, 1080p |
| Tick rate | 24, 30, 60 (default), 120 fps |
| RAM | 16MB |
| VRAM | 8MB |
| CPU budget | 4ms per tick (at 60fps) |
| ROM size | 32MB max |
| Netcode | Deterministic rollback via GGRS |

## Game Lifecycle

Your game must export these three functions:

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Called once at startup
    // Initialize game state, load assets
}

#[no_mangle]
pub extern "C" fn update() {
    // Called every tick (deterministic!)
    // Game logic, physics, input handling
    // MUST be deterministic for rollback netcode
}

#[no_mangle]
pub extern "C" fn render() {
    // Called every frame
    // Draw calls only — skipped during rollback replay
}
```

### Rollback Netcode

The console uses GGRS for deterministic rollback netcode. Key points:

- `update()` **MUST** be deterministic (same inputs → same state)
- Use `random()` for seeded RNG — never external random sources
- `save_state`/`load_state` are called by the host during rollback
- Audio and rendering are skipped during rollback replay
- Tick rate is separate from frame rate (update can run multiple times per frame during catchup)

---

## System Functions

### delta_time

```rust
fn delta_time() -> f32
```

Returns the time elapsed since the last tick in seconds.

**Example:**
```rust
let dt = delta_time();
position.x += velocity.x * dt;
```

---

### elapsed_time

```rust
fn elapsed_time() -> f32
```

Returns total elapsed time since game start in seconds.

**Example:**
```rust
let t = elapsed_time();
let pulse = (t * 2.0).sin() * 0.5 + 0.5;
```

---

### tick_count

```rust
fn tick_count() -> u64
```

Returns the current tick number (frame count for game logic).

**Example:**
```rust
let tick = tick_count();
if tick % 60 == 0 {
    // Every second at 60fps
}
```

---

### log

```rust
fn log(ptr: *const u8, len: u32)
```

Logs a message to the console output.

**Parameters:**
- `ptr` — Pointer to UTF-8 string data
- `len` — Length of string in bytes

**Example:**
```rust
let msg = b"Player spawned";
log(msg.as_ptr(), msg.len() as u32);
```

---

### quit

```rust
fn quit()
```

Exits the game and returns to the library.

**Example:**
```rust
if button_pressed(0, BUTTON_START) != 0 && in_quit_menu {
    quit();
}
```

---

## Rollback Functions

### save_state

```rust
fn save_state(ptr: *mut u8, max_len: u32) -> u32
```

Called by the host to serialize game state for rollback. Write your game state to the provided buffer.

**Parameters:**
- `ptr` — Pointer to buffer to write state into
- `max_len` — Maximum bytes available

**Returns:** Number of bytes written

**Example:**
```rust
#[no_mangle]
pub extern "C" fn save_state(ptr: *mut u8, max_len: u32) -> u32 {
    let state_bytes = serialize_game_state();
    let len = state_bytes.len().min(max_len as usize);
    unsafe {
        core::ptr::copy_nonoverlapping(state_bytes.as_ptr(), ptr, len);
    }
    len as u32
}
```

---

### load_state

```rust
fn load_state(ptr: *const u8, len: u32)
```

Called by the host to restore game state during rollback.

**Parameters:**
- `ptr` — Pointer to serialized state data
- `len` — Length of state data in bytes

**Example:**
```rust
#[no_mangle]
pub extern "C" fn load_state(ptr: *const u8, len: u32) {
    let state_bytes = unsafe {
        core::slice::from_raw_parts(ptr, len as usize)
    };
    deserialize_game_state(state_bytes);
}
```

---

### random

```rust
fn random() -> u32
```

Returns a deterministic random number from the host's seeded RNG. **Always use this instead of external random sources** to maintain determinism for rollback.

**Example:**
```rust
let r = random();
let spawn_x = (r % 320) as f32;

// Random float 0.0-1.0
let rf = (random() as f32) / (u32::MAX as f32);
```

---

## Graphics Core

### clear

```rust
fn clear(color: u32)
```

Clears the screen to the specified color.

**Parameters:**
- `color` — RGBA color as 0xRRGGBBAA

**Example:**
```rust
clear(0x1a1a2eFF); // Dark blue background
clear(0x000000FF); // Black
clear(0xFF0000FF); // Red
```

---

### frame_begin

```rust
fn frame_begin()
```

Begins a new render frame. Call at the start of `render()`.

**Example:**
```rust
fn render() {
    frame_begin();
    // ... draw calls ...
    frame_end();
}
```

---

### frame_end

```rust
fn frame_end()
```

Ends the render frame and presents to screen. Call at the end of `render()`.

---

### camera_set

```rust
fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32)
```

Sets the 3D camera position and look-at target.

**Parameters:**
- `x, y, z` — Camera position
- `target_x, target_y, target_z` — Look-at target

**Example:**
```rust
camera_set(0.0, 5.0, -10.0, 0.0, 0.0, 0.0); // Look at origin from behind/above
```

---

### camera_fov

```rust
fn camera_fov(fov_degrees: f32)
```

Sets the camera field of view.

**Parameters:**
- `fov_degrees` — Field of view in degrees (default: 60)

**Example:**
```rust
camera_fov(90.0); // Wide angle
camera_fov(45.0); // Narrow/zoomed
```

---

## Textures

### texture_load

```rust
fn texture_load(path_ptr: *const u8, path_len: u32) -> u32
```

Loads a texture from the ROM's embedded assets.

**Parameters:**
- `path_ptr` — Pointer to asset path string
- `path_len` — Length of path string

**Returns:** Texture handle (0 on failure)

**Example:**
```rust
let path = b"sprites/player.png";
let tex = texture_load(path.as_ptr(), path.len() as u32);
```

---

### texture_create

```rust
fn texture_create(width: u32, height: u32, pixels: *const u8) -> u32
```

Creates a texture from raw RGBA pixel data.

**Parameters:**
- `width` — Texture width in pixels
- `height` — Texture height in pixels
- `pixels` — Pointer to RGBA pixel data (width * height * 4 bytes)

**Returns:** Texture handle (0 on failure)

**Example:**
```rust
let pixels: [u8; 16] = [
    255, 0, 0, 255,   255, 255, 255, 255,
    255, 255, 255, 255, 255, 0, 0, 255,
];
let tex = texture_create(2, 2, pixels.as_ptr());
```

---

### texture_bind

```rust
fn texture_bind(handle: u32)
```

Binds a texture for subsequent draw calls.

**Parameters:**
- `handle` — Texture handle from `texture_load` or `texture_create`

**Example:**
```rust
texture_bind(player_texture);
draw_sprite(x, y, 32.0, 32.0, 0xFFFFFFFF);
```

---

### texture_free

```rust
fn texture_free(handle: u32)
```

Frees a texture and releases VRAM.

**Parameters:**
- `handle` — Texture handle to free

---

## 3D Drawing

### draw_triangle

```rust
fn draw_triangle(
    x0: f32, y0: f32, z0: f32, u0: f32, v0: f32,
    x1: f32, y1: f32, z1: f32, u1: f32, v1: f32,
    x2: f32, y2: f32, z2: f32, u2: f32, v2: f32,
    color: u32
)
```

Draws a single textured triangle in 3D space.

**Parameters:**
- `x0, y0, z0` — First vertex position
- `u0, v0` — First vertex UV coordinates
- `x1, y1, z1, u1, v1` — Second vertex
- `x2, y2, z2, u2, v2` — Third vertex
- `color` — Vertex color tint (RGBA)

**Example:**
```rust
draw_triangle(
    0.0, 1.0, 0.0, 0.5, 0.0,  // Top
    -1.0, 0.0, 0.0, 0.0, 1.0, // Bottom left
    1.0, 0.0, 0.0, 1.0, 1.0,  // Bottom right
    0xFFFFFFFF
);
```

---

### draw_mesh

```rust
fn draw_mesh(
    vertices: *const f32,
    vertex_count: u32,
    indices: *const u16,
    index_count: u32,
    color: u32
)
```

Draws an indexed mesh. Vertex format: x, y, z, u, v (5 floats per vertex).

**Parameters:**
- `vertices` — Pointer to vertex data
- `vertex_count` — Number of vertices
- `indices` — Pointer to index data
- `index_count` — Number of indices
- `color` — Vertex color tint

**Example:**
```rust
let verts: [f32; 20] = [
    -1.0, 0.0, -1.0, 0.0, 0.0,
     1.0, 0.0, -1.0, 1.0, 0.0,
     1.0, 0.0,  1.0, 1.0, 1.0,
    -1.0, 0.0,  1.0, 0.0, 1.0,
];
let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
draw_mesh(verts.as_ptr(), 4, indices.as_ptr(), 6, 0xFFFFFFFF);
```

---

## Transform Stack

### transform_identity

```rust
fn transform_identity()
```

Resets the current transform to identity matrix.

---

### transform_translate

```rust
fn transform_translate(x: f32, y: f32, z: f32)
```

Translates the current transform.

---

### transform_rotate

```rust
fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32)
```

Rotates the current transform around an axis.

**Parameters:**
- `angle_deg` — Rotation angle in degrees
- `x, y, z` — Rotation axis (should be normalized)

---

### transform_scale

```rust
fn transform_scale(x: f32, y: f32, z: f32)
```

Scales the current transform.

---

### transform_push

```rust
fn transform_push()
```

Pushes the current transform onto the stack.

---

### transform_pop

```rust
fn transform_pop()
```

Pops the transform stack, restoring the previous transform.

---

### transform_set

```rust
fn transform_set(matrix: *const f32)
```

Sets the transform to a 4x4 matrix directly.

**Parameters:**
- `matrix` — Pointer to 16 floats (column-major order)

---

**Example (transform stack):**
```rust
transform_identity();
transform_translate(player_x, player_y, player_z);
transform_push();
    transform_rotate(arm_angle, 0.0, 0.0, 1.0);
    draw_mesh(arm_mesh, ...);
transform_pop();
draw_mesh(body_mesh, ...);
```

---

## 2D Drawing

### draw_sprite

```rust
fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32)
```

Draws a textured sprite (requires texture_bind first).

**Parameters:**
- `x, y` — Position (top-left corner)
- `w, h` — Size in pixels
- `color` — Color tint (RGBA)

**Example:**
```rust
texture_bind(player_tex);
draw_sprite(100.0, 50.0, 32.0, 32.0, 0xFFFFFFFF);
```

---

### draw_rect

```rust
fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32)
```

Draws a filled rectangle (no texture).

**Parameters:**
- `x, y` — Position (top-left corner)
- `w, h` — Size in pixels
- `color` — Fill color (RGBA)

**Example:**
```rust
draw_rect(0.0, 0.0, 320.0, 20.0, 0x333333FF); // UI bar
draw_rect(10.0, 5.0, health * 100.0, 10.0, 0xFF0000FF); // Health bar
```

---

### draw_text

```rust
fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32)
```

Draws text using the built-in font.

**Parameters:**
- `ptr` — Pointer to UTF-8 string
- `len` — String length in bytes
- `x, y` — Position
- `size` — Font size in pixels
- `color` — Text color (RGBA)

**Example:**
```rust
let score_text = b"SCORE: 12345";
draw_text(score_text.as_ptr(), score_text.len() as u32, 10.0, 10.0, 16.0, 0xFFFFFFFF);
```

---

## Render State

### depth_test

```rust
fn depth_test(enabled: u32)
```

Enables/disables depth testing.

**Parameters:**
- `enabled` — 1 to enable, 0 to disable

---

### cull_mode

```rust
fn cull_mode(mode: u32)
```

Sets face culling mode.

**Parameters:**
- `mode` — 0 = none, 1 = back, 2 = front

---

### blend_mode

```rust
fn blend_mode(mode: u32)
```

Sets alpha blending mode.

**Parameters:**
- `mode` — 0 = none, 1 = alpha, 2 = additive, 3 = multiply

---

### vertex_wobble

```rust
fn vertex_wobble(amount: f32)
```

Enables PS1-style vertex wobble/jitter for that retro look.

**Parameters:**
- `amount` — Wobble intensity (0.0 = off, 1.0 = full PS1 style)

**Example:**
```rust
vertex_wobble(0.5); // Subtle wobble
```

---

### texture_filter

```rust
fn texture_filter(filter: u32)
```

Sets texture filtering mode.

**Parameters:**
- `filter` — 0 = nearest (pixelated), 1 = linear (smooth)

---

## Audio

### sound_load

```rust
fn sound_load(path_ptr: *const u8, path_len: u32) -> u32
```

Loads a sound from ROM assets.

**Parameters:**
- `path_ptr` — Pointer to asset path
- `path_len` — Path length

**Returns:** Sound handle (0 on failure)

**Example:**
```rust
let path = b"sfx/jump.wav";
let jump_sound = sound_load(path.as_ptr(), path.len() as u32);
```

---

### sound_play

```rust
fn sound_play(handle: u32, volume: f32, looping: u32) -> u32
```

Plays a sound.

**Parameters:**
- `handle` — Sound handle from `sound_load`
- `volume` — Volume (0.0 to 1.0)
- `looping` — 1 to loop, 0 for one-shot

**Returns:** Playing instance ID

**Example:**
```rust
sound_play(jump_sound, 1.0, 0); // Play once
let music_id = sound_play(bgm, 0.7, 1); // Loop music
```

---

### sound_stop

```rust
fn sound_stop(instance_id: u32)
```

Stops a playing sound instance.

---

### sound_stop_all

```rust
fn sound_stop_all()
```

Stops all currently playing sounds.

---

### volume_set

```rust
fn volume_set(instance_id: u32, volume: f32)
```

Sets the volume of a playing sound instance.

---

### sound_free

```rust
fn sound_free(handle: u32)
```

Frees a loaded sound and releases memory.

---

## Input

Button constants:
```rust
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;
const BUTTON_L: u32 = 8;
const BUTTON_R: u32 = 9;
const BUTTON_START: u32 = 10;
const BUTTON_SELECT: u32 = 11;
```

### button_held

```rust
fn button_held(player: u32, button: u32) -> u32
```

Returns 1 if the button is currently held, 0 otherwise.

**Parameters:**
- `player` — Player index (0-3)
- `button` — Button constant

---

### button_pressed

```rust
fn button_pressed(player: u32, button: u32) -> u32
```

Returns 1 if the button was just pressed this tick, 0 otherwise.

---

### button_released

```rust
fn button_released(player: u32, button: u32) -> u32
```

Returns 1 if the button was just released this tick, 0 otherwise.

---

### stick_x

```rust
fn stick_x(player: u32) -> f32
```

Returns the analog stick X axis (-1.0 to 1.0).

---

### stick_y

```rust
fn stick_y(player: u32) -> f32
```

Returns the analog stick Y axis (-1.0 to 1.0).

---

### player_count

```rust
fn player_count() -> u32
```

Returns the number of connected players (1-4).

---

### local_player

```rust
fn local_player() -> u32
```

Returns the local player index (for netplay, identifies which player this client controls).

---

**Example:**
```rust
fn update() {
    let p = local_player();

    if button_pressed(p, BUTTON_A) != 0 {
        player_jump();
    }

    let move_x = stick_x(p);
    let move_y = stick_y(p);
    player_move(move_x, move_y);
}
```

---

## Save Data

Save data is stored locally per-game. Maximum 64KB per save slot.

### save

```rust
fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32
```

Saves data to a slot.

**Parameters:**
- `slot` — Save slot (0-7)
- `data_ptr` — Pointer to data
- `data_len` — Data length (max 64KB)

**Returns:** 1 on success, 0 on failure

**Example:**
```rust
let save_data = serialize_save_game();
save(0, save_data.as_ptr(), save_data.len() as u32);
```

---

### load

```rust
fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32
```

Loads data from a slot.

**Parameters:**
- `slot` — Save slot (0-7)
- `data_ptr` — Pointer to buffer
- `max_len` — Buffer size

**Returns:** Bytes read (0 if slot empty or error)

**Example:**
```rust
let mut buffer = [0u8; 1024];
let len = load(0, buffer.as_mut_ptr(), buffer.len() as u32);
if len > 0 {
    deserialize_save_game(&buffer[..len as usize]);
}
```

---

### delete

```rust
fn delete(slot: u32) -> u32
```

Deletes a save slot.

**Parameters:**
- `slot` — Save slot (0-7)

**Returns:** 1 on success, 0 on failure

---

## Complete Example

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

#[link(wasm_import_module = "emberware")]
extern "C" {
    fn clear(color: u32);
    fn frame_begin();
    fn frame_end();
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn random() -> u32;
}

static mut PLAYER_Y: f32 = 120.0;
static mut SCORE: u32 = 0;

#[no_mangle]
pub extern "C" fn init() {
    // Game initialized
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Input (deterministic - reads from GGRS synced inputs)
        if button_held(0, 0) != 0 { PLAYER_Y -= 3.0; }
        if button_held(0, 1) != 0 { PLAYER_Y += 3.0; }
        PLAYER_Y = PLAYER_Y.clamp(10.0, 230.0);

        // Random spawn using deterministic RNG
        if random() % 120 == 0 {
            SCORE += 10;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        frame_begin();
        clear(0x1a1a2eFF);

        // Draw player
        draw_rect(150.0, PLAYER_Y, 20.0, 20.0, 0x4a9fffFF);

        // Draw score
        let score_str = b"SCORE";
        draw_text(score_str.as_ptr(), score_str.len() as u32, 10.0, 10.0, 12.0, 0xFFFFFFFF);

        frame_end();
    }
}
```

---

## Building Your Game

```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Output will be at target/wasm32-unknown-unknown/release/your_game.wasm
```

**Cargo.toml:**
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

Upload your `.wasm` file at [emberware.io](https://emberware.io).
