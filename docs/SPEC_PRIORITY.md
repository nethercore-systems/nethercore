# Emberware Specs - Priority & Implementation Order

> Analysis of specs in `docs/ready/` for MVP readiness, 5th-gen console authenticity, and implementation priority.

## Requirements
- **Multiplayer**: MVP requirement (even with hardcoded IPs)
- **Save Slots**: Local save required for MVP; network sync is post-MVP. Core implementation, not console-specific.
- **Runtime Limits**: Frame skip for CPU/fuel exceeded; hard failure for OOM

---

## Final Ranking

| Priority | Spec | MVP? | Breaking? | 5th-Gen Feel | Effort |
|----------|------|------|-----------|--------------|--------|
| **1** | **Runtime Limits** | YES | No* | HIGH | ~6d |
| **2** | **Tracker Playback** | YES | No | VERY HIGH | ~11d |
| **3** | **Multi-Environment** | YES | No** | HIGH | ~15d |
| 4 | Synchronized Save Slots | Post-MVP | Yes | HIGH | ~14d |
| 5 | Replay System | Post-MVP | No | Medium | ~10d |
| 6 | Hot Reload | Post-MVP | No | N/A | ~8d |
| 7 | State Snapshots | Post-MVP | No | N/A | ~8d |
| 8 | Visual Debug Overlays | Post-MVP | No | N/A | ~9d |
| 9 | Network Simulator | Post-MVP | No | N/A | ~10d |
| 10 | Memory Viewer | Post-MVP | No | N/A | ~12d |

*Frame-skip is graceful degradation
**Legacy `sky_set_colors()` maps to Gradient mode

---

## MVP Specs

### 1. Runtime Limits (~6 days) - FOUNDATION

Sets the "rules of the console" before any games are built.

| Condition | Behavior |
|-----------|----------|
| CPU/Fuel exceeded | Frame skip (simulate struggling hardware) |
| OOM (heap exceeded) | Hard failure (real hardware crash) |

Implement in core, define limits per console in `ConsoleSpecs`. Debug panel shows live usage with color coding (yellow 80%, red 100%+).

**5th-Gen Authenticity:** Real consoles had hard limits. Constraints force creative solutions.

### 2. Tracker Playback (XM) (~11 days) - AUDIO IDENTITY

Music defines game feel. Most authentic 5th-gen feature.

- XM format = 25x smaller than PCM (3-minute song: ~320KB vs ~7.9MB)
- 32 channels, full effect support
- Samples from ROM data pack (reusable between music/SFX)
- Rollback-safe with row state caching

**5th-Gen Authenticity:** This IS how PS1/N64 music worked.

### 3. Multi-Environment (~15 days) - VISUAL IDENTITY

Defines the look of Emberware games. 8 procedural modes:

1. **Gradient** - 4-color sky/ground
2. **Scatter** - Stars, rain, particles
3. **Lines** - Synthwave grids
4. **Silhouette** - Layered terrain
5. **Rectangles** - Windows, light sources
6. **Room** - 3D interior box
7. **Curtains** - Pillars, trees
8. **Rings** - Tunnels, portals

Layering with blend modes (Alpha, Add, Multiply, Screen).

**5th-Gen Authenticity:** Procedural backgrounds were everywhere (limited VRAM).

---

## Post-MVP

### 4. Synchronized Save Slots (~14 days)
Network sync is complex; local save already works. Implement in **core**, not console-specific.

### 5-6. Hot Reload + State Snapshots (~16 days)
Work together for fast iteration cycles.

### 7. Visual Debug Overlays (~9 days)
Catch bugs visually before players do.

### 8. Replay System (~10 days)
Content creation enabler. Input-based = small files, shareable.

### 9-10. Network Simulator + Memory Viewer (~22 days)
Deep debugging tools. Valuable but not urgent.

---

## Breaking Change Summary

| Spec | Risk | Notes |
|------|------|-------|
| Runtime Limits | LOW | Graceful degradation (frame skip) or authentic crash (OOM) |
| Multi-Environment | NONE | Legacy `sky_set_colors()` maps to Gradient mode |
| Save Slots Sync | LOW | Local saves unaffected, network sync is additive |
| All others | NONE | Pure additions |

---

## Effort Summary

| Phase | Specs | Days |
|-------|-------|------|
| **MVP** | Runtime Limits, Tracker, Multi-Env | ~32 days |
| Post-MVP Priority | Save Sync, Hot Reload, Snapshots | ~30 days |
| Post-MVP Polish | Overlays, Replay, NetSim, MemViewer | ~41 days |
| **Total** | All 10 specs | ~103 days |

---

## Quick Reference

| Spec | 5th-Gen Feel | Quality Impact | Dev Experience |
|------|--------------|----------------|----------------|
| Runtime Limits | Authentic constraints | Consistency | Debug panel |
| Tracker Playback | **Most authentic** | **Music = feel** | - |
| Multi-Environment | Authentic visuals | **Visual variety** | - |
| Save Slots Sync | Memory card vibes | Expected feature | - |
| Replay System | Racing game era | Content creation | Debug integration |
| Hot Reload | - | Faster iteration | **Best DX** |
| State Snapshots | - | Faster iteration | Quick save/load |
| Debug Overlays | - | Bug prevention | Visual debugging |
| Network Simulator | - | Better netcode | Test without network |
| Memory Viewer | - | Deep debugging | Hex inspection |
