// Mode 0: Unlit / Simple Lambert
// Supports all 16 vertex formats (0-15)
// Without normals: flat color
// With normals: simple Lambert shading using sky sun

// NOTE: Vertex shader (VertexIn/VertexOut structs and @vertex fn) is injected by shader_gen.rs from common.wgsl
// NOTE: Common bindings, structures, and utilities are injected by shader_gen.rs from common.wgsl

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let sky = unpack_sky(shading.sky);

    // Start with material color
    var color = material_color.rgb;

    //FS_COLOR
    //FS_UV

    // Ambient lighting (sampled from sky)
    //FS_AMBIENT

    // Normal-based diffuse lighting (if normals present)
    //FS_NORMAL

    return vec4<f32>(color, material_color.a);
}
