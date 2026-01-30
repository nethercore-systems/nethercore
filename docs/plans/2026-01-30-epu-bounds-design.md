# EPU Bounds Architecture: Design Clarification
## The Problem
### Current Confusion: "Enclosure" vs "Bounds"
The EPU shader has two separate concepts that overlap confusingly:
1. **Enclosure** (`EnclosureConfig`): Defines a coordinate system
   - `up` vector (world orientation)
   - `ceil_y`, `floor_y` (height thresholds)
   - `soft` (transition softness)
2. **Bounds** (opcodes 0x01-0x07): Layers that render visuals and output region weights
The problem: Most bounds layers **ignore** the enclosure's `ceil_y/floor_y/soft` values. They only use `up` for orientation, then compute regions from their own geometry.
```
RAMP → sets enclosure (ceil_y, floor_y, soft)
SECTOR → ignores ceil_y/floor_y/soft, computes own regions
SILHOUETTE → ignores ceil_y/floor_y/soft, computes own regions
```
This creates confusion and wasted layers (RAMP → SECTOR when SECTOR doesn't use RAMP's thresholds).
### The Real Issue: Floor Is Usually Inherited
Each bounds layer outputs `RegionWeights { sky, wall, floor }`, but today most bounds behave like **2-way splitters**:
- They preserve the incoming `base_regions.floor`
- They only divide the remaining weight (`1.0 - base_regions.floor`) into sky vs wall

RAMP is the only bounds that defines a full 3-way split from its own thresholds (and SPLIT does so for specific variants).
| Bounds | Sky | Wall | Floor |
|--------|-----|------|-------|
| RAMP | ✅ Y threshold | ✅ Y threshold | ✅ Y threshold |
| SECTOR | ✅ azimuth geometry | ✅ azimuth geometry | ⚠️ pass-through |
| SILHOUETTE | ✅ horizon geometry | ✅ horizon geometry | ⚠️ pass-through |
| SPLIT | ✅ plane geometry | ✅ plane geometry | ✅ CORNER + PRISM variants |
| CELL | ✅ cell geometry | ✅ cell geometry | ⚠️ pass-through |
| PATCHES | ✅ noise geometry | ✅ noise geometry | ⚠️ pass-through |
| APERTURE | ✅ SDF geometry | ✅ SDF geometry | ⚠️ pass-through |

**Pattern discovered**: Most non-RAMP bounds compute `(geo_sky * rem, geo_wall * rem, floor)` where `rem = 1.0 - floor_w`. They divide the "remaining" space (after floor) into sky/wall.
This means:
1. Most bounds **subdivide the non-floor space** - they assume a floor weight already exists.
2. RAMP (and SPLIT CORNER/PRISM) are the only bounds that can create a floor region from their own geometry/parameters in the current model.
3. Even without an explicit RAMP layer, the shader currently seeds a *default* enclosure (fixed `ceil_y/floor_y/soft`) and computes default regions from it. So you still get a non-zero floor, but it is **hard-coded** rather than authored.
This is why "RAMP → SECTOR is a waste" feels real: SECTOR doesn't use RAMP's sky/wall thresholds, but it *does* depend on having a preexisting floor weight (typically authored by RAMP; otherwise the hard-coded default).
---
## The Vision: Freestyle Environments
### Core Principle
Each bounds layer should **fully define 3 semantic regions** based on its native geometry. No RAMP dependency required.
### Natural 3-Region Semantics Per Bounds Type
**Universal pattern**: Sky = opening, Wall = visible edge/surface, Floor = solid/background
| Bounds | Sky (A) - Opening | Wall (B) - Visible Edge | Floor (C) - Solid/BG |
|--------|-------------------|-------------------------|----------------------|
| **RAMP** | above ceil_y | between thresholds | below floor_y |
| **SILHOUETTE** | above horizon | the hills (silhouette band) | below horizon |
| **SECTOR** | inside wedge opening | wedge edge band | outside wedge |
| **APERTURE** | inside the hole | the frame band | outside/background |
| **CELL** | cell gaps | cell boundaries | cell interiors |
| **PATCHES** | between patches | patch edges | inside patches |
| **SPLIT** | side A | split edge | side B |
**Example - SILHOUETTE (rolling hills)**:
```
     SKY (region A) - blue, stars here
  ╭╮ ╭╮ ╭╮
 ╱  ╲╱  ╲╱  ╲   WALL (region B) - the hills (green like ground, but separate region)
─────────────
    FLOOR (region C) - solid ground below (green)
```
Wall (B) is the **visible surface/edge** between sky and floor. Even when colored the same as floor, it's a separate region so you can apply different effects (textures, gradients, etc.)
---
## Resolved Questions
### Q1: Naming (floor/wall/sky vs generic)
**Decision**: Keep `sky/wall/floor` names. The universal pattern is:
- **Sky** = opening/visible area
- **Wall** = visible edge/surface
- **Floor** = solid/background
Semantics vary per bounds type, but the pattern is consistent.
### Q2: Stacking behavior
**Decision**: Default is **REPLACE** (each bounds defines fresh 3 regions).
**Optional COMPOSITE mode** using repurposed bits:

The same 3-bit field (bits 122..120) is interpreted differently depending on opcode class:

- **Feature opcodes (0x08+)**: this remains a *region mask* (SKY/WALL/FLOOR) and gates feature contribution.
- **Bounds opcodes (0x01..0x07)**: reinterpret those 3 bits as a *bounds composition mask* using the **same SKY/WALL/FLOOR bit meanings**:
  - `0b111` (ALL) = **REPLACE** (default)
  - `0b000` (NONE) = **NO-OP** (do not update regions or inherited bounds direction)
  - anything else = **COMPOSITE (apply-mask)**

Bit meanings (same as feature region masks): SKY=`0b100`, WALL=`0b010`, FLOOR=`0b001`.

Composite apply-mask semantics:

- Evaluate the bounds layer to produce `new_regions`.
- Compute `m = region_weight(new_regions, mask)` (sum of the selected regions).
- Blend the region state: `out = mix(base_regions, new_regions, m)`.

This is easy to author and reason about:
- `0b110` (SKY|WALL) composites "foreground" (everything except the new bounds' floor/outside).
- `0b100` (SKY) composites only inside the new bounds' opening.
- `0b010` (WALL) composites only on the new bounds' edge band.
- `0b001` (FLOOR) composites only on the new bounds' outside/background region.

This allows stacking like SILHOUETTE → APERTURE where APERTURE only affects the silhouette inside its opening/frame, while leaving the rest of the silhouette intact.
### Q3: Direction handling (RESOLVED)
**Decision**: Each bounds has its own direction semantics. Features inherit direction from most recent bounds for directional effects (e.g., FLOW rain from APERTURE's hole direction).
### Q4: Default without RAMP (RESOLVED)
**Decision**: Default direction = Y-axis. First bounds layer computes regions from its geometry. No inherited `ceil_y/floor_y/soft`.
### Q5: Do features need enclosure? (RESOLVED)
**Decision**: No. Features only need:
- **Direction** (inherited from most recent bounds)
- **RegionWeights** (inherited from most recent bounds)
`EnclosureConfig` struct can be removed or simplified to just direction.
---
## Final Architecture: Unified Bounds Model
### Core Principle: "Bounds ARE the World Definition"
No separate "enclosure" concept. **Bounds = world definition layer**.
### Before (confusing)
```
EnclosureConfig { up, ceil_y, floor_y, soft }  ← shared state, mostly ignored
     ↓
Bounds layer → may or may not use enclosure, outputs modified regions
     ↓
Feature layer → uses enclosure + regions
```
### After (clean)
```
Bounds layer → outputs direction + 3 regions (fully self-contained)
     ↓
Feature layer → inherits direction + regions from most recent bounds
```
### What Each Bounds Layer Outputs
1. **Direction** (16 bits) - opcode-specific meaning:
   - RAMP: up axis
   - SILHOUETTE: up axis (horizon orientation)
   - SECTOR: cylinder axis
   - APERTURE: aperture center (where hole points)
   - SPLIT: plane normal
   - CELL: grid axis
   - PATCHES: noise axis
2. **3 Regions** (RegionWeights) - computed from native geometry:
   - Sky (A): opening/visible area
   - Wall (B): visible edge/surface
   - Floor (C): solid/background
### What Features Inherit
- **Direction**: from most recent bounds (for directional effects like FLOW rain)
- **RegionWeights**: from most recent bounds (for region masking)
### Internal Implementation Details (Not Shared)
- RAMP's `ceil_y/floor_y/soft` - internal to RAMP's region computation
- SILHOUETTE's horizon parameters - internal to SILHOUETTE
- etc.
### Code Changes Required
1. **Remove `EnclosureConfig` struct** (or simplify to just `direction`)
2. **Update all bounds layers** to output full 3 regions from their geometry
3. **Update `enclosure_from_layer`** → rename to `direction_from_layer` (just extracts direction)
4. **Repurpose region bits (122..120)** on bounds for a bounds composition mask (REPLACE / COMPOSITE apply-mask / NO-OP)
---
## Per-Bounds Implementation Changes
### RAMP (0x01) - Already correct
- Currently defines all 3 regions from Y thresholds ✅
- `ceil_y/floor_y/soft` become internal (not shared)
### SECTOR (0x02) - Needs 3rd region
- Current: sky/wall only, floor pass-through
- Change: Compute floor from geometry (outside wedge = floor)
- Regions: inside wedge (sky) / wedge edge (wall) / outside wedge (floor)
### SILHOUETTE (0x03) - Needs 3rd region
- Current: sky/wall only (above/below horizon), floor pass-through
- Change: Compute floor from geometry (below horizon - margin = floor)
- Regions: above horizon (sky) / horizon band (wall) / below horizon (floor)
### SPLIT (0x04) - Partially correct
- Some variants already define 3 regions (CORNER, PRISM)
- Ensure all variants output meaningful 3 regions
- Regions: side A (sky) / split edge (wall) / side B (floor)
### CELL (0x05) - Needs 3rd region
- Current: sky/wall only (gaps/interiors), floor pass-through
- Change: Compute floor from geometry (cell boundaries = wall, or remap)
- Regions: cell gaps (sky) / cell boundaries (wall) / cell interiors (floor)
### PATCHES (0x06) - Needs 3rd region
- Current: sky/wall only (between/inside patches), floor pass-through
- Change: Compute floor from geometry (patch edges = wall)
- Regions: between patches (sky) / patch edges (wall) / inside patches (floor)
### APERTURE (0x07) - Already has 3 concepts
- Current: opening/frame/background, but background is pass-through
- Change: Compute floor from geometry (outside aperture = floor)
- Regions: inside hole (sky) / frame band (wall) / outside hole (floor)
---
## Composite Mode (Optional Stacking)
**Default**: Each bounds replaces previous regions entirely.

**Bounds composition mask** (bits 122..120 of instruction, for bounds opcodes only):
- `0b111` (ALL) = Replace
- `0b000` (NONE) = No-op
- otherwise = Composite apply-mask (SKY/WALL/FLOOR)

Example composite: SILHOUETTE → APERTURE
- SILHOUETTE runs in replace mode (`0b111`) and establishes sky/wall/floor from its horizon geometry.
- APERTURE runs in composite mode with mask `0b110` (SKY|WALL), so it only applies inside its opening+frame and leaves the outside/background untouched.
- Result: the aperture "carves" into the silhouette where it matters, without overwriting the rest of the world.
---
## Benefits of This Architecture
1. **No RAMP dependency** - Any bounds can be layer 0
2. **Cleaner mental model** - Bounds = world definition, Features = rendering
3. **More creative freedom** - Each bounds fully defines its own world
4. **Less dead code** - No more inherited ceil_y/floor_y/soft that gets ignored
5. **Consistent 3-region pattern** - All bounds use sky/wall/floor uniformly
---
## Notes From Discussion
- User identified that `enclosure_from_layer` switch cases are confusingly similar
- SECTOR/SILHOUETTE/SPLIT all extract `up` and inherit heights, but don't use heights
- APERTURE explicitly returns `prev_enc` unchanged (direction means something different)
- The inheritance chain is mostly dead code except for RAMP
- Region bits (122..120) on bounds are unused - repurpose for a bounds composition mask (keep features using them as a region mask)
