// ============================================================================
// COMMON BINDINGS, STRUCTURES, AND UTILITIES
// Shared across all rendering modes
// ============================================================================

// ============================================================================
// Bindings and Data Structures
// ============================================================================

// Per-frame storage buffers - all matrices for the frame
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;

// Packed sky data (16 bytes)
struct PackedSky {
    horizon_color: u32,              // RGBA8 packed
    zenith_color: u32,               // RGBA8 packed
    sun_direction_oct: u32,          // Octahedral encoding (2x snorm16)
    sun_color_and_sharpness: u32,    // RGB8 + sharpness u8
}

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
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 48 bytes (4 × 12-byte lights)
}

// Per-frame storage buffer - array of shading states
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// Per-frame storage buffer - unpacked MVP + shading indices (no bit-packing!)
// Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// 3x4 bone matrix struct (row-major storage, 48 bytes per bone)
// Implicit 4th row is [0, 0, 0, 1] (affine transform)
struct BoneMatrix3x4 {
    row0: vec4<f32>,  // [m00, m01, m02, tx]
    row1: vec4<f32>,  // [m10, m11, m12, ty]
    row2: vec4<f32>,  // [m20, m21, m22, tz]
}

// Bone transforms for GPU skinning (up to 256 bones, 3x4 format)
@group(0) @binding(5) var<storage, read> bones: array<BoneMatrix3x4, 256>;

// Inverse bind matrices for skeletal animation (up to 256 bones, 3x4 format)
// Contains the inverse bind pose for each bone, used in inverse bind mode
@group(0) @binding(6) var<storage, read> inverse_bind: array<BoneMatrix3x4, 256>;

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
// Sky Utilities
// ============================================================================

// Unpacked sky data
struct SkyData {
    horizon_color: vec3<f32>,
    zenith_color: vec3<f32>,
    sun_direction: vec3<f32>,
    sun_color: vec3<f32>,
    sun_sharpness: f32,
}

// Unpack PackedSky to usable values
// Format: 0xRRGGBBSS (R in highest byte, sharpness in lowest)
fn unpack_sky(packed: PackedSky) -> SkyData {
    var sky: SkyData;
    sky.horizon_color = unpack_rgb8(packed.horizon_color);
    sky.zenith_color = unpack_rgb8(packed.zenith_color);
    sky.sun_direction = unpack_octahedral(packed.sun_direction_oct);
    let sun_packed = packed.sun_color_and_sharpness;
    sky.sun_color = unpack_rgb8(sun_packed);
    sky.sun_sharpness = unpack_unorm8_from_u32(sun_packed);  // lowest byte
    return sky;
}

// Sample sky gradient only (no sun disc) - use for ambient when direct sun is computed separately
fn sample_sky_ambient(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    return mix(sky.horizon_color, sky.zenith_color, up_factor);
}

// Sample procedural sky (gradient + sun disc) - use for background or when sun is NOT computed separately
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let gradient = sample_sky_ambient(direction, sky);
    // Negate sun_direction: it's direction rays travel, not direction to sun
    let sun_dot = max(0.0, dot(direction, -sky.sun_direction));
    // Map sharpness [0,1] to power exponent [1, 200]
    // Higher sharpness = higher exponent = sharper sun disc
    let sun_power = mix(1.0, 200.0, sky.sun_sharpness);
    let sun = sky.sun_color * pow(sun_dot, sun_power);
    return gradient + sun;
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

    let model_matrix = model_matrices[model_idx];
    let view_matrix = view_matrices[view_idx];
    let projection_matrix = proj_matrices[proj_idx];

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
