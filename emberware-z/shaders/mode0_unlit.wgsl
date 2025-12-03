// Mode 0: Unlit / Simple Lambert
// Supports all 16 vertex formats (0-15)
// Without normals: flat color
// With normals: simple Lambert shading using sky sun

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame storage buffers - matrix arrays
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Unified shading states storage buffer
struct PackedSky {
    horizon_color: u32,           // RGBA8 packed
    zenith_color: u32,            // RGBA8 packed
    sun_direction: vec4<i32>,     // snorm16x4 (w unused)
    sun_color_and_sharpness: u32, // RGB8 + sharpness u8
}

struct PackedLight {
    direction: vec4<i32>,      // snorm16x4 (w = enabled flag)
    color_and_intensity: u32,  // RGB8 + intensity u8
}

struct UnifiedShadingState {
    metallic: u32,        // u8 packed (will unpack to f32)
    roughness: u32,       // u8 packed
    emissive: u32,        // u8 packed
    pad0: u32,            // padding
    
    color_rgba8: u32,     // Base color RGBA8 packed
    blend_modes: u32,     // 4× u8 packed
    
    sky: PackedSky,       // 16 bytes
    lights: array<PackedLight, 4>, // 64 bytes
}

@group(0) @binding(6) var<storage, read> shading_states: array<UnifiedShadingState>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
}

// Instance data (per-draw indices)
struct InstanceIn {
    @location(10) mvp_index: u32,
    @location(11) shading_index: u32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    //VOUT_UV
    //VOUT_COLOR
    //VOUT_NORMAL
    @location(20) @interpolate(flat) shading_state_index: u32,  // NEW: Pass shading state index to fragment
}

// ============================================================================
// Helper Functions
// ============================================================================

// Unpack MVP index from packed u32 (model: 16 bits, view: 8 bits, proj: 8 bits)
fn unpack_mvp(packed: u32) -> vec3<u32> {
    let model_idx = packed & 0xFFFFu;
    let view_idx = (packed >> 16u) & 0xFFu;
    let proj_idx = (packed >> 24u) & 0xFFu;
    return vec3<u32>(model_idx, view_idx, proj_idx);
}

// Unpack RGBA8 from u32 to vec4<f32>
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Unpack u8 to f32 (0-255 -> 0.0-1.0)
fn unpack_u8_to_f32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

// Unpack snorm16 to vec3<f32> (-32767 to 32767 -> -1.0 to 1.0)
fn unpack_snorm16(packed: vec4<i32>) -> vec3<f32> {
    return vec3<f32>(
        f32(packed.x) / 32767.0,
        f32(packed.y) / 32767.0,
        f32(packed.z) / 32767.0,
    );
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, instance: InstanceIn) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Unpack MVP indices from instance data
    let mvp = unpack_mvp(instance.mvp_index);

    let model_matrix = model_matrices[mvp.x];
    let view_matrix = view_matrices[mvp.y];
    let projection_matrix = proj_matrices[mvp.z];

    // Apply model transform
    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * model_pos;

    //VS_UV
    //VS_COLOR
    //VS_NORMAL

    // Pass shading state index to fragment shader
    out.shading_state_index = instance.shading_index;

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Simple Lambert shading using sky sun (when normals available)
fn sky_lambert(normal: vec3<f32>, state: UnifiedShadingState) -> vec3<f32> {
    // Unpack sky data from shading state
    let sun_dir = normalize(unpack_snorm16(state.sky.sun_direction));
    let sun_color = unpack_rgba8(state.sky.sun_color_and_sharpness).rgb;
    
    let n_dot_l = max(0.0, dot(normal, sun_dir));
    let direct = sun_color * n_dot_l;
    let ambient = sample_sky(normal, state) * 0.3;
    return direct + ambient;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, state: UnifiedShadingState) -> vec3<f32> {
    // Unpack sky colors from shading state
    let horizon = unpack_rgba8(state.sky.horizon_color).rgb;
    let zenith = unpack_rgba8(state.sky.zenith_color).rgb;
    let sun_dir = normalize(unpack_snorm16(state.sky.sun_direction));
    let sun_color = unpack_rgba8(state.sky.sun_color_and_sharpness).rgb;
    let sun_sharpness = unpack_u8_to_f32(state.sky.sun_color_and_sharpness) * 256.0; // Scale back to original range
    
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(horizon, zenith, up_factor);
    let sun_dot = max(0.0, dot(direction, sun_dir));
    let sun = sun_color * pow(sun_dot, sun_sharpness);
    return gradient + sun;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Fetch shading state for this draw
    let state = shading_states[in.shading_state_index];
    let base_color = unpack_rgba8(state.color_rgba8);
    
    // Start with base color from shading state
    var color = base_color.rgb;
    var alpha = base_color.a;

    //FS_COLOR
    //FS_UV
    //FS_NORMAL

    return vec4<f32>(color, alpha);
}
