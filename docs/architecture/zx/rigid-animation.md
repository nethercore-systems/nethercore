# Proposal: Rigid Animation via Unified API

> Last reviewed: 2026-01-06

**Status**: Draft (Revised v5 - Unified Approach)
**Author**: Claude (AI Assistant)
**Date**: 2025-12-30

## Executive Summary

For Nethercore ZX's retro-focused constraints, **rigid animation** (node animation without skinning) is a legitimate optimization that saves ROM space and GPU cycles. Rather than creating a parallel API, this proposal extends the existing skeletal animation system with a shader branch:

1. **Unified API** - same `keyframe_bind` + `draw_mesh` for both skinned and rigid
2. **Shader branch** - rigid meshes skip weight blending, use bone matrix directly
3. **Mesh metadata** - pack tool stores `attached_bone` for rigid meshes
4. **TRS functions** - quaternion-based transform helpers for procedural animation

---

## The Real Cost of Rigid Skinning

### Per-Mesh Overhead

| Data | Size | Purpose | Wasteful for Rigid? |
|------|------|---------|---------------------|
| JOINTS attribute | 4 bytes/vertex | Bone indices | Yes - all same value |
| WEIGHTS attribute | 16 bytes/vertex | Blend weights | Yes - all [1,0,0,0] |
| Inverse bind matrices | 64 bytes/bone | Skeleton pose | Yes - all identity |

### Example: Factory Scene

**10 machines (3000 vertices, 30 bones total):**
```
Vertex overhead:  3000 verts × 20 bytes = 60,000 bytes
Inverse bind:     30 bones × 64 bytes   =  1,920 bytes
────────────────────────────────────────────────────────
Total waste:                              61,920 bytes (~60KB)
```

For a ZX ROM targeting 256KB-1MB, 60KB is **6-24% of your budget** wasted on identity matrices and redundant weights.

---

## The Solution: Unified API with Shader Branch

### The Key Insight

For rigid meshes, we don't need:
- JOINTS/WEIGHTS vertex attributes
- Inverse bind matrices
- `skeleton_bind()`
- Weight blending in shader

We just need: `bone_matrix[attached_bone] * position`

### How It Works

**Skinned Meshes (current):**
```rust
skeleton_bind(skeleton);   // Provides inverse bind matrices
keyframe_bind(anim, frame);
draw_mesh(skinned_mesh);   // Shader does: Σ(bone * inv_bind * pos * weight)
```

**Rigid Meshes (new - same API!):**
```rust
keyframe_bind(anim, frame);
draw_mesh(rigid_mesh);     // Shader does: bone_matrix[attached_bone] * pos
```

**That's it.** No new functions. The system detects rigid vs skinned from mesh metadata.

---

## Technical Changes

### 1. Mesh ROM Format

Add two fields to mesh metadata:

```
Existing mesh header:
  - vertex_count, index_count, format_flags, etc.

New fields:
  - is_rigid: u8        // 0 = skinned or static, 1 = rigid bone attachment
  - attached_bone: u8   // Which bone this mesh follows (only if is_rigid=1)
```

### 2. Shader Change

Add a uniform branch in the vertex shader:

```wgsl
// In unified_shading_state
var<uniform> shading: ShadingState;
// Existing: keyframe_base, bone_count, inverse_bind_base, etc.
// New: is_rigid, attached_bone_idx

// In vertex shader
fn apply_animation(pos: vec3f) -> vec3f {
    if (shading.is_rigid != 0u) {
        // Rigid path - just use bone matrix directly
        let bone_idx = shading.attached_bone_idx;
        let bone_matrix = all_keyframes[shading.keyframe_base + bone_idx];
        return bone_matrix * vec4f(pos, 1.0);
    } else if (shading.bone_count > 0u) {
        // Skinned path - weight blending with inverse bind
        var result = vec3f(0.0);
        for (var i = 0u; i < 4u; i++) {
            let bone_idx = joints[i];
            let weight = weights[i];
            let bone_matrix = all_keyframes[shading.keyframe_base + bone_idx];
            let inv_bind = all_inverse_bind[shading.inverse_bind_base + bone_idx];
            result += (bone_matrix * inv_bind * vec4f(pos, 1.0)).xyz * weight;
        }
        return result;
    } else {
        // Static mesh - no animation
        return pos;
    }
}
```

### 3. Pack Tool Changes

When extracting from GLB:

```rust
fn extract_mesh(gltf: &Gltf, node: &Node) -> MeshData {
    let mesh = node.mesh();

    // Check if this is a skinned mesh
    let has_skin = mesh.primitives().any(|p| {
        p.attributes().any(|(semantic, _)|
            semantic == Semantic::Joints(0) || semantic == Semantic::Weights(0)
        )
    });

    if has_skin {
        // Skinned mesh - extract with JOINTS/WEIGHTS
        MeshData {
            is_rigid: false,
            attached_bone: 0,
            // ... existing skinned mesh extraction
        }
    } else {
        // Rigid mesh - find which bone/node it's attached to
        let attached_bone = find_bone_index(gltf, node);
        MeshData {
            is_rigid: true,
            attached_bone,
            // ... mesh data without JOINTS/WEIGHTS
        }
    }
}
```

### 4. draw_mesh FFI Change

When drawing, set uniforms based on mesh metadata:

```rust
fn draw_mesh(mesh_handle: u32) {
    let mesh = get_mesh(mesh_handle);

    if mesh.is_rigid {
        state.is_rigid = 1;
        state.attached_bone_idx = mesh.attached_bone;
    } else {
        state.is_rigid = 0;
    }

    // ... existing draw logic
}
```

---

## Usage Examples

### Simple Mechanical Animation

```rust
let anim = rom_keyframes(b"robot-arm");

fn render() {
    let frame = (elapsed_time() * 30.0) as u32 % keyframes_frame_count(anim);

    // Just bind and draw - works for both rigid and skinned!
    keyframe_bind(anim, frame);
    draw_mesh(arm_base);   // Rigid - attached to bone 0
    draw_mesh(arm_joint);  // Rigid - attached to bone 1
    draw_mesh(arm_claw);   // Rigid - attached to bone 2
}
```

### Mixed Skinned + Rigid

```rust
let skeleton = rom_skeleton(b"character");
let walk_anim = rom_keyframes(b"character-walk");
let sword_mesh = rom_mesh(b"sword");  // Rigid, attached to hand bone

fn render() {
    // Skinned character
    skeleton_bind(skeleton);
    keyframe_bind(walk_anim, frame);
    draw_mesh(character_body);  // Skinned mesh

    // Weapon follows hand bone (rigid)
    draw_mesh(sword_mesh);  // System knows it's rigid, uses bone matrix directly
}
```

### Per-Node Control (Optional Enhancement)

For cases where you need explicit bone binding:

```rust
keyframe_bind(anim, frame);

// Override attached bone for this draw
bone_bind(3);  // "Use bone 3 for next draw"
draw_mesh(special_part);
```

---

## New FFI Functions

### Phase 1: Core (Minimal - may not need any!)

The unified approach may require **zero new FFI functions** if mesh metadata handles everything automatically. However, for explicit control:

```c
/// Override attached bone for next rigid mesh draw
/// Useful when same mesh should follow different bones
void bone_bind(uint8_t bone_index);
```

### Phase 2: TRS Functions

```c
/// Set current transform from TRS (Translation, Rotation quaternion, Scale)
void transform_set_trs(
    const float* pos,    // [x, y, z]
    const float* quat,   // [x, y, z, w]
    const float* scale   // [x, y, z]
);

/// Push TRS onto transform stack
void push_trs(
    const float* pos,
    const float* quat,
    const float* scale
);

/// Multiply current transform by TRS
void apply_trs(
    const float* pos,
    const float* quat,
    const float* scale
);

/// Get bone's world transform (for attachments, effects)
void bone_get_world_trs(
    uint8_t bone_index,
    float* out_pos,
    float* out_quat,
    float* out_scale
);

/// Override a bone's transform (for IK, physics, procedural)
void bone_set_trs(
    uint8_t bone_index,
    const float* pos,
    const float* quat,
    const float* scale
);

/// Interpolate between two TRS transforms (host-side for performance)
void trs_lerp(
    const float* a_pos, const float* a_quat, const float* a_scale,
    const float* b_pos, const float* b_quat, const float* b_scale,
    float t,
    float* out_pos, float* out_quat, float* out_scale
);
```

---

## Comparison: Original vs Unified Approach

| Aspect | Original Proposal | Unified Approach |
|--------|-------------------|------------------|
| New FFI functions | 8+ (rom_node_keyframes, draw_node_keyframes, etc.) | 0-1 (optional bone_bind) |
| New ROM format | .nczxnodeanim | None |
| API complexity | Two parallel systems | One system with branch |
| Developer mental model | "Node animation vs skeletal animation" | "Meshes attached to bones" |
| Shader changes | None (used model_matrix path) | Add branch in skinning code |
| ROM savings | ~60KB | ~60KB (same) |

---

## Implementation Plan

### Phase 1: Rigid Mesh Support

1. **Mesh format**: Add `is_rigid` and `attached_bone` fields
2. **Pack tool**: Detect rigid meshes, extract attached bone from GLTF node
3. **unified_shading_state**: Add `is_rigid` and `attached_bone_idx` uniforms
4. **Shader**: Add branch for rigid path
5. **draw_mesh**: Set uniforms based on mesh metadata

### Phase 2: TRS Functions

1. **FFI**: Implement `transform_set_trs`, `push_trs`, `apply_trs`
2. **FFI**: Implement `bone_get_world_trs`, `bone_set_trs`
3. **FFI**: Implement `trs_lerp`

### Phase 3: Optional Enhancements

1. **bone_bind**: Explicit bone override for special cases
2. **Documentation**: Update animation guide

---

## ROM Savings Summary

| Data | Skinned | Rigid | Savings |
|------|---------|-------|---------|
| JOINTS attr | 4 bytes/vert | 0 | **100%** |
| WEIGHTS attr | 16 bytes/vert | 0 | **100%** |
| Inverse bind | 64 bytes/bone | 0 | **100%** |
| Mesh metadata | - | 2 bytes | +2 bytes |

**For 3000 verts, 30 bones:** ~60KB saved.

---

## Conclusion

**This proposal provides:**

1. **Unified API** - same `keyframe_bind` + `draw_mesh` for both skinned and rigid
2. **Zero new animation functions** - existing keyframe system works as-is
3. **Minimal changes** - shader branch + mesh metadata
4. **Full TRS support** - quaternion-based transform functions
5. **Significant ROM savings** - no wasted skinning data for rigid objects

**Recommendation:** Implement Phase 1 (shader branch + mesh metadata) as MVP, add TRS functions in Phase 2.
