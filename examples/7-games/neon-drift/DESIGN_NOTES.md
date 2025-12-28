# NEON DRIFT - Design Notes & Asset Critique

## Current Asset Critique

### Cars - Current Issues

| Car | Strengths | Issues |
|-----|-----------|--------|
| **Speedster** | Clean classic silhouette, good proportions | Capsule body lacks aggressive angles, headlights too boxy |
| **Muscle** | Distinct hood scoop, proper muscle car stance | Fenders too small, lacks exhaust detail prominence |
| **Racer** | Excellent F1 detail (wings, sidepods, helmet) | Front wing too simple, needs more aerodynamic elements |
| **Drift** | Good JDM hatch proportions, spoiler looks right | Wheel wells (torus) add complexity but low visual impact |

**Overall Car Critique:**
- All cars lack interior detail (windshield is opaque)
- Wheel designs are generic cylinders - could use spoke patterns
- Neon accent placement varies - should have consistent "light strips" philosophy
- Missing: Rear diffuser detail on non-Drift cars, side mirror stubs

### Tracks - Current Issues

| Track | Strengths | Issues |
|-------|-----------|--------|
| **track_straight** | Good barriers with neon strips | No road markings, no lane dividers |
| **track_curve_left** | Proper banked curve feel | Missing neon strips that straight has |
| **track_curve_right** | Mirrors left curve | No barriers at all, just road surface |
| **track_tunnel** | Great ring detail | No entry/exit lighting, rings float |
| **track_jump** | Good chevron hazard marking | Rails don't connect to ground, no landing zone |

**Overall Track Critique:**
- Inconsistent barrier treatment across segments
- No centerline markings on any road surface
- Missing: Starting grid texture, finish line, pit lane elements

### Textures - Current Issues

| Asset | Issue |
|-------|-------|
| All car textures | Good metal/gradient variety, but emissive maps are just solid colors |
| Track textures | Stone pattern for asphalt works but lacks road marking overlay |
| Prop textures | Adequate but building windows could be more varied |

### Props - Current Issues

| Prop | Issue |
|------|-------|
| **barrier** | Too thin, needs more visual weight |
| **boost_pad** | Good but arrows could be more dynamic (chevron shape) |
| **billboard** | Panel is too thin to read at distance |
| **building** | Good height, but only 1 building type (needs variety) |

---

## NEW CAR DESIGNS

### Car 5: PHANTOM - "The Ghost"
**Concept:** Stealth supercar, angular low-slung silhouette like Lamborghini meets stealth fighter

**Visual Design:**
- Long pointed nose with angular hood vents
- Extremely low roofline (0.22 height)
- Dramatic rear haunches over rear wheels
- Sharp wedge windshield angle (-35 degrees)
- Integrated rear spoiler (ducktail style)
- Twin exhaust tips
- Aggressive front splitter
- Side skirts with vent cutouts

**Color Scheme:**
- Base: Dark charcoal gray [40, 45, 50]
- Emissive: Toxic green [0, 255, 100]

**Stats:** SPD: 105% | ACC: 95% | HND: 90%
**Difficulty:** ★★☆ (Intermediate) - Well-balanced, rewards smooth driving

---

### Car 6: TITAN - "The Tank"
**Concept:** Heavy luxury GT cruiser, wide and imposing like a Bentley/Rolls tank

**Visual Design:**
- Wide boxy body (0.50 width)
- Tall hood with horizontal grille lines
- Upright windshield angle (-12 degrees)
- Long roof with thick C-pillars
- Pronounced front/rear bumpers
- Chunky wheel arches
- Twin exhaust stacks
- Subtle rear lip spoiler

**Color Scheme:**
- Base: Gunmetal silver [100, 105, 115]
- Emissive: Pure white [255, 255, 255]

**Stats:** SPD: 85% | ACC: 85% | HND: 75%
**Difficulty:** ★☆☆ (Beginner) - Heavy but forgiving, doesn't spin easily

---

### Car 7: VIPER - "The Strike"
**Concept:** Ultra-aggressive hypercar, extreme aerodynamics like Pagani meets McLaren extreme

**Visual Design:**
- Extreme wedge nose (nearly flat)
- Cockpit-forward design (like LMP cars)
- Massive rear wing on pylons
- Active aero elements (dive planes on front)
- Center-exit exhaust
- Side-mounted radiator inlets
- Dramatic rear diffuser fins
- Exposed suspension elements

**Color Scheme:**
- Base: Venom red [180, 20, 30]
- Emissive: Gold [255, 200, 0]

**Stats:** SPD: 120% | ACC: 75% | HND: 105%
**Difficulty:** ★★★ (Expert) - Fastest but requires precise inputs

---

## NEW TRACK DESIGNS

### Track 4: CRYSTAL CAVERN
**Concept:** Underground crystalline cave system with luminescent formations

**Difficulty:** ★★★☆ (Hard)
**Theme:** Deep underground, mysterious, otherworldly

**EPU Environment:**
```
Layer 0 (Background): Deep purple gradient
  - Zenith: 0x1A0533FF (deep purple)
  - Sky Horizon: 0x2D1B4EFF (dark violet)
  - Ground Horizon: 0x0D0221FF (near black)
  - Nadir: 0x000000FF (black)

Layer 1 (Crystal Scatter):
  - Variant: 2 (diamond shapes)
  - Density: 180
  - Size: 8
  - Colors: 0x00FFFFFF (cyan), 0xFF00FFFF (magenta)
  - High parallax for depth

Layer 2 (Stalactite Lines):
  - Variant: 1 (vertical)
  - Line type: 1 (dotted)
  - Colors: 0x8B5CF6FF (purple), 0x00FFFFFF (cyan)
```

**Track Features:**
- Tight S-curves through crystal formations
- Low ceiling sections (claustrophobic feel)
- Glowing crystal obstacles near track edges
- Boost pads hidden in alcoves

---

### Track 5: SOLAR HIGHWAY
**Concept:** Elevated highway in space, orbiting close to a star

**Difficulty:** ★★★★ (Expert)
**Theme:** Extreme speed, cosmic scale, solar energy

**EPU Environment:**
```
Layer 0 (Solar Background): Hot gradient
  - Zenith: 0xFFFFFFFF (pure white - sun)
  - Sky Horizon: 0xFFAA00FF (orange)
  - Ground Horizon: 0xFF4400FF (red-orange)
  - Nadir: 0x330000FF (dark red)
  - Rotation: 0.3 (tilted sun position)

Layer 1 (Solar Flare Scatter):
  - Variant: 0 (round particles)
  - Density: 100
  - Size: 12
  - Glow: 255 (maximum)
  - Colors: 0xFFFF00FF (yellow), 0xFF8800FF (orange)
  - Streak length: 30 (motion blur)

Layer 2 (Corona Rings):
  - Ring count: 30
  - Colors: 0xFFAA00FF, 0xFFFFAAFF
  - Center: White with falloff
  - Slight spiral twist
```

**Track Features:**
- Long high-speed straights
- Wide sweeping curves (requires lifting)
- Solar wind gusts (visual effect via scatter speed)
- Dramatic jump over solar flare

---

## DIFFICULTY SYSTEM

### Car Difficulty Ratings

| Car | Stars | Description |
|-----|-------|-------------|
| Titan | ★☆☆ | Heavy, stable, forgiving - perfect for beginners |
| Speedster | ★★☆ | Balanced handling, good for learning |
| Phantom | ★★☆ | Responsive but predictable |
| Muscle | ★★☆ | Power requires respect, slight oversteer |
| Drift | ★★☆ | Handling-focused, rewards aggressive driving |
| Racer | ★★★ | High grip but unforgiving at limits |
| Viper | ★★★ | Maximum speed, requires precision |

### Track Difficulty Ratings

| Track | Stars | Description |
|-------|-------|-------------|
| Sunset Strip | ★☆☆ | Wide roads, gentle curves |
| Neon City | ★★☆ | Moderate curves, good visibility |
| Void Tunnel | ★★★ | Disorienting visuals, tight spaces |
| Crystal Cavern | ★★★ | Low visibility, technical sections |
| Solar Highway | ★★★★ | High speed, requires throttle control |

---

## ROM BUDGET ESTIMATES

### Current Assets (Estimated)
- 4 cars: ~3,200 tris total
- 5 track segments: ~1,200 tris total
- 4 props: ~300 tris total
- 17 textures (128x128 avg): ~140 KB
- 11 sounds: ~100 KB

### New Assets
- 3 new cars: +2,400 tris
- 2 new track environments (EPU only, no new meshes)
- 3 new car textures + emissives: +30 KB
- 0 new sounds needed

### Total ROM Estimate: ~350 KB (well within ZX limits)

---

## IMPLEMENTATION PRIORITY

1. Add 3 new car generators (cars.rs)
2. Add new car textures (textures.rs)
3. Update nether.toml with new assets
4. Add new TrackId variants and EPU setups (lib.rs)
5. Update car/track selection UI with difficulty stars
6. Regenerate all assets
7. Test and tune
