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
### The Real Issue: Floor Is Only Defined by RAMP
Each bounds layer outputs `RegionWeights { sky, wall, floor }`, but **floor is always pass-through except for RAMP**:
| Bounds | Sky | Wall | Floor |
|--------|-----|------|-------|
| RAMP | ✅ Y threshold | ✅ Y threshold | ✅ Y threshold |
| SECTOR | ✅ azimuth geometry | ✅ azimuth geometry | ⚠️ pass-through |
| SILHOUETTE | ✅ horizon geometry | ✅ horizon geometry | ⚠️ pass-through |
| SPLIT | ✅ plane geometry | ✅ plane geometry | ✅ CORNER variant only |
| CELL | ✅ cell geometry | ✅ cell geometry | ⚠️ pass-through |
| PATCHES | ✅ noise geometry | ✅ noise geometry | ⚠️ pass-through |
| APERTURE | ✅ SDF geometry | ✅ SDF geometry | ⚠️ pass-through |
**Pattern discovered**: All non-RAMP bounds compute `(geo_sky * rem, geo_wall * rem, floor)` where `rem = 1.0 - floor_w`. They divide the "remaining" space (after floor) into sky/wall.
This means:
1. **RAMP is the only floor source** - it bootstraps the floor region
2. Other bounds **subdivide the non-floor space** - they assume floor already exists
3. **You can't have floor without RAMP** (or default enclosure gives floor=0)
This is why "RAMP → SECTOR is a waste" - SECTOR doesn't use RAMP's sky/wall, but it DOES depend on RAMP's floor!
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
- Bounds region bits (122..120) are currently ignored (hardcoded REGION_ALL)
- These 3 bits can be repurposed for bounds-specific flags:
  - Bit 122: `COMPOSITE_FLAG` (0 = replace, 1 = composite with previous)
  - Bits 121..120: Reserved or composite mode selector
This allows stacking like SILHOUETTE → APERTURE where APERTURE carves into SILHOUETTE's regions.
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
4. **Repurpose region bits (122..120)** on bounds for COMPOSITE flag
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
- Some variants already define 3 regions (CORNER)
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
**Composite flag** (bit 122 of instruction):
- 0 = Replace mode (default) - bounds outputs fresh 3 regions
- 1 = Composite mode - bounds modifies previous regions
Example composite: SILHOUETTE → APERTURE (composite)
- SILHOUETTE: sky (above) / wall (hills) / floor (below)
- APERTURE: carves hole into SILHOUETTE's sky region
- Result: hole reveals through to whatever is "behind" the sky
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
- Region bits (122..120) on bounds are unused - repurpose for COMPOSITE flag