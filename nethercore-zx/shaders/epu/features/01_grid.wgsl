// ============================================================================
// GRID - Repeating Lines / Panels
// Packed fields (v2):
//   color_a: Primary line color (RGB24)
//   color_b: Cross/secondary line color (RGB24)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..64)
//   param_b: Thickness (0..255 -> 0.001..0.1)
//   param_c[7:4]: Pattern (0=stripes, 1=grid, 2=checker)
//   param_c[3:0]: Scroll speed (0..15 -> 0..2)
//   param_d: Rotation angle (0..255 -> 0..TAU)
//   direction: Orientation (reserved for future expansion)
// ============================================================================

fn get_cyl_uv(dir: vec3f) -> vec2f {
    let u = atan2(dir.x, dir.z) / TAU;        // [-0.5..0.5]
    let v = dir.y * 0.5 + 0.5;                // [0..1]
    return vec2f(u, v);
}

fn cyl_uv_to_dir(uv: vec2f) -> vec3f {
    let theta = uv.x * TAU;
    let y = clamp(uv.y * 2.0 - 1.0, -1.0, 1.0);
    let r = sqrt(max(0.0, 1.0 - y * y));
    return vec3f(sin(theta) * r, y, cos(theta) * r);
}

fn rotate_2d(uv: vec2f, angle: f32) -> vec2f {
    let c = cos(angle);
    let s = sin(angle);
    return vec2f(uv.x * c - uv.y * s, uv.x * s + uv.y * c);
}

fn eval_grid(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let scale = mix(1.0, 64.0, u8_to_01(instr_a(instr)));
    let thickness = mix(0.001, 0.1, u8_to_01(instr_b(instr)));

    let pc = instr_c(instr);
    let pattern_type = (pc >> 4u) & 0xFu;
    let scroll_q = pc & 0xFu;
    let scroll_speed = (f32(scroll_q) / 15.0) * 2.0;

    // Rotation from param_d
    let rotation = u8_to_01(instr_d(instr)) * TAU;

    let uv0 = get_cyl_uv(dir);
    let uv_rotated = rotate_2d(uv0, rotation);
    let uv = uv_rotated + vec2f(time * scroll_speed, 0.0);

    // color_a = primary line color, color_b = cross/secondary line color
    let line_rgb = instr_color_a(instr);
    let cross_rgb = instr_color_b(instr);

    var rgb = vec3f(0.0);
    var pat: f32 = 0.0;

    switch pattern_type {
        case 0u: { // STRIPES
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
        case 1u: { // GRID - horizontal uses line_rgb, vertical uses cross_rgb
            let fx = abs(fract(uv.x * scale) - 0.5);
            let fy = abs(fract(uv.y * scale) - 0.5);
            let h_line = 1.0 - step(thickness, fx);
            let v_line = 1.0 - step(thickness, fy);
            // Blend colors based on which lines are hit
            let h_w = h_line * (1.0 - v_line);
            let v_w = v_line * (1.0 - h_line);
            let both_w = h_line * v_line;
            rgb = line_rgb * h_w + cross_rgb * v_w + mix(line_rgb, cross_rgb, 0.5) * both_w;
            pat = max(h_line, v_line);
        }
        case 2u: { // CHECKER - alternating colors
            let cell = floor(vec2f(uv.x, uv.y) * scale);
            let checker = fract((cell.x + cell.y) * 0.5) * 2.0;
            pat = 1.0;
            rgb = mix(line_rgb, cross_rgb, checker);
        }
        default: {
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
    }

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, pat * intensity * region_w);
}
