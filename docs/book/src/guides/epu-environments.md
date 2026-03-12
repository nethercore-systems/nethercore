# EPU Environments

The Environment Processing Unit (EPU) is ZX's GPU-driven procedural background and ambient environment system.

- It renders an infinite environment when you call `draw_epu()` after providing a config with `epu_set(config_ptr)` (packed 128-byte config).
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact FFI signatures and instruction encoding, see the [Environment (EPU) API](../api/epu.md).

For the full specification (opcode catalog, packing rules, WGSL details), see:
- `nethercore-design/specs/epu-feature-catalog.md`
- `nethercore/include/zx.rs` (canonical ABI docs)
- `nethercore/nethercore-zx/shaders/epu/` (shader sources)

Authoring note: the current runtime and Rust builder evaluate all 8 instructions sequentially in authored order. The real model is mixed `bounds` and `features`, with bounds rewriting region weights for later feature layers.

---

## Quick Start

1. Create a packed EPU config: 8 × 128-bit instructions (stored as 16 `u64` values as 8 `[hi, lo]` pairs).
2. Call `epu_set(config_ptr)` near the start of `render()`, then call `draw_epu()` after your 3D geometry so the environment fills only background pixels.

**Determinism note:** The EPU has no host-managed time. To animate an environment, keep a deterministic `u8 phase` in your game state (e.g. `phase = phase.wrapping_add(1)` each frame), write it into an opcode parameter that the authored variant actually consumes for motion (often `param_d`, but not universally), and call `epu_set(...)` again with the updated config.

**Authoring reality:** EPU work is procedural/generative world art. It is best at strong metaphorical place reads and ambient structural cues, not literal scene modeling or screen-space UI.

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
| 0–3 | Bounds | `RAMP` + optional bounds ops (`0x02..0x07`) |
| 4–7 | Features | `DECAL`/`GRID`/`SCATTER`/`FLOW` + feature ops (`0x0C..0x13`) |

**Bounds** defines the low-frequency envelope and region weights (sky/walls/floor).

**Features** add higher-frequency motifs (decals, grids, stars, clouds, etc.).

Note: the runtime and current Rust builder now evaluate all 8 instructions sequentially in authored order. Treat the table above as legacy shorthand only; the real model is `bounds` and `features` mixed across slots `0-7`, with bounds rewriting region weights for later feature layers.

---

## Opcode Overview

| Opcode | Name | Best For | Notes |
|--------|------|----------|-------|
| 0x01 | RAMP | Base bounds | Often used first to explicitly set `up/ceil/floor/softness`, but any bounds opcode can be layer 0. |
| 0x02 | SECTOR | Opening wedge / interior cues | Bounds modifier |
| 0x03 | SILHOUETTE | Skyline / horizon cutout | Bounds modifier |
| 0x04 | SPLIT | Geometric divisions | Bounds |
| 0x08 | DECAL | Sun disks, signage, portals | Feature |
| 0x09 | GRID | Panels, architectural lines | Feature |
| 0x0A | SCATTER | Stars, dust, particles | Feature |
| 0x0B | FLOW | Clouds, rain, caustics | Feature |
| 0x12 | LOBE | Sun glow, lamps, neon spill | Feature |
| 0x13 | BAND | Horizon bands / rings | Feature |

---

## Authoring Workflow

- Start from a known-good preset (`epu-showcase`).
- Use the `epu-showcase` debug panel (F4) to iterate on one layer at a time (opcode + params).
- Copy the resulting packed 8-layer config into your game, call `epu_set(config_ptr)`, then call `draw_epu()`.

### Slot Conventions

| Slots | Kind | Recommended Use |
|------|------|------------------|
| 0-7 | Mixed | Bounds and features are evaluated in authored order. Use early bounds to establish the envelope, then spend later slots on readable feature carriers unless you deliberately need a later bounds remap. |

### Bounds/Feature Cadence (Don\'t Waste Slots)

Bounds opcodes don\'t just draw color; they also rewrite the **region weights** (`SKY/WALLS/FLOOR`) that later feature opcodes use for masking.

- Avoid stacking multiple "plain bounds" layers back-to-back (e.g. `RAMP -> SILHOUETTE`) unless you immediately exploit the new regions with feature layers.
- Prefer a cadence like: `BOUNDS (define/reshape regions) -> FEATURES (use regions) -> BOUNDS (carve/retag: APERTURE/SPLIT) -> FEATURES (decorate + animate)`.
- If you insert a bounds opcode later in the 8-layer program, it only affects features **after** it (it cannot retroactively re-mask earlier features).

### Bounds vs Features in Practice

- Bounds are your scene envelope: horizon, enclosure, wedges, openings, and region weights.
- Features are where most readable world detail actually lives: signage, scan planes, rain curtains, stars, caustics, water reads, glow accents, and projection planes.
- If a preset keeps collapsing in direct view while the reflection still looks better, it often means the bounds are doing too much visual work and the later feature layers are not carrying enough world-readable structure.

### Variant-Specific Motion Reality

Do not assume `param_d` means smooth animation for every opcode or every variant.

- Reliable phase-driven movers in current practice: `FLOW`, `GRID`, `LOBE`, `DECAL`, `VEIL/RAIN_WALL`, `PLANE/WATER`, and `PORTAL/VORTEX`.
- `SCATTER` uses `param_d` as seed, so speed changes create shimmer/respawn rather than readable drift.
- `PORTAL/RECT` is a static SDF shape; it can read as a frame or backplate, but not as a self-animating volumetric field.
- `TRACE/LIGHTNING` is a static strike shape; use other layers for storm cadence.
- `APERTURE` is a bounds remapper, not a feature-layer detail primitive.
- `BAND` phase is azimuthal modulation, so it is better as support than as a general scrolling horizon effect.

If a scene depends on behavior the current opcode surface cannot supply, treat that as a real opcode-family or directionality gap. That is a legitimate signal to improve the engine/opcode surface instead of forcing more content churn into an impossible target.

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
