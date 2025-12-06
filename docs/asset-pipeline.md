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

### Format 1: EmberMesh (.embermesh.rs)

**Purpose:** Convert glTF/OBJ → Rust vertex arrays

**Output:** Rust source code (not binary)

**Example:**
```rust
// Generated by ember-export mesh cube.gltf
// Format: POS_UV_NORMAL (5)
// Vertices: 24, Indices: 36

pub const CUBE_VERTEX_COUNT: usize = 24;
pub const CUBE_INDEX_COUNT: usize = 36;
pub const CUBE_FORMAT: u32 = 5; // POS_UV_NORMAL

#[repr(C, align(4))]
pub struct CubeMeshData {
    pub vertices: [f32; 24 * 8], // 24 verts × 8 floats (pos+uv+normal)
    pub indices: [u16; 36],
}

pub const CUBE_MESH: CubeMeshData = CubeMeshData {
    vertices: [
        // Vertex 0: pos(x,y,z), uv(u,v), normal(nx,ny,nz)
        -1.0, -1.0, 1.0,  0.0, 0.0,  0.0, 0.0, 1.0,
        // ... (generated from glTF)
    ],
    indices: [
        0, 1, 2, 2, 3, 0, // Front face
        // ... (generated from glTF)
    ],
};
```

**Usage in game:**
```rust
// In game code
include!("assets/cube.embermesh.rs");

fn init() {
    let mesh = load_mesh_indexed(
        CUBE_MESH.vertices.as_ptr(),
        CUBE_VERTEX_COUNT as u32,
        CUBE_MESH.indices.as_ptr(),
        CUBE_INDEX_COUNT as u32,
        CUBE_FORMAT,
    );
}
```

**Why Rust code, not binary?**
1. Works with `include!()` macro
2. Type-safe (compile-time errors if format changes)
3. Human-readable (can inspect/debug)
4. No runtime parsing (const data in WASM)
5. Cargo handles optimization (dead code elimination)

**Metadata:**
```rust
// Optional metadata (comments or const)
// Original file: cube.gltf
// Exported: 2025-12-06 10:23:45 UTC
// Bounds: min(-1,-1,-1), max(1,1,1)
// Vertex format: FORMAT_UV | FORMAT_NORMAL
```

**Skinned Meshes:**
```rust
pub const CHARACTER_FORMAT: u32 = 13; // POS_UV_NORMAL_SKINNED

pub const CHARACTER_MESH: CharacterMeshData = CharacterMeshData {
    vertices: [
        // pos(3), uv(2), normal(3), bone_indices(4u8 as 1f32), bone_weights(4)
        // Total: 13 floats per vertex
    ],
    indices: [ /* ... */ ],
};

// Bone weights painted in Blender, exported automatically
```

**File Naming Convention:**
```
assets/
├── cube.gltf               # Source (Blender export)
├── cube.embermesh.rs       # Generated by ember-export
└── player.gltf             # Source
    └── player.embermesh.rs # Generated
```

---

### Format 2: EmberTexture (.embertex.rs)

**Purpose:** Convert PNG/JPG → raw RGBA8 Rust arrays

**Output:** Rust source code with pixel data

**Example:**
```rust
// Generated by ember-export texture grass.png

pub const GRASS_WIDTH: u32 = 64;
pub const GRASS_HEIGHT: u32 = 64;

pub const GRASS_PIXELS: [u8; 64 * 64 * 4] = [
    // RGBA8 pixels (generated from PNG decoder)
    0x3a, 0x7c, 0x2b, 0xff,  // Pixel 0: dark green
    0x42, 0x8a, 0x31, 0xff,  // Pixel 1
    // ... (64*64*4 = 16,384 bytes)
];
```

**Usage in game:**
```rust
include!("assets/grass.embertex.rs");

fn init() {
    let tex = load_texture(
        GRASS_WIDTH,
        GRASS_HEIGHT,
        GRASS_PIXELS.as_ptr(),
    );
}
```

**Advanced: Palette Mode (Future Optimization)**

**For authentic retro aesthetic with 256-color palettes:**

```rust
// Generated with: ember-export texture sprite.png --palette 256 --dither floyd

pub const SPRITE_WIDTH: u32 = 32;
pub const SPRITE_HEIGHT: u32 = 32;

pub const SPRITE_PALETTE: [u8; 256 * 4] = [
    // 256 RGBA8 colors
];

pub const SPRITE_INDICES: [u8; 32 * 32] = [
    // Index into palette (1 byte per pixel)
    // 75% smaller than RGBA8!
];
```

**FFI Addition (Future):**
```rust
fn load_texture_palette(
    width: u32,
    height: u32,
    indices_ptr: *const u8,
    palette_ptr: *const u8,  // 256 * 4 = 1024 bytes
) -> u32;
```

---

### Format 3: EmberFont (.emberfont.rs)

**Purpose:** Convert TTF/OTF → bitmap font atlas

**Output:** Texture + glyph metadata

**Example:**
```rust
// Generated by ember-export font roboto.ttf --size 16 --charset ascii

pub const ROBOTO_FONT_TEXTURE_WIDTH: u32 = 512;
pub const ROBOTO_FONT_TEXTURE_HEIGHT: u32 = 128;
pub const ROBOTO_FONT_LINE_HEIGHT: f32 = 16.0;
pub const ROBOTO_FONT_BASELINE: f32 = 12.0;

pub const ROBOTO_FONT_ATLAS: [u8; 512 * 128 * 4] = [
    // RGBA8 atlas texture (white glyphs on transparent)
];

pub struct GlyphInfo {
    pub codepoint: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub advance: f32,
}

pub const ROBOTO_FONT_GLYPHS: [GlyphInfo; 95] = [
    GlyphInfo { codepoint: 32, x: 0, y: 0, width: 0, height: 0, offset_x: 0, offset_y: 0, advance: 4.0 }, // Space
    GlyphInfo { codepoint: 33, x: 0, y: 0, width: 3, height: 12, offset_x: 1, offset_y: -12, advance: 5.0 }, // !
    // ... (ASCII 32-126 = 95 glyphs)
];
```

**Usage in game:**
```rust
include!("assets/roboto.emberfont.rs");

fn init() {
    let font_tex = load_texture(
        ROBOTO_FONT_TEXTURE_WIDTH,
        ROBOTO_FONT_TEXTURE_HEIGHT,
        ROBOTO_FONT_ATLAS.as_ptr(),
    );

    // Convert GlyphInfo → engine format
    let font = load_font_ex(
        font_tex,
        /* ... pass glyph data ... */
    );
}
```

**SDF Fonts (Future):**
For scalable retro fonts, generate Signed Distance Field atlases.

---

### Format 4: EmberSound (.embersnd.rs)

**Purpose:** Convert WAV/MP3/OGG → raw PCM @ 22.05kHz

**Output:** i16 PCM samples

**Example:**
```rust
// Generated by ember-export audio jump.wav

pub const JUMP_SAMPLE_RATE: u32 = 22050;
pub const JUMP_SAMPLE_COUNT: usize = 4410; // 0.2 seconds

pub const JUMP_SAMPLES: [i16; 4410] = [
    0, 127, 254, 380, // ... PCM samples
];
```

**Usage in game:**
```rust
include!("assets/jump.embersnd.rs");

fn init() {
    let jump_sfx = load_sound(
        JUMP_SAMPLES.as_ptr(),
        (JUMP_SAMPLE_COUNT * 2) as u32, // × 2 for bytes
    );
}
```

**Compression (Future):**
Use ADPCM or custom compression to save ROM space.

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

**Rust Code Generation:**

```rust
fn generate_rust_code(vertices: &[f32], indices: &[u16], format: u32) -> String {
    format!(
        r#"
// Generated by ember-export mesh
// Format: {format_name} ({format})
// Vertices: {vertex_count}, Indices: {index_count}

pub const VERTEX_COUNT: usize = {vertex_count};
pub const INDEX_COUNT: usize = {index_count};
pub const FORMAT: u32 = {format};

#[repr(C, align(4))]
pub struct MeshData {{
    pub vertices: [f32; {vertex_data_len}],
    pub indices: [u16; {index_count}],
}}

pub const MESH: MeshData = MeshData {{
    vertices: {vertices:#?},
    indices: {indices:#?},
}};
"#,
        format_name = format_name(format),
        format = format,
        vertex_count = vertices.len() / stride_in_floats(format),
        index_count = indices.len(),
        vertex_data_len = vertices.len(),
        vertices = vertices,
        indices = indices,
    )
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
fn convert_texture(input: &Path, palette: Option<usize>, dither: Option<Dither>) -> Result<String> {
    let img = image::open(input)?;
    let rgba = img.to_rgba8();

    let (width, height) = rgba.dimensions();
    let pixels = rgba.as_raw();

    let code = if let Some(colors) = palette {
        generate_palette_texture(pixels, width, height, colors, dither)
    } else {
        generate_rgba_texture(pixels, width, height)
    };

    Ok(code)
}

fn generate_rgba_texture(pixels: &[u8], width: u32, height: u32) -> String {
    format!(
        r#"
pub const WIDTH: u32 = {width};
pub const HEIGHT: u32 = {height};

pub const PIXELS: [u8; {len}] = {pixels:#?};
"#,
        width = width,
        height = height,
        len = pixels.len(),
        pixels = pixels,
    )
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

    // Convert assets
    convert_mesh("assets/player.gltf", "src/assets/player.embermesh.rs");
    convert_texture("assets/grass.png", "src/assets/grass.embertex.rs");
    convert_font("assets/font.ttf", "src/assets/font.emberfont.rs");
    convert_audio("assets/jump.wav", "src/assets/jump.embersnd.rs");
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

**Game code:**

```rust
// examples/my-game/src/main.rs
#![no_std]
#![no_main]

// Include generated assets
mod assets {
    include!("assets/player.embermesh.rs");
    include!("assets/grass.embertex.rs");
    include!("assets/font.emberfont.rs");
    include!("assets/jump.embersnd.rs");
}

use assets::*;

#[no_mangle]
pub extern "C" fn init() {
    let player_mesh = load_mesh_indexed(
        PLAYER_MESH.vertices.as_ptr(),
        PLAYER_VERTEX_COUNT as u32,
        PLAYER_MESH.indices.as_ptr(),
        PLAYER_INDEX_COUNT as u32,
        PLAYER_FORMAT,
    );

    let grass_tex = load_texture(
        GRASS_WIDTH,
        GRASS_HEIGHT,
        GRASS_PIXELS.as_ptr(),
    );

    // ...
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
src/assets/
├── player.embermesh.rs
├── enemy.embermesh.rs
├── grass.embertex.rs
├── sky.embertex.rs
├── ui.emberfont.rs
├── jump.embersnd.rs
└── music.embersnd.rs
```

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
- `/home/user/emberware/emberware-z/src/ffi/mod.rs:889-984` - load_mesh()
- `/home/user/emberware/emberware-z/src/ffi/mod.rs:735-801` - load_texture()
- `/home/user/emberware/emberware-z/src/ffi/mod.rs:2065-2234` - load_font()
- `/home/user/emberware/emberware-z/src/ffi/mod.rs:2865-3037` - Audio functions

**Vertex Formats:**
- `/home/user/emberware/emberware-z/src/graphics/vertex.rs:1-100` - Format definitions
- `/home/user/emberware/emberware-z/src/graphics/vertex.rs:195-310` - Vertex attributes

**Examples:**
- `/home/user/emberware/examples/cube/src/lib.rs` - Manual vertex data
- `/home/user/emberware/examples/textured-quad/src/lib.rs` - Procedural texture
- `/home/user/emberware/examples/skinned-mesh/src/lib.rs` - Bone weight painting

**Documentation:**
- `/home/user/emberware/docs/ffi.md` - FFI reference
- `/home/user/emberware/docs/emberware-z.md` - Z-specific API

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

