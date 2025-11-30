// Mode 2: PBR-lite
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Full PBR with up to 4 dynamic lights
// MRE texture in slot 1 (R=Metallic, G=Roughness, B=Emissive)
// Env matcap in slot 2 for environment reflections

const PI: f32 = 3.14159265359;

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame uniforms (group 0)
@group(0) @binding(0) var<uniform> view_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> projection_matrix: mat4x4<f32>;

// Sky uniforms
struct SkyUniforms {
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}

@group(0) @binding(2) var<uniform> sky: SkyUniforms;

// Material uniforms
struct MaterialUniforms {
    color: vec4<f32>,  // Albedo tint
    metallic: f32,
    roughness: f32,
    emissive: f32,
    _pad: f32,
}

@group(0) @binding(3) var<uniform> material: MaterialUniforms;

// Light uniform
struct Light {
    direction: vec3<f32>,  // Directional light (normalized)
    enabled: u32,
    color: vec3<f32>,
    intensity: f32,
}

struct LightUniforms {
    lights: array<Light, 4>,
}

@group(0) @binding(4) var<uniform> lights: LightUniforms;

// Camera position for view direction
@group(0) @binding(5) var<uniform> camera_position: vec3<f32>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(6) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo
@group(1) @binding(1) var slot1: texture_2d<f32>;  // MRE (Metallic-Roughness-Emissive)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Environment matcap
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Unused in Mode 2
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    @location(3) normal: vec3<f32>,  // Required for PBR
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

    // Transform normal to world space
    out.world_normal = normalize(in.normal);

    // Transform normal to view space for matcap UV
    let view_normal = (view_matrix * vec4<f32>(out.world_normal, 0.0)).xyz;
    out.view_normal = normalize(view_normal);

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * world_pos;

    //VS_UV
    //VS_COLOR

    return out;
}

// ============================================================================
// Fragment Shader - PBR-lite
// ============================================================================

// Sample procedural sky
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color.xyz, sky.zenith_color.xyz, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction.xyz));
    let sun = sky.sun_color_and_sharpness.xyz * pow(sun_dot, sky.sun_color_and_sharpness.w);
    return gradient + sun;
}

// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
}

// PBR-lite: Single light + ambient + emissive
fn pbr_lite(
    surface_normal: vec3<f32>,
    view_dir: vec3<f32>,       // surface TO camera
    light_dir: vec3<f32>,      // surface TO light
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    emissive: f32,
    light_color: vec3<f32>,
    ambient_color: vec3<f32>
) -> vec3<f32> {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let half_vec = normalize(light_dir + view_dir);
    let n_dot_l = max(dot(surface_normal, light_dir), 0.0);
    let n_dot_h = dot(surface_normal, half_vec);
    let v_dot_h = max(dot(view_dir, half_vec), 0.0);

    // F0: 4% for dielectrics, albedo for metals
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    // D: GGX distribution
    let d = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let D = alpha2 / (PI * d * d);

    // F: Schlick fresnel (Karis exp2 approximation)
    let F = f0 + (1.0 - f0) * exp2((-5.55473 * v_dot_h - 6.98316) * v_dot_h);

    // Specular (G term omitted)
    let specular = D * F;

    // Diffuse: energy-conserving Lambert
    let diffuse = (1.0 - f0) * (1.0 - metallic) * albedo / PI;

    // Combine
    let direct = (diffuse + specular) * light_color * n_dot_l;
    let ambient = ambient_color * albedo * (1.0 - metallic * 0.9);
    let glow = albedo * emissive;

    return direct + ambient + glow;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get albedo
    var albedo = material.color.rgb;
    //FS_COLOR
    //FS_UV

    // Sample MRE texture (defaults to correct values if not bound)
    var mre = vec3<f32>(material.metallic, material.roughness, material.emissive);
    //FS_MRE

    // View direction
    let view_dir = normalize(camera_position - in.world_position);

    // Accumulate lighting from all enabled lights
    var final_color = vec3<f32>(0.0);
    let ambient = sample_sky(in.world_normal) * 0.3;

    for (var i = 0u; i < 4u; i++) {
        if (lights.lights[i].enabled != 0u) {
            final_color += pbr_lite(
                in.world_normal,
                view_dir,
                lights.lights[i].direction,
                albedo,
                mre.r,  // metallic
                mre.g,  // roughness
                mre.b,  // emissive
                lights.lights[i].color * lights.lights[i].intensity,
                ambient
            );
        }
    }

    // Environment reflection: sky × env matcap (slot 2)
    let reflection_dir = reflect(-view_dir, in.world_normal);
    let env_matcap_uv = compute_matcap_uv(in.view_normal);
    let env_matcap = textureSample(slot2, tex_sampler, env_matcap_uv).rgb;
    let env_reflection = sample_sky(reflection_dir) * env_matcap * mre.r;

    final_color += env_reflection;

    return vec4<f32>(final_color, material.color.a);
}
