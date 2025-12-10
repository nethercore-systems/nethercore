# Asset Pack System Specification

**Status:** Design Specification
**Target:** Emberware Z (and Classic with appropriate limits)
**Last Updated:** 2025-12-10

---

## Summary

Implement a **12MB ROM + 4MB RAM** memory model with a datapack-based system. This separates immutable data from game state, enabling efficient rollback (only 4MB snapshotted) while providing generous content headroom (12MB total ROM).

---

## Final Specs: Emberware Z

| Resource | Limit | Description |
|----------|-------|-------------|
| **ROM (Cartridge)** | 12 MB | Total game content (WASM code + assets) |
| **WASM Code** | ≤4 MB | Must fit entirely in linear memory |
| **Assets** | ≤(12MB - code) | Remaining ROM space after code |
| **RAM** | 4 MB | WASM linear memory (code + heap + stack) |
| **VRAM** | 4 MB | GPU textures and mesh buffers |
| **Rollback** | 4 MB | Only RAM gets snapshotted (~0.25ms with xxHash3) |

### Memory Rules

1. **WASM code must fit in RAM** — The `.wasm` file loads entirely into linear memory, so code ≤4MB
2. **ROM = code + assets** — Total cartridge content ≤12MB
3. **Assets = 12MB - code_size** — Larger code means less asset budget

### Key Insight

Assets loaded via the asset API go directly to VRAM/audio memory on the host — they never touch WASM linear memory. Only the **handles** (u32 IDs) live in game state, making rollback fast and efficient.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    .ewz ROM File (≤12MB)                    │
├─────────────────────────────────────────────────────────────┤
│  EWZ Header (4 bytes)                                       │
│  ├── Magic: "EWZ\0"                                         │
├─────────────────────────────────────────────────────────────┤
│  ZRom (bitcode serialized)                                  │
│  ├── version: u32                                           │
│  ├── metadata: ZMetadata                                    │
│  ├── code: Vec<u8>         ← WASM bytecode (≤4MB)          │
│  ├── data_pack: Option<ZDataPack>  ← NEW: Bundled data     │
│  ├── thumbnail: Option<Vec<u8>>                            │
│  └── screenshots: Vec<Vec<u8>>                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    ZDataPack Structure                      │
├─────────────────────────────────────────────────────────────┤
│  ZDataPack {                                                │
│      textures: Vec<PackedTexture>,   // RGBA8 pixel data   │
│      meshes: Vec<PackedMesh>,        // GPU-ready vertices │
│      skeletons: Vec<PackedSkeleton>, // IBMs only (GPU)    │
│      fonts: Vec<PackedFont>,         // Bitmap font atlas  │
│      sounds: Vec<PackedSound>,       // PCM audio data     │
│      data: Vec<PackedData>,          // Raw bytes (levels) │
│  }                                                          │
│                                                             │
│  Design principle: STRICTLY GPU-ready POD data only.       │
│  No metadata that belongs in game code (bone names, etc.)  │
│  Data pack is just a memcpy source for GPU upload.         │
│                                                             │
│  Lookup: FxHash for O(1) runtime access by string ID.      │
└─────────────────────────────────────────────────────────────┘
```

---

## New ROM Loading FFI

### Design Principles

1. **String-based IDs** - Assets referenced by name, not index
2. **Init-only loading** - All `rom_*` functions only work in `init()`
3. **Returns handles** - Same handle types as existing `load_*` functions
4. **Backwards compatible** - Existing `load_texture()` etc. still work

### New FFI Functions

All ROM functions take string ID by pointer+length. GPU resources return handles, raw data copies to WASM memory.

```rust
// ═══════════════════════════════════════════════════════════
// GPU RESOURCES (return handles, memcpy to GPU)
// ═══════════════════════════════════════════════════════════

fn rom_texture(id_ptr: u32, id_len: u32) -> u32   // VRAM upload
fn rom_mesh(id_ptr: u32, id_len: u32) -> u32      // VRAM upload
fn rom_skeleton(id_ptr: u32, id_len: u32) -> u32  // Upload IBMs to GPU
fn rom_font(id_ptr: u32, id_len: u32) -> u32      // Upload atlas to VRAM
fn rom_sound(id_ptr: u32, id_len: u32) -> u32     // Register audio buffer

// ═══════════════════════════════════════════════════════════
// RAW DATA (copies into WASM linear memory)
// ═══════════════════════════════════════════════════════════

/// Get byte size of raw data (for buffer allocation)
fn rom_data_len(id_ptr: u32, id_len: u32) -> u32

/// Copy raw data into WASM memory buffer
/// Game allocates buffer, host copies bytes into it
fn rom_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) -> u32

// ═══════════════════════════════════════════════════════════
// UTILITY
// ═══════════════════════════════════════════════════════════

fn rom_exists(id_ptr: u32, id_len: u32) -> u32  // 1 = exists, 0 = not found
```

### Usage Example (Game Code)

```rust
// Before (include_bytes approach - still works for simple games!)
static PLAYER_PNG: &[u8] = include_bytes!("assets/player.png");
fn init() {
    let (w, h, pixels) = decode_png(PLAYER_PNG);
    PLAYER_TEX = load_texture(w, h, pixels.as_ptr());
}

// After (ROM data pack approach - recommended)
fn init() {
    // Textures
    PLAYER_TEX = rom_texture(b"player".as_ptr(), 6);

    // Meshes
    STAGE_MESH = rom_mesh(b"stage1".as_ptr(), 6);

    // Skinned characters
    CHAR_MESH = rom_mesh(b"character".as_ptr(), 9);
    CHAR_SKELETON = rom_skeleton(b"character".as_ptr(), 9);

    // Fonts
    MAIN_FONT = rom_font(b"pixel_font".as_ptr(), 10);

    // Audio
    JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
}
```

---

## Asset Pack Format

Uses the formats defined in `docs/reference/asset-pipeline.md`:

### Asset ID Strategy

- **Type:** `String` for ergonomics and readability
- **Lookup:** Hash internally (FxHash) for O(1) runtime lookup
- **Example:** `rom_texture("player_idle")`, `rom_mesh("stage1")`

### PackedTexture (EmberTexture format)

```rust
pub struct PackedTexture {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,          // RGBA8 pixels (width * height * 4 bytes)
}
```

**v1 Scope:** RGBA8 only. Lossless compression (e.g., Palette256) can be added later if devs push limits.

### PackedMesh (EmberMesh format)

```rust
pub struct PackedMesh {
    pub id: String,
    pub format: u8,              // Vertex format flags (0-15)
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_data: Vec<u8>,    // GPU-ready packed data (see asset-pipeline.md)
    pub index_data: Vec<u16>,    // Index buffer
}
```

**Input:** GPU-ready packed format as documented in asset-pipeline.md (EmberMesh `.embermesh` files).

### PackedSkeleton (for GPU skinning)

**STRICTLY GPU data only.** Per skeletal-animation-spec, only inverse bind matrices go to GPU. Bone names, hierarchy, rest pose belong in WASM memory (generated by ember-export as Rust const arrays).

```rust
/// GPU-ready skeleton data. Contains ONLY inverse bind matrices.
pub struct PackedSkeleton {
    pub id: String,
    pub bone_count: u32,
    pub inverse_bind_matrices: Vec<BoneMatrix3x4>,  // Ready for GPU upload
}
```

**BoneMatrix3x4:** Move the existing type from `emberware-z/src/state/mod.rs` to `shared/src/math.rs`. The shared version uses `[f32; 4]` arrays (POD), and emberware-z can add conversion methods to/from `glam::Vec4`:

```rust
// shared/src/math.rs - POD version for serialization
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct BoneMatrix3x4 {
    pub row0: [f32; 4],  // [m00, m01, m02, tx]
    pub row1: [f32; 4],  // [m10, m11, m12, ty]
    pub row2: [f32; 4],  // [m20, m21, m22, tz]
}

// emberware-z adds glam conversion methods via impl block
```

**What goes where:**
- **Asset pack (GPU):** `inverse_bind_matrices` only - memcpy to GPU buffer
- **WASM memory:** bone names, parents, rest pose - generated as `const` arrays by ember-export

**FFI:**
```rust
/// Load skeleton IBMs from ROM, upload to GPU
/// Returns skeleton handle (>0) or 0 on failure
fn rom_skeleton(id_ptr: u32, id_len: u32) -> u32
```

### PackedFont (EmberFont format)

```rust
pub struct PackedFont {
    pub id: String,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas_data: Vec<u8>,     // RGBA8 bitmap atlas (embedded, NOT a separate texture)
    pub line_height: f32,
    pub baseline: f32,
    pub glyphs: Vec<PackedGlyph>,
}

pub struct PackedGlyph {
    pub codepoint: u32,
    pub x: u16, pub y: u16,      // Position in atlas
    pub w: u16, pub h: u16,      // Size in atlas
    pub x_offset: f32,           // Render offset
    pub y_offset: f32,
    pub advance: f32,            // Horizontal advance
}
```

**Note:** The font's atlas texture is embedded in the font asset itself. When you call `rom_font()`, the host uploads the atlas to VRAM internally and returns a font handle. You don't need to manage the atlas texture separately.

**FFI:**
```rust
/// Load a font from ROM by ID
/// Returns font handle (>0) or 0 on failure
fn rom_font(id_ptr: u32, id_len: u32) -> u32
```

### PackedSound (EmberSound format)

```rust
pub struct PackedSound {
    pub id: String,
    pub data: Vec<i16>,  // 22050Hz mono PCM (sample_count = data.len())
}
```

**v1 Scope:** Mono PCM audio only (22050Hz, i16). Sample count is derived from `data.len()`.

**Future stereo support** would add a `channels: u8` field, where frame_count = `data.len() / channels`.

### PackedData (Raw bytes for custom data)

For levels, dialogue, custom formats - anything that isn't a standard GPU resource.

```rust
/// Raw byte data (levels, dialogue, custom formats)
pub struct PackedData {
    pub id: String,
    pub data: Vec<u8>,  // Opaque bytes, game interprets
}
```

**FFI for raw data access:**

The host copies bytes into WASM linear memory (no multi-memory needed):

```rust
/// Get size of raw data (for buffer allocation)
/// Returns byte count, or 0 if not found
fn rom_data_len(id_ptr: u32, id_len: u32) -> u32

/// Copy raw data into WASM memory
/// Game allocates buffer, host copies into it
/// Returns bytes written, or 0 on failure
fn rom_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) -> u32
```

**Usage:**
```rust
fn load_level(name: &str) -> Vec<u8> {
    let len = rom_data_len(name.as_ptr() as u32, name.len() as u32);
    if len == 0 { return Vec::new(); }

    let mut buffer = vec![0u8; len as usize];
    rom_data(name.as_ptr() as u32, name.len() as u32,
             buffer.as_mut_ptr() as u32, len);
    buffer
}
```

### Init-Only Loading

**All `rom_*` functions are init-only.** They can only be called during `init()`, not in `update()` or `render()`.

**Why:**
- VRAM budget is fixed at load time (no mid-game surprises)
- No rollback complications (assets never change during gameplay)
- Matches existing `load_texture()`, `load_mesh()`, `load_sound()` behavior
- Simple mental model for developers

**Future:** If streaming/level loading is needed, we can add `rom_*_streaming()` variants that are rollback-aware.

---

## Build Tooling: `ember` CLI

The `ember` CLI is a **standalone executable** — not tied to the Rust ecosystem. Game developers can use any language that compiles to WASM.

### Command

```bash
# Build from manifest (auto-detects console from manifest)
ember build --manifest ember.toml

# Build with file watching (rebuild on changes)
ember build --manifest ember.toml --watch

# Output file is auto-named based on console:
# - Emberware Z: my-game.ewz
# - Emberware Classic: my-game.ewc
```

### Distribution

The `ember` CLI is distributed as a standalone binary:
- Windows: `ember.exe`
- macOS: `ember` (universal binary)
- Linux: `ember`

Available via:
- GitHub releases (direct download)
- Homebrew: `brew install emberware/tap/ember`
- Cargo: `cargo install ember-cli` (for Rust devs)
- npm: `npx @emberware/cli build` (for JS devs)

### ember.toml Manifest

```toml
[package]
id = "my-game"
title = "My Awesome Game"
author = "Developer"
version = "0.1.0"
console = "emberware-z"  # or "emberware-classic"

[build]
wasm = "target/wasm32-unknown-unknown/release/my_game.wasm"
output = "my-game.ewz"   # Optional, auto-derived from console if omitted

[assets.textures]
player = "assets/textures/player.png"
enemy = "assets/textures/enemy.png"

[assets.meshes]  # Z-only (Classic uses tilemaps/sprites instead)
player_mesh = "assets/meshes/player.glb"
stage1 = "assets/meshes/stage1.glb"

[assets.skeletons]  # Z-only, extracts IBMs only (bone metadata via ember-export)
player_skeleton = "assets/meshes/player.glb"

[assets.fonts]
main_font = "assets/fonts/pixel.ttf"

[assets.sounds]
jump = "assets/audio/jump.wav"
hit = "assets/audio/hit.wav"

[assets.data]  # Raw bytes for custom data (levels, dialogue, etc.)
level1 = "assets/levels/level1.bin"
level2 = "assets/levels/level2.bin"
dialogue = "assets/dialogue.json"
```

**Console-specific validation:**
- **Emberware Z:** ROM ≤12MB, WASM ≤4MB, meshes/skeletons allowed
- **Emberware Classic:** ROM ≤4MB, WASM ≤2MB, no meshes/skeletons (2D only)

### Build Process

1. **Load WASM** - Validate size ≤4MB, validate WASM magic
2. **Process textures** - Decode PNG/source → convert to target format → pack
3. **Process meshes** - Load GLTF/OBJ/FBX → pack vertices in GPU format
4. **Process audio** - Convert WAV/OGG → 22050Hz mono i16 PCM
5. **Process music** - Copy tracker modules as-is
6. **Validate total** - Ensure ROM ≤12MB
7. **Bundle** - Create AssetPack, serialize ZRom, write .ewz

---

## Implementation Tasks

### Phase 1: Update Specs & Constants

**Files:**
- `shared/src/console.rs` — Update limits
- `core/src/rollback/config.rs` — Update MAX_STATE_SIZE
- `core/src/wasm/state.rs` — Update default ram_limit
- `core/src/wasm/mod.rs` — Update default in GameInstance::new()

Changes:
```rust
// shared/src/console.rs
pub const EMBERWARE_Z_ROM_LIMIT: usize = 12 * 1024 * 1024;  // 12MB cartridge
pub const EMBERWARE_Z_RAM_LIMIT: usize = 4 * 1024 * 1024;   // 4MB linear memory
pub const EMBERWARE_Z_VRAM_LIMIT: usize = 4 * 1024 * 1024;  // 4MB GPU

// core/src/rollback/config.rs
pub const MAX_STATE_SIZE: usize = 4 * 1024 * 1024;  // 4MB (RAM only)

// core/src/wasm/mod.rs - GameInstance::new() default
Self::with_ram_limit(engine, module, linker, 4 * 1024 * 1024)  // 4MB
```

### Phase 2: Data Pack Structures

**New file:** `shared/src/cart/z_data_pack.rs` (Z-specific, prevents mixing consoles)

```rust
/// Emberware Z data pack. Console-specific to prevent mixing data.
pub struct ZDataPack {
    pub textures: Vec<PackedTexture>,
    pub meshes: Vec<PackedMesh>,
    pub skeletons: Vec<PackedSkeleton>,  // IBMs only, GPU-ready
    pub fonts: Vec<PackedFont>,
    pub sounds: Vec<PackedSound>,
    pub data: Vec<PackedData>,           // Raw bytes (levels, etc.)
}

// All structs are STRICTLY GPU-ready POD data (or audio PCM)
pub struct PackedTexture { pub id: String, pub width: u32, pub height: u32, pub data: Vec<u8> }
pub struct PackedMesh { pub id: String, pub format: u8, pub vertex_count: u32, pub index_count: u32, pub vertex_data: Vec<u8>, pub index_data: Vec<u16> }
pub struct PackedSkeleton { pub id: String, pub bone_count: u32, pub inverse_bind_matrices: Vec<BoneMatrix3x4> }  // IBMs ONLY
pub struct PackedFont { pub id: String, pub atlas_width: u32, pub atlas_height: u32, pub atlas_data: Vec<u8>, pub line_height: f32, pub baseline: f32, pub glyphs: Vec<PackedGlyph> }
pub struct PackedGlyph { pub codepoint: u32, pub x: u16, pub y: u16, pub w: u16, pub h: u16, pub x_offset: f32, pub y_offset: f32, pub advance: f32 }
pub struct PackedSound { pub id: String, pub data: Vec<i16> }  // 22050Hz mono PCM
pub struct PackedData { pub id: String, pub data: Vec<u8> }  // Opaque bytes
```

**New file:** `shared/src/math.rs` - Move BoneMatrix3x4 here (POD version with `[f32; 4]`)

**Modify:** `emberware-z/src/state/mod.rs` - Re-export from shared, add glam conversion methods

### Phase 3: Update ZRom Format

**File:** `shared/src/cart/z.rs`

- Add `data_pack: Option<ZDataPack>` field to ZRom (optional so code-only games work)
- Keep version at 1 (no backwards compat needed yet)

### Phase 4: ROM Loading FFI

**New file:** `emberware-z/src/ffi/rom.rs`

```rust
// GPU resources (return handles)
fn rom_texture(id_ptr: u32, id_len: u32) -> u32   // Upload to VRAM
fn rom_mesh(id_ptr: u32, id_len: u32) -> u32      // Upload to VRAM
fn rom_skeleton(id_ptr: u32, id_len: u32) -> u32  // Upload IBMs to GPU
fn rom_font(id_ptr: u32, id_len: u32) -> u32      // Upload atlas, return font handle
fn rom_sound(id_ptr: u32, id_len: u32) -> u32     // Register audio

// Raw data (copies into WASM memory)
fn rom_data_len(id_ptr: u32, id_len: u32) -> u32                          // Get size
fn rom_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) -> u32  // Copy to WASM

// Utility
fn rom_exists(id_ptr: u32, id_len: u32) -> u32    // 1 = exists, 0 = not found
```

**File:** `emberware-z/src/ffi/mod.rs` — Register new FFI

### Phase 5: Host-Side Data Pack Storage

**File:** `emberware-z/src/state/ffi_state.rs`

```rust
pub data_pack: Option<Arc<ZDataPack>>,  // Loaded from ROM, stored on host (Z-specific)
```

**File:** `emberware-z/src/lib.rs` or game loading code — Pass data pack to FFI state

### Phase 6: `ember` CLI (Standalone Executable)

**New crate:** `ember-cli/` (standalone binary, distributed separately)

- Parse `ember.toml` manifest
- Auto-detect console from `package.console` field
- Load and validate WASM (size varies by console)
- Process textures (PNG → RGBA8)
- Process meshes (EmberMesh format) — Z only
- Process skeletons (inverse bind from glTF) — Z only
- Process fonts (TTF → bitmap atlas)
- Process audio (WAV → PCM)
- Validate total ROM size (varies by console)
- Bundle into .ewz or .ewc based on console

**Subcommands:**
- `ember build` — Build production ROM
- `ember build --watch` — Rebuild on file changes
- `ember check` — Validate manifest and assets without building

### Phase 7: Update Documentation

**Files:**
- `docs/reference/ffi.md` — Add asset API section
- `docs/reference/emberware-z.md` — Update memory model specs
- `docs/reference/asset-pipeline.md` — Update status, add ember.toml reference
- `CLAUDE.md` — Update specs

---

## Files to Modify/Create

| File | Action | Description |
|------|--------|-------------|
| **Phase 1: Constants** | | |
| `shared/src/console.rs` | Modify | Add ROM_LIMIT, update RAM_LIMIT to 4MB |
| `core/src/rollback/config.rs` | Modify | MAX_STATE_SIZE → 4MB |
| `core/src/wasm/state.rs` | Modify | Default ram_limit → 4MB |
| `core/src/wasm/mod.rs` | Modify | GameInstance::new() default → 4MB |
| **Phase 2-3: Data Pack** | | |
| `shared/src/math.rs` | Create | BoneMatrix3x4 POD type (moved from emberware-z) |
| `shared/src/cart/z_data_pack.rs` | Create | ZDataPack + packed types (GPU-ready POD) |
| `shared/src/cart/z.rs` | Modify | Add `data_pack: Option<ZDataPack>` to ZRom |
| `shared/src/cart/mod.rs` | Modify | Export z_data_pack module |
| `shared/src/lib.rs` | Modify | Export math module |
| `emberware-z/src/state/mod.rs` | Modify | Re-export BoneMatrix3x4, add glam conversions |
| **Phase 4-5: FFI** | | |
| `emberware-z/src/ffi/rom.rs` | Create | rom_* FFI functions (texture, mesh, skeleton, font, sound, data) |
| `emberware-z/src/ffi/mod.rs` | Modify | Register rom FFI functions |
| `emberware-z/src/state/ffi_state.rs` | Modify | Add data_pack: Option<Arc<ZDataPack>> |
| **Phase 6: ember CLI** | | |
| `ember-cli/` | Create | New crate for standalone ember CLI |
| `ember-cli/src/main.rs` | Create | CLI entry point with build/check subcommands |
| `ember-cli/src/manifest.rs` | Create | ember.toml parsing |
| `ember-cli/src/build.rs` | Create | ROM bundling logic |
| `ember-cli/src/watch.rs` | Create | File watcher for --watch mode |
| `ember-cli/src/processors/` | Create | Asset processors (textures, meshes, fonts, audio) |
| `ember-cli/Cargo.toml` | Create | Dependencies: clap, notify, image, hound, ttf-parser, etc. |
| **Phase 7: Docs** | | |
| `docs/reference/ffi.md` | Modify | Add Asset Loading section |
| `docs/reference/emberware-z.md` | Modify | Update Memory Model section |
| `docs/reference/asset-pipeline.md` | Modify | Update status, add ember.toml docs |
| `CLAUDE.md` | Modify | Update specs summary |

---

## Development Workflow

### Recommended: Asset Pack API

```
1. ember.toml manifest defines assets
2. ember build → my-game.ewz
3. Game uses rom_texture("player"), rom_mesh("stage1"), etc.
```

### Alternative: include_bytes! (recommended for iteration)

The datapack is **optional**. Developers can still embed data in WASM:

```rust
// Frame data in WASM - debug panel can tweak these values!
static mut PUNCH_DATA: HitboxData = HitboxData {
    startup: 4, active: 3, recovery: 8,
    damage: 30, hitbox: Rect { x: 20, y: -40, w: 50, h: 30 },
};

// Textures via include_bytes - works fine for small games
static PLAYER_PNG: &[u8] = include_bytes!("assets/player.png");
```

**When to use which:**

| Approach | Best For | Debug Panel | Memory |
|----------|----------|-------------|--------|
| WASM (`include_bytes!`, const arrays) | Iterating on data, small games | Works | Uses 4MB RAM |
| Datapack (`rom_*`) | Large assets, finalized data | N/A (data on host) | Uses 12MB ROM |

**Recommended workflow:**
1. Start with data in WASM for rapid iteration with debug panel
2. Move finalized/large assets to datapack when you need more RAM
3. Keep tweakable data (frame data, balance values) in WASM

**Note:** Data accessed every frame (frame data, hitbox tables, etc.) needs to be in WASM linear memory anyway for fast access. Calling `rom_data()` just copies it there. For such data, `include_bytes!` or const arrays are simpler and support debug panel tweaking.

---

## Development Workflow

### Build & Watch

```bash
# Build production ROM
ember build --manifest ember.toml

# Rebuild on file changes (for development)
ember build --manifest ember.toml --watch
```

When using `--watch`, the console can detect ROM changes and restart the game automatically. Same workflow as `include_bytes!` — change file, rebuild, reload.

### Workflow Comparison

| Aspect | include_bytes! | Datapack |
|--------|----------------|----------|
| Change WASM code | Rebuild WASM | Rebuild WASM |
| Change asset | Rebuild WASM | Rebuild ROM only (faster) |
| Memory budget | 4MB RAM total | 4MB RAM + 8MB ROM assets |
| Debug panel | Works | Works (for data in WASM) |

---

## v1 Scope

### Included
- ✅ 12MB ROM / 4MB RAM memory model
- ✅ `ZDataPack` in .ewz format (console-specific, prevents mixing)
- ✅ `rom_texture()`, `rom_mesh()`, `rom_skeleton()`, `rom_font()`, `rom_sound()` FFI
- ✅ `rom_data()`, `rom_data_len()` for raw bytes (levels, dialogue, custom)
- ✅ RGBA8 textures (uncompressed)
- ✅ EmberMesh GPU-ready packed format
- ✅ Skeletons: IBMs only (GPU-ready), bone metadata via ember-export to WASM
- ✅ Bitmap fonts with embedded atlas
- ✅ 22050Hz mono PCM audio
- ✅ `ember build` command (standalone CLI, unified for all consoles)
- ✅ `ember build --watch` for automatic rebuilds during development
- ✅ Size validation (ROM ≤12MB for Z, ≤4MB for Classic)
- ✅ Init-only asset loading (avoids rollback complications)
- ✅ Strictly GPU-ready POD data (no bloat in data pack)
- ✅ Optional: keep tweakable data in WASM for debug panel iteration

### Deferred
- ❌ Live asset reload (swap assets without restart) - future release
- ❌ Tracker music (MOD/XM/IT/S3M) - future release
- ❌ Texture compression (BC1/BC3/BC7) - add if devs hit limits
- ❌ Palette256 textures - add if devs hit limits
- ❌ Mid-game streaming/loading - would need rollback-aware API

---

## Performance

| Metric | Value |
|--------|-------|
| Rollback state | 4 MB |
| Rollback checksum | ~0.25ms (xxHash3) |
| 8-frame rollback | ~2ms total |
| Asset ceiling | Up to 12MB - code_size |
| WASM code limit | 4 MB |
