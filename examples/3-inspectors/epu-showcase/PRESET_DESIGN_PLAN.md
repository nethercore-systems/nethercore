# EPU v1 Inspector Preset Design Plan

This document tracks the 18 presets designed for the EPU inspector example refactor.

## Architecture Note

This document reflects the canonical EPU v1 architecture. The opcode slot assignments are:

| Opcode | Slot |
|--------|------|
| SECTOR | 0x02 |
| SILHOUETTE | 0x03 |
| SPLIT | 0x04 |
| LOBE | 0x12 |
| BAND | 0x13 |

**Coverage Gap:** The agent cataloged 51 unused variants across all opcodes that need preset coverage. The current 18 presets provide baseline coverage, but additional presets or preset variants should be added to achieve full opcode/variant coverage.

## Status Legend
- [ ] Not started
- [x] Implemented and verified

## Presets

### 1. "Neon Metropolis" - [ ]
**Genre:** Cyberpunk urban
- L0: RAMP (deep purple sky #1a0a2e, black floor #000000, dark gray walls #1c1c1c)
- L1: SILHOUETTE/CITY (black #000000, city skyline cutout on walls)
- L2: GRID (cyan #00ffff, walls only, vertical bars spacing=32)
- L3: SCATTER/WINDOWS (warm yellow #ffcc00, walls, building lights, density=128)
- L4: VEIL/LASER_BARS (magenta #ff00ff, vertical laser beams, intensity=180)
- L5: ATMOSPHERE/MIE (subtle gray haze #404050, intensity=60)
- L6: FLOW/STREAKS (cyan #00ddff, rain effect, speed=200, downward drift)
- L7: NOP

### 2. "Crimson Hellscape" - [ ]
**Genre:** Horror/demonic
- L0: RAMP (blood red sky #4a0000, charred black floor #0a0000, dark crimson walls #2a0808)
- L1: TRACE/CRACKS (orange-red #ff3300, floor/walls, volcanic fissures)
- L2: PATCHES/MEMBRANE (dark red #330000, organic tissue texture)
- L3: FLOW/NOISE (ember orange #ff4400, slowly churning lava glow)
- L4: SCATTER/EMBERS (bright orange #ff8800, rising sparks with upward drift)
- L5: ATMOSPHERE/ABSORPTION (blood mist #400000, thick fog)
- L6: CELESTIAL/ECLIPSE (black sun #000000 with red corona #ff0000)
- L7: PORTAL/RIFT (hellfire red #ff2200, dimensional tear on wall)

### 3. "Frozen Tundra" - [ ]
**Genre:** Arctic survival
- L0: RAMP (pale blue sky #c8e0f0, white floor #f8f8ff, ice blue walls #a0c8e0)
- L1: PLANE/STONE (ice white #e8f4ff, frozen ground texture)
- L2: CELL/SHATTER (pale cyan #d0f0ff, cracked ice pattern)
- L3: FLOW/NOISE (white #ffffff, slow drifting snow, octaves=3)
- L4: SCATTER/FLAKES (white #ffffff, snowfall with downward drift)
- L5: ATMOSPHERE/RAYLEIGH (arctic blue #b0d8f0, crisp cold air)
- L6: CELESTIAL/SUN (pale yellow #ffffd0, low winter sun)
- L7: NOP

### 4. "Alien Jungle" - [ ]
**Genre:** Sci-fi nature
- L0: RAMP (purple sky #3a0050, bioluminescent floor #002020, teal walls #004040)
- L1: SILHOUETTE/FOREST (dark teal #001818, alien tree silhouettes)
- L2: PATCHES/BLOBS (bioluminescent cyan #00ffaa, glowing fungal patches)
- L3: CELL/VORONOI (deep purple #200040, organic cell structure)
- L4: SCATTER/DUST (cyan #00ffcc, floating spores)
- L5: VEIL/CURTAINS (purple #8000ff, bioluminescent hanging vines)
- L6: ATMOSPHERE/ALIEN (green tint #004020, exotic gas atmosphere)
- L7: FLOW/CAUSTIC (cyan #00ddcc, rippling bioluminescence)

### 5. "Gothic Cathedral" - [ ]
**Genre:** Dark fantasy/religious
- L0: RAMP (deep blue sky #0a0a20, stone floor #1a1a1a, dark gray walls #202020)
- L1: APERTURE/ARCH (stained glass colors, gothic arch viewport)
- L2: GRID (dark stone #303030, walls, gothic window frames)
- L3: CELL/BRICK (gray #282828, stone wall texture)
- L4: TRACE/LEAD_LINES (black #000000, stained glass leading)
- L5: LOBE (golden #ffd700, shaft of divine light from above)
- L6: SCATTER/DUST (gold #ffcc00, dust motes in light beam)
- L7: ATMOSPHERE/MIE (incense haze #302820, smoky interior)

### 6. "Ocean Depths" - [ ]
**Genre:** Underwater exploration
- L0: RAMP (dark blue sky #001030, sandy floor #203040, deep teal walls #002848)
- L1: PLANE/WATER (blue #004080, rippling caustic floor)
- L2: FLOW/CAUSTIC (cyan #00a0c0, animated light patterns)
- L3: SCATTER/DUST (blue-green #40a0a0, floating particles/plankton)
- L4: VEIL/SHARDS (pale blue #80c0e0, light shafts from surface)
- L5: PATCHES/ISLANDS (dark blue #001828, distant underwater features)
- L6: ATMOSPHERE/ABSORPTION (deep blue #000820, water depth fog)
- L7: DECAL (circle, bioluminescent creature #00ffaa)

### 7. "Void Station" - [ ]
**Genre:** Sci-fi space station
- L0: RAMP (black sky #000008, dark metal floor #101018, gunmetal walls #181820)
- L1: SPLIT/HALF (two-tone walls: blue #002040 / gray #202028)
- L2: GRID (blue #0044aa, walls, technical panels)
- L3: CELL/GRID (dark blue #080820, floor grating pattern)
- L4: SCATTER/WINDOWS (white #ffffff, distant stars through viewport)
- L5: DECAL (rect, status indicator green #00ff00)
- L6: APERTURE/RECT (viewport frame, black edges)
- L7: CELESTIAL/PLANET (blue-green #00aa88, planet visible outside)

### 8. "Desert Mirage" - [x]
**Genre:** Middle Eastern fantasy
- L0: RAMP (bleached sky #f0e8d0, sand floor #d4b896, tan walls #c8a878)
- L1: SILHOUETTE/DUNES (golden #b89860, rolling sand dunes)
- L2: PLANE/SAND (warm sand #d8c090, textured desert floor)
- L3: FLOW/NOISE (heat shimmer #f8f0e0, slow wavering, low intensity)
- L4: SCATTER/DUST (sand #c8b080, blowing dust particles with wind drift)
- L5: CELESTIAL/SUN (blazing white #ffffd8, intense desert sun)
- L6: ATMOSPHERE/RAYLEIGH (haze #e8d8c0, heat distortion)
- L7: BAND (warm golden/orange #e8c090 / #d0a070, horizon heat shimmer)

### 9. "Neon Arcade" - [ ]
**Genre:** Retro arcade/synthwave
- L0: RAMP (black sky #000010, dark purple floor #100020, dark blue walls #080018)
- L1: GRID (magenta #ff00ff, floor, retro wireframe grid)
- L2: GRID (cyan #00ffff, walls, vertical scanlines)
- L3: SPLIT/BANDS (neon colors, horizontal color bands)
- L4: SCATTER/STARS (white #ffffff, background starfield)
- L5: CELESTIAL/SUN (magenta #ff0088, retro sun on horizon)
- L6: BAND (cyan #00ffff, horizon glow line)
- L7: FLOW/NOISE (purple #8000ff, subtle pulsing glow)

### 10. "Storm Front" - [ ]
**Genre:** Dramatic weather
- L0: RAMP (dark gray sky #202830, wet ground #181820, slate walls #303840)
- L1: SPLIT/WEDGE (sky division: dark #181820 / lighter gray #404850)
- L2: FLOW/NOISE (dark gray #404858, churning storm clouds, octaves=4)
- L3: TRACE/LIGHTNING (white #ffffff, sky, dramatic lightning bolts)
- L4: VEIL/RAIN_WALL (blue-gray #607080, heavy rain curtains)
- L5: SCATTER/FALL_DASHES (rain blue #8090a0, raindrops with downward drift)
- L6: ATMOSPHERE/FULL (storm gray #303038, thick storm atmosphere)
- L7: PLANE/PAVEMENT (wet gray #282830, rain-slicked ground)

### 11. "Crystal Cavern" - [ ]
**Genre:** Fantasy underground
- L0: RAMP (deep purple sky #100020, dark floor #080010, violet walls #180030)
- L1: CELL/VORONOI (crystal purple #400080, crystalline structure)
- L2: PATCHES/DEBRIS (amethyst #6020a0, scattered crystal formations)
- L3: TRACE/FILAMENTS (cyan #00e0ff, energy veins in crystals)
- L4: SCATTER/SPARKS (white #ffffff, glinting crystal facets)
- L5: LOBE (purple #a040ff, ambient crystal glow from below)
- L6: DECAL (ring, magic circle cyan #00ffff, floor)
- L7: ATMOSPHERE/ABSORPTION (purple mist #200040, cave atmosphere)

### 12. "War Zone" - [ ]
**Genre:** Military/apocalyptic
- L0: RAMP (smoke gray sky #383030, rubble floor #282020, charred walls #302820)
- L1: SILHOUETTE/RUINS (black #000000, destroyed building silhouettes)
- L2: SILHOUETTE/INDUSTRIAL (dark gray #181818, factory remnants)
- L3: PATCHES/DEBRIS (brown #483828, scattered rubble)
- L4: SCATTER/EMBERS (orange #ff6600, floating ash and embers)
- L5: FLOW/STREAKS (gray #606060, smoke trails)
- L6: ATMOSPHERE/ABSORPTION (thick smoke #302820, war smoke)
- L7: TRACE/CRACKS (black #000000, shattered ground)

### 13. "Enchanted Grove" - [ ]
**Genre:** Fairy tale forest
- L0: RAMP (golden sky #fff8d0, mossy floor #204020, forest green walls #1a3820)
- L1: SILHOUETTE/FOREST (deep green #0a2010, tree silhouettes)
- L2: PLANE/GRASS (vibrant green #308030, lush forest floor)
- L3: VEIL/CURTAINS (green #40a040, hanging moss/vines)
- L4: SCATTER/DUST (gold #ffdd00, fairy dust particles)
- L5: PATCHES/BLOBS (soft yellow #fff080, dappled sunlight)
- L6: LOBE (golden #ffd700, warm sunbeam through canopy)
- L7: FLOW/NOISE (green #60a060, gentle leaf movement)

### 14. "Astral Void" - [ ]
**Genre:** Cosmic/abstract
- L0: RAMP (void black #000004, deep purple floor #080010, indigo walls #100020)
- L1: FLOW/NOISE (nebula purple #4000a0, swirling cosmic gases, octaves=4)
- L2: SCATTER/STARS (white #ffffff, dense starfield, density=200)
- L3: CELESTIAL/GAS_GIANT (orange-red #ff6040, massive gas giant)
- L4: CELESTIAL/RINGED (pale gold #d0c080, ringed planet in distance)
- L5: PORTAL/VORTEX (blue #0080ff, cosmic wormhole)
- L6: TRACE/FILAMENTS (white #ffffff, cosmic energy streams)
- L7: ATMOSPHERE/ALIEN (purple tint #200040, exotic space dust)

### 15. "Toxic Wasteland" - [ ]
**Genre:** Post-apocalyptic industrial
- L0: RAMP (sickly green sky #304010, toxic floor #202008, corroded walls #283018)
- L1: SILHOUETTE/INDUSTRIAL (black #000000, rusted factory silhouettes)
- L2: PLANE/GRATING (rust #483820, industrial metal floor)
- L3: PATCHES/STATIC (green #40a000, radioactive patches)
- L4: CELL/HEX (toxic yellow #a0a000, hazmat pattern)
- L5: VEIL/PILLARS (green smoke #408020, rising toxic fumes)
- L6: SCATTER/DUST (yellow-green #a0c040, toxic particles)
- L7: ATMOSPHERE/ALIEN (toxic green #203008, poisonous atmosphere)

### 16. "Moonlit Graveyard" - [ ]
**Genre:** Gothic horror
- L0: RAMP (midnight blue sky #0a0a1a, dark earth floor #101010, slate walls #181820)
- L1: SILHOUETTE/SPIRES (black #000000, gothic tombstone silhouettes)
- L2: PLANE/STONE (gray #282828, weathered stone path)
- L3: PATCHES/MEMBRANE (dark green #0a1a0a, creeping moss)
- L4: SCATTER/DUST (pale blue #8090a0, floating mist particles)
- L5: CELESTIAL/MOON (pale silver #e0e8f0, full moon)
- L6: VEIL/CURTAINS (gray #404050, hanging mist)
- L7: ATMOSPHERE/MIE (blue fog #101828, ground fog)

### 17. "Volcanic Core" - [ ]
**Genre:** Primordial/elemental
- L0: RAMP (black sky #100800, magma floor #401000, obsidian walls #201008)
- L1: PLANE/STONE (volcanic black #181008, rough basalt floor)
- L2: TRACE/CRACKS (orange #ff4000, lava veins)
- L3: CELL/SHATTER (dark red #300800, cracked obsidian)
- L4: FLOW/NOISE (orange-red #ff2800, churning lava, speed=100)
- L5: SCATTER/EMBERS (bright orange #ff8000, rising sparks)
- L6: LOBE (deep red #ff2000, intense heat glow from below)
- L7: ATMOSPHERE/ABSORPTION (smoke black #100800, volcanic gases)

### 18. "Digital Matrix" - [ ]
**Genre:** Cyber/virtual reality
- L0: RAMP (black #000000, dark green floor #001000, matrix green walls #002000)
- L1: GRID (bright green #00ff00, all regions, digital grid)
- L2: SCATTER/FALL_DASHES (green #00ff00, falling code rain with downward drift)
- L3: CELL/GRID (dark green #003000, data block structure)
- L4: TRACE/FILAMENTS (cyan #00ffff, data streams)
- L5: PORTAL/RECT (green #00ff00, rectangular data portal)
- L6: APERTURE/BARS (black bars, digital scanlines)
- L7: FLOW/STREAKS (green #00dd00, code streaming effect)

## Opcode Coverage Matrix (EPU v1)

| Opcode | Slot | Count | Presets | Needs Update |
|--------|------|-------|---------|--------------|
| RAMP | 0x00 | 18 | All | - |
| SECTOR | 0x02 | 0 | - | Add coverage |
| SILHOUETTE | 0x03 | 8 | 1, 4, 8, 12, 13, 15, 16 | Verify slot |
| SPLIT | 0x04 | 2 | 7, 9, 10 | Verify slot |
| LOBE | 0x12 | 4 | 5, 11, 13, 17 | Verify slot |
| BAND | 0x13 | 2 | 8, 9 | Verify slot |
| CELL | - | 8 | 3, 4, 5, 7, 11, 15, 17, 18 | - |
| PATCHES | - | 8 | 2, 4, 6, 11, 12, 13, 15, 16 | - |
| APERTURE | - | 3 | 5, 7, 18 | - |
| DECAL | - | 3 | 6, 7, 11 | - |
| GRID | - | 6 | 1, 5, 7, 9, 18 | - |
| SCATTER | - | 17 | Most presets | - |
| FLOW | - | 13 | Most presets | - |
| TRACE | - | 8 | 2, 5, 10, 11, 12, 14, 17, 18 | - |
| VEIL | - | 6 | 1, 4, 6, 10, 13, 15, 16 | - |
| ATMOSPHERE | - | 14 | Most presets | - |
| PLANE | - | 8 | 3, 6, 8, 10, 13, 15, 16, 17 | - |
| CELESTIAL | - | 8 | 2, 3, 7, 8, 9, 14, 16 | - |
| PORTAL | - | 3 | 2, 14, 18 | - |

## Coverage Gaps

**51 unused variants identified** - The following need additional preset coverage:

- SECTOR (0x02): No current presets use this opcode
- Various sub-variants across CELL, PATCHES, SCATTER, FLOW, etc.

To achieve full coverage, consider adding specialized presets or expanding existing ones to exercise all variant combinations.
