# Shader Generation Changes for Unified Shading State

**Question:** Does shader_gen need to be updated?

**Answer:**
- ✅ **shader_gen.rs** (template replacement logic) - NO CHANGES NEEDED
- ✅ **Shader templates** (mode*.wgsl files) - YES, MAJOR CHANGES REQUIRED

---

## What shader_gen.rs Does

The `shader_gen.rs` file is a **template replacement system**. It:
1. Takes a shader template (e.g., `mode0_unlit.wgsl`)
2. Replaces placeholders based on vertex format flags (UV, COLOR, NORMAL, SKINNED)
3. Returns the final WGSL shader code

**Key insight:** Binding indices are in the **templates**, not in the generator code.

---

## What Needs to Change

### ❌ NO CHANGES: shader_gen.rs
The template replacement logic stays exactly the same. Placeholders like `//VIN_UV`, `//VOUT_COLOR`, etc. don't change.

### ✅ CHANGES REQUIRED: Shader Templates

All 4 shader templates need binding layout updates:
- `emberware-z/shaders/mode0_unlit.wgsl`
- `emberware-z/shaders/mode1_matcap.wgsl`
- `emberware-z/shaders/mode2_pbr.wgsl`
- `emberware-z/shaders/mode3_hybrid.wgsl`

---

## Template Changes (Apply to ALL 4 Modes)

### Before (Example from Mode 0/1):
```wgsl
// Binding 0-3: matrices and MVP indices (OK, no change)
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> mvp_indices: array<u32>;

// Bindings 4-5: REMOVE THESE (redundant)
@group(0) @binding(4) var<uniform> sky: SkyUniforms;        // ❌ REMOVE
@group(0) @binding(5) var<uniform> material: MaterialUniforms; // ❌ REMOVE

// Binding 6: Bones (Mode 0/1)
@group(0) @binding(6) var<storage, read> bones: array<mat4x4<f32>>;
```

### Before (Example from Mode 2/3):
```wgsl
// Bindings 0-3: matrices (same as above)

// Bindings 4-7: REMOVE THESE (redundant)
@group(0) @binding(4) var<uniform> sky: SkyUniforms;        // ❌ REMOVE
@group(0) @binding(5) var<uniform> material: MaterialUniforms; // ❌ REMOVE
@group(0) @binding(6) var<uniform> lights: LightUniforms;   // ❌ REMOVE
@group(0) @binding(7) var<uniform> camera: CameraUniforms;  // ❌ REMOVE

// Binding 8: Bones (Mode 2/3)
@group(0) @binding(8) var<storage, read> bones: array<mat4x4<f32>>;
```

### After (ALL MODES - Unified):
```wgsl
// Bindings 0-3: matrices and MVP indices (UPDATED to vec2<u32>)
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> mvp_shading_indices: array<vec2<u32>>;  // ✅ CHANGED

// Binding 4: Shading states (NEW - replaces bindings 4-7)
@group(0) @binding(4) var<storage, read> shading_states: array<UnifiedShadingState>;

// Binding 5: Bones (MOVED from 6 or 8)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>>;

// Add UnifiedShadingState struct definition
struct UnifiedShadingState {
    params_packed: u32,         // metallic, roughness, emissive, pad
    color_rgba8: u32,
    blend_modes: u32,
    _pad: u32,

    // Sky (16 bytes)
    sky_horizon: u32,
    sky_zenith: u32,
    sky_sun_dir: vec2<i32>,
    sky_sun_color: u32,
    _pad_sky: u32,

    // Lights (64 bytes = 16 bytes × 4)
    light0_dir: vec2<i32>,
    light0_color: u32,
    _pad_l0: u32,

    light1_dir: vec2<i32>,
    light1_color: u32,
    _pad_l1: u32,

    light2_dir: vec2<i32>,
    light2_color: u32,
    _pad_l2: u32,

    light3_dir: vec2<i32>,
    light3_color: u32,
    _pad_l3: u32,
}
```

---

## Vertex Shader Changes

### Before:
```wgsl
@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    //VS_SKINNED

    // Get packed MVP indices
    let mvp_packed = mvp_indices[instance_index];
    let model_idx = mvp_packed & 0xFFFFu;
    let view_idx = (mvp_packed >> 16u) & 0xFFu;
    let proj_idx = (mvp_packed >> 24u) & 0xFFu;

    // ...
}
```

### After:
```wgsl
@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    //VS_SKINNED

    // Get packed MVP + shading state indices
    let indices = mvp_shading_indices[instance_index];
    let mvp_packed = indices.x;                    // ✅ CHANGED
    let shading_state_idx = indices.y;             // ✅ NEW

    let model_idx = mvp_packed & 0xFFFFu;
    let view_idx = (mvp_packed >> 16u) & 0xFFu;
    let proj_idx = (mvp_packed >> 24u) & 0xFFu;

    // ... rest of vertex shader ...

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;   // ✅ NEW
    return out;
}
```

**Note:** Add `shading_state_index: u32` to `VertexOut` struct with appropriate `@location(N)`.

---

## Fragment Shader Changes

### Before:
```wgsl
@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    var color = material.base_color;  // ❌ Using frame-wide uniform

    // Sample textures...
    //FS_APPLY_TEXTURE

    // Use sky uniform...
    let sky_contribution = sky.horizon_color;  // ❌ Using frame-wide uniform

    return vec4<f32>(color, 1.0);
}
```

### After:
```wgsl
@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Fetch shading state for this draw
    let state = shading_states[in.shading_state_index];  // ✅ NEW

    // Unpack material properties
    let params = state.params_packed;
    let metallic = f32((params >> 24u) & 0xFFu) / 255.0;
    let roughness = f32((params >> 16u) & 0xFFu) / 255.0;
    let emissive = f32((params >> 8u) & 0xFFu) / 255.0;

    var color = unpack_rgba8(state.color_rgba8);  // ✅ CHANGED

    // Sample textures...
    //FS_APPLY_TEXTURE

    // Unpack sky
    let sky_horizon = unpack_rgba8(state.sky_horizon);  // ✅ CHANGED
    let sky_zenith = unpack_rgba8(state.sky_zenith);
    let sky_sun_dir = unpack_snorm16_vec3(state.sky_sun_dir.x, state.sky_sun_dir.y);

    // ... lighting calculations using unpacked state ...

    return vec4<f32>(color, 1.0);
}
```

**Note:** Add unpacking helper functions (see implementation plan for details).

---

## Summary of Changes per Template

| File | Current Bones Binding | New Bones Binding | Bindings to Remove | Bindings to Add |
|------|----------------------|-------------------|-------------------|-----------------|
| `mode0_unlit.wgsl` | 6 | 5 | 4 (sky), 5 (material) | 4 (shading_states) |
| `mode1_matcap.wgsl` | 6 | 5 | 4 (sky), 5 (material) | 4 (shading_states) |
| `mode2_pbr.wgsl` | 8 | 5 | 4 (sky), 5 (material), 6 (lights), 7 (camera) | 4 (shading_states) |
| `mode3_hybrid.wgsl` | 8 | 5 | 4 (sky), 5 (material), 6 (lights), 7 (camera) | 4 (shading_states) |

**Result:** All 4 templates end up with identical binding layout (0-5).

---

## Testing

After template updates:
1. ✅ Verify all 40 shader permutations compile
2. ✅ Check that binding 3 is `vec2<u32>` (not `u32`)
3. ✅ Verify bones are at binding 5 in ALL modes
4. ✅ Verify shading_states struct layout matches Rust exactly
5. ✅ Verify unpacking functions are correct

**Compile test:**
```bash
cargo check
# Should regenerate all 40 shaders and compile without errors
```

---

**Last Updated:** December 2024
**Related:** [implementation-plan-unified-shading-state.md](./implementation-plan-unified-shading-state.md)
