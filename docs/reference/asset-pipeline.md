# Emberware Asset Pipeline

Convert 3D models, textures, and audio into optimized Emberware formats.

---

## Quick Start

Getting assets into an Emberware game is 3 steps:

**1. Export from your 3D tool** (Blender, Maya, etc.) as glTF, GLB, or OBJ

**2. Create `assets.toml`:**
```toml
[output]
dir = "assets/"

[meshes]
player = "models/player.gltf"
enemy = "models/enemy.glb"

[textures]
grass = "textures/grass.png"

[sounds]
jump = "audio/jump.wav"
```

**3. Build and use:**
```bash
ember-export build assets.toml
```

```rust
static PLAYER_MESH: &[u8] = include_bytes!("assets/player.ewzmesh");
static GRASS_TEX: &[u8] = include_bytes!("assets/grass.ewztex");

fn init() {
    let player = load_zmesh(PLAYER_MESH.as_ptr() as u32, PLAYER_MESH.len() as u32);
    let grass = load_ztex(GRASS_TEX.as_ptr() as u32, GRASS_TEX.len() as u32);
}
```

One manifest, one command, simple FFI calls.

---

## Supported Input Formats

### 3D Models

| Format | Extension | Status |
|--------|-----------|--------|
| **glTF 2.0** | `.gltf`, `.glb` | Implemented |
| **OBJ** | `.obj` | Implemented |

**Recommendation:** Use glTF for new projects. It's the "JPEG of 3D" - efficient, well-documented, and supported everywhere.

### Textures

| Format | Status |
|--------|--------|
| **PNG** | Implemented |
| **JPG** | Implemented |

### Audio

| Format | Status |
|--------|--------|
| **WAV** | Implemented |

### Fonts

| Format | Status |
|--------|--------|
| **TTF** | Planned |

---

## Manifest-Based Asset Pipeline

Define all your game assets in a single `assets.toml` file, then build everything with one command.

### assets.toml Reference

```toml
# Output configuration
[output]
dir = "assets/"                  # Output directory for converted files
# rust = "src/assets.rs"         # Planned: Generated Rust module

# 3D Models
[meshes]
player = "models/player.gltf"                           # Simple: just the path
enemy = "models/enemy.glb"
level = { path = "models/level.obj", format = "POS_UV_NORMAL" }  # With options

# Textures
[textures]
player_diffuse = "textures/player.png"

# Audio
[sounds]
jump = "audio/jump.wav"

# Fonts (planned)
# [fonts]
# ui = { path = "fonts/roboto.ttf", size = 16 }
```

### Build Commands

```bash
# Build all assets from manifest
ember-export build assets.toml

# Validate manifest without building
ember-export check assets.toml

# Convert individual files
ember-export mesh player.gltf -o player.ewzmesh
ember-export texture grass.png -o grass.ewztex
ember-export audio jump.wav -o jump.ewzsnd
```

### Output Files

Running `ember-export build assets.toml` generates binary asset files:
- `player.ewzmesh`, `enemy.ewzmesh`, `level.ewzmesh`
- `player_diffuse.ewztex`
- `jump.ewzsnd`

---

## Output File Formats

### EmberZMesh (.ewzmesh)

Binary format for 3D meshes with GPU-optimized packed vertex data. POD format (no magic bytes).

**Header (12 bytes):**
```
Offset | Type | Description
-------|------|----------------------------------
0x00   | u32  | Vertex count
0x04   | u32  | Index count
0x08   | u8   | Vertex format flags (0-15)
0x09   | u8   | Reserved (padding)
0x0A   | u16  | Reserved (padding)
0x0C   | data | Vertex data (vertex_count * stride bytes)
0x??   | u16[]| Index data (index_count * 2 bytes)
```

Stride is calculated from the format flags at runtime.

### EmberZTexture (.ewztex)

Binary format for textures. POD format (no magic bytes).

**Header (8 bytes):**
```
Offset | Type | Description
-------|------|----------------------------------
0x00   | u32  | Width in pixels
0x04   | u32  | Height in pixels
0x08   | u8[] | Pixel data (RGBA8, width * height * 4 bytes)
```

### EmberZSound (.ewzsnd)

Binary format for audio. POD format (no magic bytes).

**Header (4 bytes):**
```
Offset | Type  | Description
-------|-------|----------------------------------
0x00   | u32   | Sample count
0x04   | i16[] | PCM samples (22050Hz mono)
```

---

## Vertex Formats

The Emberware runtime supports 16 vertex format combinations, controlled by format flags.

```rust
FORMAT_UV      = 1   // Texture coordinates
FORMAT_COLOR   = 2   // Per-vertex color
FORMAT_NORMAL  = 4   // Surface normals
FORMAT_SKINNED = 8   // Bone weights for skeletal animation
```

### All 16 Formats

| Format | Name | Packed Stride |
|--------|------|---------------|
| 0 | POS | 8 bytes |
| 1 | POS_UV | 12 bytes |
| 2 | POS_COLOR | 12 bytes |
| 3 | POS_UV_COLOR | 16 bytes |
| 4 | POS_NORMAL | 12 bytes |
| 5 | POS_UV_NORMAL | 16 bytes |
| 6 | POS_COLOR_NORMAL | 16 bytes |
| 7 | POS_UV_COLOR_NORMAL | 20 bytes |
| 8 | POS_SKINNED | 16 bytes |
| 9 | POS_UV_SKINNED | 20 bytes |
| 10 | POS_COLOR_SKINNED | 20 bytes |
| 11 | POS_UV_COLOR_SKINNED | 24 bytes |
| 12 | POS_NORMAL_SKINNED | 20 bytes |
| 13 | POS_UV_NORMAL_SKINNED | 24 bytes |
| 14 | POS_COLOR_NORMAL_SKINNED | 24 bytes |
| 15 | POS_UV_COLOR_NORMAL_SKINNED | 28 bytes |

**Common formats:**
- **Format 5 (POS_UV_NORMAL)**: Most common for textured, lit meshes
- **Format 13 (POS_UV_NORMAL_SKINNED)**: Animated characters

---

## Packed Vertex Data

The runtime automatically packs vertex data using GPU-optimized formats for smaller memory footprint and faster uploads.

### Attribute Encoding

| Attribute | Packed Format | Size | Notes |
|-----------|--------------|------|-------|
| Position | Float16x4 | 8 bytes | x, y, z, w=1.0 |
| UV | Unorm16x2 | 4 bytes | 65536 values in [0,1], better precision than f16 |
| Color | Unorm8x4 | 4 bytes | RGBA, alpha=255 if not provided |
| Normal | Octahedral u32 | 4 bytes | ~0.02° angular precision |
| Bone Indices | Uint8x4 | 4 bytes | Up to 256 bones |
| Bone Weights | Unorm8x4 | 4 bytes | Normalized to [0,255] |

### Octahedral Normal Encoding

Normals use octahedral encoding for uniform angular precision with 66% size reduction:

```
Standard normal: 3 floats × 4 bytes = 12 bytes
Octahedral:      1 u32              =  4 bytes
```

**How it works:**
1. Project 3D unit vector onto octahedron surface
2. Unfold octahedron to 2D square [-1, 1]²
3. Pack as 2× snorm16 into single u32

**Precision:** ~0.02° worst-case angular error - uniform across the entire sphere.

The vertex shader decodes the normal automatically.

### Memory Savings

| Format | Unpacked | Packed | Savings |
|--------|----------|--------|---------|
| POS_UV_NORMAL | 32 bytes | 16 bytes | 50% |
| POS_UV_NORMAL_SKINNED | 52 bytes | 24 bytes | 54% |
| Full format (15) | 64 bytes | 28 bytes | 56% |

---

## Skeletal Animation

### Vertex Skinning Data

Each skinned vertex stores:
- **Bone Indices**: Uint8x4 (4 bytes) - which bones affect this vertex (0-255)
- **Bone Weights**: Unorm8x4 (4 bytes) - influence weights, normalized

### Bone Matrices

Bone transforms use 3×4 affine matrices (not 4×4):

```
set_bones(matrices_ptr, count)
```

**3×4 Matrix Layout (column-major, 12 floats per bone):**
```
[m00, m10, m20]  ← column 0 (X basis)
[m01, m11, m21]  ← column 1 (Y basis)
[m02, m12, m22]  ← column 2 (Z basis)
[m03, m13, m23]  ← column 3 (translation)
```

The bottom row `[0, 0, 0, 1]` is implicit (affine transform).

**Limits:**
- Maximum 256 bones per skeleton
- 48 bytes per bone (vs 64 bytes for 4×4) - 25% memory savings

---

## Tool Reference

### ember-export

The asset conversion CLI tool.

**Build from manifest:**
```bash
ember-export build assets.toml           # Build all assets
ember-export check assets.toml           # Validate only
```

**Convert individual files:**
```bash
# Meshes
ember-export mesh player.gltf -o player.ewzmesh
ember-export mesh level.obj -o level.ewzmesh --format POS_UV_NORMAL

# Textures
ember-export texture grass.png -o grass.ewztex

# Audio
ember-export audio jump.wav -o jump.ewzsnd
```

---

## Loading Assets (FFI)

### EmberZ Format Loading (Recommended)

The simplest way to load assets is using the EmberZ format functions. These parse the POD headers host-side and upload to GPU.

```rust
// FFI declarations
extern "C" {
    fn load_zmesh(data_ptr: u32, data_len: u32) -> u32;
    fn load_ztex(data_ptr: u32, data_len: u32) -> u32;
    fn load_zsound(data_ptr: u32, data_len: u32) -> u32;
}

// Embed assets at compile time
static PLAYER_MESH: &[u8] = include_bytes!("assets/player.ewzmesh");
static GRASS_TEX: &[u8] = include_bytes!("assets/grass.ewztex");
static JUMP_SFX: &[u8] = include_bytes!("assets/jump.ewzsnd");

fn init() {
    let player = load_zmesh(PLAYER_MESH.as_ptr() as u32, PLAYER_MESH.len() as u32);
    let grass = load_ztex(GRASS_TEX.as_ptr() as u32, GRASS_TEX.len() as u32);
    let jump = load_zsound(JUMP_SFX.as_ptr() as u32, JUMP_SFX.len() as u32);
}
```

### Raw Data Loading (Advanced)

For fine-grained control, you can bypass the EmberZ format and provide raw data directly:

**Convenience API** (f32 input, auto-packed):
```rust
load_mesh(data_ptr, vertex_count, format) -> u32
load_mesh_indexed(data_ptr, vertex_count, index_ptr, index_count, format) -> u32
```

**Power User API** (pre-packed data):
```rust
load_mesh_packed(data_ptr, vertex_count, format) -> u32
load_mesh_indexed_packed(data_ptr, vertex_count, index_ptr, index_count, format) -> u32
```

---

## Constraints

Emberware enforces these limits:

| Resource | Limit |
|----------|-------|
| ROM size | 12 MB |
| VRAM | 4 MB |
| Bones per skeleton | 256 |

All assets are embedded in the WASM binary at compile time. There is no runtime asset loading - this ensures deterministic builds required for rollback netcode.

---

## Planned Features

The following features are planned but not yet implemented:

- **Font conversion** - TTF/OTF to bitmap font atlas (.ewzfont)
- **Watch mode** - `ember-export build --watch` for auto-rebuild on changes
- **Rust code generation** - Auto-generated asset loading module
