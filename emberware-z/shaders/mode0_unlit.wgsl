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

// Sky uniforms for lighting
struct SkyUniforms {
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}

@group(0) @binding(3) var<uniform> sky: SkyUniforms;

// Material uniforms
struct MaterialUniforms {
    color: vec4<f32>,  // RGBA tint color
}

@group(0) @binding(4) var<uniform> material: MaterialUniforms;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// MVP indices storage buffer (per-draw packed indices)
@group(0) @binding(6) var<storage, read> mvp_indices: array<u32>;

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

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    //VOUT_UV
    //VOUT_COLOR
    //VOUT_NORMAL
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

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Fetch MVP index from instance buffer and unpack
    let mvp_packed = mvp_indices[instance_index];
    let mvp = unpack_mvp(mvp_packed);

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

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Simple Lambert shading using sky sun (when normals available)
fn sky_lambert(normal: vec3<f32>) -> vec3<f32> {
    let n_dot_l = max(0.0, dot(normal, sky.sun_direction.xyz));
    let direct = sky.sun_color_and_sharpness.xyz * n_dot_l;
    let ambient = sample_sky(normal) * 0.3;
    return direct + ambient;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color.xyz, sky.zenith_color.xyz, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction.xyz));
    let sun = sky.sun_color_and_sharpness.xyz * pow(sun_dot, sky.sun_color_and_sharpness.w);
    return gradient + sun;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Start with material color (uniform tint)
    var color = material.color.rgb;

    //FS_COLOR
    //FS_UV
    //FS_NORMAL

    return vec4<f32>(color, material.color.a);
}
