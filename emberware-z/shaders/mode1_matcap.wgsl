// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

// ============================================================================
// Uniforms and Bindings
// ============================================================================

// Per-frame storage buffer - all model matrices for the frame
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;

// Per-frame uniforms (group 0)
@group(0) @binding(1) var<uniform> view_matrix: mat4x4<f32>;
@group(0) @binding(2) var<uniform> projection_matrix: mat4x4<f32>;

// Sky uniforms (for ambient)
struct SkyUniforms {
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}

@group(0) @binding(3) var<uniform> sky: SkyUniforms;

// Material uniforms
struct MaterialUniforms {
    color: vec4<f32>,              // RGBA tint color
    matcap_blend_modes: vec4<u32>, // Blend modes for slots 0-3 (0=Multiply, 1=Add, 2=HSV Modulate)
}

@group(0) @binding(4) var<uniform> material: MaterialUniforms;

// Bone transforms for GPU skinning (up to 256 bones)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

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
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get model matrix from storage buffer using instance index
    let model_matrix = model_matrices[instance_index];

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
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color.xyz, sky.zenith_color.xyz, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction.xyz));
    let sun = sky.sun_color_and_sharpness.xyz * pow(sun_dot, sky.sun_color_and_sharpness.w);
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
    // Start with material color (uniform tint)
    var color = material.color.rgb;

    //FS_COLOR
    //FS_UV

    // Compute matcap UV once for all matcaps
    let matcap_uv = compute_matcap_uv(in.view_normal);

    // Sample and blend matcaps from slots 1-3 using their blend modes
    // Slot 0 is for albedo (used in FS_UV), slots 1-3 are matcaps
    // Slots default to 1×1 white texture if not bound
    let matcap1 = textureSample(slot1, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap1, material.matcap_blend_modes.y);

    let matcap2 = textureSample(slot2, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap2, material.matcap_blend_modes.z);

    let matcap3 = textureSample(slot3, tex_sampler, matcap_uv).rgb;
    color = blend_colors(color, matcap3, material.matcap_blend_modes.w);

    return vec4<f32>(color, material.color.a);
}
