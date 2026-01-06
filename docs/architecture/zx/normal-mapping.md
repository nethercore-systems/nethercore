# Normal Map Implementation Plan

> Status: Implemented
> Last reviewed: 2026-01-06

> **Revision Note (2026-01-03)**: This spec has been validated against the codebase. Key corrections:
> - `FLAG_USE_NORMAL_MAP` changed from `0x100` (bit 8) to `0x10000` (bit 16) — bits 8-15 are used by dither
> - Added `intel_tex_2::bc5` compression implementation details
> - Added tangent packing Rust/WGSL code
> - Added channel detection logic for auto-BC5 selection

## Executive Summary

Adding normal maps to the ZX rendering system requires:
1. A new texture slot allocation strategy
2. A new vertex format flag for tangents
3. BC5 texture compression support
4. Shader updates for tangent-space normal mapping
5. Plugin updates for procgen and asset pipelines

---

## Part 1: Texture Slot Architecture Decision

### The Problem

You asked about two competing orderings:

**MR Workflow:**
- Option A: Albedo(0), MRE(1), Normal(2), unused(3)
- Option B: Albedo(0), Normal(1), MRE(2), unused(3)

**SS Workflow:**
- Option A: Albedo(0), SS(1), Normal(2), Specmap(3)
- Option B: Albedo(0), Normal(1), SS(2), Specmap(3)
- Option C: Albedo(0), SS(1), Specmap(2), Normal(3)

### Recommended Solution: Slot 3 = Normal (Consistent)

Your vertex attribute hierarchy insight is correct:
```
UVs > Vertex Colors > Normals > Tangents > Bones
```

Applying this to texture slots creates a consistent pattern:

| Slot | MR Workflow | SS Workflow | Hierarchy Principle |
|------|-------------|-------------|---------------------|
| 0 | Albedo | Albedo | Base appearance (UV-mapped) |
| 1 | MRE | SSE | Material properties (requires slot 0) |
| 2 | unused | Specmap | Additional modulation (optional) |
| 3 | Normal | Normal | **Always normal maps** (requires tangents) |

### Why Slot 3 for Normals?

1. **Mirrors vertex hierarchy**: Tangents come after normals/colors in vertex complexity
2. **Consistent across workflows**: Both MR and SS have normals in same slot
3. **Progressive complexity**: Games can omit slot 3 entirely for simpler materials
4. **Clean fallback**: No normal map = use vertex normals (already supported)

### Final Slot Definitions

```
Mode 2 (MR Blinn-Phong):
  Slot 0: Albedo texture
  Slot 1: MRE texture (R=Metallic, G=Roughness, B=Emissive)
  Slot 2: [unused/reserved]
  Slot 3: Normal map (optional)

Mode 3 (SS Blinn-Phong):
  Slot 0: Albedo texture
  Slot 1: SSE texture (R=SpecDamping, G=Shininess, B=Emissive)
  Slot 2: Specular map (RGB specular color multiplier)
  Slot 3: Normal map (optional)

Mode 1 (Matcap) - unchanged:
  Slot 0: Albedo texture
  Slots 1-3: Matcap textures (no normal mapping, image-based)

Mode 0 (Lambert):
  Slot 0: Albedo texture
  Slots 1-2: [unused]
  Slot 3: Normal map (optional, for detail)
```

### Mode 1 Matcap - Flexible Slot Usage

Mode 1 becomes flexible based on tangent vertex data presence:

**Without tangents (no normal map):**
```
Slot 0: Albedo OR Matcap 1
Slot 1: Matcap 1 or 2
Slot 2: Matcap 2 or 3
Slot 3: Matcap 3 or 4
```
Allows up to 4 matcaps (or 3 matcaps + albedo).

**With tangents (normal map enabled):**
```
Slot 0: Albedo
Slot 1: Matcap 1
Slot 2: Matcap 2
Slot 3: Normal map (perturbs matcap UV lookup)
```
Max 2 matcaps + albedo + normal.

**Normal map effect in matcap**: Perturbs the view-space normal used to sample the matcap texture, adding surface detail without changing lighting calculation.

---

## Part 2: Specular Workflow Refinement

### Current Mode 3 (SS) State

Slot 1 (SSE) reinterprets MRE channels:
- R: Specular damping (0=full spec, 1=no spec)
- G: Shininess (maps 0-1 → 1-256)
- B: Emissive intensity

**Problem**: No per-pixel specular color (only uniform via `material_specular_color()`).

### New Mode 3 with Slot 2 Specular Map

With normal maps in slot 3, slot 2 becomes available for a specular color map:

```
Slot 2: Specular Map (RGB)
  - R: Specular color R multiplier
  - G: Specular color G multiplier
  - B: Specular color B multiplier
  - A: [unused or additional shininess]
```

**Shader change**: Final specular = uniform_specular_color × texture_specular_color × lighting

This enables:
- Colored specular highlights varying across surface
- Per-pixel specular tinting
- More realistic metallic-like effects in SS workflow

---

## Part 3: Vertex Format - Adding Tangents

### Current Vertex Format Flags

```rust
pub const FORMAT_UV: u8 = 1;      // Bit 0
pub const FORMAT_COLOR: u8 = 2;   // Bit 1
pub const FORMAT_NORMAL: u8 = 4;  // Bit 2
pub const FORMAT_SKINNED: u8 = 8; // Bit 3
```

### New Tangent Flag

```rust
pub const FORMAT_TANGENT: u8 = 16; // Bit 4 (NEW)
```

This creates 32 vertex format permutations (up from 16).

### Tangent Representation

**Option A: Full TBN (tangent + bitangent)**
- 6 floats (24 bytes unpacked, ~8 bytes packed)
- Most accurate but expensive

**Option B: Tangent + handedness sign** (RECOMMENDED)
- 4 floats: tangent.xyz + sign for bitangent
- Bitangent = cross(normal, tangent) × sign
- ~5 bytes packed (octahedral tangent + 1 sign bit)

**Recommended packing:**
```
Packed tangent (4 bytes):
  - Bits 0-15: Octahedral tangent.xy (snorm16 × 2)
  - Bit 16: Handedness sign (0=positive, 1=negative)
  - Bits 17-31: [reserved]
```

**Rust implementation:**
```rust
/// Pack tangent vector + handedness into u32
/// Uses octahedral encoding for tangent direction, sign bit for handedness
pub fn pack_tangent(tangent: [f32; 3], handedness: f32) -> u32 {
    let oct = octahedral_encode([tangent[0], tangent[1], tangent[2]]);
    let sign_bit = if handedness < 0.0 { 1u32 << 16 } else { 0 };
    oct | sign_bit
}

/// Unpack tangent from u32 to (tangent_xyz, handedness_sign)
pub fn unpack_tangent(packed: u32) -> ([f32; 3], f32) {
    let oct_xy = packed & 0xFFFF;
    let tangent = octahedral_decode(oct_xy);
    let sign = if (packed & 0x10000) != 0 { -1.0 } else { 1.0 };
    (tangent, sign)
}
```

**WGSL implementation:**
```wgsl
fn unpack_tangent(packed: u32) -> vec4<f32> {
    let oct_x = f32(i32(packed & 0xFFFFu) - 32768) / 32767.0;
    let oct_y = f32(i32((packed >> 16u) & 0x7FFFu) - 16384) / 16383.0;
    let tangent = octahedral_decode(vec2<f32>(oct_x, oct_y));
    let sign = select(1.0, -1.0, (packed & 0x10000u) != 0u);
    return vec4<f32>(tangent, sign);
}
```

### Shader Location Assignment

```rust
const LOC_POS: u32 = 0;           // unchanged
const LOC_UV: u32 = 1;            // unchanged
const LOC_COLOR: u32 = 2;         // unchanged
const LOC_NORMAL: u32 = 3;        // unchanged
const LOC_BONE_INDICES: u32 = 4;  // unchanged
const LOC_BONE_WEIGHTS: u32 = 5;  // unchanged
const LOC_TANGENT: u32 = 6;       // NEW
```

### Dependency: Tangent Requires Normal

If `FORMAT_TANGENT` is set, `FORMAT_NORMAL` must also be set. Validation:
```rust
if (format & FORMAT_TANGENT != 0) && (format & FORMAT_NORMAL == 0) {
    panic!("Tangent format requires normal format");
}
```

This matches the hierarchy: can't have tangents without normals.

---

## Part 4: BC5 Compression for Normal Maps

### Why BC5?

| Format | BPP | Channels | Quality for Normals |
|--------|-----|----------|---------------------|
| RGBA8 | 32 | 4 | Perfect but huge |
| BC7 | 8 | 4 | Good but overkill (has alpha) |
| BC5 | 8 | 2 | **Optimal** (RG only, high quality) |

Normal maps only need X and Y; Z is reconstructed: `z = sqrt(1 - x² - y²)`

BC5 provides:
- Same compression ratio as BC7 (4:1)
- All bits dedicated to RG channels
- Better quality for normal data than BC7

### Implementation: Developer Experience

**Problem**: Users manually tagging textures in nether.toml kills DevEx.

**Solution: Channel-Based Auto-Detection + Slot Binding**

The import pipeline detects based on how the texture is bound at runtime:

```toml
[[assets.textures]]
id = "player"
path = "assets/player.png"          # 4-channel → BC7 (albedo/MRE)

[[assets.textures]]
id = "player_normal"
path = "assets/player_normal.png"   # 2 or 3-channel → BC5 (normal)
```

**Channel-based detection in `nether-export`:**
1. **2-channel texture**: Compress directly to BC5 (RG only)
2. **3-channel texture**: Discard B channel (reconstructed), compress to BC5
3. **4-channel texture**: BC7 (standard color/MRE texture)

**Why this works**: Normal maps only need X and Y. The B channel is mathematically derivable: `Z = sqrt(1 - X² - Y²)`. By detecting 2-3 channel input, we auto-select BC5 without any config.

**Shader stays simple**: Always receives BC5 (RG), always reconstructs Z. No branching for different normal formats.

**No manifest changes needed**: The existing `compress = true/false` flag continues to work:
- `compress = true` + 4-channel → BC7 (albedo, MRE, specular)
- `compress = true` + 2-3 channel → BC5 (normal maps)
- `compress = false` → RGBA8 (uncompressed)

This is fully backward compatible - existing projects work unchanged.

### Texture Format Enum Extension

```rust
// zx-common/src/formats/zx_data_pack.rs
pub enum TextureFormat {
    Rgba8,      // Uncompressed
    Bc7,        // Compressed color (albedo, MRE, specular)
    Bc5,        // Compressed 2-channel (normal maps)
}

impl TextureFormat {
    pub fn data_size(&self, width: u16, height: u16) -> usize {
        let w = width as usize;
        let h = height as usize;
        match self {
            TextureFormat::Rgba8 => w * h * 4,
            TextureFormat::Bc7 | TextureFormat::Bc5 => {
                // Both BC7 and BC5 use 16 bytes per 4x4 block
                let blocks_x = w.div_ceil(4);
                let blocks_y = h.div_ceil(4);
                blocks_x * blocks_y * 16
            }
        }
    }

    pub fn wgpu_format_name(&self) -> &'static str {
        match self {
            TextureFormat::Rgba8 => "Rgba8Unorm",
            TextureFormat::Bc7 => "Bc7RgbaUnorm",
            TextureFormat::Bc5 => "Bc5RgUnorm",
        }
    }
}
```

### BC5 Compression Implementation

The `intel_tex_2` crate (already in workspace) supports BC5 compression via `intel_tex_2::bc5`:

```rust
// tools/nether-cli/src/pack/mod.rs (and nether-export/src/texture.rs)
use intel_tex_2::bc5;

/// Compress RG8 pixels to BC5 format for normal maps
pub fn compress_bc5(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let w = width as usize;
    let h = height as usize;

    // Pad to multiple of 4 (BC5 requirement)
    let padded_width = (w + 3) & !3;
    let padded_height = (h + 3) & !3;
    let blocks_x = padded_width / 4;
    let blocks_y = padded_height / 4;

    // BC5 = 16 bytes per 4x4 block
    let mut output = vec![0u8; blocks_x * blocks_y * 16];

    // Pad source if needed (edge extension)
    let padded = if padded_width != w || padded_height != h {
        pad_rg_texture(pixels, w, h, padded_width, padded_height)
    } else {
        pixels.to_vec()
    };

    // Create RG surface for intel_tex_2
    let surface = intel_tex_2::RgSurface {
        width: padded_width as u32,
        height: padded_height as u32,
        stride: padded_width as u32 * 2,  // 2 bytes per pixel (RG)
        data: &padded,
    };

    bc5::compress_blocks_into(&surface, &mut output);
    Ok(output)
}
```

### Channel Detection for Auto-BC5

```rust
fn load_texture(id: &str, path: &Path, compress: bool) -> Result<PackedTexture> {
    let img = image::open(path)?;
    let channels = img.color().channel_count();
    let (width, height) = img.dimensions();

    let (format, data) = if !compress {
        // Uncompressed: always RGBA8
        let rgba = img.to_rgba8();
        (TextureFormat::Rgba8, rgba.into_raw())
    } else if channels <= 3 && is_normal_map_by_name(path) {
        // 2-3 channel + name hint = normal map = BC5
        let rg = extract_rg_channels(&img);
        let compressed = compress_bc5(&rg, width, height)?;
        (TextureFormat::Bc5, compressed)
    } else {
        // 4 channel or non-normal = BC7
        let rgba = img.to_rgba8();
        let compressed = compress_bc7(rgba.as_raw(), width, height)?;
        (TextureFormat::Bc7, compressed)
    };

    Ok(PackedTexture::with_format(id, width as u16, height as u16, format, data))
}

/// Detect normal maps by filename convention
fn is_normal_map_by_name(path: &Path) -> bool {
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    name.ends_with("_normal") || name.ends_with("_n") || name.contains("normal")
}
```

---

## Part 5: Shader Implementation

### Normal Map Sampling

```wgsl
// In fragment shader (modes 0, 2, 3)
fn sample_normal_map(uv: vec2<f32>, tbn: mat3x3<f32>) -> vec3<f32> {
    let flags = material.flags;

    // Check if normal map is bound (bit 16)
    if ((flags & 0x10000u) == 0u) {
        return tbn[2]; // Return vertex normal (TBN's Z column)
    }

    // Sample BC5 texture (RG channels)
    let normal_sample = textureSample(slot3, sampler_linear, uv).rg;

    // Reconstruct Z from unit sphere
    let xy = normal_sample * 2.0 - 1.0;
    let z = sqrt(max(0.0, 1.0 - dot(xy, xy)));
    let tangent_normal = vec3<f32>(xy, z);

    // Transform to world space
    return normalize(tbn * tangent_normal);
}
```

### TBN Matrix Construction

```wgsl
// Vertex shader output
struct VertexOutput {
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tangent: vec3<f32>,      // NEW
    @location(5) bitangent_sign: f32,     // NEW
    // ... existing fields
}

// Fragment shader TBN construction
fn build_tbn(input: VertexOutput) -> mat3x3<f32> {
    let N = normalize(input.normal);
    let T = normalize(input.tangent);
    let B = cross(N, T) * input.bitangent_sign;
    return mat3x3<f32>(T, B, N);
}
```

### Current Flag Layout

The flags field in `PackedUnifiedShadingState` is allocated as follows:
```
Bits 0-7:   Feature flags (skinning, filter, uniform overrides, matcap)
Bits 8-11:  FLAG_UNIFORM_ALPHA_MASK (dither transparency level)
Bits 12-13: FLAG_DITHER_OFFSET_X_MASK
Bits 14-15: FLAG_DITHER_OFFSET_Y_MASK
Bits 16+:   Available for new flags
```

### New Material Flag

```rust
const FLAG_USE_NORMAL_MAP: u32 = 0x10000; // Bit 16 (NOT bit 8 - that's used by dither)
```

FFI function:
```rust
fn use_normal_map(enabled: bool);
```

---

## Part 6: Implementation Steps

### Phase 1: Core Infrastructure

1. **Vertex format extension** (`zx-common/src/packing.rs`)
   - Add `FORMAT_TANGENT = 16`
   - Implement tangent packing (octahedral + sign)
   - Update stride calculations for 32 formats

2. **GPU vertex layouts** (`nethercore-zx/src/graphics/vertex.rs`)
   - Add LOC_TANGENT = 6
   - Generate all 32 vertex format permutations
   - Handle tangent attribute binding

3. **Mesh format update** (`zx-common/src/formats/mesh.rs`)
   - Support new format flag in `.nczxmesh`
   - Update mesh validation

### Phase 2: Texture Pipeline

4. **BC5 compression** (`tools/nether-export/src/texture.rs`)
   - Add BC5 compression via `intel_tex_2`
   - Implement naming convention detection
   - Add optional `format = "bc5"` in manifest

5. **Texture format enum** (`zx-common/src/formats/texture.rs`)
   - Add `TextureFormat::Bc5` variant
   - Update `bc5_size()` calculation

6. **Runtime loading** (`nethercore-zx/src/graphics/texture_manager.rs`)
   - Add `load_texture_bc5_internal()`
   - Handle RG format in wgpu

### Phase 3: Shaders

7. **Shader updates** (`nethercore-zx/shaders/`)
   - Update `common.wgsl`: Add slot3 binding, TBN structs
   - Update `blinnphong_common.wgsl`: Normal map sampling
   - Update mode shaders: Use TBN for lighting
   - Add `FLAG_USE_NORMAL_MAP` flag

8. **FFI bindings** (`nethercore-zx/src/ffi/material.rs`)
   - Add `material_normal(texture)` → binds to slot 3
   - Add `use_normal_map(enabled)` flag setter

### Phase 4: Documentation & Tools

9. **Documentation** (`docs/book/src/api/`)
   - Update texture API docs
   - Add normal mapping guide
   - Update material reference

10. **nether-cli updates** (`tools/nether-cli/`)
    - Handle BC5 format in build pipeline
    - Update asset validation

---

## Part 7: Plugin Updates Required

### nethercore-ai-plugins Affected Plugins:

| Plugin | Files to Update | Changes |
|--------|-----------------|---------|
| `zx-procgen` | `agents/asset-generator.md`, `skills/procedural-textures/` | Add normal map generation patterns |
| `zx-procgen` | `skills/asset-quality-tiers/references/texture-enhancements.md` | Add normal map quality tiers |
| `zx-dev` | `skills/zx-ffi-reference/references/ffi-api.md` | Document new FFI functions |
| `zx-dev` | `skills/project-templates/` | Update templates with tangent vertex format |
| `zx-game-design` | `skills/zx-constraints/references/resource-budgets.md` | Add normal map memory budgets |
| `sound-design` | None | N/A |
| `creative-direction` | `skills/art-vision/` | Add normal map guidelines to art direction |

### Specific Plugin Changes:

**zx-procgen** (highest priority):
- Add `generate-normal-map` skill or extend `generate-texture`
- Add height-to-normal conversion routine
- Add normal map synthesis patterns (procedural normals)
- Update quality reviewer for normal map validation

**zx-dev**:
- Update FFI cheat sheet with `material_normal()`, `use_normal_map()`
- Add tangent vertex format to format reference
- Update project scaffolding templates

**creative-direction**:
- Add normal map considerations to art direction skill
- Document when normal maps add value vs. waste memory

---

## Part 8: Memory Budget Considerations

### Normal Map Cost

Per texture:
- BC5: `(width × height) / 2` bytes (same as BC7)
- Example: 256×256 normal map = 32KB

Per vertex (tangent):
- Packed: 4 bytes per vertex
- Example: 1000-vertex mesh = 4KB additional

### Budget Recommendations

| Quality Tier | Normal Map Resolution | Vertex Budget |
|--------------|----------------------|---------------|
| Placeholder | None (vertex normals) | 0 |
| Temp | 64×64 (2KB) | +4B/vert |
| Final | 128×128 (8KB) | +4B/vert |
| Hero | 256×256 (32KB) | +4B/vert |

### Total Impact

Worst case (hero assets with normal maps everywhere):
- 10 hero normal maps: 320KB textures
- 50K vertices with tangents: 200KB vertex data
- **Total**: ~520KB additional memory

Well within ZX memory budget (16MB recommended).

---

## Critical Files to Modify

### Core Infrastructure
| File | Changes |
|------|---------|
| `nethercore/zx-common/src/packing.rs` | Add FORMAT_TANGENT, tangent packing functions, update stride calculations |
| `nethercore/zx-common/src/formats/texture.rs` | Add TextureFormat::Bc5, bc5_size() |
| `nethercore/zx-common/src/formats/mesh.rs` | Support new format flag |
| `nethercore/nethercore-zx/src/graphics/vertex.rs` | Add LOC_TANGENT, generate 32 format permutations |
| `nethercore/nethercore-zx/src/graphics/texture_manager.rs` | Add load_texture_bc5_internal() |
| `nethercore/nethercore-zx/src/ffi/material.rs` | Add material_normal(), use_normal_map() |

### Shaders
| File | Changes |
|------|---------|
| `nethercore/nethercore-zx/shaders/common.wgsl` | Add slot3 binding, TBN structs, FLAG_USE_NORMAL_MAP |
| `nethercore/nethercore-zx/shaders/blinnphong_common.wgsl` | Add sample_normal_map(), TBN construction |
| `nethercore/nethercore-zx/shaders/mode0_lambert.wgsl` | Integrate normal map sampling |
| `nethercore/nethercore-zx/shaders/mode1_matcap.wgsl` | Perturb matcap UV with normal map |
| `nethercore/nethercore-zx/shaders/mode2_mr.wgsl` | Integrate normal map sampling |
| `nethercore/nethercore-zx/shaders/mode3_ss.wgsl` | Integrate normal map sampling |

### Tools
| File | Changes |
|------|---------|
| `nethercore/tools/nether-export/src/texture.rs` | Add BC5 compression, channel detection |
| `nethercore/tools/nether-export/src/manifest.rs` | Add format field to texture config |

### Plugins (nethercore-ai-plugins)
| Plugin | Files |
|--------|-------|
| `zx-procgen` | agents/asset-generator.md, skills/procedural-textures/ |
| `zx-dev` | skills/zx-ffi-reference/references/ffi-api.md |
| `creative-direction` | skills/art-vision/ |

---

## Decisions Made

1. **BC5 Detection**: Channel-based auto-detection (2-3 channels → BC5, 4 channels → BC7). Shader always receives BC5, no complexity.

2. **Mode 1 Matcap**: Flexible - tangent presence determines if slot 3 is matcap or normal map. Max 2 matcaps with normals, max 4 matcaps without.

3. **Mode 0 Lambert**: Supports normal maps. Users who don't want it simply don't upload tangent vertex data.

4. **Tangent space only**: KISS - only tangent space normal maps supported. No height/bump map conversion.
