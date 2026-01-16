// ============================================================================
// Unified Vertex Input/Output (all modes)
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
    //VIN_TANGENT
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    //VOUT_WORLD_NORMAL
    //VOUT_VIEW_NORMAL
    @location(3) @interpolate(flat) shading_state_index: u32,
    //VOUT_VIEW_POS
    //VOUT_CAMERA_POS
    //VOUT_TANGENT
    //VOUT_VIEW_TANGENT
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Unified Vertex Shader (all modes)
// ============================================================================

fn extract_camera_position(view_matrix: mat4x4<f32>) -> vec3<f32> {
    let r00 = view_matrix[0][0]; let r10 = view_matrix[0][1]; let r20 = view_matrix[0][2];
    let r01 = view_matrix[1][0]; let r11 = view_matrix[1][1]; let r21 = view_matrix[1][2];
    let r02 = view_matrix[2][0]; let r12 = view_matrix[2][1]; let r22 = view_matrix[2][2];
    let tx = view_matrix[3][0];
    let ty = view_matrix[3][1];
    let tz = view_matrix[3][2];
    return -vec3<f32>(
        r00 * tx + r10 * ty + r20 * tz,
        r01 * tx + r11 * ty + r21 * tz,
        r02 * tx + r12 * ty + r22 * tz
    );
}

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    // Extract indices first (needed by skinning code)
    let indices = mvp_shading_indices[instance_index];
    let model_idx = indices.x;
    let view_idx = indices.y;
    let proj_idx = indices.z;
    let shading_state_idx = indices.w;

    //VS_SKINNED

    // Access unified_transforms directly - indices are pre-offset by CPU
    let model_matrix = unified_transforms[model_idx];
    let view_matrix = unified_transforms[view_idx];
    let projection_matrix = unified_transforms[proj_idx];

    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    //VS_WORLD_NORMAL
    //VS_VIEW_NORMAL
    //VS_VIEW_POS
    //VS_TANGENT
    //VS_VIEW_TANGENT

    out.clip_position = projection_matrix * view_matrix * model_pos;
    out.shading_state_index = shading_state_idx;

    //VS_CAMERA_POS

    //VS_UV
    //VS_COLOR

    return out;
}
