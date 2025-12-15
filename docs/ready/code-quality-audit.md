# Post-Refactor Code Quality Audit Plan

## Executive Summary

Following the major refactor, this audit identified **no critical issues**. The architecture is clean, and the codebase demonstrates excellent quality. However, there are opportunities to reduce code duplication and clean up minor issues.

**Overall Rating: A (Excellent)**

---

## Findings Summary

| Category | Count | Severity |
|----------|-------|----------|
| TODOs | 8 | Low-Medium |
| DRY Violations | 10 patterns | Medium |
| Clippy Warnings | 9 | Low |
| Dead Code | 31 annotations | Low (mostly justified) |
| Architecture Violations | 0 | N/A |
| Code Smells | 0 | N/A |

---

## 1. TODOs Left in Code (8 items)

### Medium Priority

1. **Frame Controller State Integration** (2 locations)
   - Files: `core/src/debug/ffi.rs:860`, `core/src/debug/ffi.rs:875`
   - Issue: `debug_get_frame_count()` and `debug_get_time_scale()` return hardcoded values
   - Impact: Debug panel frame control features don't work

2. **Data Pack Support in ember CLI**
   - File: `xtask/src/cart/create_z.rs:161`
   - Issue: `data_pack: None` placeholder
   - Impact: Cannot package asset data packs via CLI

3. **Font Loading in Pack System**
   - File: `tools/ember-cli/src/pack.rs:279`
   - Issue: Font loading not implemented
   - Impact: Games cannot load custom fonts from data packs

### Low Priority

4-7. **Example Game Incomplete Transformations** (4 locations)
   - `examples/lighting/src/lib.rs:391`
   - `examples/platformer/src/lib.rs:593, 616, 635`
   - Issue: Missing transform_set() implementations
   - Impact: Low - educational examples

---

## 2. DRY Violations (10 patterns)

### High Priority

#### 2.1 Memory Reading Pattern Duplication
**Files (8+):**
- `emberware-z/src/ffi/transform.rs:43-70`
- `emberware-z/src/ffi/texture.rs:50-86`
- `emberware-z/src/ffi/mesh.rs:84-113`
- `emberware-z/src/ffi/draw_3d.rs:71-99`
- `emberware-z/src/ffi/assets.rs:77-88`
- `emberware-z/src/ffi/skinning.rs:82-104`
- `core/src/ffi.rs:116-126`

**Pattern:**
```rust
let memory = match caller.data().game.memory {
    Some(m) => m,
    None => { warn!("..."); return 0; }
};
let mem_data = memory.data(&caller);
if ptr + byte_size > mem_data.len() { ... }
```

**Fix:** Extract to `emberware-z/src/ffi/mod.rs`:
```rust
pub fn read_memory_slice<'a>(
    caller: &'a Caller<'_, ZGameContext>,
    ptr: u32,
    len: usize,
    context: &str,
) -> Option<&'a [u8]>
```

#### 2.2 Parameter Validation Duplication
**Files (10+):**
- `emberware-z/src/ffi/billboard.rs:35-38`
- `emberware-z/src/ffi/render_state.rs:54-60, 73-80, 92-100`
- `emberware-z/src/ffi/material.rs:84-89`
- `emberware-z/src/ffi/mesh.rs:57-65`
- `emberware-z/src/ffi/draw_3d.rs:38-45`
- `emberware-z/src/ffi/lighting.rs:50-56`
- `emberware-z/src/ffi/sky.rs:68-73`

**Fix:** Create validation helpers in `ffi/mod.rs`:
```rust
pub fn validate_vertex_format(format: u32) -> Result<u8, ()>
pub fn validate_mode_range(mode: u32, max: u32, name: &str) -> Result<u8, ()>
pub fn validate_direction_vector(x: f32, y: f32, z: f32) -> Result<(), ()>
```

### Medium Priority

#### 2.3 Config Function Guard Pattern
**Files (7+):**
- `emberware-z/src/ffi/config.rs:32-47, 72-87`
- `emberware-z/src/ffi/texture.rs:38-42`
- `emberware-z/src/ffi/mesh.rs:51-55`
- `emberware-z/src/ffi/assets.rs:50-54`
- `emberware-z/src/ffi/skinning.rs:48-52`

**Fix:** Extend `guards` module with combined check:
```rust
pub fn check_init_only_once(caller: &Caller<...>, field: &mut bool, name: &str) -> Result<()>
```

#### 2.4 Matrix Conversion Duplication
**Files:**
- `emberware-z/src/ffi/transform.rs:68-82`
- `emberware-z/src/ffi/skinning.rs:106-138`

**Fix:** Extract to `ffi/mod.rs`:
```rust
pub fn read_matrix_3x4(bytes: &[u8]) -> Option<BoneMatrix3x4>
pub fn read_matrix_4x4(bytes: &[u8]) -> Option<Mat4>
```

#### 2.5 Procedural Mesh Boilerplate
**File:** `emberware-z/src/ffi/mesh_generators.rs` (6 functions)

**Pattern:** Each function has identical handle allocation and pending mesh queuing.

**Fix:** Extract helper:
```rust
fn queue_procedural_mesh(state: &mut ZFFIState, mesh: MeshData, format: u8) -> u32
```

### Low Priority

#### 2.6 Inconsistent `get_wasm_memory()` Usage
Some files use the helper, others duplicate the pattern. Standardize usage.

#### 2.7 Debug FFI Registration Repetition
**File:** `core/src/debug/ffi.rs:35-116`

**Fix:** Use a macro to reduce registration boilerplate.

---

## 3. Clippy Warnings (9 items)

All are `collapsible_if` warnings - nested if statements that can be combined using `if let ... && let ...` syntax.

**Files:**
- `core/src/analysis/mod.rs:228, 261`
- `core/src/debug/panel.rs:219, 220, 221`
- `core/src/ffi.rs:78`
- `core/src/library/game.rs:86, 87, 88`
- `core/src/library/rom.rs:169`

**Fix:** Collapse nested if statements:
```rust
// Before
if let Some(res) = call.arg {
    if res > 3 { ... }
}

// After
if let Some(res) = call.arg && res > 3 {
    ...
}
```

---

## 4. Dead Code Annotations (31 items)

### Justified (Keep)
- `core/src/runtime.rs:44` - `console` field for lifetime management
- `emberware-z/src/console.rs:49, 67, 105` - Button enum/impl used by tests
- `emberware-z/src/graphics/init.rs:34, 37` - Texture fields needed for view lifetimes
- `emberware-z/src/graphics/pipeline.rs:559` - Debug helper
- `emberware-z/src/shader_gen.rs:103, 126, 136` - Debug/inspection helpers
- `library/src/registry.rs:88, 416, 422` - Future-compatible API

### Review Needed
- `core/src/wasm/mod.rs:110` - Verify if still needed
- `emberware-z/src/graphics/unified_shading_state.rs:201, 208, 260` - Check usage
- `emberware-z/src/graphics/texture_manager.rs:19, 391` - Check usage
- `emberware-z/src/ffi/keyframes.rs:536` - Check usage
- `emberware-z/src/graphics/mod.rs:418` - Check usage
- `emberware-z/src/graphics/draw.rs:156, 167, 184, 195` - Conversion helpers
- `tools/ember-export/src/texture.rs:153` - Check usage
- `tools/ember-export/src/manifest.rs:110, 127` - Check usage

---

## 5. Architecture Analysis

**Status: EXCELLENT - No violations found**

- `core/` is fully console-agnostic
- `emberware-z/` correctly depends only on `core/` and `shared/`
- `library/` has no console-specific code
- `shared/` has no dependencies on other workspace crates
- Clear separation via Console trait abstraction

---

## Implementation Plan

### Phase 1: Quick Wins (30 min)
1. Fix 9 Clippy collapsible_if warnings
2. Run `cargo clippy --fix` to auto-fix

### Phase 2: FFI Helpers (2-3 hours)
1. Create `read_memory_slice()` helper in `emberware-z/src/ffi/mod.rs`
2. Create validation helpers (`validate_vertex_format`, `validate_mode_range`, etc.)
3. Refactor 8+ FFI files to use new helpers

### Phase 3: Guard Consolidation (1 hour)
1. Extend `guards.rs` with `check_init_only_once()`
2. Refactor config functions to use consolidated guard

### Phase 4: Mesh Generator Cleanup (30 min)
1. Create `queue_procedural_mesh()` helper
2. Refactor 6 mesh generator functions

### Phase 5: Dead Code Review (1 hour)
1. Review 14 "review needed" annotations
2. Remove unused code or add justification comments

### Phase 6: TODO Resolution (2+ hours)
1. Implement frame controller state reading
2. Add data pack support to ember CLI (if needed now)
3. Implement font loading (if needed now)

---

## Files to Modify

### Core Changes
- `core/src/analysis/mod.rs` - Clippy fix
- `core/src/debug/panel.rs` - Clippy fix
- `core/src/ffi.rs` - Clippy fix
- `core/src/library/game.rs` - Clippy fix
- `core/src/library/rom.rs` - Clippy fix
- `core/src/debug/ffi.rs` - TODO: frame controller state

### FFI Refactoring
- `emberware-z/src/ffi/mod.rs` - Add helpers
- `emberware-z/src/ffi/guards.rs` - Extend guards
- `emberware-z/src/ffi/transform.rs` - Use helpers
- `emberware-z/src/ffi/texture.rs` - Use helpers
- `emberware-z/src/ffi/mesh.rs` - Use helpers
- `emberware-z/src/ffi/draw_3d.rs` - Use helpers
- `emberware-z/src/ffi/assets.rs` - Use helpers
- `emberware-z/src/ffi/skinning.rs` - Use helpers
- `emberware-z/src/ffi/mesh_generators.rs` - Consolidate boilerplate
- `emberware-z/src/ffi/config.rs` - Use guards
- `emberware-z/src/ffi/billboard.rs` - Use validation
- `emberware-z/src/ffi/render_state.rs` - Use validation
- `emberware-z/src/ffi/material.rs` - Use validation
- `emberware-z/src/ffi/lighting.rs` - Use validation
- `emberware-z/src/ffi/sky.rs` - Use validation

### Tools
- `tools/ember-cli/src/pack.rs` - TODO: font loading
- `xtask/src/cart/create_z.rs` - TODO: data pack support
