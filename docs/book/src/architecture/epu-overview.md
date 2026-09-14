# EPU Architecture Overview

The Environment Processing Unit (EPU) is Nethercore ZX's GPU-driven environment system with procedural and imported cube-face sources. This page provides an architectural overview.

For the complete specification (opcode catalog, packing rules, and shader implementations), see:
- `nethercore-zx/shaders/epu/`
- `include/zx/mod.rs` and `include/zx/epu.rs`
- `nethercore-zx/src/graphics/epu/layer.rs` (opcode and packing types)

For the current API reference and quick-start guide, see:
- [EPU Environments Guide](../guides/epu-environments.md)
- [Environment (EPU) API Reference](../api/epu.md)

---

## Introduction

The EPU provides a universal, stylized environment system that:

- Renders **backgrounds** (sky/walls/void) with strong, art-directable motifs
- Provides **lighting data** for objects (diffuse ambient + reflection color)

The system is designed around these hard constraints:

| Constraint | Value |
|------------|-------|
| Config size | 128 bytes per environment state |
| Layer count | 8 sequential instructions |
| Instruction size | 128 bits (two u64 values) |
| Sources | Procedural layers or imported cube-face textures/assets |
| Mipmaps | Yes (compute-generated downsample pyramid) |
| Color model | Direct RGB24 x 2 per layer |
| Aesthetic | PS1/PS2-era stylized, quantized params |

---

## System Diagram

```
CPU (game)                                         GPU
---------                                         ---
Call epu_set(...), epu_textures(...), or epu_asset(...)   --->   [Compute] EPU_Build(configs/imports)
Call draw_epu() to request a background draw            - Evaluate 8-layer microprogram into EnvRadiance (mip 0)
Capture (viewport, pass) draw requests                  - Generate mip pyramid from EnvRadiance mip 0
                                                    - Extract SH9 from source radiance (mip 0)

Main render (background + objects)          --->   [Render] Sample prebuilt results
                                                  - Background: EPU environment draw per viewport/pass
                                                  - Specular:   View-dependent Blinn-Phong radiance integral
                                                  - Diffuse:    SH9 evaluated at the shading normal
```

---

## Radiance Flow

The EPU produces a single directional radiance signal per environment (`EnvRadiance`, mip 0).
SH9 diffuse coefficients are extracted from source radiance, independently of
material roughness. Specular integrates radiance with the material’s Blinn–Phong
lobe. A projection-space downsample pyramid is not itself that convolution.

Procedural mip 0 averages four evaluations inside each octahedral texel with normalized solid-angle weights. The background evaluates the same program directly at its pixel direction, so source-cache filtering can differ from background detail. Reflections sample source mip 0, not roughness-selected downsample mips; SH9 extraction also reads source mip 0.

---

## Data Model

### PackedEnvironmentState (128 bytes)

Each environment is exactly **8 x 128-bit instructions**:

| Slots | Kind | Recommended Use |
|------|------|-----------------|
| 0-7 | Mixed | Author layers in the order you want them evaluated. Bounds rewrite `RegionWeights`; later feature layers consume the current regions. A common cadence is `BOUNDS -> FEATURES -> BOUNDS -> FEATURES`. |

Implementation note: in the shaders, bounds opcodes return `(sample, regions)`. Dispatch updates `regions` after every bounds layer, and feature layers apply region masking using the current regions.

### Instruction Bit Layout (128-bit)

Each instruction is packed as two `u64` values:

**High word (bits 127..64):**
```
bits 127..123: opcode     (5)  - 32 opcodes available
bits 122..120: region     (3)  - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
bits 119..117: blend      (3)  - 8 blend modes
bits 116..112: meta5      (5)  - (domain_id<<3)|variant_id; use 0 when unused
bits 111..88:  color_a    (24) - RGB24 primary color
bits 87..64:   color_b    (24) - RGB24 secondary color
```

**Low word (bits 63..0):**
```
bits 63..56:   intensity  (8)  - Layer brightness
bits 55..48:   param_a    (8)  - Opcode-specific
bits 47..40:   param_b    (8)  - Opcode-specific
bits 39..32:   param_c    (8)  - Opcode-specific
bits 31..24:   param_d    (8)  - Opcode-specific
bits 23..8:    direction  (16) - Octahedral-encoded direction (u8,u8)
bits 7..4:     alpha_a    (4)  - color_a alpha (0-15)
bits 3..0:     alpha_b    (4)  - color_b alpha (0-15)
```

### Opcodes

| Opcode | Name | Kind | Purpose |
|--------|------|------|---------|
| `0x00` | `NOP` | Any | Disable layer |
| `0x01` | `RAMP` | Bounds | Bounds gradient (sky/walls/floor) |
| `0x02` | `SECTOR` | Bounds | Azimuthal opening wedge modifier |
| `0x03` | `SILHOUETTE` | Bounds | Skyline/horizon cutout modifier |
| `0x04` | `SPLIT` | Bounds | Geometric divisions |
| `0x05` | `CELL` | Bounds | Voronoi/mosaic cells |
| `0x06` | `PATCHES` | Bounds | Noise patches |
| `0x07` | `APERTURE` | Bounds | Shaped opening/viewport |
| `0x08` | `DECAL` | Feature | Sharp SDF shape (disk/ring/rect/line) |
| `0x09` | `GRID` | Feature | Repeating lines/panels |
| `0x0A` | `SCATTER` | Feature | Point field (stars/dust/bubbles) |
| `0x0B` | `FLOW` | Feature | Looping noise/streaks/caustic patterns |
| `0x0C` | `TRACE` | Feature | Line/crack patterns |
| `0x0D` | `VEIL` | Feature | Curtain/ribbon effects |
| `0x0E` | `ATMOSPHERE` | Feature | Directional gradient/halo tint; no geometry fog or world simulation |
| `0x0F` | `PLANE` | Feature | Ground/surface textures |
| `0x10` | `CELESTIAL` | Feature | Moon/sun/planet bodies |
| `0x11` | `PORTAL` | Feature | Portal/vortex effects |
| `0x12` | `LOBE` | Feature | Region-masked directional glow |
| `0x13` | `BAND` | Feature | Region-masked horizon band |
| `0x14` | `MOTTLE` | Feature | Abstract texture breakup / base variation |
| `0x15` | `ADVECT` | Feature | Broad transport / mass motion carrier |
| `0x16` | `SURFACE` | Feature | Broad material / surface response carrier |
| `0x17` | `MASS` | Feature | Broad scene-owning body carrier |
| `0x18` | `SCATTER_PHASED` | Feature | Fixed points with independent guest-phase brightness |

### Blend Modes (8 modes)

`src` and `a` are opcode output RGB and weight. `a` and every blend result are clamped to `0..1` (RGB component-wise); these formulas omit the shared outer clamp.

| Value | Name | Formula |
|-------|------|---------|
| 0 | ADD | `dst + src * a` |
| 1 | MULTIPLY | `dst * mix(1, src, a)` |
| 2 | MAX | `max(dst, src * a)` |
| 3 | LERP | `mix(dst, src, a)` |
| 4 | SCREEN | `1 - (1-dst)*(1-src*a)` |
| 5 | HSV_MOD (legacy identifier) | RGB Offset: `clamp(dst + (src - 0.5) * clamp(a, 0, 1) * 2, 0, 1)`, per RGB component; not HSV modulation. Source 0.5 is neutral. |
| 6 | MIN | `min(dst, mix(1, src, a))` |
| 7 | OVERLAY | Photoshop-style overlay |

---

## Compute Pipeline

Implementation note: Internally, the runtime still stores outputs in array slots, but those slots are private implementation detail. Games use immediate-mode EPU setters (`epu_set(...)`, `epu_textures(...)`, `epu_asset(...)`), and each draw captures whichever EPU source is current at that moment.

The EPU runtime maintains these outputs per internal slot:

| Output | Type | Purpose |
|--------|------|---------|
| `EnvRadiance[slot]` | mip-mapped octahedral 2D array | Source mip 0 for specular integration and SH9 extraction; imported-background fallback |
| `SH9[slot]` | storage buffer | L2 diffuse irradiance (spherical harmonics) |

### Frame Execution Order

1. Capture EPU draw requests (per viewport/pass) and determine active immediate-mode EPU sources
2. Resolve those sources to internal slots, cap to `MAX_ACTIVE_ENVS`
3. Determine which procedural slots are dirty by configuration hash; submitted imported sources follow the import path without equivalent dirty-config filtering
4. Dispatch compute passes:
   - Environment evaluation (build `EnvRadiance` mip 0)
   - Imported cube-face conversion when needed
   - Mip pyramid generation (2x2 downsample chain)
   - Irradiance extraction (SH9)
5. Barrier: compute to render
6. Render background + objects (sampling by resolved internal slot)

---

## Render Integration

### Background Sampling

Procedural EPU sources render the background by evaluating the EPU directly per
pixel (`L_hi(dir)`), not by sampling `EnvRadiance`. This guarantees the sky is
never limited by the `EnvRadiance` base resolution.

Imported face-texture sources sample the stored cube faces directly for the
background, with octahedral source radiance as the fallback when faces are unavailable.

### Reflection Sampling

Environment specular uses the same half-vector Blinn–Phong equation as direct
lighting. For normal `N`, view direction `V`, incoming direction `L`, and
`H = normalize(V + L)`, integrate:

`radiance(L) * specular_color * C(s) * max(dot(N,H),0)^s * max(dot(N,L),0)`

Here `C(s) = 0.0397436*s + 0.0856832`. Mode 2 maps roughness to
`s = 1 + 255*(1-roughness)`; Mode 3 uses `s = 1 + 255*value` for normalized shininess `value` in `0..1`.
Material specular color is applied once. There is no additional split-sum Fresnel
fit, shell suppression, or procedural high-frequency residual gain.

The current implementation uses deterministic half-vector quadrature over source
radiance. Both the actual view direction and shading normal matter; replacing
this with an axial power lobe around the mirror direction is not equivalent.
Source-cache discretization and finite quadrature limit accuracy; this does not
promise exact mirrors or arbitrary-frequency fidelity. The source-filter repair,
finite-case numerical acceptance, and deferred performance gate are tracked in
the repository’s `docs/plans/epu-progress.md`; this description is not a claim
that those gates have passed.

### Ambient Lighting

Diffuse ambient is evaluated from SH9 coefficients at the shading normal `n`.

Extraction integrates every source mip-0 texel with octahedral solid-angle weights, normalized to `4*pi`, then applies the Lambertian band factors `pi`, `2*pi/3`, and `pi/4`. Sampling reconstructs L2 irradiance, divides by `pi`, and clamps negative components. This is low-order diffuse approximation, not specular-prefiltered lighting or an exact arbitrary-frequency hemispherical integral.

---

## Multiple Environments

The EPU supports multiple environments per frame through internal texture-array slot indexing:

- All outputs are stored in array layers indexed by an internal resolved slot
- Renderers pass that resolved slot per draw/instance (internal)
- No per-draw rebinding required

### Current Runtime Limits and Defaults

| Constant | Typical Value |
|----------|---------------|
| `MAX_ENV_STATES` | 256 |
| `MAX_ACTIVE_ENVS` | 32 |
| `EPU_MAP_SIZE` | 128 (default; override via `NETHERCORE_EPU_MAP_SIZE`) |
| `EPU_MIN_MIP_SIZE` | 4 (default; override via `NETHERCORE_EPU_MIN_MIP_SIZE`) |

---

## Dirty-State Caching

For procedural environments, the EPU tracks:

- `state_hash`: Hash of the 128-byte procedural config
- `valid`: Whether the cached entry has been initialized

Imported face-handle/asset resolution is separate from this procedural state cache.
`EpuRuntime::build_imported_envs` processes each submitted import; it does not
apply the procedural unchanged-config skip. Do not assume imported GPU rebuilds
are eliminated merely because a texture handle or asset ID is reused.

Procedural update policy:

| Condition | Action |
|-----------|--------|
| Unused this frame | Skip |
| Used + unchanged | Skip |
| Used + changed | Rebuild, then update `state_hash` |

---

## Format Summary

| Aspect | Value |
|--------|----|
| Instruction size | 128-bit |
| Environment size | 128 bytes |
| Opcode bits | 5-bit (32 opcodes) |
| Region | 3-bit mask (combinable) |
| Blend modes | 8 modes |
| Color | RGB24 × 2 per layer |
| Alpha | 4-bit × 2 (per-color) |
| Parameters | 4 bytes (`param_a` through `param_d`) |

---

## Full Specification

For complete details including:

- WGSL shader implementations
- Per-opcode parameter tables
- Example configurations
- Performance considerations

See:
- Opcode implementation and parameter decoding: `nethercore-zx/shaders/epu/` in the source checkout.
- Guest FFI declarations: `include/zx/mod.rs` and `include/zx/epu.rs` in the source checkout.
