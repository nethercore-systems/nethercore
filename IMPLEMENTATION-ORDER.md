# Implementation Order Analysis: docs/ready Specs

## Summary

11 specs identified in `docs/ready/`. Analysis reveals clear dependency chains and optimal parallelization opportunities.

---

## Dependency Graph

```
State Snapshots ─┬─> Replay System
                 ├─> Hot Reload
                 └─> Memory Viewer ←── Debug Inspection

Ember-Z Export ────> Skeletal Animation

Independent: Point Lighting, Visual Debug Overlays, Network Simulator, Asset Pack System
```

---

## Recommended Implementation Order

### Phase 1: Foundation (Parallel - No Dependencies)
| Feature | Est. Days | Unlocks |
|---------|-----------|---------|
| **State Snapshots** | 8 | Hot Reload, Replay System, Memory Viewer |
| **Debug Inspection** | 6 | Memory Viewer |
| **Ember-Z Export Tool** | 8 | Skeletal Animation |

**Rationale:** These three have no dependencies on other ready specs and collectively unlock 4 additional features. Can be developed in parallel.

---

### Phase 2: Developer Productivity (Depends on Phase 1)
| Feature | Est. Days | Depends On |
|---------|-----------|------------|
| **Hot Reload** | 4 | State Snapshots |
| **Replay System** | 10 | State Snapshots |

**Rationale:** Both depend on State Snapshots for state preservation/checkpointing. Hot Reload is lower effort and provides immediate iteration speed. Replay System is critical for QA and bug reproduction.

---

### Phase 3: Advanced Debugging (Depends on Phase 1)
| Feature | Est. Days | Depends On |
|---------|-----------|------------|
| **Memory Viewer** | 8 | State Snapshots + Debug Inspection |

**Rationale:** Requires both State Snapshots (for memory comparison) and Debug Inspection (for struct type registry). Worth waiting for both.

---

### Phase 4: Graphics Features (Can Start in Phase 1)
| Feature | Est. Days | Depends On |
|---------|-----------|------------|
| **Visual Debug Overlays** | 9 | None (standalone) |
| **Point Lighting** | 12 | None (standalone) |
| **Skeletal Animation** | 14 | Ember-Z Export Tool |

**Rationale:** Overlays and Lighting are independent - can start immediately. Skeletal Animation should wait for Ember-Z Export to provide artist workflow.

---

### Phase 5: Infrastructure (Independent - Lowest Priority)
| Feature | Est. Days | Depends On |
|---------|-----------|------------|
| **Asset Pack System** | 10 | None |
| **Network Simulator** | 6 | None |

**Rationale:** Both are standalone. Asset Pack is for larger games (not urgent for early development). Network Simulator only matters once netplay is being actively tested.

---

## Optimal Parallel Schedule

```
Week 1-2:  State Snapshots | Debug Inspection | Ember-Z Export | Visual Debug Overlays
              ↓                    ↓                 ↓
Week 3:    Hot Reload        (done)           (continues)        Point Lighting
              ↓                    ↓                 ↓                  ↓
Week 4:    Replay System     Memory Viewer    Skeletal Animation    (continues)
              ↓                    ↓                 ↓                  ↓
Week 5+:   (continues)       (done)           (continues)         (done)
           Network Simulator | Asset Pack System (as needed)
```

---

## Critical Path

**Shortest path to full debugging suite:**
1. State Snapshots (8d) → Hot Reload (4d) = 12 days to fast iteration
2. Debug Inspection (6d) + State Snapshots (8d) → Memory Viewer (8d) = 14 days (parallel)

**Shortest path to graphics features:**
1. Ember-Z Export (8d) → Skeletal Animation (14d) = 22 days
2. Point Lighting (12d) - can start immediately
3. Visual Debug Overlays (9d) - can start immediately

---

## Blockers Summary

| Feature | Blocked By | Blocking |
|---------|------------|----------|
| State Snapshots | Nothing | Hot Reload, Replay, Memory Viewer |
| Debug Inspection | Nothing | Memory Viewer |
| Ember-Z Export | Nothing | Skeletal Animation |
| Hot Reload | State Snapshots | Nothing |
| Replay System | State Snapshots | Nothing |
| Memory Viewer | State Snapshots, Debug Inspection | Nothing |
| Skeletal Animation | Ember-Z Export | Nothing |
| Point Lighting | Nothing | Nothing |
| Visual Debug Overlays | Nothing | Nothing |
| Asset Pack System | Nothing | Nothing |
| Network Simulator | Nothing | Nothing |

---

## Recommendations

1. **Start with State Snapshots** - It's the highest-value foundation piece, unlocking 3 other features

2. **Parallel track graphics** - Visual Debug Overlays and Point Lighting have zero dependencies; start them alongside foundation work

3. **Hot Reload is quick win** - Only 4 days after State Snapshots, massive productivity boost

4. **Defer Asset Pack & Network Simulator** - These solve problems you don't have yet (large assets, netplay testing)

5. **Ember-Z Export before Skeletal Animation** - Without the export tool, skeletal animation requires manual asset authoring

---

## Total Estimated Effort

- **Sequential (worst case):** ~95 days
- **Fully parallelized (3 developers):** ~35-40 days
- **Single developer, smart ordering:** ~60-65 days

---

## Spec Locations

All specs are in `docs/ready/`:
- `asset-pack-system-spec.md` ← Currently in progress
- `debug-inspection-spec.md`
- `ember-z-export-spec.md`
- `hot-reload-spec.md`
- `memory-viewer-spec.md`
- `network-simulator-spec.md`
- `point-lighting-spec.md`
- `replay-system-spec.md`
- `skeletal-animation-spec.md`
- `state-snapshots-spec.md`
- `visual-debug-overlays-spec.md`
