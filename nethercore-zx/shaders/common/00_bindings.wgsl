// ============================================================================
// COMMON BINDINGS, STRUCTURES, AND UTILITIES
// Shared across all rendering modes
// ============================================================================

// ============================================================================
// FRAME BIND GROUP (group 0) - grouped by purpose
// ============================================================================
// Reduced from 9 storage buffers to 4 core storage buffers for WebGPU compatibility.
// All mat4x4 matrices merged into unified_transforms.
// All mat3x4 animation matrices merged into unified_animation.
// CPU pre-computes absolute indices into unified_transforms (no frame_offsets needed).
// Screen dimensions eliminated - resolution_index packed into QuadInstance.mode.
//
// Layout (grouped by purpose):
// - Binding 0-1: Transforms (unified_transforms, mvp_shading_indices)
// - Binding 2: Shading (shading_states)
// - Binding 3: Animation (unified_animation)
// - Binding 5: Quad rendering (quad_instances)
// - Binding 6-7: EPU textures (env_radiance, sampler)
// - Binding 8-9: EPU state + frame uniforms
// - Binding 11: EPU SH9 (diffuse irradiance)

// Binding 0: unified_transforms - all mat4x4 matrices [models | views | projs]
// Indices are pre-computed on CPU to be absolute offsets into this array
@group(0) @binding(0) var<storage, read> unified_transforms: array<mat4x4<f32>>;

// Binding 1: mvp_shading_indices - absolute indices into unified_transforms + shading_states
// Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
// view_idx and proj_idx are PRE-OFFSET by CPU (already absolute indices)
@group(0) @binding(1) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// Packed light data (12 bytes) - supports directional and point lights
struct PackedLight {
    data0: u32,  // Directional: octahedral direction (snorm16x2), Point: position XY (f16x2)
    data1: u32,  // RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
    data2: u32,  // Directional: unused (0), Point: position Z + range (f16x2)
}

// Unified per-draw shading state (80 bytes)
struct PackedUnifiedShadingState {
    color_rgba8: u32,                // Material color (RGBA8 packed)
    uniform_set_0: u32,              // Mode-specific: [b0, b1, b2, rim_intensity]
    uniform_set_1: u32,              // Mode-specific: [b0, b1, b2, rim_power]
    flags: u32,                      // Bit 0: skinning_mode (0=raw, 1=inverse bind)
    lights: array<PackedLight, 4>,   // 48 bytes (4 × 12-byte lights)
    // Animation system fields (8 bytes) + padding (4 bytes) + env_id (4 bytes)
    keyframe_base: u32,              // Base offset into all_keyframes buffer
    inverse_bind_base: u32,          // Base offset into inverse_bind buffer
    _pad: u32,                       // Unused padding for struct alignment
    environment_index: u32,          // EPU environment ID (env_id)
}

// Binding 2: shading_states - per-draw shading state array
@group(0) @binding(2) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// 3x4 bone matrix struct (row-major storage, 48 bytes per bone)
// Implicit 4th row is [0, 0, 0, 1] (affine transform)
struct BoneMatrix3x4 {
    row0: vec4<f32>,  // [m00, m01, m02, tx]
    row1: vec4<f32>,  // [m10, m11, m12, ty]
    row2: vec4<f32>,  // [m20, m21, m22, tz]
}

// Binding 3: unified_animation - all mat3x4 animation data [inverse_bind | keyframes | immediate]
// CPU pre-computes keyframe_base to point to the correct section
@group(0) @binding(3) var<storage, read> unified_animation: array<BoneMatrix3x4>;

// Binding 5: quad_instances - for GPU-instanced quad rendering (declared in quad_template.wgsl)
// (not declared here - only used by quad shader)

// ============================================================================
// EPU (Environment Processing Unit) Texture - Binding 6-7
// ============================================================================
// Precomputed octahedral environment radiance maps from EPU compute pipeline.
// The texture is mip-mapped, enabling continuous roughness -> LOD sampling for
// reflections.

// Binding 6: EPU EnvRadiance texture array (octahedral, 256 layers)
@group(0) @binding(6) var epu_env_radiance: texture_2d_array<f32>;

// Binding 7: EPU linear sampler for environment map sampling
@group(0) @binding(7) var epu_sampler: sampler;

// ============================================================================
// EPU STATE + FRAME UNIFORMS (Bindings 8-9)
// ============================================================================
// These enable procedural evaluation of the EPU per-fragment for sky/background
// and for specular high-frequency residuals.

// Packed EPU environment state (128 bytes): 8 x 128-bit layers as vec4<u32>.
struct EpuPackedEnvironmentState {
    layers: array<vec4u, 8>,
}

// Binding 8: Packed EPU environment states (256 entries)
@group(0) @binding(8) var<storage, read> epu_states: array<EpuPackedEnvironmentState>;

// Frame uniforms shared between compute and render EPU evaluation.
struct EpuFrameUniforms {
    active_count: u32,
    map_size: u32,
    _pad0: u32,
    _pad1: u32,
}

// Binding 9: EPU frame uniforms (active_count + map sizing)
@group(0) @binding(9) var<uniform> epu_frame: EpuFrameUniforms;

// ============================================================================
// EPU SH9 Buffer - Binding 11
// ============================================================================
// Pre-computed L2 (9-coefficient) spherical harmonics for diffuse irradiance.
// Much smoother on curved surfaces than the 6-direction ambient cube.

// SH9 storage for L2 diffuse irradiance (9 x vec3 padded to vec4 = 144 bytes)
struct EpuSh9 {
    c0: vec3f, _pad0: f32,
    c1: vec3f, _pad1: f32,
    c2: vec3f, _pad2: f32,
    c3: vec3f, _pad3: f32,
    c4: vec3f, _pad4: f32,
    c5: vec3f, _pad5: f32,
    c6: vec3f, _pad6: f32,
    c7: vec3f, _pad7: f32,
    c8: vec3f, _pad8: f32,
}

// Binding 11: EPU SH9 storage buffer (256 entries)
@group(0) @binding(11) var<storage, read> epu_sh9: array<EpuSh9>;

// Helper to expand 3x4 bone matrix → 4x4 for skinning calculations
// Input is row-major, output is column-major (WGSL mat4x4 convention)
fn bone_to_mat4(bone: BoneMatrix3x4) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(bone.row0.x, bone.row1.x, bone.row2.x, 0.0), // column 0
        vec4<f32>(bone.row0.y, bone.row1.y, bone.row2.y, 0.0), // column 1
        vec4<f32>(bone.row0.z, bone.row1.z, bone.row2.z, 0.0), // column 2
        vec4<f32>(bone.row0.w, bone.row1.w, bone.row2.w, 1.0)  // column 3 (translation)
    );
}

// Skinning mode flag constant (bit 0 of flags field)
const FLAG_SKINNING_MODE: u32 = 1u;

// Texture filter mode flag constant (bit 1 of flags field)
// 0 = nearest (pixelated), 1 = linear (smooth)
const FLAG_TEXTURE_FILTER_LINEAR: u32 = 2u;

// Animation system: CPU pre-computes absolute keyframe_base offsets
// (no flags needed - shader just reads from unified_animation[keyframe_base + bone_idx])

// ============================================================================
// Material Override Flags (bits 2-7)
// See material-overrides-spec.md for details
// ============================================================================

// Uniform color override (bit 2): 0 = texture/vertex, 1 = uniform color_rgba8
const FLAG_USE_UNIFORM_COLOR: u32 = 4u;

// Uniform metallic override (bit 3): Mode 2 = metallic, Mode 3 = spec_damping
const FLAG_USE_UNIFORM_METALLIC: u32 = 8u;

// Uniform roughness override (bit 4): Mode 2 = roughness, Mode 3 = shininess
const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 16u;

// Uniform emissive override (bit 5): 0 = texture, 1 = uniform intensity
const FLAG_USE_UNIFORM_EMISSIVE: u32 = 32u;

// Uniform specular override (bit 6, Mode 3 only): 0 = texture, 1 = uniform RGB
const FLAG_USE_UNIFORM_SPECULAR: u32 = 64u;

// Matcap reflection override (bit 7, Mode 1 only): 0 = environment, 1 = matcap texture
const FLAG_USE_MATCAP_REFLECTION: u32 = 128u;

// ============================================================================
// Normal Mapping Flag (Bit 16)
// ============================================================================

// Skip normal map flag (opt-out): when set, normal map sampling is disabled
// Default behavior (flag NOT set): sample normal map from slot 3 when tangent data exists
// This is developer-friendly: tangent data → normal mapping works automatically
const FLAG_SKIP_NORMAL_MAP: u32 = 0x10000u;

// Helper function to check if a flag is set
fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

// ============================================================================
// Dither Transparency Constants and Helpers (bits 8-15)
// ============================================================================

// Dither field masks in PackedUnifiedShadingState.flags
const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF00u;      // Bits 8-11
const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8u;
const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3000u;   // Bits 12-13
const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12u;
const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0xC000u;   // Bits 14-15
const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14u;

// 4x4 Bayer matrix - classic Saturn/PS1 dither (16 alpha levels)
const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);

// Extract uniform alpha (0-15 → 0.0-1.0)
fn get_uniform_alpha(flags: u32) -> f32 {
    let level = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
    return f32(level) / 15.0;
}

// Extract dither offset
fn get_dither_offset(flags: u32) -> vec2<u32> {
    let x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
    return vec2<u32>(x, y);
}

// Dither transparency check with two-layer alpha system
// - base_alpha: per-pixel alpha from texture/material (0.0-1.0)
// - effect_alpha: global multiplier from uniform_alpha (0-15 → 0.0-1.0)
// Returns true if the fragment should be discarded
fn should_discard_dither(frag_coord: vec2<f32>, flags: u32, base_alpha: f32) -> bool {
    let effect_alpha = get_uniform_alpha(flags);
    let final_alpha = base_alpha * effect_alpha;

    let offset = get_dither_offset(flags);

    // Apply offset to break pattern alignment for stacked meshes
    let x = (u32(frag_coord.x) + offset.x) % 4u;
    let y = (u32(frag_coord.y) + offset.y) % 4u;

    let threshold = BAYER_4X4[y * 4u + x];
    return final_alpha <= threshold;
}

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var sampler_nearest: sampler;
@group(1) @binding(5) var sampler_linear: sampler;

// Sample texture with per-draw filter selection via shading state flags
// This reduces code bloat - all texture sampling goes through this helper
fn sample_filtered(tex: texture_2d<f32>, flags: u32, uv: vec2<f32>) -> vec4<f32> {
    if ((flags & FLAG_TEXTURE_FILTER_LINEAR) != 0u) {
        return textureSample(tex, sampler_linear, uv);
    }
    return textureSample(tex, sampler_nearest, uv);
}
