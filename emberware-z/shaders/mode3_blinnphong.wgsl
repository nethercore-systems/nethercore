// Mode 3: Normalized Blinn-Phong
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Classic Blinn-Phong lighting with Gotanda energy-conserving normalization
// Reference: Gotanda 2010 - "Practical Implementation at tri-Ace"
// RSE texture in slot 1 (R=Rim intensity, G=Shininess, B=Emissive)
// Specular RGB texture in slot 2

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
// Mode 3 reinterprets fields: metallic→rim_intensity, roughness→shininess
struct PackedUnifiedShadingState {
    metallic_roughness_emissive_pad: u32,  // 4x u8 packed: [rim_intensity, shininess, emissive, pad]
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,         // Bytes 0-2 = specular_rgb, Byte 3 = rim_power (0-255 → 0-32 range)
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
@group(1) @binding(1) var slot1: texture_2d<f32>;  // RSE (Rim-Shininess-Emissive)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Specular RGB
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Unused in Mode 3
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
    @location(3) normal: vec3<f32>,  // Required for Blinn-Phong
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
// Fragment Shader - Normalized Blinn-Phong
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

// Gotanda 2010 linear approximation (Equation 12)
// Fitted for shininess 0-1000, extrapolates fine beyond
// Energy-conserving normalization factor for Blinn-Phong BRDF
fn gotanda_normalization(shininess: f32) -> f32 {
    return shininess * 0.0397436 + 0.0856832;
}

// Normalized Blinn-Phong specular lighting
// Convention: light_dir = direction rays travel (negate for lighting calculations)
// No geometry term - era-authentic, classical Blinn-Phong didn't have it
fn normalized_blinn_phong_specular(
    N: vec3<f32>,               // Surface normal (normalized)
    V: vec3<f32>,               // View direction (normalized, surface TO camera)
    light_dir: vec3<f32>,       // Light direction (direction rays travel)
    shininess: f32,             // 1-256 range (mapped from texture)
    specular_color: vec3<f32>,  // From Slot 2 RGB (or defaults to white)
    light_color: vec3<f32>,
) -> vec3<f32> {
    // Negate light_dir because it represents "direction rays travel"
    let L = -light_dir;
    let H = normalize(L + V);

    let NdotH = max(dot(N, H), 0.0);
    let NdotL = max(dot(N, L), 0.0);

    // Gotanda normalization for energy conservation
    // No geometry term - era-authentic, classical Blinn-Phong didn't have it
    let norm = gotanda_normalization(shininess);
    let spec = norm * pow(NdotH, shininess);

    return specular_color * spec * light_color * NdotL;
}

// Rim lighting (edge highlights)
// Uses sun color for coherent scene lighting
fn rim_lighting(
    N: vec3<f32>,
    V: vec3<f32>,
    rim_color: vec3<f32>,
    rim_intensity: f32,
    rim_power: f32,
) -> vec3<f32> {
    let NdotV = max(dot(N, V), 0.0);
    let rim_factor = pow(1.0 - NdotV, rim_power);
    return rim_color * rim_factor * rim_intensity;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let sky = unpack_sky(shading.sky);

    // Unpack material properties from packed u32
    // Mode 3 reinterprets: metallic→rim_intensity, roughness→shininess, emissive→emissive
    let rse_packed = shading.metallic_roughness_emissive_pad;
    let rim_intensity_uniform = unpack_unorm8_from_u32(rse_packed & 0xFFu);
    let shininess_uniform = unpack_unorm8_from_u32((rse_packed >> 8u) & 0xFFu);
    let emissive_uniform = unpack_unorm8_from_u32((rse_packed >> 16u) & 0xFFu);

    // Unpack rim_power from matcap_blend_modes byte 3 (uniform-only, no texture)
    let rim_power_raw = unpack_unorm8_from_u32(shading.matcap_blend_modes >> 24u);
    let rim_power = rim_power_raw * 32.0;  // Map 0-1 → 0-32 range

    // Get albedo
    var albedo = material_color.rgb;
    //FS_COLOR
    //FS_UV

    // Initialize with uniform defaults
    var rim_intensity = rim_intensity_uniform;
    var shininess_raw = shininess_uniform;
    var emissive = emissive_uniform;
    var specular_color = vec3<f32>(1.0, 1.0, 1.0);  // Will be multiplied by light color (defaults to white base)

    // Sample RSE texture (Rim-Shininess-Emissive) - overrides uniforms if bound
    //FS_MODE3_SLOT1

    // Sample Specular RGB texture - overrides white default if bound
    //FS_MODE3_SLOT2

    // Map shininess 0-1 → 1-256 (linear mapping)
    let shininess = mix(1.0, 256.0, shininess_raw);

    // View direction - from surface to camera (for specular calculations)
    // Camera position extracted from view matrix in vertex shader
    let view_dir = normalize(in.camera_position - in.world_position);

    // AMBIENT IRRADIANCE: Sample sky based on surface normal
    // This approximates hemispherical irradiance - varies with surface orientation
    // Top faces receive zenith color, sides/bottom receive horizon color
    let ambient_color = sample_sky(in.world_normal, sky);
    let ambient = ambient_color * albedo * 0.3;  // Scale down ambient contribution

    // EMISSIVE: Self-illumination (albedo × emissive intensity)
    let glow = albedo * emissive;

    // Start with ambient + emissive (always visible)
    var final_color = ambient + glow;

    // DIFFUSE + SPECULAR contribution from sun
    // User controls brightness via sun_color (255,255,255 = full sun, 10,10,10 = dim moon)
    let sun_L = -sky.sun_direction;  // Direction to sun (negated from rays-travel convention)
    let sun_NdotL = max(dot(in.world_normal, sun_L), 0.0);

    // Diffuse: Lambert (albedo × light_color × N·L)
    final_color += albedo * sky.sun_color * sun_NdotL;

    // Specular: Normalized Blinn-Phong
    final_color += normalized_blinn_phong_specular(
        in.world_normal,
        view_dir,
        sky.sun_direction,  // Direction rays travel (negated in function)
        shininess,
        specular_color,
        sky.sun_color
    );

    // Add contribution from each enabled dynamic light (up to 4)
    for (var i = 0u; i < 4u; i++) {
        let packed_light = shading.lights[i];
        let light = unpack_light(packed_light);

        if (light.enabled) {
            let light_color = light.color * light.intensity;
            let L = -light.direction;  // Direction to light
            let NdotL = max(dot(in.world_normal, L), 0.0);

            // Diffuse
            final_color += albedo * light_color * NdotL;

            // Specular
            final_color += normalized_blinn_phong_specular(
                in.world_normal,
                view_dir,
                light.direction,  // Direction rays travel
                shininess,
                specular_color,
                light_color
            );
        }
    }

    // RIM LIGHTING: Edge highlights using sun color
    // rim_intensity from Slot 1.R (or uniform), rim_power from uniform (matcap_blend_modes byte 0)
    let rim = rim_lighting(
        in.world_normal,
        view_dir,
        sky.sun_color,  // Use sun color for coherent scene lighting
        rim_intensity,
        rim_power
    );

    final_color += rim;

    return vec4<f32>(final_color, material_color.a);
}
