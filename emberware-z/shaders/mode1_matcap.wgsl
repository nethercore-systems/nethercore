// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

// NOTE: Vertex shader (VertexIn/VertexOut structs and @vertex fn) is injected by shader_gen.rs from common.wgsl
// NOTE: Common bindings, structures, and utilities are injected by shader_gen.rs from common.wgsl

// ============================================================================
// Fragment Shader Utilities
// ============================================================================

// Compute matcap UV with perspective correction
// This prevents distortion at screen edges with wide FOV
// Note: In view space, Z is negative for objects in front of camera (looking down -Z)
fn compute_matcap_uv(view_position: vec3<f32>, view_normal: vec3<f32>) -> vec2<f32> {
    let depth = -view_position.z;  // Convert to positive depth
    let inv_depth = 1.0 / (1.0 + depth);
    let proj_factor = -view_position.x * view_position.y * inv_depth;
    let basis1 = vec3<f32>(1.0 - view_position.x * view_position.x * inv_depth, proj_factor, -view_position.x);
    let basis2 = vec3<f32>(proj_factor, 1.0 - view_position.y * view_position.y * inv_depth, -view_position.y);
    let matcap_uv = vec2<f32>(dot(basis1, view_normal), dot(basis2, view_normal));

    return matcap_uv * vec2<f32>(0.5, -0.5) + 0.5;
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

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    // Get shading state for this draw
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);

    // Unpack matcap blend modes from uniform_set_1 (Mode 1 uses this for blend modes)
    let blend_modes_packed = shading.uniform_set_1;
    let blend_mode_0 = (blend_modes_packed) & 0xFFu;
    let blend_mode_1 = (blend_modes_packed >> 8u) & 0xFFu;
    let blend_mode_2 = (blend_modes_packed >> 16u) & 0xFFu;
    let blend_mode_3 = (blend_modes_packed >> 24u) & 0xFFu;

    // Start with material color
    var color = material_color.rgb;

    //FS_COLOR
    //FS_UV

    // Compute matcap UV once for all matcaps (perspective-correct)
    let matcap_uv = compute_matcap_uv(in.view_position, in.view_normal);

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
