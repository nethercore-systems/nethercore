// Blit shader for scaling the offscreen render target to the window
// Simple fullscreen quad with texture sampling

// Texture bindings
@group(0) @binding(0) var render_target: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

// Vertex shader output
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Fullscreen triangle vertex shader
// Uses a single triangle that covers the entire screen (-1 to 1 in NDC)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var out: VertexOut;

    // Generate fullscreen triangle coordinates
    // Vertex 0: (-1, -1) -> UV (0, 1)
    // Vertex 1: ( 3, -1) -> UV (2, 1)
    // Vertex 2: (-1,  3) -> UV (0, -1)
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, 1.0 - (y + 1.0) * 0.5);

    return out;
}

// Fragment shader - simple texture sampling
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(render_target, tex_sampler, in.uv);
}
