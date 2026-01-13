# Nethercore Asset Pipeline

Convert 3D models, textures, and audio into optimized Nethercore formats.

---

## Quick Start

Getting assets into a Nethercore game is 3 steps:

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
nether-export build assets.toml
```

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static PLAYER_MESH: &[u8] = include_bytes!("assets/player.nczxmesh");
static GRASS_TEX: &[u8] = include_bytes!("assets/grass.nczxtex");

fn init() {
    let player = load_zmesh(PLAYER_MESH.as_ptr() as u32, PLAYER_MESH.len() as u32);
    let grass = load_ztex(GRASS_TEX.as_ptr() as u32, GRASS_TEX.len() as u32);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Embed assets at compile time (platform-specific)
extern const unsigned char player_nczxmesh_data[];
extern const unsigned int player_nczxmesh_size;
extern const unsigned char grass_nczxtex_data[];
extern const unsigned int grass_nczxtex_size;

NCZX_EXPORT void init(void) {
    uint32_t player = load_zmesh((uint32_t)player_nczxmesh_data, player_nczxmesh_size);
    uint32_t grass = load_ztex((uint32_t)grass_nczxtex_data, grass_nczxtex_size);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const player_mesh = @embedFile("assets/player.nczxmesh");
const grass_tex = @embedFile("assets/grass.nczxtex");

export fn init() void {
    const player = load_zmesh(@intFromPtr(player_mesh.ptr), player_mesh.len);
    const grass = load_ztex(@intFromPtr(grass_tex.ptr), grass_tex.len);
}
```
{{#endtab}}

{{#endtabs}}

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
nether-export build assets.toml

# Validate manifest without building
nether-export check assets.toml

# Convert individual files
nether-export mesh player.gltf -o player.nczxmesh
nether-export texture grass.png -o grass.nczxtex
nether-export audio jump.wav -o jump.nczxsnd
```

### Output Files

Running `nether-export build assets.toml` generates binary asset files:
- `player.nczxmesh`, `enemy.nczxmesh`, `level.nczxmesh`
- `player_diffuse.nczxtex`
- `jump.nczxsnd`

---

## Output File Formats

### NetherZXMesh (.nczxmesh)

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

### NetherZTexture (.nczxtex)

Binary format for textures. POD format (no magic bytes).

**Current Header (4 bytes):**
```
Offset | Type | Description
-------|------|----------------------------------
0x00   | u16  | Width in pixels (max 65535)
0x02   | u16  | Height in pixels (max 65535)
0x04   | u8[] | Pixel data (RGBA8, width * height * 4 bytes)
```

**⚠️ Format Change (Dec 12, 2024):**
- **Old format** (before commit 3ed67ef): 8-byte header with `u32 width + u32 height`
- **Current format**: 4-byte header with `u16 width + u16 height`
- If you have old `.nczxtex` files, regenerate them with:
  ```bash
  nether-export texture <source.png> -o <output.nczxtex>
  ```
- **Symptom of old format**: "invalid dimensions" error during load

### NetherZSound (.nczxsnd)

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

The Nethercore runtime supports 16 vertex format combinations, controlled by format flags.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const FORMAT_UV: u8 = 1;      // Texture coordinates
const FORMAT_COLOR: u8 = 2;   // Per-vertex color
const FORMAT_NORMAL: u8 = 4;  // Surface normals
const FORMAT_SKINNED: u8 = 8; // Bone weights for skeletal animation
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define FORMAT_UV      1   // Texture coordinates
#define FORMAT_COLOR   2   // Per-vertex color
#define FORMAT_NORMAL  4   // Surface normals
#define FORMAT_SKINNED 8   // Bone weights for skeletal animation
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const FORMAT_UV: u8 = 1;      // Texture coordinates
const FORMAT_COLOR: u8 = 2;   // Per-vertex color
const FORMAT_NORMAL: u8 = 4;  // Surface normals
const FORMAT_SKINNED: u8 = 8; // Bone weights for skeletal animation
```
{{#endtab}}

{{#endtabs}}

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

### nether-export

The asset conversion CLI tool.

**Build from manifest:**
```bash
nether-export build assets.toml           # Build all assets
nether-export check assets.toml           # Validate only
```

**Convert individual files:**
```bash
# Meshes
nether-export mesh player.gltf -o player.nczxmesh
nether-export mesh level.obj -o level.nczxmesh --format POS_UV_NORMAL

# Textures
nether-export texture grass.png -o grass.nczxtex

# Audio
nether-export audio jump.wav -o jump.nczxsnd
```

---

## Loading Assets (FFI)

### NetherZ Format Loading (Recommended)

The simplest way to load assets is using the NetherZ format functions. These parse the POD headers host-side and upload to GPU.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// FFI declarations
extern "C" {
    fn load_zmesh(data_ptr: u32, data_len: u32) -> u32;
    fn load_ztex(data_ptr: u32, data_len: u32) -> u32;
    fn load_zsound(data_ptr: u32, data_len: u32) -> u32;
}

// Embed assets at compile time
static PLAYER_MESH: &[u8] = include_bytes!("assets/player.nczxmesh");
static GRASS_TEX: &[u8] = include_bytes!("assets/grass.nczxtex");
static JUMP_SFX: &[u8] = include_bytes!("assets/jump.nczxsnd");

fn init() {
    let player = load_zmesh(PLAYER_MESH.as_ptr() as u32, PLAYER_MESH.len() as u32);
    let grass = load_ztex(GRASS_TEX.as_ptr() as u32, GRASS_TEX.len() as u32);
    let jump = load_zsound(JUMP_SFX.as_ptr() as u32, JUMP_SFX.len() as u32);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// FFI declarations
NCZX_IMPORT uint32_t load_zmesh(uint32_t data_ptr, uint32_t data_len);
NCZX_IMPORT uint32_t load_ztex(uint32_t data_ptr, uint32_t data_len);
NCZX_IMPORT uint32_t load_zsound(uint32_t data_ptr, uint32_t data_len);

// Embed assets at compile time (platform-specific)
extern const unsigned char player_nczxmesh_data[];
extern const unsigned int player_nczxmesh_size;
extern const unsigned char grass_nczxtex_data[];
extern const unsigned int grass_nczxtex_size;
extern const unsigned char jump_nczxsnd_data[];
extern const unsigned int jump_nczxsnd_size;

NCZX_EXPORT void init(void) {
    uint32_t player = load_zmesh((uint32_t)player_nczxmesh_data, player_nczxmesh_size);
    uint32_t grass = load_ztex((uint32_t)grass_nczxtex_data, grass_nczxtex_size);
    uint32_t jump = load_zsound((uint32_t)jump_nczxsnd_data, jump_nczxsnd_size);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// FFI declarations
pub extern fn load_zmesh(data_ptr: u32, data_len: u32) u32;
pub extern fn load_ztex(data_ptr: u32, data_len: u32) u32;
pub extern fn load_zsound(data_ptr: u32, data_len: u32) u32;

// Embed assets at compile time
const player_mesh = @embedFile("assets/player.nczxmesh");
const grass_tex = @embedFile("assets/grass.nczxtex");
const jump_sfx = @embedFile("assets/jump.nczxsnd");

export fn init() void {
    const player = load_zmesh(@intFromPtr(player_mesh.ptr), player_mesh.len);
    const grass = load_ztex(@intFromPtr(grass_tex.ptr), grass_tex.len);
    const jump = load_zsound(@intFromPtr(jump_sfx.ptr), jump_sfx.len);
}
```
{{#endtab}}

{{#endtabs}}

### Raw Data Loading (Advanced)

For fine-grained control, you can bypass the NetherZ format and provide raw data directly:

**Convenience API** (f32 input, auto-packed):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
extern "C" {
    fn load_mesh(data_ptr: u32, vertex_count: u32, format: u8) -> u32;
    fn load_mesh_indexed(data_ptr: u32, vertex_count: u32, index_ptr: u32, index_count: u32, format: u8) -> u32;
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load_mesh(uint32_t data_ptr, uint32_t vertex_count, uint8_t format);
NCZX_IMPORT uint32_t load_mesh_indexed(uint32_t data_ptr, uint32_t vertex_count, uint32_t index_ptr, uint32_t index_count, uint8_t format);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh(data_ptr: u32, vertex_count: u32, format: u8) u32;
pub extern fn load_mesh_indexed(data_ptr: u32, vertex_count: u32, index_ptr: u32, index_count: u32, format: u8) u32;
```
{{#endtab}}

{{#endtabs}}

**Power User API** (pre-packed data):

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
extern "C" {
    fn load_mesh_packed(data_ptr: u32, vertex_count: u32, format: u8) -> u32;
    fn load_mesh_indexed_packed(data_ptr: u32, vertex_count: u32, index_ptr: u32, index_count: u32, format: u8) -> u32;
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load_mesh_packed(uint32_t data_ptr, uint32_t vertex_count, uint8_t format);
NCZX_IMPORT uint32_t load_mesh_indexed_packed(uint32_t data_ptr, uint32_t vertex_count, uint32_t index_ptr, uint32_t index_count, uint8_t format);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh_packed(data_ptr: u32, vertex_count: u32, format: u8) u32;
pub extern fn load_mesh_indexed_packed(data_ptr: u32, vertex_count: u32, index_ptr: u32, index_count: u32, format: u8) u32;
```
{{#endtab}}

{{#endtabs}}

---

## Constraints

Nethercore enforces these limits:

| Resource | Limit |
|----------|-------|
| ROM size | 16 MB |
| VRAM | 4 MB |
| Bones per skeleton | 256 |

All assets are packaged into the ROM at build time. There is no runtime filesystem/network asset loading — this ensures deterministic builds required for rollback netcode.

---

## Starter Assets

Don't have assets yet? Here are some ready-to-use procedural assets you can copy directly into your game.

### Procedural Textures

**Checkerboard (8x8)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const CHECKERBOARD: [u8; 256] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let gray = [0x88, 0x88, 0x88, 0xFF];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if (x + y) % 2 == 0 { white } else { gray };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static const uint8_t CHECKERBOARD[256] = {
    0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,
    0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,
    0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,
    0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,
    0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,  0x88, 0x88, 0x88, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF,
};
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const CHECKERBOARD: [256]u8 = blk: {
    var pixels: [256]u8 = undefined;
    const white = [4]u8{ 0xFF, 0xFF, 0xFF, 0xFF };
    const gray = [4]u8{ 0x88, 0x88, 0x88, 0xFF };
    var y: usize = 0;
    while (y < 8) : (y += 1) {
        var x: usize = 0;
        while (x < 8) : (x += 1) {
            const idx = (y * 8 + x) * 4;
            const color = if ((x + y) % 2 == 0) white else gray;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
    break :blk pixels;
};
```
{{#endtab}}

{{#endtabs}}

**Player Sprite (8x8)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const PLAYER_SPRITE: [u8; 256] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];
    let pattern: [[u8; 8]; 8] = [
        [0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 1, 0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0, 1, 0, 0],
    ];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if pattern[y][x] == 1 { white } else { trans };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static void generate_player_sprite(uint8_t pixels[256]) {
    const uint8_t white[4] = {0xFF, 0xFF, 0xFF, 0xFF};
    const uint8_t trans[4] = {0x00, 0x00, 0x00, 0x00};
    const uint8_t pattern[8][8] = {
        {0, 0, 1, 1, 1, 1, 0, 0},
        {0, 1, 1, 1, 1, 1, 1, 0},
        {0, 1, 1, 1, 1, 1, 1, 0},
        {0, 0, 1, 1, 1, 1, 0, 0},
        {0, 1, 1, 1, 1, 1, 1, 0},
        {1, 1, 1, 1, 1, 1, 1, 1},
        {0, 0, 1, 0, 0, 1, 0, 0},
        {0, 0, 1, 0, 0, 1, 0, 0},
    };
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int idx = (y * 8 + x) * 4;
            const uint8_t *color = pattern[y][x] == 1 ? white : trans;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const PLAYER_SPRITE: [256]u8 = blk: {
    var pixels: [256]u8 = undefined;
    const white = [4]u8{ 0xFF, 0xFF, 0xFF, 0xFF };
    const trans = [4]u8{ 0x00, 0x00, 0x00, 0x00 };
    const pattern = [8][8]u8{
        .{ 0, 0, 1, 1, 1, 1, 0, 0 },
        .{ 0, 1, 1, 1, 1, 1, 1, 0 },
        .{ 0, 1, 1, 1, 1, 1, 1, 0 },
        .{ 0, 0, 1, 1, 1, 1, 0, 0 },
        .{ 0, 1, 1, 1, 1, 1, 1, 0 },
        .{ 1, 1, 1, 1, 1, 1, 1, 1 },
        .{ 0, 0, 1, 0, 0, 1, 0, 0 },
        .{ 0, 0, 1, 0, 0, 1, 0, 0 },
    };
    var y: usize = 0;
    while (y < 8) : (y += 1) {
        var x: usize = 0;
        while (x < 8) : (x += 1) {
            const idx = (y * 8 + x) * 4;
            const color = if (pattern[y][x] == 1) white else trans;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
    break :blk pixels;
};
```
{{#endtab}}

{{#endtabs}}

**Coin/Collectible (8x8)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const COIN_SPRITE: [u8; 256] = {
    let mut pixels = [0u8; 256];
    let gold = [0xFF, 0xD7, 0x00, 0xFF];
    let shine = [0xFF, 0xFF, 0x88, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];
    let pattern: [[u8; 8]; 8] = [
        [0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 2, 2, 1, 1, 1, 0],
        [1, 2, 2, 1, 1, 1, 1, 1],
        [1, 2, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 1, 1, 1, 1, 0, 0],
    ];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = match pattern[y][x] {
                0 => trans, 1 => gold, _ => shine,
            };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static void generate_coin_sprite(uint8_t pixels[256]) {
    const uint8_t gold[4] = {0xFF, 0xD7, 0x00, 0xFF};
    const uint8_t shine[4] = {0xFF, 0xFF, 0x88, 0xFF};
    const uint8_t trans[4] = {0x00, 0x00, 0x00, 0x00};
    const uint8_t pattern[8][8] = {
        {0, 0, 1, 1, 1, 1, 0, 0},
        {0, 1, 2, 2, 1, 1, 1, 0},
        {1, 2, 2, 1, 1, 1, 1, 1},
        {1, 2, 1, 1, 1, 1, 1, 1},
        {1, 1, 1, 1, 1, 1, 1, 1},
        {1, 1, 1, 1, 1, 1, 1, 1},
        {0, 1, 1, 1, 1, 1, 1, 0},
        {0, 0, 1, 1, 1, 1, 0, 0},
    };
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int idx = (y * 8 + x) * 4;
            const uint8_t *color = (pattern[y][x] == 0) ? trans : (pattern[y][x] == 1) ? gold : shine;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const COIN_SPRITE: [256]u8 = blk: {
    var pixels: [256]u8 = undefined;
    const gold = [4]u8{ 0xFF, 0xD7, 0x00, 0xFF };
    const shine = [4]u8{ 0xFF, 0xFF, 0x88, 0xFF };
    const trans = [4]u8{ 0x00, 0x00, 0x00, 0x00 };
    const pattern = [8][8]u8{
        .{ 0, 0, 1, 1, 1, 1, 0, 0 },
        .{ 0, 1, 2, 2, 1, 1, 1, 0 },
        .{ 1, 2, 2, 1, 1, 1, 1, 1 },
        .{ 1, 2, 1, 1, 1, 1, 1, 1 },
        .{ 1, 1, 1, 1, 1, 1, 1, 1 },
        .{ 1, 1, 1, 1, 1, 1, 1, 1 },
        .{ 0, 1, 1, 1, 1, 1, 1, 0 },
        .{ 0, 0, 1, 1, 1, 1, 0, 0 },
    };
    var y: usize = 0;
    while (y < 8) : (y += 1) {
        var x: usize = 0;
        while (x < 8) : (x += 1) {
            const idx = (y * 8 + x) * 4;
            const color = if (pattern[y][x] == 0) trans else if (pattern[y][x] == 1) gold else shine;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
    break :blk pixels;
};
```
{{#endtab}}

{{#endtabs}}

### Procedural Sounds

**Beep (short hit sound)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn generate_beep() -> [i16; 2205] {
    let mut samples = [0i16; 2205]; // 0.1 sec @ 22050 Hz
    for i in 0..2205 {
        let t = i as f32 / 22050.0;
        let envelope = 1.0 - (i as f32 / 2205.0);
        let value = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <math.h>

void generate_beep(int16_t samples[2205]) {
    for (int i = 0; i < 2205; i++) {
        float t = (float)i / 22050.0f;
        float envelope = 1.0f - ((float)i / 2205.0f);
        float value = sinf(2.0f * 3.14159265f * 440.0f * t) * envelope;
        samples[i] = (int16_t)(value * 32767.0f * 0.3f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn generate_beep() [2205]i16 {
    var samples: [2205]i16 = undefined;
    for (0..2205) |i| {
        const t = @as(f32, @floatFromInt(i)) / 22050.0;
        const envelope = 1.0 - (@as(f32, @floatFromInt(i)) / 2205.0);
        const value = @sin(2.0 * std.math.pi * 440.0 * t) * envelope;
        samples[i] = @intFromFloat(value * 32767.0 * 0.3);
    }
    return samples;
}
```
{{#endtab}}

{{#endtabs}}

**Jump sound (ascending)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn generate_jump() -> [i16; 4410] {
    let mut samples = [0i16; 4410]; // 0.2 sec
    for i in 0..4410 {
        let t = i as f32 / 22050.0;
        let progress = i as f32 / 4410.0;
        let freq = 200.0 + (400.0 * progress); // 200 → 600 Hz
        let envelope = 1.0 - progress;
        let value = libm::sinf(2.0 * core::f32::consts::PI * freq * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void generate_jump(int16_t samples[4410]) {
    for (int i = 0; i < 4410; i++) {
        float t = (float)i / 22050.0f;
        float progress = (float)i / 4410.0f;
        float freq = 200.0f + (400.0f * progress); // 200 → 600 Hz
        float envelope = 1.0f - progress;
        float value = sinf(2.0f * 3.14159265f * freq * t) * envelope;
        samples[i] = (int16_t)(value * 32767.0f * 0.3f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn generate_jump() [4410]i16 {
    var samples: [4410]i16 = undefined;
    for (0..4410) |i| {
        const t = @as(f32, @floatFromInt(i)) / 22050.0;
        const progress = @as(f32, @floatFromInt(i)) / 4410.0;
        const freq = 200.0 + (400.0 * progress); // 200 → 600 Hz
        const envelope = 1.0 - progress;
        const value = @sin(2.0 * std.math.pi * freq * t) * envelope;
        samples[i] = @intFromFloat(value * 32767.0 * 0.3);
    }
    return samples;
}
```
{{#endtab}}

{{#endtabs}}

**Coin collect (sparkle)**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn generate_collect() -> [i16; 6615] {
    let mut samples = [0i16; 6615]; // 0.3 sec
    for i in 0..6615 {
        let t = i as f32 / 22050.0;
        let progress = i as f32 / 6615.0;
        // Two frequencies for shimmer effect
        let f1 = 880.0;
        let f2 = 1320.0; // Perfect fifth
        let envelope = 1.0 - progress;
        let v1 = libm::sinf(2.0 * core::f32::consts::PI * f1 * t);
        let v2 = libm::sinf(2.0 * core::f32::consts::PI * f2 * t);
        let value = (v1 + v2 * 0.5) * envelope;
        samples[i] = (value * 32767.0 * 0.2) as i16;
    }
    samples
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void generate_collect(int16_t samples[6615]) {
    for (int i = 0; i < 6615; i++) {
        float t = (float)i / 22050.0f;
        float progress = (float)i / 6615.0f;
        // Two frequencies for shimmer effect
        float f1 = 880.0f;
        float f2 = 1320.0f; // Perfect fifth
        float envelope = 1.0f - progress;
        float v1 = sinf(2.0f * 3.14159265f * f1 * t);
        float v2 = sinf(2.0f * 3.14159265f * f2 * t);
        float value = (v1 + v2 * 0.5f) * envelope;
        samples[i] = (int16_t)(value * 32767.0f * 0.2f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn generate_collect() [6615]i16 {
    var samples: [6615]i16 = undefined;
    for (0..6615) |i| {
        const t = @as(f32, @floatFromInt(i)) / 22050.0;
        const progress = @as(f32, @floatFromInt(i)) / 6615.0;
        // Two frequencies for shimmer effect
        const f1 = 880.0;
        const f2 = 1320.0; // Perfect fifth
        const envelope = 1.0 - progress;
        const v1 = @sin(2.0 * std.math.pi * f1 * t);
        const v2 = @sin(2.0 * std.math.pi * f2 * t);
        const value = (v1 + v2 * 0.5) * envelope;
        samples[i] = @intFromFloat(value * 32767.0 * 0.2);
    }
    return samples;
}
```
{{#endtab}}

{{#endtabs}}

### Using Starter Assets

Load procedural assets in your `init()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PLAYER_TEX: u32 = 0;
static mut JUMP_SFX: u32 = 0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Load texture
        PLAYER_TEX = load_texture(8, 8, PLAYER_SPRITE.as_ptr());
        texture_filter(0); // Nearest-neighbor for crisp pixels

        // Load sound
        let jump = generate_jump();
        JUMP_SFX = load_sound(jump.as_ptr(), (jump.len() * 2) as u32);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t PLAYER_TEX = 0;
static uint32_t JUMP_SFX = 0;

NCZX_EXPORT void init(void) {
    // Load texture
    uint8_t player_sprite[256];
    generate_player_sprite(player_sprite);
    PLAYER_TEX = load_texture(8, 8, (uint32_t)player_sprite);
    texture_filter(0); // Nearest-neighbor for crisp pixels

    // Load sound
    int16_t jump[4410];
    generate_jump(jump);
    JUMP_SFX = load_sound((uint32_t)jump, sizeof(jump));
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var PLAYER_TEX: u32 = 0;
var JUMP_SFX: u32 = 0;

export fn init() void {
    // Load texture
    PLAYER_TEX = load_texture(8, 8, @intFromPtr(&PLAYER_SPRITE));
    texture_filter(0); // Nearest-neighbor for crisp pixels

    // Load sound
    const jump = generate_jump();
    JUMP_SFX = load_sound(@intFromPtr(&jump), jump.len * 2);
}
```
{{#endtab}}

{{#endtabs}}

---

## nether.toml vs include_bytes!()

There are two ways to include assets in your game:

### Method 1: nether.toml + ROM Packing

Best for: Production games with many assets

```toml
# nether.toml
[game]
id = "my-game"
title = "My Game"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.sounds]]
id = "jump"
path = "assets/jump.wav"
```

Load with ROM functions:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
let player_tex = rom_texture(b"player".as_ptr(), 6);
let jump_sfx = rom_sound(b"jump".as_ptr(), 4);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t player_tex = rom_texture((uint32_t)"player", 6);
uint32_t jump_sfx = rom_sound((uint32_t)"jump", 4);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const player_tex = rom_texture(@intFromPtr("player".ptr), 6);
const jump_sfx = rom_sound(@intFromPtr("jump".ptr), 4);
```
{{#endtab}}

{{#endtabs}}

Benefits:
- Assets are pre-processed and compressed
- Single `.nczx` ROM file
- Automatic GPU format conversion

### Method 2: include_bytes!() + Procedural

Best for: Small games, prototyping, tutorials

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Compile-time embedding
static TEXTURE_DATA: &[u8] = include_bytes!("../assets/player.nczxtex");

// Or generate at runtime
const PIXELS: [u8; 256] = generate_pixels();
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Compile-time embedding (platform-specific)
extern const unsigned char player_nczxtex_data[];
extern const unsigned int player_nczxtex_size;

// Or generate at runtime
uint8_t pixels[256];
generate_pixels(pixels);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Compile-time embedding
const texture_data = @embedFile("../assets/player.nczxtex");

// Or generate at runtime
const pixels: [256]u8 = generate_pixels();
```
{{#endtab}}

{{#endtabs}}

Benefits:
- Simple, no build step
- Good for procedural content
- Self-contained WASM file

### Which Should I Use?

| Scenario | Recommendation |
|----------|----------------|
| Learning/prototyping | include_bytes!() or procedural |
| Simple arcade games | Either works |
| Complex games with many assets | nether.toml + ROM |
| Games with large textures | nether.toml (compression) |

---

## Planned Features

The following features are planned but not yet implemented:

- **Font conversion** - TTF/OTF to bitmap font atlas (.nczxfont)
- **Watch mode** - `nether-export build --watch` for auto-rebuild on changes
- **Rust code generation** - Auto-generated asset loading module
