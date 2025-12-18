# Emberware FFI Reference

This document covers the **shared FFI** common to all Emberware consoles. For console-specific APIs, see:

- [Emberware ZX](./emberware-zx.md) — 5th gen (PS1/N64/Saturn)
- [Emberware Chroma](./emberware-chroma.md) — 4th gen (Genesis/SNES/Neo Geo) *(Coming Soon)*

---

## Game Lifecycle

All Emberware games must export these three functions:

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Called once at startup
    // Set console configuration (resolution, tick rate, render mode)
    // Initialize game state, create textures from embedded assets
}

#[no_mangle]
pub extern "C" fn update() {
    // Called every tick (deterministic!)
    // Game logic, physics, input handling
    // MUST produce identical results given identical inputs
}

#[no_mangle]
pub extern "C" fn render() {
    // Called every frame
    // Draw calls only — skipped during rollback replay
}
```

### Console Configuration (init-only)

Certain settings define the "mood" of your game and **must be set in `init()`**. These cannot be changed at runtime — calling them in `update()` or `render()` will fail/error.

```rust
fn set_resolution(res: u32)                 // Console-specific resolution enum (see console docs)
fn set_tick_rate(fps: u32)                  // 24, 30, 60, or 120
fn set_clear_color(color: u32)              // Auto-clear color (0xRRGGBBAA), default: black
fn render_mode(mode: u32)                   // Emberware ZX only: 0-3
```

Resolution values are console-specific:
- **Emberware ZX**: 0=360p, 1=540p (default), 2=720p, 3=1080p
- **Emberware Chroma**: 0-7 (see console docs for specific resolutions)

If not set, the console uses its defaults (540p @ 60fps for ZX, 384×216 @ 60fps for Chroma).

**Why init-only?**
- Resolution affects framebuffer allocation
- Tick rate affects GGRS synchronization
- Render mode determines shader pipelines
- Changing these mid-game would break determinism and netplay

### Mode 2 Migration (2025)

**Emberware ZX Mode 2 was migrated from PBR-lite to Metallic-Roughness Blinn-Phong:**

**What changed in the rendering:**
- Specular model: GGX → Normalized Blinn-Phong (Gotanda 2010)
- Environment reflections: Removed (slot 2 freed)
- Specular color: Derived from metallic (F0=0.04 for dielectrics, albedo for metals)
- Roughness mapping: Power curve `pow(256.0, 1.0 - roughness)` (0→256, 1→1 shininess range)
- Rim lighting: Added as uniform-only feature (same code as Mode 3)
- Ambient lighting: Now uses Gotanda-based energy conservation (like Mode 3)

**What stayed the same (no API changes):**
- FFI functions: `material_metallic()`, `material_roughness()`, `material_emissive()` work identically
- Texture slot 1: MRE (R=Metallic, G=Roughness, B=Emissive) layout unchanged
- Light functions: `light_set()`, `light_color()`, `light_intensity()` all work the same
- Material workflow: Physics-based metallic-roughness still applies

**Mode 3 changes (related):**
- Texture slot 1, channel R: Changed from "Rim intensity" to "Specular intensity"
- Rim lighting now modulated by specular intensity (both specular highlights and rim affect each other)

**Migration guide for existing content:**
- **Roughness adjustment:** If specular highlights look different, try adjusting roughness ±0.1-0.2 for similar sharpness
- **Slot 2 matcap:** Previously optional for environment reflections — no longer sampled. Remove `texture_bind_slot(2, ...)` calls (safe no-op)
- **Rim lighting:** Mode 2 now supports rim lighting via `material_rim(intensity, power)` FFI functions (uniform-only, no texture)
- **Mode 3 assets:** If you have Mode 3 textures, slot 1.R now controls specular intensity instead of rim intensity
- **Fresnel effects:** View-dependent grazing angle brightening is gone. Accept as design change or adjust roughness values

### Rollback Netcode

Emberware uses GGRS for deterministic rollback netcode. Key rules:

- `update()` **MUST** be deterministic (same inputs → same state)
- Use `random()` for RNG — never external random sources
- Game state is **automatically snapshotted** by the host during rollback (entire WASM linear memory)
- `render()` is skipped during rollback replay
- Tick rate is separate from frame rate

**No manual serialization needed!** All game state in WASM linear memory is automatically saved and restored by the host. Your `update()` function just needs to be deterministic — resources (textures, meshes, sounds) stay in GPU/host memory and are never rolled back, only the game state handles in WASM memory.

### Memory Limits

Emberware uses a **split ROM + RAM memory model** for efficient rollback:

| Console | ROM (Cartridge) | RAM (Linear Memory) | VRAM |
|---------|-----------------|---------------------|------|
| **Emberware ZX** | 12 MB | 4 MB | 4 MB |
| **Emberware Chroma** | 4 MB | 2 MB | 1 MB |

**ROM (Cartridge):** Contains WASM code + bundled assets (via data pack). Not snapshotted.
- WASM bytecode (typically 50-200 KB)
- Data pack assets: textures, meshes, sounds, fonts, raw data
- Assets loaded via `rom_*` FFI go directly to VRAM/audio memory

**RAM (Linear Memory):** Your game's working memory. Fully snapshotted for rollback.
- Stack space (function calls, local variables)
- Heap allocations (game state, dynamic data)
- Only resource handles (u32 IDs) stored here — actual data in VRAM

**Enforcement:**
- Games that declare more memory than allowed will **fail to load**
- Games that try to grow memory past the limit will **fail at runtime**
- The host uses wasmtime's `ResourceLimiter` — this cannot be bypassed

**Rollback Performance:**
Only RAM is snapshotted for rollback netcode. With xxHash3 checksums:
- 4MB: ~0.25ms per save (Emberware ZX)
- 2MB: ~0.10ms per save (Emberware Chroma)

During an 8-frame rollback at 60fps, the total overhead is ~2ms — well within the 16.67ms frame budget.

**Tips:**
- Use `rom_*` functions to load assets from the data pack (doesn't use RAM)
- Legacy `include_bytes!()` still works for small assets
- Keep game state small for faster rollback
- Only handles live in WASM memory — textures, meshes, sounds stay in host memory

---

## System Functions

### delta_time

```rust
fn delta_time() -> f32
```

Returns time elapsed since the last tick in seconds.

```rust
position.x += velocity.x * delta_time();
```

---

### elapsed_time

```rust
fn elapsed_time() -> f32
```

Returns total elapsed time since game start in seconds.

```rust
let pulse = (elapsed_time() * 2.0).sin() * 0.5 + 0.5;
```

---

### tick_count

```rust
fn tick_count() -> u64
```

Returns the current tick number.

```rust
if tick_count() % 60 == 0 {
    // Every second at 60fps
}
```

---

### log

```rust
fn log(ptr: *const u8, len: u32)
```

Logs a message to the console output.

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

---

## Rollback Functions

### random

```rust
fn random() -> u32
```

Returns a deterministic random number from the host's seeded RNG. **Always use this** instead of external random sources.

```rust
let r = random();
let spawn_x = (r % 320) as f32;

// Random float 0.0-1.0
let rf = (random() as f32) / (u32::MAX as f32);
```

---

## Session Functions

### player_count

```rust
fn player_count() -> u32
```

Returns the number of players in the session (1-4).

---

### local_player_mask

```rust
fn local_player_mask() -> u32
```

Returns a bitmask of which players are local to this client.

```rust
let mask = local_player_mask();
let p0_local = (mask & 1) != 0;  // Is player 0 local?
let p1_local = (mask & 2) != 0;  // Is player 1 local?
```

### Multiplayer Model

Emberware supports up to 4 players in any mix of local and remote:
- 4 local players (couch co-op)
- 1 local + 3 remote (online)
- 2 local + 2 remote (mixed)

All player inputs are synchronized via GGRS, so games process all players uniformly:

```rust
fn update() {
    for p in 0..player_count() {
        // Process player p — GGRS handles input sync
    }
}
```

---

## Save Data

Save data is stored locally per-game. Maximum 64KB per save slot, 8 slots (0-7).

### save

```rust
fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32
```

Saves data to a slot. Returns 0 on success, 1 if invalid slot, 2 if data too large.

```rust
let save_data = serialize_save();
save(0, save_data.as_ptr(), save_data.len() as u32);
```

---

### load

```rust
fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32
```

Loads data from a slot. Returns bytes read (0 if empty or error).

```rust
let mut buffer = [0u8; 1024];
let len = load(0, buffer.as_mut_ptr(), buffer.len() as u32);
if len > 0 {
    deserialize_save(&buffer[..len as usize]);
}
```

---

### delete

```rust
fn delete(slot: u32) -> u32
```

Deletes a save slot. Returns 0 on success, 1 if invalid slot.

---

## ROM Data Pack Functions

These functions load assets from the ROM's data pack. Assets go directly to VRAM/audio memory, bypassing WASM linear memory for efficient rollback.

**All `rom_*` functions are init-only** — they must be called in `init()`, not `update()` or `render()`.

### rom_texture

```rust
fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32
```

Loads a texture from the data pack by string ID. Returns a texture handle (0 if not found).

```rust
let tex = rom_texture(b"player".as_ptr(), 6);
```

---

### rom_mesh

```rust
fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32
```

Loads a mesh from the data pack by string ID. Returns a mesh handle (0 if not found).

```rust
let mesh = rom_mesh(b"enemy".as_ptr(), 5);
```

---

### rom_sound

```rust
fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32
```

Loads a sound from the data pack by string ID. Returns a sound handle (0 if not found).

```rust
let sfx = rom_sound(b"jump".as_ptr(), 4);
```

---

### rom_skeleton

```rust
fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32
```

Loads a skeleton from the data pack by string ID. Returns a skeleton handle (0 if not found).

```rust
let skel = rom_skeleton(b"player_rig".as_ptr(), 10);
```

---

### rom_animation

```rust
fn rom_animation(id_ptr: *const u8, id_len: u32) -> u32
```

Loads an animation from the data pack by string ID. Returns an animation handle (0 if not found).

```rust
let anim = rom_animation(b"walk".as_ptr(), 4);
```

---

### rom_font

```rust
fn rom_font(id_ptr: *const u8, id_len: u32) -> u32
```

Loads a bitmap font from the data pack by string ID. Returns a font handle (0 if not found).

```rust
let font = rom_font(b"ui_font".as_ptr(), 7);
```

---

### rom_data_len

```rust
fn rom_data_len(id_ptr: *const u8, id_len: u32) -> u32
```

Returns the size in bytes of raw data in the data pack. Returns 0 if not found.

```rust
let len = rom_data_len(b"level1".as_ptr(), 6);
```

---

### rom_data

```rust
fn rom_data(id_ptr: *const u8, id_len: u32, out_ptr: *mut u8, max_len: u32) -> u32
```

Copies raw data from the data pack into WASM memory. Returns bytes copied (0 if not found or buffer too small).

```rust
let len = rom_data_len(b"level1".as_ptr(), 6);
let mut buffer = vec![0u8; len as usize];
rom_data(b"level1".as_ptr(), 6, buffer.as_mut_ptr(), len);
```

---

## Building Your Game

```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/your_game.wasm
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

### Loading Assets

**Recommended: Data Pack Loading (rom_* functions)**

Assets bundled in the ROM's data pack bypass WASM memory entirely:

```rust
fn init() {
    // Load from data pack — goes directly to VRAM
    let tex = rom_texture(b"player_sprite".as_ptr(), 13);
    let mesh = rom_mesh(b"enemy_model".as_ptr(), 11);
    let sfx = rom_sound(b"jump".as_ptr(), 4);

    // For raw level data, copies into WASM memory
    let len = rom_data_len(b"level1".as_ptr(), 6);
    let mut buffer = vec![0u8; len as usize];
    rom_data(b"level1".as_ptr(), 6, buffer.as_mut_ptr(), len);
}
```

**Legacy: Embedded Assets**

You can still embed small assets directly in the WASM binary:

```rust
// Embed at compile time (uses RAM!)
static SPRITE_PNG: &[u8] = include_bytes!("assets/sprite.png");

fn init() {
    // Decode and upload to GPU at runtime
    let (w, h, pixels) = decode_png(SPRITE_PNG);
    let tex = load_texture(w, h, pixels.as_ptr());
}
```

**Which to use?**
- **Data pack** for large assets (textures, meshes, sounds) — doesn't use RAM
- **include_bytes!** for tiny files or generated content (<10KB)

---

## Console-Specific APIs

Each console has its own graphics, input, and audio APIs:

| Console | Input | Graphics | Status | Doc |
|---------|-------|----------|--------|-----|
| **Emberware ZX** | Dual analog sticks, analog triggers, 4 face buttons | 2D + 3D, transforms | Available | [emberware-zx.md](./emberware-zx.md) |
| **Emberware Chroma** | D-pad only, 6 face buttons, no analog | 2D sprites, tilemaps | Coming Soon | [emberware-chroma.md](./emberware-chroma.md) |

---

Upload your `.wasm` file at [emberware.io](https://emberware.io).
