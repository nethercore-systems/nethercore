// ============================================================================
// BAND (Radiance) - Region-Masked Horizon Band
// Opcode: 0x13
// Role: Radiance (additive feature layer)
//
// Creates horizon rings or bands around an axis with edge gradients and
// optional azimuthal modulation/scrolling. This opcode lives in the radiance
// range so it can be region-masked and layered with other radiance effects.
//
// Packed fields:
//   color_a: Band center color (RGB24)
//   color_b: Band edge color (RGB24)
//   intensity: Brightness (0..255 -> 0.0..1.0)
//   param_a: Band width (0..255 -> 0.005..0.5)
//   param_b: Y offset from equator (0..255 -> -0.5..0.5)
//   param_c: Edge softness (0..255 -> 0.0..1.0)
//   param_d: Scroll/modulation speed (0..255 -> 0.0..1.0)
//   direction: Band axis/normal (oct-u16)
//   alpha_a: Coverage alpha (0..15 -> 0.0..1.0)
//   alpha_b: Unused (set to 0)
//
// Meta (via meta5):
//   domain_id: Ignored
//   variant_id: Ignored
// ============================================================================

fn eval_band_radiance(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    // Early out if region weight is negligible
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Ignore meta5 (domain_id and variant_id are unused for BAND)
    _ = instr_meta5(instr);

    // 1. Decode band axis from oct-u16
    let axis = decode_dir16(instr_dir16(instr));

    // 2. Compute projection onto axis
    let u = dot(dir, axis);

    // Extract parameters
    // param_a: Band width (0..255 -> 0.005..0.5)
    let width = mix(0.005, 0.5, u8_to_01(instr_a(instr)));
    // param_b: Y offset from equator (0..255 -> -0.5..0.5)
    let offset = mix(-0.5, 0.5, u8_to_01(instr_b(instr)));
    // param_c: Edge softness (0..255 -> 0.0..1.0)
    let softness = u8_to_01(instr_c(instr));
    // param_d: Scroll/modulation speed (0..255 -> 0.0..1.0)
    let scroll_speed = u8_to_01(instr_d(instr));

    // 3. Compute distance from band center
    let dist = abs(u - offset);

    // 4. Compute band mask: smoothstep from width to 0
    let band = smoothstep(width, 0.0, dist);

    // 5. Compute edge factor for color gradient
    let edge_factor = smoothstep(0.0, width, dist * (1.0 + softness));

    // 6. Blend colors from center to edge
    let center_color = instr_color_a(instr);
    let edge_color = instr_color_b(instr);
    let rgb = mix(center_color, edge_color, edge_factor);

    // 7. Apply azimuthal modulation if scroll_speed > 0
    var modulated = band;
    if scroll_speed > 0.0 {
        // Build orthonormal basis (t, b) around axis
        // Choose a reference vector that is not parallel to axis
        let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
        let t = normalize(cross(axis, ref_vec));
        let b = cross(axis, t);

        // Compute azimuth angle
        let azimuth = atan2(dot(dir, b), dot(dir, t)) / TAU;

        // Apply scroll: phase = fract(azimuth + time * scroll_speed)
        let phase = fract(azimuth + time * scroll_speed);

        // Modulate: 0.7 + 0.3 * sin(phase * 8 * PI)
        // This creates 4 wavelengths around the band
        modulated = band * (0.7 + 0.3 * sin(phase * 8.0 * PI));
    }

    // Extract intensity (0..255 -> 0.0..1.0)
    let intensity = u8_to_01(instr_intensity(instr));

    // Extract alpha_a (0..15 -> 0.0..1.0)
    let alpha_a = instr_alpha_a_f32(instr);

    // 8. Compute final weight
    let w = modulated * intensity * alpha_a * region_w;

    return LayerSample(rgb, w);
}
