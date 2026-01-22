// ============================================================================
// SCATTER - Point Field (Stars / Dust / Windows)
// Packed fields (v2):
//   color_a: Primary point color (RGB24)
//   color_b: Color variation (RGB24) - points randomly vary between a and b
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Density (0..255 -> 1..256)
//   param_b: Point size (0..255 -> 0.001..0.05 rad)
//   param_c[7:4]: Twinkle amount (0..15 -> 0..1)
//   param_c[3:0]: Twinkle speed (0..15 -> 0..5)
//   param_d: Seed for randomization (0..255)
//   direction: Drift direction (oct-u16). If non-zero, the scatter field scrolls over time
//              using the twinkle speed nibble as the drift speed control.
// ============================================================================

fn hash3(p: vec3f) -> vec4f {
    var p4 = fract(vec4f(p.xyzx) * vec4f(0.1031, 0.1030, 0.0973, 0.1099));
    p4 += dot(p4, p4.wzxy + 33.33);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

fn eval_scatter(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let density = mix(1.0, 256.0, u8_to_01(instr_a(instr)));
    let size = mix(0.001, 0.05, u8_to_01(instr_b(instr)));

    let pc = instr_c(instr);
    let twinkle = f32((pc >> 4u) & 0xFu) / 15.0;
    let twinkle_speed = f32(pc & 0xFu) / 15.0 * 5.0;

    // Seed from param_d
    let seed = f32(instr_d(instr));

    // Optional drift: scroll the scatter field over time if direction != 0.
    // This is intentionally "stylized" (cylindrical UV scroll) rather than a physically
    // correct spherical advection; it keeps motion smooth and looping for effects like
    // snowfall / rainfall particles.
    var dir_s = dir;
    let drift_dir16 = instr_dir16(instr);
    if drift_dir16 != 0u && twinkle_speed > 0.001 {
        let drift3 = decode_dir16(drift_dir16);
        let drift_uv0 = vec2f(drift3.x, drift3.y);
        let drift_len2 = dot(drift_uv0, drift_uv0);
        if drift_len2 > 1e-5 {
            let drift_uv = drift_uv0 / sqrt(drift_len2);
            // Map twinkle_speed (0..5) -> drift speed in UV units per second.
            // 1.0 UV unit corresponds to a full wrap (360 deg in U, 0..1 in V).
            let drift_speed = twinkle_speed * 0.12;

            let uv0 = get_cyl_uv(dir_s);
            let u = fract(uv0.x + 0.5 + drift_uv.x * time * drift_speed) - 0.5;
            let v = fract(uv0.y + drift_uv.y * time * drift_speed);
            dir_s = cyl_uv_to_dir(vec2f(u, v));
        }
    }

    // Cell on direction sphere (cheap hash distribution).
    let cell = floor(dir_s * density);
    let h = hash3(cell + vec3f(seed));
    let point_offset = h.xyz * 2.0 - 1.0;
    var v = cell + point_offset * 0.5;
    if length(v) < 1e-5 {
        v = vec3f(1.0, 0.0, 0.0);
    }
    let point_dir = normalize(v);

    let dist = acos(epu_saturate(dot(dir_s, point_dir)));
    let point = smoothstep(size, size * 0.3, dist);

    let tw = select(1.0, (0.5 + 0.5 * sin(h.w * TAU + time * twinkle_speed)), twinkle > 0.001);

    // color_a = primary point color, color_b = variation color
    // Mix between colors based on hash for per-point variation
    let point_rgb = instr_color_a(instr);
    let var_rgb = instr_color_b(instr);
    let rgb = mix(point_rgb, var_rgb, h.x);

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, point * intensity * mix(1.0, tw, twinkle) * region_w);
}
