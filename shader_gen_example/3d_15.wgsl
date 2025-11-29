const EPSILON: f32 = 1e-10;

struct PushConstants {
    blend_modes: array<u32, 8>,
}

var<push_constant> push_constants: PushConstants;

// PER FRAME BINIDINGS
@group(0) @binding(0)
var<storage> model_matrices: array<mat4x4<f32>>;

@group(0) @binding(1)
var<storage> view_matrices: array<mat4x4<f32>>;

@group(0) @binding(2)
var<storage> projection_matrices: array<mat4x4<f32>>;

@group(0) @binding(3)
var<storage> camera_positions: array<vec3<f32>>;

@group(0) @binding(4)
var texture_2d_sampler: sampler;

@group(0) @binding(5)
var texture_3d_sampler: sampler;

// TEXTURE BINDINGS
@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var cubemap1: texture_cube<f32>;

@group(1) @binding(2)
var cubemap2: texture_cube<f32>;

@group(1) @binding(3)
var matcap1: texture_2d<f32>;

@group(1) @binding(4)
var matcap2: texture_2d<f32>;

@group(1) @binding(5)
var matcap3: texture_2d<f32>;

@group(1) @binding(6)
var matcap4: texture_2d<f32>;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normals: vec3<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) view_normal: vec3<f32>,
};

struct InstanceInput {
    @location(5) model_index: u32,
    @location(6) view_pos_index: u32,
    @location(7) projection_index: u32,
}

@vertex
fn vs(
    model: VertexIn,
    instance: InstanceInput,
) -> VertexOut {
    var out: VertexOut;
    let model_matrix = model_matrices[instance.model_index];
    let view_matrix = view_matrices[instance.view_pos_index];
    let projection_matrix = projection_matrices[instance.projection_index];

    let view_position = view_matrix * model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = projection_matrix * view_position;

    out.color = model.color;
    out.uv = model.uv;
    out.world_normal = normalize((model_matrix * vec4<f32>(model.normals, 0.0)).xyz);
    out.view_pos = view_position.xyz;
    out.view_normal = normalize((view_matrix * model_matrix * vec4<f32>(model.normals, 0.0)).xyz);

    return out;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    var out_color = vec3<f32>(1.0);
    var i: u32 = 0;

    out_color = blend_colors(out_color, in.color.rgb, 0);
    out_color = blend_colors(out_color, textureSample(texture, texture_2d_sampler, in.uv).rgb, 1);
    out_color = blend_colors(out_color, textureSample(cubemap1, texture_3d_sampler, in.world_normal).rgb, 2);
    out_color = blend_colors(out_color, textureSample(cubemap2, texture_3d_sampler, in.world_normal).rgb, 3);
    let matcap_uv = matcap_uv(in.view_pos, in.view_normal);
    out_color = blend_colors(out_color, textureSample(matcap1, texture_3d_sampler, matcap_uv).rgb, 4);
    out_color = blend_colors(out_color, textureSample(matcap2, texture_3d_sampler, matcap_uv).rgb, 5);
    out_color = blend_colors(out_color, textureSample(matcap3, texture_3d_sampler, matcap_uv).rgb, 6);
    out_color = blend_colors(out_color, textureSample(matcap4, texture_3d_sampler, matcap_uv).rgb, 7);

    return vec4<f32>(out_color, 1.0);
}

fn matcap_uv(view_position: vec3<f32>, view_normal: vec3<f32>) -> vec2<f32> {
  let inv_depth = 1.0 / (1.0 + view_position.z);
  let proj_factor = -view_position.x * view_position.y * inv_depth;
  let basis1 = vec3(1.0 - view_position.x * view_position.x * inv_depth, proj_factor, -view_position.x);
  let basis2 = vec3(proj_factor, 1.0 - view_position.y * view_position.y * inv_depth, -view_position.y);
  let matcap_uv = vec2(dot(basis1, view_normal), dot(basis2, view_normal));

  return matcap_uv * vec2(0.5, -0.5) + 0.5;
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let r = rgb.x;
    let g = rgb.y;
    let b = rgb.z;

    let K = vec4<f32>(0.0, -1.0/3.0, 2.0/3.0, -1.0);

    // Compute component differences
    let p = mix(vec4<f32>(b, g, K.w, K.z), vec4<f32>(g, b, K.x, K.y), step(b, g));
    let q = mix(vec4<f32>(p.x, p.y, p.w, r), vec4<f32>(r, p.y, p.z, p.x), step(p.x, r));

    let d = q.x - min(q.w, q.y);
    let h = abs(q.z + (q.w - q.y) / (6.0 * d + EPSILON));
    let s = d / (q.x + EPSILON);
    let v = q.x;

    return vec3<f32>(h, s, v);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = fract(hsv.x) * 6.0; // hue scaled to [0,6)
    let s = hsv.y;
    let v = hsv.z;

    // Offset for R,G,B channels
    let k = vec3<f32>(0.0, 4.0, 2.0) + h;

    // Fractional component per channel
    let f = fract(k / 6.0);

    // Triangle wave per channel, fully branchless and accurate

    let rgb = v - v * s * abs(clamp(f * 6.0 - 3.0, vec3<f32>(-1.0), vec3<f32>(1.0)));
    return rgb;
}

// Blend Order
// 0. White
// 1. Vertex Color
// 2. Texture Color
// 3. Cube Map 1
// 4. Cube Map 2
// 5. Matcap 1
// 6. Matcap 2
// 7. Matcap 3
// 8. Matcap 4

// Blend Modes
// 0: None
// 1: Replace
// 2: Add
// 3: Multiply
// 4: Subtract
// 5: Divide
// 6: HSV Modulate
// 7: ??? Some other cool color modulate?
fn blend_colors(base: vec3<f32>, layer: vec3<f32>, index: u32) -> vec3<f32> {
    let mode = push_constants.blend_modes[index];
    switch(mode) {
        case 0u: { // None
            return base;
        }
        case 1u: { // Replace
            return layer;
        }
        case 2u: { // Add
            return base + layer;
        }
        case 3u: { // Multiply
            return base * layer;
        }
        case 4u: { // Subtract
            return base - layer;
        }
        case 5u: { // Divide
            return base / max(layer, vec3<f32>(EPSILON)); // avoid divide by zero
        }
        case 6u: { // HSV modulate
            // Convert base to HSV, modulate by layer, then back
            let base_hsv = rgb_to_hsv(base);
            let layer_hsv = rgb_to_hsv(layer);
            let hsv = vec3<f32>(
                fract(base_hsv.x + layer_hsv.x), // hue
                base_hsv.y * layer_hsv.y, // saturation
                base_hsv.z * layer_hsv.z  // value
            );
            return hsv_to_rgb(hsv);
        }
        case 7u: { // TODO
            return base;
        }
        default: {
            return base;
        }
    }
}
