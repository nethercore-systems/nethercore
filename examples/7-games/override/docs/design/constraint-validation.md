# OVERRIDE - ZX Constraint Validation Report

**Date:** 2025-12-28
**GDD Version:** 1.0
**Validator:** Design Phase Validation

---

## Executive Summary

**Overall Status:** PASS

The OVERRIDE design is well within ZX console constraints. The asymmetric multiplayer approach is technically feasible with rollback netcode, and the procedural facility generation will help stay under asset budgets.

---

## 1. Memory Constraints

### RAM Budget: 8 MB (1 MB reserved for netcode)

| System | Estimated Usage | Notes |
|--------|-----------------|-------|
| Facility Map | ~8 KB | 64 tiles x 128 bytes each |
| Entity States | ~2 KB | 4 players + 10 drones max |
| Trap States | ~1 KB | 20 traps max |
| Door States | ~512 B | 30 doors max |
| Game State | ~1 KB | Timer, scores, phase |
| Render Buffers | ~200 KB | Double-buffered 320x180 |
| Audio Buffers | ~512 KB | Streaming music + SFX |
| Asset Cache | ~2 MB | Loaded sprites/tiles |
| Netcode Reserve | 1 MB | Fixed allocation |
| **Total** | **~3.7 MB** | **OK** |

**Status:** PASS - Well under 8 MB limit

---

## 2. ROM Size Constraints

### WASM Code: 256 KB limit

| Module | Estimated Size | Notes |
|--------|----------------|-------|
| Core Game Loop | ~20 KB | Main update/render |
| Facility Generator | ~30 KB | Procedural generation |
| Entity System | ~25 KB | Players, drones, traps |
| AI (Drone Pathfinding) | ~15 KB | Simple A* |
| Netcode Integration | ~40 KB | Rollback handling |
| Input System | ~10 KB | 4-player input |
| Rendering | ~30 KB | Tile/sprite rendering |
| Audio System | ~10 KB | SFX/Music playback |
| UI System | ~15 KB | Menus, HUD |
| **Total** | **~195 KB** | **OK** |

**Status:** PASS - ~61 KB headroom

### Asset Storage: 768 KB limit

| Category | Estimated Size | Notes |
|----------|----------------|-------|
| Tilesets | ~50 KB | Floors, walls, doors, traps |
| Characters | ~30 KB | Runner (24 frames), Drone (4 frames) |
| Effects | ~20 KB | Particles, overlays |
| UI | ~30 KB | HUD, icons, menus |
| Fonts | ~20 KB | Two font sets |
| Sound Effects | ~150 KB | 16 effects compressed |
| Music | ~100 KB | 5 tracks compressed |
| **Total** | **~400 KB** | **OK** |

**Status:** PASS - ~368 KB headroom

---

## 3. Performance Constraints

### Frame Rate: 60 FPS target

| System | Per-Frame Budget | Estimated Cost | Notes |
|--------|------------------|----------------|-------|
| Input Processing | 0.5 ms | ~0.2 ms | 4 players |
| Game Logic | 4 ms | ~2 ms | Entity updates |
| Pathfinding | 2 ms | ~1 ms | 1-2 drones active |
| Collision | 2 ms | ~1 ms | Grid-based, fast |
| Rendering | 6 ms | ~4 ms | Tile-based |
| Audio | 1 ms | ~0.5 ms | Streaming |
| Netcode | 2 ms | ~1.5 ms | Rollback overhead |
| **Total** | **16.67 ms** | **~10.2 ms** | **OK** |

**Status:** PASS - Good performance margin

---

## 4. Multiplayer Constraints

### Rollback Netcode Requirements

| Requirement | Design Compliance | Notes |
|-------------|-------------------|-------|
| Deterministic Logic | YES | Fixed-point positions, seeded RNG |
| Input-Based State | YES | All state derived from inputs |
| Serializable State | YES | Flat struct design |
| Rollback Window | 8 frames | Sufficient for 133ms latency |

### Asymmetric View Handling

| Challenge | Solution | Feasibility |
|-----------|----------|-------------|
| Different cameras | Local render, shared state | Easy |
| Different UI | Role-based HUD rendering | Easy |
| Different inputs | Unified input struct | Easy |
| Information hiding | Render-time only, state shared | Medium |

**Status:** PASS - Asymmetric views are purely presentational

---

## 5. Input Constraints

### Controller Support

| Input | Runner | Overseer | Notes |
|-------|--------|----------|-------|
| D-Pad/Stick | Movement | Cursor | OK |
| A Button | Interact | Confirm action | OK |
| B Button | Sprint | Cancel | OK |
| X Button | Crouch | - | OK |
| Y Button | - | Cycle powers | OK |

**Status:** PASS - Standard gamepad layout works

---

## 6. Resolution Constraints

### Display: 320x180 (16:9)

| Element | Size | Visibility | Notes |
|---------|------|------------|-------|
| Facility Grid | 320x176 | Full screen | 8x8 tiles = 40x22 grid |
| Runner Sprite | 8x12 | Good | Clear silhouette |
| Drone Sprite | 6x6 | Acceptable | Small but distinct |
| Core Item | 8x8 | Good | Glowing helps |
| HUD Bar | 4px height | Tight | Bottom strip |

**Status:** PASS - Layout fits well

---

## 7. Audio Constraints

### Channels: 16 SFX + 1 Music

| Audio Type | Channels Used | Notes |
|------------|---------------|-------|
| Music | 1 | Background track |
| Footsteps | 2 | Pool and cycle |
| Doors | 2 | Quick sounds |
| Traps | 3 | High priority |
| Drones | 2 | Hovering loop |
| UI | 1 | Beeps, clicks |
| Stingers | 1 | Victory/defeat |
| Ambient | 2 | Background layers |
| **Total** | **14** | **OK** |

**Status:** PASS - 2 channels spare

---

## 8. Validation Summary

### Must Have (Blocking)
- [x] Total ROM under 1 MB
- [x] RAM usage under 8 MB
- [x] 60 FPS achievable
- [x] 4-player netcode compatible
- [x] Deterministic game logic
- [x] Resolution 320x180

### Resource Summary

| Constraint | Budget | Planned | Status |
|------------|--------|---------|--------|
| WASM Code | 256 KB | ~195 KB | PASS |
| Assets | 768 KB | ~400 KB | PASS |
| Total ROM | 1 MB | ~595 KB | PASS |
| RAM | 8 MB | ~3.7 MB | PASS |
| Frame Budget | 16.67 ms | ~10.2 ms | PASS |
| Audio Channels | 16 SFX + 1 Music | 14 SFX | PASS |

---

## 9. Conclusion

**OVERRIDE is approved for implementation on Nethercore ZX.**

The design fits comfortably within all technical constraints with significant headroom for iteration and polish.

---

*Validation complete. Proceed to asset generation phase.*
