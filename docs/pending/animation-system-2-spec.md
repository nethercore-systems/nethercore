# Animation System v2 Specification

**Status:** Pending
**Author:** Zerve
**Version:** 2.1
**Last Updated:** December 2025
**Builds on:** `docs/ready/animation-system-spec.md` (v1)
**Revision Notes:** Added error handling, memory analysis, buffer strategy, test coverage expansion

---

## Problem Statement

The current animation system has significant performance issues:

1. **Per-draw bone uploads**: `keyframe_bind()` decodes compressed keyframe data and uploads bone matrices every call
2. **Per-frame inverse bind uploads**: When a skeleton is bound, inverse bind matrices are uploaded via `queue.write_buffer()` every frame
3. **Rollback inefficiency**: During rollback (8+ times/second), the same static animation data is repeatedly decoded and uploaded
4. **VRAM budget confusion**: Animation data (static after init) counts against the 4MB "procedural" VRAM limit meant for dynamic data

### Current Flow
```
keyframe_bind(handle, frame) →
  decode_bone_transform() for each bone (CPU) →
  bone_transform_to_matrix() →
  store in ffi_state.bone_matrices →
  queue.write_buffer() every frame
```

## Proposed Solution

### Core Idea
Pre-decode and upload ALL animation data (keyframe bone matrices + inverse bind matrices) to GPU storage buffers ONCE after `init()`. During gameplay, animation binding becomes a simple index update - no decoding, no uploads.

### New GPU Memory Layout

**Separate from 4MB procedural VRAM budget** (static after init):

```
┌─────────────────────────────────────────────────────────────┐
│ @group(2) @binding(0): all_inverse_bind_mats               │
│ array<BoneMatrix3x4, N>  (grows during init)               │
│                                                             │
│ Layout:                                                     │
│ [skeleton_0_bones...][skeleton_1_bones...][skeleton_N...]  │
│ └─offset=0          └─offset=bone_count_0                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ @group(2) @binding(1): all_keyframes                       │
│ array<BoneMatrix3x4, M>  (grows during init)               │
│                                                             │
│ Layout per animation:                                       │
│ [frame_0_bones...][frame_1_bones...][frame_N_bones...]     │
│                                                             │
│ Each animation at different offset in global buffer         │
└─────────────────────────────────────────────────────────────┘
```

### Index Tracking Structures

```rust
/// Tracks where a skeleton's inverse bind matrices live in the global buffer
pub struct SkeletonGpuInfo {
    pub inverse_bind_offset: u32,  // Start index in all_inverse_bind_mats
    pub bone_count: u8,
}

/// Tracks where an animation's keyframes live in the global buffer
pub struct KeyframeGpuInfo {
    pub keyframe_base_offset: u32,  // Start index in all_keyframes
    pub bone_count: u8,
    pub frame_count: u16,
}
```

### Animation State in Shading State

**Alignment constraint**: PackedUnifiedShadingState must be 16-byte aligned.
- 80 bytes = 5 × 16 ✓
- 96 bytes = 6 × 16 ✓

Add 16 bytes (4 × u32) to `PackedUnifiedShadingState` (now 96 bytes):

```rust
pub struct PackedUnifiedShadingState {
    // ... existing 80 bytes (color, uniforms, flags, sky, lights) ...

    /// Animation state - 16 bytes total for alignment
    /// Full 32-bit indices allow massive animation budgets
    pub keyframe_base: u32,       // Start index in all_keyframes for THIS frame
    pub inverse_bind_base: u32,   // Start index in all_inverse_bind_mats (0 = none)
    pub animation_flags: u32,     // Bit 0: use_dynamic (0 = static, 1 = @group(0) bones)
                                  // Bits 1-31: reserved for v2.1 (blend factor, etc.)
    pub _animation_reserved: u32, // Reserved for v2.1 GPU interpolation
                                  // (second keyframe base for blending)
}
```

> **Important:** The new `animation_flags` field is **separate** from the existing `flags` field.
> The existing `flags` field (in the first 80 bytes) retains its current usage:
> - Bit 0: `FLAG_SKINNING_MODE` (raw vs inverse bind) — **unchanged, still used**
> - Bit 1: `FLAG_TEXTURE_FILTER_LINEAR`
> - Bits 2-7: Material override flags
> - Bits 8-11: `FLAG_UNIFORM_ALPHA_MASK` (dither transparency)
> - Bits 12-15: Dither offsets
>
> The new `animation_flags` field handles buffer selection:
> - Bit 0: `use_dynamic` (0 = read from @group(2) `all_keyframes`, 1 = read from @group(0) `immediate_bones`)
> - Bits 1-7: reserved
> - Bits 8-15: reserved for v2.1 blend factor (0-255 → 0.0-1.0)

**Key insight**: We don't need to store bone_count in the shading state!
- The shader iterates over the 4 bone indices/weights from vertex data (standard GPU skinning)
- bone_count is only needed CPU-side to compute the frame offset
- Once we have the base offset, that's all the shader needs

**v2.1 breadcrumb**: The `_animation_reserved` field can later become:
```rust
pub keyframe_base_b: u32,  // Second keyframe for GPU interpolation
// blend factor packed into animation_flags bits 8-15
```

When `keyframe_bind(handle, frame)` is called:
```
keyframe_base = gpu_info.keyframe_base_offset + (frame * bone_count)  // CPU-side
```

This allows each draw call to reference a different frame without CPU-side decoding.

### Indexed Dynamic Bones (Immediate Mode)

**Problem with current dynamic bones:** The existing `@group(0) @binding(5)` bones buffer only holds ONE bone state. All draws in a frame share it. This forces re-uploads if different meshes need different bone states.

**Solution:** Add `immediate_bones` buffer following the same indexed pattern as `model_matrices`, `view_matrices`, `proj_matrices`:

```rust
/// Per-frame dynamic bones buffer (grows during frame, reset each frame)
/// Similar to how model_matrices accumulates transforms
immediate_bones: Vec<BoneMatrix3x4>,  // Capacity: MAX_IMMEDIATE_BONE_STATES * MAX_BONES

/// When set_bones() is called:
fn set_bones(matrices_ptr: u32, count: u32) -> u32 {
    let base_offset = state.immediate_bones.len() as u32;

    // Append bones to the per-frame buffer (don't overwrite)
    for i in 0..count {
        let matrix = /* decode from WASM memory */;
        state.immediate_bones.push(matrix);
    }

    // Set this draw's keyframe_base to point into immediate_bones
    state.current_keyframe_base = base_offset;
    state.current_use_dynamic = true;
    state.animation_state_dirty = true;

    // Return the bone state index for advanced users who want to reuse
    base_offset
}
```

**KeyframeSource enum** (mirrors `BufferSource` pattern for retained vs immediate meshes):

```rust
/// Determines which buffer the shader reads keyframe bone matrices from.
/// Only relevant for skinned meshes - non-skinned meshes use different shader
/// permutations that don't have skinning code at all.
#[derive(Clone, Copy, Default)]
pub enum KeyframeSource {
    #[default]
    Static { offset: u32 },    // Read from all_keyframes[@group(2)] at offset
    Immediate { offset: u32 }, // Read from immediate_bones[@group(0)] at offset
}
```

**Note:** No `None` variant needed - non-skinned meshes are handled by shader permutations. The 16 vertex format × skinned/unskinned permutations already separate skinned from non-skinned rendering paths.

**Key insight:** Both static and immediate paths use the same indexing pattern:
- `KeyframeSource::Static { offset }` → `all_keyframes[offset + bone_idx]`
- `KeyframeSource::Immediate { offset }` → `immediate_bones[offset + bone_idx]`

The `use_dynamic` flag in `animation_flags` selects which buffer, `keyframe_base` provides the offset.

This allows multiple bone states per frame without re-uploading.

### New Bind Group Structure

```
@group(0) - Per-frame data
  @binding(0): model_matrices      array<mat4x4<f32>>
  @binding(1): view_matrices       array<mat4x4<f32>>
  @binding(2): proj_matrices       array<mat4x4<f32>>
  @binding(3): shading_states      array<PackedUnifiedShadingState>
  @binding(4): mvp_shading_indices array<vec4<u32>>
  @binding(5): immediate_bones     array<BoneMatrix3x4>  (indexed per-draw, data changes each frame)
  @binding(6): (REMOVED - inverse_bind moved to @group(2))
  @binding(7): quad_instances      array<QuadInstance>
  @binding(8): screen_dims         vec2<f32>

@group(1) - Textures (unchanged)
  @binding(0-3): texture slots
  @binding(4): sampler_nearest
  @binding(5): sampler_linear

@group(2) - Static animation data (NEW - uploaded once after init)
  @binding(0): all_inverse_bind_mats  array<BoneMatrix3x4>
  @binding(1): all_keyframes          array<BoneMatrix3x4>
```

**Buffer lifecycle:**
- `@group(0)` buffers: Fixed-size, data written each frame via `queue.write_buffer()`
- `@group(2)` buffers: Sized during init, data uploaded once, never modified

## Implementation Plan

### Phase 0: Fix pending_keyframes Bug (Prerequisite)

**Problem**: The current `resource_manager.rs` processes pending textures, meshes, and skeletons, but **never drains `pending_keyframes`**. This must be fixed before implementing v2.

**File to modify:**
- `emberware-z/src/resource_manager.rs`

**Fix**: Add keyframe processing after skeleton processing (around line 156):

```rust
// Process pending keyframes (move to finalized storage)
for pending in state.pending_keyframes.drain(..) {
    let index = pending.handle as usize - 1;
    while state.keyframes.len() <= index {
        state.keyframes.push(LoadedKeyframeCollection {
            bone_count: 0,
            frame_count: 0,
            data: Vec::new(),
        });
    }
    state.keyframes[index] = LoadedKeyframeCollection {
        bone_count: pending.bone_count,
        frame_count: pending.frame_count,
        data: pending.data,
    };
}
```

### Phase 1: GPU Infrastructure

**Files to modify:**
- `emberware-z/src/graphics/init.rs` - Create static animation buffers and bind group layout
- `emberware-z/src/graphics/frame.rs` - Bind @group(2) once per frame

**Tasks:**
1. Add `all_inverse_bind_buffer: wgpu::Buffer` and `all_keyframes_buffer: wgpu::Buffer` to ZGraphics
2. Create bind group layout for @group(2) with two storage buffer bindings
3. Create **placeholder** bind group initially (before init resources are known)
4. **Recreate** the bind group in Phase 3 after calculating exact buffer sizes
5. Bind @group(2) once at start of render pass (immutable after init)

**Bind Group Lifecycle:**

```
graphics init (Phase 1)
    └─→ Create placeholder buffers (48 bytes each)
    └─→ Create placeholder bind group (valid but minimal)

init() runs (game loads animations)
    └─→ pending_keyframes accumulates

process_pending_resources (Phase 3)
    └─→ Calculate exact buffer sizes
    └─→ Create correctly-sized buffers
    └─→ Upload all animation data
    └─→ RECREATE bind group with final buffers  ← Important!
    └─→ Drop placeholder buffers

render loop
    └─→ Bind final @group(2) once per frame
```

**Empty Animation Handling:**

When a game has no skeletons or keyframes, the placeholder buffers remain in use:

```rust
use crate::state::MAX_BONES;  // 256
const BONE_MATRIX_SIZE: u64 = 48;  // 3×4 f32 = 12 floats × 4 bytes

// Minimum size: 1 BoneMatrix3x4 (48 bytes) - placeholder for empty games
let all_inverse_bind_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("All Inverse Bind Matrices"),
    size: BONE_MATRIX_SIZE,  // Replaced in Phase 3 if skeletons loaded
    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    mapped_at_creation: false,
});

let all_keyframes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("All Keyframes"),
    size: BONE_MATRIX_SIZE,  // Replaced in Phase 3 if animations loaded
    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    mapped_at_creation: false,
});
```

The bind group must be valid even with no animation data. Shaders check `inverse_bind_base > 0u` before accessing, so the placeholder is never read.

### Phase 2: Index Tracking

**Files to modify:**
- `emberware-z/src/state/resources.rs` - Add GPU info structs, KeyframeSource enum
- `emberware-z/src/state/ffi_state.rs` - Track skeleton/keyframe GPU offsets

**Tasks:**
1. Add `KeyframeSource` enum (Static, Immediate) - no None variant needed
2. Add `SkeletonGpuInfo` and `KeyframeGpuInfo` structs
3. Extend `SkeletonData` to include `gpu_offset: u32`
4. Extend `LoadedKeyframeCollection` to include `gpu_base_offset: u32`
5. Add `current_keyframe_source: KeyframeSource` to per-frame state
6. Add `immediate_bones: Vec<BoneMatrix3x4>` to per-frame state (reset each frame)
7. Track global offset counters during init

### Phase 3: Init-Time Upload

**Files to modify:**
- `emberware-z/src/resource_manager.rs` - Process pending animations to GPU

**Tasks:**
1. After all `pending_skeletons` are processed:
   - Concatenate all inverse bind matrices into single buffer
   - Record offset for each skeleton
   - Upload to `all_inverse_bind_buffer`

2. After all `pending_keyframes` are processed:
   - For each keyframe collection, decode ALL frames to BoneMatrix3x4
   - Concatenate into single buffer
   - Record base offset for each collection
   - Upload to `all_keyframes_buffer`

3. Create @group(2) bind group with populated buffers

### Phase 4: Shading State Extension

**Files to modify:**
- `emberware-z/src/graphics/unified_shading_state.rs` - Add animation_state field
- `emberware-z/shaders/common.wgsl` - Update WGSL struct

**Tasks:**
1. Add animation state fields to `PackedUnifiedShadingState` (now 96 bytes):
   - `keyframe_base: u32`
   - `inverse_bind_base: u32`
   - `animation_flags: u32`
   - `_animation_reserved: u32` (zeroed, for v2.1)
2. Update WGSL `PackedUnifiedShadingState` struct (add 4 × u32)
3. Update all size assertions (80 → 96)

### Phase 5: FFI API Changes

**Files to modify:**
- `emberware-z/src/ffi/skinning.rs` - Update skeleton_bind
- `emberware-z/src/ffi/keyframes.rs` - Update keyframe_bind

**New behavior for `skeleton_bind(handle)`:**
```rust
fn skeleton_bind(handle: u32) {
    if handle == 0 {
        // Clear inverse bind base (no skeleton)
        state.current_inverse_bind_base = 0;
    } else {
        // Look up GPU info for this skeleton
        let gpu_info = &state.skeleton_gpu_info[handle - 1];
        state.current_inverse_bind_base = gpu_info.inverse_bind_offset; // full u32
    }
    state.animation_state_dirty = true;
}
```

**New behavior for `keyframe_bind(handle, frame)`:**
```rust
fn keyframe_bind(handle: u32, frame: u32) {
    if handle == 0 {
        // Unbind keyframes - reset to default static offset 0
        // (non-skinned meshes ignore this entirely via shader permutations)
        state.current_keyframe_source = KeyframeSource::Static { offset: 0 };
        state.animation_state_dirty = true;
        return;
    }

    let gpu_info = &state.keyframe_gpu_info[handle - 1];

    // Validate frame index
    if frame >= gpu_info.frame_count as u32 {
        bail!("frame index out of bounds");
    }

    // Compute the global buffer index for this specific frame
    let frame_offset = frame * gpu_info.bone_count as u32;
    let offset = gpu_info.keyframe_base_offset + frame_offset;

    state.current_keyframe_source = KeyframeSource::Static { offset };
    state.animation_state_dirty = true;
}
```

**Updated `set_bones()` with indexed immediate bones:**
```rust
/// Appends bone matrices to the per-frame immediate_bones buffer.
/// Subsequent draws use this bone state until set_bones() or keyframe_bind() is called again.
fn set_bones(matrices_ptr: u32, count: u32) {
    // Record the starting offset in the immediate bones buffer
    let offset = state.immediate_bones.len() as u32;

    // Append to the per-frame buffer (accumulates like model_matrices)
    for i in 0..count {
        let matrix = /* decode BoneMatrix3x4 from WASM memory */;
        state.immediate_bones.push(matrix);
    }

    // Set current draw state to use immediate bones at this offset
    state.current_keyframe_source = KeyframeSource::Immediate { offset };
    state.animation_state_dirty = true;
}
```

**State precedence and usage patterns:** See [Animation State Precedence](#animation-state-precedence) section below for complete documentation.

### Phase 6: Shader Updates

**Files to modify:**
- `emberware-z/shaders/common.wgsl` - Add @group(2) bindings
- `emberware-z/src/graphics/pipeline.rs` - Update shader template generation

**Shader Template System:**

The current skinning code is injected via the `//VS_SKINNED` placeholder in the template system (see `pipeline.rs:create_shader_module()`). The pipeline generates 40 shader permutations at compile-time:
- **Mode 0 (Unlit):** 16 permutations (all 16 vertex formats)
- **Modes 1-3 (Matcap, MR, SS):** 8 permutations each (only formats with NORMAL flag)

The template system uses string replacement to inject mode-specific and format-specific code:
```rust
// In pipeline.rs
let shader_source = COMMON_WGSL
    .replace("//MODE_SPECIFIC", &mode_specific_code)
    .replace("//VS_SKINNED", &skinning_code)
    .replace("//VERTEX_FORMAT", &format_specific_code);
```

**Update tasks:**

1. **Add @group(2) binding declarations to `common.wgsl`** (shared by all permutations):
   ```wgsl
   // Static animation data - bound once after init, never changes
   @group(2) @binding(0) var<storage, read> all_inverse_bind_mats: array<BoneMatrix3x4>;
   @group(2) @binding(1) var<storage, read> all_keyframes: array<BoneMatrix3x4>;
   ```

2. **Update `//VS_SKINNED` template expansion in `pipeline.rs`:**
   - Extract animation fields from expanded `PackedUnifiedShadingState` (indices 20-23 after unpacking)
   - Branch on `use_dynamic` flag to select buffer source
   - Keep existing `skin_vertex()` signature for skinned permutations

3. **Update `create_frame_bind_group_layout()` in `pipeline.rs`:**
   - Remove @binding(6) (inverse_bind moved to @group(2))
   - Keep @binding(5) as `immediate_bones` (renamed from `bones`)

4. **Add `create_static_bind_group_layout()` in `pipeline.rs`:**
   ```rust
   fn create_static_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
       device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
           label: Some("Static Animation Bind Group Layout"),
           entries: &[
               wgpu::BindGroupLayoutEntry {
                   binding: 0,
                   visibility: wgpu::ShaderStages::VERTEX,
                   ty: wgpu::BindingType::Buffer {
                       ty: wgpu::BufferBindingType::Storage { read_only: true },
                       has_dynamic_offset: false,
                       min_binding_size: None,
                   },
                   count: None,
               },
               wgpu::BindGroupLayoutEntry {
                   binding: 1,
                   visibility: wgpu::ShaderStages::VERTEX,
                   ty: wgpu::BindingType::Buffer {
                       ty: wgpu::BufferBindingType::Storage { read_only: true },
                       has_dynamic_offset: false,
                       min_binding_size: None,
                   },
                   count: None,
               },
           ],
       })
   }
   ```

**New shader code:**

> **Note:** `BoneMatrix3x4` uses **row-major** storage (matching existing codebase convention):
> ```wgsl
> struct BoneMatrix3x4 {
>     row0: vec4<f32>,  // [m00, m01, m02, tx]
>     row1: vec4<f32>,  // [m10, m11, m12, ty]
>     row2: vec4<f32>,  // [m20, m21, m22, tz]
> }
> ```
> Use `bone_to_mat4()` helper in `common.wgsl` to convert to column-major 4×4 for WGSL operations.

```wgsl
// @group(0) @binding(5) - Per-frame dynamic bones (indexed, grows during frame)
@group(0) @binding(5) var<storage, read> immediate_bones: array<BoneMatrix3x4>;

// @group(2) - Static animation data (bound once after init)
@group(2) @binding(0) var<storage, read> all_inverse_bind_mats: array<BoneMatrix3x4>;
@group(2) @binding(1) var<storage, read> all_keyframes: array<BoneMatrix3x4>;

// PackedUnifiedShadingState now has 4 new u32 fields at the end:
// - keyframe_base: u32     (index into all_keyframes OR immediate_bones)
// - inverse_bind_base: u32 (index into all_inverse_bind_mats, 0 = none)
// - animation_flags: u32   (bit 0 = use_dynamic: 0=all_keyframes, 1=immediate_bones)
// - _reserved: u32         (for v2.1 GPU interpolation)

// In vertex shader skinning:
// Note: bone_count NOT needed in shader - we iterate over vertex's 4 bone influences
// Both static and dynamic paths use keyframe_base as the starting offset
fn skin_vertex(
    pos: vec3<f32>,
    normal: vec3<f32>,
    bone_indices: vec4<u32>,
    bone_weights: vec4<f32>,
    keyframe_base: u32,
    inverse_bind_base: u32,
    use_dynamic: bool
) -> SkinResult {
    var result = SkinResult(vec3(0.0), vec3(0.0));

    // Iterate over the vertex's 4 bone influences (standard GPU skinning)
    for (var i = 0u; i < 4u; i++) {
        let bone_idx = bone_indices[i];
        let weight = bone_weights[i];
        if weight <= 0.0 { continue; }

        // Both paths use keyframe_base + bone_idx (unified indexing!)
        var bone_mat: BoneMatrix3x4;
        if use_dynamic {
            // Dynamic from per-frame immediate_bones buffer
            bone_mat = immediate_bones[keyframe_base + bone_idx];
        } else {
            // Static from init-time all_keyframes buffer
            bone_mat = all_keyframes[keyframe_base + bone_idx];
        }

        // Apply inverse bind if skeleton is bound
        if inverse_bind_base > 0u {
            let inv_bind = all_inverse_bind_mats[inverse_bind_base + bone_idx];
            bone_mat = multiply_3x4(bone_mat, inv_bind);
        }

        // Accumulate weighted transform
        result.position += weight * transform_point(bone_mat, pos);
        result.normal += weight * transform_normal(bone_mat, normal);
    }

    result.normal = normalize(result.normal);
    return result;
}

// v2.1: GPU interpolation will use _reserved field as keyframe_base_b
// and bits 8-15 of animation_flags as blend factor (0-255 → 0.0-1.0)
```

## Memory Budget

**Constants (from `emberware-z/src/state/mod.rs`):**
```rust
pub const MAX_BONES: usize = 256;
pub const BONE_MATRIX_SIZE: usize = 48;  // 3×4 f32 = 12 floats × 4 bytes
pub const SHADING_STATE_SIZE: usize = 96;  // Updated from 80
```

### Before (4MB procedural VRAM)
- `MAX_BONES` × `BONE_MATRIX_SIZE` = 12KB per skeleton (uploaded every frame)
- Keyframe decoding CPU cost per bind

### After
- **Procedural VRAM (4MB)**: Only dynamic data (`set_bones` fallback, per-frame matrices)
- **Static Animation VRAM (separate)**: Pre-uploaded, immutable
  - Inverse bind: `bone_count × BONE_MATRIX_SIZE` per skeleton (one-time)
  - Keyframes: `bone_count × BONE_MATRIX_SIZE × frame_count` per animation (one-time)

**Typical skeleton sizes** (most games use 20-60 bones, not 256):
- Small character (20 bones): 20 × 48 = 960 bytes
- Medium character (40 bones): 40 × 48 = 1.9 KB
- Complex character (60 bones): 60 × 48 = 2.9 KB
- Maximum (256 bones): 256 × 48 = 12 KB

**Example game** with 10 skeletons (avg 40 bones), 50 animations (avg 60 frames):
- Inverse bind: 10 × 1.9KB = 19KB
- Keyframes: 50 × 60 × 1.9KB = 5.7MB

This is acceptable because:
1. It's static data loaded from ROM
2. Modern GPUs have plenty of VRAM
3. It's separate from the 4MB procedural budget
4. Zero runtime cost after init

### Init-Time Cost Analysis

Pre-decoding ALL keyframes during init adds CPU time. Estimate for the example game above:

| Operation | Data Size | Estimated Time |
|-----------|-----------|----------------|
| Decode compressed keyframes (16B→48B per bone) | 50 × 60 × 40 × 16B = 1.9MB input | ~10ms |
| Transform to matrices (quat→mat3x3) | 50 × 60 × 40 matrices | ~15ms |
| GPU buffer upload | 5.7MB | ~5ms |
| **Total init overhead** | | **~30ms** |

For a game with 100+ animations (large RPG), expect ~60-100ms added to init time. This is acceptable as init happens once at game launch.

### Buffer Growth Strategy

Static animation buffers are sized **once** after `init()` completes:

```rust
// In process_pending_resources(), after draining pending_keyframes:

// 1. Calculate exact sizes needed
let total_inverse_bind_matrices: usize = state.skeletons
    .iter()
    .map(|s| s.bone_count as usize)
    .sum();

let total_keyframe_matrices: usize = state.keyframes
    .iter()
    .map(|k| k.bone_count as usize * k.frame_count as usize)
    .sum();

// 2. Create correctly-sized buffers (or use placeholders if empty)
let inverse_bind_size = (total_inverse_bind_matrices.max(1) * BONE_MATRIX_SIZE) as u64;
let keyframes_size = (total_keyframe_matrices.max(1) * BONE_MATRIX_SIZE) as u64;

// 3. Upload all data in a single write_buffer() call per buffer
// 4. Create final bind group with correctly-sized buffers
```

**Limitation:** Animations cannot be loaded after `init()` completes. The @group(2) buffers are immutable after creation. This matches existing patterns (textures/meshes also loaded during init).

### Peak Memory During Init

During init, keyframe data temporarily exists in multiple locations:

```
Peak memory = WASM linear memory (ROM data)
            + pending_keyframes (compressed, host)
            + decoded matrices (staging, host)
            + GPU buffer (final destination)
```

For the example game (5.7MB keyframes):
- WASM: ~1.9MB (compressed)
- Host staging: ~5.7MB (decoded matrices)
- GPU: ~5.7MB (uploaded)
- **Peak: ~13MB** (briefly, during upload)

After init completes, only GPU memory remains (~5.7MB). The WASM and host staging memory is freed.

## Rollback Benefits

During rollback (8+ times/second):
- **Before**: Decode keyframes → upload bones → upload inverse bind × N rollback frames
- **After**: Update 1 u32 index per draw call

This is a massive win for determinism and performance.

## Animation State Precedence

This section is the canonical reference for how animation state works. Both `keyframe_bind()` and `set_bones()` set `current_keyframe_source`. The last one called before `draw_mesh()` determines which buffer the shader reads.

### Last Call Wins

```rust
keyframe_bind(anim, frame);  // KeyframeSource::Static { offset }
set_bones(ptr, count);       // KeyframeSource::Immediate { offset }
draw_mesh(skinned_mesh);     // Uses Immediate (last call wins)

set_bones(ptr, count);       // KeyframeSource::Immediate { offset }
keyframe_bind(anim, frame);  // KeyframeSource::Static { offset }
draw_mesh(skinned_mesh);     // Uses Static (last call wins)

// Non-skinned meshes ignore keyframe_source entirely (different shader permutation)
draw_mesh(static_mesh);      // No skinning code runs, keyframe_source irrelevant
```

### Common Usage Patterns

```rust
// Pattern 1: Same bones for multiple draws
// Just don't call set_bones() again - state persists
set_bones(blended_bones.as_ptr(), bone_count);
skeleton_bind(skeleton_handle);
draw_mesh(mesh_a);
draw_mesh(mesh_b);  // Reuses same bone state automatically
draw_mesh(mesh_c);  // Still same bones

// Pattern 2: Different bone states in same frame
// Each set_bones() appends a new entry (doesn't overwrite)
set_bones(bones_a.as_ptr(), count);
draw_mesh(mesh_a);
set_bones(bones_b.as_ptr(), count);  // Appends new entry
draw_mesh(mesh_b);

// Pattern 3: Mix static keyframes and dynamic bones
keyframe_bind(walk_anim, walk_frame);
draw_mesh(character_a);  // Uses static keyframe
set_bones(custom_pose.as_ptr(), count);
draw_mesh(character_b);  // Uses dynamic bones
keyframe_bind(run_anim, run_frame);
draw_mesh(character_c);  // Back to static keyframe
```

### Frame Boundary Behavior

At each frame boundary (`clear_frame()`):
- `immediate_bones` buffer is cleared (resets to empty)
- `current_keyframe_source` is reset to `KeyframeSource::Static { offset: 0 }`
- `current_inverse_bind_base` is preserved (skeleton binding persists)

This means:
- Dynamic bone states must be re-set each frame via `set_bones()`
- Static keyframe bindings must be re-set each frame via `keyframe_bind()`
- Skeleton bindings persist across frames (call `skeleton_bind()` once)

## Migration Guide

### Games Using `keyframe_bind()` — Automatic Benefit

**No code changes required.** The `keyframe_bind(handle, frame)` FFI now sets a buffer index instead of decoding + uploading. Performance improves automatically.

```rust
// This code works identically before and after v2
keyframe_bind(anim_handle, current_frame);
skeleton_bind(skeleton_handle);
draw_mesh(mesh_handle);
```

### Games Using `set_bones()` — Continues Working (With New Capabilities)

**No code changes required.** The `set_bones()` FFI now appends to the indexed `immediate_bones` buffer. Existing code works unchanged.

```rust
// Existing code works identically
let blended = lerp_bones(&frame_a, &frame_b, t);
set_bones(blended.as_ptr(), bone_count);
skeleton_bind(skeleton_handle);
draw_mesh(mesh_handle);
```

**NEW capability:** Multiple bone states per frame (each `set_bones()` appends, doesn't overwrite). See [Animation State Precedence](#animation-state-precedence) for complete usage patterns.

### Games Using `keyframe_read()` + Custom Blending — Continues Working

**No code changes required.** The `keyframe_read()` FFI still decodes to WASM memory for CPU-side blending. Can optionally migrate to v2.1 GPU blending later.

### Semantic Changes (Non-Breaking)

#### `set_bones()`: Overwrite → Append

The internal behavior of `set_bones()` changes, but external behavior remains compatible:

**Before (v1):**
```rust
set_bones(bones_a, count);  // Stores in bone_matrices (overwrites)
set_bones(bones_b, count);  // OVERWRITES bone_matrices
draw_mesh(mesh);            // Uses bones_b
```

**After (v2):**
```rust
set_bones(bones_a, count);  // Appends to immediate_bones at offset 0
set_bones(bones_b, count);  // Appends at offset N
draw_mesh(mesh);            // Uses bones_b (current_keyframe_source points to offset N)
```

**Why this isn't breaking:** The last `set_bones()` call before `draw_mesh()` still determines which bones are used. Games that follow the pattern `set_bones() → draw_mesh()` see identical behavior.

**New capability unlocked:** Games can now have multiple distinct bone states in the same frame without re-uploading. Each `set_bones()` call creates a new entry that persists until frame end.

**Memory consideration:** Games that call `set_bones()` repeatedly without drawing (unusual pattern) will accumulate unused bone data in the per-frame buffer. This is bounded by `MAX_IMMEDIATE_BONE_MATRICES` (4096 matrices = 192KB) and cleared each frame.

### Breaking Changes

**None.** This update is fully backwards compatible for all documented usage patterns.

### New Capabilities

After v2 implementation:
- Rollback is now O(1) for animation state (index update only)
- Multiple draw calls can reference different frames without re-upload
- Foundation laid for v2.1 GPU-side frame interpolation

### Future Migration (v2.1)

When GPU interpolation is added, games can optionally migrate:
```rust
// v2.0 (CPU blending via set_bones)
let blended = lerp_bones(&frame_a, &frame_b, t);
set_bones(blended.as_ptr(), bone_count);

// v2.1 (GPU blending - zero CPU cost)
keyframe_blend(anim_handle, frame_a, frame_b, t);
```

## Design Decisions

1. **Buffer indices**: Full 32-bit indices (due to 16-byte alignment requirement → 96 bytes)
   - Max 4 billion bone matrices per buffer - effectively unlimited
   - ROM size is the practical limit

2. **Animation budget**: No separate limit - ROM size already enforces budget

3. **Shading state**: 80 → 96 bytes (16-byte aligned)
   - 20% increase in shading state buffer traffic
   - Acceptable trade-off for massive rollback/runtime gains
   - Includes reserved field for v2.1 GPU interpolation

## Error Handling

### Invalid Handle Access

```rust
fn keyframe_bind(handle: u32, frame: u32) -> Result<()> {
    if handle == 0 {
        // Valid: unbind keyframes
        state.current_keyframe_source = KeyframeSource::Static { offset: 0 };
        return Ok(());
    }

    let index = handle as usize - 1;
    let gpu_info = state.keyframe_gpu_info.get(index)
        .ok_or_else(|| anyhow!("keyframe_bind: invalid handle {}", handle))?;

    // ... continue with valid handle
}

fn skeleton_bind(handle: u32) -> Result<()> {
    if handle == 0 {
        // Valid: unbind skeleton
        state.current_inverse_bind_base = 0;
        return Ok(());
    }

    let index = handle as usize - 1;
    let gpu_info = state.skeleton_gpu_info.get(index)
        .ok_or_else(|| anyhow!("skeleton_bind: invalid handle {}", handle))?;

    // ... continue with valid handle
}
```

### Out-of-Bounds Frame Index

```rust
fn keyframe_bind(handle: u32, frame: u32) -> Result<()> {
    // ... handle validation ...

    if frame >= gpu_info.frame_count as u32 {
        bail!(
            "keyframe_bind: frame {} out of bounds for animation {} (max {})",
            frame, handle, gpu_info.frame_count - 1
        );
    }

    // ... continue with valid frame
}
```

### GPU Buffer Allocation Failure

```rust
fn create_animation_buffers(
    device: &wgpu::Device,
    inverse_bind_size: u64,
    keyframes_size: u64,
) -> Result<(wgpu::Buffer, wgpu::Buffer)> {
    // wgpu panics on allocation failure, so we check limits first
    let limits = device.limits();

    if inverse_bind_size > limits.max_storage_buffer_binding_size as u64 {
        bail!(
            "Animation data too large: inverse bind {} bytes exceeds GPU limit {} bytes",
            inverse_bind_size, limits.max_storage_buffer_binding_size
        );
    }

    if keyframes_size > limits.max_storage_buffer_binding_size as u64 {
        bail!(
            "Animation data too large: keyframes {} bytes exceeds GPU limit {} bytes",
            keyframes_size, limits.max_storage_buffer_binding_size
        );
    }

    // Safe to create buffers
    let inverse_bind_buffer = device.create_buffer(&wgpu::BufferDescriptor { ... });
    let keyframes_buffer = device.create_buffer(&wgpu::BufferDescriptor { ... });

    Ok((inverse_bind_buffer, keyframes_buffer))
}
```

### Immediate Bones Buffer Overflow

```rust
const MAX_IMMEDIATE_BONE_MATRICES: usize = 4096;  // 192KB per frame

fn set_bones(matrices_ptr: u32, count: u32) -> Result<()> {
    let new_total = state.immediate_bones.len() + count as usize;

    if new_total > MAX_IMMEDIATE_BONE_MATRICES {
        bail!(
            "set_bones: would exceed per-frame limit ({} + {} > {})",
            state.immediate_bones.len(), count, MAX_IMMEDIATE_BONE_MATRICES
        );
    }

    // ... continue with append
}
```

### Error Recovery

All errors are **non-fatal** - they return `Err` to the WASM caller, which can choose to:
1. Log and continue with default state (no animation)
2. Retry with different parameters
3. Panic (game's choice)

The graphics state remains valid after any error. Failed operations leave the previous state unchanged.

## Future Work (v2.1)

**GPU Interpolation**: `keyframe_blend(handle, frame_a, frame_b, t)`
- Uses `_animation_reserved` field as `keyframe_base_b`
- Blend factor (0-255) packed in `animation_flags` bits 8-15
- Shader lerps between two frames without CPU involvement
- No API changes needed - reserved fields already allocated

## Implementation Files

| File | Purpose |
|------|---------|
| `emberware-z/src/graphics/init.rs` | Create @group(2) static buffers, remove @binding(6) |
| `emberware-z/src/graphics/frame.rs` | Bind @group(2), upload immediate_bones per-frame |
| `emberware-z/src/graphics/pipeline.rs` | Update `//VS_SKINNED` template for indexed bones |
| `emberware-z/src/graphics/unified_shading_state.rs` | Add 4 animation fields (16 bytes, 80→96) |
| `emberware-z/src/state/resources.rs` | Add KeyframeSource enum, SkeletonGpuInfo, KeyframeGpuInfo |
| `emberware-z/src/state/ffi_state.rs` | Add immediate_bones vec, track GPU offsets |
| `emberware-z/src/resource_manager.rs` | Fix pending_keyframes bug, process to @group(2) |
| `emberware-z/src/ffi/skinning.rs` | Update set_bones() to append to immediate_bones |
| `emberware-z/src/ffi/keyframes.rs` | Update keyframe_bind for static buffer indexing |
| `emberware-z/shaders/common.wgsl` | Add @group(2) bindings, immediate_bones |
| `docs/reference/ffi.md` | Document immediate_bones behavior, state precedence |

---

## Verification Plan

### Unit Tests

Add tests to `emberware-z/src/` test modules:

**Offset Calculations:**
- [ ] `SkeletonGpuInfo` offset calculation correctness
- [ ] `KeyframeGpuInfo` offset calculation correctness
- [ ] Multiple skeletons produce non-overlapping offsets
- [ ] Multiple animations produce non-overlapping offsets

**Bounds Checking:**
- [ ] Frame index bounds checking returns error (not panic)
- [ ] Invalid skeleton handle returns error (not panic)
- [ ] Invalid keyframe handle returns error (not panic)
- [ ] Handle 0 is valid (unbind operation)

**Edge Cases:**
- [ ] Dynamic fallback (`set_bones`) still works
- [ ] Empty animation handling (no skeletons/keyframes loaded)
- [ ] Zero-bone skeleton (degenerate case)
- [ ] Single-frame animation works
- [ ] `MAX_BONES` (256) per skeleton works

**Size Assertions:**
- [ ] `PackedUnifiedShadingState` is exactly 96 bytes
- [ ] `PackedUnifiedShadingState` is 16-byte aligned
- [ ] WGSL struct size matches Rust struct size

### Negative Tests

Verify error paths work correctly:

- [ ] `keyframe_bind(9999, 0)` → error "invalid handle"
- [ ] `keyframe_bind(valid, 9999)` → error "frame out of bounds"
- [ ] `skeleton_bind(9999)` → error "invalid handle"
- [ ] `set_bones()` exceeding `MAX_IMMEDIATE_BONE_MATRICES` → error
- [ ] Loading animations after init → error or warning
- [ ] GPU buffer size exceeding device limits → error with clear message

### State Machine Tests

Verify state transitions work correctly:

- [ ] `keyframe_bind()` → `set_bones()` → uses dynamic bones
- [ ] `set_bones()` → `keyframe_bind()` → uses static keyframes
- [ ] `keyframe_bind()` → `keyframe_bind()` → uses second keyframe
- [ ] `set_bones()` → `set_bones()` → uses second bone state (append behavior)
- [ ] `skeleton_bind(A)` → `skeleton_bind(B)` → uses skeleton B
- [ ] `skeleton_bind(A)` → `skeleton_bind(0)` → no inverse bind applied
- [ ] State persists across multiple `draw_mesh()` calls
- [ ] State resets correctly at frame boundary (`clear_frame()`)

### Integration Tests

Add tests to `examples/` or dedicated test game:

**Basic Rendering:**
- [ ] Skeleton with 1 bone renders correctly
- [ ] Skeleton with `MAX_BONES` (256) renders correctly
- [ ] Animation with 1 frame works
- [ ] Animation with 1000+ frames works

**Multi-Entity Scenarios:**
- [ ] Multiple skeletons bound across different draw calls in same frame
- [ ] Mixed static keyframes + dynamic `set_bones()` in same frame
- [ ] Switching between static and dynamic mid-frame
- [ ] 100 skinned characters in same frame (stress test)

**Animation Playback:**
- [ ] Looping animation plays smoothly
- [ ] Animation plays at correct speed (frame timing)
- [ ] Switching animations mid-play works
- [ ] Blending via `set_bones()` produces smooth transitions

### Rollback Tests

Test with GGRS rollback enabled:

- [ ] Save state → rollback → restore produces identical render output
- [ ] 100 rapid rollbacks don't accumulate visual drift
- [ ] `keyframe_bind()` during rollback replay is deterministic
- [ ] Switching animations mid-rollback works correctly
- [ ] Frame indices remain valid after rollback
- [ ] `immediate_bones` buffer clears correctly on rollback frame reset
- [ ] Animation state serialization is complete (no missed fields)

### Memory Tests

Verify memory behavior:

- [ ] Buffer sizes match expected calculations after init
- [ ] No memory leaks on game shutdown (valgrind/sanitizers)
- [ ] Peak memory during init matches documented estimate
- [ ] Per-frame `immediate_bones` memory is bounded
- [ ] `immediate_bones` clears each frame (no accumulation)
- [ ] Placeholder buffers work for games with no animations

### Regression Tests

Verify backwards compatibility:

- [ ] Existing games using `keyframe_bind()` work unchanged
- [ ] Existing games using `set_bones()` work unchanged
- [ ] Existing games using `keyframe_read()` work unchanged
- [ ] Existing games mixing all three patterns work unchanged
- [ ] `skeleton_bind()` behavior unchanged for existing games
- [ ] Non-skinned meshes render identically (no regression)

### Performance Validation

Measure before and after v2 implementation:

- [ ] Init time acceptable for large animation sets (100+ animations)
- [ ] Frame time during rollback improved (target: <1ms vs current ~5ms)
- [ ] No per-frame buffer uploads when using `keyframe_bind()` (verify with GPU profiler)
- [ ] Memory usage matches expected budget calculations
- [ ] CPU usage during animation playback reduced (no decoding)
- [ ] GPU skinning performance unchanged (same vertex shader work)
