// ============================================================================
// LOBE (v2) - Region-Masked Directional Glow
// Opcode: 0x12
// Role: Radiance (additive feature layer)
//
// Creates directional glow or light spill effects (suns, lamps, neon washes,
// spotlights) centered around a direction. This is the v2 LOBE algorithm
// moved to the radiance range (from 0x02) so it can be region-masked and
// layered with other radiance effects.
//
// Packed fields (v2):
//   color_a: Core glow color (RGB24)
//   color_b: Edge/falloff color (RGB24)
//   intensity: Brightness (0..255 -> 0.0..2.0)
//   param_a: Exponent/sharpness (0..255 -> 1..64)
//   param_b: Edge falloff curve (0..255 -> 0.5..4.0)
//   param_c: Animation mode (0..255 -> 0..2 mod 3: 0=none, 1=pulse, 2=flicker)
//   param_d: Animation speed (0..255 -> 0.0..10.0)
//   direction: Lobe center direction (oct-u16)
//   alpha_a: Coverage alpha (0..15 -> 0.0..1.0)
//   alpha_b: Unused (set to 0)
//
// Meta (via meta5):
//   domain_id: Ignored
//   variant_id: Ignored
// ============================================================================

// Deterministic hash for flicker animation (1D -> 1D)
// Uses the same hash as the legacy v2 bounds lobe for identical behavior
fn lobe_v2_hash11(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453123);
}

fn eval_lobe_v2(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
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

    // Extract animation parameters
    // param_c: Animation mode (0..255 -> 0..2 mod 3)
    let mode = instr_c(instr) % 3u;
    // param_d: Animation speed (0..255 -> 0.0..10.0)
    let speed = u8_to_01(instr_d(instr)) * 10.0;

    // Apply animation based on mode
    var anim = 1.0;
    if mode == 1u && speed > 0.0 {
        // Mode 1 (pulse): anim = 0.7 + 0.3 * sin(time * speed)
        anim = 0.7 + 0.3 * sin(time * speed);
    } else if mode == 2u && speed > 0.0 {
        // Mode 2 (flicker): anim = 0.5 + 0.5 * hash(floor(time * speed))
        // Discrete steps for deterministic flicker
        anim = 0.5 + 0.5 * lobe_v2_hash11(floor(time * speed));
    }

    // Extract intensity: bits 63..56 (0..255 -> 0.0..2.0)
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0;

    // Extract alpha_a: bits 7..4 (0..15 -> 0.0..1.0)
    let alpha_a = instr_alpha_a_f32(instr);

    // Compute final weight: w = base * intensity * anim * alpha_a * region_w
    let w = base * intensity * anim * alpha_a * region_w;

    return LayerSample(rgb, w);
}
