# EPU Environments

The Environment Processing Unit (EPU) is ZX’s procedural background + ambient environment system.

- It renders an infinite environment when you call `draw_env()`.
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact function signatures, packed layouts, and parameter docs, see the [Environment (EPU) API](../api/epu.md).

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

## Troubleshooting

- Shimmer in reflections: reduce `motion`/high-frequency detail and keep `phase` smooth.
- Too noisy: reduce `density`, `intensity`, or `contrast`-style controls.
- Overlay looks wrong: try a different `env_blend()` mode (Alpha/Add/Multiply/Screen).
- Aliasing: increase thickness/softness-style controls or reduce detail/warp.
