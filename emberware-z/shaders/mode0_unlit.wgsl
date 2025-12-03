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

// Per-frame storage buffer - packed MVP indices (model: 16 bits, view: 8 bits, proj: 8 bits, reserved: 32 bits)
// Each entry is 2 × u32: [packed_mvp, reserved]
@group(0) @binding(3) var<storage, read> mvp_indices: array<vec2<u32>>;

// Sky uniforms for lighting
struct SkyUniforms {
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}

@group(0) @binding(4) var<uniform> sky: SkyUniforms;

// Material uniforms
struct MaterialUniforms {
    color: vec4<f32>,  // RGBA tint color
}

@group(0) @binding(5) var<uniform> material: MaterialUniforms;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(6) var<storage, read> bones: array<mat4x4<f32>, 256>;

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
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get packed MVP indices from storage buffer using instance index
    let mvp_packed = mvp_indices[instance_index].x;
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
