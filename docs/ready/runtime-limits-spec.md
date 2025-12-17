# Console Runtime Limits Specification

**Status:** Ready for Implementation
**Author:** Zerve
**Version:** 1.1
**Last Updated:** December 2024

---

## Summary

Define and enforce per-frame runtime limits for fantasy console authenticity. Currently, VRAM tracking (4MB), vertex format validation, and memory bounds are enforced. This spec adds enforcement for draw calls, vertex counts, mesh counts, and CPU budget (via WASM fuel metering).

---

## Motivation

- **Authenticity:** PS1/N64 had strict hardware limits that shaped their aesthetic
- **Performance:** Prevents runaway resource usage from degrading experience
- **Developer Guidance:** Helps developers understand platform constraints
- **Consistency:** Ensures games perform reliably across different hardware

---

## Current State

**Enforced:**
- VRAM tracking (4MB limit for textures)
- Vertex format validation
- Memory bounds checking

**Not Yet Enforced:**
- Draw calls per frame
- Vertices per frame (immediate mode)
- Retained mesh count/size
- CPU budget per tick (via WASM fuel)
- Texture/mesh/sound count limits

---

## Proposed Limits (Emberware Z)

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max draw calls/frame | 512 | PS1/N64 ~500-1000 triangles/sec @ 30fps |
| Max immediate vertices/frame | 100,000 | Reasonable for fantasy console aesthetic |
| Max retained meshes | 256 | Encourages efficient resource management |
| Max vertices per mesh | 65,536 | u16 index limit, PS1-era constraint |
| Max WASM fuel/tick | 10,000,000 | ~10M instructions per update() call |
| Max WASM heap | 4MB | Matches `ram_limit` in ConsoleSpecs |
| Max textures | 256 | VRAM subdivision constraint |
| Max sounds | 64 | Audio channel management |
| VRAM limit | 4MB | Already enforced for textures |

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

## Resolved Decisions

All open questions have been resolved:

| Question | Decision | Rationale |
|----------|----------|-----------|
| **1. Enforcement Mode** | **B) Warning + Continue** | Non-breaking for developers; logs warning but allows operation to proceed. Games can opt into strict mode later. |
| **2. Per-Console Limits** | **A) Yes** | Limits are part of `ConsoleSpecs`. Different consoles represent different "hardware" generations with unique constraints. |
| **3. Limit Categories** | **A) Single Pool** | All immediate mode data (matrices, procedural meshes, instance data) shares one budget. Authentic to real hardware. |
| **4. Debug Mode Overrides** | **C) Configurable** | Developers can adjust limits via debug panel during development to test edge cases. |
| **5. Violation Reporting** | **D) All of the Above** | Console log + debug overlay + return codes. Comprehensive feedback through all channels. |

### CPU Budget Enforcement

The original spec mentioned "4ms CPU budget" without specifying how to enforce it. Decision:

- **Use WASM Fuel metering** via wasmtime's `consume_fuel()` config
- **Fixed instruction limit**: 10,000,000 fuel units per `update()` call
- **Why fuel over wall clock?** Fuel is deterministic across hardware, essential for rollback netcode
- **Overhead**: ~10-20% execution overhead (acceptable for fantasy console)

---

## Proposed Implementation

### Phase 1: Limit Definitions

Add limits to `ConsoleSpecs`:

```rust
// shared/src/console.rs
pub struct ConsoleSpecs {
    // Existing fields...

    // Runtime limits
    pub max_draw_calls_per_frame: u32,
    pub max_immediate_vertices_per_frame: u32,
    pub max_retained_meshes: u32,
    pub max_vertices_per_mesh: u32,
    pub max_fuel_per_tick: u64,
    pub max_textures: u32,
    pub max_sounds: u32,
}
```

### Phase 2: Runtime Tracking

Add per-frame counters to `ZFFIState`:

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
    MeshCountExceeded { current: u32, max: u32 },
    MeshVerticesExceeded { mesh_id: u32, vertices: u32, max: u32 },
    TextureCountExceeded { current: u32, max: u32 },
    SoundCountExceeded { current: u32, max: u32 },
    FuelExhausted { used: u64, max: u64 },
}
```

Reset `FrameLimits` at start of each frame in `clear_frame()`.

### Phase 3: FFI Validation

Check limits in draw functions:

```rust
// emberware-z/src/ffi/draw_3d.rs (and draw_2d.rs)
pub fn draw_triangles(/* ... */) {
    let state = get_state();
    let specs = get_specs();

    // Track vertices
    state.frame_limits.immediate_vertices += vertex_count;
    if state.frame_limits.immediate_vertices > specs.max_immediate_vertices_per_frame {
        state.frame_limits.violations.push(
            LimitViolation::ImmediateVerticesExceeded { /* ... */ }
        );
        warn!("Immediate vertex limit exceeded");
        // Warning + Continue: log but proceed
    }

    // Track draw call
    state.frame_limits.draw_calls += 1;
    if state.frame_limits.draw_calls > specs.max_draw_calls_per_frame {
        state.frame_limits.violations.push(
            LimitViolation::DrawCallsExceeded { /* ... */ }
        );
        warn!("Draw call limit exceeded");
    }

    // Actual draw logic (unchanged)
    state.render_pass.record_triangles(...);
}
```

### Phase 4: Debug Integration

Expose limits in debug overlay via the Console trait's `debug_stats` method:

```rust
// emberware-z/src/lib.rs (EmberwareZ impl)
fn debug_stats(&self, state: &ZFFIState) -> Vec<DebugStat> {
    vec![
        // Existing stats...
        DebugStat::new("Draw Calls", format!(
            "{}/{}",
            state.frame_limits.draw_calls,
            self.specs.max_draw_calls_per_frame
        )),
        DebugStat::new("Imm. Verts", format!(
            "{}/{}",
            state.frame_limits.immediate_vertices,
            self.specs.max_immediate_vertices_per_frame
        )),
        // Consider color-coding: yellow at 80%, red at 100%+
    ]
}
```

### Phase 5: WASM Fuel Metering

Enable fuel consumption in wasmtime:

```rust
// core/src/wasm/mod.rs
impl WasmEngine {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }
}

// Before each update() call:
store.set_fuel(specs.max_fuel_per_tick)?;

// After update(), check if fuel exhausted (trap indicates exhaustion)
```

### Phase 6: Documentation

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
| `shared/src/console.rs` | Add limit fields to `ConsoleSpecs` |
| `emberware-z/src/lib.rs` | Define Z-specific limit values, update `debug_stats()` |
| `emberware-z/src/state/ffi_state.rs` | Add `FrameLimits` struct, `LimitViolation` enum |
| `emberware-z/src/ffi/draw_3d.rs` | Track draw calls + vertices in each draw fn |
| `emberware-z/src/ffi/draw_2d.rs` | Track draw calls + vertices for 2D draws |
| `emberware-z/src/ffi/mesh.rs` | Check mesh count + vertices per mesh limits |
| `emberware-z/src/ffi/texture.rs` | Check texture count limit |
| `emberware-z/src/ffi/rom.rs` | Check limits for ROM asset loading |
| `emberware-z/src/ffi/audio.rs` | Check sound count limit |
| `core/src/wasm/mod.rs` | Enable fuel metering in `WasmEngine` config |
| `docs/reference/emberware-z.md` | Document console limits |

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
