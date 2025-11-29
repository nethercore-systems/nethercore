// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame uniforms (group 0)
@group(0) @binding(0) var<uniform> view_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> projection_matrix: mat4x4<f32>;

// Sky uniforms (for ambient)
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

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo (UV-sampled)
@group(1) @binding(1) var slot1: texture_2d<f32>;  // Matcap 1 (normal-sampled)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Matcap 2 (normal-sampled)
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Matcap 3 (normal-sampled)
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    @location(3) normal: vec3<f32>,  // Required for matcaps
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) view_normal: vec3<f32>,
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Apply model transform (will be integrated later)
    let world_pos = vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;

    // Transform normal to world space (will use model matrix later)
    out.world_normal = normalize(in.normal);

    // Transform normal to view space for matcap UV computation
    let view_normal = (view_matrix * vec4<f32>(out.world_normal, 0.0)).xyz;
    out.view_normal = normalize(view_normal);

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * world_pos;

    //VS_UV
    //VS_COLOR

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
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

    // Compute matcap UV once for all matcaps
    let matcap_uv = compute_matcap_uv(in.view_normal);

    // Sample and multiply matcaps from slots 1-3
    // Slots default to 1×1 white texture if not bound
    color *= textureSample(slot1, tex_sampler, matcap_uv).rgb;
    color *= textureSample(slot2, tex_sampler, matcap_uv).rgb;
    color *= textureSample(slot3, tex_sampler, matcap_uv).rgb;

    return vec4<f32>(color, material.color.a);
}
