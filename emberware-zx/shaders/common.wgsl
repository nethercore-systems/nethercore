// ============================================================================
// COMMON BINDINGS, STRUCTURES, AND UTILITIES
// Shared across all rendering modes
// ============================================================================

// ============================================================================
// UNIFIED BUFFER ARCHITECTURE (6 bindings, grouped by purpose)
// ============================================================================
// Reduced from 9 storage buffers to 6 storage for WebGPU compatibility.
// All mat4x4 matrices merged into unified_transforms.
// All mat3x4 animation matrices merged into unified_animation.
// CPU pre-computes absolute indices into unified_transforms (no frame_offsets needed).
// Screen dimensions eliminated - resolution_index packed into QuadInstance.mode.
//
// Layout (grouped by purpose):
// - Binding 0-1: Transforms (unified_transforms, mvp_shading_indices)
// - Binding 2: Shading (shading_states)
// - Binding 3: Animation (unified_animation)
// - Binding 4: Environment (environment_states) - Multi-Environment v3
// - Binding 5: Quad rendering (quad_instances)

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
    // Animation System v2 fields (8 bytes) + padding (4 bytes) + environment index (4 bytes)
    keyframe_base: u32,              // Base offset into all_keyframes buffer
    inverse_bind_base: u32,          // Base offset into inverse_bind buffer
    _pad: u32,                       // Unused padding for struct alignment
    environment_index: u32,          // Index into environment_states buffer
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

// ============================================================================
// ENVIRONMENT SYSTEM (Multi-Environment v3)
// ============================================================================
// Procedural environment rendering with layering and blend modes.
// 8 modes: Gradient, Scatter, Lines, Silhouette, Rectangles, Room, Curtains, Rings

// Packed environment state (48 bytes)
// Header: base_mode(3) + overlay_mode(3) + blend_mode(2) + reserved(24)
// Data: base[0..5], overlay[5..10], shared[10]
struct PackedEnvironmentState {
    header: u32,
    data: array<u32, 11>,
}

// Environment mode constants
const ENV_MODE_GRADIENT: u32 = 0u;
const ENV_MODE_SCATTER: u32 = 1u;
const ENV_MODE_LINES: u32 = 2u;
const ENV_MODE_SILHOUETTE: u32 = 3u;
const ENV_MODE_RECTANGLES: u32 = 4u;
const ENV_MODE_ROOM: u32 = 5u;
const ENV_MODE_CURTAINS: u32 = 6u;
const ENV_MODE_RINGS: u32 = 7u;

// Blend mode constants
const ENV_BLEND_ALPHA: u32 = 0u;     // lerp(base, overlay, overlay.a)
const ENV_BLEND_ADD: u32 = 1u;       // base + overlay
const ENV_BLEND_MULTIPLY: u32 = 2u;  // base * overlay
const ENV_BLEND_SCREEN: u32 = 3u;    // 1 - (1-base) * (1-overlay)

// Binding 4: environment_states - per-frame array of PackedEnvironmentState
@group(0) @binding(4) var<storage, read> environment_states: array<PackedEnvironmentState>;

// Binding 5: quad_instances - for GPU-instanced quad rendering (declared in quad_template.wgsl)
// (not declared here - only used by quad shader)

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

// Animation System v2: CPU pre-computes absolute keyframe_base offsets
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

// Matcap reflection override (bit 7, Mode 1 only): 0 = sky, 1 = matcap texture
const FLAG_USE_MATCAP_REFLECTION: u32 = 128u;

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

// ============================================================================
// Data Unpacking Utilities
// ============================================================================

// Unpack u8 from low byte of u32 to f32 [0.0, 1.0]
fn unpack_unorm8_from_u32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

// Unpack RGBA8 from u32 to vec4<f32>
// Format: 0xRRGGBBAA (R in highest byte, A in lowest)
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Unpack RGB8 from u32 to vec3<f32> (ignore alpha)
fn unpack_rgb8(packed: u32) -> vec3<f32> {
    return unpack_rgba8(packed).rgb;
}

// Decode octahedral encoding to normalized direction
// Uses signed octahedral mapping for uniform precision distribution
fn unpack_octahedral(packed: u32) -> vec3<f32> {
    // Extract i16 components with sign extension
    // NOTE: We use bitcast<i32>() instead of i32() because i32(u32) for values >= 2^31
    // is implementation-defined in WGSL, while bitcast always preserves the bit pattern.
    let u_i16 = bitcast<i32>((packed & 0xFFFFu) << 16u) >> 16;  // Sign-extend low 16 bits
    let v_i16 = bitcast<i32>(packed) >> 16;                      // Arithmetic shift sign-extends

    // Convert snorm16 to float [-1, 1]
    let u = f32(u_i16) / 32767.0;
    let v = f32(v_i16) / 32767.0;

    // Reconstruct 3D direction
    var dir: vec3<f32>;
    dir.x = u;
    dir.y = v;
    dir.z = 1.0 - abs(u) - abs(v);

    // Unfold lower hemisphere (z < 0 case)
    if (dir.z < 0.0) {
        let old_x = dir.x;
        dir.x = (1.0 - abs(dir.y)) * select(-1.0, 1.0, old_x >= 0.0);
        dir.y = (1.0 - abs(old_x)) * select(-1.0, 1.0, dir.y >= 0.0);
    }

    return normalize(dir);
}

// ============================================================================
// Environment Sampling (Multi-Environment v3)
// ============================================================================

// Sample gradient environment (Mode 0)
// data[0]: zenith color (RGBA8)
// data[1]: sky_horizon color (RGBA8)
// data[2]: ground_horizon color (RGBA8)
// data[3]: nadir color (RGBA8)
// data[4]: rotation (f16) + shift (f16) packed
fn sample_gradient(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let zenith = unpack_rgba8(data[offset]);
    let sky_horizon = unpack_rgba8(data[offset + 1u]);
    let ground_horizon = unpack_rgba8(data[offset + 2u]);
    let nadir = unpack_rgba8(data[offset + 3u]);
    let rotation_shift = unpack2x16float(data[offset + 4u]);
    let rotation = rotation_shift.x;
    let shift = rotation_shift.y;

    // Apply Y-axis rotation to direction
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    let rotated_dir = vec3<f32>(
        direction.x * cos_r + direction.z * sin_r,
        direction.y,
        -direction.x * sin_r + direction.z * cos_r
    );

    // Calculate blend factor with shift applied
    let y = clamp(rotated_dir.y + shift, -1.0, 1.0);

    // Smooth 4-color gradient: nadir (-1) -> ground_horizon (-0.1) -> sky_horizon (0.1) -> zenith (1)
    // Small horizon band for smooth ground/sky transition
    var result: vec4<f32>;

    if (y < -0.1) {
        // Below horizon: nadir to ground_horizon
        // Map [-1, -0.1] → [0, 1]
        let t = (y + 1.0) / 0.9;
        result = mix(nadir, ground_horizon, t);
    } else if (y < 0.1) {
        // Horizon band: ground_horizon to sky_horizon
        // Map [-0.1, 0.1] → [0, 1]
        let t = (y + 0.1) / 0.2;
        result = mix(ground_horizon, sky_horizon, t);
    } else {
        // Above horizon: sky_horizon to zenith
        // Map [0.1, 1] → [0, 1]
        let t = (y - 0.1) / 0.9;
        result = mix(sky_horizon, zenith, t);
    }

    return result;
}

// Sample a single environment mode
fn sample_mode(mode: u32, data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    switch (mode) {
        case 0u: { return sample_gradient(data, offset, direction); }
        // Future modes will be added here
        default: { return sample_gradient(data, offset, direction); }
    }
}

// Blend two layers together
fn blend_layers(base: vec4<f32>, overlay: vec4<f32>, mode: u32) -> vec4<f32> {
    switch (mode) {
        case 0u: { return mix(base, overlay, overlay.a); }  // Alpha blend
        case 1u: { return base + overlay; }                  // Add
        case 2u: { return base * overlay; }                  // Multiply
        case 3u: {
            // Screen: 1 - (1-base) * (1-overlay)
            return vec4<f32>(1.0) - (vec4<f32>(1.0) - base) * (vec4<f32>(1.0) - overlay);
        }
        default: { return base; }
    }
}

// Sample complete environment (base + overlay with blend)
fn sample_environment(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    let env = environment_states[env_index];
    let base_mode = env.header & 0x7u;
    let overlay_mode = (env.header >> 3u) & 0x7u;
    let blend_mode = (env.header >> 6u) & 0x3u;

    let base_color = sample_mode(base_mode, env.data, 0u, direction);

    // If same mode for both layers, skip overlay
    if (overlay_mode == base_mode) {
        return base_color;
    }

    let overlay_color = sample_mode(overlay_mode, env.data, 5u, direction);
    return blend_layers(base_color, overlay_color, blend_mode);
}

// Sample environment ambient (average of zenith and horizon)
// Used for material lighting when sky data is needed
fn sample_environment_ambient(env_index: u32, direction: vec3<f32>) -> vec3<f32> {
    let env_color = sample_environment(env_index, direction);
    return env_color.rgb;
}

// Lambert diffuse lighting (unified for all modes)
// light_dir convention: direction rays travel (negated to get L in lighting calculations)
fn lambert_diffuse(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    albedo: vec3<f32>,
    light_color: vec3<f32>,
) -> vec3<f32> {
    // Re-normalize after interpolation (interpolated normals have length < 1.0)
    let N = normalize(normal);
    let L = -light_dir;
    let n_dot_l = max(dot(N, L), 0.0);
    return albedo * light_color * n_dot_l;
}

// Smooth distance attenuation for point lights
// Returns 1.0 at distance=0, 0.0 at distance>=range
fn point_light_attenuation(distance: f32, range: f32) -> f32 {
    if (range <= 0.0) {
        return 0.0;
    }
    let t = clamp(distance / range, 0.0, 1.0);
    // Smooth falloff: quadratic ease-out (feels more natural than linear)
    let inv_t = 1.0 - t;
    return inv_t * inv_t;
}

// ============================================================================
// Light Utilities
// ============================================================================

// Unpacked light data (extended for point lights)
struct LightData {
    direction: vec3<f32>,    // For directional lights
    position: vec3<f32>,     // For point lights
    color: vec3<f32>,
    intensity: f32,
    range: f32,              // For point lights
    light_type: u32,         // 0 = directional, 1 = point
    enabled: bool,
}

// Unpack PackedLight to usable values
// Supports both directional and point lights
fn unpack_light(packed: PackedLight) -> LightData {
    var light: LightData;

    // Extract type from bit 7 of data1
    light.light_type = (packed.data1 >> 7u) & 1u;

    // Extract intensity from bits 0-6, map 0-127 -> 0.0-8.0
    let intensity_7bit = f32(packed.data1 & 0x7Fu);
    light.intensity = intensity_7bit / 127.0 * 8.0;
    light.enabled = intensity_7bit > 0.0;

    // Extract color (bits 31-8)
    light.color = vec3<f32>(
        f32((packed.data1 >> 24u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 16u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 8u) & 0xFFu) / 255.0
    );

    if (light.light_type == 0u) {
        // Directional light: unpack octahedral direction
        light.direction = unpack_octahedral(packed.data0);
        light.position = vec3<f32>(0.0);
        light.range = 0.0;
    } else {
        // Point light: unpack position (f16x2 + f16) and range (f16)
        let xy = unpack2x16float(packed.data0);
        let z_range = unpack2x16float(packed.data2);
        light.position = vec3<f32>(xy.x, xy.y, z_range.x);
        light.range = z_range.y;
        light.direction = vec3<f32>(0.0, -1.0, 0.0);  // Default, unused
    }

    return light;
}

// Computed light data after resolving direction and attenuation
struct ComputedLight {
    direction: vec3<f32>,  // Light direction (ray direction convention)
    color: vec3<f32>,      // Light color × intensity × attenuation
}

// Compute light direction and effective color for both directional and point lights
// Resolves the directional vs point light branching in one place
fn compute_light(light: LightData, world_position: vec3<f32>) -> ComputedLight {
    var result: ComputedLight;

    if (light.light_type == 0u) {
        // Directional light: use stored direction
        result.direction = light.direction;
        result.color = light.color * light.intensity;
    } else {
        // Point light: compute direction and attenuation
        let to_light = light.position - world_position;
        let distance = length(to_light);
        result.direction = -normalize(to_light);  // Negate: convention is "ray direction"
        let attenuation = point_light_attenuation(distance, light.range);
        result.color = light.color * light.intensity * attenuation;
    }

    return result;
}

// ============================================================================
// Unified Vertex Input/Output (all modes)
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    //VOUT_WORLD_NORMAL
    //VOUT_VIEW_NORMAL
    @location(3) @interpolate(flat) shading_state_index: u32,
    //VOUT_VIEW_POS
    //VOUT_CAMERA_POS
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Unified Vertex Shader (all modes)
// ============================================================================

fn extract_camera_position(view_matrix: mat4x4<f32>) -> vec3<f32> {
    let r00 = view_matrix[0][0]; let r10 = view_matrix[0][1]; let r20 = view_matrix[0][2];
    let r01 = view_matrix[1][0]; let r11 = view_matrix[1][1]; let r21 = view_matrix[1][2];
    let r02 = view_matrix[2][0]; let r12 = view_matrix[2][1]; let r22 = view_matrix[2][2];
    let tx = view_matrix[3][0];
    let ty = view_matrix[3][1];
    let tz = view_matrix[3][2];
    return -vec3<f32>(
        r00 * tx + r10 * ty + r20 * tz,
        r01 * tx + r11 * ty + r21 * tz,
        r02 * tx + r12 * ty + r22 * tz
    );
}

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    // Extract indices first (needed by skinning code)
    let indices = mvp_shading_indices[instance_index];
    let model_idx = indices.x;
    let view_idx = indices.y;
    let proj_idx = indices.z;
    let shading_state_idx = indices.w;

    //VS_SKINNED

    // Access unified_transforms directly - indices are pre-offset by CPU
    let model_matrix = unified_transforms[model_idx];
    let view_matrix = unified_transforms[view_idx];
    let projection_matrix = unified_transforms[proj_idx];

    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    //VS_WORLD_NORMAL
    //VS_VIEW_NORMAL
    //VS_VIEW_POS

    out.clip_position = projection_matrix * view_matrix * model_pos;
    out.shading_state_index = shading_state_idx;

    //VS_CAMERA_POS

    //VS_UV
    //VS_COLOR

    return out;
}
