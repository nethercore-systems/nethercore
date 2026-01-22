// ============================================================================
// LOBE - Directional Glow (v2 legacy)
// 128-bit packed fields:
//   color_a: Core glow color (RGB24)
//   color_b: Edge/falloff color (RGB24) - gradient from core to edge
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Exponent/sharpness (0..255 -> 1..64)
//   param_b: Falloff curve for edge color (0..255 -> 0.5..4.0)
//   param_c: Animation mode (0=none, 1=pulse, 2=flicker)
//   param_d: Animation speed (0..255 -> 0..10)
//   direction: Lobe center direction (oct-u16)
// ============================================================================

fn epu_hash11(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453123);
}

fn eval_lobe(
    dir: vec3f,
    instr: vec4u,
    time: f32,
) -> LayerSample {
    let lobe_dir = decode_dir16(instr_dir16(instr));
    let d = epu_saturate(dot(dir, lobe_dir));

    let exp = mix(1.0, 64.0, u8_to_01(instr_a(instr)));
    let base = pow(d, exp);

    // Edge color falloff: blend from core to edge based on angle
    let falloff_curve = mix(0.5, 4.0, u8_to_01(instr_b(instr)));
    let edge_factor = pow(1.0 - d, falloff_curve);

    let core_color = instr_color_a(instr);
    let edge_color = instr_color_b(instr);
    let rgb = mix(core_color, edge_color, edge_factor);

    // Animation
    let speed = u8_to_01(instr_d(instr)) * 10.0;
    let mode = instr_c(instr);

    var anim = 1.0;
    if mode == 1u && speed > 0.0 {
        anim = 0.7 + 0.3 * sin(time * speed);
    } else if mode == 2u && speed > 0.0 {
        anim = 0.5 + 0.5 * epu_hash11(floor(time * speed));
    }

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, base * intensity * anim);
}
