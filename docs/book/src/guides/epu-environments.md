# EPU Environments

The Environment Processing Unit (EPU) is ZX's GPU-driven procedural background and ambient environment system.

- It renders an infinite environment when you call `draw_epu()` after providing a config with `epu_set(config_ptr)` (packed 128-byte config).
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact FFI signatures and instruction encoding, see the [Environment (EPU) API](../api/epu.md).

For the full specification (opcode catalog, packing rules, WGSL details), see:
- `nethercore-design/specs/epu-feature-catalog.md`
- `nethercore/include/zx.rs` (canonical ABI docs)
- `nethercore/nethercore-zx/shaders/epu/` (shader sources)

---

## Quick Start

1. Create a packed EPU config: 8 × 128-bit instructions (stored as 16 `u64` values as 8 `[hi, lo]` pairs).
2. Call `epu_set(config_ptr)` near the start of `render()`, then call `draw_epu()` after your 3D geometry so the environment fills only background pixels.

**Determinism note:** The EPU has no host-managed time. To animate an environment, keep a deterministic `u8 phase` in your game state (e.g. `phase = phase.wrapping_add(1)` each frame), write it into the opcode parameter you want to drive (commonly `param_d`), and call `epu_set(...)` again with the updated config.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
// 8 x [hi, lo]
static ENV: [[u64; 2]; 8] = [
    [0, 0], [0, 0], [0, 0], [0, 0],
    [0, 0], [0, 0], [0, 0], [0, 0],
];

fn render() {
    unsafe {
        epu_set(ENV.as_ptr().cast()); // Set environment config
        // ... draw scene geometry
        draw_epu(); // Draw environment background
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static const uint64_t env_config[16] = {
    /* hi0, lo0, hi1, lo1, ... */
};

void render(void) {
    epu_set(env_config);  // Set environment config
    // ... draw scene geometry
    draw_epu();           // Draw environment background
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const env_config: [16]u64 = .{
    // hi0, lo0, hi1, lo1, ...
};

export fn render() void {
    epu_set(&env_config);  // Set environment config
    // ... draw scene geometry
    draw_epu();            // Draw environment background
}
```
{{#endtab}}

{{#endtabs}}

Reference presets and packing helpers:
- `nethercore/examples/3-inspectors/epu-showcase/src/presets.rs`
- `nethercore/examples/3-inspectors/epu-showcase/src/constants.rs`

---

## Architecture Overview

The EPU uses a 128-byte instruction-based configuration:

| Slot | Kind | Recommended Use |
|------|------|------------------|
| 0–3 | Enclosure | `RAMP` + optional enclosure ops (`0x02..0x07`) |
| 4–7 | Radiance | `DECAL`/`GRID`/`SCATTER`/`FLOW` + radiance ops (`0x0C..0x13`) |

**Enclosure** defines the low-frequency envelope and region weights (sky/walls/floor).

**Radiance** adds higher-frequency motifs (decals, grids, stars, clouds, etc.).

---

## Opcode Overview

| Opcode | Name | Best For | Notes |
|--------|------|----------|-------|
| 0x01 | RAMP | Base enclosure | Slot 0 recommended |
| 0x02 | SECTOR | Opening wedge / interior cues | Enclosure modifier |
| 0x03 | SILHOUETTE | Skyline / horizon cutout | Enclosure modifier |
| 0x04 | SPLIT | Geometric divisions | Enclosure |
| 0x08 | DECAL | Sun disks, signage, portals | Radiance |
| 0x09 | GRID | Panels, architectural lines | Radiance |
| 0x0A | SCATTER | Stars, dust, particles | Radiance |
| 0x0B | FLOW | Clouds, rain, caustics | Radiance |
| 0x12 | LOBE_RADIANCE | Sun glow, lamps, neon spill | Radiance |
| 0x13 | BAND_RADIANCE | Horizon bands / rings | Radiance |

---

## Authoring Workflow

- Start from a known-good preset (`epu-showcase`).
- Use the `epu-showcase` debug panel (F4) to iterate on one layer at a time (opcode + params).
- Copy the resulting packed 8-layer config into your game, call `epu_set(config_ptr)`, then call `draw_epu()`.

### Slot Conventions

| Slot | Kind | Recommended Use |
|------|------|------------------|
| 0 | Enclosure | `RAMP` (base enclosure + region weights) |
| 1-3 | Enclosure | `SECTOR` / `SILHOUETTE` / `SPLIT` / `CELL` / `PATCHES` / `APERTURE` |
| 4-7 | Radiance | `DECAL` / `GRID` / `SCATTER` / `FLOW` + radiance ops (`0x0C..0x13`) |

### meta5 Behavior

- `meta5` encodes `(domain_id << 3) | variant_id` for opcodes that support domain/variant selection.
- For opcodes that do not use domain/variant, set `meta5 = 0`.

---

## Split-Screen / Multiple Viewports

Call `viewport(...)` and then `draw_epu()` per viewport/pass where you want an environment background.

---

## See Also

- [EPU API Reference](../api/epu.md) - FFI signatures and instruction encoding
- [EPU Architecture Overview](../architecture/epu-overview.md) - Compute pipeline details
- [EPU Feature Catalog](../../../../../nethercore-design/specs/epu-feature-catalog.md)
