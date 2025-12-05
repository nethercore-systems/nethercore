// Mode 0: Unlit / Simple Lambert
// Supports all 16 vertex formats (0-15)
// Without normals: flat color
// With normals: simple Lambert shading using sky sun

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
struct PackedUnifiedShadingState {
    metallic_roughness_emissive_pad: u32,  // 4x u8 packed: [metallic, roughness, emissive, pad]
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,
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
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
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

// ============================================================================
// Vertex Input/Output
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) @interpolate(flat) shading_state_index: u32,
    //VOUT_UV
    //VOUT_COLOR
    //VOUT_NORMAL
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

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * model_pos;

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;

    //VS_UV
    //VS_COLOR
    //VS_NORMAL

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Simple Lambert shading using sky sun (when normals available)
// Convention: light direction = direction rays travel (negate for dot product)
fn sky_lambert(normal: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let n_dot_l = max(0.0, dot(normal, -sky.sun_direction));
    let direct = sky.sun_color * n_dot_l;
    let ambient = sample_sky(normal, sky) * 0.3;
    return direct + ambient;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    // Negate sun_direction: it's direction rays travel, not direction to sun
    let sun_dot = max(0.0, dot(direction, -sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}

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
    //FS_NORMAL

    return vec4<f32>(color, material_color.a);
}
