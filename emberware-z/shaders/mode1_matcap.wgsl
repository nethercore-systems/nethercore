// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame storage buffers - matrix arrays
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// MVP indices storage buffer (per-draw packed indices)
@group(0) @binding(6) var<storage, read> mvp_indices: array<u32>;

// Unified shading states storage buffer (NEW)
struct PackedSky {
    horizon_color: u32,
    zenith_color: u32,
    sun_direction: vec4<i32>,
    sun_color_and_sharpness: u32,
}

struct PackedLight {
    direction: vec4<i32>,
    color_and_intensity: u32,
}

struct UnifiedShadingState {
    metallic: u32,
    roughness: u32,
    emissive: u32,
    pad0: u32,
    color_rgba8: u32,
    blend_modes: u32,
    sky: PackedSky,
    lights: array<PackedLight, 4>,
}

@group(0) @binding(7) var<storage, read> shading_states: array<UnifiedShadingState>;

// Shading state indices storage buffer (per-draw u32 indices)
@group(0) @binding(8) var<storage, read> shading_indices: array<u32>;

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo (UV-sampled)
@group(1) @binding(1) var slot1: texture_2d<f32>;  // Matcap 1 (normal-sampled)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Matcap 2 (normal-sampled)
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Matcap 3 (normal-sampled)
@group(1) @binding(4) var tex_sampler: sampler;

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
    //VOUT_UV
    //VOUT_COLOR
    @location(20) @interpolate(flat) shading_state_index: u32,
}

// ============================================================================
// Helper Functions
// ============================================================================

// Unpack MVP index from packed u32 (model: 16 bits, view: 8 bits, proj: 8 bits)
fn unpack_mvp(packed: u32) -> vec3<u32> {
    let model_idx = packed & 0xFFFFu;
    let view_idx = (packed >> 16u) & 0xFFu;
    let proj_idx = (packed >> 24u) & 0xFFu;
    return vec3<u32>(model_idx, view_idx, proj_idx);
}

fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

fn unpack_u8_to_f32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

fn unpack_snorm16(packed: vec4<i32>) -> vec3<f32> {
    return vec3<f32>(
        f32(packed.x) / 32767.0,
        f32(packed.y) / 32767.0,
        f32(packed.z) / 32767.0,
    );
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Fetch MVP index from instance buffer and unpack
    let mvp_packed = mvp_indices[instance_index];
    let mvp = unpack_mvp(mvp_packed);

    let model_matrix = model_matrices[mvp.x];
    let view_matrix = view_matrices[mvp.y];
    let projection_matrix = proj_matrices[mvp.z];

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

    //VS_UV
    //VS_COLOR

    // Pass shading state index (read from buffer after sorting)
    out.shading_state_index = shading_indices[instance_index];

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
fn sample_sky(direction: vec3<f32>, state: UnifiedShadingState) -> vec3<f32> {
    let horizon = unpack_rgba8(state.sky.horizon_color).rgb;
    let zenith = unpack_rgba8(state.sky.zenith_color).rgb;
    let sun_dir = normalize(unpack_snorm16(state.sky.sun_direction));
    let sun_color = unpack_rgba8(state.sky.sun_color_and_sharpness).rgb;
    let sun_sharpness = unpack_u8_to_f32(state.sky.sun_color_and_sharpness) * 256.0;
    
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(horizon, zenith, up_factor);
    let sun_dot = max(0.0, dot(direction, sun_dir));
    let sun = sun_color * pow(sun_dot, sun_sharpness);
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
    // Fetch shading state for this draw (NEW)
    let state = shading_states[in.shading_state_index];
    let base_color = unpack_rgba8(state.color_rgba8);
    
    // Extract blend modes from packed state
    let blend_mode1 = (state.blend_modes >> 16u) & 0xFFu;
    let blend_mode2 = (state.blend_modes >> 8u) & 0xFFu;
    let blend_mode3 = state.blend_modes & 0xFFu;
    
    // Start with base color from shading state
    var color = base_color.rgb;
    var alpha = base_color.a;

    //FS_COLOR
    //FS_UV

    // Compute matcap UV once for all matcaps
    let matcap_uv = compute_matcap_uv(in.view_normal);

    // Sample and blend matcaps from slots 1-3 using their blend modes
    // Slot 0 is for albedo (used in FS_UV), slots 1-3 are matcaps
    // Slots default to 1×1 white texture if not bound
    let matcap1 = textureSample(slot1, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap1, blend_mode1);

    let matcap2 = textureSample(slot2, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap2, blend_mode2);

    let matcap3 = textureSample(slot3, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap3, blend_mode3);

    return vec4<f32>(color, alpha);
}
