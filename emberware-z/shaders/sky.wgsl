// Sky rendering shader
// Renders a fullscreen gradient with sun disc using procedural sky data
// Uses the existing common.wgsl utilities and bind group layouts

// ============================================================================
// Bindings (same as other shaders - reuse common bind group layouts)
// ============================================================================

// Per-frame storage buffers
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
    color_and_intensity: u32,        // RGB8 + intensity u8
}

// Unified per-draw shading state (64 bytes)
struct PackedUnifiedShadingState {
    metallic_roughness_emissive_rim: u32,  // 4x u8 packed: [metallic, roughness, emissive, rim_intensity]
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 32 bytes
}

@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// Texture bindings (required by bind group layout, but unused)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Data Unpacking Utilities (duplicated from common.wgsl for standalone shader)
// ============================================================================

fn unpack_unorm8_from_u32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

fn unpack_rgb8(packed: u32) -> vec3<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    return vec3<f32>(r, g, b);
}

fn unpack_octahedral(packed: u32) -> vec3<f32> {
    let u_i16 = i32((packed & 0xFFFFu) << 16u) >> 16;
    let v_i16 = i32(packed) >> 16;
    let u = f32(u_i16) / 32767.0;
    let v = f32(v_i16) / 32767.0;

    var dir: vec3<f32>;
    dir.x = u;
    dir.y = v;
    dir.z = 1.0 - abs(u) - abs(v);

    if (dir.z < 0.0) {
        let old_x = dir.x;
        dir.x = (1.0 - abs(dir.y)) * select(-1.0, 1.0, old_x >= 0.0);
        dir.y = (1.0 - abs(old_x)) * select(-1.0, 1.0, dir.y >= 0.0);
    }

    return normalize(dir);
}

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

fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, -sky.sun_direction));
    // Map sharpness [0,1] to power exponent [1, 200]
    // Higher sharpness = higher exponent = sharper sun disc
    let sun_power = mix(1.0, 200.0, sky.sun_sharpness);
    let sun = sky.sun_color * pow(sun_dot, sun_power);
    return gradient + sun;
}

// ============================================================================
// Vertex and Fragment Shaders
// ============================================================================

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_ray: vec3<f32>,
    @location(1) @interpolate(flat) shading_state_index: u32,
}

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    // Generate fullscreen triangle coordinates (same pattern as blit.wgsl)
    // Vertex 0: (-1, -1)
    // Vertex 1: ( 3, -1)
    // Vertex 2: (-1,  3)
    // This covers the entire screen with a single triangle
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;

    // Position at far plane (depth=1.0, behind all geometry)
    out.clip_position = vec4<f32>(x, y, 1.0, 1.0);

    // Get indices for this instance (shading_state_index passed via instance_index)
    // The draw call uses shading_state_index as instance range
    let indices = mvp_shading_indices[instance_index];
    let view_idx = indices.y;
    out.shading_state_index = indices.w;

    // Extract camera basis from view matrix
    // View matrix transforms world -> view, so its inverse (view -> world) basis vectors
    // are the columns of the inverse, which for a rotation+translation matrix are:
    // - Column 0: right vector
    // - Column 1: up vector
    // - Column 2: -forward vector (view matrix uses -Z forward)
    let view_matrix = view_matrices[view_idx];

    // Extract basis vectors (transpose of rotation part, since view matrix is world->view)
    // For view matrix: right = (m00, m10, m20), up = (m01, m11, m21), back = (m02, m12, m22)
    let cam_right = vec3<f32>(view_matrix[0].x, view_matrix[1].x, view_matrix[2].x);
    let cam_up = vec3<f32>(view_matrix[0].y, view_matrix[1].y, view_matrix[2].y);
    let cam_back = vec3<f32>(view_matrix[0].z, view_matrix[1].z, view_matrix[2].z);
    let cam_forward = -cam_back;

    // Reconstruct view ray from NDC position
    // Assume FOV ~60-90 degrees, aspect ratio from screen
    // For a fullscreen quad, we map NDC [-1,1] to view space
    // This is a simplified approach that works well for typical FOVs
    // tan(60°/2) ≈ 0.577, tan(90°/2) = 1.0
    // We'll use 1.0 as a reasonable default (vertical FOV ~90°)
    let tan_half_fov = 1.0;

    // Note: We don't have easy access to aspect ratio here, but for sky rendering
    // slight distortion at extreme screen edges is acceptable.
    // For perfect accuracy, we'd need to pass aspect as a uniform.
    // Using 1.0 for both X and Y gives square pixels, close enough for sky.
    let view_ray_x = x * tan_half_fov;
    let view_ray_y = y * tan_half_fov;

    // Create normalized view ray in camera space, then transform to world space
    let view_ray_cam = normalize(vec3<f32>(view_ray_x, view_ray_y, -1.0));
    out.view_ray = cam_right * view_ray_cam.x + cam_up * view_ray_cam.y + cam_forward * view_ray_cam.z;

    return out;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get sky data from shading state
    let shading = shading_states[in.shading_state_index];
    let sky_data = unpack_sky(shading.sky);

    // Compute sky color for this view direction
    let direction = normalize(in.view_ray);
    let sky_color = sample_sky(direction, sky_data);

    return vec4<f32>(sky_color, 1.0);
}
