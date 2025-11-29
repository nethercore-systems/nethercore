// Mode 0: Unlit / Simple Lambert
// Supports all 16 vertex formats (0-15)
// Without normals: flat color
// With normals: simple Lambert shading using sky sun

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame uniforms (group 0)
@group(0) @binding(0) var<uniform> view_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> projection_matrix: mat4x4<f32>;

// Sky uniforms for lighting
struct SkyUniforms {
    horizon_color: vec3<f32>,
    _pad0: f32,
    zenith_color: vec3<f32>,
    _pad1: f32,
    sun_direction: vec3<f32>,
    _pad2: f32,
    sun_color: vec3<f32>,
    sun_sharpness: f32,
}

@group(0) @binding(2) var<uniform> sky: SkyUniforms;

// Material uniforms
struct MaterialUniforms {
    color: vec4<f32>,  // RGBA tint color
}

@group(0) @binding(3) var<uniform> material: MaterialUniforms;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(4) var<storage, read> bones: array<mat4x4<f32>, 256>;

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
fn vs(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Apply model transform (passed per instance or via push constants)
    // For now, use identity - will be added in later integration
    //VS_POSITION
    out.world_position = world_pos.xyz;

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * world_pos;

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
    let n_dot_l = max(0.0, dot(normal, sky.sun_direction));
    let direct = sky.sun_color * n_dot_l;
    let ambient = sample_sky(normal) * 0.3;
    return direct + ambient;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
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
