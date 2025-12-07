# Emberware Asset Pipeline Design

**Status:** Research & Design Document
**Last Updated:** 2025-12-06
**Author:** Research Task (TASKS.md)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Research Findings](#research-findings)
4. [File Format Specifications](#file-format-specifications)
5. [Tool Architecture](#tool-architecture)
6. [Integration Guide](#integration-guide)
7. [Performance Considerations](#performance-considerations)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Answers to Key Questions](#answers-to-key-questions)

---

## Executive Summary

This document presents a comprehensive design for Emberware's asset pipeline, addressing the challenge of converting industry-standard 3D models, textures, fonts, and audio into Emberware's native formats. The design prioritizes **compile-time embedding**, **deterministic builds**, and **rollback netcode compatibility** while maintaining developer productivity.

**Key Recommendations:**

1. **Build-time conversion** via Cargo build scripts (not runtime loading)
2. **Custom binary formats** optimized for embedded use (not glTF-as-is)
3. **Rust code generation** for include_bytes!() compatibility
4. **CLI-first tooling** (`ember-export`) with future Blender plugin
5. **No asset streaming** — all assets embedded in WASM at compile-time

**Target Workflow:**
```
Artist creates 3D model in Blender
  ↓
Export as glTF
  ↓
ember-export mesh input.gltf → vertices.rs
  ↓
Game includes! generated Rust code
  ↓
Cargo builds WASM with embedded assets
```

---

## Current State Analysis

### How Assets Work Today

**Comprehensive analysis from codebase exploration:**

| Asset Type | Format | Loading Method | Examples |
|---|---|---|---|
| **Meshes** | Raw vertex data (f32 arrays) | `load_mesh()`, `load_mesh_indexed()` | Cube, skinned-mesh |
| **Textures** | Raw RGBA8 pixels | `load_texture(width, height, pixels_ptr)` | Textured-quad |
| **Fonts** | Bitmap atlas + metadata | `load_font()`, `load_font_ex()` | Hello-world |
| **Audio** | Raw 16-bit PCM @ 22.05kHz | `load_sound(data_ptr, byte_len)` | (No examples yet!) |

**Key Characteristics:**

1. **All assets embedded at compile-time:**
   ```rust
   static SPRITE_PNG: &[u8] = include_bytes!("assets/sprite.png");
   static CUBE_VERTS: [f32; 192] = [ /* hardcoded vertex data */ ];
   ```

2. **No file-based loading:** WASM has no filesystem; all data is in linear memory

3. **Manual conversion required:**
   - PNG → raw RGBA (developers use external tools or write decoders)
   - OBJ/glTF → vertex arrays (developers write parsers or copy-paste from Blender)
   - WAV → raw PCM (use `ffmpeg -ar 22050 -ac 1 -f s16le`)

4. **Vertex format flags:**
   ```rust
   const FORMAT_UV: u32 = 1;
   const FORMAT_COLOR: u32 = 2;
   const FORMAT_NORMAL: u32 = 4;
   const FORMAT_SKINNED: u32 = 8;
   // Combine: FORMAT_UV | FORMAT_NORMAL = 5
   ```

5. **No asset management:**
   - No handles, UUIDs, or metadata
   - No compression or streaming
   - No hot reload (yet — task exists)

### Current Developer Pain Points

**From examining examples (cube, platformer, textured-quad):**

1. **Tedious mesh creation:**
   ```rust
   // 188 lines just to define a cube!
   static CUBE_VERTICES: [f32; 24 * 8] = [
       -1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0,  // Front face, vertex 0
       // ... 23 more vertices
   ];
   ```

2. **No bone weight painting:** Skinning data must be hand-calculated or scripted

3. **Texture conversion:** Must decode PNG to raw RGBA at compile-time or runtime

4. **Audio conversion:** Manual ffmpeg commands, no automation

5. **No iteration:** Change a mesh → regenerate arrays → rebuild WASM

### What Works Well

1. **Deterministic builds:** Same source → same WASM (critical for rollback netcode)
2. **Fast loading:** Assets in WASM linear memory, no I/O
3. **Simple deployment:** Single .wasm file, no external assets
4. **VRAM/ROM limits enforced:** 4 MB VRAM, 12 MB ROM hard limits

---

## Research Findings

### Game Engine Asset Pipeline Best Practices

**Sources:**
- [Asset Pipelines (CMU Graphics)](https://graphics.cs.cmu.edu/courses/15-466-f19/notes/asset-pipelines.html)
- [Unreal Engine Asset Pipeline](https://dev.epicgames.com/community/learning/courses/qEl/unreal-engine-technical-guide-to-linear-content-creation-pipeline-development/ryax/unreal-engine-asset-pipeline)
- [Asset Pipeline Optimization (Medium)](https://medium.com/@lemapp09/beginning-game-development-asset-pipeline-optimization-96495a2a795e)
- [Modern Asset Pipeline Introduction (Game Developer)](https://www.gamedeveloper.com/production/a-modern-asset-pipeline-introduction)
- [Amethyst distill (Rust Asset Pipeline)](https://github.com/amethyst/distill)

**Key Principles:**

1. **Start from the ends:** Design from both authoring tools (Blender) and runtime code (WASM)

2. **Make on-disc format mirror in-memory format:**
   - Faster loading (no conversion)
   - Simpler code

3. **Automate repetitive tasks:**
   > "For every repetitive task there should be a script automating it"

4. **Early testing:**
   > "Try the whole thing within the game engine as soon as possible"

5. **Modern features:**
   - Dependency tracking between assets
   - Import & build caching
   - Cross-device hot reloading
   - Packing for shippable builds
   - **Pure functional builds:** All inputs known, enables parallelization

**Emberware Applicability:**

✅ **Applicable:**
- Automation via build scripts
- Early testing (immediate feedback loop)
- On-disc format matching runtime format
- Pure builds (Cargo build system)

❌ **Not Applicable:**
- Runtime asset loading (WASM embeds assets)
- Streaming (12 MB ROM limit, embed everything)
- Cross-device hot reload (desktop-only for now, web player is future work)

### glTF Format Analysis

**Sources:**
- [glTF 2.0 Specification (Khronos)](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
- [glTF Wikipedia](https://en.wikipedia.org/wiki/GlTF)
- [glTF Files Explained (Jinolo)](https://jinolo.com/blog/gltf-files-explained/)
- [glTF Issues (DEV Community)](https://dev.to/drbearhands/the-gltf-file-format-for-3d-models-has-some-issues-46jo)

**Advantages:**

✅ **Performance & Efficiency:**
- Optimized for the web, lightweight
- Fast loading (data structures mirror GPU APIs)
- Minimizes runtime processing

✅ **Feature Completeness:**
- Entire scenes (nodes, transforms, meshes, materials, cameras, animations)
- Not restricted to single objects
- Runtime-independent (pure asset format)

✅ **Extensibility:**
- Fully extensible (compression, vendor extensions)
- ISO/IEC 12113:2022 International Standard

✅ **Industry Adoption:**
- Microsoft using glTF 2.0 across product line
- Supported by Blender, Unity, Unreal, Three.js

**Disadvantages:**

❌ **Not a streaming format:**
- Binary data is streamable, but no progressive loading constructs

❌ **Adoption gaps:**
- Not as universal as OBJ or FBX (yet)

❌ **Implementation complexity:**
- Indirection (bufferViews, accessors) makes extracting data "strenuous"
- Defaults missing from spec (must look up WebGL functions)

❌ **Technical issues:**
- Negative scale as mirror function (breaks expected behavior)

**Emberware Decision:**

**Use glTF as input, but NOT as runtime format.**

**Rationale:**
- glTF's indirection (bufferViews, accessors, buffers) is overkill for embedded assets
- We need raw vertex arrays, not JSON + binary blobs
- Simpler to convert glTF → flat vertex arrays at build-time
- Matches current `load_mesh()` API (already takes `*const f32` vertex arrays)

### WASM Asset Embedding

**Sources:**
- [WASM Downloading Files vs include_bytes! (Medium)](https://emilio-moretti.medium.com/rust-wasm-downloading-files-in-runtime-instead-of-include-bytes-f8c29a958e20)
- [Run Rust Games in Browser (Hands-On Rust)](https://hands-on-rust.com/2021/11/06/run-your-rust-games-in-a-browser-hands-on-rust-bonus-content/)
- [include_bytes! with wasm_bindgen (GitHub Issue)](https://github.com/rustwasm/wasm-bindgen/issues/3230)
- [Roguelike Tutorial - Web Build](https://bfnightly.bracketproductions.com/rustbook/webbuild.html)
- [wasset Crate](https://docs.rs/wasset/latest/wasset/)

**Key Findings:**

**`include_bytes!` for WASM:**
- ✅ Works perfectly in WASM
- ✅ Embeds files in WASM module
- ✅ No CDN, no browser caching (trade-off)
- ⚠️ Makes binaries larger (12 MB ROM limit)
- ⚠️ Longer compile times

**Alternative: Runtime Loading:**
- ✅ Leverages CDN and browser caching
- ❌ WASM doesn't have native filesystem
- ❌ Requires web-based asset loading system (HTTP fetch)
- ❌ Not deterministic (network failures)
- ❌ Incompatible with rollback netcode (external state)

**Game Engine Practices:**

**bracket-lib:**
```rust
embedded_resource!(TILESET, "../assets/tileset.png");
// Thin wrapper over include_bytes!
```

**Bevy:**
```rust
embedded_asset!(app, "assets/player.png");
```

**wasset (Advanced):**
- Embeds assets in WASM custom data section
- For WASM plugins with external assets

**Emberware Decision:**

**Stick with `include_bytes!()` — it's the right choice.**

**Rationale:**
1. Deterministic builds (same source → same WASM)
2. Compatible with rollback netcode (no external state)
3. Fast loading (assets in linear memory)
4. Simpler: no HTTP fetch, no CDN, no async loading
5. 12 MB ROM limit is plenty for retro aesthetic
6. Already used in all examples

---

## File Format Specifications

### Philosophy

**Formats designed for:**
1. **Compile-time embedding:** Generate Rust code, not binary files
2. **Minimal runtime processing:** Pre-converted to native format
3. **Inspect-ability:** Human-readable where possible
4. **Future-proof:** Versioned, extensible

### Format 1: EmberMesh (.embermesh)

**Purpose:** Convert glTF/OBJ → binary mesh data

**Output:** Binary file (language-agnostic)

**Binary Format:**
```
Offset | Type   | Description
-------|--------|----------------------------------
0x00   | char[4]| Magic: "EMSH"
0x04   | u32    | Format version (1)
0x08   | u32    | Vertex count
0x0C   | u32    | Index count
0x10   | u32    | Vertex format flags (0-15)
0x14   | u32    | Reserved (padding)
0x18   | f32[]  | Vertex data (vertex_count * stride)
0x??   | u16[]  | Index data (index_count * 2 bytes)
```

**Example Binary Layout (Cube Mesh):**
```
[EMSH] [0x00000001] [0x00000018] [0x00000024] [0x00000005] [0x00000000]
[vertex data: 24 verts × 32 bytes = 768 bytes]
[index data: 36 indices × 2 bytes = 72 bytes]
```

**Usage in any language:**

**Rust:**
```rust
static CUBE_MESH: &[u8] = include_bytes!("assets/cube.embermesh");

fn init() {
    let header = parse_mesh_header(CUBE_MESH);
    let mesh = load_mesh_indexed(
        header.vertices_ptr(),
        header.vertex_count,
        header.indices_ptr(),
        header.index_count,
        header.format,
    );
}

fn parse_mesh_header(data: &[u8]) -> MeshHeader {
    // Simple parsing: read u32s from offsets 0x08, 0x0C, 0x10
    MeshHeader {
        vertex_count: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        index_count: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        format: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
    }
}
```

**C/C++:**
```c
// Embedded at compile-time
extern const uint8_t CUBE_MESH[];
extern const size_t CUBE_MESH_len;

void init() {
    uint32_t* header = (uint32_t*)(CUBE_MESH + 8);
    uint32_t vertex_count = header[0];
    uint32_t index_count = header[1];
    uint32_t format = header[2];

    float* vertices = (float*)(CUBE_MESH + 24);
    uint16_t* indices = (uint16_t*)(CUBE_MESH + 24 + vertex_count * stride);

    load_mesh_indexed(vertices, vertex_count, indices, index_count, format);
}
```

**AssemblyScript/Zig/Other WASM languages:**
Same binary parsing approach - just read bytes at known offsets.

**Why binary, not source code?**
1. ✅ **Language-agnostic:** Works with any WASM-targeting language
2. ✅ **Smaller files:** Binary is ~70% smaller than generated source code
3. ✅ **Faster compilation:** No large const arrays to compile
4. ✅ **Simple parsing:** Fixed offsets, read headers, done
5. ✅ **Works with `include_bytes!()`** in Rust, `--embed-file` in Emscripten, etc.

**Metadata (Optional Sidecar File):**
```json
// cube.embermesh.json (optional, for debugging/tools)
{
  "source": "cube.gltf",
  "exported": "2025-12-06T10:23:45Z",
  "bounds": { "min": [-1, -1, -1], "max": [1, 1, 1] },
  "format": "POS_UV_NORMAL"
}
```

**Skinned Meshes:**
Same binary format, just with `FORMAT_SKINNED` flag set:
```
format = 13  // POS_UV_NORMAL_SKINNED
stride = 44  // pos(12) + uv(8) + normal(12) + bone_indices(4) + bone_weights(16)
```

Bone weights painted in Blender are automatically exported in vertex data.

**File Naming Convention:**
```
assets/
├── cube.gltf          # Source (Blender export)
├── cube.embermesh     # Generated binary by ember-export
└── player.gltf        # Source
    └── player.embermesh  # Generated binary
```

---

### Format 2: EmberTexture (.embertex)

**Purpose:** Convert PNG/JPG → binary texture data

**Output:** Binary file with header + pixel data

**Binary Format:**
```
Offset | Type   | Description
-------|--------|----------------------------------
0x00   | char[4]| Magic: "EMTX"
0x04   | u32    | Format version (1)
0x08   | u32    | Width in pixels
0x0C   | u32    | Height in pixels
0x10   | u32    | Pixel format (0=RGBA8, 1=Palette256, future: DXT, etc.)
0x14   | u32    | Reserved (padding)
0x18   | u8[]   | Pixel data (width * height * 4 for RGBA8)
```

**Usage in any language:**

**Rust:**
```rust
static GRASS_TEX: &[u8] = include_bytes!("assets/grass.embertex");

fn init() {
    let width = u32::from_le_bytes([GRASS_TEX[8], GRASS_TEX[9], GRASS_TEX[10], GRASS_TEX[11]]);
    let height = u32::from_le_bytes([GRASS_TEX[12], GRASS_TEX[13], GRASS_TEX[14], GRASS_TEX[15]]);
    let pixels = &GRASS_TEX[24..]; // Skip header

    let tex = load_texture(width, height, pixels.as_ptr());
}
```

**C:**
```c
extern const uint8_t GRASS_TEX[];

void init() {
    uint32_t* header = (uint32_t*)(GRASS_TEX + 8);
    uint32_t width = header[0];
    uint32_t height = header[1];
    const uint8_t* pixels = GRASS_TEX + 24;

    load_texture(width, height, pixels);
}
```

**Advanced: Palette Mode (Future Optimization)**

**Binary format with pixel_format=1:**
```
Offset | Type   | Description
-------|--------|----------------------------------
0x00   | char[4]| Magic: "EMTX"
0x04   | u32    | Format version (1)
0x08   | u32    | Width in pixels
0x0C   | u32    | Height in pixels
0x10   | u32    | Pixel format (1 = Palette256)
0x14   | u32    | Reserved
0x18   | u8[1024] | Palette (256 RGBA8 colors = 1024 bytes)
0x418  | u8[]   | Pixel indices (width * height bytes)
```

**Benefits:** 75% size reduction (1 byte/pixel vs 4 bytes/pixel)

**FFI Addition (Future):**
```c
load_texture_palette(width, height, indices_ptr, palette_ptr);
```

---

### Format 3: EmberFont (.emberfont)

**Purpose:** Convert TTF/OTF → bitmap font atlas

**Output:** Binary file with atlas texture + glyph metadata

**Binary Format:**
```
Offset | Type    | Description
-------|---------|----------------------------------
0x00   | char[4] | Magic: "EMFT"
0x04   | u32     | Format version (1)
0x08   | u32     | Atlas texture width
0x0C   | u32     | Atlas texture height
0x10   | f32     | Line height
0x14   | f32     | Baseline offset
0x18   | u32     | Glyph count
0x1C   | u32     | Reserved
0x20   | Glyph[] | Glyph metadata array (16 bytes each)
0x??   | u8[]    | Atlas texture data (RGBA8)

Glyph structure (16 bytes):
  u32 codepoint
  u16 x, y           // Position in atlas
  u16 width, height  // Glyph size
  i16 offset_x, offset_y
  f32 advance
```

**Usage in C:**
```c
static const uint8_t FONT_DATA[] = {
    #include "roboto.emberfont.h"  // Hex array
};

void init() {
    uint32_t* header = (uint32_t*)FONT_DATA;
    uint32_t atlas_width = header[2];
    uint32_t atlas_height = header[3];
    uint32_t glyph_count = header[6];

    // Glyph metadata starts at offset 0x20
    const uint8_t* glyphs_ptr = FONT_DATA + 0x20;

    // Atlas texture starts after glyphs
    const uint8_t* atlas_pixels = glyphs_ptr + (glyph_count * 16);

    // Load texture and create font
    uint32_t font_tex = load_texture(atlas_width, atlas_height, atlas_pixels);
    // Use glyphs_ptr for glyph lookup
}
```

**SDF Fonts (Future):**
Set a flag in header to indicate SDF encoding for scalable rendering.

---

### Format 4: EmberSound (.embersnd)

**Purpose:** Convert WAV/MP3/OGG → raw PCM @ 22.05kHz

**Output:** Binary file with PCM data

**Binary Format:**
```
Offset | Type   | Description
-------|--------|----------------------------------
0x00   | char[4]| Magic: "EMSN"
0x04   | u32    | Format version (1)
0x08   | u32    | Sample rate (22050)
0x0C   | u32    | Sample count
0x10   | u32    | Audio format (0=PCM16, 1=ADPCM, etc.)
0x14   | u32    | Reserved
0x18   | i16[]  | PCM samples (sample_count * 2 bytes)
```

**Usage in C:**
```c
extern const uint8_t JUMP_SFX[];

void init() {
    uint32_t* header = (uint32_t*)(JUMP_SFX + 8);
    uint32_t sample_rate = header[0];  // Should be 22050
    uint32_t sample_count = header[1];

    const int16_t* samples = (int16_t*)(JUMP_SFX + 24);

    load_sound(samples, sample_count * 2);  // × 2 for bytes
}
```

**Rust:**
```rust
static JUMP_SFX: &[u8] = include_bytes!("assets/jump.embersnd");

fn init() {
    let sample_count = u32::from_le_bytes([JUMP_SFX[12], JUMP_SFX[13], JUMP_SFX[14], JUMP_SFX[15]]);
    let samples_ptr = &JUMP_SFX[24] as *const u8 as *const i16;

    load_sound(samples_ptr, sample_count * 2);
}
```

**Compression (Future):**
Use ADPCM (format=1) for 4:1 compression.

---

## Tool Architecture

### CLI Tool: `ember-export`

**Location:** `tools/ember-export/` (new Cargo workspace member)

**Subcommands:**

```bash
# Mesh conversion
ember-export mesh <input.gltf|.obj|.fbx> [--output mesh.embermesh.rs] [--format FORMAT]

# Texture conversion
ember-export texture <input.png|.jpg> [--output tex.embertex.rs] [--palette COLORS] [--dither METHOD]

# Font conversion
ember-export font <input.ttf|.otf> [--output font.emberfont.rs] [--size SIZE] [--charset CHARSET]

# Audio conversion
ember-export audio <input.wav|.mp3|.ogg> [--output sound.embersnd.rs]

# Batch conversion (future)
ember-export batch <manifest.toml>
```

**Architecture:**

```rust
// tools/ember-export/src/main.rs
mod mesh;
mod texture;
mod font;
mod audio;

enum Command {
    Mesh(MeshArgs),
    Texture(TextureArgs),
    Font(FontArgs),
    Audio(AudioArgs),
}

fn main() {
    let cmd = parse_args();
    match cmd {
        Command::Mesh(args) => mesh::convert(args),
        // ...
    }
}
```

**Dependencies:**

```toml
[dependencies]
gltf = "1.4"          # glTF parsing
obj = "0.10"          # OBJ parsing
image = "0.25"        # PNG/JPG decoding
ttf-parser = "0.25"   # TTF/OTF parsing
symphonia = "0.5"     # Audio decoding (WAV/MP3/OGG)
clap = "4.0"          # CLI argument parsing
```

### Mesh Converter

**Input:** glTF 2.0 (.gltf or .glb)

**Process:**
1. Parse glTF using `gltf` crate
2. Extract vertex positions, normals, UVs, colors
3. Extract bone indices/weights (if skinned)
4. Triangulate (convert quads/ngons → triangles)
5. Generate vertex indices (if not indexed)
6. Determine vertex format flags (UV? NORMAL? SKINNED?)
7. Calculate stride
8. Write Rust source code

**Pseudocode:**

```rust
fn convert_mesh(input: &Path) -> Result<String> {
    let gltf = gltf::Gltf::open(input)?;
    let mesh = gltf.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive")?;

    // Extract attributes
    let positions = primitive.get_positions()?;
    let normals = primitive.get_normals();
    let uvs = primitive.get_tex_coords(0);
    let colors = primitive.get_colors(0);
    let (bone_indices, bone_weights) = primitive.get_skin_data();

    // Determine format
    let mut format = 0u32;
    if uvs.is_some() { format |= FORMAT_UV; }
    if colors.is_some() { format |= FORMAT_COLOR; }
    if normals.is_some() { format |= FORMAT_NORMAL; }
    if bone_indices.is_some() { format |= FORMAT_SKINNED; }

    // Interleave vertex data
    let vertices = interleave_vertices(positions, uvs, colors, normals, bone_indices, bone_weights);

    // Extract indices
    let indices = primitive.indices().unwrap_or_else(|| generate_indices(positions.len()));

    // Generate Rust code
    let code = generate_rust_code(&vertices, &indices, format);

    Ok(code)
}
```

**Vertex Interleaving:**

```rust
fn interleave_vertices(
    positions: &[[f32; 3]],
    uvs: Option<&[[f32; 2]]>,
    colors: Option<&[[f32; 3]]>,
    normals: Option<&[[f32; 3]]>,
    bone_indices: Option<&[[u8; 4]]>,
    bone_weights: Option<&[[f32; 4]]>,
) -> Vec<f32> {
    let mut data = Vec::new();

    for i in 0..positions.len() {
        // Position (always present)
        data.extend_from_slice(&positions[i]);

        // UV (if present)
        if let Some(uvs) = uvs {
            data.extend_from_slice(&uvs[i]);
        }

        // Color (if present)
        if let Some(colors) = colors {
            data.extend_from_slice(&colors[i]);
        }

        // Normal (if present)
        if let Some(normals) = normals {
            data.extend_from_slice(&normals[i]);
        }

        // Skinning data (if present)
        if let Some(bone_indices) = bone_indices {
            // Pack 4 u8 indices into a single f32
            let packed = u32::from_le_bytes(bone_indices[i]);
            data.push(f32::from_bits(packed));

            // Bone weights (4 floats)
            data.extend_from_slice(&bone_weights.unwrap()[i]);
        }
    }

    data
}
```

**Binary Generation:**

```rust
fn generate_binary(vertices: &[f32], indices: &[u16], format: u32) -> Vec<u8> {
    let mut output = Vec::new();

    // Header
    output.extend_from_slice(b"EMSH");  // Magic
    output.extend_from_slice(&1u32.to_le_bytes());  // Version
    output.extend_from_slice(&(vertices.len() as u32 / stride_in_floats(format)).to_le_bytes());  // Vertex count
    output.extend_from_slice(&(indices.len() as u32).to_le_bytes());  // Index count
    output.extend_from_slice(&format.to_le_bytes());  // Format flags
    output.extend_from_slice(&0u32.to_le_bytes());  // Reserved

    // Vertex data (as bytes)
    for &f in vertices {
        output.extend_from_slice(&f.to_le_bytes());
    }

    // Index data (as bytes)
    for &i in indices {
        output.extend_from_slice(&i.to_le_bytes());
    }

    output
}
```

### Texture Converter

**Input:** PNG/JPG

**Process:**
1. Decode using `image` crate
2. Convert to RGBA8 (force alpha channel)
3. Resize if exceeds limits (optional: `--max-size 2048`)
4. Optionally: Generate palette + dithering
5. Write Rust source

**Pseudocode:**

```rust
fn convert_texture(input: &Path, palette: Option<usize>, dither: Option<Dither>) -> Result<Vec<u8>> {
    let img = image::open(input)?;
    let rgba = img.to_rgba8();

    let (width, height) = rgba.dimensions();
    let pixels = rgba.as_raw();

    let binary = if let Some(colors) = palette {
        generate_palette_binary(pixels, width, height, colors, dither)
    } else {
        generate_rgba_binary(pixels, width, height)
    };

    Ok(binary)
}

fn generate_rgba_binary(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut output = Vec::new();

    // Header
    output.extend_from_slice(b"EMTX");
    output.extend_from_slice(&1u32.to_le_bytes());  // Version
    output.extend_from_slice(&width.to_le_bytes());
    output.extend_from_slice(&height.to_le_bytes());
    output.extend_from_slice(&0u32.to_le_bytes());  // Format: RGBA8
    output.extend_from_slice(&0u32.to_le_bytes());  // Reserved

    // Pixel data
    output.extend_from_slice(pixels);

    output
}
```

### Font Converter

**Input:** TTF/OTF

**Process:**
1. Parse using `ttf-parser`
2. Rasterize glyphs at specified size
3. Pack into atlas texture (bin-packing algorithm)
4. Generate glyph metadata (UV coords, advance, kerning)
5. Write Rust source

**Future:** SDF (Signed Distance Field) generation for scalable fonts

### Audio Converter

**Input:** WAV/MP3/OGG

**Process:**
1. Decode using `symphonia`
2. Resample to 22,050 Hz
3. Convert stereo → mono
4. Convert to i16 PCM
5. Optionally: Normalize, trim silence, add loop points
6. Write Rust source

---

## Integration Guide

### Cargo Build Script Integration

**Goal:** Auto-convert assets during `cargo build`

**Approach:** Use `build.rs` in game crates

**Example: `examples/my-game/build.rs`**

```rust
use std::path::Path;
use std::process::Command;

fn main() {
    // Tell Cargo to re-run if assets change
    println!("cargo:rerun-if-changed=assets/");

    // Convert assets to binary files
    convert_mesh("assets/player.gltf", "assets/player.embermesh");
    convert_texture("assets/grass.png", "assets/grass.embertex");
    convert_font("assets/font.ttf", "assets/font.emberfont");
    convert_audio("assets/jump.wav", "assets/jump.embersnd");
}

fn convert_mesh(input: &str, output: &str) {
    let status = Command::new("ember-export")
        .args(&["mesh", input, "--output", output])
        .status()
        .expect("Failed to run ember-export");

    if !status.success() {
        panic!("ember-export failed");
    }
}

// ... (similar for texture, font, audio)
```

**Game code (Rust):**

```rust
// examples/my-game/src/main.rs
#![no_std]
#![no_main]

// Embed binary assets at compile-time
static PLAYER_MESH: &[u8] = include_bytes!("../assets/player.embermesh");
static GRASS_TEX: &[u8] = include_bytes!("../assets/grass.embertex");
static UI_FONT: &[u8] = include_bytes!("../assets/font.emberfont");
static JUMP_SFX: &[u8] = include_bytes!("../assets/jump.embersnd");

#[no_mangle]
pub extern "C" fn init() {
    // Parse mesh header
    let mesh_header = parse_mesh_header(PLAYER_MESH);
    let player_mesh = load_mesh_indexed(
        mesh_header.vertices_ptr(),
        mesh_header.vertex_count,
        mesh_header.indices_ptr(),
        mesh_header.index_count,
        mesh_header.format,
    );

    // Parse texture header
    let tex_width = u32::from_le_bytes([GRASS_TEX[8], GRASS_TEX[9], GRASS_TEX[10], GRASS_TEX[11]]);
    let tex_height = u32::from_le_bytes([GRASS_TEX[12], GRASS_TEX[13], GRASS_TEX[14], GRASS_TEX[15]]);
    let grass_tex = load_texture(tex_width, tex_height, &GRASS_TEX[24] as *const u8);

    // ... same pattern for font and audio
}

// Simple helper to parse mesh headers
fn parse_mesh_header(data: &[u8]) -> MeshHeader {
    MeshHeader {
        vertex_count: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        index_count: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        format: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
    }
}
```

**Game code (C/C++):**

```c
// Use Emscripten's --embed-file or similar to embed assets
extern const unsigned char PLAYER_MESH[];
extern const unsigned char GRASS_TEX[];

void init() {
    // Parse mesh (same binary format, different language)
    uint32_t* mesh_header = (uint32_t*)(PLAYER_MESH + 8);
    uint32_t vertex_count = mesh_header[0];
    uint32_t index_count = mesh_header[1];
    uint32_t format = mesh_header[2];

    float* vertices = (float*)(PLAYER_MESH + 24);
    uint32_t stride = calculate_stride(format);
    uint16_t* indices = (uint16_t*)(PLAYER_MESH + 24 + vertex_count * stride);

    load_mesh_indexed(vertices, vertex_count, indices, index_count, format);
}
```

### Manifest-Based Batch Conversion (Future)

**`assets/manifest.toml`:**

```toml
[meshes]
player = { input = "player.gltf", format = "POS_UV_NORMAL_SKINNED" }
enemy = { input = "enemy.gltf", format = "POS_UV_NORMAL" }

[textures]
grass = { input = "grass.png" }
sky = { input = "sky.png", palette = 256, dither = "floyd" }

[fonts]
ui = { input = "roboto.ttf", size = 16, charset = "ascii" }

[audio]
jump = { input = "jump.wav" }
music = { input = "music.ogg" }
```

**Build script:**

```rust
fn main() {
    Command::new("ember-export")
        .args(&["batch", "assets/manifest.toml", "--output", "src/assets/"])
        .status()
        .expect("Failed to batch export");
}
```

**Generated:**

```
assets/
├── player.embermesh  (binary)
├── enemy.embermesh   (binary)
├── grass.embertex    (binary)
├── sky.embertex      (binary)
├── ui.emberfont      (binary)
├── jump.embersnd     (binary)
└── music.embersnd    (binary)
```

Then use `include_bytes!()` in your game code to embed them.

---

## Performance Considerations

### 1. Build Time

**Problem:** Converting assets on every build slows iteration

**Solutions:**

1. **Caching:** Only re-convert if source newer than output
   ```rust
   if input_modified_time() > output_modified_time() {
       convert_asset(input, output);
   }
   ```

2. **Incremental builds:** Use `cargo:rerun-if-changed` in build.rs

3. **Parallel conversion:** Convert multiple assets in parallel
   ```rust
   use rayon::prelude::*;
   assets.par_iter().for_each(|asset| convert(asset));
   ```

4. **Skip in dev mode:** Only convert in release builds (optional)
   ```rust
   #[cfg(not(debug_assertions))]
   convert_assets();
   ```

### 2. WASM Size

**Problem:** Embedding assets makes WASM larger

**Solutions:**

1. **ROM limit enforcement:** 12 MB hard limit (already in place)

2. **Compression:** Use `wasm-opt` after build
   ```bash
   wasm-opt -Oz --enable-bulk-memory game.wasm -o game.opt.wasm
   ```

3. **Asset optimization:**
   - Resize textures to minimum necessary
   - Use palette mode for sprites (75% size reduction)
   - Compress audio (ADPCM: 4:1 ratio)
   - Reduce mesh vertex counts (use LODs)

4. **Dead code elimination:** Cargo already does this

### 3. Runtime Loading

**Problem:** Uploading large assets to GPU takes time

**Current state:** All assets uploaded in `init()`, which is fine (one-time cost)

**Future optimization:** Lazy loading (upload on first use)
```rust
fn init() {
    // Register assets but don't upload
    register_texture(GRASS_PIXELS);
}

fn render() {
    // Upload on first bind
    texture_bind(grass_tex); // Uploads if not yet uploaded
}
```

### 4. Memory Usage

**Problem:** Vertex data duplicated (once in WASM const, once in GPU buffer)

**Mitigation:**
- Const data is in WASM read-only memory (shared across instances)
- GPU upload frees CPU-side buffer after upload (not yet implemented, but possible)

---

## Implementation Roadmap

### Phase 1: MVP (Mesh Converter)

**Goal:** Replace manual vertex arrays with glTF export

**Deliverables:**
1. `ember-export mesh` CLI tool
2. glTF → Rust vertex array conversion
3. Documentation with examples
4. Update cube example to use converted mesh

**Success Criteria:**
- ✅ Convert cube.gltf → cube.embermesh.rs
- ✅ Load in game with `load_mesh_indexed()`
- ✅ Visual result identical to manual vertex data
- ✅ <5 second conversion time for typical mesh

**Estimated Time:** 1-2 weeks

**Files to Create:**
- `tools/ember-export/` (new crate)
- `tools/ember-export/src/main.rs`
- `tools/ember-export/src/mesh.rs`
- `examples/cube/assets/cube.gltf` (Blender export)
- Update `examples/cube/build.rs`

### Phase 2: Texture & Audio

**Goal:** Automate texture and audio conversion

**Deliverables:**
1. `ember-export texture` (PNG → RGBA8)
2. `ember-export audio` (WAV → PCM)
3. Update textured-quad example

**Success Criteria:**
- ✅ Convert grass.png → grass.embertex.rs
- ✅ Convert jump.wav → jump.embersnd.rs
- ✅ Audio example works (create if missing)

**Estimated Time:** 1 week

### Phase 3: Font Converter

**Goal:** TTF → bitmap font atlas

**Deliverables:**
1. `ember-export font`
2. TTF rasterization
3. Atlas bin-packing
4. Glyph metadata generation

**Estimated Time:** 1-2 weeks

### Phase 4: Skinned Mesh Support

**Goal:** Export bone weights from Blender

**Deliverables:**
1. glTF skinning data extraction
2. Bone weight painting workflow docs
3. Update skinned-mesh example

**Estimated Time:** 1 week

### Phase 5: Advanced Features

**Optional enhancements:**

1. **Palette Mode Textures** (1 week)
   - Color quantization
   - Dithering (Floyd-Steinberg, Bayer)

2. **SDF Fonts** (1 week)
   - Signed Distance Field generation
   - Scalable retro fonts

3. **Audio Compression** (1 week)
   - ADPCM encoding
   - Loop point markers

4. **Batch Conversion** (3 days)
   - Manifest-based workflow
   - Parallel conversion

5. **Blender Plugin** (2-3 weeks)
   - Direct export from Blender
   - One-click workflow

---

## Answers to Key Questions

### Q1: Should assets be embedded in WASM or loaded at runtime?

**Answer: Embedded in WASM.**

**Rationale:**
- ✅ Deterministic builds (rollback netcode requirement)
- ✅ No filesystem in WASM
- ✅ Fast loading (assets in linear memory)
- ✅ Simple deployment (single .wasm file)
- ✅ Already used in all examples
- ✅ 12 MB ROM limit is plenty for retro aesthetic

**Trade-off:** Larger WASM files, longer compile times (mitigated by caching)

### Q2: What's the target workflow: export → copy files, or integrated build?

**Answer: Integrated build via build.rs.**

**Workflow:**
```
1. Artist exports glTF from Blender
2. Game developer runs `cargo build`
3. build.rs runs ember-export automatically
4. Generated .rs files included in build
5. Single WASM output
```

**Alternative (manual):**
```
1. Developer runs: ember-export mesh cube.gltf
2. Manually include!() generated file
3. Commit generated file to repo (optional)
```

**Rationale:**
- Automated workflow reduces errors
- Cargo handles dependencies and caching
- Developer doesn't need to remember export commands
- Still inspect-able (generated Rust code in version control is optional)

### Q3: How do we handle versioning (format changes break old assets)?

**Answer: Version field in generated code + compatibility checks.**

**Generated code includes:**
```rust
// Generated by ember-export v0.2.0
// Format version: 1
pub const ASSET_FORMAT_VERSION: u32 = 1;
```

**FFI validates:**
```rust
fn load_mesh(...) {
    if asset_format_version != CURRENT_FORMAT_VERSION {
        error!("Asset format mismatch: expected {}, got {}", CURRENT, PROVIDED);
        return 0;
    }
}
```

**Migration strategy:**
- Bump format version when changing layout
- Regenerate assets with new ember-export
- Old assets fail with clear error message
- Optional: ember-export can upgrade old formats

### Q4: Should we use existing formats (glTF as-is) or custom binary?

**Answer: Use glTF as INPUT, custom Rust code as OUTPUT.**

**Input formats supported:**
- glTF 2.0 (industry standard, Blender default)
- OBJ (simple, widely supported)
- FBX (future, if needed)

**Output format:**
- Rust source code (const arrays)
- Not binary files

**Rationale:**
- glTF's indirection is overkill for embedded use
- Rust const arrays are simpler and faster
- Type-safe, human-readable, Cargo-optimized
- Matches current `load_mesh()` API

### Q5: How to balance file size vs. load time vs. runtime performance?

**Answer: Optimize for runtime performance, constrain file size.**

**Priorities:**
1. **Runtime performance:** Most important (60fps target)
2. **File size:** 12 MB ROM limit (enforced)
3. **Load time:** One-time cost in init() (acceptable)

**Strategies:**

**File size:**
- Texture palettes (75% reduction)
- Audio compression (4:1 ADPCM)
- Mesh simplification (Blender LOD tools)
- wasm-opt (10-30% reduction)

**Runtime performance:**
- Pre-converted formats (no runtime parsing)
- GPU-friendly layouts (already optimal)
- Immediate upload (no streaming latency)

**Load time:**
- Acceptable: 100ms for typical game
- Lazy upload (future): Only upload on first use

---

## References

### Research Sources

**Asset Pipeline Design:**
- [Asset Pipelines (CMU Graphics)](https://graphics.cs.cmu.edu/courses/15-466-f19/notes/asset-pipelines.html)
- [Unreal Engine Asset Pipeline](https://dev.epicgames.com/community/learning/courses/qEl/unreal-engine-technical-guide-to-linear-content-creation-pipeline-development/ryax/unreal-engine-asset-pipeline)
- [Asset Pipeline Optimization (Medium)](https://medium.com/@lemapp09/beginning-game-development-asset-pipeline-optimization-96495a2a795e)
- [Modern Asset Pipeline Introduction (Game Developer)](https://www.gamedeveloper.com/production/a-modern-asset-pipeline-introduction)
- [Amethyst distill (Rust Asset Pipeline)](https://github.com/amethyst/distill)

**glTF Format:**
- [glTF 2.0 Specification (Khronos)](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
- [glTF Wikipedia](https://en.wikipedia.org/wiki/GlTF)
- [glTF Files Explained (Jinolo)](https://jinolo.com/blog/gltf-files-explained/)
- [glTF Issues (DEV Community)](https://dev.to/drbearhands/the-gltf-file-format-for-3d-models-has-some-issues-46jo)

**WASM Asset Embedding:**
- [WASM Downloading Files vs include_bytes! (Medium)](https://emilio-moretti.medium.com/rust-wasm-downloading-files-in-runtime-instead-of-include-bytes-f8c29a958e20)
- [Run Rust Games in Browser (Hands-On Rust)](https://hands-on-rust.com/2021/11/06/run-your-rust-games-in-a-browser-hands-on-rust-bonus-content/)
- [include_bytes! with wasm_bindgen (GitHub Issue)](https://github.com/rustwasm/wasm-bindgen/issues/3230)
- [Roguelike Tutorial - Web Build](https://bfnightly.bracketproductions.com/rustbook/webbuild.html)
- [wasset Crate](https://docs.rs/wasset/latest/wasset/)

### Codebase References

**FFI Implementation:**
- `emberware-z/src/ffi/mesh.rs` - load_mesh()
- `emberware-z/src/ffi/texture.rs` - load_texture()
- `emberware-z/src/ffi/draw_2d.rs` - load_font()
- `emberware-z/src/ffi/audio.rs` - Audio functions

**Vertex Formats:**
- `emberware-z/src/graphics/vertex.rs` - Format definitions and vertex attributes

**Examples:**
- `examples/cube/src/lib.rs` - Manual vertex data
- `examples/textured-quad/src/lib.rs` - Procedural texture
- `examples/skinned-mesh/src/lib.rs` - Bone weight painting

**Documentation:**
- `docs/ffi.md` - FFI reference
- `docs/emberware-z.md` - Z-specific API

---

## Conclusion

This asset pipeline design provides a **pragmatic, Rust-first solution** for converting industry-standard assets into Emberware's embedded format. By generating Rust source code instead of binary files, we maintain type safety, human readability, and Cargo build integration while supporting the fantasy console's unique constraints (embedded assets, rollback netcode, 12 MB ROM limit).

**Next Steps:**

1. ✅ Review this design document
2. ⬜ Approve or request changes
3. ⬜ Create implementation tasks in TASKS.md
4. ⬜ Begin Phase 1: Mesh converter

**Estimated Total Time:** 6-10 weeks for full implementation (all phases)

**Minimum Viable Pipeline (Phase 1 only):** 1-2 weeks

