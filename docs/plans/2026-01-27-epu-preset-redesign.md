# EPU Showcase Preset Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign all 24 EPU showcase presets to have distinct spatial identities with proper bounds/regions/features architecture, and fix opcode naming.

**Architecture:** Each preset gets a two-phase treatment: (1) design the environment concept with proper bounds-first layer ordering, region-specific targeting, and meaningful colors, then (2) write the actual opcode arrays. Bounds (opcodes 0x01-0x07) define spatial enclosures in layers 1-2, features (0x08-0x13) add visual detail in layers 3-7. Layer 0 is always RAMP.

**Tech Stack:** Rust (const arrays), EPU 128-bit instruction format via `hi()`/`hi_meta()`/`lo()` helpers from `constants.rs`

---

## Reference: EPU System Quick Guide

This section is for the implementing engineer. Read this before touching any preset.

### 128-bit Layer Format

Each layer = `[hi_word: u64, lo_word: u64]`.

**hi word** (built with `hi()` or `hi_meta()`):
- `opcode` (5 bits) — which algorithm
- `region` (3 bits) — SKY=0b100, WALLS=0b010, FLOOR=0b001, ALL=0b111
- `blend` (3 bits) — ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
- `meta5` (5 bits) — `(domain_id << 3) | variant_id`
- `color_a` (24 bits) — primary RGB
- `color_b` (24 bits) — secondary RGB

**lo word** (built with `lo()`):
- `intensity` (8 bits) — layer brightness (0-255)
- `param_a` through `param_d` (8 bits each) — opcode-specific
- `direction` (16 bits) — octahedral encoded direction
- `alpha_a`, `alpha_b` (4 bits each) — color alphas (0-15)

### Opcodes

**Bounds (0x01-0x07)** — define spatial enclosures:
| Opcode | Const | Variants |
|--------|-------|----------|
| 0x01 RAMP | `OP_RAMP` | none (param_a/b/c=wall RGB, param_d=threshold) |
| 0x02 SECTOR | `OP_SECTOR` | BOX=0, TUNNEL=1, CAVE=2 |
| 0x03 SILHOUETTE | `OP_SILHOUETTE` | MOUNTAINS=0, CITY=1, FOREST=2, DUNES=3, WAVES=4, RUINS=5, INDUSTRIAL=6, SPIRES=7 |
| 0x04 SPLIT | `OP_SPLIT` | HALF=0, WEDGE=1, CORNER=2, BANDS=3, CROSS=4, PRISM=5 |
| 0x05 CELL | `OP_CELL` | GRID=0, HEX=1, VORONOI=2, RADIAL=3, SHATTER=4, BRICK=5 |
| 0x06 PATCHES | `OP_PATCHES` | BLOBS=0, ISLANDS=1, DEBRIS=2, MEMBRANE=3, STATIC=4, STREAKS=5 |
| 0x07 APERTURE | `OP_APERTURE` | CIRCLE=0, RECT=1, ROUNDED_RECT=2, ARCH=3, BARS=4, MULTI=5, IRREGULAR=6 |

**Features (0x08-0x13)** — visual effects layered on bounded regions:
| Opcode | Const | Variants |
|--------|-------|----------|
| 0x08 DECAL | `OP_DECAL` | (SDF shapes) |
| 0x09 GRID | `OP_GRID` | (repeating lines) |
| 0x0A SCATTER | `OP_SCATTER` | STARS=0, DUST=1, WINDOWS=2, BUBBLES=3, EMBERS=4, RAIN=5, SNOW=6 |
| 0x0B FLOW | `OP_FLOW` | (animated noise) |
| 0x0C TRACE | `OP_TRACE` | LIGHTNING=0, CRACKS=1, LEAD_LINES=2, FILAMENTS=3 |
| 0x0D VEIL | `OP_VEIL` | CURTAINS=0, PILLARS=1, LASER_BARS=2, RAIN_WALL=3, SHARDS=4 |
| 0x0E ATMOSPHERE | `OP_ATMOSPHERE` | ABSORPTION=0, RAYLEIGH=1, MIE=2, FULL=3, ALIEN=4 |
| 0x0F PLANE | `OP_PLANE` | TILES=0, HEX=1, STONE=2, SAND=3, WATER=4, GRATING=5, GRASS=6, PAVEMENT=7 |
| 0x10 CELESTIAL | `OP_CELESTIAL` | MOON=0, SUN=1, PLANET=2, GAS_GIANT=3, RINGED=4, BINARY=5, ECLIPSE=6 |
| 0x11 PORTAL | `OP_PORTAL` | CIRCLE=0, RECT=1, TEAR=2, VORTEX=3, CRACK=4, RIFT=5 |
| 0x12 LOBE | `OP_LOBE` | (directional glow) |
| 0x13 BAND | `OP_BAND` | (horizon band) |

### Domains (for `hi_meta()`)
- `DOMAIN_DIRECT3D` (0) — spherical/infinite 3D
- `DOMAIN_AXIS_CYL` (1) — cylindrical wrap (columns, rain)
- `DOMAIN_AXIS_POLAR` (2) — radial/spoke patterns
- `DOMAIN_TANGENT_LOCAL` (3) — local tangent plane (portals, decals)

### Thresholds (RAMP param_d)
- `THRESH_BALANCED` (0xA5) — equal sky/wall/floor
- `THRESH_INTERIOR` (0xC3) — more walls, less sky
- `THRESH_OPEN` (0x96) — more sky
- `THRESH_VAST` (0x87) — almost all sky+floor
- `THRESH_SEMI` (0xB4) — semi-enclosed

### Direction Constants
- `DIR_UP` (0x80FF), `DIR_DOWN` (0x8000), `DIR_SUN` (0xC0A0), `DIR_SUNSET` (0xC190)

### Structural Rules
1. **L0 = RAMP always**
2. **Bounds in L1-L2 (sometimes L3)** — spatial enclosure first
3. **No black-on-MULTIPLY** — if using BLEND_MULTIPLY, colors must be nonzero and meaningful (warm tints, colored filters). Use LERP/SCREEN/ADD for dark effects.
4. **Region-specific targeting** — REGION_ALL only for RAMP (L0), ATMOSPHERE, and truly omnipresent. Bounds target their logical region. Features target where they make sense.
5. **NOP is fine** — 5 good layers beats 8 muddy ones. Use `NOP_LAYER` for empty slots.

---

## Task 0: Fix Opcode Naming

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/lib.rs:237-238`

**Step 1: Edit opcode_name()**

Change line 237-238 from:
```rust
        0x12 => b"LOBE_RADIANCE",
        0x13 => b"BAND_RADIANCE",
```
To:
```rust
        0x12 => b"LOBE",
        0x13 => b"BAND",
```

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`
Expected: compiles without errors

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/lib.rs
git commit -m "fix: rename LOBE_RADIANCE/BAND_RADIANCE to LOBE/BAND in opcode_name()"
```

---

## Task 1: Redesign Presets 1-4 (set_01_04.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_01_04.rs`
- Reference: `examples/3-inspectors/epu-showcase/src/constants.rs`

### Preset 1: "Neon Metropolis" — Cyberpunk urban alley

**Design concept:** A narrow rain-soaked alley between towering neon-lit buildings. Tunnel-shaped enclosure with city skyline. Cyan grid lines on walls, yellow window lights, magenta laser bars cutting through misty air, rain streaming down.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#1a0a2e, floor=#080810 | walls=(0x1c,0x1c,0x1c), THRESH_INTERIOR |
| L1 | SECTOR/TUNNEL | ALL | LERP | #1a0a2e / #0a0618 | Bound: tunnel enclosure, param_a=80 |
| L2 | SILHOUETTE/CITY | WALLS | LERP | #0a0510 / #000000 | Bound: city skyline, param_a=128 |
| L3 | GRID | WALLS | ADD | #00ffff / #000000 | Cyan grid, param_a=32, param_c=3 scroll |
| L4 | SCATTER/WINDOWS | WALLS | ADD | #ffcc00 / #000000 | Yellow window lights, AXIS_CYL |
| L5 | VEIL/LASER_BARS | WALLS | SCREEN | #ff00ff / #000000 | Magenta lasers, AXIS_CYL |
| L6 | FLOW | ALL | SCREEN | #00ddff / #000000 | Rain, dir=DOWN |
| L7 | ATMOSPHERE/MIE | ALL | LERP | #404050 / #000000 | Gray haze |

### Preset 2: "Crimson Hellscape" — Horror/demonic volcanic

**Design concept:** Standing inside a volcanic crater. Organic membrane textures coat the walls. Glowing lava cracks in the floor, embers rise, a blood-red eclipse hangs in the sky, a dimensional rift tears through the wall.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#4a0000, floor=#0a0000 | walls=(0x2a,0x08,0x08), THRESH_BALANCED |
| L1 | PATCHES/MEMBRANE | WALLS | SCREEN | #330000 / #1a0000 | Bound: organic tissue, DIRECT3D |
| L2 | TRACE/CRACKS | FLOOR | ADD | #ff3300 / #000000 | Lava veins, TANGENT_LOCAL |
| L3 | FLOW | FLOOR | SCREEN | #ff4400 / #000000 | Churning lava |
| L4 | SCATTER/EMBERS | ALL | ADD | #ff8800 / #000000 | Rising sparks |
| L5 | CELESTIAL/ECLIPSE | SKY | ADD | #200000 / #ff0000 | Blood eclipse, dir=SUN |
| L6 | PORTAL/RIFT | WALLS | SCREEN | #ff2200 / #400000 | Dimensional tear, TANGENT_LOCAL |
| L7 | ATMOSPHERE/ABSORPTION | ALL | LERP | #400000 / #000000 | Blood mist |

### Preset 3: "Frozen Tundra" — Arctic survival

**Design concept:** Standing on a cracked ice shelf under a pale arctic sky. Shattered ice patterns on the floor, snow blowing, cold blue atmosphere, distant icy landscape.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#c8e0f0, floor=#f8f8ff | walls=(0xa0,0xc8,0xe0), THRESH_OPEN |
| L1 | CELL/SHATTER | FLOOR | LERP | #d0f0ff / #a0c8e0 | Bound: cracked ice |
| L2 | PLANE/STONE | FLOOR | LERP | #e8f4ff / #c0d8e8 | Frozen ground texture |
| L3 | SCATTER/SNOW | ALL | ADD | #ffffff / #000000 | Blizzard, dir=DOWN |
| L4 | FLOW | SKY | ADD | #ffffff / #000000 | Drifting snow clouds |
| L5 | ATMOSPHERE/RAYLEIGH | ALL | LERP | #b0d8f0 / #000000 | Crisp cold air |
| L6 | LOBE | ALL | ADD | #e0f0ff / #000000 | Pale sun glow, dir=SUN |
| L7 | NOP | - | - | - | - |

### Preset 4: "Alien Jungle" — Bioluminescent alien canopy

**Design concept:** Under a dense alien forest canopy. Purple sky peeks through, bioluminescent patches streak the walls, organic radial cell patterns, floating spores, hanging vine curtains, alien atmosphere with green tint.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#3a0050, floor=#002020 | walls=(0x00,0x40,0x40), THRESH_SEMI |
| L1 | SILHOUETTE/FOREST | WALLS | LERP | #001818 / #003030 | Bound: alien tree silhouettes |
| L2 | PATCHES/STREAKS | WALLS | ADD | #00ffaa / #004040 | Bound: bioluminescent streaks, AXIS_CYL |
| L3 | VEIL/CURTAINS | WALLS | SCREEN | #8000ff / #000000 | Hanging bioluminescent vines, AXIS_CYL |
| L4 | SCATTER/DUST | ALL | ADD | #00ffcc / #000000 | Floating spores |
| L5 | FLOW | FLOOR | SCREEN | #00ddcc / #000000 | Rippling bioluminescence |
| L6 | ATMOSPHERE/ALIEN | ALL | LERP | #004020 / #000000 | Exotic gas |
| L7 | LOBE | SKY | ADD | #3a0050 / #000000 | Canopy glow, dir=UP |

**Step 1: Write preset 1-4 code**

Replace the entire contents of `set_01_04.rs` with the new preset arrays following the designs above. Use `hi()`/`hi_meta()`/`lo()` helpers. Include comment blocks matching the existing style (header comment with layer summary, inline comments per layer).

Key differences from current code:
- Preset 1: SECTOR moved to L1 as bound, SILHOUETTE to L2. Use BLEND_LERP not MULTIPLY for silhouette (with actual colors, not black).
- Preset 2: PATCHES/MEMBRANE moved to L1 as bound with SCREEN blend and colored. TRACE targets FLOOR only.
- Preset 3: CELL/SHATTER moved to L1 as bound. Removed APERTURE/BARS and APERTURE/CIRCLE (were just darkening). Added LOBE for pale sun.
- Preset 4: Changed THRESH to SEMI. SILHOUETTE uses LERP with teal colors instead of MULTIPLY with black. Added LOBE for canopy glow.

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`
Expected: compiles without errors

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_01_04.rs
git commit -m "preset: redesign presets 1-4 with proper bounds/regions structure"
```

---

## Task 2: Redesign Presets 5-8 (set_05_08.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_05_08.rs`

### Preset 5: "Gothic Cathedral" — Candlelit stone interior

**Design concept:** Inside a vast gothic cathedral. Stone brick walls, arched windows letting in shafts of golden light. Incense smoke hangs in the air, golden dust motes drift through light beams. Stained glass lead lines on walls.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#0a0a20, floor=#1a1a1a | walls=(0x20,0x20,0x20), THRESH_INTERIOR |
| L1 | APERTURE/ARCH | WALLS | LERP | #181818 / #303028 | Bound: gothic arch frames, nonzero |
| L2 | CELL/BRICK | WALLS | LERP | #282828 / #1a1a18 | Bound: stone wall texture |
| L3 | TRACE/LEAD_LINES | WALLS | ADD | #806040 / #000000 | Stained glass leading, TANGENT_LOCAL |
| L4 | LOBE | ALL | ADD | #ffd700 / #000000 | Divine golden light, dir=SUN |
| L5 | SCATTER/DUST | ALL | ADD | #ffcc00 / #000000 | Golden dust motes |
| L6 | ATMOSPHERE/MIE | ALL | LERP | #302820 / #000000 | Incense haze |
| L7 | NOP | - | - | - | - |

### Preset 6: "Ocean Depths" — Deep sea trench

**Design concept:** Deep underwater in a cave-like trench. Water caustics ripple on the floor, bioluminescent particles drift, light shafts pierce from above, coral reef patches on the walls.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#001030, floor=#203040 | walls=(0x00,0x28,0x48), THRESH_INTERIOR |
| L1 | SECTOR/CAVE | ALL | LERP | #001828 / #002040 | Bound: cave enclosure |
| L2 | PLANE/WATER | FLOOR | LERP | #004080 / #002848 | Caustic floor |
| L3 | FLOW | FLOOR | ADD | #00a0c0 / #000000 | Animated caustics |
| L4 | SCATTER/BUBBLES | ALL | ADD | #40a0a0 / #000000 | Floating bubbles |
| L5 | VEIL/SHARDS | SKY | ADD | #80c0e0 / #000000 | Light shafts from surface, AXIS_CYL |
| L6 | ATMOSPHERE/ABSORPTION | ALL | LERP | #000820 / #000000 | Deep water fog |
| L7 | DECAL | WALLS | ADD | #00ffaa / #000000 | Bioluminescent glow spot |

### Preset 7: "Void Station" — Derelict space station

**Design concept:** Inside a damaged space station. Box-shaped sector with rectangular viewports. Stars visible through hull breach, technical blue grid on walls, floor grating pattern, binary star system outside.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#000008, floor=#101018 | walls=(0x18,0x18,0x20), THRESH_INTERIOR |
| L1 | SECTOR/BOX | ALL | LERP | #101820 / #0a0a18 | Bound: box enclosure |
| L2 | APERTURE/RECT | WALLS | LERP | #0a0a14 / #181820 | Bound: rectangular viewport |
| L3 | GRID | WALLS | ADD | #0044aa / #000000 | Blue panel lines |
| L4 | CELL/GRID | FLOOR | LERP | #080820 / #101018 | Floor grating |
| L5 | SCATTER/STARS | SKY | ADD | #ffffff / #000000 | Stars through viewport |
| L6 | CELESTIAL/BINARY | SKY | ADD | #00aa88 / #4488aa | Binary star, dir=SUN |
| L7 | DECAL | WALLS | ADD | #00ff00 / #000000 | Green status indicator |

### Preset 8: "Desert Mirage" — Vast dunes under blazing sun

**Design concept:** Standing in a vast open desert. Rolling sand dunes on the horizon, textured sand floor, blazing sun overhead, heat shimmer distorting the air, warm atmospheric haze. Horizon band glow.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#f0e8d0, floor=#d4b896 | walls=(0xc8,0xa8,0x78), THRESH_VAST |
| L1 | SILHOUETTE/DUNES | WALLS | LERP | #b89860 / #d0b080 | Bound: sand dune silhouettes |
| L2 | PLANE/SAND | FLOOR | LERP | #d8c090 / #c0a870 | Textured sand floor |
| L3 | CELESTIAL/SUN | SKY | ADD | #ffffd8 / #000000 | Blazing sun, dir=SUN |
| L4 | FLOW | WALLS | ADD | #f8f0e0 / #000000 | Heat shimmer, low intensity |
| L5 | BAND | ALL | ADD | #ffe0a0 / #000000 | Warm horizon glow, dir=SUNSET |
| L6 | ATMOSPHERE/MIE | ALL | LERP | #e8d8c0 / #000000 | Desert haze |
| L7 | SCATTER/DUST | FLOOR | ADD | #c8b080 / #000000 | Blowing sand |

**Step 1: Write preset 5-8 code**

Replace the entire contents of `set_05_08.rs` with new preset arrays following designs above.

Key differences from current code:
- Preset 5: APERTURE/ARCH now uses BLEND_LERP with nonzero colors (#181818/#303028) instead of MULTIPLY with black. CELL/BRICK also LERP with two-tone. Removed GRID (was redundant with TRACE/LEAD_LINES). Added NOP for cleaner design.
- Preset 6: Added SECTOR/CAVE as L1 bound. PATCHES/ISLANDS removed (was using MULTIPLY+black).
- Preset 7: SPLIT/HALF replaced with SECTOR/BOX as L1. APERTURE/IRREGULAR replaced with APERTURE/RECT as L2. Both now use LERP with meaningful colors.
- Preset 8: SILHOUETTE/DUNES now uses LERP with two-tone sand colors. Added BAND for horizon glow. Removed SECTOR/BOX (was HSV_MOD with black).

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`
Expected: compiles without errors

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_05_08.rs
git commit -m "preset: redesign presets 5-8 with proper bounds/regions structure"
```

---

## Task 3: Redesign Presets 9-12 (set_09_12.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs`

### Preset 9: "Neon Arcade" — Retro synthwave

**Design concept:** Retrowave aesthetic. Magenta wireframe grid floor, neon bands across walls, starfield sky, retro planet on horizon, cyan horizon glow line, pulsing purple ambient.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#000010, floor=#100020 | walls=(0x08,0x00,0x18), THRESH_BALANCED |
| L1 | SPLIT/BANDS | WALLS | ADD | #00ffff / #ff00ff | Bound: neon horizontal bands |
| L2 | GRID | FLOOR | ADD | #ff00ff / #000000 | Magenta wireframe grid |
| L3 | SCATTER/STARS | SKY | SCREEN | #ffffff / #000000 | Starfield |
| L4 | CELESTIAL/PLANET | SKY | ADD | #ff0088 / #000000 | Retro planet, dir=SUNSET |
| L5 | BAND | ALL | ADD | #00ffff / #000000 | Cyan horizon glow |
| L6 | FLOW | ALL | SCREEN | #8000ff / #000000 | Purple pulsing glow |
| L7 | NOP | - | - | - | - |

### Preset 10: "Storm Front" — Dramatic thunderstorm

**Design concept:** Open plain under a massive thunderstorm. Mountain silhouette on horizon, churning clouds overhead, lightning bolts, heavy rain curtains, wet pavement below, dramatic directional light from lightning.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#202830, floor=#181820 | walls=(0x30,0x38,0x40), THRESH_OPEN |
| L1 | SILHOUETTE/MOUNTAINS | WALLS | LERP | #181820 / #282830 | Bound: distant mountains |
| L2 | FLOW | SKY | ADD | #404858 / #000000 | Churning storm clouds |
| L3 | TRACE/LIGHTNING | SKY | ADD | #ffffff / #000000 | Lightning, AXIS_POLAR |
| L4 | VEIL/RAIN_WALL | ALL | SCREEN | #607080 / #000000 | Rain curtains, AXIS_CYL |
| L5 | SCATTER/RAIN | ALL | SCREEN | #8090a0 / #000000 | Raindrops, AXIS_CYL, dir=DOWN |
| L6 | PLANE/PAVEMENT | FLOOR | LERP | #282830 / #202028 | Wet ground |
| L7 | ATMOSPHERE/FULL | ALL | LERP | #303038 / #000000 | Storm atmosphere |

### Preset 11: "Crystal Cavern" — Fantasy underground geode

**Design concept:** Inside a massive crystal geode. Voronoi crystal structures on walls, amethyst debris scattered around, cyan energy filaments running through crystals, purple glow from below, magic circle on floor.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#100020, floor=#080010 | walls=(0x18,0x00,0x30), THRESH_INTERIOR |
| L1 | CELL/VORONOI | WALLS | ADD | #400080 / #200040 | Bound: crystal structure |
| L2 | PATCHES/DEBRIS | FLOOR | SCREEN | #6020a0 / #300060 | Bound: crystal formations |
| L3 | TRACE/FILAMENTS | WALLS | ADD | #00e0ff / #000000 | Energy veins, TANGENT_LOCAL |
| L4 | SCATTER/DUST | ALL | SCREEN | #ffffff / #000000 | Crystal sparkles |
| L5 | LOBE | ALL | ADD | #a040ff / #000000 | Purple glow from below, dir=DOWN |
| L6 | PORTAL/CIRCLE | FLOOR | ADD | #00ffff / #200040 | Magic circle, TANGENT_LOCAL |
| L7 | ATMOSPHERE/ABSORPTION | ALL | LERP | #200040 / #000000 | Purple cave mist |

### Preset 12: "War Zone" — Military/apocalyptic battlefield

**Design concept:** Ruined urban battlefield. Building ruins on horizon, industrial floor grating, scattered rubble debris, floating ash and embers, smoke trails, thick brown smoke atmosphere, cave-like enclosure from collapsed buildings.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#383030, floor=#282020 | walls=(0x30,0x28,0x20), THRESH_SEMI |
| L1 | SILHOUETTE/RUINS | WALLS | LERP | #201810 / #302820 | Bound: ruined buildings |
| L2 | APERTURE/IRREGULAR | SKY | LERP | #201810 / #383030 | Bound: broken roof opening |
| L3 | PLANE/GRATING | FLOOR | LERP | #484040 / #302820 | Industrial floor |
| L4 | SCATTER/EMBERS | ALL | ADD | #ff6600 / #000000 | Floating ash/embers |
| L5 | FLOW | SKY | ADD | #606060 / #000000 | Smoke trails |
| L6 | ATMOSPHERE/ABSORPTION | ALL | LERP | #302820 / #000000 | War smoke |
| L7 | DECAL | WALLS | ADD | #ff4400 / #200800 | Burning fire spot |

**Step 1: Write preset 9-12 code**

Replace contents of `set_09_12.rs`.

Key differences:
- Preset 9: Removed duplicate SPLIT layers. Cleaner 7-layer design with NOP at end.
- Preset 10: SILHOUETTE uses LERP with dark tones instead of MULTIPLY+black. Added LOBE removed from previous (lightning provides the drama).
- Preset 11: CELL/VORONOI and PATCHES/DEBRIS as L1-L2 bounds with ADD/SCREEN and colored.
- Preset 12: Changed THRESH to SEMI. SILHOUETTE/RUINS uses LERP. Added APERTURE/IRREGULAR as L2 bound (broken roof). Removed SECTOR/CAVE (redundant). Added DECAL for fire.

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs
git commit -m "preset: redesign presets 9-12 with proper bounds/regions structure"
```

---

## Task 4: Redesign Presets 13-16 (set_13_16.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_13_16.rs`

### Preset 13: "Enchanted Grove" — Fairy tale forest

**Design concept:** Standing in a sunlit fairy tale forest. Forest silhouette forms the walls, lush grass floor, hanging moss vines, golden fairy dust particles, dappled sunlight patches, warm sunbeam through canopy.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#fff8d0, floor=#204020 | walls=(0x1a,0x38,0x20), THRESH_OPEN |
| L1 | SILHOUETTE/FOREST | WALLS | LERP | #0a2010 / #1a3820 | Bound: forest silhouette (green, not black) |
| L2 | PLANE/GRASS | FLOOR | LERP | #308030 / #204020 | Lush forest floor |
| L3 | VEIL/CURTAINS | WALLS | ADD | #40a040 / #000000 | Hanging moss, AXIS_CYL |
| L4 | SCATTER/DUST | ALL | ADD | #ffdd00 / #000000 | Fairy dust (gold) |
| L5 | PATCHES/BLOBS | SKY | SCREEN | #fff080 / #000000 | Dappled sunlight |
| L6 | LOBE | ALL | ADD | #ffd700 / #000000 | Sunbeam, dir=SUN |
| L7 | FLOW | FLOOR | ADD | #60a060 / #000000 | Gentle leaf movement |

### Preset 14: "Astral Void" — Cosmic void

**Design concept:** Floating in deep space. Nebula gas clouds, dense starfield, a gas giant planet, a ringed planet in the distance, a cosmic tear, swirling vortex portal, faint structural lattice.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#000004, floor=#080010 | walls=(0x10,0x00,0x20), THRESH_VAST |
| L1 | PATCHES/BLOBS | ALL | SCREEN | #200840 / #100420 | Bound: nebula gas clouds |
| L2 | FLOW | ALL | SCREEN | #4000a0 / #000000 | Swirling cosmic gases |
| L3 | SCATTER/STARS | SKY | ADD | #ffffff / #000000 | Dense starfield |
| L4 | CELESTIAL/GAS_GIANT | SKY | ADD | #ff6040 / #000000 | Gas giant, dir=SUN |
| L5 | CELESTIAL/RINGED | SKY | ADD | #d0c080 / #000000 | Ringed planet, dir=SUNSET |
| L6 | PORTAL/VORTEX | WALLS | SCREEN | #ffffff / #8040ff | Cosmic vortex, TANGENT_LOCAL |
| L7 | BAND | ALL | ADD | #4020a0 / #000000 | Nebula horizon glow |

### Preset 15: "Toxic Wasteland" — Post-apocalyptic industrial

**Design concept:** A poisoned industrial landscape. Factory smokestacks on horizon, cracked toxic tile floor, radioactive green patches, hexagonal hazmat patterns, rising toxic fumes, poisonous atmosphere.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#304010, floor=#202008 | walls=(0x28,0x30,0x18), THRESH_BALANCED |
| L1 | SILHOUETTE/INDUSTRIAL | WALLS | LERP | #181808 / #283018 | Bound: factory smokestacks |
| L2 | PATCHES/ISLANDS | FLOOR | ADD | #40a000 / #204000 | Bound: radioactive puddles |
| L3 | PLANE/TILES | FLOOR | LERP | #483820 / #302810 | Cracked industrial floor |
| L4 | CELL/HEX | WALLS | SCREEN | #a0a000 / #000000 | Hazmat hex pattern |
| L5 | VEIL/PILLARS | WALLS | ADD | #408020 / #000000 | Toxic fume columns, AXIS_CYL |
| L6 | SCATTER/DUST | ALL | ADD | #a0c040 / #000000 | Toxic particles |
| L7 | ATMOSPHERE/ALIEN | ALL | LERP | #203008 / #000000 | Poisonous air |

### Preset 16: "Moonlit Graveyard" — Gothic horror

**Design concept:** A misty cemetery under a full moon. Gothic spire tombstones on the horizon, weathered stone path, creeping moss, pale mist particles, full moon overhead, hanging mist curtains, horizon band of eerie blue glow.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#0a0a1a, floor=#101010 | walls=(0x18,0x18,0x20), THRESH_OPEN |
| L1 | SILHOUETTE/SPIRES | WALLS | LERP | #0a0810 / #141420 | Bound: gothic tombstones |
| L2 | PLANE/STONE | FLOOR | LERP | #282828 / #1a1a20 | Weathered path |
| L3 | CELESTIAL/MOON | SKY | ADD | #e0e8f0 / #000000 | Full moon, dir=SUN |
| L4 | BAND | SKY | ADD | #202840 / #000000 | Eerie blue horizon glow |
| L5 | SCATTER/DUST | ALL | ADD | #8090a0 / #000000 | Mist particles |
| L6 | VEIL/CURTAINS | WALLS | ADD | #404050 / #000000 | Hanging mist, AXIS_CYL |
| L7 | ATMOSPHERE/FULL | ALL | LERP | #101020 / #000000 | Heavy night fog |

**Step 1: Write preset 13-16 code**

Replace contents of `set_13_16.rs`.

Key differences:
- Preset 13: SILHOUETTE uses LERP with green tones. SILHOUETTE_MOUNTAINS changed to SILHOUETTE_FOREST (matches concept). Removed FLOW from L7 (was weak), added it back as floor-only.
- Preset 14: PATCHES/BLOBS as L1 bound with SCREEN (nebula clouds, not black). Removed duplicate PORTAL layers, kept VORTEX. Added BAND.
- Preset 15: SILHOUETTE uses LERP. PATCHES/ISLANDS moved to L2 as bound (radioactive puddles). Reordered so bounds come first.
- Preset 16: SILHOUETTE uses LERP with dark purple tones. PATCHES/MEMBRANE removed (was MULTIPLY+black). Added BAND for eerie horizon. Replaced PORTAL/CRACK with ATMOSPHERE/FULL.

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_13_16.rs
git commit -m "preset: redesign presets 13-16 with proper bounds/regions structure"
```

---

## Task 5: Redesign Presets 17-20 (set_17_20.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_17_20.rs`

### Preset 17: "Volcanic Core" — Inside active volcano

**Design concept:** Deep inside a volcanic chamber. Hexagonal basalt columns, debris-covered floor, glowing lava cracks, churning lava flow, rising sparks, intense heat glow from below, choking volcanic gases.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#100800, floor=#401000 | walls=(0x20,0x10,0x08), THRESH_INTERIOR |
| L1 | CELL/HEX | WALLS | LERP | #181008 / #100800 | Bound: hexagonal basalt columns |
| L2 | PATCHES/DEBRIS | FLOOR | ADD | #301800 / #200c00 | Bound: volcanic rubble |
| L3 | PLANE/STONE | FLOOR | LERP | #181008 / #100800 | Rocky volcanic floor |
| L4 | TRACE/CRACKS | FLOOR | ADD | #ff4000 / #000000 | Lava veins, TANGENT_LOCAL |
| L5 | FLOW | FLOOR | SCREEN | #ff2800 / #000000 | Churning lava |
| L6 | SCATTER/EMBERS | ALL | ADD | #ff8000 / #000000 | Rising sparks |
| L7 | ATMOSPHERE/ABSORPTION | ALL | LERP | #100800 / #000000 | Volcanic gas |

### Preset 18: "Digital Matrix" — Cyber virtual reality

**Design concept:** Inside a digital simulation. Green data grid everywhere, falling code rain, data block cells, rectangular data portals, code streaming animation. Clean digital aesthetic.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#000000, floor=#001000 | walls=(0x00,0x20,0x00), THRESH_BALANCED |
| L1 | SPLIT/CROSS | ALL | ADD | #003000 / #001800 | Bound: data grid structure |
| L2 | CELL/GRID | WALLS | LERP | #003000 / #001000 | Bound: data block cells |
| L3 | GRID | WALLS | ADD | #00ff00 / #000000 | Green wireframe |
| L4 | SCATTER/RAIN | ALL | SCREEN | #00ff00 / #000000 | Falling code rain, AXIS_CYL, dir=DOWN |
| L5 | FLOW | ALL | ADD | #00dd00 / #000000 | Code streaming, dir=DOWN |
| L6 | DECAL | WALLS | ADD | #00ffff / #000000 | Data HUD element |
| L7 | PORTAL/RECT | WALLS | ADD | #00ffff / #004000 | Data portal, TANGENT_LOCAL |

### Preset 19: "Noir Detective" — 1940s private eye office

**Design concept:** A cramped detective's office at night. Box-shaped sector, circular desk lamp aperture, venetian blind shadow stripes from window, warm desk lamp glow, cigarette smoke particles, smoky haze.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#101008, floor=#302820 | walls=(0x38,0x34,0x28), THRESH_INTERIOR |
| L1 | SECTOR/BOX | ALL | LERP | #282418 / #1a1810 | Bound: office box enclosure |
| L2 | APERTURE/RECT | WALLS | LERP | #101008 / #282418 | Bound: window frame |
| L3 | SPLIT/WEDGE | WALLS | LERP | #101008 / #383020 | Venetian blind shadows, dir=SUN |
| L4 | LOBE | FLOOR | ADD | #ffe0a0 / #000000 | Desk lamp glow, dir=DOWN |
| L5 | SCATTER/DUST | ALL | ADD | #808070 / #000000 | Cigarette smoke |
| L6 | ATMOSPHERE/MIE | ALL | LERP | #302820 / #000000 | Smoky haze |
| L7 | FLOW | WALLS | ADD | #404030 / #000000 | Rain on window, low intensity |

### Preset 20: "Steampunk Airship" — Victorian observation deck

**Design concept:** Inside a brass-and-copper airship observation deck. Rounded rectangular portholes, hexagonal riveted floor plates, brass framework girders, setting sun through viewport, rising steam columns, warm amber haze.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#ffa040, floor=#604020 | walls=(0x80,0x50,0x30), THRESH_SEMI |
| L1 | APERTURE/ROUNDED_RECT | WALLS | LERP | #402010 / #604030 | Bound: porthole frames |
| L2 | CELL/HEX | FLOOR | LERP | #503020 / #402010 | Bound: riveted hex plates |
| L3 | GRID | WALLS | ADD | #c09040 / #000000 | Brass girders |
| L4 | CELESTIAL/SUN | SKY | ADD | #ffc060 / #000000 | Setting sun, dir=SUNSET |
| L5 | VEIL/PILLARS | WALLS | ADD | #fff0d0 / #000000 | Steam columns, AXIS_CYL |
| L6 | SCATTER/DUST | ALL | ADD | #ffe8c0 / #000000 | Steam particles |
| L7 | ATMOSPHERE/MIE | ALL | LERP | #604020 / #000000 | Warm amber haze |

**Step 1: Write preset 17-20 code**

Replace contents of `set_17_20.rs`.

Key differences:
- Preset 17: CELL/HEX and PATCHES/DEBRIS as L1-L2 bounds with LERP/ADD and proper colors. Removed SPLIT/CROSS (was Add with dark/bright — confusing). Removed LOBE (heat glow conveyed by TRACE+FLOW).
- Preset 18: SPLIT/CROSS and CELL/GRID as L1-L2 bounds. Removed duplicate APERTURE layers. Added DECAL for HUD element.
- Preset 19: SECTOR/BOX and APERTURE/RECT as L1-L2 bounds with LERP. SPLIT/WEDGE moved to L3 feature. All MULTIPLY replaced with LERP.
- Preset 20: APERTURE/ROUNDED_RECT and CELL/HEX as L1-L2 bounds. Removed APERTURE/MULTI (was MULTIPLY+black).

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_17_20.rs
git commit -m "preset: redesign presets 17-20 with proper bounds/regions structure"
```

---

## Task 6: Redesign Presets 21-24 (set_21_24.rs)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs`

### Preset 21: "Stormy Shores" — Coastal cliffs, crashing waves

**Design concept:** Standing on coastal cliffs during a storm. Wave silhouette on horizon, wet rocky shore floor, churning sea foam spray, light breaking through clouds, heavy coastal fog, lighthouse beam sweeping.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#1a2028, floor=#202830 | walls=(0x30,0x38,0x40), THRESH_OPEN |
| L1 | SILHOUETTE/WAVES | WALLS | LERP | #101820 / #1a2028 | Bound: crashing waves |
| L2 | PLANE/STONE | FLOOR | LERP | #303840 / #202830 | Wet rocky shore |
| L3 | FLOW | WALLS | ADD | #607080 / #000000 | Sea foam and spray |
| L4 | SCATTER/RAIN | ALL | ADD | #90a0b0 / #000000 | Storm spray, dir=DOWN |
| L5 | VEIL/SHARDS | SKY | ADD | #8090a0 / #000000 | Light through clouds, AXIS_CYL |
| L6 | ATMOSPHERE/FULL | ALL | LERP | #283038 / #000000 | Coastal storm fog |
| L7 | LOBE | ALL | ADD | #ffffd0 / #000000 | Lighthouse beam, dir=SUNSET |

### Preset 22: "Polar Aurora" — Arctic night with northern lights

**Design concept:** Arctic night landscape. Radial cell pattern on ice floor, aurora curtains spreading across sky, aurora band on horizon, starfield, bright moon, snow/ice ground, crisp air.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#080818, floor=#d0e0f0 | walls=(0x40,0x50,0x60), THRESH_OPEN |
| L1 | CELL/RADIAL | FLOOR | ADD | #406080 / #203040 | Bound: radial ice pattern, AXIS_POLAR |
| L2 | BAND | SKY | ADD | #00ff80 / #00ffff | Aurora horizon band |
| L3 | VEIL/CURTAINS | SKY | ADD | #40ff80 / #000000 | Aurora curtains, AXIS_POLAR |
| L4 | SCATTER/STARS | SKY | ADD | #ffffff / #000000 | Night starfield |
| L5 | CELESTIAL/MOON | SKY | ADD | #f0f8ff / #000000 | Bright arctic moon, dir=SUN |
| L6 | PLANE/STONE | FLOOR | LERP | #e0f0ff / #c0d0e0 | Snow/ice ground |
| L7 | ATMOSPHERE/RAYLEIGH | ALL | LERP | #102030 / #000000 | Crisp arctic air |

### Preset 23: "Sacred Geometry" — Abstract mathematical temple

**Design concept:** A temple of pure geometric forms. Prism-split walls, grid cell floor, geometric frame lines, radial energy filaments, central circular aperture, divine light from center, golden sacred particles.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#100828, floor=#c0a040 | walls=(0x40,0x20,0x50), THRESH_SEMI |
| L1 | SPLIT/PRISM | WALLS | ADD | #402060 / #604080 | Bound: prismatic wall divisions |
| L2 | CELL/GRID | FLOOR | ADD | #ffd080 / #a08040 | Bound: geometric floor tiles, AXIS_POLAR |
| L3 | GRID | WALLS | ADD | #806040 / #000000 | Geometric frame lines |
| L4 | TRACE/FILAMENTS | WALLS | ADD | #ffffff / #000000 | Radial energy, AXIS_POLAR |
| L5 | APERTURE/CIRCLE | ALL | LERP | #200810 / #402040 | Central opening |
| L6 | LOBE | ALL | ADD | #fff0c0 / #000000 | Divine central light, dir=DOWN |
| L7 | SCATTER/DUST | ALL | SCREEN | #ffd040 / #000000 | Golden sacred particles |

### Preset 24: "Ritual Chamber" — Dark magic summoning room

**Design concept:** An occult summoning room. Circular aperture overhead, voronoi stone walls, magic circle pentagram on floor, summoning portal, arcane energy veins, energy pillars at cardinal points, magical sparks.

**Layer plan:**
| Layer | Opcode | Region | Blend | Colors | Notes |
|-------|--------|--------|-------|--------|-------|
| L0 | RAMP | ALL | LERP | sky=#000004, floor=#100808 | walls=(0x18,0x10,0x18), THRESH_INTERIOR |
| L1 | APERTURE/CIRCLE | SKY | LERP | #080408 / #100810 | Bound: circular chamber opening |
| L2 | CELL/VORONOI | WALLS | LERP | #201020 / #100810 | Bound: rough stone walls |
| L3 | DECAL | FLOOR | ADD | #ff2000 / #400000 | Magic pentagram |
| L4 | PORTAL/CIRCLE | FLOOR | ADD | #8020ff / #200040 | Summoning portal, TANGENT_LOCAL |
| L5 | TRACE/FILAMENTS | WALLS | ADD | #a040ff / #000000 | Arcane energy veins, TANGENT_LOCAL |
| L6 | SCATTER/EMBERS | ALL | SCREEN | #ff8040 / #000000 | Magical sparks |
| L7 | ATMOSPHERE/ALIEN | ALL | LERP | #100010 / #000000 | Otherworldly atmosphere |

**Step 1: Write preset 21-24 code**

Replace contents of `set_21_24.rs`.

Key differences:
- Preset 21: SILHOUETTE uses LERP with sea-dark tones. SCATTER changed to RAIN variant.
- Preset 22: CELL/RADIAL as L1 bound (was just a feature). BAND moved to L2 (aurora showcase).
- Preset 23: SPLIT/PRISM and CELL/GRID as L1-L2 bounds with ADD and colors. APERTURE/CIRCLE moved to L5 with LERP (not MULTIPLY+black). TRACE/LEAD_LINES changed to FILAMENTS.
- Preset 24: APERTURE/CIRCLE and CELL/VORONOI as L1-L2 bounds with LERP and stone colors. Removed VEIL/PILLARS and FLOW (too many effects). Cleaner 8-layer design.

**Step 2: Verify build**

Run: `cargo build -p epu-showcase`

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs
git commit -m "preset: redesign presets 21-24 with proper bounds/regions structure"
```

---

## Task 7: Full Verification

**Step 1: Full build and lint**

Run: `cargo build -p epu-showcase && cargo clippy -p epu-showcase --all-targets -- -D warnings`
Expected: no errors, no warnings

**Step 2: Run the showcase**

Run: `cargo run -p epu-showcase` (or however the example is launched)
Cycle through all 24 presets using A/B buttons. Verify:
- Each preset has a distinct spatial feel (not just flat colors)
- Sky/wall/floor regions are clearly differentiated
- No presets are mostly black/dark with no visible features
- Bound layers create meaningful spatial structure
- Feature layers add visible detail within the bounded regions

**Step 3: Final commit (if any fixes needed)**

```bash
git add -A examples/3-inspectors/epu-showcase/
git commit -m "preset: fix any issues found during visual verification"
```

---

## Opcode Coverage Summary

Every opcode appears at least 2-3 times across all 24 presets:

| Opcode | Presets Using It |
|--------|-----------------|
| RAMP | All 24 (L0) |
| SECTOR | 1 (TUNNEL), 6 (CAVE), 7 (BOX), 19 (BOX) |
| SILHOUETTE | 4 (FOREST), 8 (DUNES), 10 (MOUNTAINS), 12 (RUINS), 13 (FOREST), 15 (INDUSTRIAL), 16 (SPIRES), 21 (WAVES) |
| SPLIT | 9 (BANDS), 10 (implied via MOUNTAINS), 17 (removed), 18 (CROSS), 19 (WEDGE), 23 (PRISM) |
| CELL | 3 (SHATTER), 7 (GRID), 11 (VORONOI), 15 (HEX), 17 (HEX), 18 (GRID), 20 (HEX), 22 (RADIAL), 23 (GRID), 24 (VORONOI) |
| PATCHES | 2 (MEMBRANE), 4 (STREAKS), 11 (DEBRIS), 13 (BLOBS), 14 (BLOBS), 15 (ISLANDS), 17 (DEBRIS) |
| APERTURE | 5 (ARCH), 7 (RECT), 12 (IRREGULAR), 19 (RECT), 20 (ROUNDED_RECT), 23 (CIRCLE), 24 (CIRCLE) |
| DECAL | 6, 7, 12, 18, 24 |
| GRID | 1, 5, 9, 18, 20, 23 |
| SCATTER | 1, 2, 3, 4, 5, 10, 11, 13, 14, 15, 16, 17, 18, 20, 21, 22, 23, 24 |
| FLOW | 1, 2, 3, 6, 9, 10, 13, 14, 17, 18, 19, 21 |
| TRACE | 2, 5, 10, 11, 17, 23, 24 |
| VEIL | 1, 3 (removed), 4, 10, 13, 15, 16, 20, 21, 22 |
| ATMOSPHERE | 2, 4, 5, 6, 8, 10, 15, 16, 17, 19, 20, 21, 22, 24 |
| PLANE | 3, 6, 8, 10, 12, 13, 15, 17, 21, 22 |
| CELESTIAL | 2, 7, 8, 9, 14, 16, 20, 22 |
| PORTAL | 2, 11, 14, 18, 24 |
| LOBE | 3, 5, 11, 13, 19, 21, 23 |
| BAND | 8, 9, 14, 16, 22 |
