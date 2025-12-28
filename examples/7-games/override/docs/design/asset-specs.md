# OVERRIDE - Asset Specifications

**Version:** 1.0
**Generated From:** Game Design Document v1.0
**Target Platform:** Nethercore ZX

---

## Asset Budget Summary

| Category | Budget | Allocation | Remaining |
|----------|--------|------------|-----------|
| WASM Code | 256 KB | ~180 KB | 76 KB |
| Total Assets | 768 KB | ~400 KB | 368 KB |

### Asset Breakdown

| Type | Estimated Size | Priority |
|------|----------------|----------|
| Tilesets | 50 KB | Critical |
| Characters | 30 KB | Critical |
| Effects | 20 KB | High |
| UI Elements | 30 KB | High |
| Sound Effects | 150 KB | High |
| Music | 100 KB | Medium |
| Fonts | 20 KB | High |
| **Total** | **400 KB** | - |

---

## 1. Tileset Assets

### 1.1 Floor Tiles

**Specification:**
- Size: 8x8 pixels per tile
- Format: Indexed color (16 color palette)
- Style: Dark industrial metal

| Tile | ID | Description | Procgen Suitable |
|------|----|-------------|------------------|
| Metal Floor | floor_01 | Basic steel plating, subtle texture | Yes |
| Grate Floor | floor_02 | Metal grating with gaps | Yes |
| Panel Floor | floor_03 | Tech panels with seams | Yes |
| Damaged Floor | floor_04 | Cracked/worn metal | Yes |

**Color Palette (Floor):**
```
#1a1a2e - Dark base
#2d2d44 - Shadow
#3d3d5c - Mid tone
#4a4a6a - Highlight
#252538 - Accent dark
#353548 - Accent light
```

**SADL Hints:**
- Use perlin noise for surface variation
- Add rivet patterns for industrial feel
- Subtle wear patterns on edges

### 1.2 Wall Tiles

**Specification:**
- Size: 8x8 pixels per tile
- Format: Indexed color (16 color palette)
- Style: Industrial barriers, tech surfaces

| Tile | ID | Description | Procgen Suitable |
|------|----|-------------|------------------|
| Solid Wall | wall_01 | Basic steel wall | Yes |
| Window Wall | wall_02 | Wall with small viewport | Partial |
| Vent Wall | wall_03 | Wall with grating | Yes |
| Pipe Wall | wall_04 | Wall with pipes | Yes |
| Screen Wall | wall_05 | Wall with monitor | Partial |
| Door Frame | wall_06 | Wall segment for doors | Yes |

**Color Palette (Wall):**
```
#0f0f1a - Darkest
#1a1a2e - Dark
#2a2a3e - Mid-dark
#3a3a4e - Mid
#4a4a5e - Light
#5a5a6e - Highlight
#00ff88 - Screen glow (accent)
```

### 1.3 Door Tiles

**Specification:**
- Size: 8x16 pixels (double height for visibility)
- Format: Indexed color
- States: 3 (Open, Closed, Locked)

| State | ID | Description |
|-------|----|-------------|
| Open | door_open | Door retracted into frame |
| Closed | door_closed | Door blocking passage |
| Locked | door_locked | Door with red lock indicator |

**Color Accents:**
- Open: Neutral (match walls)
- Closed: Yellow trim (#ffcc00)
- Locked: Red indicator (#ff3333)

### 1.4 Trap Tiles

**Specification:**
- Size: 8x8 pixels base
- Format: Indexed color
- Must be visible but not obtrusive when inactive

| Trap | ID | States | Description |
|------|----|--------|-------------|
| Floor Spike | trap_spike | 3 | Retracted, Warning, Extended |
| Gas Vent | trap_gas | 2 | Inactive, Active (with particle) |
| Laser Emitter | trap_laser | 2 | Inactive, Active (with beam) |

**Spike Animation:**
- Frame 1: Flat/retracted (hidden)
- Frame 2: Yellow warning flash
- Frame 3: Spikes extended (red tips)

---

## 2. Character Sprites

### 2.1 Runner Character

**Specification:**
- Size: 8x12 pixels (tall, humanoid)
- Format: Indexed color (8 color palette)
- Animations: 5 sets

| Animation | Frames | Duration | Loop |
|-----------|--------|----------|------|
| Idle | 2 | 500ms each | Yes |
| Walk | 8 | 100ms each | Yes |
| Sprint | 8 | 75ms each | Yes |
| Crouch | 4 | 150ms each | Yes |
| Death | 2 | 200ms each | No |

**Total Frames:** 24

**Direction Handling:**
- 4 directions (N, S, E, W)
- Diagonal uses closest cardinal
- Sprite flip for E/W (save frames)

**Color Palette (Runner):**
```
#00ff88 - Primary (suit glow)
#00cc66 - Secondary
#008844 - Shadow
#1a1a2e - Dark accents
#ffffff - Visor highlight
#2d2d44 - Body base
```

### 2.2 Patrol Drone

**Specification:**
- Size: 6x6 pixels (small, flying)
- Format: Indexed color (6 colors)
- Animations: 1 set

| Animation | Frames | Duration | Loop |
|-----------|--------|----------|------|
| Hover | 4 | 150ms each | Yes |

**Color Palette (Drone):**
```
#ff3333 - Primary (danger)
#cc2222 - Secondary
#881111 - Shadow
#ffcc00 - Eye/sensor glow
#1a1a2e - Body dark
```

---

## 3. Visual Effects

### 3.1 Gas Cloud

**Specification:**
- Size: 16x16 pixels (fills partial room)
- Format: RGBA (transparency needed)
- Animation: 4 frames, loop

**Color:**
```
#00ff00 - Gas green (50% alpha)
#88ff88 - Highlight (30% alpha)
```

### 3.2 Laser Beam

**Specification:**
- Size: 1x8 pixels (vertical line, scalable)
- Format: Indexed
- Animation: 2 frames (flicker)

**Color:**
```
#ff0000 - Core
#ff6666 - Glow
```

### 3.3 Core Glow

**Specification:**
- Size: 8x8 pixels
- Format: RGBA
- Animation: 4 frames (pulse)

**Color:**
```
#00ffff - Cyan core
#88ffff - Bright pulse
#004444 - Dark phase
```

### 3.4 Sprint Dust

**Specification:**
- Size: 4x4 pixels (particle)
- Format: Indexed
- Animation: 3 frames (dissipate)

### 3.5 Elimination Flash

**Specification:**
- Size: 16x16 pixels
- Format: RGBA
- Animation: 4 frames (flash out)

**Color:**
```
#ffffff - Initial flash
#ff3333 - Red fade
Transparent - End
```

---

## 4. UI Elements

### 4.1 Energy Bar (Overseer)

**Specification:**
- Size: 64x6 pixels
- Format: Indexed
- States: Dynamic fill

**Colors:**
```
#1a1a2e - Frame
#00ffff - Full energy
#ffcc00 - Low energy (<30%)
#ff3333 - Critical (<10%)
```

### 4.2 Core Indicators

**Specification:**
- Size: 8x8 pixels per core (24x8 total)
- Format: Indexed
- States: Empty, Collected

### 4.3 Timer Display

**Specification:**
- Size: 32x8 pixels
- Format: Font-based
- Display: MM:SS format

**Colors:**
```
#ffffff - Normal
#ffcc00 - Warning (<60s)
#ff3333 - Critical (<30s)
```

### 4.4 Overseer Power Buttons

**Specification:**
- Size: 24x12 pixels each
- Format: Indexed
- States: Available, Cooldown, Insufficient Energy

**Icons Needed:**
| Power | Icon Symbol |
|-------|-------------|
| Door Lock | Padlock |
| Door Open | Open door |
| Lights | Light bulb |
| Trap | Exclamation |
| Drone | Drone silhouette |
| Alarm | Speaker |
| Ping | Eye |

### 4.5 Minimap (Overseer)

**Specification:**
- Size: 48x48 pixels
- Format: Procedurally rendered
- Shows: Room layout, Runner positions (dots)

---

## 5. Audio Assets

### 5.1 Sound Effects

| ID | Name | Duration | Priority | Description |
|----|------|----------|----------|-------------|
| sfx_footstep | Footstep | 100ms | Low | Metal footstep, subtle |
| sfx_footstep_sprint | Sprint Step | 80ms | Low | Louder, faster |
| sfx_door_open | Door Open | 300ms | Medium | Hydraulic hiss |
| sfx_door_close | Door Close | 300ms | Medium | Metal clank |
| sfx_door_locked | Door Locked | 200ms | Medium | Lock engage + denial beep |
| sfx_trap_spike | Spike Trap | 400ms | High | Metal scrape + impact |
| sfx_trap_gas | Gas Release | 800ms | High | Hiss + bubbling |
| sfx_trap_laser | Laser Active | 500ms | High | Electric hum |
| sfx_core_pickup | Core Collect | 500ms | High | Positive chime + energy |
| sfx_drone_spawn | Drone Deploy | 400ms | High | Mechanical deployment |
| sfx_drone_hover | Drone Hover | Loop | Medium | Electric hum |
| sfx_runner_death | Elimination | 600ms | Critical | Impact + static |
| sfx_alarm | Alarm Sound | 1000ms | Medium | Alert siren |
| sfx_lights_off | Lights Out | 300ms | Medium | Electric shutdown |
| sfx_timer_warning | Timer Tick | 200ms | High | Urgent beep |
| sfx_extraction_open | Extraction Ready | 800ms | Critical | Success fanfare |

### 5.2 Music Tracks

| ID | Name | Duration | Loop | Description |
|----|------|----------|------|-------------|
| mus_menu | Main Menu | 60s | Yes | Ambient, mysterious |
| mus_gameplay | Gameplay | 120s | Yes | Tense, building |
| mus_chase | Chase | 30s | Yes | Fast, urgent |
| mus_victory | Victory | 5s | No | Triumphant sting |
| mus_defeat | Defeat | 5s | No | Ominous sting |

---

## 6. Font Assets

### 6.1 Primary Font

**Specification:**
- Size: 5x7 pixels per character
- Characters: A-Z, 0-9, basic punctuation
- Style: Clean pixel font, readable at small size

### 6.2 Title Font

**Specification:**
- Size: Variable (stylized)
- Characters: "OVERRIDE" only (logo)
- Style: Bold, angular, sci-fi

---

## 7. Procedural Generation Recommendations

### 7.1 Textures (SADL)

**Floor Tiles - Recommended Approach:**
```
Algorithm: Perlin noise base + rivet overlay
Steps:
1. Generate 8x8 noise field (low frequency)
2. Map to palette indices (4 colors)
3. Add rivet pattern at corners
4. Apply edge darkening
```

**Wall Tiles - Recommended Approach:**
```
Algorithm: Panel-based with detail overlay
Steps:
1. Define panel grid (2x2 in 8x8)
2. Add seam lines between panels
3. Apply random detail (pipe, vent) with probability
4. Color map to palette
```

### 7.2 Characters (SADL)

**Runner - Recommended Approach:**
```
Base: 8x12 humanoid template
Steps:
1. Head (2x3 top region, visor highlight)
2. Body (6x6 center, suit color)
3. Legs (6x3 bottom, walking animation)
4. Add glow outline (1px)
```

### 7.3 Audio (Procedural)

**SFX - Recommended Synthesis:**
- Footsteps: White noise burst + low-pass filter
- Doors: FM synthesis (mechanical)
- Traps: Noise + pitch envelope
- UI: Simple waveform (sine/square) with ADSR

---

## 8. Asset Pipeline

### 8.1 Generation Order

1. **Color Palettes** - Define all palettes first
2. **Tilesets** - Floor, Wall, Door, Trap tiles
3. **Characters** - Runner, Drone sprites
4. **Effects** - Particles and overlays
5. **UI** - HUD elements and icons
6. **Audio** - SFX then Music
7. **Fonts** - Primary and Title

### 8.2 Integration Points

| Asset | Used By | Integration |
|-------|---------|-------------|
| Tilesets | Facility renderer | Tile IDs in map data |
| Characters | Entity system | Sprite sheets with metadata |
| Effects | VFX system | Particle definitions |
| UI | HUD system | Anchor points + scaling |
| Audio | Sound manager | Event triggers |

---

## 9. Quality Checklist

### Per-Asset Validation

- [ ] Within size budget
- [ ] Correct dimensions
- [ ] Palette compliant
- [ ] Visually distinct from similar assets
- [ ] Readable at 1:1 scale
- [ ] No aliasing artifacts
- [ ] Animation smooth (if applicable)

### Overall Validation

- [ ] Total asset size under 768 KB
- [ ] All gameplay-critical assets present
- [ ] Consistent art style across assets
- [ ] Color blind friendly alternatives exist
- [ ] All audio normalized to same level

---

*Document Version: 1.0*
*Last Updated: 2025-12-28*
*Status: Ready for Asset Generation*
