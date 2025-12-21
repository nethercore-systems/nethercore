// Mode 1: Matcap
// Requires NORMAL flag - only 8 permutations (formats 4-7 and 12-15)
// Matcaps in slots 1-3 multiply together

// NOTE: Vertex shader (VertexIn/VertexOut structs and @vertex fn) is injected by shader_gen.rs from common.wgsl
// NOTE: Common bindings, structures, and utilities are injected by shader_gen.rs from common.wgsl

// ============================================================================
// Fragment Shader Utilities
// ============================================================================

// Compute matcap UV with perspective correction using orthonormal basis construction
// This prevents distortion at screen edges with wide FOV
fn compute_matcap_uv(view_position: vec3<f32>, view_normal: vec3<f32>) -> vec2<f32> {
    let n = normalize(view_normal);
    // Direction from surface TO camera (negate because view_position.z is negative in front)
    // The formula requires positive Z for objects in front of camera
    let I = normalize(-view_position);

    // Orthonormal basis construction
    let a = 1.0 / (1.0 + I.z);
    let b = -I.x * I.y * a;
    let b1 = vec3<f32>(1.0 - I.x * I.x * a, b, -I.x);
    let b2 = vec3<f32>(b, 1.0 - I.y * I.y * a, -I.y);

    let matcap_uv = vec2<f32>(dot(b1, n), dot(b2, n));

    // Negate Y for wgpu texture coords (Y=0 at top), clamp to avoid edge artifacts
    let uv = vec2<f32>(matcap_uv.x * 0.5 + 0.5, -matcap_uv.y * 0.5 + 0.5);
    return clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
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

    // Unpack matcap blend modes from uniform_set_0 (Mode 1 uses this for blend modes)
    let blend_modes_packed = shading.uniform_set_0;
    let blend_mode_0 = (blend_modes_packed) & 0xFFu;
    let blend_mode_1 = (blend_modes_packed >> 8u) & 0xFFu;
    let blend_mode_2 = (blend_modes_packed >> 16u) & 0xFFu;
    let blend_mode_3 = (blend_modes_packed >> 24u) & 0xFFu;

    // Start with material color, base_alpha defaults to material alpha
    var color = material_color.rgb;
    var base_alpha = material_color.a;

    //FS_COLOR
    //FS_UV

    // Compute matcap UV once for all matcaps (perspective-correct)
    let matcap_uv = compute_matcap_uv(in.view_position, in.view_normal);

    // Check use_matcap_reflection flag:
    // 0 = use procedural sky for reflection (default)
    // 1 = use matcap textures for stylized reflection
    if has_flag(shading.flags, FLAG_USE_MATCAP_REFLECTION) {
        // Sample and blend matcaps from slots 1-3 using their blend modes
        // Slot 0 is for albedo (used in FS_UV), slots 1-3 are matcaps
        // Slots default to 1×1 white texture if not bound
        let matcap1 = sample_filtered(slot1, shading.flags, matcap_uv).rgb;
        color = blend_colors(color, matcap1, blend_mode_1);

        let matcap2 = sample_filtered(slot2, shading.flags, matcap_uv).rgb;
        color = blend_colors(color, matcap2, blend_mode_2);

        let matcap3 = sample_filtered(slot3, shading.flags, matcap_uv).rgb;
        color = blend_colors(color, matcap3, blend_mode_3);
    } else {
        // Use procedural environment for reflection instead of matcaps
        // Sample 4-color environment gradient in world normal direction
        let N = normalize(in.world_normal);
        let env_color = sample_environment_ambient(shading.environment_index, N);
        // Apply using first blend mode for consistency
        color = blend_colors(color, env_color, blend_mode_1);
    }

    // Dither transparency (two-layer: base_alpha × effect_alpha)
    if should_discard_dither(in.clip_position.xy, shading.flags, base_alpha) {
        discard;
    }

    return vec4<f32>(color, base_alpha);
}
