// ============================================================================
// EPU BOUNDS OPCODES
// Low-frequency environment layers: RAMP, LOBE, BAND, FOG
// EPU v2: 128-bit instructions with direct RGB24 colors
// ============================================================================

// ============================================================================
// RAMP - Enclosure Gradient
// 128-bit packed fields:
//   color_a: Sky/ceiling color (RGB24)
//   color_b: Floor/ground color (RGB24)
//   param_a: Wall/horizon color R (0..255)
//   param_b: Wall/horizon color G (0..255)
//   param_c: Wall/horizon color B (0..255)
//   param_d[7:4]: ceil_y threshold (0..15 -> -1..1)
//   param_d[3:0]: floor_y threshold (0..15 -> -1..1)
//   intensity: Softness (0..255 -> 0.01..0.5)
//   direction: Up vector (oct-u16)
// ============================================================================

fn eval_ramp(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
) -> LayerSample {
    // v2 packing matches `EpuBuilder::ramp_enclosure()`:
    // - color_a: sky/ceiling
    // - color_b: floor/ground
    // - param_a/b/c: wall/horizon
    let sky = instr_color_a(instr);
    let floor = instr_color_b(instr);

    // Wall/horizon color from param_a/b/c
    let wall = vec3f(
        u8_to_01(instr_a(instr)),
        u8_to_01(instr_b(instr)),
        u8_to_01(instr_c(instr))
    );

    let weights = compute_region_weights(dir, enc);
    let rgb = sky * weights.sky + wall * weights.wall + floor * weights.floor;

    // RAMP is a base layer: treat as fully weighted (w=1).
    return LayerSample(rgb, 1.0);
}

// ============================================================================
// LOBE - Directional Glow
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

// ============================================================================
// BAND - Horizon Ring
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

// ============================================================================
// FOG - Atmospheric Absorption
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
