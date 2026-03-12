# Coverage Matrix

This file tracks required method coverage across the final set.

Current audit below reflects the 12 presets presently wired in `src/presets.rs`.

## Current Summary

- Inventory coverage across the current 12 presets is `22/23` opcodes.
- `PATCHES` is now the only opcode family still absent in current preset code.
- Authored domains present in code: `DIRECT3D`, `AXIS_CYL`, `AXIS_POLAR`, `TANGENT_LOCAL`.
- Inventory coverage means "authored in code somewhere", not "proven shippable carrier quality." Check current shader metadata, benchmark history, and capability guidance before forcing a domain swap.
- `Combat Lab` closes the indoor proof-of-life gate on the current reviewed runs; the remaining proof-of-life blocker is outdoor quality.
- `Frozen Tundra` is the primary outdoor convergence target. `Storm Front` remains the weather follow-up. `Ocean Depths` is no longer the next outdoor bet.
- Newly closed in the current runtime/preset lane: `MOTTLE`, `ADVECT`, `SURFACE`, and `MASS` are all now authored in code.

## Proven Capability Summary

- Inventory coverage is high, but proven benchmark/showcase coverage is still narrow.
- Current positive proof-of-life:
  - `Projection Bay` benchmark passes
  - `Combat Lab` passes as the indoor showcase proof-of-life
- Current blocked proof-of-life:
  - `Open Horizon`
  - `Region Isolation`
  - `Transport Sweep`
  - `Front Mass`
  - `Frozen Bed`
  - outdoor showcase proof-of-life via `Frozen Tundra`
  - weather showcase proof-of-life via `Storm Front`

Treat this split as authoritative:

- `inventory coverage` answers "is the family authored somewhere?"
- `proven coverage` answers "has this surface class survived benchmark/showcase review at shipping quality?"

## Opcode Coverage

| Opcode | Required | Covered By | Gap | Status |
|--------|----------|------------|-----|--------|
| RAMP | yes | Ocean Depths, Astral Void, Sky Ruins | none in current code | covered |
| SECTOR | yes | Void Station, Combat Lab | only `BOX` is present | partial |
| SILHOUETTE | yes | Neon Metropolis, Sakura Shrine, Desert Mirage, Enchanted Grove, Sky Ruins, Frozen Tundra, Storm Front | only `INDUSTRIAL` is still absent | partial |
| SPLIT | yes | Frozen Tundra, Storm Front | only the newer `FACE` and `TIER` structural variants are in use so far | partial |
| CELL | yes | Hell Core | only `SHATTER` is present | partial |
| PATCHES | yes | none in current 12 | entire opcode absent | missing |
| APERTURE | yes | Void Station | only `ROUNDED_RECT` is present | partial |
| DECAL | yes | Neon Metropolis, Void Station, Combat Lab | none in current code | covered |
| GRID | yes | Void Station, Sky Ruins, Combat Lab | none in current code | covered |
| SCATTER | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Void Station, Desert Mirage, Enchanted Grove, Astral Void, Hell Core, Frozen Tundra, Storm Front | all current SCATTER variants are represented | covered |
| FLOW | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Desert Mirage, Enchanted Grove, Astral Void, Sky Ruins, Frozen Tundra, Storm Front | none in current code | covered |
| TRACE | yes | Hell Core, Storm Front | `LEAD_LINES` and `FILAMENTS` are still absent | partial |
| VEIL | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Enchanted Grove, Sky Ruins, Combat Lab, Storm Front | all current VEIL variants are represented | covered |
| ATMOSPHERE | yes | Sakura Shrine, Desert Mirage, Astral Void, Hell Core, Frozen Tundra, Storm Front | only `ALIEN` is still absent | partial |
| PLANE | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Void Station, Desert Mirage, Enchanted Grove, Hell Core, Sky Ruins, Combat Lab, Frozen Tundra, Storm Front | only `HEX` is still absent | partial |
| CELESTIAL | yes | Void Station, Astral Void | only `MOON` and `ECLIPSE` are present | partial |
| PORTAL | yes | Ocean Depths, Desert Mirage, Hell Core, Combat Lab | `CIRCLE`, `TEAR`, and `CRACK` are still absent | partial |
| LOBE | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Void Station, Desert Mirage, Enchanted Grove, Hell Core, Sky Ruins, Combat Lab, Frozen Tundra | none in current code | covered |
| BAND | yes | Sakura Shrine, Desert Mirage, Enchanted Grove, Astral Void, Hell Core, Sky Ruins, Frozen Tundra, Storm Front | none in current code | covered |
| MOTTLE | yes | Frozen Tundra, Storm Front | none in current code | covered |
| ADVECT | yes | Frozen Tundra, Storm Front | all current work is concentrated in the outdoor/weather lane | covered |
| SURFACE | yes | Frozen Tundra, benchmarks | currently only present in the frozen/material lane | covered |
| MASS | yes | Storm Front, benchmarks | current body-carrier coverage is narrow and still benchmark-blocked | covered |

## Domain Coverage

| Domain | Required | Covered By | Gap | Status |
|--------|----------|------------|-----|--------|
| DIRECT3D | yes | all 12 current presets | dominant authored domain; no gap in current code | covered |
| AXIS_CYL | yes | Neon Metropolis, Sakura Shrine, Ocean Depths, Enchanted Grove, Sky Ruins, Combat Lab, Storm Front | none in current code | covered |
| AXIS_POLAR | yes | Storm Front | none in current code | covered |
| TANGENT_LOCAL | yes | Ocean Depths, Desert Mirage, Hell Core, Combat Lab | none in current code | covered |

## Current Proof-Of-Life Status

| Preset | Status | Notes |
|--------|--------|-------|
| Combat Lab | indoor proof-of-life pass | protect from regression; keep the read world-integrated and non-HUD |
| Frozen Tundra | primary outdoor fix target | keep `SCATTER` and `BAND` support-only; put motion on proven movers such as `FLOW`, `PLANE/WATER`, or `LOBE` |
| Storm Front | weather proof-of-life blocker | needs readable horizon, water, and rain structure in direct view; treat `TRACE/LIGHTNING` as a static accent, not the cadence system |
| Ocean Depths | deprioritized for now | two full fix loops failed in different directions; do not keep burning loops here ahead of `Frozen Tundra` |

`PATCHES` is still a genuine inventory gap, but it is not the highest-value next step. Current loop priority is proven proof-of-life quality, not just filling empty checkboxes.

## Variant Checklist

### SECTOR

- [x] BOX
- [ ] TUNNEL
- [ ] CAVE

### SILHOUETTE

- [x] MOUNTAINS
- [x] CITY
- [x] FOREST
- [x] DUNES
- [x] WAVES
- [x] RUINS
- [ ] INDUSTRIAL
- [x] SPIRES

### SPLIT

- [ ] HALF
- [ ] WEDGE
- [ ] CORNER
- [ ] BANDS
- [ ] CROSS
- [ ] PRISM
- [x] TIER
- [x] FACE

### CELL

- [ ] GRID
- [ ] HEX
- [ ] VORONOI
- [ ] RADIAL
- [x] SHATTER
- [ ] BRICK

### PATCHES

- [ ] BLOBS
- [ ] ISLANDS
- [ ] DEBRIS
- [ ] MEMBRANE
- [ ] STATIC
- [ ] STREAKS

### APERTURE

- [ ] CIRCLE
- [ ] RECT
- [x] ROUNDED_RECT
- [ ] ARCH
- [ ] BARS
- [ ] MULTI
- [ ] IRREGULAR

### TRACE

- [x] LIGHTNING
- [x] CRACKS
- [ ] LEAD_LINES
- [ ] FILAMENTS

### VEIL

- [x] CURTAINS
- [x] PILLARS
- [x] LASER_BARS
- [x] RAIN_WALL
- [x] SHARDS

### SCATTER

- [x] STARS
- [x] DUST
- [x] WINDOWS
- [x] BUBBLES
- [x] EMBERS
- [x] RAIN
- [x] SNOW

### ATMOSPHERE

- [x] ABSORPTION
- [x] RAYLEIGH
- [x] MIE
- [x] FULL
- [ ] ALIEN

### PLANE

- [x] TILES
- [ ] HEX
- [x] STONE
- [x] SAND
- [x] WATER
- [x] GRATING
- [x] GRASS
- [x] PAVEMENT

### CELESTIAL

- [x] MOON
- [ ] SUN
- [ ] PLANET
- [ ] GAS_GIANT
- [ ] RINGED
- [ ] BINARY
- [x] ECLIPSE

### PORTAL

- [ ] CIRCLE
- [x] RECT
- [ ] TEAR
- [x] VORTEX
- [ ] CRACK
- [x] RIFT
