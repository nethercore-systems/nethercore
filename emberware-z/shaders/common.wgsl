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

// Packed light data (8 bytes)
struct PackedLight {
    direction_oct: u32,              // Octahedral encoding (2x snorm16)
    color_and_intensity: u32,        // RGB8 + intensity u8 (intensity=0 means disabled)
}

// Unified per-draw shading state (64 bytes)
struct PackedUnifiedShadingState {
    color_rgba8: u32,                // Material color (RGBA8 packed)
    uniform_set_0: u32,              // Mode-specific: [b0, b1, b2, rim_intensity]
    uniform_set_1: u32,              // Mode-specific: [b0, b1, b2, rim_power]
    _pad0: u32,                      // Reserved for alignment/future use
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 32 bytes (4 × 8-byte lights)
}

// Per-frame storage buffer - array of shading states
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// Per-frame storage buffer - unpacked MVP + shading indices (no bit-packing!)
// Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var tex_sampler: sampler;

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
    let u_i16 = i32((packed & 0xFFFFu) << 16u) >> 16;  // Sign-extend low 16 bits
    let v_i16 = i32(packed) >> 16;                      // Arithmetic shift sign-extends

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

// Sample procedural sky (gradient + sun)
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
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
    let L = -light_dir;
    let n_dot_l = max(dot(normal, L), 0.0);
    return albedo * light_color * n_dot_l;
}

// ============================================================================
// Light Utilities
// ============================================================================

// Unpacked light data
struct LightData {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    enabled: bool,
}

// Unpack PackedLight to usable values
// Format: 0xRRGGBBII (R in highest byte, intensity in lowest)
fn unpack_light(packed: PackedLight) -> LightData {
    var light: LightData;
    light.direction = unpack_octahedral(packed.direction_oct);
    light.enabled = (packed.color_and_intensity & 0xFFu) != 0u;  // intensity byte != 0
    let color_intensity = packed.color_and_intensity;
    light.color = unpack_rgb8(color_intensity);
    light.intensity = unpack_unorm8_from_u32(color_intensity);  // lowest byte
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

    //VS_SKINNED

    let indices = mvp_shading_indices[instance_index];
    let model_idx = indices.x;
    let view_idx = indices.y;
    let proj_idx = indices.z;
    let shading_state_idx = indices.w;

    let model_matrix = model_matrices[model_idx];
    let view_matrix = view_matrices[view_idx];
    let projection_matrix = proj_matrices[proj_idx];

    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    //VS_WORLD_NORMAL
    //VS_VIEW_NORMAL

    out.clip_position = projection_matrix * view_matrix * model_pos;
    out.shading_state_index = shading_state_idx;

    //VS_CAMERA_POS

    //VS_UV
    //VS_COLOR

    return out;
}
