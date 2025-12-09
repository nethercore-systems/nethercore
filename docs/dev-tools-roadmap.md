# Developer Tools Roadmap

This document ranks and prioritizes developer-facing debug and development tools for Emberware.

## Priority Tiers

### Tier 1: MVP / High Priority
Essential tools for efficient game development. Implement before or shortly after MVP.

### Tier 2: Important
Significant productivity boosters. Implement post-MVP.

### Tier 3: Nice to Have
Useful but not critical. Implement when time permits.

---

## Feature Rankings

| Rank | Feature | Tier | Effort | Spec |
|------|---------|------|--------|------|
| 1 | **Debug Inspection Panel** | MVP | ~15 days | [debug-inspection-spec.md](./debug-inspection-spec.md) |
| 2 | **State Snapshots** | MVP | ~8 days | [state-snapshots-spec.md](./state-snapshots-spec.md) |
| 3 | **WASM Hot Reload** | High | ~8 days | [hot-reload-spec.md](./hot-reload-spec.md) |
| 4 | **Visual Debug Overlays** | High | ~9 days | [visual-debug-overlays-spec.md](./visual-debug-overlays-spec.md) |
| 5 | **Replay System** | Important | ~10 days | [replay-system-spec.md](./replay-system-spec.md) |
| 6 | **Network Condition Simulator** | Important | ~10 days | [network-simulator-spec.md](./network-simulator-spec.md) |
| 7 | **Memory Viewer** | Important | ~12 days | [memory-viewer-spec.md](./memory-viewer-spec.md) |
| 8 | Performance Timeline | Nice to Have | ~15 days | Not written |
| 9 | Log Viewer / Console | Nice to Have | ~6 days | Not written |
| 10 | Asset Hot Reload | Nice to Have | ~5 days | Not written |

---

## Tier 1: MVP / High Priority

### 1. Debug Inspection Panel
**Priority:** MVP
**Effort:** ~15 days
**Spec:** [debug-inspection-spec.md](./debug-inspection-spec.md) + [Implementation Plan](./debug-inspection-implementation-plan.md)

**Why MVP:**
- Core debugging capability - inspect any game variable in real-time
- Frame stepping and time control for precise debugging
- Foundation for other debug tools
- Already fully specified and planned

**Key Features:**
- Register variables from game code via FFI
- Real-time value display with graphs
- Frame stepping, pause, time scale control
- Change detection and callbacks
- Groups and categories

---

### 2. State Snapshots (Quick Save/Load)
**Priority:** MVP
**Effort:** ~8 days
**Spec:** [state-snapshots-spec.md](./state-snapshots-spec.md)

**Why MVP:**
- Leverages existing `save_state`/`load_state` (minimal new code)
- Massive iteration speedup - retry tricky sections instantly
- Essential for bug reproduction
- Low effort, high impact

**Key Features:**
- Quick slots (F5/F9)
- Named snapshots with thumbnails
- Auto-save
- Snapshot browser UI

---

### 3. WASM Hot Reload
**Priority:** High
**Effort:** ~8 days
**Spec:** [hot-reload-spec.md](./hot-reload-spec.md)

**Why High Priority:**
- Biggest single iteration speedup (changes visible in <1 second)
- Leverages existing save/load infrastructure
- Transforms development workflow
- Medium complexity

**Key Features:**
- File watcher for `rom.wasm`
- Save state, swap module, restore state
- Compilation error handling
- Graceful state incompatibility handling

---

### 4. Visual Debug Overlays
**Priority:** High
**Effort:** ~9 days
**Spec:** [visual-debug-overlays-spec.md](./visual-debug-overlays-spec.md)

**Why High Priority:**
- Essential for spatial debugging (collision, AI, paths)
- Complements Debug Inspection (data vs visualization)
- Standard game dev tool
- Category toggling allows production builds to strip

**Key Features:**
- 3D primitives (boxes, spheres, lines, cones)
- Category system with toggles
- Style customization (color, transparency, depth test)
- 2D screen-space overlays

---

## Tier 2: Important

### 5. Replay System
**Priority:** Important
**Effort:** ~10 days
**Spec:** [replay-system-spec.md](./replay-system-spec.md)

**Why Important:**
- "Free" due to GGRS determinism (just record inputs)
- Bug reproduction with exact input sequences
- Content creation (trailers, tutorials)
- Regression testing automation

**Key Features:**
- Record/playback inputs
- Periodic checkpoints for fast seeking
- Timeline UI with scrubbing
- Playback controls (pause, step, speed)

---

### 6. Network Condition Simulator
**Priority:** Important
**Effort:** ~10 days
**Spec:** [network-simulator-spec.md](./network-simulator-spec.md)

**Why Important:**
- Critical for netcode quality assurance
- Test rollback behavior without real bad connections
- Presets for common scenarios
- Local multiplayer testing on single machine

**Key Features:**
- Latency, jitter, packet loss simulation
- Preset network profiles
- Live rollback metrics
- Asymmetric conditions

---

### 7. Memory Viewer
**Priority:** Important
**Effort:** ~12 days
**Spec:** [memory-viewer-spec.md](./memory-viewer-spec.md)

**Why Important:**
- Low-level debugging capability
- Find memory corruption, understand layouts
- Search for values (cheat engine style)
- Compare snapshots to find changes

**Key Features:**
- Hex dump view
- Search and filter
- Bookmarks and watches
- Snapshot comparison

---

## Tier 3: Nice to Have

### 8. Performance Timeline
**Priority:** Nice to Have
**Effort:** ~15 days (estimated)
**Spec:** Not written

**Description:**
Chrome DevTools-style timeline showing frame breakdown:
- WASM execution time
- FFI call overhead
- GPU render time
- Audio processing
- Frame budget visualization

**Why Nice to Have:**
- Complex to implement accurately
- Requires careful instrumentation
- Most games won't need this level of analysis
- Debug overlay FPS counter may be sufficient

---

### 9. Log Viewer / Console
**Priority:** Nice to Have
**Effort:** ~6 days (estimated)
**Spec:** Not written

**Description:**
In-emulator console showing game logs:
- `debug_log()` FFI function
- Log levels (debug, info, warn, error)
- Filtering and search
- Timestamps and frame numbers

**Why Nice to Have:**
- Games can already use browser console / stdout
- Debug Inspection provides structured alternative
- Useful but not essential

---

### 10. Asset Hot Reload
**Priority:** Nice to Have
**Effort:** ~5 days (estimated)
**Spec:** Not written

**Description:**
Hot reload textures, audio, and other assets without code changes:
- Watch asset files
- Reload on change
- Update resource manager handles

**Why Nice to Have:**
- Code hot reload covers most iteration needs
- Asset pipeline varies by game
- Can add later when needed

---

## Implementation Order Recommendation

### Phase 1: Core Debug Tools (MVP)
1. **Debug Inspection Panel** - Foundation for all debugging
2. **State Snapshots** - Low effort, high impact

### Phase 2: Iteration Speed (Post-MVP)
3. **Hot Reload** - Transforms development workflow
4. **Visual Debug Overlays** - Essential spatial debugging

### Phase 3: Testing & Analysis
5. **Replay System** - Bug reproduction, testing
6. **Network Simulator** - Netcode quality assurance
7. **Memory Viewer** - Deep debugging

### Phase 4: Polish (When Time Permits)
8. Performance Timeline
9. Log Viewer
10. Asset Hot Reload

---

## Effort Summary

| Tier | Features | Total Effort |
|------|----------|--------------|
| MVP/High Priority | 4 features | ~40 days |
| Important | 3 features | ~32 days |
| Nice to Have | 3 features | ~26 days |
| **Total** | **10 features** | **~98 days** |

---

## Dependencies

```
Debug Inspection Panel
         │
         ├──► Visual Debug Overlays (uses category system)
         │
         ├──► Memory Viewer (cross-references with inspection)
         │
         └──► State Snapshots (integrates with frame control)
                    │
                    └──► Hot Reload (uses save/load)
                              │
                              └──► Replay System (uses determinism)

Network Simulator ──► (standalone, requires GGRS)
```

---

## Console-Agnostic Summary

All features are designed to be console-agnostic:

| Feature | Core Components | Console Components |
|---------|-----------------|-------------------|
| Debug Inspection | Registry, FFI, Panel | 2-3 lines wiring |
| State Snapshots | Manager, FFI | None |
| Hot Reload | File watcher, swap logic | None |
| Visual Overlays | Buffer, FFI | Overlay renderer |
| Replay System | Recording, playback | None |
| Network Simulator | Simulation layer | None |
| Memory Viewer | Access layer, UI | None |

Most work lives in `emberware-core`. Consoles need minimal integration code.

---

## Notes

- Effort estimates include testing and documentation
- Estimates assume single developer
- Some features have synergies (e.g., Debug Panel + Memory Viewer integration)
- Priorities may shift based on user feedback
- All specs include pending questions that should be resolved before implementation
