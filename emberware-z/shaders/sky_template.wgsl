// Sky rendering shader template
// Prepended with common.wgsl by build.rs
// Renders a fullscreen gradient with sun disc using procedural sky data

// ============================================================================
// Vertex and Fragment Shaders
// ============================================================================

struct SkyVertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_ray: vec3<f32>,
    @location(1) @interpolate(flat) shading_state_index: u32,
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

    // Reconstruct view ray from NDC position
    // Assume FOV ~60-90 degrees, aspect ratio from screen
    // For a fullscreen quad, we map NDC [-1,1] to view space
    // This is a simplified approach that works well for typical FOVs
    // tan(60/2) = 0.577, tan(90/2) = 1.0
    // We'll use 1.0 as a reasonable default (vertical FOV ~90)
    let tan_half_fov = 1.0;

    // Note: We don't have easy access to aspect ratio here, but for sky rendering
    // slight distortion at extreme screen edges is acceptable.
    // For perfect accuracy, we'd need to pass aspect as a uniform.
    // Using 1.0 for both X and Y gives square pixels, close enough for sky.
    let view_ray_x = x * tan_half_fov;
    let view_ray_y = y * tan_half_fov;

    // Create normalized view ray in camera space, then transform to world space
    // In camera space, -Z is forward, so view_ray_cam.z = -1 for looking into the scene.
    // Use cam_back (not cam_forward) because: cam_back * (-1) = -cam_back = cam_forward
    let view_ray_cam = normalize(vec3<f32>(view_ray_x, view_ray_y, -1.0));
    out.view_ray = cam_right * view_ray_cam.x + cam_up * view_ray_cam.y + cam_back * view_ray_cam.z;

    return out;
}

@fragment
fn fs(in: SkyVertexOut) -> @location(0) vec4<f32> {
    // Get sky data from shading state
    let shading = shading_states[in.shading_state_index];
    let sky_data = unpack_sky(shading.sky);

    // Compute sky color for this view direction
    let direction = normalize(in.view_ray);
    let sky_color = sample_sky(direction, sky_data);

    return vec4<f32>(sky_color, 1.0);
}
