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
| Config size | 64 bytes per environment state |
| Layer count | 8 instructions (4 Bounds + 4 Features) |
| Cubemaps | None (fully procedural octahedral maps) |
| Mipmaps | None (compute blur pyramid instead) |
| Aesthetic | PS1/PS2-era palette-indexed colors, quantized params |

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
| `L_light0(d)` | Bounds + emissive Features | Blur pyramid source for lighting |

The blur pyramid is built **only from `L_light0`**, ensuring rough reflections and irradiance remain stable. Emissive Features (those using `ADD` blend mode) contribute to lighting energy after blur.

---

## Data Model

### PackedEnvironmentState (64 bytes)

Each environment is exactly **8 x 64-bit instructions**:

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

### Instruction Bit Layout (64-bit)

```
  63..60  opcode        (4)   Which algorithm to run
  59..58  region_mask   (2)   Feature mask: ALL / SKY / WALLS / FLOOR
  57..56  blend_mode    (2)   How to combine layer output
  55..48  color_index   (8)   Palette index (0..255)
  47..40  intensity     (8)   Opcode-specific (usually brightness)
  39..32  param_a       (8)   Opcode-specific
  31..24  param_b       (8)   Opcode-specific
  23..16  param_c       (8)   Opcode-specific
  15..0   direction     (16)  Octahedral-encoded direction (u8,u8)
```

### Opcodes

| Opcode | Name | Kind | Purpose |
|--------|------|------|---------|
| `0x0` | `NOP` | Any | Disable layer |
| `0x1` | `RAMP` | Bounds | Enclosure gradient (sky/walls/floor) |
| `0x2` | `LOBE` | Bounds | Directional glow (sun, lamp, neon spill) |
| `0x3` | `BAND` | Bounds | Horizon band / ring |
| `0x4` | `FOG` | Bounds | Atmospheric absorption |
| `0x5` | `DECAL` | Feature | Sharp SDF shape (disk/ring/rect/line) |
| `0x6` | `GRID` | Feature | Repeating lines/panels |
| `0x7` | `SCATTER` | Feature | Point field (stars/dust/bubbles) |
| `0x8` | `FLOW` | Feature | Animated noise/streaks/caustics |

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

- `state_hash`: Hash of the 64-byte config
- `time_dependent`: True if any layer uses animation

Update policy:

| Condition | Action |
|-----------|--------|
| Unused this frame | Skip |
| Used + time-dependent | Rebuild every frame |
| Used + static | Rebuild only when `state_hash` changes |

---

## Full Specification

For complete details including:

- WGSL shader implementations
- Rust API and builders
- Per-opcode parameter tables
- Example configurations
- Performance considerations

See the canonical [EPU RFC.md](../../../../../EPU%20RFC.md) specification document.
