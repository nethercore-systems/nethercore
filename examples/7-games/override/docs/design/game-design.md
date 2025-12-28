# OVERRIDE - Game Design Document

**Version:** 1.0
**Target Platform:** Nethercore ZX
**Genre:** Asymmetric Multiplayer (1v3)
**Players:** 4 (1 Overseer + 3 Runners)
**Round Duration:** 3-5 minutes
**Status:** Design Complete

---

## Executive Summary

OVERRIDE is a 4-player asymmetric multiplayer game where one player (the Overseer) controls a procedurally-generated facility while three players (the Runners) attempt to collect data cores and escape. The Overseer has a god-view and controls doors, traps, lights, and security drones, but is limited by an energy system. Runners have a close chase-cam with limited visibility, relying on stealth, speed, and teamwork to survive.

The game showcases ZX's rollback netcode with asymmetric game states, procedural generation for infinite replayability, and creates viral "almost got you" moments perfect for streaming and social sharing.

---

## 1. Core Concept

### 1.1 High Concept
"One player sees everything. Three players see nothing. Who will outsmart whom?"

### 1.2 Pillars
1. **Asymmetric Tension** - Drastically different perspectives create unique gameplay for each role
2. **Mind Games** - Energy management forces Overseer to think strategically; Runners must read and bait
3. **Quick Thrills** - 3-5 minute rounds enable "one more game" loops
4. **Procedural Variety** - Every facility is different, preventing memorization meta

### 1.3 Target Experience
- **Overseer:** Feels like a puppet master, enjoying the cat-and-mouse hunt
- **Runners:** Feels tense and claustrophobic, celebrating narrow escapes
- **Both:** Creates memorable stories ("Remember when you almost had me?")

---

## 2. Gameplay

### 2.1 Match Structure

```
MATCH FLOW
──────────────────────────────────────────────────
1. Lobby (4 players join)
2. First Overseer selected (random)
3. Facility generated (procedural)
4. Round begins:
   - Runners spawn at entry point
   - 3 Data Cores placed in facility
   - Timer starts (4 minutes)
5. Round ends when:
   - Runners extract with all 3 cores (Runner Win)
   - All Runners eliminated (Overseer Win)
   - Timer expires (Overseer Win)
6. Overseer rotates (loser becomes next Overseer)
7. Repeat until match victory condition

Match Victory: First player to win 3 rounds
──────────────────────────────────────────────────
```

### 2.2 Runner Gameplay

**Perspective:** Close chase-cam (third-person, camera behind and slightly above character)
- Limited forward field of view (~90 degrees)
- Cannot see behind without turning
- Creates tension from unknown threats

**Abilities:**
- **Move** - 8-directional movement at consistent speed
- **Sprint** - Temporarily faster, but creates noise (visible to Overseer)
- **Crouch** - Slower, quieter, smaller silhouette
- **Interact** - Pick up data cores, open manual doors, activate switches

**Objectives:**
1. Locate and collect 3 Data Cores scattered in the facility
2. Reach the Extraction Point with all cores collected
3. Survive Overseer's traps and security measures

**Team Dynamics:**
- Runners share core collection progress (any runner picking up a core counts)
- No revival - eliminated runners spectate until round ends
- Communication via proximity (if in-game voice) or external

### 2.3 Overseer Gameplay

**Perspective:** Top-down god-view of entire facility
- Can see all rooms, corridors, and Runner positions
- Can see Data Core locations
- Can see own trap/drone placements and cooldowns

**Energy System:**
- Maximum Energy: 100 units
- Regeneration: 5 units/second
- All actions cost energy
- Creates strategic decision-making

**Powers:**

| Power | Energy Cost | Cooldown | Effect |
|-------|-------------|----------|--------|
| **Lock Door** | 10 | 2s | Locks a door for 5 seconds |
| **Open Door** | 5 | 1s | Forces a door open |
| **Lights Off** | 15 | 5s | Darkens a room for 8 seconds |
| **Activate Trap** | 20 | 10s | Triggers a pre-placed trap |
| **Spawn Drone** | 30 | 15s | Deploys a patrol drone |
| **Sound Alarm** | 10 | 8s | Creates noise at location (distraction) |
| **Camera Ping** | 5 | 3s | Highlights Runners briefly |

**Strategy:**
- Must manage energy pool vs regeneration
- Runners can bait energy expenditure
- Timing matters - save energy for critical moments
- Combo potential (lights off + spawn drone)

### 2.4 Win Conditions

**Runners Win:**
- Collect all 3 Data Cores AND
- At least one Runner reaches Extraction Point

**Overseer Wins:**
- Eliminate all 3 Runners OR
- Timer expires (4:00) before extraction

---

## 3. Procedural Facility Generation

### 3.1 Generation Algorithm

**Tile-Based Grid System:**
- Facility is 8x8 grid of room tiles
- Each tile is 40x22 pixels (320/8 x 176/8)
- Total playable area: 320x176 (bottom 4 pixels for minimal UI)

**Room Types:**

| Type | Frequency | Features |
|------|-----------|----------|
| **Corridor** | 40% | Simple passage, 2-4 exits |
| **Storage** | 20% | Obstacles, potential core location |
| **Control Room** | 10% | Central hub, multiple exits |
| **Security** | 15% | Pre-placed traps, cameras |
| **Server Room** | 10% | Guaranteed core location |
| **Extraction** | 1 per map | Single exit point |
| **Entry** | 1 per map | Runner spawn point |

**Generation Rules:**
1. Place Entry room at random edge
2. Place Extraction room at opposite edge (guaranteed distance)
3. Place 3 Server Rooms (core locations) spread across facility
4. Fill remaining with weighted random rooms
5. Ensure path connectivity (flood fill validation)
6. Place doors between adjacent rooms
7. Distribute pre-placed traps in Security rooms

### 3.2 Core Placement
- 3 cores always in Server Rooms
- Server Rooms placed to ensure:
  - Minimum 4 rooms apart from each other
  - Minimum 3 rooms from Entry
  - Random but validated positions

### 3.3 Trap Placement
- Security rooms have 1-2 pre-placed traps
- Trap types: Floor spike, Gas vent, Laser grid
- Overseer activates but doesn't place (fair for Runners)

---

## 4. Entities

### 4.1 Runner (Player Character)

```
Stats:
- Walk Speed: 40 pixels/second
- Sprint Speed: 70 pixels/second
- Crouch Speed: 20 pixels/second
- Hitbox: 8x12 pixels
- Health: 1 (one-hit elimination)
```

**States:**
- Idle
- Walking (8 directions)
- Sprinting (creates noise indicator)
- Crouching
- Interacting (brief animation)
- Eliminated (death animation, then spectate)

### 4.2 Patrol Drone

```
Stats:
- Speed: 30 pixels/second
- Detection Range: 32 pixels (line of sight)
- Hitbox: 6x6 pixels
- Lifetime: 15 seconds after spawn
```

**Behavior:**
1. Spawns at Overseer-selected location
2. Patrols toward nearest Runner (pathfinding)
3. On contact: Eliminates Runner
4. Despawns after lifetime or on elimination

### 4.3 Traps

**Floor Spike:**
- Activation: Instant
- Effect: Eliminates Runner on tile
- Visual: Brief warning flash (0.3s) before activation
- Reset: 5 seconds

**Gas Vent:**
- Activation: 0.5s delay
- Effect: Fills room with gas for 3 seconds
- Damage: Eliminates Runners in room after 1.5s exposure
- Visual: Green gas particles

**Laser Grid:**
- Activation: Instant
- Effect: Blocks doorway with damaging lasers
- Duration: 4 seconds
- Visual: Red laser lines

### 4.4 Doors

**States:**
- Open (default)
- Closed (Runner can open manually - 1s interaction)
- Locked (Overseer only - 5s duration, then auto-unlocks)

---

## 5. Visual Design

### 5.1 Art Direction

**Style:** Dark sci-fi industrial
- Muted color palette (grays, dark blues, steel)
- High contrast for readability
- Accent colors for gameplay elements:
  - Cyan: Data Cores, Extraction Point
  - Red: Traps, Danger zones
  - Yellow: Doors, Interactive elements
  - Green: Runners (friendly indicators)

**Resolution:** 320x180 native (ZX standard)

### 5.2 Asset List

**Tilesets:**
| Asset | Size | Variants | Notes |
|-------|------|----------|-------|
| Floor tiles | 8x8 | 4 | Metal, grate, panel, damaged |
| Wall tiles | 8x8 | 6 | Solid, window, vent, pipe, screen, door frame |
| Door | 8x16 | 3 states | Open, closed, locked |
| Trap indicators | 8x8 | 3 | Spike, vent, laser |

**Characters:**
| Asset | Size | Frames | Notes |
|-------|------|--------|-------|
| Runner | 8x12 | 24 | Idle(2), Walk(8), Sprint(8), Crouch(4), Death(2) |
| Drone | 6x6 | 4 | Hover animation loop |

**Effects:**
| Asset | Size | Frames | Notes |
|-------|------|--------|-------|
| Gas cloud | 16x16 | 4 | Loop animation |
| Laser beam | 1x8 | 2 | Flicker effect |
| Spike | 8x8 | 3 | Retract, extend, hold |
| Core glow | 8x8 | 4 | Pulse animation |
| Sprint dust | 4x4 | 3 | Particle effect |

**UI Elements:**
| Asset | Size | Notes |
|-------|------|-------|
| Energy bar | 64x6 | Overseer HUD |
| Core indicator | 8x8 x3 | Shows collected cores |
| Timer | 32x8 | Countdown display |
| Minimap frame | 48x48 | Overseer only |

### 5.3 Camera Systems

**Runner Camera (Chase-Cam):**
```
- Position: 24 pixels behind Runner, 16 pixels above
- View cone: ~90 degrees forward
- Smooth follow with slight lag (0.1s lerp)
- Screen shows: 160x90 pixel area (half of full res)
- Black fog outside view cone
```

**Overseer Camera (God-View):**
```
- Position: Centered on facility
- Shows: Entire 320x176 facility
- Zoom: Fixed 1:1
- Can click/select anywhere
```

---

## 6. Audio Design

### 6.1 Sound Direction

**Atmosphere:** Industrial tension
- Ambient: Low hum, distant machinery, occasional radio static
- Diegetic: Footsteps, door mechanisms, trap activations
- Non-diegetic: Tension stingers, victory/defeat fanfares

### 6.2 Sound Effects List

| Sound | Trigger | Priority |
|-------|---------|----------|
| Runner footstep | Walk/Sprint | Low |
| Door open/close | Door state change | Medium |
| Door locked | Lock attempt | Medium |
| Trap activation | Overseer triggers trap | High |
| Core pickup | Runner collects core | High |
| Drone hover | Drone active | Medium |
| Drone spawn | Overseer spawns drone | High |
| Runner eliminated | Death | Critical |
| Alarm | Overseer sounds alarm | Medium |
| Lights off | Room darkened | Medium |
| Extraction open | All cores collected | Critical |
| Victory fanfare | Round end | Critical |
| Defeat fanfare | Round end | Critical |
| Timer warning | 30s, 10s remaining | High |

### 6.3 Music

**Track List:**
1. **Main Menu** - Mysterious, anticipatory (loop)
2. **Gameplay Tension** - Building dread, adaptive layers (loop)
3. **Chase** - Fast-paced, triggered when drone pursues (dynamic)
4. **Victory Sting** - Short triumphant (one-shot)
5. **Defeat Sting** - Short ominous (one-shot)

---

## 7. Multiplayer & Netcode

### 7.1 Architecture

**Rollback Netcode Requirements:**
- All game state must be deterministic
- Fixed-point math for positions
- Seeded RNG for procedural generation
- Input-based state (no physics randomness)

**State Synchronization:**
```rust
struct GameState {
    // Facility (generated once, shared seed)
    facility_seed: u64,

    // Runners
    runners: [RunnerState; 3],

    // Overseer
    overseer_energy: FixedPoint,
    active_effects: Vec<Effect>,

    // Collectibles
    cores_collected: u8,
    core_positions: [Option<Position>; 3],

    // Timing
    round_timer: u32, // frames remaining

    // Entities
    drones: Vec<DroneState>,
    trap_states: Vec<TrapState>,
    door_states: Vec<DoorState>,
}
```

### 7.2 Input Handling

**Runner Inputs (per frame):**
```rust
struct RunnerInput {
    move_x: i8,      // -1, 0, 1
    move_y: i8,      // -1, 0, 1
    sprint: bool,
    crouch: bool,
    interact: bool,
}
```

**Overseer Inputs (per frame):**
```rust
struct OverseerInput {
    action: Option<OverseerAction>,
    target_tile: Option<(u8, u8)>,
}

enum OverseerAction {
    LockDoor,
    OpenDoor,
    LightsOff,
    ActivateTrap,
    SpawnDrone,
    SoundAlarm,
    CameraPing,
}
```

### 7.3 Asymmetric Rendering

**Challenge:** Different players see different views

**Solution:**
- Core game state is identical across all clients
- Rendering is local and role-dependent
- Runners only render their visible area (fog of war)
- Overseer renders full facility

```rust
fn render(state: &GameState, local_player: PlayerId) {
    match get_role(local_player) {
        Role::Overseer => render_godview(state),
        Role::Runner(idx) => render_chasecam(state, idx),
    }
}
```

---

## 8. Technical Specifications

### 8.1 ZX Resource Budget

| Resource | Budget | Estimated Use | Remaining |
|----------|--------|---------------|-----------|
| WASM Code | 256 KB | ~180 KB | 76 KB |
| Assets | 768 KB | ~400 KB | 368 KB |
| RAM | 8 MB | ~4 MB | 4 MB |
| Netcode Reserve | 1 MB | 1 MB (fixed) | 0 |

### 8.2 Asset Budget Breakdown

| Category | Budget | Items |
|----------|--------|-------|
| Tilesets | 50 KB | Floor, walls, doors, traps |
| Characters | 30 KB | Runner (24 frames), Drone (4 frames) |
| Effects | 20 KB | Gas, laser, spike, particles |
| UI | 30 KB | HUD elements, fonts, icons |
| Audio SFX | 150 KB | ~15 sound effects |
| Audio Music | 100 KB | 5 tracks (compressed) |
| **Total** | **380 KB** | Well under 768 KB limit |

### 8.3 Performance Targets

- **Frame Rate:** 60 FPS locked
- **Input Latency:** <3 frames (50ms)
- **Rollback Window:** 8 frames
- **Netcode Tick Rate:** 60 Hz

---

## 9. User Interface

### 9.1 Screens

**Main Menu:**
```
┌────────────────────────────────────────┐
│                                        │
│            O V E R R I D E             │
│                                        │
│           [ PLAY ONLINE ]              │
│           [ LOCAL GAME ]               │
│           [  SETTINGS  ]               │
│                                        │
└────────────────────────────────────────┘
```

**Runner HUD:**
```
┌────────────────────────────────────────┐
│ CORES: [■][■][□]          TIME: 3:42   │
│                                        │
│                                        │
│           (Chase-cam view)             │
│                                        │
│                                        │
└────────────────────────────────────────┘
```

**Overseer HUD:**
```
┌────────────────────────────────────────┐
│ ENERGY: [████████░░]      TIME: 3:42   │
├───────────────────────────┬────────────┤
│                           │ [DOOR]  10 │
│                           │ [LIGHT] 15 │
│    (God-view facility)    │ [TRAP]  20 │
│                           │ [DRONE] 30 │
│                           │ [ALARM] 10 │
│                           │ [PING]   5 │
└───────────────────────────┴────────────┘
```

---

## 10. Accessibility

### 10.1 Visual
- High contrast mode option
- Colorblind-friendly indicators (shapes + colors)
- Screen shake toggle

### 10.2 Audio
- Visual indicators for all audio cues
- Subtitles for any voice/alert text
- Adjustable SFX/Music volumes

### 10.3 Controls
- Rebindable keys
- Controller support with customizable layout
- One-handed play mode consideration

---

## 11. Development Milestones

### Phase 1: Core Prototype
- [ ] Procedural facility generation
- [ ] Runner movement and camera
- [ ] Overseer view and basic powers
- [ ] Core collection and extraction
- [ ] Basic win conditions

### Phase 2: Polish
- [ ] Full trap implementation
- [ ] Drone AI and pathfinding
- [ ] Audio integration
- [ ] UI implementation
- [ ] Visual effects

### Phase 3: Multiplayer
- [ ] Rollback netcode integration
- [ ] Lobby system
- [ ] Match flow (round rotation)
- [ ] Spectator mode

### Phase 4: Release
- [ ] Performance optimization
- [ ] Accessibility features
- [ ] Platform submission
- [ ] Marketing assets

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Overseer** | The player controlling the facility (1 per match) |
| **Runner** | Players attempting to escape (3 per match) |
| **Data Core** | Collectible objective items (3 per round) |
| **Extraction Point** | Exit location where Runners win |
| **Chase-Cam** | Third-person camera following Runner |
| **God-View** | Top-down view of entire facility |
| **Drone** | AI-controlled enemy spawned by Overseer |

---

## Appendix B: Comparable Games

| Game | Similarity | Difference |
|------|------------|------------|
| Dead by Daylight | 1v4 asymmetric | DBD is 3D, longer matches, horror focus |
| Among Us | Social deduction | Among Us has hidden roles, no active control |
| Crawl | Asymmetric dungeon | Crawl rotates villain role on death |
| SpyParty | 1v1 asymmetric | SpyParty is about blending in |

OVERRIDE differentiates through:
- Quick round times (3-5 min vs 10-20 min)
- Energy-based power management
- Procedural facilities
- Retro aesthetic

---

*Document Version: 1.0*
*Last Updated: 2025-12-28*
*Status: Ready for Implementation*
