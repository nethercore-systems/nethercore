# Nethercore ROM Format Specification

## Overview

Nethercore uses console-specific binary ROM formats for game distribution. Each fantasy console has its own ROM format with a unique file extension and structure, ensuring type-safety and preventing incompatible games from being loaded on the wrong console.

**Current ROM Formats:**
- `.nczx` - Nethercore ZX (3D home console)
- `.ncc` - Nethercore Chroma (2D handheld console) - Future
- `.ncz` - Nethercore z (3D handheld console) - Future
- `.nccz` - Nethercore ChroMAX (2D Home console) - Future


**Why Console-Specific?**
- Type-safe - impossible to load wrong ROM in wrong console
- Console-specific metadata embedded in ROM structure
- Compile-time guarantees about compatibility
- No runtime format detection needed
- Each console can evolve independently

**Why Bitcode?**
- Compact binary format with excellent compression
- Native Rust struct serialization via `bitcode::Encode`/`Decode`
- Fast deserialization (faster than ZIP + TOML parsing)
- Games can `#[derive(Encode)]` for ROM metadata
- No external dependencies for inspection (just Rust tooling)

## Nethercore ZX ROM Format (.nczx)

### File Structure

```
game.nczx (binary file, max 16MB)
├── Magic bytes: "NCZX" (4 bytes)
└── ZXRom (bitcode-encoded):
    ├── version: u32
    ├── metadata: ZXMetadata
    ├── code: Vec<u8> (WASM bytes, max 4MB)
    ├── data_pack: Option<ZXDataPack> (bundled assets)
    ├── thumbnail: Option<Vec<u8>> (256x256 PNG)
    └── screenshots: Vec<Vec<u8>> (PNG, max 5)
```

### Magic Bytes

All Nethercore ZX ROM files start with the magic bytes: `NCZX` (hex: `4E 43 5A 58`)

This allows tools to quickly identify the ROM type and reject invalid files.

### ZXRom Structure

```rust
pub struct ZXRom {
    /// ROM format version (currently 1)
    pub version: u32,

    /// Game metadata
    pub metadata: ZXMetadata,

    /// Compiled WASM code (max 4MB)
    pub code: Vec<u8>,

    /// Optional bundled assets (textures, meshes, sounds, etc.)
    pub data_pack: Option<ZXDataPack>,

    /// Optional thumbnail (256x256 PNG, extracted locally during installation)
    pub thumbnail: Option<Vec<u8>>,

    /// Optional screenshots (PNG bytes, max 5)
    /// Stored in ROM but NOT extracted during installation to save disk space.
    pub screenshots: Vec<Vec<u8>>,
}
```

### ZXMetadata Structure

```rust
pub struct ZXMetadata {
    // Core game info
    /// Game slug (e.g., "platformer")
    pub id: String,

    /// Display title
    pub title: String,

    /// Primary author/studio name (for display)
    pub author: String,

    /// Semantic version (e.g., "1.0.0")
    pub version: String,

    /// Game description
    pub description: String,

    /// Category tags
    pub tags: Vec<String>,

    // Platform integration (optional foreign keys)
    /// UUID linking to platform game record
    pub platform_game_id: Option<String>,

    /// UUID linking to platform user/studio
    pub platform_author_id: Option<String>,

    // Creation info
    /// ISO 8601 timestamp when ROM was created
    pub created_at: String,

    /// Tool version that created this ROM (e.g., nether-cli/xtask)
    pub tool_version: String,

    // Z-specific settings
    /// Render mode: 0=Lambert, 1=Matcap, 2=MR-Blinn-Phong, 3=Specular-Shininess
    pub render_mode: Option<u32>,

    /// Default resolution (e.g., "640x480")
    pub default_resolution: Option<String>,

    /// Target FPS
    pub target_fps: Option<u32>,

    /// Netplay metadata for the NCHS protocol (tick rate, max players, ROM hash)
    pub netplay: NetplayMetadata,
}
```

---

## ZXDataPack (Bundled Assets)

Games can bundle pre-processed assets directly in the ROM for efficient loading.

### Structure

```rust
pub struct ZXDataPack {
    pub textures: Vec<PackedTexture>,      // GPU-ready textures (RGBA8/BC7/BC5)
    pub meshes: Vec<PackedMesh>,           // GPU-ready meshes
    pub skeletons: Vec<PackedSkeleton>,    // Inverse bind matrices
    pub keyframes: Vec<PackedKeyframes>,   // Animation clips
    pub fonts: Vec<PackedFont>,            // Bitmap font atlases
    pub sounds: Vec<PackedSound>,          // PCM audio data
    pub data: Vec<PackedData>,             // Raw opaque data
    pub trackers: Vec<PackedTracker>,      // XM tracker modules
}
```

### Textures (PackedTexture)

```rust
pub struct PackedTexture {
    pub id: String,             // Asset ID
    pub width: u16,             // Max 65535 pixels
    pub height: u16,            // Max 65535 pixels
    pub format: TextureFormat,  // RGBA8, BC7, or BC5
    pub data: Vec<u8>,          // Raw pixel/block data
}

pub enum TextureFormat {
    Rgba8,  // Uncompressed: 4 bytes/pixel
    Bc7,    // Compressed: 16 bytes/4x4 block (~4x smaller)
    Bc5,    // Compressed (RG): 16 bytes/4x4 block (normal maps)
}
```

**Compression Selection (nether-cli `nether pack`):**
- `compress_textures = false`: `Rgba8` for all textures (pixel-perfect, full alpha)
- `compress_textures = true`: `Bc7` for all textures (4× compression, stipple transparency)
- `Bc5` is reserved for normal maps (2-channel RG; Z reconstructed in shader)

**Size calculation:**
- RGBA8: `width * height * 4` bytes
- BC7/BC5: `((width+3)/4) * ((height+3)/4) * 16` bytes

### Meshes (PackedMesh)

```rust
pub struct PackedMesh {
    pub id: String,           // Asset ID
    pub format: u8,           // Vertex format flags (0-31)
    pub vertex_count: u32,    // Number of vertices
    pub index_count: u32,     // Number of indices
    pub vertex_data: Vec<u8>, // Packed GPU-ready vertices
    pub index_data: Vec<u16>, // u16 indices
}
```

**Vertex format flags (bitmask):**

Flags are defined in `zx-common/src/packing.rs`. `vertex_data` is written in the **packed GPU layout**; compute stride with `zx_common::vertex_stride_packed(format)`.

| Bit | Flag | Packed representation | Adds |
|-----|------|------------------------|------|
| 0 (0x01) | UV | `unorm16x2` | +4 bytes |
| 1 (0x02) | Color | `unorm8x4` | +4 bytes |
| 2 (0x04) | Normal | octahedral `u32` | +4 bytes |
| 3 (0x08) | Skinned | `u8x4` indices + `unorm8x4` weights | +8 bytes |
| 4 (0x10) | Tangent | octahedral `u32` + sign bit | +4 bytes |

**Base packed stride:** 8 bytes (position: `f16x4`)

**Notes:**
- `TANGENT` requires `NORMAL` (tangent-space normal mapping).
- The human-readable “format N” values are just bitwise combinations (0–31).

**Examples (packed stride):**
- Format 0: 8 bytes (position only)
- Format 1: 12 bytes (position + UV)
- Format 7: 20 bytes (position + UV + color + normal)
- Format 15: 28 bytes (position + UV + color + normal + skinned)
- Format 31: 32 bytes (all flags)

**Binary File Format (.nczxmesh):**
```
Header (12 bytes):
  0x00: vertex_count (u32, LE)
  0x04: index_count (u32, LE)
  0x08: format (u8)
  0x09: padding (3 bytes)

Data:
  stride = zx_common::vertex_stride_packed(format)
  vertex_count * stride bytes: vertex data
  index_count * 2 bytes: u16 indices (LE)
```

### Skeletons (PackedSkeleton)

```rust
pub struct PackedSkeleton {
    pub id: String,                                // Asset ID
    pub bone_count: u32,                           // Max 256 bones
    pub inverse_bind_matrices: Vec<BoneMatrix3x4>, // GPU-ready matrices
}
```

**Binary File Format (.nczxskel):**
```
Header (8 bytes):
  0x00: bone_count (u32, LE)
  0x04: reserved (u32, must be 0)

Data:
  bone_count * 48 bytes: inverse bind matrices
  Each matrix: 12 floats (3x4 column-major)
```

### Keyframes/Animations (PackedKeyframes)

```rust
pub struct PackedKeyframes {
    pub id: String,        // Asset ID
    pub bone_count: u8,    // Max 255 bones per frame
    pub frame_count: u16,  // Max 65535 frames
    pub data: Vec<u8>,     // Raw frame data
}
```

**Binary File Format (.nczxanim):**
```
Header (4 bytes):
  0x00: bone_count (u8)
  0x01: flags (u8, must be 0)
  0x02: frame_count (u16, LE)

Frame Data (frame_count * bone_count * 16 bytes):
  Each bone keyframe: 16 bytes
    rotation: u32 (smallest-three packed quaternion)
    position: 3x u16 (half-precision floats)
    scale: 3x u16 (half-precision floats)
```

### Fonts (PackedFont)

```rust
pub struct PackedFont {
    pub id: String,                 // Asset ID
    pub atlas_width: u32,           // Pixels
    pub atlas_height: u32,          // Pixels
    pub atlas_data: Vec<u8>,        // RGBA8 bitmap data
    pub line_height: f32,           // Pixels
    pub baseline: f32,              // Pixels from top
    pub glyphs: Vec<PackedGlyph>,   // Glyph metrics
}

pub struct PackedGlyph {
    pub codepoint: u32,   // Unicode codepoint
    pub x: u16,           // X in atlas (pixels)
    pub y: u16,           // Y in atlas (pixels)
    pub w: u16,           // Width (pixels)
    pub h: u16,           // Height (pixels)
    pub x_offset: f32,    // Horizontal render offset
    pub y_offset: f32,    // Vertical render offset
    pub advance: f32,     // Horizontal advance to next glyph
}
```

### Sounds (PackedSound)

```rust
pub struct PackedSound {
    pub id: String,      // Asset ID
    pub data: Vec<i16>,  // PCM samples
}
```

**Audio Specification:**
- Sample rate: 22050 Hz (mono)
- Format: i16 PCM (2 bytes per sample)
- Duration: `sample_count / 22050.0` seconds

### Raw Data (PackedData)

```rust
pub struct PackedData {
    pub id: String,      // Asset ID
    pub data: Vec<u8>,   // Opaque byte data
}
```

Used for level data, configuration, dialogue, or any custom binary format.

---

## Memory Limits

| Resource | Limit |
|----------|-------|
| ROM (total) | 16 MB |
| WASM code | 4 MB |
| RAM (linear memory) | 4 MB |
| VRAM (GPU resources) | 4 MB |

**Asset Loading Model:**
- Assets loaded via `rom_*` FFI go directly to VRAM/audio memory (host-managed)
- Only handles (u32) live in game state
- This enables efficient rollback (only 4MB RAM snapshotted)

---

## Asset File Extensions

These are the **standalone exported** ZX asset formats (see `nethercore_shared::ZX_ROM_FORMAT` and `tools/nether-export/`). Most projects will reference source assets (PNG/GLB/WAV/XM) in `nether.toml`, and `nether pack` will bundle them into the `.nczx` ROM.

| Asset Type | Extension | Format |
|------------|-----------|--------|
| ROM | `.nczx` | Bitcode |
| Mesh | `.nczxmesh` | POD binary |
| Texture | `.nczxtex` | POD binary |
| Sound | `.nczxsnd` | WAV (parsed to PCM) |
| Skeleton | `.nczxskel` | POD binary |
| Keyframes/Animation | `.nczxanim` | POD binary |

---

## Validation Rules

### Format Validation
- Magic bytes must be "NCZX"
- File must deserialize successfully using bitcode

### Version Validation
- ROM version must be <= current supported version (currently 1)
- Future versions may introduce new features while maintaining backward compatibility

### Metadata Validation
- Required fields must not be empty: `id`, `title`, `author`, `version`

### WASM Code Validation
- Code must be at least 4 bytes
- Must start with WASM magic bytes: `\0asm` (hex: `00 61 73 6D`)
- Note: `init`/`update`/`render` exports are optional at load time (missing exports are treated as no-ops), but real games should provide at least `update()` and `render()`.
- Note: rollback requires a `memory` export; without it, state snapshotting will fail at runtime.

### Console Settings Validation
- `nether-cli` validates `nether.toml` at build/pack time (e.g., `render_mode` range, `tick_rate` allowed values, `max_players` range).
- `zx-common::ZXRom::validate()` currently only validates required strings and WASM magic bytes; optional console settings are not range-checked when parsing a ROM.

---

## Creating ROMs

Use `nether pack` to create ROM files:

```bash
nether pack
```

This reads `nether.toml` in your project and creates the ROM with all assets bundled.

### nether.toml Example

```toml
[game]
id = "my-game"
title = "My Awesome Game"
author = "YourName"
version = "1.0.0"
description = "A fun game!"
tags = ["platformer", "action"]

# ZX-only rendering settings (defaults shown)
render_mode = 0         # 0=Lambert (default), 1=Matcap, 2=MR-Blinn-Phong, 3=Specular-Shininess
compress_textures = false

# Netplay-critical config (baked into ROM metadata)
tick_rate = 60          # 30, 60, or 120
max_players = 4         # 1-4

[netplay]
enabled = true

[build]
# Optional: override build command and/or WASM output path
# script = "cargo build --target wasm32-unknown-unknown --release"
# wasm = "target/wasm32-unknown-unknown/release/my_game.wasm"

[[assets.textures]]
id = "player"
path = "assets/textures/player.png"

[[assets.meshes]]
id = "player_mesh"
path = "assets/meshes/player.glb"

[[assets.sounds]]
id = "jump"
path = "assets/sounds/jump.wav"
```

## Inspecting ROMs

```bash
nether info my-game.nczx
```

Displays:
- Game information (title, author, version, description, tags)
- Creation info (timestamp, tool version, ROM version)
- Platform integration (UUIDs if present)
- Console settings (render mode, resolution, FPS)
- ROM contents (file sizes, asset counts)

---

## Installation to Local Library

When a ROM is installed to the local library:

```
<nethercore_data_dir>/games/{game_id}/
├── manifest.json        # Metadata for library UI
├── rom.wasm            # Extracted WASM code
└── thumbnail.png       # Extracted thumbnail (if present)
                        # Screenshots NOT extracted (save disk space)
```

---

## Technical Details

### Bitcode Serialization

ROMs use the [bitcode](https://crates.io/crates/bitcode) crate with native `Encode`/`Decode` traits.

**Format Properties:**
- Binary (not human-readable)
- Deterministic (same input -> same output)
- Compact (better compression than JSON/TOML)
- Fast (faster than ZIP + parsing)

### File Size Expectations

```
Minimal game (hello-world):
- WASM code: ~1KB
- Metadata: ~200 bytes
- Total: ~1.2KB

Medium game:
- WASM code: ~100KB
- Thumbnail: ~20KB
- Data pack: ~500KB
- Total: ~620KB

Large game with assets:
- WASM code: ~500KB
- Thumbnail: ~20KB
- 5 screenshots: ~2.5MB
- Data pack: ~8MB
- Total: ~11MB
```

---

## Error Messages

**"Invalid NCZX magic bytes"**
- File is not a Nethercore ZX ROM
- File may be corrupted
- Wrong ROM type

**"Unsupported NCZX version: X"**
- ROM was created with a newer tool version
- Update your launcher to support this ROM

**"Game ID cannot be empty"**
- ROM metadata is missing required `id` field

**"Invalid WASM code (missing \\0asm magic bytes)"**
- WASM file is corrupted or invalid

**"Failed to decode NCZX ROM"**
- ROM file is corrupted
- ROM was created with incompatible bitcode version

---

## Source Files

| Component | Location |
|-----------|----------|
| ROM Format Constants | `shared/src/rom_format.rs` |
| ZXRom Struct | `zx-common/src/formats/zx_rom.rs` |
| ZXDataPack Struct | `zx-common/src/formats/zx_data_pack/mod.rs` |
| Mesh Format | `zx-common/src/formats/mesh.rs` |
| Skeleton Format | `zx-common/src/formats/skeleton.rs` |
| Animation/Keyframes Format | `zx-common/src/formats/animation/` |
| Build Process | `tools/nether-cli/src/pack/mod.rs` |

---

## See Also

- [distributing-games.md](../contributing/distributing-games.md) - Complete guide for game developers
- [ffi.md](./ffi.md) - Nethercore FFI API reference
- [nethercore-zx.md](./nethercore-zx.md) - ZX-specific API documentation
