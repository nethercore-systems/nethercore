fn legacy_portal_circle(uv: vec2f, size: f32) -> f32 {
    return length(uv) - size;
}
fn legacy_portal(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant (domain_id is fixed to TANGENT_LOCAL for PORTAL)
    let variant_id = instr_variant_id(instr);

    // Decode portal center direction
    let center = decode_dir16(instr_dir16(instr));

    // Compute dot product with portal center
    let d = dot(dir, center);

    // Reject if behind the portal center (d <= 0)
    if d <= 0.0 { return LayerSample(vec3f(0.0), 0.0); }

    // Project onto tangent plane: gnomonic projection (tangent-local UV).
    // Divide by `d` so UV is unbounded as d -> 0 (approaching 90° from center).
    // (This avoids "portal rotates with world axes" artifacts.)
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));
    let uv = vec2f(dot(dir, t) / d, dot(dir, b) / d);

    // Compute grazing fade to prevent edge artifacts
    let grazing_w = smoothstep(0.1, 0.3, d);

    // Extract parameters
    // param_a: Size (0..255 -> 0.05..0.8)
    let size = mix(0.05, 0.8, u8_to_01(instr_a(instr)));

    // param_b: Edge glow width (0..255 -> 0.01..0.3)
    let edge_width = mix(0.01, 0.3, u8_to_01(instr_b(instr)));

    // param_c: Edge roughness (0..255 -> 0.0..1.0)
    let roughness = u8_to_01(instr_c(instr));

    // param_d: Phase for VORTEX (0..255 -> 0..1)
    let phase = epu_loop_phase01(instr_d(instr));

    // Apply VORTEX warp if needed (before SDF evaluation)
    var warped_uv = uv;
    if variant_id == PORTAL_VARIANT_VORTEX {
        warped_uv = portal_apply_vortex_warp(uv, phase);
    }

    // Evaluate shape SDF by variant
    var sdf: f32;
    switch variant_id {
        case PORTAL_VARIANT_CIRCLE: {
            sdf = portal_sdf_circle(warped_uv, size);
        }
        case PORTAL_VARIANT_RECT: {
            sdf = portal_sdf_rect(warped_uv, size);
        }
        case PORTAL_VARIANT_TEAR: {
            sdf = portal_sdf_tear(warped_uv, size, roughness);
        }
        case PORTAL_VARIANT_VORTEX: {
            sdf = legacy_portal_circle(warped_uv, size);
        }
        case PORTAL_VARIANT_CRACK: {
            sdf = portal_sdf_crack(warped_uv, size, roughness);
        }
        case PORTAL_VARIANT_RIFT: {
            sdf = portal_sdf_rift(warped_uv, size, roughness);
        }
        default: {
            // Reserved/unknown variants: no output.
            return LayerSample(vec3f(0.0), 0.0);
        }
    }

    // Compute AA width based on projected pixel size
    // Use a fixed AA width since fwidth is not available in compute shaders
    let aa_width = 0.01;

    // Compute interior mask with smooth anti-aliasing
    let interior = smoothstep(aa_width, -aa_width, sdf);

    // Compute edge glow: visible outside the shape, fading with distance
    let edge = smoothstep(edge_width, 0.0, sdf) * (1.0 - interior);

    // Extract colors
    let color_a = instr_color_a(instr);  // Interior/void color
    let color_b = instr_color_b(instr);  // Edge glow color
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0; // 0..2 range
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // Blend colors: interior + edge glow
    let rgb = color_a * interior + color_b * edge * intensity;

    // Compute final weight
    let w = (interior * alpha_a + edge * alpha_b) * grazing_w * region_w;

    return LayerSample(rgb, w);
}
