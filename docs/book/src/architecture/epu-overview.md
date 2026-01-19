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
| Mipmaps | None (compute blur pyramid instead) |
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
                                                       - L_sharp  = Bounds + Features
                                                       - L_light0 = Bounds + EmissiveFeatures
                                                     - Blur pyramid from L_light0:
                                                       - L_light1, L_light2, ...
                                                     - Extract AmbientCube from most-blurred level

Main render (background + objects)         --->   [Render] Sample prebuilt results
                                                 - Background: EnvSharp[env_id]
                                                 - Specular:   EnvSharp OR EnvLight{level}[env_id]
                                                 - Diffuse:    AmbientCube[env_id] (6-direction)
```

---

## Dual-Map Flow

The EPU produces two directional radiance signals per environment:

| Signal | Contents | Used For |
|--------|----------|----------|
| `L_sharp(d)` | Bounds + all Features | Background + glossy reflections |
| `L_light0(d)` | Bounds + emissive Features (scaled by emissive field) | Blur pyramid source for lighting |

The blur pyramid is built **only from `L_light0`**, ensuring rough reflections and irradiance remain stable. The 4-bit emissive field (0-15) per layer controls how much each layer contributes to `L_light0`.

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
bits 116..113: emissive   (4)  - L_light0 contribution (0=none, 15=full)
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

## Emissive Control

Each layer has a 4-bit `emissive` field (0-15) that controls lighting contribution:

| Value | Contribution |
|-------|--------------|
| 0 | Decorative only (not in L_light0) |
| 1-14 | Scaled contribution (value/15) |
| 15 | Full emissive (100% to L_light0) |

This explicit control replaces the v1 policy where blend mode implied emissive behavior.

---

## Compute Pipeline

The EPU runtime maintains these outputs per `env_id`:

| Output | Type | Purpose |
|--------|------|---------|
| `EnvSharp[env_id]` | octahedral 2D array | Background + glossy reflections |
| `EnvLight0..k[env_id]` | octahedral 2D array | Blur pyramid levels |
| `AmbientCube[env_id]` | storage buffer | 6-direction diffuse irradiance |

### Frame Execution Order

1. Build draw list (each draw has `env_id`)
2. Deduplicate `env_id` list, cap to `MAX_ACTIVE_ENV_STATES_PER_FRAME`
3. Determine which `env_id`s are dirty (hash/time-dependent)
4. Dispatch compute passes:
   - Environment evaluation (build `EnvSharp` + `EnvLight0`)
   - Blur pyramid generation (Kawase blur passes)
   - Irradiance extraction (6-direction ambient cube)
5. Barrier: compute to render
6. Render background + objects (sampling by `env_id`)

---

## Render Integration

### Background Sampling

Sample `EnvSharp` using octahedral encoding for the view direction.

### Reflection Sampling

To avoid "double images," sample either the sharp or blurred representation based on roughness:

- **Low roughness** (< 0.15): sample `EnvSharp`
- **Higher roughness**: sample `EnvLight*` levels, interpolating between blur levels

### Ambient Lighting

The 6-direction ambient cube provides fast diffuse irradiance lookup:

```
ambient =
    cube.pos_x * max(n.x, 0) +
    cube.neg_x * max(-n.x, 0) +
    cube.pos_y * max(n.y, 0) +
    cube.neg_y * max(-n.y, 0) +
    cube.pos_z * max(n.z, 0) +
    cube.neg_z * max(-n.z, 0)
```

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
| `EPU_MAP_SIZE` | 64 |
| `EPU_BLUR_LEVELS` | 2 |

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
| Emissive | Implicit (ADD=emissive) | Explicit 4-bit (0-15) |
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
