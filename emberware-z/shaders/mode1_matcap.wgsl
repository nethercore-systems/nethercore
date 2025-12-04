// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

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
    sun_direction: vec4<i16>,        // snorm16x4 (w unused)
    sun_color_and_sharpness: u32,    // RGB8 + sharpness u8
}

// Packed light data (16 bytes)
struct PackedLight {
    direction: vec4<i16>,            // snorm16x4 (w = enabled flag: 0x7FFF if enabled, 0 if disabled)
    color_and_intensity: u32,        // RGB8 + intensity u8
}

// Unified per-draw shading state (96 bytes)
struct PackedUnifiedShadingState {
    metallic: u8,
    roughness: u8,
    emissive: u8,
    pad0_byte: u8,
    color_rgba8: u32,
    blend_mode: u32,
    matcap_blend_modes: u32,
    pad1: u32,
    sky: PackedSky,                  // 16 bytes
    lights: array<PackedLight, 4>,   // 64 bytes
}

// Per-frame storage buffer - array of shading states
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// Per-frame storage buffer - packed MVP + shading indices (model: 16 bits, view: 8 bits, proj: 8 bits, shading_state_index: 32 bits)
// Each entry is 2 × u32: [packed_mvp, shading_state_index]
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo (UV-sampled)
@group(1) @binding(1) var slot1: texture_2d<f32>;  // Matcap 1 (normal-sampled)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Matcap 2 (normal-sampled)
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Matcap 3 (normal-sampled)
@group(1) @binding(4) var tex_sampler: sampler;

// ============================================================================
// Unpacking Helper Functions
// ============================================================================

// Unpack u8 to f32 [0.0, 1.0]
fn unpack_unorm8(packed: u8) -> f32 {
    return f32(packed) / 255.0;
}

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

// Unpack snorm16 to f32 [-1.0, 1.0]
fn unpack_snorm16(packed: i16) -> f32 {
    return f32(packed) / 32767.0;
}

// Unpack vec4<i16> direction to vec3<f32>
fn unpack_direction(packed: vec4<i16>) -> vec3<f32> {
    return vec3<f32>(
        unpack_snorm16(packed.x),
        unpack_snorm16(packed.y),
        unpack_snorm16(packed.z)
    );
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
    sky.sun_direction = unpack_direction(packed.sun_direction);
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
    @location(3) normal: vec3<f32>,  // Required for matcaps
    //VIN_SKINNED
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) view_normal: vec3<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get packed MVP indices and shading state index from storage buffer
    let mvp_data = mvp_shading_indices[instance_index];
    let mvp_packed = mvp_data.x;
    let shading_state_idx = mvp_data.y;

    // Unpack MVP indices
    let model_idx = mvp_packed & 0xFFFFu;           // Lower 16 bits
    let view_idx = (mvp_packed >> 16u) & 0xFFu;     // Next 8 bits
    let proj_idx = (mvp_packed >> 24u) & 0xFFu;     // Top 8 bits

    // Get matrices from storage buffers using unpacked indices
    let model_matrix = model_matrices[model_idx];
    let view_matrix = view_matrices[view_idx];
    let projection_matrix = proj_matrices[proj_idx];

    // Apply model transform
    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    // Transform normal to world space (using model matrix for orthogonal transforms)
    let model_normal = (model_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.world_normal = normalize(model_normal);

    // Transform normal to view space for matcap UV computation
    let view_normal = (view_matrix * vec4<f32>(out.world_normal, 0.0)).xyz;
    out.view_normal = normalize(view_normal);

    // View-projection transform
    out.clip_position = projection_matrix * view_matrix * model_pos;

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;

    //VS_UV
    //VS_COLOR

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
}

// Sample procedural sky
fn sample_sky(direction: vec3<f32>, sky: SkyData) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}

// Convert RGB to HSV
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(max(rgb.r, rgb.g), rgb.b);
    let cmin = min(min(rgb.r, rgb.g), rgb.b);
    let delta = cmax - cmin;

    var h: f32 = 0.0;
    var s: f32 = 0.0;
    let v: f32 = cmax;

    if (delta > 0.00001) {
        s = delta / cmax;

        if (rgb.r >= cmax) {
            h = (rgb.g - rgb.b) / delta;
        } else if (rgb.g >= cmax) {
            h = 2.0 + (rgb.b - rgb.r) / delta;
        } else {
            h = 4.0 + (rgb.r - rgb.g) / delta;
        }

        h = h / 6.0;
        if (h < 0.0) {
            h = h + 1.0;
        }
    }

    return vec3<f32>(h, s, v);
}

// Convert HSV to RGB
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x * 6.0;
    let s = hsv.y;
    let v = hsv.z;

    let c = v * s;
    let x = c * (1.0 - abs((h % 2.0) - 1.0));
    let m = v - c;

    var rgb: vec3<f32>;

    if (h < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + vec3<f32>(m, m, m);
}

// Blend two colors based on mode
// mode: 0=Multiply, 1=Add, 2=HSV Modulate
fn blend_colors(base: vec3<f32>, blend: vec3<f32>, mode: u32) -> vec3<f32> {
    switch (mode) {
        case 0u: {
            // Multiply (default matcap behavior)
            return base * blend;
        }
        case 1u: {
            // Add (for glow/emission effects)
            return base + blend;
        }
        case 2u: {
            // HSV Modulate (hue shifting, iridescence)
            let base_hsv = rgb_to_hsv(base);
            let blend_hsv = rgb_to_hsv(blend);
            // Modulate hue and saturation, multiply value
            let result_hsv = vec3<f32>(
                fract(base_hsv.x + blend_hsv.x),  // Add hues (wrapping)
                base_hsv.y * blend_hsv.y,          // Multiply saturation
                base_hsv.z * blend_hsv.z           // Multiply value
            );
            return hsv_to_rgb(result_hsv);
        }
        default: {
            // Fallback to multiply
            return base * blend;
        }
    }
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);

    // Unpack matcap blend modes
    let blend_modes_packed = shading.matcap_blend_modes;
    let blend_mode_0 = (blend_modes_packed) & 0xFFu;
    let blend_mode_1 = (blend_modes_packed >> 8u) & 0xFFu;
    let blend_mode_2 = (blend_modes_packed >> 16u) & 0xFFu;
    let blend_mode_3 = (blend_modes_packed >> 24u) & 0xFFu;

    // Start with material color
    var color = material_color.rgb;

    //FS_COLOR
    //FS_UV

    // Compute matcap UV once for all matcaps
    let matcap_uv = compute_matcap_uv(in.view_normal);

    // Sample and blend matcaps from slots 1-3 using their blend modes
    // Slot 0 is for albedo (used in FS_UV), slots 1-3 are matcaps
    // Slots default to 1×1 white texture if not bound
    let matcap1 = textureSample(slot1, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap1, blend_mode_1);

    let matcap2 = textureSample(slot2, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap2, blend_mode_2);

    let matcap3 = textureSample(slot3, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap3, blend_mode_3);

    return vec4<f32>(color, material_color.a);
}
