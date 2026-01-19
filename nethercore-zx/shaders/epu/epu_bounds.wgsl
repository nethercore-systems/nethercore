// ============================================================================
// EPU BOUNDS OPCODES
// Low-frequency environment layers: RAMP, LOBE, BAND, FOG
// ============================================================================

// ============================================================================
// RAMP - Enclosure Gradient
// Packed fields:
//   color_index: Wall/horizon palette index
//   param_a: Sky/ceiling palette index
//   param_b: Floor/ground palette index
//   param_c[7:4]: ceil_y threshold (0..15 -> -1..1)
//   param_c[3:0]: floor_y threshold (0..15 -> -1..1)
//   intensity: Softness (0..255 -> 0.01..0.5)
//   direction: Up vector (oct-u16)
// ============================================================================

fn eval_ramp(
    dir: vec3f,
    lo: u32,
    hi: u32,
    enc: EnclosureConfig,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    let sky = palette_lookup(palette, instr_a(lo, hi));
    let wall = palette_lookup(palette, instr_color(lo, hi));
    let floor = palette_lookup(palette, instr_b(lo, hi));

    let weights = compute_region_weights(dir, enc);
    let rgb = sky * weights.sky + wall * weights.wall + floor * weights.floor;

    // RAMP is a base layer: treat as fully weighted (w=1).
    return LayerSample(rgb, 1.0);
}

// ============================================================================
// LOBE - Directional Glow
// Packed fields:
//   color_index: Glow color palette index
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Exponent/sharpness (0..255 -> 1..64)
//   param_b: Animation speed (0..255 -> 0..10)
//   param_c: Animation mode (0=none, 1=pulse, 2=flicker)
//   direction: Lobe center direction (oct-u16)
// ============================================================================

fn hash11(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453123);
}

fn eval_lobe(
    dir: vec3f,
    lo: u32,
    hi: u32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    let lobe_dir = decode_dir16(instr_dir16(lo, hi));
    let d = saturate(dot(dir, lobe_dir));

    let exp = mix(1.0, 64.0, u8_to_01(instr_a(lo, hi)));
    let base = pow(d, exp);

    let speed = u8_to_01(instr_b(lo, hi)) * 10.0;
    let mode = instr_c(lo, hi);

    var anim = 1.0;
    if mode == 1u && speed > 0.0 {
        anim = 0.7 + 0.3 * sin(time * speed);
    } else if mode == 2u && speed > 0.0 {
        anim = 0.5 + 0.5 * hash11(floor(time * speed));
    }

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, base * intensity * anim);
}

// ============================================================================
// BAND - Horizon Ring
// Packed fields:
//   color_index: Band color
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Width (0..255 -> 0.005..0.5)
//   param_b: Offset (0..255 -> -0.5..0.5)
//   param_c: Scroll speed (0..255 -> 0..1)
//   direction: Band normal axis (oct-u16)
// ============================================================================

fn eval_band(
    dir: vec3f,
    lo: u32,
    hi: u32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    let n = decode_dir16(instr_dir16(lo, hi));
    let u = dot(dir, n);

    let width = mix(0.005, 0.5, u8_to_01(instr_a(lo, hi)));
    let offset = mix(-0.5, 0.5, u8_to_01(instr_b(lo, hi)));

    let band = smoothstep(width, 0.0, abs(u - offset));

    // Optional azimuthal modulation for stylized motion.
    let scroll = u8_to_01(instr_c(lo, hi)) * time;
    let phase = fract(atan2(dir.x, dir.z) / TAU + scroll);
    let modulated = band * (0.7 + 0.3 * sin(phase * 8.0));

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, modulated * intensity);
}

// ============================================================================
// FOG - Atmospheric Absorption
// Packed fields:
//   color_index: Fog tint color (used as multiplicative tint)
//   intensity: Density (0..255 -> 0..1)
//   param_a: Vertical bias (0..255 -> -1..1)
//   param_b: Falloff curve (0..255 -> 0.5..4.0)
//   direction: Up vector (oct-u16)
// Note: Use blend_mode = MULTIPLY for fog/absorption.
// ============================================================================

fn eval_fog(
    dir: vec3f,
    lo: u32,
    hi: u32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    let up = decode_dir16(instr_dir16(lo, hi));
    let vertical_bias = mix(-1.0, 1.0, u8_to_01(instr_a(lo, hi)));

    // depth=0 near "up", depth=1 near "down" (with bias).
    let depth = 1.0 - saturate(dot(dir, up) * vertical_bias + 0.5);

    let density = u8_to_01(instr_intensity(lo, hi));
    let falloff = mix(0.5, 4.0, u8_to_01(instr_b(lo, hi)));

    let fog_amount = 1.0 - exp(-density * pow(depth, falloff));

    // For MULTIPLY blend, rgb is a tint factor.
    let rgb = palette_lookup(palette, instr_color(lo, hi));
    return LayerSample(rgb, fog_amount);
}
