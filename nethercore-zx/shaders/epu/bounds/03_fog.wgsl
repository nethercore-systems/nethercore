// ============================================================================
// FOG - Atmospheric Absorption (v2 legacy)
// 128-bit packed fields:
//   color_a: Near fog color (RGB24) - color at near distance
//   color_b: Far fog color (RGB24) - color at far distance (gradient)
//   intensity: Density (0..255 -> 0..1)
//   param_a: Distance factor (0..255 -> 0..1) affects gradient position
//   param_b: Bias / falloff curve (0..255 -> 0.5..4.0)
//   direction: Up vector (oct-u16)
// Note: Use blend_mode = MULTIPLY for fog/absorption.
// ============================================================================

fn eval_fog(
    dir: vec3f,
    instr: vec4u,
) -> LayerSample {
    let up = decode_dir16(instr_dir16(instr));
    let vertical_bias = mix(-1.0, 1.0, u8_to_01(instr_a(instr)));

    // depth=0 near "up", depth=1 near "down" (with bias).
    let depth = 1.0 - epu_saturate(dot(dir, up) * vertical_bias + 0.5);

    let density = u8_to_01(instr_intensity(instr));
    let falloff = mix(0.5, 4.0, u8_to_01(instr_b(instr)));

    let fog_amount = 1.0 - exp(-density * pow(depth, falloff));

    // Gradient from near color to far color based on depth
    let near_color = instr_color_a(instr);
    let far_color = instr_color_b(instr);
    let rgb = mix(near_color, far_color, depth);

    return LayerSample(rgb, fog_amount);
}
