// Sky rendering shader template
// Prepended with common.wgsl by build.rs
// Renders a fullscreen procedural environment using Multi-Environment v3

// ============================================================================
// Vertex and Fragment Shaders
// ============================================================================

struct SkyVertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_pos: vec2<f32>,  // NDC x,y for fragment shader to compute view ray
    @location(1) @interpolate(flat) shading_state_index: u32,
    @location(2) @interpolate(flat) view_idx: u32,
    @location(3) @interpolate(flat) proj_idx: u32,
}

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> SkyVertexOut {
    var out: SkyVertexOut;

    // Generate fullscreen triangle coordinates (same pattern as blit.wgsl)
    // Vertex 0: (-1, -1)
    // Vertex 1: ( 3, -1)
    // Vertex 2: (-1,  3)
    // This covers the entire screen with a single triangle
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;

    // Position at far plane (depth=1.0, behind all geometry)
    out.clip_position = vec4<f32>(x, y, 1.0, 1.0);

    // Pass screen position for fragment shader to compute view ray per-pixel
    // (Interpolating view rays from triangle vertices causes errors)
    out.screen_pos = vec2<f32>(x, y);

    // Get indices for this instance
    let indices = mvp_shading_indices[instance_index];
    out.shading_state_index = indices.w;
    out.view_idx = indices.y;
    out.proj_idx = indices.z;

    return out;
}

@fragment
fn fs(in: SkyVertexOut) -> @location(0) vec4<f32> {
    // Get environment index from shading state (Multi-Environment v3)
    let shading = shading_states[in.shading_state_index];
    let env_index = shading.environment_index;

    // Compute view ray per-pixel (not interpolated from vertices)
    // view_idx and proj_idx are pre-computed absolute indices by CPU
    let view_matrix = unified_transforms[in.view_idx];
    let proj_matrix = unified_transforms[in.proj_idx];

    // Extract camera basis vectors (rows of view matrix rotation part)
    let cam_right = vec3<f32>(view_matrix[0].x, view_matrix[1].x, view_matrix[2].x);
    let cam_up = vec3<f32>(view_matrix[0].y, view_matrix[1].y, view_matrix[2].y);
    let cam_back = vec3<f32>(view_matrix[0].z, view_matrix[1].z, view_matrix[2].z);

    // Extract FOV from projection matrix, hardcode 16:9 aspect
    let tan_half_fov = 1.0 / proj_matrix[1][1];
    let aspect = 16.0 / 9.0;

    // Compute camera-space ray direction from screen position
    let view_ray_x = in.screen_pos.x * tan_half_fov * aspect;
    let view_ray_y = in.screen_pos.y * tan_half_fov;
    let view_ray_cam = normalize(vec3<f32>(view_ray_x, view_ray_y, -1.0));

    // Transform to world space
    let view_ray = cam_right * view_ray_cam.x + cam_up * view_ray_cam.y + cam_back * view_ray_cam.z;

    // Sample environment (Multi-Environment v3)
    let env_color = sample_environment(env_index, normalize(view_ray));

    return env_color;
}
