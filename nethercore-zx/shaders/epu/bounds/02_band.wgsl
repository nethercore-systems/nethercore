// ============================================================================
// BAND - Horizon Ring (v2 legacy)
// 128-bit packed fields:
//   color_a: Band center color (RGB24)
//   color_b: Band edge color (RGB24) - soft edge gradient
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Width (0..255 -> 0.005..0.5)
//   param_b: Y offset (0..255 -> -0.5..0.5)
//   param_c: Edge softness (0..255 -> 0..1)
//   param_d: Scroll speed (0..255 -> 0..1)
//   direction: Band normal axis (oct-u16)
// ============================================================================

fn eval_band(
    dir: vec3f,
    instr: vec4u,
    time: f32,
) -> LayerSample {
    let n = decode_dir16(instr_dir16(instr));
    let u = dot(dir, n);

    let width = mix(0.005, 0.5, u8_to_01(instr_a(instr)));
    let offset = mix(-0.5, 0.5, u8_to_01(instr_b(instr)));
    let softness = u8_to_01(instr_c(instr));

    // Distance from band center
    let dist = abs(u - offset);
    let band = smoothstep(width, 0.0, dist);

    // Gradient from center to edge
    let edge_factor = smoothstep(0.0, width, dist * (1.0 + softness));
    let band_color = instr_color_a(instr);
    let edge_color = instr_color_b(instr);
    let rgb = mix(band_color, edge_color, edge_factor);

    // Optional azimuthal modulation for stylized motion.
    let scroll = u8_to_01(instr_d(instr)) * time;
    let phase = fract(atan2(dir.x, dir.z) / TAU + scroll);
    let modulated = band * (0.7 + 0.3 * sin(phase * 8.0));

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, modulated * intensity);
}
