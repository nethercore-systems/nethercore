// Mode 0: Unlit / Simple Lambert
// Supports all 16 vertex formats (0-15)
// Without normals: flat color
// With normals: simple Lambert shading using sky sun

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame storage buffers - all matrices for the frame
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;

// Packed sky data (16 bytes)
struct PackedSky {
    horizon_color: u32,              // RGBA8 packed
    zenith_color: u32,               // RGBA8 packed
    sun_direction: vec4<i16>,        // snorm16x4 (w unused)
    sun_color_and_sharpness: u32,    // RGB8 + sharpness u8
}

// Packed light data (16 bytes)
struct PackedLight {
    direction: vec4<i16>,            // snorm16x4 (w = enabled flag: 0x7FFF if enabled, 0 if disabled)
    color_and_intensity: u32,        // RGB8 + intensity u8
}

// Unified per-draw shading state (96 bytes)
struct PackedUnifiedShadingState {
    metallic: u8,
    roughness: u8,
    emissive: u8,
    pad0_byte: u8,
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,
    pad1: u32,
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 64 bytes
}

// Per-frame storage buffer - array of shading states
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// Per-frame storage buffer - packed MVP + shading indices (model: 16 bits, view: 8 bits, proj: 8 bits, shading_state_index: 32 bits)
// Each entry is 2 × u32: [packed_mvp, shading_state_index]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Unpacking Helper Functions
// ============================================================================

// Unpack u8 to f32 [0.0, 1.0]
fn unpack_unorm8(packed: u8) -> f32 {
    return f32(packed) / 255.0;
}

// Unpack u8 from low byte of u32 to f32 [0.0, 1.0]
fn unpack_unorm8_from_u32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

// Unpack RGBA8 from u32 to vec4<f32>
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Unpack RGB8 from u32 to vec3<f32> (ignore alpha)
fn unpack_rgb8(packed: u32) -> vec3<f32> {
    return unpack_rgba8(packed).rgb;
}

// Unpack snorm16 to f32 [-1.0, 1.0]
fn unpack_snorm16(packed: i16) -> f32 {
    return f32(packed) / 32767.0;
}

// Unpack vec4<i16> direction to vec3<f32>
fn unpack_direction(packed: vec4<i16>) -> vec3<f32> {
    return vec3<f32>(
        unpack_snorm16(packed.x),
        unpack_snorm16(packed.y),
        unpack_snorm16(packed.z)
    );
}

// Unpack PackedSky to usable values
struct SkyData {
    horizon_color: vec3<f32>,
    zenith_color: vec3<f32>,
    sun_direction: vec3<f32>,
    sun_color: vec3<f32>,
    sun_sharpness: f32,
}

fn unpack_sky(packed: PackedSky) -> SkyData {
    var sky: SkyData;
    sky.horizon_color = unpack_rgb8(packed.horizon_color);
    sky.zenith_color = unpack_rgb8(packed.zenith_color);
    sky.sun_direction = unpack_direction(packed.sun_direction);
    let sun_packed = packed.sun_color_and_sharpness;
    sky.sun_color = unpack_rgb8(sun_packed);
    sky.sun_sharpness = unpack_unorm8_from_u32(sun_packed >> 24u);
    return sky;
}

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

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) @interpolate(flat) shading_state_index: u32,
    //VOUT_UV
    //VOUT_COLOR
    //VOUT_NORMAL
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get packed MVP indices and shading state index from storage buffer
    let mvp_data = mvp_shading_indices[instance_index];
    let mvp_packed = mvp_data.x;
    let shading_state_idx = mvp_data.y;

    // Unpack MVP indices
    let model_idx = mvp_packed & 0xFFFFu;           // Lower 16 bits
    let view_idx = (mvp_packed >> 16u) & 0xFFu;     // Next 8 bits
    let proj_idx = (mvp_packed >> 24u) & 0xFFu;     // Top 8 bits

    // Get matrices from storage buffers using unpacked indices
    let model_matrix = model_matrices[model_idx];
    let view_matrix = view_matrices[view_idx];
    let projection_matrix = proj_matrices[proj_idx];

    // Apply model transform
    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * model_pos;

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;

    //VS_UV
    //VS_COLOR
    //VS_NORMAL

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Simple Lambert shading using sky sun (when normals available)
fn sky_lambert(normal: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let n_dot_l = max(0.0, dot(normal, sky.sun_direction));
    let direct = sky.sun_color * n_dot_l;
    let ambient = sample_sky(normal, sky) * 0.3;
    return direct + ambient;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let sky = unpack_sky(shading.sky);

    // Start with material color
    var color = material_color.rgb;

    //FS_COLOR
    //FS_UV
    //FS_NORMAL

    return vec4<f32>(color, material_color.a);
}
