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
| Layer count | 8 instructions (4 Bounds + 4 Radiance) |
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
Call environment_index(...)+epu_set(...) during render()   --->   [Compute] EPU_Build(configs)
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

| Slot | Kind | Recommended Use |
|------|------|-----------------|
| 0-3 | Bounds | Any bounds opcode (`0x01..0x07`). Any bounds opcode can be first; each bounds layer outputs `RegionWeights` consumed by later feature/radiance layers. |
| 4-7 | Radiance | `DECAL` / `GRID` / `SCATTER` / `FLOW` + radiance ops (`0x0C..0x13`) |

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
| `0x08` | `DECAL` | Radiance | Sharp SDF shape (disk/ring/rect/line) |
| `0x09` | `GRID` | Radiance | Repeating lines/panels |
| `0x0A` | `SCATTER` | Radiance | Point field (stars/dust/bubbles) |
| `0x0B` | `FLOW` | Radiance | Animated noise/streaks/caustics |
| `0x0C` | `TRACE` | Radiance | Line/crack patterns |
| `0x0D` | `VEIL` | Radiance | Curtain/ribbon effects |
| `0x0E` | `ATMOSPHERE` | Radiance | Atmospheric absorption + scattering |
| `0x0F` | `PLANE` | Radiance | Ground/surface textures |
| `0x10` | `CELESTIAL` | Radiance | Moon/sun/planet bodies |
| `0x11` | `PORTAL` | Radiance | Portal/vortex effects |
| `0x12` | `LOBE` | Radiance | Region-masked directional glow |
| `0x13` | `BAND` | Radiance | Region-masked horizon band |

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

Implementation note: Internally, the runtime stores outputs in arrays indexed by `env_id`. Games can provide configs for one or more `env_id`s by setting `environment_index(env_id)` then calling `epu_set(config_ptr)`. Any `env_id` without an explicit config falls back to `env_id = 0`, and then to the built-in default config.

The EPU runtime maintains these outputs per `env_id`:

| Output | Type | Purpose |
|--------|------|---------|
| `EnvRadiance[env_id]` | mip-mapped octahedral 2D array | Background + roughness-based reflections |
| `SH9[env_id]` | storage buffer | L2 diffuse irradiance (spherical harmonics) |

### Frame Execution Order

1. Capture EPU draw requests (per viewport/pass) and determine active environment states
2. Deduplicate `env_id` list, cap to `MAX_ACTIVE_ENVS`
3. Determine which `env_id`s are dirty (hash)
4. Dispatch compute passes:
   - Environment evaluation (build `EnvRadiance` mip 0)
   - Mip pyramid generation (2x2 downsample chain)
   - Irradiance extraction (SH9)
5. Barrier: compute to render
6. Render background + objects (sampling by `env_id`)

---

## Render Integration

### Background Sampling

Render sky/background by evaluating the EPU directly per pixel (`L_hi(dir)`),
not by sampling `EnvRadiance`. This guarantees the sky is never limited by the
`EnvRadiance` base resolution.

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

### Ambient Lighting

Diffuse ambient is evaluated from SH9 coefficients at the shading normal `n`.

---

## Multiple Environments

The EPU supports multiple environments per frame through texture array indexing:

- All outputs are stored in array layers indexed by `env_id`
- Renderers pass `env_id` per draw/instance (internal)
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

- `state_hash`: Hash of the 128-byte config
- `valid`: Whether the cached entry has been initialized

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
