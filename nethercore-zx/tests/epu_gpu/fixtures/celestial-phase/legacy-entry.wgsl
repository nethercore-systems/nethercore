fn legacy_phase_entry(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant (domain_id is ignored for CELESTIAL)
    let variant_id = instr_variant_id(instr);

    // Decode body direction
    let body_dir = decode_dir16(instr_dir16(instr));

    // Compute angular distance from body center
    let body_dot = clamp(dot(dir, body_dir), -1.0, 1.0);
    let angle = acos(body_dot);

    // Extract parameters
    // param_a: Angular size (0..255 -> 0.5..45 degrees)
    let angular_size_deg = mix(0.5, 45.0, u8_to_01(instr_a(instr)));
    let angular_size_rad = angular_size_deg * PI / 180.0;

    // param_b: Limb darkening exponent (0..255 -> 0.5..4.0)
    let limb_exp = mix(0.5, 4.0, u8_to_01(instr_b(instr)));

    // param_c: Phase angle (0..255 -> 0..360 degrees)
    let phase_deg = u8_to_01(instr_c(instr)) * 360.0;
    let phase_rad = phase_deg * PI / 180.0;

    // param_d: Variant-specific parameter
    let param_d_raw = u8_to_01(instr_d(instr));

    // Convert angle to radii (normalized disk distance)
    let r = angle / angular_size_rad;

    // Compute limb darkening: pow(1 - r, exponent) for r < 1
    let limb = pow(epu_saturate(1.0 - r), limb_exp);

    // Compute phase illumination (for MOON/PLANET)
    // Phase: 0 = full, 90 = half, 180 = new
    // Sun direction rotated from body_dir by phase angle
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let phase_axis = normalize(cross(hint, body_dir));
    let sun_dir = normalize(body_dir * cos(phase_rad) + cross(phase_axis, body_dir) * sin(phase_rad));

    // Local surface normal approximation (points outward from disk center)
    let surface_normal = normalize(dir - body_dir * body_dot + body_dir * 0.3);
    let phase_factor = epu_saturate(dot(surface_normal, sun_dir) * 0.5 + 0.5);

    // Compute surface UV for detail texturing
    let surface_uv = celestial_surface_uv(dir, body_dir, sin(angular_size_rad));

    // Extract colors
    let color_a = instr_color_a(instr);  // Body surface color
    let color_b = instr_color_b(instr);  // Atmosphere/corona/ring color
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0; // 0..2 range
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // Evaluate variant-specific rendering
    var sample = LayerSample(vec3f(0.0), 0.0);

    switch variant_id {
        case CELESTIAL_VARIANT_MOON: {
            sample = eval_celestial_moon(r, surface_uv, phase_factor, limb, color_a, color_b, alpha_b);
        }
        case CELESTIAL_VARIANT_SUN: {
            // param_d: Corona extent (0..255 -> 1.0..3.0)
            let corona_extent = mix(1.0, 3.0, param_d_raw);
            sample = eval_celestial_sun(r, limb, color_a, color_b, corona_extent, alpha_b);
        }
        case CELESTIAL_VARIANT_PLANET: {
            // param_d: Cloud band count (0..255 -> 0..8)
            let band_count = param_d_raw * 8.0;
            sample = eval_celestial_planet(r, surface_uv, phase_factor, limb, color_a, color_b, band_count, alpha_b);
        }
        case CELESTIAL_VARIANT_GAS_GIANT: {
            // param_d: Horizontal band count (0..255 -> 2..16)
            let band_count = mix(2.0, 16.0, param_d_raw);
            sample = eval_celestial_gas_giant(r, surface_uv, limb, color_a, color_b, band_count, alpha_b);
        }
        case CELESTIAL_VARIANT_RINGED: {
            // param_d: Ring tilt (0..255 -> 0..90 degrees)
            let ring_tilt = param_d_raw * 90.0;
            sample = eval_celestial_ringed(dir, body_dir, r, angular_size_rad, limb, color_a, color_b, ring_tilt, alpha_b);
        }
        case CELESTIAL_VARIANT_BINARY: {
            // param_d: Secondary size ratio (0..255 -> 0.2..2.0)
            let size_ratio = mix(0.2, 2.0, param_d_raw);
            sample = eval_celestial_binary(dir, body_dir, angular_size_rad, limb_exp, color_a, color_b, size_ratio, alpha_b);
        }
        case CELESTIAL_VARIANT_ECLIPSE: {
            // param_d: Corona brightness (0..255 -> 1.0..2.5)
            let corona_brightness = mix(1.0, 2.5, param_d_raw);
            sample = eval_celestial_eclipse(r, color_a, color_b, corona_brightness, alpha_b);
        }
        default: {
            // Reserved/unknown variants: no output.
            return LayerSample(vec3f(0.0), 0.0);
        }
    }

    // Apply intensity and region weight
    let final_w = sample.w * intensity * alpha_a * region_w;

    // Shared blending applies weight to RGB; intensity belongs here only once.
    return LayerSample(sample.rgb, final_w);
}
