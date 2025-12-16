# Console Runtime Limits Specification

**Status:** Pending (Needs Clarification)
**Author:** Zerve
**Version:** 1.0
**Last Updated:** December 2024

---

## Summary

Define and enforce per-frame runtime limits for fantasy console authenticity. Currently, VRAM tracking (8MB), vertex format validation, and memory bounds are enforced. This spec adds enforcement for draw calls, vertex counts, mesh counts, and CPU budget.

---

## Motivation

- **Authenticity:** PS1/N64 had strict hardware limits that shaped their aesthetic
- **Performance:** Prevents runaway resource usage from degrading experience
- **Developer Guidance:** Helps developers understand platform constraints
- **Consistency:** Ensures games perform reliably across different hardware

---

## Current State

**Enforced:**
- VRAM tracking (8MB limit)
- Vertex format validation
- Memory bounds checking

**Not Enforced:**
- Draw calls per frame
- Vertices per frame (immediate mode)
- Retained mesh count/size
- CPU budget per tick
- WASM heap size

---

## Proposed Limits (Emberware Z)

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max draw calls/frame | 512 | PS1/N64 ~500-1000 triangles/sec @ 30fps |
| Max immediate vertices/frame | 100,000 | Reasonable for fantasy console aesthetic |
| Max retained meshes | 256 | Encourages efficient resource management |
| Max vertices per mesh | 65,536 | u16 index limit, PS1-era constraint |
| Max CPU time per tick | 4ms | Console spec (60fps = 16.67ms budget) |
| Max WASM heap | 16MB | Console spec (not currently enforced) |
| Max textures | 256 | VRAM subdivision constraint |
| Max sounds | 64 | Audio channel management |

---

## Open Questions

### 1. Enforcement Mode
How should limit violations be handled?

**Options:**
- **A) Hard Error:** Reject draw call, return error code
- **B) Warning + Continue:** Log warning, allow operation
- **C) Warning + Clamp:** Log warning, clamp to limit
- **D) Debug-Only:** Only enforce in debug builds

**Recommendation:** Option B (Warning + Continue) for development, with configurable hard errors for strict compliance testing.

### 2. Per-Console Limits
Should each console define its own limits?

**Options:**
- **A) Yes:** Each console has unique constraints (Z vs Classic vs future)
- **B) No:** Single set of limits for simplicity

**Recommendation:** Option A - limits should be part of `ConsoleSpecs` since different consoles represent different "hardware" generations.

### 3. Limit Categories
Should some limits be separate (e.g., 2D UI vs 3D world)?

**Options:**
- **A) Single Pool:** All draws count toward same limit
- **B) Separate Pools:** UI overlay has separate budget
- **C) Weighted:** 2D draws count as fraction of 3D

**Recommendation:** Option A for simplicity. Fantasy consoles didn't distinguish UI from world rendering.

### 4. Debug Mode Overrides
Should limits be adjustable during development?

**Options:**
- **A) Fixed:** Same limits always
- **B) Disable in Debug:** No limits in debug builds
- **C) Configurable:** Dev can set custom limits via debug panel

**Recommendation:** Option C - allow developers to test with stricter/looser limits during development.

### 5. Violation Reporting
How to surface limit violations to developers?

**Options:**
- **A) Console Log:** Print to stdout
- **B) Debug Overlay:** Show in F3 panel
- **C) Return Codes:** FFI functions return error codes
- **D) All of the Above**

**Recommendation:** Option D - comprehensive feedback through all channels.

---

## Proposed Implementation

### Phase 1: Limit Definitions

Add limits to `ConsoleSpecs`:

```rust
// core/src/console.rs
pub struct ConsoleSpecs {
    // Existing fields...

    // Runtime limits
    pub max_draw_calls_per_frame: u32,
    pub max_immediate_vertices_per_frame: u32,
    pub max_retained_meshes: u32,
    pub max_vertices_per_mesh: u32,
    pub max_tick_time_micros: u32,
    pub max_wasm_heap_bytes: usize,
    pub max_textures: u32,
    pub max_sounds: u32,
}
```

### Phase 2: Runtime Tracking

Add per-frame counters:

```rust
// emberware-z/src/state/ffi_state.rs
pub struct FrameLimits {
    pub draw_calls: u32,
    pub immediate_vertices: u32,
    pub violations: Vec<LimitViolation>,
}

pub enum LimitViolation {
    DrawCallsExceeded { current: u32, max: u32 },
    ImmediateVerticesExceeded { current: u32, max: u32 },
    MeshVerticesExceeded { mesh_id: u32, vertices: u32, max: u32 },
    // etc.
}
```

### Phase 3: FFI Validation

Check limits in draw functions:

```rust
// emberware-z/src/ffi/draw.rs
pub fn draw_triangles(/* ... */) -> i32 {
    let state = get_state();
    let vertex_count = /* ... */;

    state.frame_limits.immediate_vertices += vertex_count;

    if state.frame_limits.immediate_vertices > specs.max_immediate_vertices_per_frame {
        state.frame_limits.violations.push(
            LimitViolation::ImmediateVerticesExceeded { /* ... */ }
        );
        // Depending on enforcement mode: return error, warn, or continue
    }

    state.frame_limits.draw_calls += 1;
    // Similar check for draw calls...

    // Actual draw logic
}
```

### Phase 4: Debug Integration

Expose limits in debug overlay:

```rust
// emberware-z/src/debug.rs (or via existing debug system)
fn debug_stats(&self, state: &ZState) -> Vec<DebugStat> {
    vec![
        DebugStat::new("Draw Calls", format!(
            "{}/{}",
            state.frame_limits.draw_calls,
            self.specs.max_draw_calls_per_frame
        )),
        DebugStat::new("Immediate Verts", format!(
            "{}/{}",
            state.frame_limits.immediate_vertices,
            self.specs.max_immediate_vertices_per_frame
        )),
        // etc.
    ]
}
```

### Phase 5: Documentation

Update console documentation with limits:

```markdown
## Hardware Limits

Emberware Z enforces the following per-frame limits:

| Resource | Limit |
|----------|-------|
| Draw calls | 512 |
| Immediate vertices | 100,000 |
| Retained meshes | 256 |
| ...
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `core/src/console.rs` | Add limit fields to `ConsoleSpecs` |
| `core/src/app/types.rs` | Add `FrameLimits`, `LimitViolation` types |
| `emberware-z/src/lib.rs` | Define Z-specific limit values |
| `emberware-z/src/graphics/mod.rs` | Track draw calls, vertices per frame |
| `emberware-z/src/ffi/*.rs` | Validate limits in draw functions |
| `emberware-z/src/debug.rs` | Display limits in debug overlay |
| `docs/reference/emberware-z.md` | Document console limits |
| `docs/book/src/guides/limits.md` | Add developer guide for limits |

---

## Testing Strategy

1. **Unit Tests:** Verify limit tracking increments correctly
2. **Integration Tests:** Verify violations are detected and reported
3. **Example Game:** Create `limits-demo` showing limit feedback
4. **Performance Tests:** Ensure tracking overhead is negligible

---

## Migration / Backwards Compatibility

- Existing games continue to work (warnings only by default)
- Games can opt into strict mode via manifest flag
- Debug panel shows limits regardless of enforcement mode
