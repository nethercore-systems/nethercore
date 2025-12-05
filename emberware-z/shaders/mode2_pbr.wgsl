// Mode 2: PBR-lite
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Full PBR with up to 4 dynamic lights
// MRE texture in slot 1 (R=Metallic, G=Roughness, B=Emissive)
// Env matcap in slot 2 for environment reflections

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

// Per-frame storage buffer - unpacked MVP + shading indices (no bit-packing!)
// Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo
@group(1) @binding(1) var slot1: texture_2d<f32>;  // MRE (Metallic-Roughness-Emissive)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Environment matcap
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Unused in Mode 2
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
    @location(3) normal: vec3<f32>,  // Required for PBR
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) view_normal: vec3<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
    @location(4) @interpolate(flat) camera_position: vec3<f32>,  // Camera position in world space (flat, not interpolated)
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Camera Position Extraction
// ============================================================================

// Extract camera position from view matrix
// View matrix transforms world→view, so camera is at origin in view space
// Camera position in world space = -R^T * translation_part
fn extract_camera_position(view_matrix: mat4x4<f32>) -> vec3<f32> {
    // Extract rotation part (upper 3x3) - matrices are column-major in WGSL
    let r00 = view_matrix[0][0]; let r10 = view_matrix[0][1]; let r20 = view_matrix[0][2];
    let r01 = view_matrix[1][0]; let r11 = view_matrix[1][1]; let r21 = view_matrix[1][2];
    let r02 = view_matrix[2][0]; let r12 = view_matrix[2][1]; let r22 = view_matrix[2][2];

    // Extract translation part (4th column, first 3 rows)
    let tx = view_matrix[3][0];
    let ty = view_matrix[3][1];
    let tz = view_matrix[3][2];

    // Camera position = -R^T * t
    return -vec3<f32>(
        r00 * tx + r10 * ty + r20 * tz,
        r01 * tx + r11 * ty + r21 * tz,
        r02 * tx + r12 * ty + r22 * tz
    );
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get unpacked MVP + shading indices from storage buffer (no bit-packing!)
    let indices = mvp_shading_indices[instance_index];
    let model_idx = indices.x;
    let view_idx = indices.y;
    let proj_idx = indices.z;
    let shading_state_idx = indices.w;

    // Get matrices from storage buffers using indices
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

    // Extract camera position from view matrix (for correct specular calculations)
    out.camera_position = extract_camera_position(view_matrix);

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;

    //VS_UV
    //VS_COLOR

    return out;
}

// ============================================================================
// Fragment Shader - PBR-lite
// ============================================================================

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    // Negate sun_direction: it's direction rays travel, not direction to sun
    let sun_dot = max(0.0, dot(direction, -sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}

// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
}

// PBR-lite: Per-light direct lighting only
// Convention: light_dir = direction rays travel (negate for lighting calculations)
// NOTE: Emissive should be added ONCE in main shader, not per-light!
fn pbr_lite(
    surface_normal: vec3<f32>,
    view_dir: vec3<f32>,       // surface TO camera
    light_dir: vec3<f32>,      // direction rays travel
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    light_color: vec3<f32>,
) -> vec3<f32> {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    // Negate light_dir because it represents "direction rays travel", not "direction to light"
    let to_light = -light_dir;
    let half_vec = normalize(to_light + view_dir);
    let n_dot_l = max(dot(surface_normal, to_light), 0.0);
    let n_dot_h = max(dot(surface_normal, half_vec), 0.0);
    let v_dot_h = max(dot(view_dir, half_vec), 0.0);

    // F0: 4% for dielectrics, albedo for metals
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    // D: GGX distribution with exp2 approximation (5th gen console charm!)
    // This adds subtle artifacts at grazing angles for that retro console feel
    // Based on Hammon 2017 "PBR Diffuse Lighting for GGX+Smith Microsurfaces"
    let d = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let D = (alpha2 / PI) * exp2(-2.0 * log2(d));

    // F: Schlick fresnel (Karis exp2 approximation)
    let F = f0 + (1.0 - f0) * exp2((-5.55473 * v_dot_h - 6.98316) * v_dot_h);

    // Specular (G term omitted)
    let specular = (D * F) / 4.0;

    // Diffuse: energy-conserving Lambert
    let diffuse = (1.0 - f0) * (1.0 - metallic) * albedo / PI;

    // Direct lighting only (no emissive - that's a material property, not lighting)
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
    let emissive = unpack_unorm8_from_u32((mre_packed >> 16u) & 0xFFu);

    // Get albedo
    var albedo = material_color.rgb;
    //FS_COLOR
    //FS_UV

    // Sample MRE texture (defaults to material uniforms if not bound)
    var mre = vec3<f32>(metallic, roughness, emissive);
    //FS_MRE

    // View direction - from surface to camera (for specular calculations)
    // Camera position extracted from view matrix in vertex shader
    let view_dir = normalize(in.camera_position - in.world_position);

    // DIFFUSE IRRADIANCE (ambient): Sample sky based on surface normal
    // This approximates hemispherical irradiance - varies with surface orientation
    // Top faces receive zenith color, sides/bottom receive horizon color
    // Only affects non-metals (metals have zero diffuse)
    let ambient_color = sample_sky(in.world_normal, sky);
    let ambient = ambient_color * albedo * (1.0 - mre.r);  // Zero for full metal
    let glow = albedo * mre.b;  // Emissive

    // Start with ambient + emissive (always visible)
    var final_color = ambient + glow;

    // Add sun as directional light
    // User controls brightness via sun_color (255,255,255 = full sun, 10,10,10 = dim moon)
    final_color += pbr_lite(
        in.world_normal,
        view_dir,
        sky.sun_direction,  // Sun direction (already negated in pbr_lite)
        albedo,
        mre.r,
        mre.g,
        sky.sun_color  // User controls intensity via color brightness
    );

    // Add contribution from each enabled dynamic light
    for (var i = 0u; i < 4u; i++) {
        let packed_light = shading.lights[i];
        let light = unpack_light(packed_light);

        if (light.enabled) {
            let light_color = light.color * light.intensity;

            // Add direct lighting
            final_color += pbr_lite(
                in.world_normal,
                view_dir,
                light.direction,
                albedo,
                mre.r,  // metallic
                mre.g,  // roughness
                light_color
            );
        }
    }

    // SPECULAR IBL (environment reflection): Directional sky sampling + matcap
    // This is the IBL specular substitute - samples sky gradient in reflection direction
    // Dominant term for metals, weak for non-metals
    let reflection_dir = reflect(-view_dir, in.world_normal);
    let env_matcap_uv = compute_matcap_uv(in.view_normal);
    let env_matcap = textureSample(slot2, tex_sampler, env_matcap_uv).rgb;
    // Reflection strength = metallic * (1 - roughness)
    // Full metal + shiny = 1.0, full metal + rough = 0.0, non-metal = 0.0
    let reflection_strength = mre.r * (1.0 - mre.g);
    let env_reflection = sample_sky(reflection_dir, sky) * env_matcap * reflection_strength;

    final_color += env_reflection;

    return vec4<f32>(final_color, material_color.a);
}
