// Mode 3: Hybrid (PBR direct + matcap ambient)
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// PBR direct lighting from single directional light
// Matcap (slot 3) for ambient/stylized reflections
// Env matcap (slot 2) for environment reflections

const PI: f32 = 3.14159265359;

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
    metallic_roughness_emissive_pad: u32,  // 4x u8 packed: [metallic, roughness, emissive, pad]
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 32 bytes (4 × 8-byte lights)
}

// Per-frame storage buffer - array of shading states
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// Per-frame storage buffer - packed MVP + shading indices (model: 16 bits, view: 8 bits, proj: 8 bits, shading_state_index: 32 bits)
// Each entry is 2 × u32: [packed_mvp, shading_state_index]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo
@group(1) @binding(1) var slot1: texture_2d<f32>;  // MRE (Metallic-Roughness-Emissive)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Environment matcap
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Ambient matcap
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Unpacking Helper Functions
// ============================================================================

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
    sky.sun_direction = unpack_octahedral(packed.sun_direction_oct);
    let sun_packed = packed.sun_color_and_sharpness;
    sky.sun_color = unpack_rgb8(sun_packed);
    sky.sun_sharpness = unpack_unorm8_from_u32(sun_packed >> 24u);
    return sky;
}

// Unpack PackedLight to usable values
struct LightData {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    enabled: bool,
}

fn unpack_light(packed: PackedLight) -> LightData {
    var light: LightData;
    light.direction = unpack_octahedral(packed.direction_oct);
    light.enabled = (packed.color_and_intensity >> 24u) != 0u;  // intensity byte != 0
    let color_intensity = packed.color_and_intensity;
    light.color = unpack_rgb8(color_intensity);
    light.intensity = unpack_unorm8_from_u32(color_intensity >> 24u);
    return light;
}

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    @location(3) normal: vec3<f32>,  // Required
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) view_normal: vec3<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
    //VOUT_UV
    //VOUT_COLOR
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

    // Transform normal to world space (using model matrix for orthogonal transforms)
    let model_normal = (model_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.world_normal = normalize(model_normal);

    // Transform normal to view space for matcap UV
    let view_normal = (view_matrix * vec4<f32>(out.world_normal, 0.0)).xyz;
    out.view_normal = normalize(view_normal);

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * model_pos;

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;

    //VS_UV
    //VS_COLOR

    return out;
}

// ============================================================================
// Fragment Shader - Hybrid
// ============================================================================

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}

// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
}

// PBR-lite for direct lighting only
fn pbr_direct(
    surface_normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    light_color: vec3<f32>
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

    // F: Schlick fresnel
    let F = f0 + (1.0 - f0) * exp2((-5.55473 * v_dot_h - 6.98316) * v_dot_h);

    // Specular
    let specular = D * F;

    // Diffuse: energy-conserving Lambert
    let diffuse = (1.0 - f0) * (1.0 - metallic) * albedo / PI;

    // Direct lighting only (no ambient here, matcaps provide it)
    return (diffuse + specular) * light_color * n_dot_l;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let sky = unpack_sky(shading.sky);

    // Unpack material properties from packed u32
    let mre_packed = shading.metallic_roughness_emissive_pad;
    let metallic = unpack_unorm8_from_u32(mre_packed & 0xFFu);
    let roughness = unpack_unorm8_from_u32((mre_packed >> 8u) & 0xFFu);
    let emissive_strength = unpack_unorm8_from_u32((mre_packed >> 16u) & 0xFFu);

    // Get albedo
    var albedo = material_color.rgb;
    //FS_COLOR
    //FS_UV

    // Sample MRE texture (defaults to material properties if not bound)
    var mre = vec3<f32>(metallic, roughness, emissive_strength);
    //FS_MRE

    // View direction - calculate from world position to camera
    // Camera position derived from view matrix would require passing view index,
    // so we use world_position directly (view_dir points from surface toward camera)
    let view_dir = normalize(-in.world_position);  // Assumes camera near origin

    // Direct lighting from first directional light (sun)
    // Hybrid mode uses only shading.lights[0]
    let packed_light0 = shading.lights[0];
    let light0 = unpack_light(packed_light0);
    let light_color = light0.color * light0.intensity;

    let direct = pbr_direct(
        in.world_normal,
        view_dir,
        light0.direction,
        albedo,
        mre.r,  // metallic
        mre.g,  // roughness
        light_color
    );

    // Environment reflection: sky × env matcap (slot 2)
    let reflection_dir = reflect(-view_dir, in.world_normal);
    let matcap_uv = compute_matcap_uv(in.view_normal);
    let env_matcap = textureSample(slot2, tex_sampler, matcap_uv).rgb;
    let env_reflection = sample_sky(reflection_dir, sky) * env_matcap * mre.r;

    // Ambient from matcap (slot 3) × sky
    let ambient_matcap = textureSample(slot3, tex_sampler, matcap_uv).rgb;
    let ambient = ambient_matcap * sample_sky(in.world_normal, sky) * albedo * (1.0 - mre.r);

    // Emissive
    let emissive = albedo * mre.b;

    let final_color = direct + env_reflection + ambient + emissive;

    return vec4<f32>(final_color, material_color.a);
}
