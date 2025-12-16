# ROM Data Pack Functions

Load assets from the ROM's bundled data pack.

## Overview

Assets loaded via `rom_*` functions go **directly to VRAM/audio memory**, bypassing WASM linear memory for efficient rollback. Only u32 handles are stored in your game's RAM.

**All `rom_*` functions are init-only** — call in `init()`, not `update()` or `render()`.

---

## Asset Loading

### rom_texture

Loads a texture from the data pack.

**Signature:**
```rust
fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID string |
| id_len | `u32` | Length of asset ID |

**Returns:** Texture handle (non-zero on success, 0 if not found)

**Example:**
```rust
fn init() {
    unsafe {
        PLAYER_TEX = rom_texture(b"player".as_ptr(), 6);
        ENEMY_TEX = rom_texture(b"enemy_sheet".as_ptr(), 11);
        TERRAIN_TEX = rom_texture(b"terrain".as_ptr(), 7);
    }
}
```

---

### rom_mesh

Loads a mesh from the data pack.

**Signature:**
```rust
fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32
```

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        LEVEL_MESH = rom_mesh(b"level1".as_ptr(), 6);
        PLAYER_MESH = rom_mesh(b"player_model".as_ptr(), 12);
        ENEMY_MESH = rom_mesh(b"enemy".as_ptr(), 5);
    }
}
```

---

### rom_skeleton

Loads a skeleton (inverse bind matrices) from the data pack.

**Signature:**
```rust
fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32
```

**Returns:** Skeleton handle

**Example:**
```rust
fn init() {
    unsafe {
        PLAYER_SKELETON = rom_skeleton(b"player_rig".as_ptr(), 10);
    }
}
```

---

### rom_font

Loads a bitmap font from the data pack.

**Signature:**
```rust
fn rom_font(id_ptr: *const u8, id_len: u32) -> u32
```

**Returns:** Font handle

**Example:**
```rust
fn init() {
    unsafe {
        UI_FONT = rom_font(b"ui_font".as_ptr(), 7);
        TITLE_FONT = rom_font(b"title_font".as_ptr(), 10);
    }
}
```

---

### rom_sound

Loads a sound from the data pack.

**Signature:**
```rust
fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32
```

**Returns:** Sound handle

**Example:**
```rust
fn init() {
    unsafe {
        JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
        COIN_SFX = rom_sound(b"coin".as_ptr(), 4);
        MUSIC = rom_sound(b"level1_bgm".as_ptr(), 10);
    }
}
```

---

## Raw Data Access

For custom data formats (level data, dialog scripts, etc.).

### rom_data_len

Gets the size of raw data in the pack.

**Signature:**
```rust
fn rom_data_len(id_ptr: *const u8, id_len: u32) -> u32
```

**Returns:** Size in bytes (0 if not found)

**Example:**
```rust
fn init() {
    unsafe {
        let len = rom_data_len(b"level1_map".as_ptr(), 10);
        if len > 0 {
            // Allocate buffer and load
        }
    }
}
```

---

### rom_data

Copies raw data from the pack into WASM memory.

**Signature:**
```rust
fn rom_data(id_ptr: *const u8, id_len: u32, out_ptr: *mut u8, max_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID |
| id_len | `u32` | Length of asset ID |
| out_ptr | `*mut u8` | Destination buffer in WASM memory |
| max_len | `u32` | Maximum bytes to copy |

**Returns:** Bytes copied (0 if not found or buffer too small)

**Example:**
```rust
static mut LEVEL_DATA: [u8; 4096] = [0; 4096];

fn init() {
    unsafe {
        let len = rom_data_len(b"level1".as_ptr(), 6);
        if len <= 4096 {
            rom_data(b"level1".as_ptr(), 6, LEVEL_DATA.as_mut_ptr(), 4096);
            parse_level(&LEVEL_DATA[..len as usize]);
        }
    }
}
```

---

## Game Manifest (ember.toml)

Assets are bundled using the `ember.toml` manifest:

```toml
[game]
id = "my-game"
title = "My Awesome Game"
author = "Developer Name"
version = "1.0.0"
render_mode = 2

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.textures]]
id = "enemy_sheet"
path = "assets/enemies.png"

[[assets.meshes]]
id = "level1"
path = "assets/level1.ewzmesh"

[[assets.meshes]]
id = "player_model"
path = "assets/player.ewzmesh"

[[assets.skeletons]]
id = "player_rig"
path = "assets/player.ewzskel"

[[assets.animations]]
id = "walk"
path = "assets/walk.ewzanim"

[[assets.sounds]]
id = "jump"
path = "assets/sfx/jump.raw"

[[assets.sounds]]
id = "level1_bgm"
path = "assets/music/level1.raw"

[[assets.fonts]]
id = "ui_font"
path = "assets/fonts/ui.ewzfont"

[[assets.data]]
id = "level1"
path = "assets/levels/level1.bin"
```

Build with:
```bash
ember build
ember pack  # Creates .ewz ROM file
```

---

## Memory Model

```
ROM (12MB)          RAM (4MB)           VRAM (4MB)
┌────────────┐      ┌────────────┐      ┌────────────┐
│ WASM code  │      │ Game state │      │ Textures   │
│ (50-200KB) │      │ (handles)  │      │ (from ROM) │
├────────────┤      │            │      ├────────────┤
│ Data Pack: │      │ u32 tex_id │─────▶│ Uploaded   │
│ - textures │      │ u32 mesh_id│─────▶│ GPU data   │
│ - meshes   │      │ u32 snd_id │      └────────────┘
│ - sounds   │      │            │
│ - fonts    │      │ Level data │◀──── rom_data()
│ - data     │      │ (copied)   │      copies here
└────────────┘      └────────────┘
```

**Key points:**
- `rom_texture/mesh/sound/font` → Data stays in host memory, only handle in WASM RAM
- `rom_data` → Data copied to WASM RAM (use sparingly for level data, etc.)
- Only WASM RAM (4MB) is snapshotted for rollback

---

## Complete Example

```rust
// Asset handles
static mut PLAYER_TEX: u32 = 0;
static mut PLAYER_MESH: u32 = 0;
static mut PLAYER_SKEL: u32 = 0;
static mut WALK_ANIM: u32 = 0;
static mut IDLE_ANIM: u32 = 0;
static mut JUMP_SFX: u32 = 0;
static mut MUSIC: u32 = 0;
static mut UI_FONT: u32 = 0;

// Level data (copied to WASM memory)
static mut LEVEL_DATA: [u8; 8192] = [0; 8192];
static mut LEVEL_SIZE: u32 = 0;

fn init() {
    unsafe {
        // Graphics assets → VRAM
        PLAYER_TEX = rom_texture(b"player".as_ptr(), 6);
        PLAYER_MESH = rom_mesh(b"player".as_ptr(), 6);
        PLAYER_SKEL = rom_skeleton(b"player_rig".as_ptr(), 10);

        // Animations → GPU storage
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
        IDLE_ANIM = rom_keyframes(b"idle".as_ptr(), 4);

        // Audio → Audio memory
        JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
        MUSIC = rom_sound(b"music".as_ptr(), 5);

        // Font → VRAM
        UI_FONT = rom_font(b"ui".as_ptr(), 2);

        // Level data → WASM RAM (copied)
        LEVEL_SIZE = rom_data_len(b"level1".as_ptr(), 6);
        if LEVEL_SIZE <= 8192 {
            rom_data(b"level1".as_ptr(), 6, LEVEL_DATA.as_mut_ptr(), 8192);
        }

        // Start music
        music_play(MUSIC, 0.7);
    }
}

fn render() {
    unsafe {
        // Use loaded assets
        texture_bind(PLAYER_TEX);
        skeleton_bind(PLAYER_SKEL);
        keyframe_bind(WALK_ANIM, frame);
        draw_mesh(PLAYER_MESH);

        font_bind(UI_FONT);
        draw_text(b"SCORE: 0".as_ptr(), 8, 10.0, 10.0, 16.0, 0xFFFFFFFF);
    }
}
```

**See Also:** [Textures](./textures.md), [Meshes](./meshes.md), [Audio](./audio.md), [Animation](./animation.md)
