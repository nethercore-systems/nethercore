# 2D Layer Ordering Fix - Implementation Plan

## Problem Summary

**Current Issue:** All quad rendering (text, sprites, 2D UI) is completely broken - nothing renders.

**Root Cause:** Two compounding issues:
1. **Incomplete stencil implementation** - Mode 0 (disabled) uses `wgpu::StencilState::default()` which has undefined behavior with `Depth24PlusStencil8` format
2. **Depth testing hack for layers** - Screen-space quads at layer 0 map to depth 1.0, which equals the depth buffer clear value, causing `CompareFunction::Less` to fail (1.0 < 1.0 = false)

## Temporary Fixes Applied

### Fix 1: Explicit Stencil State for Mode 0
**File:** `nethercore-zx/src/graphics/pipeline.rs:120-134`

Changed from `wgpu::StencilState::default()` to explicit configuration:
```rust
let face = wgpu::StencilFaceState {
    compare: wgpu::CompareFunction::Always,  // Always pass
    fail_op: wgpu::StencilOperation::Keep,
    depth_fail_op: wgpu::StencilOperation::Keep,
    pass_op: wgpu::StencilOperation::Keep,
};
wgpu::StencilState {
    front: face,
    back: face,
    read_mask: 0xFF,
    write_mask: 0x00,  // Never write to stencil buffer
}
```

### Fix 2: LessOrEqual for Quad Depth Testing
**File:** `nethercore-zx/src/graphics/pipeline.rs:317`

Changed quad pipeline depth compare from `Less` to `LessOrEqual` to allow layer 0 (depth=1.0) to pass when depth buffer = 1.0.

## Why Current Approach is Wrong

**The Depth Hack:**
- Uses depth buffer for 2D layer ordering (depth = layer hack)
- Wastes GPU depth testing on 2D elements that don't need 3D occlusion
- Creates edge cases (layer 0 = clear value = broken)
- Not how professional engines work

**The Sorting Problem:**
Current command sort order (`frame.rs:185-224`):
```
viewport → stencil → render_type → depth/cull → textures
```

**Missing:** Layer is NOT in the sort key! Commands are batched by texture, breaking layer ordering. The depth hack tries to fix this at GPU level, but it should be fixed at CPU sorting level.

## How Professional Engines Do It

**Unity, Unreal, Godot, Bevy:** All use the same pattern:

1. ✅ **Sort by layer FIRST** (highest priority in sort key)
2. ✅ **Batch by texture WITHIN same layer** (performance optimization)
3. ✅ **No depth testing for 2D** (depth buffer used only for 3D)

**Example Sort Key:**
```rust
(viewport, layer, stencil_mode, texture_id)
```

**Result:**
- Different layers → strictly ordered (no batching across layers)
- Same layer → batched together by texture (performance)
- No depth buffer waste

## Proper Long-Term Solution

### Phase 1: Add Layer to Sort Key

#### 1.1 Update VRPCommand::Quad
**File:** `nethercore-zx/src/graphics/command_buffer.rs`

Add `layer` field to Quad variant:
```rust
pub enum VRPCommand {
    Quad {
        base_vertex: u32,
        first_index: u32,
        base_instance: u32,
        instance_count: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
        viewport: Viewport,
        stencil_mode: u8,
        layer: u32,  // ADD THIS
    },
    // ... other variants
}
```

#### 1.2 Update CommandSortKey
**File:** `nethercore-zx/src/graphics/command_buffer.rs`

Update quad sort key to include layer:
```rust
impl CommandSortKey {
    pub fn quad(
        viewport: Viewport,
        layer: u32,  // ADD THIS
        stencil_mode: u8,
        depth_test: bool,
        textures: [u32; 4],
    ) -> Self {
        // Pack into u64 with layer as high priority
        // Sort order: viewport (bits 63-48) → layer (bits 47-32) → stencil (bits 31-24) → textures
        // ...
    }
}
```

#### 1.3 Update Quad Command Creation
**File:** `nethercore-zx/src/graphics/draw.rs:149-160`

Capture layer when creating quad commands:
```rust
self.command_buffer.add_command(VRPCommand::Quad {
    base_vertex: self.unit_quad_base_vertex,
    first_index: self.unit_quad_first_index,
    base_instance,
    instance_count,
    texture_slots,
    depth_test: !is_screen_space && z_state.depth_test,  // DISABLE for screen-space
    cull_mode: CullMode::from_u8(z_state.cull_mode),
    viewport,
    stencil_mode,
    layer: /* capture from quad batch */,  // ADD THIS
});
```

#### 1.4 Update QuadBatch to Store Layer
**File:** `nethercore-zx/src/state/mod.rs` (QuadBatch struct)

Add layer to QuadBatch:
```rust
pub struct QuadBatch {
    pub is_screen_space: bool,
    pub textures: [u32; 4],
    pub instances: Vec<QuadInstance>,
    pub viewport: Viewport,
    pub stencil_mode: u8,
    pub layer: u32,  // ADD THIS - capture from first instance or state
}
```

#### 1.5 Update Frame Rendering Sort
**File:** `nethercore-zx/src/graphics/frame.rs:185-224`

Update sort to extract and use layer:
```rust
self.command_buffer.commands_mut().sort_unstable_by_key(|cmd| match cmd {
    VRPCommand::Quad {
        viewport,
        layer,
        stencil_mode,
        texture_slots,
        depth_test,
        ..
    } => CommandSortKey::quad(
        *viewport,
        *layer,  // Use layer in sort
        *stencil_mode,
        *depth_test,
        [texture_slots[0].0, texture_slots[1].0, texture_slots[2].0, texture_slots[3].0],
    ),
    // ... other cases
});
```

### Phase 2: Disable Depth Testing for Screen-Space Quads

#### 2.1 Update Quad Command Creation Logic
**File:** `nethercore-zx/src/graphics/draw.rs:156`

Change from:
```rust
depth_test: is_screen_space || z_state.depth_test,
```

To:
```rust
depth_test: !is_screen_space && z_state.depth_test,
```

**Rationale:**
- Screen-space quads (2D) → no depth test, rely on layer-based sorting
- World-space quads (3D billboards) → depth test for 3D occlusion

### Phase 3: Remove Depth Hack from layer_to_depth

#### 3.1 Simplify or Remove layer_to_depth
**File:** `nethercore-zx/src/ffi/draw_2d.rs:23-26`

**Option A:** Keep for 3D billboards, but document it's not used for screen-space:
```rust
/// Convert layer to depth for 3D billboards (world-space quads only)
/// Screen-space quads don't use this - they rely on layer-based sorting
#[inline]
fn layer_to_depth(layer: u32) -> f32 {
    1.0 - (layer.min(65535) as f32 / 65535.0)
}
```

**Option B:** Remove entirely if not needed for billboards

### Phase 4: Revert Temporary Fixes

#### 4.1 Revert LessOrEqual Change
**File:** `nethercore-zx/src/graphics/pipeline.rs:317`

Change back from `LessOrEqual` to `Less`:
```rust
depth_compare: if state.depth_test {
    wgpu::CompareFunction::Less  // Back to Less - proper fix is layer sorting
} else {
    wgpu::CompareFunction::Always
},
```

#### 4.2 Keep Stencil Fix
**File:** `nethercore-zx/src/graphics/pipeline.rs:120-134`

**Keep this fix** - it's correct. Mode 0 should explicitly disable stencil operations, not rely on undefined defaults.

## Implementation Checklist

- [ ] Phase 1.1: Add `layer` field to `VRPCommand::Quad`
- [ ] Phase 1.2: Update `CommandSortKey::quad()` to include layer
- [ ] Phase 1.3: Capture layer when creating quad commands
- [ ] Phase 1.4: Add layer to `QuadBatch` struct
- [ ] Phase 1.5: Update frame rendering sort to use layer
- [ ] Phase 2.1: Disable depth testing for screen-space quads
- [ ] Phase 3.1: Document/simplify `layer_to_depth`
- [ ] Phase 4.1: Revert `LessOrEqual` to `Less`
- [ ] Phase 4.2: Verify stencil fix remains

## Testing Plan

### Unit Tests
- [ ] Test that quads with different layers are sorted correctly
- [ ] Test that quads with same layer are batched by texture
- [ ] Test that screen-space quads don't use depth testing
- [ ] Test that world-space quads DO use depth testing

### Integration Tests
1. **hello-world** - Verify text and square render correctly
2. **platformer** - Verify UI text, score, and sprites layer correctly
3. **stencil-demo** - Verify stencil masking still works with UI overlay text
4. **Multiple layers** - Create test with text/sprites at layers 0, 1, 2, verify ordering

### Visual Tests
- [ ] Text at layer 0 renders (no depth test failure)
- [ ] Sprites at different layers render in correct order
- [ ] Overlapping sprites respect layer ordering
- [ ] Stencil masking works with layered UI
- [ ] 3D billboards still work if using world-space quads

## Performance Considerations

**Benefits:**
- ✅ Removes unnecessary depth testing for 2D
- ✅ Better batching within same layer
- ✅ Simpler mental model (layer = sort order, not depth hack)

**Potential Concerns:**
- ⚠️ More sort key bits needed (layer takes 32 bits)
- ⚠️ May reduce batching across layers (intended behavior)

**Mitigation:**
- Use smaller layer range if needed (e.g., 16 bits = 65536 layers)
- Profile before/after to measure impact

## Alternative Approaches Considered

### A. Separate 2D Render Pass
- Render all 3D first
- Clear depth buffer
- Render all 2D in second pass sorted by layer
- **Pros:** Clean separation, guaranteed ordering
- **Cons:** Extra render pass overhead, more complex

### B. Keep Depth Hack, Fix Edge Cases
- Change `layer_to_depth(0)` to return `0.999` instead of `1.0`
- Keep using depth buffer for layer ordering
- **Pros:** Minimal code changes
- **Cons:** Still a hack, wastes depth buffer, not how pro engines work

### C. Chosen Approach (Layer in Sort Key)
- Add layer to CPU sort key
- Disable depth testing for screen-space quads
- **Pros:** Matches industry standard, clean, performant
- **Cons:** Requires more changes

## References

- [Unity 2D Sorting](https://docs.unity3d.com/Manual/2DSorting.html)
- [Bevy 2D Rendering](https://github.com/bevyengine/bevy/blob/main/crates/bevy_core_pipeline/src/core_2d/mod.rs)
- [wgpu Depth/Stencil State](https://docs.rs/wgpu/latest/wgpu/struct.DepthStencilState.html)

## Timeline

**Estimated Effort:** 4-6 hours
- Phase 1: 2-3 hours (add layer to sort key)
- Phase 2: 30 minutes (disable depth for screen-space)
- Phase 3: 30 minutes (cleanup)
- Phase 4: 30 minutes (revert temp fixes)
- Testing: 1-2 hours

## Status

- ✅ **Temporary fixes applied** - quad rendering works with `LessOrEqual` hack
- ⏳ **Proper fix planned** - this document
- ❌ **Not yet implemented** - waiting for implementation phase

---

**Created:** 2025-12-26
**Author:** Claude (with user guidance)
**Status:** Planning Complete - Ready for Implementation
