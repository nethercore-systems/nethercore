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
game.nczx (binary file, max 12MB)
├── Magic bytes: "NCZX" (4 bytes)
└── ZXRom (bitcode-encoded):
    ├── version: u32
    ├── metadata: ZMetadata
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
    pub metadata: ZMetadata,

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

### ZMetadata Structure

```rust
pub struct ZMetadata {
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

    /// nether-cli version that created this ROM
    pub tool_version: String,

    // Z-specific settings
    /// Render mode: 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid
    pub render_mode: Option<u32>,

    /// Default resolution (e.g., "640x480")
    pub default_resolution: Option<String>,

    /// Target FPS
    pub target_fps: Option<u32>,
}
```

---

## ZXDataPack (Bundled Assets)

Games can bundle pre-processed assets directly in the ROM for efficient loading.

### Structure

```rust
pub struct ZXDataPack {
    pub textures: Vec<PackedTexture>,      // GPU-ready textures
    pub meshes: Vec<PackedMesh>,           // GPU-ready meshes
    pub skeletons: Vec<PackedSkeleton>,    // Inverse bind matrices
    pub keyframes: Vec<PackedKeyframes>,   // Animation clips
    pub fonts: Vec<PackedFont>,            // Bitmap font atlases
    pub sounds: Vec<PackedSound>,          // PCM audio data
    pub data: Vec<PackedData>,             // Raw opaque data
}
```

### Textures (PackedTexture)

```rust
pub struct PackedTexture {
    pub id: String,             // Asset ID
    pub width: u16,             // Max 65535 pixels
    pub height: u16,            // Max 65535 pixels
    pub format: TextureFormat,  // RGBA8 or BC7
    pub data: Vec<u8>,          // Raw pixel/block data
}

pub enum TextureFormat {
    Rgba8,  // Uncompressed: 4 bytes/pixel
    Bc7,    // Compressed: 16 bytes/4x4 block (~4x smaller)
}
```

**Compression Selection (automated by nether-cli):**
- Render Mode 0 (Lambert): RGBA8 (pixel-perfect, full alpha)
- Render Modes 1-3 (Matcap/PBR/Hybrid): BC7 (4x compression, stipple transparency)

**Size calculation:**
- RGBA8: `width * height * 4` bytes
- BC7: `((width+3)/4) * ((height+3)/4) * 16` bytes

### Meshes (PackedMesh)

```rust
pub struct PackedMesh {
    pub id: String,           // Asset ID
    pub format: u8,           // Vertex format flags (0-15)
    pub vertex_count: u32,    // Number of vertices
    pub index_count: u32,     // Number of indices
    pub vertex_data: Vec<u8>, // Packed GPU-ready vertices
    pub index_data: Vec<u16>, // u16 indices
}
```

**Vertex Format Flags (bitwise):**
| Bit | Flag | Adds |
|-----|------|------|
| 0 (0x01) | UV | 8 bytes (2x f32) |
| 1 (0x02) | Color | 4 bytes (RGBA u8) |
| 2 (0x04) | Normal | 12 bytes (3x f32) |
| 3 (0x08) | Skinned | 8 bytes (4x u8 indices + 4x u8 weights) |

**Base vertex stride:** 12 bytes (position: 3x f32)

**Examples:**
- Format 0: 12 bytes (position only)
- Format 1: 20 bytes (position + UV)
- Format 3: 24 bytes (position + UV + color)
- Format 7: 36 bytes (position + UV + color + normal)
- Format 15: 44 bytes (all features)

**Binary File Format (.nczxmesh):**
```
Header (12 bytes):
  0x00: vertex_count (u32, LE)
  0x04: index_count (u32, LE)
  0x08: format (u8)
  0x09: padding (3 bytes)

Data:
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
| ROM (total) | 12 MB |
| WASM code | 4 MB |
| RAM (linear memory) | 4 MB |
| VRAM (GPU resources) | 4 MB |

**Asset Loading Model:**
- Assets loaded via `rom_*` FFI go directly to VRAM/audio memory (host-managed)
- Only handles (u32) live in game state
- This enables efficient rollback (only 4MB RAM snapshotted)

---

## Asset File Extensions

| Asset Type | Extension | Format |
|------------|-----------|--------|
| ROM | `.nczx` | Bitcode |
| Mesh | `.nczxmesh` | POD binary |
| Texture | `.nczxtex` | POD binary |
| Sound | `.nczxsnd` | WAV (parsed to PCM) |
| Skeleton | `.nczxskel` | POD binary |
| Animation | `.nczxanim` | POD binary |

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
- Required exports: `init`, `update`, `render`

### Console Settings Validation
- **render_mode**: Must be 0-3 if specified
- **default_resolution**: Parsed at runtime
- **target_fps**: Any positive integer

---

## Creating ROMs

Use `nether pack` to create ROM files:

```bash
nether pack
```

This reads `nether.toml` in your project and creates the ROM with all assets bundled.

### nether.toml Example

```toml
[package]
id = "my-game"
title = "My Awesome Game"
author = "YourName"
version = "1.0.0"
description = "A fun game!"

[package.tags]
tags = ["platformer", "action"]

[assets]
textures = "assets/textures"
meshes = "assets/meshes"
sounds = "assets/sounds"
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
~/.nethercore/games/{game_id}/
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
- File is not an Nethercore ZX ROM
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
| ZXRom Struct | `zx-common/src/formats/z_rom.rs` |
| ZXDataPack Struct | `zx-common/src/formats/z_data_pack.rs` |
| Mesh Format | `zx-common/src/formats/mesh.rs` |
| Skeleton Format | `zx-common/src/formats/skeleton.rs` |
| Animation Format | `zx-common/src/formats/animation.rs` |
| Build Process | `tools/nether-cli/src/pack.rs` |

---

## See Also

- [distributing-games.md](./distributing-games.md) - Complete guide for game developers
- [ffi.md](./ffi.md) - Nethercore FFI API reference
- [nethercore-zx.md](./nethercore-zx.md) - ZX-specific API documentation
