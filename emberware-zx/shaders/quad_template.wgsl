// Quad Instanced Rendering Shader Template
// Prepended with common.wgsl bindings/utilities by build.rs

// ============================================================================
// Quad-Specific Bindings
// Uses bindings from common.wgsl: 0-4 (binding 5 is quad_instances below)
// ============================================================================

struct QuadInstance {
    position: vec4<f32>,       // world/screen position (xyz used)
    size: vec2<f32>,           // width, height
    rotation: f32,             // radians (screen-space only)
    mode_packed: u32,          // bits 0-7: QuadMode, bits 8-9: resolution_index
    uv: vec4<f32>,             // texture atlas rect (u0, v0, u1, v1)
    color: u32,                // packed RGBA8
    shading_state_index: u32,
    view_index: u32,           // Absolute index into unified_transforms
    proj_index: u32,           // Absolute index into unified_transforms
}

// Binding 5: quad_instances (only used by quad shader)
@group(0) @binding(5) var<storage, read> quad_instances: array<QuadInstance>;

// Quad modes (bits 0-7 of mode_packed)
const BILLBOARD_SPHERICAL: u32 = 0u;
const BILLBOARD_CYLINDRICAL_Y: u32 = 1u;
const BILLBOARD_CYLINDRICAL_X: u32 = 2u;
const BILLBOARD_CYLINDRICAL_Z: u32 = 3u;
const SCREEN_SPACE: u32 = 4u;
const WORLD_SPACE: u32 = 5u;

// Hardcoded resolutions (indexed by bits 8-9 of mode_packed)
// 0=360p, 1=540p, 2=720p, 3=1080p
const RESOLUTIONS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(640.0, 360.0),
    vec2<f32>(960.0, 540.0),
    vec2<f32>(1280.0, 720.0),
    vec2<f32>(1920.0, 1080.0)
);

// Extract mode and resolution from packed u32
fn unpack_mode(packed: u32) -> u32 {
    return packed & 0xFFu;
}

fn unpack_resolution_index(packed: u32) -> u32 {
    return (packed >> 8u) & 0x3u;
}

// ============================================================================
// Billboard Math
// ============================================================================

fn apply_billboard(local_xy: vec2<f32>, size: vec2<f32>, mode: u32, view_matrix: mat4x4<f32>) -> vec3<f32> {
    let cam_right = vec3<f32>(view_matrix[0].x, view_matrix[0].y, view_matrix[0].z);
    let cam_up = vec3<f32>(view_matrix[1].x, view_matrix[1].y, view_matrix[1].z);
    let cam_fwd = -vec3<f32>(view_matrix[2].x, view_matrix[2].y, view_matrix[2].z);

    let scaled_x = local_xy.x * size.x;
    let scaled_y = local_xy.y * size.y;

    if (mode == BILLBOARD_SPHERICAL) {
        return cam_right * scaled_x + cam_up * scaled_y;
    } else if (mode == BILLBOARD_CYLINDRICAL_Y) {
        let fwd_xz = normalize(vec3<f32>(cam_fwd.x, 0.0, cam_fwd.z));
        let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), fwd_xz));
        return right * scaled_x + vec3<f32>(0.0, 1.0, 0.0) * scaled_y;
    } else if (mode == BILLBOARD_CYLINDRICAL_X) {
        let fwd_yz = normalize(vec3<f32>(0.0, cam_fwd.y, cam_fwd.z));
        let up = normalize(cross(fwd_yz, vec3<f32>(1.0, 0.0, 0.0)));
        return vec3<f32>(1.0, 0.0, 0.0) * scaled_x + up * scaled_y;
    } else if (mode == BILLBOARD_CYLINDRICAL_Z) {
        let fwd_xy = normalize(vec3<f32>(cam_fwd.x, cam_fwd.y, 0.0));
        let right = normalize(cross(vec3<f32>(0.0, 0.0, 1.0), fwd_xy));
        return right * scaled_x + vec3<f32>(0.0, 0.0, 1.0) * scaled_y;
    }
    return vec3<f32>(scaled_x, scaled_y, 0.0);
}

fn apply_screen_space(local_xy: vec2<f32>, size: vec2<f32>, rotation: f32) -> vec2<f32> {
    var pos = local_xy * size;
    if (rotation != 0.0) {
        let c = cos(rotation);
        let s = sin(rotation);
        pos = vec2<f32>(pos.x * c - pos.y * s, pos.x * s + pos.y * c);
    }
    return pos;
}

// ============================================================================
// Vertex/Fragment Shaders
// ============================================================================

struct QuadVertexIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct QuadVertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
    @location(4) @interpolate(flat) mode: u32,
}

@vertex
fn vs(in: QuadVertexIn, @builtin(instance_index) instance_idx: u32) -> QuadVertexOut {
    var out: QuadVertexOut;
    let instance = quad_instances[instance_idx];
    // Access view/proj directly - indices are pre-computed by CPU
    let view_matrix = unified_transforms[instance.view_index];
    let projection_matrix = unified_transforms[instance.proj_index];

    // Unpack mode and resolution from packed field
    let mode = unpack_mode(instance.mode_packed);
    let resolution_idx = unpack_resolution_index(instance.mode_packed);
    let screen_dims = RESOLUTIONS[resolution_idx];

    var world_pos: vec3<f32>;

    if (mode == SCREEN_SPACE) {
        let screen_offset = apply_screen_space(in.position.xy, instance.size, instance.rotation);
        let ndc_x = (instance.position.x + screen_offset.x) / screen_dims.x * 2.0 - 1.0;
        let ndc_y = 1.0 - (instance.position.y + screen_offset.y) / screen_dims.y * 2.0;
        out.clip_position = vec4<f32>(ndc_x, ndc_y, instance.position.z, 1.0);
        out.world_position = instance.position.xyz;
    } else if (mode == WORLD_SPACE) {
        let scaled_pos = vec3<f32>(in.position.x * instance.size.x, in.position.y * instance.size.y, 0.0);
        world_pos = instance.position.xyz + scaled_pos;
        out.world_position = world_pos;
        out.clip_position = projection_matrix * view_matrix * vec4<f32>(world_pos, 1.0);
    } else {
        let billboard_offset = apply_billboard(in.position.xy, instance.size, mode, view_matrix);
        world_pos = instance.position.xyz + billboard_offset;
        out.world_position = world_pos;
        out.clip_position = projection_matrix * view_matrix * vec4<f32>(world_pos, 1.0);
    }

    // Flip UV.v for 3D quads (billboards, world-space) to match image file convention
    // Screen-space already has Y flipped in NDC calculation
    var vertex_uv = in.uv;
    if (mode != SCREEN_SPACE) {
        vertex_uv.y = 1.0 - vertex_uv.y;
    }
    out.uv = mix(instance.uv.xy, instance.uv.zw, vertex_uv);
    out.color = unpack_rgba8(instance.color);
    out.shading_state_index = instance.shading_state_index;
    out.mode = mode;
    return out;
}

@fragment
fn fs(in: QuadVertexOut) -> @location(0) vec4<f32> {
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let tex_color = sample_filtered(slot0, shading.flags, in.uv);
    let color = tex_color.rgb * in.color.rgb * material_color.rgb;
    let base_alpha = tex_color.a * in.color.a * material_color.a;

    // Dither transparency for all quads (including screen-space text/sprites)
    if should_discard_dither(in.clip_position.xy, shading.flags, base_alpha) {
        discard;
    }

    return vec4<f32>(color, base_alpha);
}
