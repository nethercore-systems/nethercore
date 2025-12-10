# Skeletal Animation System Specification

**Status:** Design Specification
**Target:** Emberware Z
**Last Updated:** 2025-12-10

---

## Table of Contents

1. [Overview](#overview)
2. [Design Philosophy](#design-philosophy)
3. [Architecture](#architecture)
4. [FFI API Specification](#ffi-api-specification)
5. [Shader Implementation](#shader-implementation)
6. [Asset Pipeline Integration](#asset-pipeline-integration)
7. [Animation Clip Format](#animation-clip-format)
8. [Usage Examples](#usage-examples)
9. [Implementation Checklist](#implementation-checklist)

---

## Overview

This document specifies the skeletal animation system for Emberware Z. The system provides GPU-accelerated skeletal skinning while remaining unopinionated about how developers implement their animation logic.

### Goals

- **Zero friction for common cases** — Standard glTF workflow should "just work"
- **Full flexibility** — Support keyframe, IK, procedural, physics-driven, and pre-baked animation
- **Minimal FFI surface** — Only GPU-related functions cross the FFI boundary
- **Rollback compatible** — All animation state lives in WASM memory (automatically snapshotted)
- **Backward compatible** — Existing code using raw `set_bones()` continues to work

### Non-Goals

- Providing a complete animation runtime (blend trees, state machines, etc.)
- Managing animation playback state
- Storing bone hierarchy on GPU

---

## Design Philosophy

### Separation of Concerns

```
┌─────────────────────────────────────────────────────────────┐
│  WASM Game Code (CPU)                                       │
│                                                             │
│  • Bone hierarchy data (const arrays from export)           │
│  • Animation logic (developer's code)                       │
│  • Keyframe sampling, blending, IK, etc.                    │
│  • Outputs: model-space bone transforms                     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  FFI Boundary                                               │
│                                                             │
│  • load_skeleton() — upload inverse bind matrices (once)    │
│  • skeleton_bind() — enable/disable inverse bind mode       │
│  • set_bones() — upload bone transforms (per frame)         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  GPU                                                        │
│                                                             │
│  • Stores inverse bind matrices                             │
│  • Applies skinning in vertex shader                        │
│  • Computes: final = bone_model × inverse_bind              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### What Lives Where

| Data | Location | Rationale |
|------|----------|-----------|
| Inverse bind matrices | GPU (via FFI) | Used by shader, static data |
| Bone hierarchy (parents) | WASM memory | CPU-only, for hierarchy walk |
| Bone rest pose | WASM memory | CPU-only, for additive anims |
| Bone names | WASM memory | CPU-only, debugging/targeting |
| Current local transforms | WASM memory | Animated state (rollback-safe) |
| Current model transforms | WASM memory | Computed state (rollback-safe) |

### Coordinate Space Clarification

**Model-space** (not world-space): Bone transforms passed to `set_bones()` are relative to the mesh's origin. The object transform (`transform_set()`) is applied separately by the GPU after skinning. This allows the same skinned mesh to be drawn at multiple positions without re-uploading bone data.

```
Skinning pipeline:
1. vertex_local → (bone_model × inverse_bind) → vertex_skinned
2. vertex_skinned → (object_transform) → vertex_world
3. vertex_world → (view × projection) → vertex_clip
```

---

## Architecture

### Two Skinning Modes

**Mode 1: Skeleton Bound (Recommended)**
- Developer uploads **model-space** bone transforms
- GPU automatically applies inverse bind matrices
- Formula: `skinned_pos = Σ(weight × (bone_model × inverse_bind) × vertex)`

**Mode 2: Raw (Backward Compatible)**
- Developer uploads **final skinning matrices** directly
- GPU uses matrices as-is (no inverse bind application)
- Formula: `skinned_pos = Σ(weight × bone_matrix × vertex)`

### Data Flow

```
Developer's Animation System
            │
            ▼
┌───────────────────────┐
│ Compute local transforms │  (keyframes, procedural, etc.)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ Walk bone hierarchy   │  (parent × local for each bone)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ Model-space transforms │  (output of hierarchy walk)
└───────────┬───────────┘
            │
            ▼ set_bones()
┌───────────────────────┐
│ GPU Storage Buffer    │
└───────────┬───────────┘
            │
            ▼ Vertex Shader
┌───────────────────────┐
│ bone_model × inv_bind │  (if skeleton bound)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ Skinned vertex pos    │
└───────────────────────┘
```

---

## FFI API Specification

### New Functions

#### `load_skeleton`

```rust
/// Load a skeleton's inverse bind matrices to GPU.
///
/// Call once during init() after loading skinned meshes.
/// The inverse bind matrices transform vertices from model space
/// to bone-local space at bind time.
///
/// # Arguments
/// * `inverse_bind_ptr` — Pointer to array of 3×4 matrices in WASM memory
///                        (12 floats per matrix, row-major order)
/// * `bone_count` — Number of bones (maximum 256)
///
/// # Returns
/// * Skeleton handle (non-zero) on success
/// * 0 on error (logged to console)
///
/// # Errors
/// * bone_count exceeds 256
/// * inverse_bind_ptr is null or out of bounds
///
/// # Notes
/// * Inverse bind matrices are uploaded immediately and stored on GPU
/// * Uses BoneMatrix3x4 format (same as set_bones) for consistency
fn load_skeleton(inverse_bind_ptr: *const f32, bone_count: u32) -> u32;
```

#### `skeleton_bind`

```rust
/// Bind a skeleton for subsequent skinned mesh rendering.
///
/// When a skeleton is bound, set_bones() expects model-space transforms
/// and the GPU automatically applies the inverse bind matrices.
///
/// # Arguments
/// * `skeleton` — Skeleton handle from load_skeleton(), or 0 to unbind
///
/// # Behavior
/// * skeleton > 0: Enable inverse bind mode. set_bones() receives model transforms.
/// * skeleton = 0: Disable inverse bind mode (raw). set_bones() receives final matrices.
///
/// # Notes
/// * Binding persists until changed (not reset per frame)
/// * Call multiple times per frame to render different skeletons
/// * Invalid handles are ignored with a warning
fn skeleton_bind(skeleton: u32);
```

### Modified Functions

#### `set_bones` (Updated Behavior)

```rust
/// Upload bone transforms for GPU skinning.
///
/// # Arguments
/// * `matrices_ptr` — Pointer to array of 3×4 matrices in WASM memory
///                    (12 floats per matrix, row-major order)
/// * `count` — Number of bones to upload
///
/// # Behavior
///
/// **With skeleton bound (skeleton_bind called with valid handle):**
/// * Expects model-space bone transforms
/// * GPU computes: final_matrix = model_transform × inverse_bind
/// * Use for: keyframe animation, IK, procedural, blending
///
/// **Without skeleton (skeleton_bind(0) or never called):**
/// * Expects final skinning matrices (inverse bind pre-applied)
/// * GPU uses matrices directly
/// * Use for: pre-baked animations, fully custom setups, procedural meshes
///
/// # Validation
/// * If skeleton bound and count != skeleton bone count: warning logged
/// * If count > 256: clamped with warning
fn set_bones(matrices_ptr: *const f32, count: u32);
```

### Existing Functions (Unchanged)

```rust
/// Load a mesh with optional skinning data.
/// When FORMAT_SKINNED is set, vertices include bone indices and weights.
fn load_mesh(data_ptr: *const f32, vertex_count: u32, format: u32) -> u32;
fn load_mesh_indexed(data_ptr: *const f32, vertex_count: u32,
                     indices_ptr: *const u16, index_count: u32, format: u32) -> u32;
```

---

## Shader Implementation

### Storage Buffers

The existing bone buffer uses `BoneMatrix3x4` format. Add a matching buffer for inverse bind matrices:

```wgsl
// Existing: 3x4 bone matrix struct (row-major storage, 48 bytes per bone)
struct BoneMatrix3x4 {
    row0: vec4<f32>,  // [m00, m01, m02, tx]
    row1: vec4<f32>,  // [m10, m11, m12, ty]
    row2: vec4<f32>,  // [m20, m21, m22, tz]
}

// Existing bone transforms
@group(0) @binding(5) var<storage, read> bones: array<BoneMatrix3x4, 256>;

// NEW: Inverse bind matrices (uploaded once via load_skeleton)
@group(0) @binding(6) var<storage, read> inverse_bind: array<BoneMatrix3x4, 256>;
```

### Uniforms

Add skinning mode flag to existing frame uniforms or create minimal uniform:

```wgsl
// Option A: Add to existing uniform struct
// skinning_mode: u32,  // 0 = raw, 1 = apply inverse bind

// Option B: Separate small uniform (simpler)
@group(0) @binding(7) var<uniform> skinning_mode: u32;
```

### Vertex Shader Modification

Update the skinning calculation in `build.rs` shader generation. The inverse bind check is a **runtime branch** within the existing skinned shader permutations (not a new compile-time permutation):

```wgsl
// GPU skinning: compute skinned position and normal
var skinned_pos = vec3<f32>(0.0, 0.0, 0.0);
var skinned_normal = vec3<f32>(0.0, 0.0, 0.0);
//VS_SKINNED_UNPACK_NORMAL

for (var i = 0u; i < 4u; i++) {
    let bone_idx = in.bone_indices[i];
    let weight = in.bone_weights[i];

    if (weight > 0.0 && bone_idx < 256u) {
        // Get bone transform
        var bone_matrix = bone_to_mat4(bones[bone_idx]);

        // Apply inverse bind if skeleton is bound (runtime check)
        if (skinning_mode == 1u) {
            let inv_bind = bone_to_mat4(inverse_bind[bone_idx]);
            bone_matrix = bone_matrix * inv_bind;
        }

        // Accumulate weighted position
        skinned_pos += (bone_matrix * vec4<f32>(in.position, 1.0)).xyz * weight;

        // Accumulate weighted normal (using upper 3x3)
        //VS_SKINNED_NORMAL
    }
}

let final_position = skinned_pos;
//VS_SKINNED_FINAL_NORMAL
```

---

## Asset Pipeline Integration

### Vertex Data Format

Skinned vertices use packed format on GPU:

| Attribute | Format | Size | Description |
|-----------|--------|------|-------------|
| bone_indices | `Uint8x4` | 4 bytes | 4 bone indices (0-255 each) |
| bone_weights | `Unorm8x4` | 4 bytes | 4 weights (normalized 0.0-1.0) |

**Weight normalization**: The exporter MUST ensure weights sum to 1.0. Runtime trusts the data without validation.

### ember-export Output Format

When `ember-export mesh` processes a glTF with skinning data, it outputs:

```rust
// Generated by ember-export vX.X.X from character.gltf
// Skeleton: character_armature (24 bones)

// =============================================================================
// Mesh Data
// =============================================================================

/// Vertex data (FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED)
/// Packed layout: pos(f16x4) + uv(unorm16x2) + normal(oct32) + indices(u8x4) + weights(unorm8x4)
pub const VERTICES: [u8; 16800] = [ /* packed binary data */ ];
pub const INDICES: [u16; 4800] = [ /* ... */ ];
pub const VERTEX_COUNT: u32 = 600;
pub const INDEX_COUNT: u32 = 4800;
pub const VERTEX_FORMAT: u32 = 13; // FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED

// =============================================================================
// Skeleton Data
// =============================================================================

/// Inverse bind matrices — pass to load_skeleton()
/// 24 bones × 12 floats = 288 floats (row-major 3x4 format)
pub const INVERSE_BIND_MATRICES: [f32; 288] = [
    // Bone 0 (root): row0, row1, row2
    1.0, 0.0, 0.0, 0.0,  // row0: [m00, m01, m02, tx]
    0.0, 1.0, 0.0, 0.0,  // row1: [m10, m11, m12, ty]
    0.0, 0.0, 1.0, 0.0,  // row2: [m20, m21, m22, tz]
    // Bone 1 (spine): ...
    // ...
];

/// Bone parent indices — for hierarchy traversal
/// -1 indicates root bone (no parent)
/// Sorted topologically: parents always precede children
pub const BONE_PARENTS: [i32; 24] = [
    -1,  // 0: root (no parent)
     0,  // 1: spine (parent: root)
     1,  // 2: chest (parent: spine)
     2,  // 3: neck (parent: chest)
     3,  // 4: head (parent: neck)
     2,  // 5: shoulder_l (parent: chest)
     5,  // 6: upper_arm_l (parent: shoulder_l)
     6,  // 7: lower_arm_l (parent: upper_arm_l)
     7,  // 8: hand_l (parent: lower_arm_l)
    // ... etc
];

/// Bone rest pose — local transforms in bind/rest position
/// Useful for additive animations, resetting to T-pose
/// 24 bones × 12 floats = 288 floats (row-major 3x4 format)
pub const BONE_REST_LOCAL: [f32; 288] = [ /* ... */ ];

/// Bone names — for debugging and animation targeting
pub const BONE_NAMES: [&str; 24] = [
    "root", "spine", "chest", "neck", "head",
    "shoulder_l", "upper_arm_l", "lower_arm_l", "hand_l",
    // ... etc
];

/// Total bone count
pub const BONE_COUNT: u32 = 24;
```

### glTF Extraction Details

When parsing glTF skin data:

1. **Inverse bind matrices**: Extract from `skin.inverseBindMatrices` accessor, convert to 3x4 row-major
2. **Bone hierarchy**: Build from `skin.joints` array and node parent relationships
3. **Rest pose**: Extract local transforms from joint nodes at bind time
4. **Bone names**: Extract from joint node names
5. **Topological sort**: Ensure `BONE_PARENTS` array has parents before children
6. **Weight normalization**: Normalize weights to sum to 1.0 per vertex

---

## Animation Clip Format

Animation clips are exported separately from skeleton data. Each clip contains keyframes for bone local transforms.

### ember-export Animation Output

```rust
// Generated by ember-export vX.X.X from character_walk.gltf
// Animation: "walk" (1.0s, 30 frames)

/// Animation metadata
pub const ANIMATION_NAME: &str = "walk";
pub const ANIMATION_DURATION: f32 = 1.0;  // seconds
pub const ANIMATION_FRAME_COUNT: u32 = 30;
pub const ANIMATION_FPS: f32 = 30.0;

/// Keyframe times (seconds) — may be non-uniform
pub const KEYFRAME_TIMES: [f32; 30] = [
    0.0, 0.0333, 0.0667, /* ... */ 0.9667, 1.0
];

/// Bone transforms per keyframe
/// Layout: [frame0_bone0, frame0_bone1, ..., frame1_bone0, ...]
/// Each transform is 12 floats (row-major 3x4)
/// Total: 30 frames × 24 bones × 12 floats = 8640 floats
pub const KEYFRAMES: [f32; 8640] = [ /* ... */ ];

/// Which bones are animated (sparse optimization)
/// If all bones animated, this equals 0..BONE_COUNT
pub const ANIMATED_BONES: [u32; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, /* ... */];

/// Interpolation mode (future: could be per-bone)
/// 0 = Step, 1 = Linear, 2 = CubicSpline
pub const INTERPOLATION: u32 = 1;  // Linear
```

### Usage in Game Code

```rust
// Sample animation at time t
fn sample_animation(
    time: f32,
    keyframe_times: &[f32],
    keyframes: &[f32],  // flat array of 3x4 matrices
    bone_count: usize,
    output: &mut [[f32; 12]],
) {
    // Find surrounding keyframes
    let (frame_a, frame_b, blend) = find_keyframes(time, keyframe_times);

    // Interpolate each bone
    for bone in 0..bone_count {
        let offset_a = (frame_a * bone_count + bone) * 12;
        let offset_b = (frame_b * bone_count + bone) * 12;

        // Linear interpolation of 3x4 matrices
        // (For production: decompose to TRS, slerp rotation, lerp T/S)
        for i in 0..12 {
            output[bone][i] = keyframes[offset_a + i] * (1.0 - blend)
                            + keyframes[offset_b + i] * blend;
        }
    }
}
```

---

## Usage Examples

### Example 1: Standard Keyframe Animation

```rust
//! Typical workflow for imported glTF character

#![no_std]
#![no_main]

include!("assets/character.rs");
include!("assets/walk_anim.rs");

static mut MESH: u32 = 0;
static mut SKELETON: u32 = 0;

// Animation state (lives in WASM memory — rollback safe)
static mut BONE_LOCAL: [[f32; 12]; 24] = [[0.0; 12]; 24];
static mut BONE_MODEL: [[f32; 12]; 24] = [[0.0; 12]; 24];
static mut ANIM_TIME: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Load mesh with bone weights
        MESH = load_mesh_indexed(
            VERTICES.as_ptr(),
            VERTEX_COUNT,
            INDICES.as_ptr(),
            INDEX_COUNT,
            VERTEX_FORMAT,
        );

        // Load skeleton (inverse bind matrices) — uploaded to GPU once
        SKELETON = load_skeleton(INVERSE_BIND_MATRICES.as_ptr(), BONE_COUNT);

        // Initialize local transforms to rest pose
        for i in 0..BONE_COUNT as usize {
            copy_mat3x4(&BONE_REST_LOCAL[i * 12..], &mut BONE_LOCAL[i]);
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        ANIM_TIME += 1.0 / 60.0;
        if ANIM_TIME > ANIMATION_DURATION {
            ANIM_TIME -= ANIMATION_DURATION;  // Loop
        }

        // 1. Sample animation → local transforms
        sample_animation(ANIM_TIME, &mut BONE_LOCAL);

        // 2. Walk hierarchy → model-space transforms
        compute_model_transforms(&BONE_LOCAL, &BONE_PARENTS, &mut BONE_MODEL);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Bind skeleton (enables inverse bind application)
        skeleton_bind(SKELETON);

        // Upload model-space transforms (GPU applies inverse bind)
        set_bones(BONE_MODEL.as_ptr() as *const f32, BONE_COUNT);

        // Draw at origin
        transform_identity();
        draw_mesh(MESH);

        // Draw another instance at different position (same bones!)
        transform_set_position(5.0, 0.0, 0.0);
        draw_mesh(MESH);
    }
}

// Helper: Walk hierarchy to compute model-space transforms
fn compute_model_transforms(
    local: &[[f32; 12]; 24],
    parents: &[i32; 24],
    model: &mut [[f32; 12]; 24],
) {
    for i in 0..24 {
        let parent = parents[i];
        if parent < 0 {
            // Root bone: model = local
            model[i] = local[i];
        } else {
            // Child: model = parent_model × local
            mat3x4_multiply(&model[parent as usize], &local[i], &mut model[i]);
        }
    }
}
```

### Example 2: Multiple Characters (Different Skeletons)

```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Character A
        skeleton_bind(SKELETON_A);
        set_bones(bones_a.as_ptr(), BONE_COUNT_A);
        transform_set_position(0.0, 0.0, 0.0);
        draw_mesh(MESH_A);

        // Character B (different skeleton)
        skeleton_bind(SKELETON_B);
        set_bones(bones_b.as_ptr(), BONE_COUNT_B);
        transform_set_position(3.0, 0.0, 0.0);
        draw_mesh(MESH_B);

        // Static props (no skeleton needed)
        skeleton_bind(0);  // Disable inverse bind mode
        transform_set_position(-2.0, 0.0, 0.0);
        draw_mesh(STATIC_PROP);
    }
}
```

### Example 3: Pre-Baked Animation (Fighting Game)

```rust
//! Frame-perfect animation with pre-computed matrices

// Pre-baked at export time: inverse bind already applied
static mut ATTACK_FRAMES: [[[f32; 12]; 24]; 30] = [[[0.0; 12]; 24]; 30];

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Raw mode — no inverse bind application (already baked in)
        skeleton_bind(0);

        // Upload pre-baked final matrices
        set_bones(ATTACK_FRAMES[current_frame].as_ptr() as *const f32, 24);
        draw_mesh(MESH);
    }
}
```

---

## Implementation Checklist

### Phase 1: Core FFI Implementation

- [ ] **Add skeleton storage to ZFFIState**
  - `skeletons: Vec<SkeletonData>` where `SkeletonData { inverse_bind: Vec<[f32; 12]>, bone_count: u32 }`
  - `bound_skeleton: Option<u32>` (currently bound skeleton handle)
  - File: `emberware-z/src/ffi/skinning.rs`

- [ ] **Implement `load_skeleton` FFI**
  - Validate bone count ≤ 256
  - Read 3x4 matrices from WASM memory (12 floats per bone)
  - Store in `ZFFIState.skeletons`
  - Upload to GPU inverse bind buffer immediately
  - Return handle (1-indexed)

- [ ] **Implement `skeleton_bind` FFI**
  - Validate handle or accept 0
  - Set `ZFFIState.bound_skeleton`
  - Update skinning mode uniform (0 = raw, 1 = inverse bind)

- [ ] **Update `set_bones` behavior**
  - Check if skeleton is bound
  - If bound: validate bone count matches, set skinning mode = 1
  - If not bound: set skinning mode = 0

### Phase 2: GPU Implementation

- [ ] **Add inverse bind storage buffer**
  - File: `emberware-z/src/graphics/mod.rs`
  - Create `inverse_bind_buffer: wgpu::Buffer` (256 × 48 bytes = 12KB)
  - Add to bind group layout at binding 6

- [ ] **Add skinning mode uniform**
  - Simple `u32` uniform at binding 7 (or pack into existing uniforms)
  - Updated on `skeleton_bind()` call

- [ ] **Update shader generation**
  - File: `emberware-z/build.rs`
  - Add `inverse_bind` storage buffer declaration to skinned shaders
  - Add `skinning_mode` uniform declaration
  - Update skinning loop with runtime inverse bind check

### Phase 3: Asset Pipeline

- [ ] **Update ember-export mesh converter**
  - Extract `inverseBindMatrices` from glTF skin, convert to 3x4 row-major
  - Extract bone hierarchy from joint nodes
  - Extract bone rest pose (local transforms)
  - Extract bone names
  - Topologically sort bones (parents before children)
  - Normalize bone weights to sum to 1.0
  - Generate Rust const arrays

- [ ] **Add animation clip export**
  - Extract animation data from glTF
  - Output keyframe times and bone transforms
  - Support linear interpolation (cubic later)

- [ ] **Update documentation**
  - File: `docs/emberware-z.md`
  - Document new FFI functions
  - Add skeleton workflow section

### Phase 4: Examples and Testing

- [ ] **Update skinned-mesh example**
  - Demonstrate skeleton-bound mode
  - Show multiple instances with same bones

- [ ] **Create new example: gltf-character**
  - Import actual glTF with skeleton
  - Show standard keyframe workflow
  - Demonstrate animation sampling

- [ ] **Test cases**
  - Skeleton load/bind/unbind
  - Bone count validation
  - Multiple skeletons per frame
  - Mode switching mid-frame

---

## Appendix: Matrix Format Reference

### BoneMatrix3x4 (Row-Major)

All bone matrices use 3x4 row-major format (12 floats). The implicit 4th row is `[0, 0, 0, 1]` (affine transform).

```
Memory layout: [m00, m01, m02, tx, m10, m11, m12, ty, m20, m21, m22, tz]

Matrix interpretation:
| m00 m01 m02 tx |   | row0 |
| m10 m11 m12 ty | = | row1 |
| m20 m21 m22 tz |   | row2 |
| 0   0   0   1  |   (implicit)

Translation is: [tx, ty, tz] = [row0.w, row1.w, row2.w]
```

### Hierarchy Walk

```rust
// Parents array is topologically sorted (parents before children)
for i in 0..bone_count {
    let parent = parents[i];
    if parent < 0 {
        model[i] = local[i];
    } else {
        // Matrix multiply: model = parent_model × local
        model[i] = mat3x4_multiply(model[parent], local[i]);
    }
}
```

### Skinning Formula

```
// For each vertex:
skinned_position = vec3(0)
skinned_normal = vec3(0)

for i in 0..4:
    bone_idx = vertex.bone_indices[i]   // u8, 0-255
    weight = vertex.bone_weights[i]     // unorm8, 0.0-1.0

    if skeleton_bound:
        matrix = bones[bone_idx] × inverse_bind[bone_idx]
    else:
        matrix = bones[bone_idx]

    skinned_position += weight × (matrix × vec4(position, 1.0)).xyz
    skinned_normal += weight × (matrix × vec4(normal, 0.0)).xyz

skinned_normal = normalize(skinned_normal)
```
