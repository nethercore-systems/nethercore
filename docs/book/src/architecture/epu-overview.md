# EPU Architecture Overview

The Environment Processing Unit (EPU) is Nethercore ZX's GPU-driven, fully procedural environment system. This page provides an architectural overview; for the complete specification, see [EPU RFC.md](../../../../../EPU%20RFC.md) in the project root.

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
| Layer count | 8 instructions (4 Bounds + 4 Features) |
| Instruction size | 128 bits (two u64 values) |
| Cubemaps | None (fully procedural octahedral maps) |
| Mipmaps | Yes (compute-generated downsample pyramid) |
| Color model | Direct RGB24 x 2 per layer |
| Aesthetic | PS1/PS2-era stylized, quantized params |

---

## System Diagram

```
CPU (immediate-mode)                              GPU
--------------------                              ---
Record draws with per-draw env_id          --->   [Compute] EPU_Build(active_env_list, time)
Collect + deduplicate env_id list                 - For each active env_id:
Cap to MAX_ACTIVE_ENV_STATES_PER_FRAME               - Evaluate microprogram into octahedral maps:
                                                       - EnvRadiance mip0 = Bounds + Features (radiance)
                                                     - Generate mip pyramid from EnvRadiance mip0:
                                                       - mip1..k via 2x2 downsample chain
                                                     - Extract SH9 from a coarse mip (e.g. 16x16)

Main render (background + objects)         --->   [Render] Sample prebuilt results
                                                 - Background: procedural L_hi(env_id, dir) (no texture sample)
                                                 - Specular:   EnvRadiance[env_id] (roughness -> LOD) + residual L_hi
                                                 - Diffuse:    SH9[env_id] (L2 spherical harmonics)
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

| Slot | Type | Recommended Use |
|------|------|-----------------|
| 0 | Bounds | `RAMP` enclosure + base colors |
| 1 | Bounds | `LOBE` (sun/neon spill) |
| 2 | Bounds | `BAND` (horizon ring) |
| 3 | Bounds | `FOG` (absorption/haze) |
| 4 | Feature | `DECAL` (sun disk, signage, portals) |
| 5 | Feature | `GRID` (panels, architectural lines) |
| 6 | Feature | `SCATTER` (stars, dust, windows) |
| 7 | Feature | `FLOW` (clouds, rain, caustics) |

### Instruction Bit Layout (128-bit)

Each instruction is packed as two `u64` values:

**High word (bits 127..64):**
```
bits 127..123: opcode     (5)  - 32 opcodes available
bits 122..120: region     (3)  - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
bits 119..117: blend      (3)  - 8 blend modes
bits 116..113: reserved   (4)
bit  112:      reserved   (1)  - Future flag
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
| `0x01` | `RAMP` | Bounds | Enclosure gradient (sky/walls/floor) |
| `0x02` | `LOBE` | Bounds | Directional glow (sun, lamp, neon spill) |
| `0x03` | `BAND` | Bounds | Horizon band / ring |
| `0x04` | `FOG` | Bounds | Atmospheric absorption |
| `0x05` | `DECAL` | Feature | Sharp SDF shape (disk/ring/rect/line) |
| `0x06` | `GRID` | Feature | Repeating lines/panels |
| `0x07` | `SCATTER` | Feature | Point field (stars/dust/bubbles) |
| `0x08` | `FLOW` | Feature | Animated noise/streaks/caustics |

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

The EPU runtime maintains these outputs per `env_id`:

| Output | Type | Purpose |
|--------|------|---------|
| `EnvRadiance[env_id]` | mip-mapped octahedral 2D array | Background + roughness-based reflections |
| `SH9[env_id]` | storage buffer | L2 diffuse irradiance (spherical harmonics) |

### Frame Execution Order

1. Build draw list (each draw has `env_id`)
2. Deduplicate `env_id` list, cap to `MAX_ACTIVE_ENV_STATES_PER_FRAME`
3. Determine which `env_id`s are dirty (hash/time-dependent)
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

## Multi-Environment Support

The EPU supports multiple environments per frame through texture array indexing:

- All outputs are stored in array layers indexed by `env_id`
- Renderers pass `env_id` per draw/instance
- No per-draw rebinding required

### Recommended Caps

| Constant | Typical Value |
|----------|---------------|
| `MAX_ENV_STATES` | 256 |
| `MAX_ACTIVE_ENV_STATES_PER_FRAME` | 32 |
| `EPU_MAP_SIZE` | 128 (default; override via `NETHERCORE_EPU_MAP_SIZE`) |
| `EPU_MIN_MIP_SIZE` | 4 (default; override via `NETHERCORE_EPU_MIN_MIP_SIZE`) |
| `EPU_IRRAD_TARGET_SIZE` | 16 |

---

## Dirty-State Caching

For static environments, the EPU tracks:

- `state_hash`: Hash of the 128-byte config
- `time_dependent`: True if any layer uses animation

Update policy:

| Condition | Action |
|-----------|--------|
| Unused this frame | Skip |
| Used + time-dependent | Rebuild every frame |
| Used + static | Rebuild only when `state_hash` changes |

---

## v2 Changes Summary

| Aspect | v1 | v2 |
|--------|----|----|
| Instruction size | 64-bit | 128-bit |
| Environment size | 64 bytes | 128 bytes |
| Opcode bits | 4-bit (16 opcodes) | 5-bit (32 opcodes) |
| Region | 2-bit enum | 3-bit mask (combinable) |
| Blend modes | 4 modes | 8 modes |
| Color | 8-bit palette index | RGB24 x 2 per layer |
| Emissive | Implicit (ADD=emissive) | Reserved (future use) |
| Alpha | None | 4-bit x 2 (per-color) |
| Parameters | 3 (a/b/c) | 4 (+param_d) |

---

## Full Specification

For complete details including:

- WGSL shader implementations
- Rust API and builders
- Per-opcode parameter tables
- Example configurations
- Performance considerations

See the canonical [EPU RFC.md](../../../../../EPU%20RFC.md) specification document.
