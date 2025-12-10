# Asset Pipeline Implementation Spec (Emberware Z)

**Status:** Implementation Guide
**Last Updated:** 2025-12-10
**Target:** Emberware Z console

This document specifies how to implement the asset pipeline tooling for **Emberware Z**.

---

## Overview

Build a CLI tool (`ember-export`) that converts industry-standard assets into Emberware Z's optimized binary formats, with optional code generation.

### Key Principle: Use emberware-z as a Dependency

The `ember-export` tool should depend on `emberware-z` crate to reuse existing code:
- **Vertex packing** - `emberware-z::graphics::packing` already implements all packing functions
- **Format definitions** - `emberware-z::graphics::vertex` has format flags and stride calculations
- **Ensures 1:1 compatibility** - Same code path for tool and runtime

```toml
# tools/ember-export/Cargo.toml
[dependencies]
emberware-z = { path = "../../emberware-z", default-features = false, features = ["export-tools"] }
```

### Goals

1. **Single command builds** - `ember-export build assets.toml` converts everything
2. **GPU-optimized output** - Use emberware-z's existing packed vertex formats
3. **Language-agnostic binaries** - Binary formats work with any WASM language
4. **Optional code generation** - Opt-in Rust module generation (other languages can be added later)
5. **DRY** - No duplicated packing/format code

### Non-Goals

- Runtime asset loading (all assets embedded at compile time)
- Asset streaming (12 MB ROM limit, embed everything)
- Hot reload (future work, not MVP)
- Code generation for non-Rust languages (community can contribute backends later)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ember-export CLI                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Mesh    │  │ Texture  │  │   Font   │  │  Audio   │    │
│  │Converter │  │Converter │  │Converter │  │Converter │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │             │             │           │
│       ▼             ▼             ▼             ▼           │
│  .embermesh    .embertex    .emberfont    .embersnd        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Code Generator (Rust)                   │   │
│  │  - include_bytes! for each asset                     │   │
│  │  - AssetPack struct with typed handles               │   │
│  │  - load() function wiring everything up              │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## CLI Design

### Location

```
tools/ember-export/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point, argument parsing
│   ├── manifest.rs       # assets.toml parsing
│   ├── mesh.rs           # glTF/FBX/OBJ → .embermesh (uses emberware-z packing)
│   ├── texture.rs        # PNG/JPG → .embertex
│   ├── font.rs           # TTF/OTF → .emberfont
│   ├── audio.rs          # WAV/OGG/MP3 → .embersnd
│   ├── codegen/
│   │   ├── mod.rs
│   │   └── rust.rs       # Rust code generation (optional)
│   └── formats/
│       ├── mod.rs
│       ├── embermesh.rs  # Binary format writers
│       ├── embertex.rs
│       ├── emberfont.rs
│       └── embersnd.rs
```

**Note:** No local `packing.rs` - uses `emberware_z::graphics::packing` directly.

### Commands

```bash
# Build all assets from manifest (PRIMARY WORKFLOW)
ember-export build assets.toml
ember-export build assets.toml --watch
ember-export build assets.toml --output-dir ./generated

# Validate manifest
ember-export check assets.toml

# Convert individual files (for debugging/testing)
ember-export mesh player.gltf -o player.embermesh
ember-export texture grass.png -o grass.embertex
ember-export font roboto.ttf -o ui.emberfont --size 16
ember-export audio jump.wav -o jump.embersnd
```

### Dependencies

```toml
[package]
name = "ember-export"
version = "0.1.0"
edition = "2021"

[dependencies]
# Emberware Z - vertex packing, format definitions (DRY!)
emberware-z = { path = "../../emberware-z", default-features = false, features = ["export-tools"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Manifest parsing
toml = "0.8"
serde = { version = "1", features = ["derive"] }

# Mesh loading
gltf = "1.4"                    # glTF 2.0
tobj = "4"                      # OBJ (simple, no dependencies)
# fbx = ???                     # FBX support TBD - consider russimp

# Texture loading
image = "0.25"                  # PNG/JPG decoding

# Font rasterization
fontdue = "0.8"                 # Fast font rasterization

# Audio decoding
symphonia = "0.5"               # WAV/MP3/OGG decoding

# File watching (optional, for --watch)
notify = "6"
```

Note: `half`, `glam`, and `bytemuck` come from `emberware-z` - no need to duplicate.

---

## Manifest Format (assets.toml)

### Schema

```toml
# Required: output directory for binary files
[output]
dir = "assets/"                 # Directory for .embermesh, .embertex, etc.

# Optional: code generation (ONE language or none)
# Omit this section entirely if you don't want code generation.
# Users of unsupported languages parse the binary formats directly.
[codegen]
rust = "src/assets.rs"          # Generate Rust module (only supported language for now)
# Future: community can contribute backends for other languages

# Optional: global settings
[settings]
# default_mesh_format = "POS_UV_NORMAL"  # Default vertex format

# Meshes: name → path or config
[meshes]
player = "models/player.gltf"
enemy = "models/enemy.fbx"
level = { path = "models/level.obj", format = "POS_UV_NORMAL" }
character = { path = "models/char.gltf", format = "POS_UV_NORMAL_SKINNED" }

# Textures: name → path or config
[textures]
player_diffuse = "textures/player.png"
grass = { path = "textures/grass.png", palette = 256 }
ui_atlas = { path = "textures/ui.png" }

# Fonts: name → config (size required)
[fonts]
ui = { path = "fonts/roboto.ttf", size = 16 }
title = { path = "fonts/title.otf", size = 32, charset = "ascii" }
# charset options: "ascii", "latin1", "all", or explicit "ABCabc123..."

# Audio: name → path or config
[sounds]
jump = "audio/jump.wav"
music = { path = "audio/theme.ogg" }
# All audio resampled to 22050 Hz mono PCM16
```

### Rust Types for Parsing

```rust
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Manifest {
    pub output: OutputConfig,
    #[serde(default)]
    pub codegen: Option<CodegenConfig>,  // Optional - users can skip code generation
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub meshes: HashMap<String, MeshEntry>,
    #[serde(default)]
    pub textures: HashMap<String, TextureEntry>,
    #[serde(default)]
    pub fonts: HashMap<String, FontEntry>,
    #[serde(default)]
    pub sounds: HashMap<String, SoundEntry>,
}

#[derive(Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_dir")]
    pub dir: PathBuf,  // Directory for binary files
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("assets")
}

/// Optional code generation config (ONE language or none)
/// If not present, only binary files are generated.
/// Users of unsupported languages parse the binary formats directly.
#[derive(Deserialize)]
pub struct CodegenConfig {
    pub rust: Option<PathBuf>,  // Generate Rust module (supported)
    // Future: community can contribute backends for other languages
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum MeshEntry {
    Simple(PathBuf),
    Config { path: PathBuf, format: Option<String> },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TextureEntry {
    Simple(PathBuf),
    Config { path: PathBuf, palette: Option<u32> },
}

#[derive(Deserialize)]
pub struct FontEntry {
    pub path: PathBuf,
    pub size: u32,
    #[serde(default = "default_charset")]
    pub charset: String,
}

fn default_charset() -> String {
    "ascii".to_string()
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SoundEntry {
    Simple(PathBuf),
    Config { path: PathBuf },
}
```

### Manifest Validation

Before processing, the manifest is validated to catch errors early:

```rust
use std::collections::HashSet;

impl Manifest {
    pub fn validate(&self, base_dir: &Path) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut all_names = HashSet::new();

        // Validate mesh entries
        for (name, entry) in &self.meshes {
            self.validate_asset_entry(name, entry.path(), &mut all_names, &mut errors, base_dir);
            // Validate format string if specified
            if let MeshEntry::Config { format: Some(fmt), .. } = entry {
                if parse_format_string(fmt).is_none() {
                    errors.push(ValidationError::InvalidFormat {
                        name: name.clone(),
                        format: fmt.clone(),
                    });
                }
            }
        }

        // Validate texture entries
        for (name, entry) in &self.textures {
            self.validate_asset_entry(name, entry.path(), &mut all_names, &mut errors, base_dir);
        }

        // Validate font entries
        for (name, entry) in &self.fonts {
            self.validate_asset_entry(name, &entry.path, &mut all_names, &mut errors, base_dir);
        }

        // Validate sound entries
        for (name, entry) in &self.sounds {
            self.validate_asset_entry(name, entry.path(), &mut all_names, &mut errors, base_dir);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    fn validate_asset_entry(
        &self,
        name: &str,
        path: &Path,
        all_names: &mut HashSet<String>,
        errors: &mut Vec<ValidationError>,
        base_dir: &Path,
    ) {
        // Check name is valid identifier
        if !is_valid_identifier(name) {
            errors.push(ValidationError::InvalidName(name.to_string()));
        }
        // Check for duplicates across ALL asset types
        if !all_names.insert(name.to_string()) {
            errors.push(ValidationError::Duplicate(name.to_string()));
        }
        // Check file exists
        if !base_dir.join(path).exists() {
            errors.push(ValidationError::FileNotFound {
                name: name.to_string(),
                path: path.to_path_buf(),
            });
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidName(String),
    Duplicate(String),
    FileNotFound { name: String, path: PathBuf },
    InvalidFormat { name: String, format: String },
}

/// Valid identifier: [a-zA-Z_][a-zA-Z0-9_]*
/// This ensures generated code (Rust, C, etc.) will compile.
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        None => false,
        Some(c) if !c.is_ascii_alphabetic() && c != '_' => false,
        _ => chars.all(|c| c.is_ascii_alphanumeric() || c == '_'),
    }
}

/// Parse format string like "POS_UV_NORMAL" into format flags.
/// Returns None if the format string is invalid.
fn parse_format_string(s: &str) -> Option<u8> {
    let mut format = 0u8;
    for part in s.split('_') {
        match part {
            "POS" => {} // Always present, no flag needed
            "UV" => format |= FORMAT_UV,
            "COLOR" => format |= FORMAT_COLOR,
            "NORMAL" => format |= FORMAT_NORMAL,
            "SKINNED" => format |= FORMAT_SKINNED,
            _ => return None, // Unknown component
        }
    }
    Some(format)
}
```

---

## Binary Format Conventions

### Endianness

All multi-byte integers are **little-endian** (x86/ARM native order). This is the native byte order for the target platforms and avoids byte-swapping overhead.

### Alignment

Headers are naturally aligned. Reserved fields ensure:
- Future extensibility without breaking compatibility
- Natural alignment for efficient memory access on all platforms

### Version Strategy

- **Version 1**: Initial format (current spec)
- Future versions must either be backward-compatible or the tool must provide a migration utility
- Version field at fixed offset (0x04) allows quick format detection

### Magic Numbers

Each format has a 4-byte ASCII magic for file type identification:

| Magic | Format | Description |
|-------|--------|-------------|
| `EMSH` | EmberMesh | 3D mesh with packed vertices |
| `EMTX` | EmberTexture | Texture atlas or sprite |
| `EMFT` | EmberFont | Bitmap font atlas |
| `EMSN` | EmberSound | Audio samples |

Magic bytes allow tools to identify file types without relying on extensions.

---

## Binary Format Specifications

### EmberMesh (.embermesh)

```
Offset | Size | Type    | Description
-------|------|---------|----------------------------------
0x00   | 4    | char[4] | Magic: "EMSH"
0x04   | 4    | u32     | Format version (1)
0x08   | 4    | u32     | Vertex count
0x0C   | 4    | u32     | Index count (0 if non-indexed)
0x10   | 1    | u8      | Vertex format flags (0-15)
0x11   | 1    | u8      | Reserved (0)
0x12   | 2    | u16     | Reserved (0)
0x14   | 4    | u32     | Vertex stride in bytes
0x18   | var  | bytes   | Vertex data (vertex_count * stride)
var    | var  | u16[]   | Index data (index_count * 2), if indexed
```

**Vertex format flags:**
```rust
const FORMAT_UV: u8 = 1;
const FORMAT_COLOR: u8 = 2;
const FORMAT_NORMAL: u8 = 4;
const FORMAT_SKINNED: u8 = 8;
```

**Packed vertex layout (in order, only present if flag set):**

| Attribute | Format | Size | Condition |
|-----------|--------|------|-----------|
| Position | Float16x4 | 8 bytes | Always |
| UV | Unorm16x2 | 4 bytes | FORMAT_UV |
| Color | Unorm8x4 | 4 bytes | FORMAT_COLOR |
| Normal | Uint32 (octahedral) | 4 bytes | FORMAT_NORMAL |
| Bone Indices | Uint8x4 | 4 bytes | FORMAT_SKINNED |
| Bone Weights | Unorm8x4 | 4 bytes | FORMAT_SKINNED |

### EmberTexture (.embertex)

```
Offset | Size | Type    | Description
-------|------|---------|----------------------------------
0x00   | 4    | char[4] | Magic: "EMTX"
0x04   | 4    | u32     | Format version (1)
0x08   | 4    | u32     | Width in pixels
0x0C   | 4    | u32     | Height in pixels
0x10   | 4    | u32     | Pixel format (0=RGBA8, 1=Palette256)
0x14   | 4    | u32     | Reserved (0)
0x18   | var  | bytes   | Pixel data (format-dependent)
```

**Pixel formats:**
- **0 (RGBA8):** `width * height * 4` bytes, row-major, top-to-bottom
- **1 (Palette256):** 1024 bytes palette (256 × RGBA8) + `width * height` bytes indices

### EmberFont (.emberfont)

```
Offset | Size | Type    | Description
-------|------|---------|----------------------------------
0x00   | 4    | char[4] | Magic: "EMFT"
0x04   | 4    | u32     | Format version (1)
0x08   | 4    | u32     | Atlas width
0x0C   | 4    | u32     | Atlas height
0x10   | 4    | f32     | Line height
0x14   | 4    | f32     | Ascent
0x18   | 4    | u32     | Glyph count
0x1C   | 4    | u32     | Reserved (0)
0x20   | var  | Glyph[] | Glyph array (glyph_count * 20 bytes)
var    | var  | bytes   | Atlas texture (RGBA8, width * height * 4)
```

**Glyph structure (20 bytes):**
```
Offset | Size | Type | Description
-------|------|------|----------------------------------
0x00   | 4    | u32  | Unicode codepoint
0x04   | 2    | u16  | X position in atlas
0x06   | 2    | u16  | Y position in atlas
0x08   | 2    | u16  | Width in atlas
0x0A   | 2    | u16  | Height in atlas
0x0C   | 2    | i16  | X offset (bearing)
0x0E   | 2    | i16  | Y offset (bearing)
0x10   | 4    | f32  | Advance width
```

### EmberSound (.embersnd)

```
Offset | Size | Type    | Description
-------|------|---------|----------------------------------
0x00   | 4    | char[4] | Magic: "EMSN"
0x04   | 4    | u32     | Format version (1)
0x08   | 4    | u32     | Sample rate (always 22050)
0x0C   | 4    | u32     | Sample count
0x10   | 4    | u32     | Audio format (0=PCM16)
0x14   | 4    | u32     | Reserved (0)
0x18   | var  | i16[]   | PCM samples (sample_count * 2 bytes)
```

---

## Vertex Packing (Use emberware-z)

**Do NOT duplicate packing code.** Use emberware-z crate as a dependency:

```rust
// In ember-export mesh converter
use emberware_z::graphics::packing::{
    pack_position_f16,
    pack_uv_unorm16,
    pack_color_unorm8,
    pack_normal_octahedral,
    pack_bone_weights_unorm8,
};
use emberware_z::graphics::vertex::{
    vertex_stride_packed,
    FORMAT_UV, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED,
};
```

**Reference implementations:**
- Packing functions: [emberware-z/src/graphics/packing.rs](../emberware-z/src/graphics/packing.rs)
- Format definitions: [emberware-z/src/graphics/vertex.rs](../emberware-z/src/graphics/vertex.rs)

This ensures 1:1 compatibility between the export tool and runtime.

---

## Mesh Converter Implementation

### glTF Loading

```rust
use gltf::Gltf;
use emberware_z::graphics::packing::*;
use emberware_z::graphics::vertex::*;

pub fn convert_gltf(path: &Path, format: u8) -> Result<EmberMesh> {
    let gltf = Gltf::open(path)?;
    let buffers = gltf::import_buffers(&gltf, Some(path.parent().unwrap()))?;

    let mesh = gltf.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive")?;

    // Extract positions (required)
    let positions = read_positions(&primitive, &buffers)?;

    // Extract optional attributes based on format
    let uvs = if format & FORMAT_UV != 0 {
        read_uvs(&primitive, &buffers)?
    } else {
        None
    };

    let colors = if format & FORMAT_COLOR != 0 {
        read_colors(&primitive, &buffers)?
    } else {
        None
    };

    let normals = if format & FORMAT_NORMAL != 0 {
        read_normals(&primitive, &buffers)?
    } else {
        None
    };

    let (bone_indices, bone_weights) = if format & FORMAT_SKINNED != 0 {
        read_skinning(&primitive, &buffers)?
    } else {
        (None, None)
    };

    // Extract indices
    let indices = read_indices(&primitive, &buffers)?;

    // Pack vertices using emberware-z functions
    let vertex_count = positions.len();
    let stride = vertex_stride_packed(format as u32);  // Use emberware-z
    let mut vertex_data = Vec::with_capacity(vertex_count * stride as usize);

    for i in 0..vertex_count {
        // Position (always) - uses emberware-z::graphics::packing
        vertex_data.extend_from_slice(&pack_position_f16(
            positions[i][0],
            positions[i][1],
            positions[i][2],
        ));

        // UV
        if let Some(ref uvs) = uvs {
            vertex_data.extend_from_slice(&pack_uv_unorm16(uvs[i][0], uvs[i][1]));
        }

        // Color
        if let Some(ref colors) = colors {
            vertex_data.extend_from_slice(&pack_color_unorm8(
                colors[i][0],
                colors[i][1],
                colors[i][2],
            ));
        }

        // Normal (octahedral encoding)
        if let Some(ref normals) = normals {
            vertex_data.extend_from_slice(&pack_normal_octahedral(
                normals[i][0],
                normals[i][1],
                normals[i][2],
            ));
        }

        // Skinning
        if let (Some(ref indices), Some(ref weights)) = (&bone_indices, &bone_weights) {
            vertex_data.extend_from_slice(&indices[i]);  // Already u8x4
            vertex_data.extend_from_slice(&pack_bone_weights_unorm8(weights[i]));
        }
    }

    Ok(EmberMesh {
        vertex_count: vertex_count as u32,
        index_count: indices.as_ref().map(|i| i.len() as u32).unwrap_or(0),
        format,
        stride,
        vertex_data,
        index_data: indices,
    })
}
```

---

## Code Generation

### Generated Rust Module

```rust
pub fn generate_rust_module(
    manifest: &Manifest,
    assets: &ProcessedAssets,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str("// AUTO-GENERATED by ember-export - do not edit\n");
    output.push_str(&format!("// Source: {}\n\n", manifest.source_path.display()));

    // Module start
    output.push_str("pub mod assets {\n");
    output.push_str("    use crate::ffi::*;\n\n");

    // Static byte arrays
    for (name, path) in &assets.meshes {
        output.push_str(&format!(
            "    static {}_MESH: &[u8] = include_bytes!(\"{}\");\n",
            name.to_uppercase(),
            path.display()
        ));
    }
    for (name, path) in &assets.textures {
        output.push_str(&format!(
            "    static {}_TEX: &[u8] = include_bytes!(\"{}\");\n",
            name.to_uppercase(),
            path.display()
        ));
    }
    // ... fonts, sounds

    output.push_str("\n");

    // AssetPack struct
    output.push_str("    /// All loaded asset handles\n");
    output.push_str("    pub struct AssetPack {\n");
    for name in assets.meshes.keys() {
        output.push_str(&format!("        pub {}: u32,\n", name));
    }
    for name in assets.textures.keys() {
        output.push_str(&format!("        pub {}: u32,\n", name));
    }
    // ... fonts, sounds
    output.push_str("    }\n\n");

    // load() function
    output.push_str("    /// Load all assets. Call once in init().\n");
    output.push_str("    pub fn load() -> AssetPack {\n");
    output.push_str("        AssetPack {\n");
    for name in assets.meshes.keys() {
        output.push_str(&format!(
            "            {}: load_mesh_from_embermesh({}_MESH),\n",
            name,
            name.to_uppercase()
        ));
    }
    for name in assets.textures.keys() {
        output.push_str(&format!(
            "            {}: load_texture_from_embertex({}_TEX),\n",
            name,
            name.to_uppercase()
        ));
    }
    // ... fonts, sounds
    output.push_str("        }\n");
    output.push_str("    }\n");

    // Module end
    output.push_str("}\n");

    output
}
```

---

## Runtime Loader Functions (FFI Side)

These functions need to be added to the Emberware runtime to load the binary formats:

```rust
// In emberware-z/src/ffi/assets.rs

/// Load mesh from .embermesh binary data
pub fn load_mesh_from_embermesh(data: &[u8]) -> u32 {
    // Parse header
    let magic = &data[0..4];
    assert_eq!(magic, b"EMSH", "Invalid embermesh magic");

    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    assert_eq!(version, 1, "Unsupported embermesh version");

    let vertex_count = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let index_count = u32::from_le_bytes(data[12..16].try_into().unwrap());
    let format = data[16];
    let stride = u32::from_le_bytes(data[20..24].try_into().unwrap());

    let vertex_data_start = 24;
    let vertex_data_size = (vertex_count * stride) as usize;
    let index_data_start = vertex_data_start + vertex_data_size;

    if index_count > 0 {
        load_mesh_indexed_packed(
            &data[vertex_data_start],
            vertex_count,
            &data[index_data_start],
            index_count,
            format as u32,
        )
    } else {
        load_mesh_packed(
            &data[vertex_data_start],
            vertex_count,
            format as u32,
        )
    }
}

/// Load texture from .embertex binary data
pub fn load_texture_from_embertex(data: &[u8]) -> u32 {
    let magic = &data[0..4];
    assert_eq!(magic, b"EMTX", "Invalid embertex magic");

    let width = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let height = u32::from_le_bytes(data[12..16].try_into().unwrap());
    let pixel_format = u32::from_le_bytes(data[16..20].try_into().unwrap());

    match pixel_format {
        0 => {
            // RGBA8
            load_texture(width, height, &data[24])
        }
        1 => {
            // Palette256 - needs load_texture_palette FFI
            let palette = &data[24..24 + 1024];
            let indices = &data[24 + 1024..];
            load_texture_palette(width, height, indices, palette)
        }
        _ => panic!("Unknown pixel format"),
    }
}

// Similar for load_font_from_emberfont, load_sound_from_embersnd
```

---

## Implementation Order

### Phase 1: MVP - Mesh Pipeline
1. Create `tools/ember-export/` crate structure with emberware-z dependency
2. Implement CLI argument parsing with clap
3. Implement `assets.toml` manifest parsing
4. Implement glTF mesh loading (positions, UVs, normals)
5. Wire up emberware-z packing functions (no new packing code!)
6. Implement .embermesh binary writer
7. Implement Rust code generation (optional feature)
8. Add `load_mesh_from_embermesh` to runtime FFI
9. Test with cube/platformer examples

### Phase 2: Full Asset Types
1. Add OBJ mesh loading
2. Add texture converter (PNG → .embertex)
3. Add `load_texture_from_embertex` to runtime
4. Add font converter (TTF → .emberfont with fontdue)
5. Add `load_font_from_emberfont` to runtime
6. Add audio converter (WAV/OGG → .embersnd)
7. Add `load_sound_from_embersnd` to runtime

### Phase 3: Skinned Meshes
1. Add glTF skinning data extraction (joints, weights)
2. Use emberware-z bone packing functions
3. Test with skinned-mesh example

### Phase 4: Polish
1. Add FBX support (if needed)
2. Add `--watch` mode with notify crate
3. Add palette texture support
4. Error messages and validation
5. Documentation and examples

---

## Error Handling

### Error Types

The tool uses structured errors with clear messages for both humans and automated systems:

```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    // IO errors
    #[error("Failed to read '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // Manifest errors
    #[error("Invalid manifest: {0}")]
    ManifestParse(#[from] toml::de::Error),

    #[error("Asset name '{name}' is not a valid identifier (must be [a-zA-Z_][a-zA-Z0-9_]*)")]
    InvalidAssetName { name: String },

    #[error("Duplicate asset name '{name}' found in manifest")]
    DuplicateAssetName { name: String },

    // Mesh errors
    #[error("Mesh '{name}': file not found at '{path}'")]
    MeshNotFound { name: String, path: PathBuf },

    #[error("Mesh '{name}': missing required attribute '{attribute}'")]
    MissingAttribute { name: String, attribute: String },

    #[error("Mesh '{name}': {count} vertices exceeds u16 index limit (65535)")]
    TooManyVertices { name: String, count: usize },

    #[error("Mesh '{name}': invalid format string '{format}'")]
    InvalidFormat { name: String, format: String },

    #[error("Mesh '{name}': glTF parsing failed: {reason}")]
    GltfParse { name: String, reason: String },

    // Texture errors
    #[error("Texture '{name}': unsupported format (only PNG/JPG supported)")]
    UnsupportedTextureFormat { name: String },

    #[error("Texture '{name}': dimensions {width}x{height} exceed maximum 4096x4096")]
    TextureTooLarge { name: String, width: u32, height: u32 },

    // Font errors
    #[error("Font '{name}': failed to parse TTF/OTF: {reason}")]
    FontParseFailed { name: String, reason: String },

    // Audio errors
    #[error("Audio '{name}': unsupported format (only WAV/OGG/MP3 supported)")]
    UnsupportedAudioFormat { name: String },

    #[error("Audio '{name}': decoding failed: {reason}")]
    AudioDecodeFailed { name: String, reason: String },
}
```

### Exit Codes

Consistent exit codes allow scripts and CI systems to handle errors appropriately:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | All assets converted |
| 1 | Manifest error | Parse failure, invalid config |
| 2 | Asset error | Missing file, invalid format, conversion failure |
| 3 | IO error | Permission denied, disk full |

### Error Output Format

Errors are printed to stderr in a format inspired by rustc for familiarity:

```
error[E002]: Mesh 'player': missing required attribute 'TEXCOORD_0'
  --> assets.toml:12
   |
12 | player = { path = "models/player.gltf", format = "POS_UV_NORMAL" }
   |                                                   ^^^^^^^^^^^^^^
   |
   = help: The mesh file doesn't contain UV coordinates but format requires them
   = help: Either add UVs to the mesh or use format "POS_NORMAL"
```

For JSON output (useful for IDE integration), use `--message-format=json`:

```json
{
  "level": "error",
  "code": "E002",
  "message": "Mesh 'player': missing required attribute 'TEXCOORD_0'",
  "file": "assets.toml",
  "line": 12,
  "help": ["Add UVs to the mesh", "Use format 'POS_NORMAL'"]
}
```

---

## Incremental Build Support

### Design for Future Caching

While MVP rebuilds all assets on every run, the architecture is designed to support incremental builds in the future. This section documents the design decisions and breadcrumbs left in the code.

### Build Cache Structure

```
.ember-cache/
├── manifest_hash          # xxHash of assets.toml content
├── sources.json           # { "player": { "hash": 12345, "mtime": "..." } }
└── outputs.json           # { "player.embermesh": "player" }
```

### Cache-Friendly Interfaces

Each converter implements a trait that exposes the information needed for caching:

```rust
use std::path::PathBuf;

/// Trait for cache-aware asset conversion.
/// MVP: Implement this trait even though caching is not used yet.
/// Future: Use content_hash() for incremental builds.
pub trait AssetConverter {
    /// Unique identifier for this asset (name from manifest)
    fn name(&self) -> &str;

    /// All source files this asset depends on.
    /// For meshes, includes the main file plus any referenced textures/materials.
    fn sources(&self) -> Vec<PathBuf>;

    /// Compute content hash of all sources (for change detection).
    /// Default implementation hashes file contents; override for custom behavior.
    fn content_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        for source in self.sources() {
            if let Ok(content) = std::fs::read(&source) {
                content.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Perform the conversion. Must be deterministic:
    /// same inputs must always produce identical outputs.
    fn convert(&self) -> Result<Vec<u8>, ExportError>;
}
```

### MVP Code Breadcrumbs

To ensure future caching can be added without major refactoring:

1. **Each converter implements `AssetConverter`** (even if `content_hash()` is unused)
2. **Build process is structured as phases**: validate → collect → convert → write
3. **Conversion is pure**: same inputs always produce byte-identical outputs
4. **No global mutable state** in converters
5. **Output paths are deterministic**: `{output_dir}/{name}.{extension}`

### Future Incremental Algorithm

When incremental builds are implemented:

```rust
pub fn build_incremental(manifest: &Manifest, cache: &mut BuildCache) -> Result<BuildStats> {
    let mut stats = BuildStats::default();

    for asset in manifest.all_assets() {
        let current_hash = asset.content_hash();
        let cached_hash = cache.get_hash(asset.name());

        if cached_hash == Some(current_hash) {
            stats.skipped += 1;
            continue; // No changes, skip conversion
        }

        // Convert and write
        let output = asset.convert()?;
        write_output(asset.name(), &output)?;
        cache.set_hash(asset.name(), current_hash);
        stats.converted += 1;
    }

    Ok(stats)
}
```

### Watch Mode Integration

The `--watch` flag (Phase 4) will use file system events to trigger rebuilds. With incremental caching:
- Only changed files are reconverted
- Dependent assets are automatically rebuilt (via dependency tracking in `sources()`)
- Cache invalidation is automatic when manifest changes

---

## Testing Strategy

A comprehensive test suite ensures the asset pipeline produces correct, stable output that can be verified by both humans and automated systems.

### Test Organization

```
tools/ember-export/
├── src/
│   └── ... (implementation)
├── tests/
│   ├── common/
│   │   └── mod.rs              # Shared test utilities
│   ├── unit/
│   │   ├── manifest.rs         # Manifest parsing tests
│   │   ├── validation.rs       # Validation logic tests
│   │   ├── mesh_format.rs      # EmberMesh binary format tests
│   │   ├── texture_format.rs   # EmberTexture binary format tests
│   │   ├── font_format.rs      # EmberFont binary format tests
│   │   └── audio_format.rs     # EmberSound binary format tests
│   ├── integration/
│   │   ├── gltf_pipeline.rs    # glTF → embermesh end-to-end
│   │   ├── full_manifest.rs    # Complete manifest processing
│   │   └── codegen.rs          # Generated Rust code compiles
│   └── fixtures/
│       ├── meshes/
│       │   ├── cube.gltf       # Minimal valid mesh (8 verts, 12 tris)
│       │   ├── cube_skinned.gltf # Skinned mesh with 2 bones
│       │   ├── no_uvs.gltf     # Mesh without UVs (for error testing)
│       │   └── large.gltf      # Stress test (10k+ vertices)
│       ├── textures/
│       │   ├── 2x2_rgba.png    # Minimal texture
│       │   ├── 256x256.png     # Standard texture
│       │   └── palette_test.png # For palette mode testing
│       ├── fonts/
│       │   └── test.ttf        # Minimal TTF for testing
│       ├── audio/
│       │   └── beep.wav        # Short test sound (~100ms)
│       ├── manifests/
│       │   ├── minimal.toml    # Single mesh only
│       │   ├── full.toml       # All asset types
│       │   ├── invalid_name.toml   # Invalid asset name
│       │   └── missing_file.toml   # Reference to non-existent file
│       └── golden/
│           ├── cube.embermesh  # Known-good binary outputs
│           └── 2x2_rgba.embertex
```

### Unit Tests

#### Manifest Parsing (`tests/unit/manifest.rs`)

```rust
#[test]
fn parse_minimal_manifest() {
    let toml = r#"
        [output]
        dir = "assets/"
        [meshes]
        cube = "cube.gltf"
    "#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert_eq!(manifest.meshes.len(), 1);
    assert!(manifest.meshes.contains_key("cube"));
}

#[test]
fn parse_full_manifest() {
    let manifest = Manifest::from_file("tests/fixtures/manifests/full.toml").unwrap();
    assert!(manifest.meshes.contains_key("player"));
    assert!(manifest.textures.contains_key("grass"));
    assert!(manifest.fonts.contains_key("ui"));
    assert!(manifest.sounds.contains_key("jump"));
}

#[test]
fn reject_invalid_asset_name() {
    let toml = r#"
        [output]
        dir = "assets/"
        [meshes]
        "invalid-name" = "mesh.gltf"
    "#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let errors = manifest.validate(Path::new(".")).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidName(_))));
}

#[test]
fn reject_duplicate_names_across_types() {
    let toml = r#"
        [output]
        dir = "assets/"
        [meshes]
        player = "mesh1.gltf"
        [textures]
        player = "tex.png"
    "#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let errors = manifest.validate(Path::new(".")).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::Duplicate(_))));
}

#[test]
fn parse_format_strings() {
    assert_eq!(parse_format_string("POS"), Some(0));
    assert_eq!(parse_format_string("POS_UV"), Some(FORMAT_UV));
    assert_eq!(parse_format_string("POS_UV_NORMAL"), Some(FORMAT_UV | FORMAT_NORMAL));
    assert_eq!(parse_format_string("POS_UV_NORMAL_SKINNED"),
               Some(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED));
    assert_eq!(parse_format_string("INVALID"), None);
    assert_eq!(parse_format_string("POS_UNKNOWN"), None);
}
```

#### Binary Format Round-Trip (`tests/unit/mesh_format.rs`)

```rust
use ember_export::formats::{EmberMesh, write_embermesh, read_embermesh};

#[test]
fn embermesh_roundtrip_pos_only() {
    let original = create_test_mesh(0); // POS only

    let mut buffer = Vec::new();
    write_embermesh(&mut buffer, &original).unwrap();

    let restored = read_embermesh(&buffer).unwrap();
    assert_eq!(original.vertex_count, restored.vertex_count);
    assert_eq!(original.format, restored.format);
    assert_eq!(original.vertex_data, restored.vertex_data);
}

#[test]
fn embermesh_roundtrip_all_formats() {
    // Test all 16 vertex format combinations
    for format in 0..=15u8 {
        let mesh = create_test_mesh(format);
        let buffer = write_embermesh_to_vec(&mesh).unwrap();
        let restored = read_embermesh(&buffer).unwrap();

        assert_eq!(mesh.format, restored.format,
                   "Format {} roundtrip failed: format mismatch", format);
        assert_eq!(mesh.vertex_data, restored.vertex_data,
                   "Format {} roundtrip failed: vertex data mismatch", format);
        assert_eq!(mesh.index_data, restored.index_data,
                   "Format {} roundtrip failed: index data mismatch", format);
    }
}

#[test]
fn embermesh_header_validation() {
    // Wrong magic
    let bad_magic = b"NOPE\x01\x00\x00\x00\x00\x00\x00\x00";
    assert!(matches!(
        read_embermesh(bad_magic),
        Err(FormatError::InvalidMagic(_))
    ));

    // Wrong version
    let bad_version = b"EMSH\x99\x00\x00\x00\x00\x00\x00\x00";
    assert!(matches!(
        read_embermesh(bad_version),
        Err(FormatError::UnsupportedVersion(0x99))
    ));

    // Truncated header
    let truncated = b"EMSH\x01";
    assert!(matches!(
        read_embermesh(truncated),
        Err(FormatError::UnexpectedEof)
    ));
}

#[test]
fn embermesh_endianness() {
    // Verify little-endian encoding
    let mesh = EmberMesh {
        vertex_count: 0x12345678,
        index_count: 0,
        format: 0,
        stride: 8,
        vertex_data: vec![0; 8],
        index_data: None,
    };
    let buffer = write_embermesh_to_vec(&mesh).unwrap();

    // Vertex count at offset 0x08, little-endian
    assert_eq!(&buffer[0x08..0x0C], &[0x78, 0x56, 0x34, 0x12],
               "Vertex count should be little-endian");
}
```

#### Golden File Tests (`tests/unit/mesh_format.rs`)

```rust
/// Golden file tests ensure binary format stability across versions.
/// If these fail after intentional format changes, regenerate with:
/// `cargo test -p ember-export update_golden -- --ignored`

#[test]
fn embermesh_golden_cube() {
    let mesh = convert_gltf(
        Path::new("tests/fixtures/meshes/cube.gltf"),
        FORMAT_UV | FORMAT_NORMAL
    ).unwrap();
    let output = write_embermesh_to_vec(&mesh).unwrap();

    let golden = std::fs::read("tests/fixtures/golden/cube.embermesh")
        .expect("Golden file missing - run update_golden test first");

    assert_eq!(output, golden,
        "Output differs from golden file.\n\
         If this is intentional, run: cargo test -p ember-export update_golden -- --ignored");
}

#[test]
#[ignore] // Run manually: cargo test -p ember-export update_golden -- --ignored
fn update_golden_files() {
    // Regenerate golden files after intentional format changes
    let mesh = convert_gltf(
        Path::new("tests/fixtures/meshes/cube.gltf"),
        FORMAT_UV | FORMAT_NORMAL
    ).unwrap();
    std::fs::write("tests/fixtures/golden/cube.embermesh",
                   write_embermesh_to_vec(&mesh).unwrap()).unwrap();

    let tex = convert_texture(Path::new("tests/fixtures/textures/2x2_rgba.png")).unwrap();
    std::fs::write("tests/fixtures/golden/2x2_rgba.embertex",
                   write_embertex_to_vec(&tex).unwrap()).unwrap();

    println!("Golden files updated successfully");
}
```

### Integration Tests

#### End-to-End Pipeline (`tests/integration/gltf_pipeline.rs`)

```rust
use emberware_z::graphics::packing::*;
use emberware_z::graphics::vertex::*;

#[test]
fn gltf_to_embermesh_to_runtime_load() {
    // 1. Convert glTF to embermesh
    let mesh = convert_gltf(
        Path::new("tests/fixtures/meshes/cube.gltf"),
        FORMAT_UV | FORMAT_NORMAL
    ).unwrap();
    let binary = write_embermesh_to_vec(&mesh).unwrap();

    // 2. Parse binary (simulating runtime loader)
    let loaded = read_embermesh(&binary).unwrap();

    // 3. Verify format and stride
    assert_eq!(loaded.format, FORMAT_UV | FORMAT_NORMAL);
    assert_eq!(loaded.stride, vertex_stride_packed(loaded.format as u32));
    assert_eq!(loaded.stride, 16); // 8 (pos) + 4 (uv) + 4 (normal)

    // 4. Verify vertex data is valid packed format
    assert_eq!(loaded.vertex_data.len(),
               loaded.vertex_count as usize * loaded.stride as usize);
}

#[test]
fn skinned_mesh_preserves_bone_data() {
    let mesh = convert_gltf(
        Path::new("tests/fixtures/meshes/cube_skinned.gltf"),
        FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED
    ).unwrap();

    assert_eq!(mesh.format & FORMAT_SKINNED, FORMAT_SKINNED);
    assert_eq!(mesh.stride, 24); // 8 + 4 + 4 + 4 (indices) + 4 (weights)

    // Verify bone indices are within valid range
    let stride = mesh.stride as usize;
    for i in 0..mesh.vertex_count as usize {
        let bone_offset = i * stride + 16; // After pos+uv+normal
        let bone_indices = &mesh.vertex_data[bone_offset..bone_offset+4];
        for &idx in bone_indices {
            assert!(idx < 2, "Bone index {} out of range for test mesh", idx);
        }
    }
}
```

#### Full Manifest Processing (`tests/integration/full_manifest.rs`)

```rust
#[test]
fn process_complete_manifest() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manifest_content = r#"
        [output]
        dir = "out/"

        [codegen]
        rust = "assets.rs"

        [meshes]
        cube = "meshes/cube.gltf"

        [textures]
        test = "textures/2x2_rgba.png"
    "#;

    // Setup test directory
    setup_test_project(temp_dir.path(), manifest_content);

    // Run build
    let result = ember_export::build(temp_dir.path().join("assets.toml"));
    assert!(result.is_ok(), "Build failed: {:?}", result.err());

    // Verify binary outputs exist and are non-empty
    let mesh_path = temp_dir.path().join("out/cube.embermesh");
    let tex_path = temp_dir.path().join("out/test.embertex");

    assert!(mesh_path.exists(), "Mesh output missing");
    assert!(tex_path.exists(), "Texture output missing");
    assert!(std::fs::metadata(&mesh_path).unwrap().len() > 24,
            "Mesh file too small (header only?)");

    // Verify generated Rust code
    let rust_path = temp_dir.path().join("assets.rs");
    assert!(rust_path.exists(), "Rust codegen output missing");

    let rust_code = std::fs::read_to_string(&rust_path).unwrap();
    assert!(rust_code.contains("static CUBE_MESH: &[u8]"));
    assert!(rust_code.contains("static TEST_TEX: &[u8]"));
    assert!(rust_code.contains("pub struct AssetPack"));
    assert!(rust_code.contains("pub fn load()"));
}
```

#### Codegen Verification (`tests/integration/codegen.rs`)

```rust
#[test]
fn generated_code_is_valid_rust() {
    let assets = ProcessedAssets {
        meshes: vec![("player".into(), PathBuf::from("player.embermesh"))].into_iter().collect(),
        textures: vec![("grass".into(), PathBuf::from("grass.embertex"))].into_iter().collect(),
        ..Default::default()
    };

    let code = generate_rust_module(&assets);

    // Parse with syn to verify valid Rust syntax
    syn::parse_file(&code).expect("Generated code should be valid Rust");

    // Verify expected content
    assert!(code.contains("// AUTO-GENERATED"));
    assert!(code.contains("include_bytes!"));
    assert!(code.contains("pub struct AssetPack"));
}

#[test]
fn generated_identifiers_are_valid() {
    // Test edge cases for identifier generation
    let test_cases = vec![
        ("simple", "SIMPLE"),
        ("with_underscore", "WITH_UNDERSCORE"),
        ("MixedCase", "MIXEDCASE"),
        ("123numeric", "_123NUMERIC"), // Leading digit gets underscore
    ];

    for (input, expected) in test_cases {
        let sanitized = sanitize_identifier(input);
        assert!(is_valid_identifier(&sanitized),
                "Sanitized '{}' to '{}' which is not a valid identifier", input, sanitized);
    }
}
```

### Error Case Tests

```rust
#[test]
fn error_missing_mesh_file() {
    let result = convert_gltf(Path::new("nonexistent.gltf"), 0);
    assert!(matches!(result, Err(ExportError::FileRead { .. })));
}

#[test]
fn error_mesh_missing_required_uvs() {
    let result = convert_gltf(
        Path::new("tests/fixtures/meshes/no_uvs.gltf"),
        FORMAT_UV // Request UVs that don't exist
    );
    assert!(matches!(
        result,
        Err(ExportError::MissingAttribute { attribute, .. }) if attribute == "TEXCOORD_0"
    ));
}

#[test]
fn error_too_many_vertices_for_u16_indices() {
    // Create or use a mesh with >65535 vertices
    let result = convert_gltf(
        Path::new("tests/fixtures/meshes/large.gltf"),
        0
    );
    // Should fail if mesh has >65535 vertices and uses indices
    if let Err(ExportError::TooManyVertices { count, .. }) = result {
        assert!(count > 65535);
    }
}

#[test]
fn error_invalid_format_string_in_manifest() {
    let toml = r#"
        [output]
        dir = "assets/"
        [meshes]
        test = { path = "test.gltf", format = "POS_INVALID_FORMAT" }
    "#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let errors = manifest.validate(Path::new(".")).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidFormat { .. })));
}
```

### Test Utilities (`tests/common/mod.rs`)

```rust
use emberware_z::graphics::packing::*;
use emberware_z::graphics::vertex::*;

/// Create a synthetic test mesh with the given format for round-trip testing.
/// Uses emberware-z packing functions to ensure compatibility.
pub fn create_test_mesh(format: u8) -> EmberMesh {
    let stride = vertex_stride_packed(format as u32);
    let vertex_count = 3u32; // Triangle

    let mut vertex_data = Vec::with_capacity((vertex_count * stride) as usize);

    for i in 0..vertex_count {
        // Position (always present) - simple triangle
        let pos = pack_position_f16(i as f32, 0.0, 0.0);
        vertex_data.extend_from_slice(bytemuck::bytes_of(&pos));

        if format & FORMAT_UV != 0 {
            let uv = pack_uv_unorm16(i as f32 / 3.0, 0.0);
            vertex_data.extend_from_slice(bytemuck::bytes_of(&uv));
        }

        if format & FORMAT_COLOR != 0 {
            let color = pack_color_unorm8(1.0, 1.0, 1.0);
            vertex_data.extend_from_slice(&color);
        }

        if format & FORMAT_NORMAL != 0 {
            let normal = pack_normal_octahedral(0.0, 1.0, 0.0);
            vertex_data.extend_from_slice(&normal.to_le_bytes());
        }

        if format & FORMAT_SKINNED != 0 {
            vertex_data.extend_from_slice(&[0u8, 0, 0, 0]); // Bone indices
            let weights = pack_bone_weights_unorm8([1.0, 0.0, 0.0, 0.0]);
            vertex_data.extend_from_slice(&weights);
        }
    }

    EmberMesh {
        vertex_count,
        index_count: 0,
        format,
        stride,
        vertex_data,
        index_data: None,
    }
}

/// Setup a test project directory with fixtures
pub fn setup_test_project(dir: &Path, manifest: &str) {
    std::fs::write(dir.join("assets.toml"), manifest).unwrap();

    // Copy fixture files
    let fixtures = Path::new("tests/fixtures");
    copy_dir_recursive(&fixtures.join("meshes"), &dir.join("meshes"));
    copy_dir_recursive(&fixtures.join("textures"), &dir.join("textures"));
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path);
        } else {
            std::fs::copy(entry.path(), dest_path).unwrap();
        }
    }
}
```

### Running Tests

```bash
# All tests
cargo test -p ember-export

# Unit tests only
cargo test -p ember-export --lib

# Integration tests only
cargo test -p ember-export --test '*'

# Specific test file
cargo test -p ember-export --test gltf_pipeline

# Update golden files (after intentional format changes)
cargo test -p ember-export update_golden -- --ignored

# Verbose output for debugging
cargo test -p ember-export -- --nocapture

# Run tests with all features
cargo test -p ember-export --all-features
```

### CI Integration

Add to `.github/workflows/test.yml`:

```yaml
test-ember-export:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Run ember-export tests
      run: cargo test -p ember-export --all-features
    - name: Verify golden files unchanged
      run: |
        cargo test -p ember-export golden -- --ignored
        git diff --exit-code tests/fixtures/golden/
```

### Example Updates

After the asset pipeline is implemented:
- Update `examples/cube/` to use asset pipeline
- Update `examples/textured-quad/` to use asset pipeline
- Create new `examples/asset-pipeline/` demonstrating full workflow
