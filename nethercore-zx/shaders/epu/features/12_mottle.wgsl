// @epu_meta_begin
// opcode = 0x14
// name = MOTTLE
// kind = feature
// variants = [SOFT, GRAIN, RIDGE, DAPPLE]
// domains = []
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=0.5, max=20.0, unit="x" }
// field param_b = { label="contrast", map="u8_01" }
// field param_c = { label="detail", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// MOTTLE - Abstract texture breakup / base variation
// Opcode: 0x14
// Role: Feature
//
// Purpose:
//   A low-strength generic texture carrier for breaking up broad flat fills on
//   sky, walls, fog, and floor regions without turning into a literal scene noun.
//
// Packed fields:
//   color_a: Primary tint
//   color_b: Secondary tint / recess tone
//   intensity: Brightness / overall strength (0..255 -> 0..1)
//   param_a: Pattern scale (0..255 -> 0.5..20.0)
//   param_b: Contrast (0..255 -> 0..1)
//   param_c: Detail / warp amount (0..255 -> 0..1)
//   param_d: Loop phase (0..255 -> 0..1) for gentle drift only
//   direction: Optional orientation / bias axis
//   alpha_a: Layer alpha
//   alpha_b: Unused
//
// Variants:
//   0 SOFT   - broad cloudy breakup, good for sky and fog support
//   1 GRAIN  - macro + fine grain, good for subtle surface variation
//   2 RIDGE  - broken ridged breakup, good for rough storm or rock texture
//   3 DAPPLE - patchy cellular breakup, good for uneven pooled variation
// ============================================================================

const MOTTLE_VARIANT_SOFT: u32 = 0u;
const MOTTLE_VARIANT_GRAIN: u32 = 1u;
const MOTTLE_VARIANT_RIDGE: u32 = 2u;
const MOTTLE_VARIANT_DAPPLE: u32 = 3u;

fn mottle_value_noise3(p: vec3f) -> f32 {
    return epu_value_noise3(p);
}

fn mottle_fbm3(p: vec3f, octaves: u32) -> f32 {
    return epu_fbm3(p, octaves);
}

fn mottle_apply_contrast(x: f32, contrast: f32) -> f32 {
    let gain = 1.0 + contrast * 3.0;
    return epu_saturate((x - 0.5) * gain + 0.5);
}

fn eval_mottle(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let variant = instr_variant_id(instr);
    let axis16 = instr_dir16(instr);
    let axis = normalize(select(vec3f(0.0, 1.0, 0.0), decode_dir16(axis16), axis16 != 0u));
    let scale = mix(0.5, 20.0, u8_to_01(instr_a(instr)));
    let contrast = u8_to_01(instr_b(instr));
    let detail = u8_to_01(instr_c(instr));
    let phase01 = epu_loop_phase01(instr_d(instr));
    let phase = phase01 * TAU;
    let drift_amount = mix(0.05, 0.35, detail);

    // Gentle drift only. This is a texture-breakup carrier, not a hero mover.
    let drift = epu_phase_orbit3(axis, phase01, drift_amount, drift_amount * 0.73);
    var p = dir * scale + drift;

    let warp = vec3f(
        mottle_value_noise3(p * 1.4 + vec3f(11.3, 0.0, 0.0)),
        mottle_value_noise3(p * 1.4 + vec3f(0.0, 17.9, 0.0)),
        mottle_value_noise3(p * 1.4 + vec3f(0.0, 0.0, 23.7))
    ) * mix(0.04, 0.22, detail);
    p += warp;

    var pat = 0.5;
    switch variant {
        case MOTTLE_VARIANT_SOFT: {
            let base = mottle_fbm3(p * 0.75 + axis * sin(phase) * 0.12, 3u);
            pat = base * 0.5 + 0.5;
        }
        case MOTTLE_VARIANT_GRAIN: {
            let macro_noise = mottle_fbm3(p * 0.6, 3u) * 0.5 + 0.5;
            let grain = mottle_value_noise3(p * mix(4.0, 10.0, detail) + axis * cos(phase) * 0.18) * 0.5 + 0.5;
            pat = mix(macro_noise, macro_noise * (0.7 + grain * 0.6), 0.3 + detail * 0.45);
        }
        case MOTTLE_VARIANT_RIDGE: {
            let ridge = 1.0 - abs(mottle_fbm3(p * 0.9, 3u));
            let breakup = mottle_value_noise3(p * 2.2 + vec3f(7.0, 19.0, 3.0)) * 0.5 + 0.5;
            pat = ridge * 0.75 + breakup * 0.25;
        }
        case MOTTLE_VARIANT_DAPPLE: {
            let a = mottle_fbm3(p * 0.8 + vec3f(13.0, 7.0, 19.0), 3u);
            let b = mottle_fbm3(p * 1.6 + vec3f(-5.0, 17.0, 2.0), 2u);
            pat = smoothstep(0.28, 0.72, a * 0.6 - b * 0.35 + 0.5);
        }
        default: {
            let base = mottle_fbm3(p * 0.75, 3u);
            pat = base * 0.5 + 0.5;
        }
    }

    pat = mottle_apply_contrast(pat, contrast);

    let rgb = mix(instr_color_a(instr), instr_color_b(instr), pat);
    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr) * intensity * region_w;
    let coverage = mix(0.22, 1.0, pat);
    return LayerSample(rgb, alpha * coverage);
}
