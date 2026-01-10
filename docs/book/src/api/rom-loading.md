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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_texture(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_texture(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID string |
| id_len | `u32` | Length of asset ID |

**Returns:** Texture handle (non-zero on success, 0 if not found)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        PLAYER_TEX = rom_texture(b"player".as_ptr(), 6);
        ENEMY_TEX = rom_texture(b"enemy_sheet".as_ptr(), 11);
        TERRAIN_TEX = rom_texture(b"terrain".as_ptr(), 7);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    player_tex = rom_texture("player", 6);
    enemy_tex = rom_texture("enemy_sheet", 11);
    terrain_tex = rom_texture("terrain", 7);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    player_tex = rom_texture("player", 6);
    enemy_tex = rom_texture("enemy_sheet", 11);
    terrain_tex = rom_texture("terrain", 7);
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_mesh

Loads a mesh from the data pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_mesh(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_mesh(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        LEVEL_MESH = rom_mesh(b"level1".as_ptr(), 6);
        PLAYER_MESH = rom_mesh(b"player_model".as_ptr(), 12);
        ENEMY_MESH = rom_mesh(b"enemy".as_ptr(), 5);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    level_mesh = rom_mesh("level1", 6);
    player_mesh = rom_mesh("player_model", 12);
    enemy_mesh = rom_mesh("enemy", 5);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    level_mesh = rom_mesh("level1", 6);
    player_mesh = rom_mesh("player_model", 12);
    enemy_mesh = rom_mesh("enemy", 5);
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_skeleton

Loads a skeleton (inverse bind matrices) from the data pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_skeleton(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_skeleton(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Skeleton handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        PLAYER_SKELETON = rom_skeleton(b"player_rig".as_ptr(), 10);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    player_skeleton = rom_skeleton("player_rig", 10);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    player_skeleton = rom_skeleton("player_rig", 10);
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_font

Loads a bitmap font from the data pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_font(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_font(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_font(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Font handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        UI_FONT = rom_font(b"ui_font".as_ptr(), 7);
        TITLE_FONT = rom_font(b"title_font".as_ptr(), 10);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    ui_font = rom_font("ui_font", 7);
    title_font = rom_font("title_font", 10);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    ui_font = rom_font("ui_font", 7);
    title_font = rom_font("title_font", 10);
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_sound

Loads a sound from the data pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_sound(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_sound(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Sound handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
        COIN_SFX = rom_sound(b"coin".as_ptr(), 4);
        MUSIC = rom_sound(b"level1_bgm".as_ptr(), 10);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    jump_sfx = rom_sound("jump", 4);
    coin_sfx = rom_sound("coin", 4);
    music = rom_sound("level1_bgm", 10);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    jump_sfx = rom_sound("jump", 4);
    coin_sfx = rom_sound("coin", 4);
    music = rom_sound("level1_bgm", 10);
}
```
{{#endtab}}

{{#endtabs}}

---

## Raw Data Access

For custom data formats (level data, dialog scripts, etc.).

### rom_data_len

Gets the size of raw data in the pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_data_len(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_data_len(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_data_len(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Size in bytes (0 if not found)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    uint32_t len = rom_data_len("level1_map", 10);
    if (len > 0) {
        // Allocate buffer and load
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    const len = rom_data_len("level1_map", 10);
    if (len > 0) {
        // Allocate buffer and load
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_data

Copies raw data from the pack into WASM memory.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_data(id_ptr: *const u8, id_len: u32, out_ptr: *mut u8, max_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_data(const uint8_t* id_ptr, uint32_t id_len, uint8_t* out_ptr, uint32_t max_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_data(id_ptr: [*]const u8, id_len: u32, out_ptr: [*]u8, max_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID |
| id_len | `u32` | Length of asset ID |
| out_ptr | `*mut u8` | Destination buffer in WASM memory |
| max_len | `u32` | Maximum bytes to copy |

**Returns:** Bytes copied (0 if not found or buffer too small)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint8_t level_data[4096] = {0};

NCZX_EXPORT void init(void) {
    uint32_t len = rom_data_len("level1", 6);
    if (len <= 4096) {
        rom_data("level1", 6, level_data, 4096);
        parse_level(level_data, len);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var level_data: [4096]u8 = [_]u8{0} ** 4096;

export fn init() void {
    const len = rom_data_len("level1", 6);
    if (len <= 4096) {
        _ = rom_data("level1", 6, &level_data, 4096);
        parse_level(level_data[0..len]);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Game Manifest (nether.toml)

Assets are bundled using the `nether.toml` manifest:

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
path = "assets/level1.nczxmesh"

[[assets.meshes]]
id = "player_model"
path = "assets/player.nczxmesh"

[[assets.skeletons]]
id = "player_rig"
path = "assets/player.nczxskel"

[[assets.animations]]
id = "walk"
path = "assets/walk.nczxanim"

[[assets.sounds]]
id = "jump"
path = "assets/sfx/jump.raw"

[[assets.sounds]]
id = "level1_bgm"
path = "assets/music/level1.raw"

[[assets.fonts]]
id = "ui_font"
path = "assets/fonts/ui.nczxfont"

[[assets.data]]
id = "level1"
path = "assets/levels/level1.bin"
```

Build with:
```bash
nether build
nether pack  # Creates .nczx ROM file
```

---

## Memory Model

```
ROM (16MB)          RAM (4MB)           VRAM (4MB)
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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Asset handles
static uint32_t player_tex = 0;
static uint32_t player_mesh = 0;
static uint32_t player_skel = 0;
static uint32_t walk_anim = 0;
static uint32_t idle_anim = 0;
static uint32_t jump_sfx = 0;
static uint32_t music = 0;
static uint32_t ui_font = 0;

// Level data (copied to WASM memory)
static uint8_t level_data[8192] = {0};
static uint32_t level_size = 0;

NCZX_EXPORT void init(void) {
    // Graphics assets → VRAM
    player_tex = rom_texture("player", 6);
    player_mesh = rom_mesh("player", 6);
    player_skel = rom_skeleton("player_rig", 10);

    // Animations → GPU storage
    walk_anim = rom_keyframes("walk", 4);
    idle_anim = rom_keyframes("idle", 4);

    // Audio → Audio memory
    jump_sfx = rom_sound("jump", 4);
    music = rom_sound("music", 5);

    // Font → VRAM
    ui_font = rom_font("ui", 2);

    // Level data → WASM RAM (copied)
    level_size = rom_data_len("level1", 6);
    if (level_size <= 8192) {
        rom_data("level1", 6, level_data, 8192);
    }

    // Start music
    music_play(music, 0.7);
}

NCZX_EXPORT void render(void) {
    // Use loaded assets
    texture_bind(player_tex);
    skeleton_bind(player_skel);
    keyframe_bind(walk_anim, frame);
    draw_mesh(player_mesh);

    font_bind(ui_font);
    draw_text("SCORE: 0", 8, 10.0, 10.0, 16.0, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Asset handles
var player_tex: u32 = 0;
var player_mesh: u32 = 0;
var player_skel: u32 = 0;
var walk_anim: u32 = 0;
var idle_anim: u32 = 0;
var jump_sfx: u32 = 0;
var music: u32 = 0;
var ui_font: u32 = 0;

// Level data (copied to WASM memory)
var level_data: [8192]u8 = [_]u8{0} ** 8192;
var level_size: u32 = 0;

export fn init() void {
    // Graphics assets → VRAM
    player_tex = rom_texture("player", 6);
    player_mesh = rom_mesh("player", 6);
    player_skel = rom_skeleton("player_rig", 10);

    // Animations → GPU storage
    walk_anim = rom_keyframes("walk", 4);
    idle_anim = rom_keyframes("idle", 4);

    // Audio → Audio memory
    jump_sfx = rom_sound("jump", 4);
    music = rom_sound("music", 5);

    // Font → VRAM
    ui_font = rom_font("ui", 2);

    // Level data → WASM RAM (copied)
    level_size = rom_data_len("level1", 6);
    if (level_size <= 8192) {
        _ = rom_data("level1", 6, &level_data, 8192);
    }

    // Start music
    music_play(music, 0.7);
}

export fn render() void {
    // Use loaded assets
    texture_bind(player_tex);
    skeleton_bind(player_skel);
    keyframe_bind(walk_anim, frame);
    draw_mesh(player_mesh);

    font_bind(ui_font);
    draw_text("SCORE: 0", 8, 10.0, 10.0, 16.0, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Textures](./textures.md), [Meshes](./meshes.md), [Audio](./audio.md), [Animation](./animation.md)
