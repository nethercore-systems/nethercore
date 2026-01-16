# Normal Mapping (ZX)

> Status: Implemented
> Last reviewed: 2026-01-14

> **Revision Note (2026-01-03)**: This spec has been validated against the codebase. Key corrections:
> - The normal-map opt-out flag is `FLAG_SKIP_NORMAL_MAP = 0x10000` (bit 16) — bits 8-15 are used by dither
> - Added `intel_tex_2::bc5` compression implementation details
> - Added tangent packing Rust/WGSL code
> - Clarified that BC5 is supported, but `nether.toml` does not currently expose per-texture format selection

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

Normal mapping is enabled/disabled by **what you draw**, not by special texture metadata:

- Mesh has tangents → shader will treat **slot 3** as a normal map (unless opted out)
- Mesh has no tangents → slot 3 is treated as a regular texture (e.g., extra matcap in Mode 1)

**Texture packing today**

`nether-cli pack` selects a single texture format for **all** textures via `game.compress_textures` in `nether.toml`:

- `compress_textures = false` → RGBA8
- `compress_textures = true` → BC7

Normal maps still work under this scheme because shaders only read the **R/G** channels and reconstruct Z.

**BC5 support (optional optimization)**

- Runtime supports BC5 (`TextureFormat::Bc5`) in `nethercore-zx/src/graphics/texture_manager.rs`.
- Tooling can emit BC5: see `compress_bc5` in `tools/nether-cli/src/pack/assets/texture.rs` and `tools/nether-export/src/texture.rs`.
- `nether.toml` does not currently expose per-texture format selection or auto-detection for BC5.

### Texture Format Enum Extension

```rust
// zx-common/src/formats/zx_data_pack/types.rs
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

BC5 compression is implemented (via `intel_tex_2`) in:

- `tools/nether-cli/src/pack/assets/texture.rs` (`compress_bc5`)
- `tools/nether-export/src/texture.rs` (`compress_bc5`, `convert_image_with_format`)

Both implementations:
- pad to 4×4 blocks with edge extension
- extract R/G channels from RGBA input
- compress to 16 bytes per 4×4 block (`Bc5RgUnorm`)

---

## Part 5: Shader Implementation

### Normal Map Sampling

```wgsl
// In fragment shader (modes 0, 2, 3)
fn sample_normal_map(uv: vec2<f32>, tbn: mat3x3<f32>, flags: u32) -> vec3<f32> {
    // Opt-out flag: when set, use vertex normals instead of sampling slot 3.
    if ((flags & FLAG_SKIP_NORMAL_MAP) != 0u) {
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
Bits 16+:   Extensions (currently: bit 16 = FLAG_SKIP_NORMAL_MAP)
```

### Normal Map Flag (Opt-out)

```rust
pub const FLAG_SKIP_NORMAL_MAP: u32 = 1 << 16;
```

FFI function:
```rust
fn skip_normal_map(skip: u32);
```

---

## Implementation Map (Where Things Live)

### Mesh + Vertex Formats

- `zx-common/src/packing.rs` — vertex format flags + stride calculations (tangent = bit 4; tangents require normals).
- `zx-common/src/formats/mesh.rs` — packed mesh format stores `format_flags` and packed vertex bytes.
- `nethercore-zx/src/graphics/vertex/` — wgpu vertex buffer layouts for each vertex format.

### Textures + Compression

- `zx-common/src/formats/zx_data_pack/types.rs` — `TextureFormat::{Rgba8, Bc7, Bc5}`.
- `nethercore-zx/src/graphics/texture_manager.rs` — uploads textures and maps formats to wgpu (`Bc5RgUnorm` for BC5).
- `tools/nether-cli/src/pack/assets/texture.rs` — BC7 + BC5 compression helpers; pack currently uses a single `TextureFormat` for all textures.
- `tools/nether-export/src/texture.rs` — converts images to `.ncztex` with explicit format (RGBA8/BC7/BC5).

### Shading + FFI

- `include/zx.rs` — `material_normal(texture)` and `skip_normal_map(skip)`.
- `nethercore-zx/src/ffi/material.rs` — binds slot 3 and toggles the skip flag.
- `nethercore-zx/src/graphics/unified_shading_state/shading_state.rs` — `FLAG_SKIP_NORMAL_MAP` (bit 16) and flag packing.
- `nethercore-zx/shaders/common/10_unpacking.wgsl` — `build_tbn()` + `sample_normal_map()` (slot 3).
- `nethercore-zx/shaders/mode1_matcap.wgsl` and `nethercore-zx/shaders/blinnphong_common.wgsl` — consume shading normals.

## Using Normal Maps (Today)

1. Use a mesh with UV + NORMAL + TANGENT (tangents require normals).
2. Bind a texture to slot 3 with `material_normal(texture)`.
3. (Optional) call `skip_normal_map(1)` to force vertex normals for a draw.

Notes:
- Slot 3 is the normal map slot across render modes.
- Mode 1 (Matcap) shares slot 3: if tangents are present it behaves as a normal map; otherwise it behaves like an extra matcap texture.
- Normal maps work with RGBA8 or BC7 textures; BC5 is recommended but not required.

## Budget Notes

- Tangents add **4 bytes per vertex** in the packed vertex format.
- Normal maps:
  - RGBA8: `width × height × 4` bytes
  - BC7/BC5: `ceil(width/4) × ceil(height/4) × 16` bytes

Rule of thumb: a 256×256 normal map is ~32KB in BC7/BC5.

## Future Work (Tooling)

- Expose per-texture format selection in `nether.toml` (e.g. `format = "bc5"`) and plumb through `tools/nether-cli`.
- (Optional) add naming-based normal-map detection (`*_normal`) to auto-select BC5 when compression is enabled.
