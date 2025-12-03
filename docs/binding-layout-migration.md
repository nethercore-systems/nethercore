# Binding Layout Migration

**Status:** Planning complete, ready for implementation after matrix packing

---

## Current State (INCORRECT - Multiple Layouts)

### Mode 0/1 (Unlit, Matcap)
| Binding | Type | Contents | Problem |
|---------|------|----------|---------|
| 0 | Storage | `model_matrices` | ✅ OK |
| 1 | Storage | `view_matrices` | ✅ OK |
| 2 | Storage | `proj_matrices` | ✅ OK |
| 3 | Storage | `mvp_indices` | ✅ OK |
| 4 | Uniform | `sky` | ❌ Frame-wide, should be per-draw |
| 5 | Uniform | `material` | ❌ Frame-wide, should be per-draw |
| 6 | Storage | `bones` | ⚠️ Inconsistent with Mode 2/3 |

### Mode 2/3 (PBR, Hybrid)
| Binding | Type | Contents | Problem |
|---------|------|----------|---------|
| 0 | Storage | `model_matrices` | ✅ OK |
| 1 | Storage | `view_matrices` | ✅ OK |
| 2 | Storage | `proj_matrices` | ✅ OK |
| 3 | Storage | `mvp_indices` | ✅ OK |
| 4 | Uniform | `sky` | ❌ Frame-wide, should be per-draw |
| 5 | Uniform | `material` | ❌ Frame-wide, should be per-draw |
| 6 | Uniform | `lights` | ❌ Frame-wide, should be per-draw |
| 7 | Uniform | `camera` | ❌ Redundant (in view matrix) |
| 8 | Storage | `bones` | ⚠️ Inconsistent with Mode 0/1 |

**Issues:**
- 🔴 **Critical:** Material properties are frame-wide instead of per-draw (BUG)
- 🟡 **Maintenance:** Bones at different binding indices (6 vs 8) causes off-by-N errors
- 🟡 **Complexity:** Different binding layouts between modes
- 🟡 **Redundancy:** Sky, material, lights, camera all duplicated

---

## Target State (CORRECT - Unified Layout)

### All Modes (0-3)
| Binding | Type | Contents | Notes |
|---------|------|----------|-------|
| 0 | Storage | `model_matrices: array<mat4x4<f32>>` | Per-frame pool |
| 1 | Storage | `view_matrices: array<mat4x4<f32>>` | Per-frame pool |
| 2 | Storage | `proj_matrices: array<mat4x4<f32>>` | Per-frame pool |
| 3 | Storage | `shading_states: array<UnifiedShadingState>` | Per-frame pool, per-draw indexed |
| 4 | Storage | `mvp_shading_indices: array<vec2<u32>>` | Per-draw indices |
| 5 | Storage | `bones: array<mat4x4<f32>>` | Per-frame pool (optional) |

**Logical grouping:**
- **Bindings 0-3:** Data buffers (matrices, shading states)
- **Bindings 4-5:** Indices/structural (per-draw indices, bones)

**Key Properties:**
- ✅ All modes use **identical** binding layout
- ✅ Bones always at binding 5 (consistent)
- ✅ Shading state is per-draw (fixes the bug)
- ✅ No redundant uniforms

---

## What's in UnifiedShadingState?

The `shading_states` buffer at binding 4 contains **everything that was scattered across multiple uniforms:**

```wgsl
struct UnifiedShadingState {
    // Material properties (was binding 5)
    params_packed: u32,         // metallic, roughness, emissive, pad
    color_rgba8: u32,
    blend_modes: u32,
    _pad: u32,

    // Sky data (was binding 4)
    sky_horizon: u32,
    sky_zenith: u32,
    sky_sun_dir: vec2<i32>,
    sky_sun_color: u32,
    _pad_sky: u32,

    // Light data (was binding 6 in Mode 2/3)
    light0_dir: vec2<i32>,
    light0_color: u32,
    _pad_l0: u32,
    // ... light1, light2, light3 (64 bytes total)
}
```

**Camera position** is derivable from the view matrix, so no separate binding needed.

**Total size:** ~96 bytes per unique shading state (POD, hashable, deduplicated)

---

## Migration Strategy

### Before Implementation (MUST COMPLETE FIRST)
1. ✅ Matrix packing implementation (allocates `vec2<u32>` in binding 3)

### Implementation Order
1. **Phase 1:** Define `PackedUnifiedShadingState` structure in Rust
2. **Phase 2:** Add shading state pool to `ZFFIState` (mirrors matrix pools)
3. **Phase 3:** Update `VRPCommand` to store `shading_state_index` instead of separate fields
4. **Phase 4:** Update FFI layer to pack shading state per-draw
5. **Phase 5:** Update render pass execution to upload shading states
6. **Phase 6:** Update all 4 shader templates to use unified binding layout
7. **Phase 7:** Update bind group layout in `pipeline.rs` (remove mode switch)
8. **Phase 8:** Update pipeline extraction to read blend mode from shading state
9. **Phase 9:** Testing and validation

### Breaking Changes
- ✅ Shader binding layout changes (acceptable pre-release)
- ✅ All pipelines regenerated
- ✅ VRPCommand structure changes

---

## Benefits Summary

| Aspect | Before | After | Improvement |
|--------|--------|-------|-------------|
| Binding layout | 2 layouts (7 vs 9 bindings) | 1 layout (6 bindings) | -33% bindings, 100% consistency |
| Material control | Frame-wide (BUG) | Per-draw (CORRECT) | Bug fix |
| Bones binding | 6 (Mode 0/1), 8 (Mode 2/3) | 5 (all modes) | Consistent |
| Maintenance | High (off-by-N errors) | Low (one layout) | Much easier |
| Code complexity | Mode switch everywhere | Unified code path | Simpler |

---

**Last Updated:** December 2024
**Related:** [implementation-plan-unified-shading-state.md](./implementation-plan-unified-shading-state.md)
