# EPU Architecture Overview

The Environment Processing Unit (EPU) is Nethercore ZX's GPU-driven, fully procedural environment system. This page provides an architectural overview.

For the complete specification (opcode catalog, packing rules, and shader implementations), see:
- `nethercore-design/specs/epu-feature-catalog.md`
- `nethercore/include/zx.rs`
- `nethercore/nethercore-zx/shaders/epu/`

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
| Cubemaps | None (fully procedural octahedral maps) |
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
                                                    - Extract SH9 from a coarse mip (e.g. 16x16)

Main render (background + objects)          --->   [Render] Sample prebuilt results
                                                  - Background: EPU environment draw per viewport/pass
                                                  - Specular:   EnvRadiance sampled by roughness (LOD)
                                                  - Diffuse:    SH9 evaluated at the shading normal
```

---

## Radiance Flow

The EPU produces a single directional radiance signal per environment (`EnvRadiance`, mip 0).
From that radiance, the runtime builds a downsample mip pyramid used for continuous
roughness-based reflections, and extracts SH9 coefficients for diffuse ambient.

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
| `0x0B` | `FLOW` | Feature | Animated noise/streaks/caustics |
| `0x0C` | `TRACE` | Feature | Line/crack patterns |
| `0x0D` | `VEIL` | Feature | Curtain/ribbon effects |
| `0x0E` | `ATMOSPHERE` | Feature | Atmospheric absorption + scattering |
| `0x0F` | `PLANE` | Feature | Ground/surface textures |
| `0x10` | `CELESTIAL` | Feature | Moon/sun/planet bodies |
| `0x11` | `PORTAL` | Feature | Portal/vortex effects |
| `0x12` | `LOBE` | Feature | Region-masked directional glow |
| `0x13` | `BAND` | Feature | Region-masked horizon band |

### Blend Modes (8 modes)

| Value | Name | Formula |
|-------|------|---------|
| 0 | ADD | `dst + src * a` |
| 1 | MULTIPLY | `dst * mix(1, src, a)` |
| 2 | MAX | `max(dst, src * a)` |
| 3 | LERP | `mix(dst, src, a)` |
| 4 | SCREEN | `1 - (1-dst)*(1-src*a)` |
| 5 | HSV_MOD | HSV shift dst by src |
| 6 | MIN | `min(dst, src * a)` |
| 7 | OVERLAY | Photoshop-style overlay |

---

## Compute Pipeline

Implementation note: Internally, the runtime still stores outputs in array slots, but those slots are private implementation detail. Games use immediate-mode EPU setters (`epu_set(...)`, `epu_textures(...)`, `epu_asset(...)`), and each draw captures whichever EPU source is current at that moment.

The EPU runtime maintains these outputs per internal slot:

| Output | Type | Purpose |
|--------|------|---------|
| `EnvRadiance[slot]` | mip-mapped octahedral 2D array | Background + roughness-based reflections |
| `SH9[slot]` | storage buffer | L2 diffuse irradiance (spherical harmonics) |

### Frame Execution Order

1. Capture EPU draw requests (per viewport/pass) and determine active immediate-mode EPU sources
2. Resolve those sources to internal slots, cap to `MAX_ACTIVE_ENVS`
3. Determine which internal slots are dirty (hash or imported-face cache miss)
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

Imported face-texture sources render the background from `EnvRadiance` mip 0
after cube-to-octahedral conversion.

### Reflection Sampling

Sample `EnvRadiance` with a continuous roughness-to-LOD mapping across mip levels.
A common mapping is:

- `lod = (roughness^2) * (mip_count - 1)`

Then sample at that LOD (trilinear) or lerp between `floor(lod)` and `ceil(lod)`.

To avoid hard cutoffs while still preserving mirror-quality reflections, add a
high-frequency residual term that fades out with roughness:

- `alpha = roughness^2`
- `L_spec = L_lp + (1 - alpha) * (L_hi - L0)`
  - `L_hi` is procedural EPU evaluation at the reflection direction
  - `L0` is `EnvRadiance` sampled at mip 0
  - `L_lp` is `EnvRadiance` sampled at the roughness-derived LOD

Imported face-texture sources skip the procedural residual and use only the
octahedral mip chain.

### Ambient Lighting

Diffuse ambient is evaluated from SH9 coefficients at the shading normal `n`.

---

## Multiple Environments

The EPU supports multiple environments per frame through internal texture-array slot indexing:

- All outputs are stored in array layers indexed by an internal resolved slot
- Renderers pass that resolved slot per draw/instance (internal)
- No per-draw rebinding required

### Recommended Caps

| Constant | Typical Value |
|----------|---------------|
| `MAX_ENV_STATES` | 256 |
| `MAX_ACTIVE_ENVS` | 32 |
| `EPU_MAP_SIZE` | 128 (default; override via `NETHERCORE_EPU_MAP_SIZE`) |
| `EPU_MIN_MIP_SIZE` | 4 (default; override via `NETHERCORE_EPU_MIN_MIP_SIZE`) |
| `EPU_IRRAD_TARGET_SIZE` | 16 |

---

## Dirty-State Caching

For environments, the EPU tracks:

- `state_hash`: Hash of the 128-byte procedural config
- `valid`: Whether the cached entry has been initialized
- imported-face cache entries keyed by face-handle tuple or asset ID

Update policy:

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
| Emissive | Reserved (future use) |
| Alpha | 4-bit × 2 (per-color) |
| Parameters | 4 (+param_d) |

---

## Full Specification

For complete details including:

- WGSL shader implementations
- Per-opcode parameter tables
- Example configurations
- Performance considerations

See:
- [EPU Feature Catalog](../../../../../nethercore-design/specs/epu-feature-catalog.md)
- [ZX FFI Bindings](../../../../../nethercore/include/zx.rs)
