// Quad Instanced Rendering Shader
// For GPU-driven billboard and sprite rendering
// Uses storage buffer lookup for per-instance data

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// NOTE: This shader uses the unified bind group layout for simplicity.
// Some bindings are unused by quads but must be present for layout compatibility.

// UNUSED - Quads position themselves, don't use model matrices
@group(0) @binding(0) var<storage, read> _unused_model_matrices: array<mat4x4<f32>>;

// USED - For billboard math (camera basis vectors)
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;

// USED - To transform world position to clip space
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

// USED - For material colors
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// UNUSED - Quads get shading_state_index from QuadInstance, not from MVP indices
@group(0) @binding(4) var<storage, read> _unused_mvp_shading_indices: array<vec2<u32>>;

// UNUSED - Quads don't use GPU skinning
@group(0) @binding(5) var<storage, read> _unused_bones: array<mat4x4<f32>, 256>;

// Quad instance data (56 bytes per instance)
struct QuadInstance {
    position: vec3<f32>,           // 12 bytes - world-space or screen-space position
    size: vec2<f32>,               // 8 bytes - width, height
    rotation: f32,                 // 4 bytes - rotation in radians (for screen-space)
    mode: u32,                     // 4 bytes - QuadMode enum value
    uv: vec4<f32>,                 // 16 bytes - texture atlas UV rect (u0, v0, u1, v1)
    color: u32,                    // 4 bytes - packed RGBA8
    shading_state_index: u32,      // 4 bytes - index into shading_states
    view_index: u32,               // 4 bytes - index into view_matrices for billboard math
}

// Per-frame storage buffer - array of quad instances
@group(0) @binding(6) var<storage, read> quad_instances: array<QuadInstance>;

// Texture bindings (group 1)
// USED - Albedo/sprite texture
@group(1) @binding(0) var slot0: texture_2d<f32>;
// UNUSED - Quads only need one texture (slots 1-3 unused)
@group(1) @binding(1) var _unused_slot1: texture_2d<f32>;
@group(1) @binding(2) var _unused_slot2: texture_2d<f32>;
@group(1) @binding(3) var _unused_slot3: texture_2d<f32>;
// USED - Texture sampler
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Quad Modes
// ============================================================================

const BILLBOARD_SPHERICAL: u32 = 0u;
const BILLBOARD_CYLINDRICAL_Y: u32 = 1u;
const BILLBOARD_CYLINDRICAL_X: u32 = 2u;
const BILLBOARD_CYLINDRICAL_Z: u32 = 3u;
const SCREEN_SPACE: u32 = 4u;
const WORLD_SPACE: u32 = 5u;

// ============================================================================
// Unpacking Helper Functions
// ============================================================================

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

// ============================================================================
// Billboard Math Functions
// ============================================================================

// Apply billboard transformation to local quad vertex
// Returns offset in world space to add to instance position
fn apply_billboard(
    local_xy: vec2<f32>,
    size: vec2<f32>,
    mode: u32,
    view_matrix: mat4x4<f32>,
) -> vec3<f32> {
    // Extract camera basis vectors from view matrix
    let cam_right = vec3<f32>(view_matrix[0].x, view_matrix[0].y, view_matrix[0].z);
    let cam_up = vec3<f32>(view_matrix[1].x, view_matrix[1].y, view_matrix[1].z);
    let cam_fwd = -vec3<f32>(view_matrix[2].x, view_matrix[2].y, view_matrix[2].z);

    // Scale local position by quad size
    let scaled_x = local_xy.x * size.x;
    let scaled_y = local_xy.y * size.y;

    // Spherical billboard (fully camera-facing)
    if (mode == BILLBOARD_SPHERICAL) {
        return cam_right * scaled_x + cam_up * scaled_y;
    }
    // Cylindrical Y-axis (rotate around Y, lock up vector)
    else if (mode == BILLBOARD_CYLINDRICAL_Y) {
        let fwd_xz = normalize(vec3<f32>(cam_fwd.x, 0.0, cam_fwd.z));
        let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), fwd_xz));
        let up = vec3<f32>(0.0, 1.0, 0.0);
        return right * scaled_x + up * scaled_y;
    }
    // Cylindrical X-axis (rotate around X, lock right vector)
    else if (mode == BILLBOARD_CYLINDRICAL_X) {
        let fwd_yz = normalize(vec3<f32>(0.0, cam_fwd.y, cam_fwd.z));
        let up = normalize(cross(fwd_yz, vec3<f32>(1.0, 0.0, 0.0)));
        let right = vec3<f32>(1.0, 0.0, 0.0);
        return right * scaled_x + up * scaled_y;
    }
    // Cylindrical Z-axis (rotate around Z, lock forward vector)
    else if (mode == BILLBOARD_CYLINDRICAL_Z) {
        let fwd_xy = normalize(vec3<f32>(cam_fwd.x, cam_fwd.y, 0.0));
        let right = normalize(cross(vec3<f32>(0.0, 0.0, 1.0), fwd_xy));
        let up = vec3<f32>(0.0, 0.0, 1.0);
        return right * scaled_x + up * scaled_y;
    }

    // Default: no billboard (shouldn't happen for billboard modes)
    return vec3<f32>(scaled_x, scaled_y, 0.0);
}

// Apply screen-space transformation with rotation
fn apply_screen_space(
    local_xy: vec2<f32>,
    size: vec2<f32>,
    rotation: f32,
) -> vec2<f32> {
    // Scale by size
    var pos = local_xy * size;

    // Apply rotation (if non-zero)
    if (rotation != 0.0) {
        let c = cos(rotation);
        let s = sin(rotation);
        let rotated_x = pos.x * c - pos.y * s;
        let rotated_y = pos.x * s + pos.y * c;
        pos = vec2<f32>(rotated_x, rotated_y);
    }

    return pos;
}

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,  // Unit quad vertex position (-0.5 to 0.5)
    @location(1) uv: vec2<f32>,        // Unit quad UVs (0 to 1)
    @location(2) color: vec3<f32>,     // Unit quad color (white)
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_idx: u32) -> VertexOut {
    var out: VertexOut;

    // Get instance data from storage buffer
    let instance = quad_instances[instance_idx];

    // Get view matrix for billboard math
    let view_matrix = view_matrices[instance.view_index];

    // Get projection matrix (use view_index as proj_index for simplicity)
    let projection_matrix = proj_matrices[instance.view_index];

    var world_pos: vec3<f32>;

    // Handle different quad modes
    if (instance.mode == SCREEN_SPACE) {
        // Screen-space sprite (2D overlay)
        let screen_offset = apply_screen_space(in.position.xy, instance.size, instance.rotation);

        // Convert screen position to NDC
        // Assume instance.position.xy is in screen pixels, instance.position.z is depth
        let ndc_x = (instance.position.x + screen_offset.x) / 640.0 * 2.0 - 1.0;  // TODO: Use actual screen size
        let ndc_y = 1.0 - (instance.position.y + screen_offset.y) / 360.0 * 2.0; // TODO: Use actual screen size
        let ndc_z = instance.position.z;  // Depth [0, 1]

        out.clip_position = vec4<f32>(ndc_x, ndc_y, ndc_z, 1.0);
        out.world_position = instance.position;  // Not actually world position for screen-space
    }
    else if (instance.mode == WORLD_SPACE) {
        // World-space quad (uses model matrix if needed)
        // For now, just render at instance.position with fixed orientation
        let scaled_pos = vec3<f32>(
            in.position.x * instance.size.x,
            in.position.y * instance.size.y,
            0.0
        );
        world_pos = instance.position + scaled_pos;
        out.world_position = world_pos;
        out.clip_position = projection_matrix * view_matrix * vec4<f32>(world_pos, 1.0);
    }
    else {
        // Billboard modes (SPHERICAL, CYLINDRICAL_Y/X/Z)
        let billboard_offset = apply_billboard(in.position.xy, instance.size, instance.mode, view_matrix);
        world_pos = instance.position + billboard_offset;
        out.world_position = world_pos;
        out.clip_position = projection_matrix * view_matrix * vec4<f32>(world_pos, 1.0);
    }

    // Map unit quad UV (0-1) to instance UV rect
    out.uv = mix(instance.uv.xy, instance.uv.zw, in.uv);

    // Blend instance color with vertex color
    out.color = unpack_rgba8(instance.color);

    // Pass shading state index to fragment shader
    out.shading_state_index = instance.shading_state_index;

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);

    // Sample texture from slot0 (albedo)
    let tex_color = textureSample(slot0, tex_sampler, in.uv);

    // Combine: texture * instance color * material color
    var color = tex_color.rgb * in.color.rgb * material_color.rgb;
    let alpha = tex_color.a * in.color.a * material_color.a;

    return vec4<f32>(color, alpha);
}
