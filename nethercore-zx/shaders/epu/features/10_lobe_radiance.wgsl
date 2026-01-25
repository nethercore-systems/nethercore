// ============================================================================
// LOBE (Radiance) - Region-Masked Directional Glow
// Opcode: 0x12
// Role: Radiance (additive feature layer)
//
// Creates directional glow or light spill effects (suns, lamps, neon washes,
// spotlights) centered around a direction. This opcode lives in the radiance
// range so it can be region-masked and layered with other radiance effects.
//
// Packed fields:
//   color_a: Core glow color (RGB24)
//   color_b: Edge/falloff color (RGB24)
//   intensity: Brightness (0..255 -> 0.0..2.0)
//   param_a: Exponent/sharpness (0..255 -> 1..64)
//   param_b: Edge falloff curve (0..255 -> 0.5..4.0)
//   param_c: Phase waveform (0=off, 1=sine, 2=triangle, 3=strobe)
//   param_d: Phase (0..255 -> 0..1)
//   direction: Lobe center direction (oct-u16)
//   alpha_a: Coverage alpha (0..15 -> 0.0..1.0)
//   alpha_b: Unused (set to 0)
//
// Meta (via meta5):
//   domain_id: Ignored
//   variant_id: Ignored
// ============================================================================

fn eval_lobe_radiance(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    // Early out if region weight is negligible
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Ignore meta5 (domain_id and variant_id are unused for LOBE)
    _ = instr_meta5(instr);

    // Decode lobe center direction from oct-u16
    let lobe_dir = decode_dir16(instr_dir16(instr));

    // Compute alignment: d = saturate(dot(dir, lobe_dir))
    let d = epu_saturate(dot(dir, lobe_dir));

    // Extract exponent/sharpness: param_a (0..255 -> 1..64)
    let exponent = mix(1.0, 64.0, u8_to_01(instr_a(instr)));

    // Compute base intensity: base = pow(d, exponent)
    let base = pow(d, exponent);

    // Extract edge falloff curve: param_b (0..255 -> 0.5..4.0)
    let falloff_curve = mix(0.5, 4.0, u8_to_01(instr_b(instr)));

    // Compute edge factor: edge_factor = pow(1.0 - d, falloff_curve)
    let edge_factor = pow(1.0 - d, falloff_curve);

    // Extract colors
    let core_color = instr_color_a(instr);
    let edge_color = instr_color_b(instr);

    // Blend colors: rgb = mix(color_a, color_b, edge_factor)
    let rgb = mix(core_color, edge_color, edge_factor);

    let waveform = instr_c(instr);
    let phase = u8_to_01(instr_d(instr));
    var anim = 1.0;
    switch waveform {
        case 0u: { anim = 1.0; }
        case 1u: { anim = 0.5 + 0.5 * sin(phase * TAU); }
        case 2u: { anim = 1.0 - abs(phase * 2.0 - 1.0); }
        case 3u: { anim = step(0.5, fract(phase * 4.0)); }
        default: { anim = 0.5 + 0.5 * sin(phase * TAU); }
    }

    // Extract intensity: bits 63..56 (0..255 -> 0.0..2.0)
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0;

    // Extract alpha_a: bits 7..4 (0..15 -> 0.0..1.0)
    let alpha_a = instr_alpha_a_f32(instr);

    // Compute final weight: w = base * intensity * anim * alpha_a * region_w
    let w = base * intensity * anim * alpha_a * region_w;

    return LayerSample(rgb, w);
}
