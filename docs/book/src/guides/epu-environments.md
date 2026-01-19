# EPU Environments

The Environment Processing Unit (EPU) is ZX's procedural background + ambient environment system.

- It renders an infinite environment when you call `draw_env()`.
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact function signatures, packed layouts, and parameter docs, see the [Environment (EPU) API](../api/epu.md).

For the underlying architecture (dual-map flow, compute pipeline, data model), see the [EPU Architecture Overview](../architecture/epu-overview.md).

---

## Quick Start

1. Configure the **base layer** (layer `0`) with an `env_*` function.
2. Optionally configure the **overlay layer** (layer `1`).
3. Set how the overlay composites with `env_blend()`.
4. Call `draw_env()` in your `render()` function.

Tips:
- Start from the inspector presets, then tweak `density/intensity/colors`.
- Animate with `phase` (wraps as u16) using `wrapping_add()` in your game code.

---

## Mode Overview

| Mode | Name | Best for | Common role |
|---:|---|---|---|
| 0 | Gradient | Sky/ground anchor + featured sun | Base |
| 1 | Cells | Particles (stars/snow/rain) and emissive tiles/lights | Overlay (sometimes base) |
| 2 | Lines | Floors/ceilings, scanlines, speed streaks | Overlay |
| 3 | Silhouette | Mountains/city/forest/waves on the horizon | Base |
| 4 | Nebula | Fog/clouds/aurora/ink/plasma/abstract space | Base or overlay |
| 5 | Room | Indoor “box” environments | Base |
| 6 | Veil | Drapes/pillars/shards/atmosphere | Overlay |
| 7 | Rings | Portals/tunnels/radar focal effects | Overlay (sometimes base) |

---

## Common Recipes

- Outdoor day/night: Gradient base + (optional) Cells or Nebula overlay
- Neon city: Silhouette (City skyline) base + Cells (Tiles/Lights) overlay (Screen/Add)
- Space: Nebula base + Cells (stars) overlay + Rings overlay
- Indoor sci-fi: Room base + Lines overlay + (optional) Veil overlay

---

## Rust Preset Recipes

For Rust users, the EPU provides ready-to-use preset factory functions. These implement common environment configurations using the EPU builder API.

### Available Presets

| Preset | Environment Type | Layers Used |
|--------|-----------------|-------------|
| `void_with_stars()` | Space / void | RAMP + SCATTER |
| `sunny_meadow()` | Outdoor daytime | RAMP + LOBE + DECAL + FLOW |
| `cyberpunk_alley()` | Urban night | RAMP + 2x LOBE + FOG + GRID + DECAL + FLOW + SCATTER |
| `underwater_cave()` | Underwater | RAMP + LOBE + FOG + FLOW + SCATTER |
| `space_station()` | Industrial interior | RAMP + LOBE + BAND + GRID + DECAL + SCATTER |
| `sunset_beach()` | Sunset scene | RAMP + LOBE + DECAL + FLOW |
| `haunted_forest()` | Dark forest | RAMP + LOBE + FOG + SCATTER |
| `lava_cave()` | Volcanic | RAMP + LOBE + FOG + FLOW + SCATTER |

### Usage

```rust
use nethercore_zx::graphics::epu::presets;

// Use a preset directly
let config = presets::sunny_meadow();

// Presets are re-exported at the module level
use nethercore_zx::graphics::epu::void_with_stars;
let stars = void_with_stars();
```

### Key Patterns

**Emissive vs Visual-Only Features:**
- Features with `EpuBlend::Add` are emissive (contribute to lighting)
- Features with `EpuBlend::Lerp` or `EpuBlend::Max` are visual-only

**Common Layer Combinations:**
- Sun/moon: `LOBE` (glow) + `DECAL` (disk, emissive)
- Fog/haze: `FOG` (uses MULTIPLY blend for absorption)
- Stars/particles: `SCATTER` (emissive)
- Clouds/caustics: `FLOW` (visual-only or emissive depending on intent)
- Industrial panels: `GRID` (emissive) + `BAND` (accent)

For complete API documentation, see the [Environment (EPU) API Reference](../api/epu.md#rust-presets).

---

## Troubleshooting

- Shimmer in reflections: reduce `motion`/high-frequency detail and keep `phase` smooth.
- Too noisy: reduce `density`, `intensity`, or `contrast`-style controls.
- Overlay looks wrong: try a different `env_blend()` mode (Alpha/Add/Multiply/Screen).
- Aliasing: increase thickness/softness-style controls or reduce detail/warp.
