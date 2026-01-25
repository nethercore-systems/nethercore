// ============================================================================
// GRID - Repeating Lines / Panels
// Packed fields:
//   color_a: Primary line color (RGB24)
//   color_b: Reserved (set to 0)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..64)
//   param_b: Thickness (0..255 -> 0.001..0.1)
//   param_c[7:4]: Pattern (0=stripes, 1=grid, 2=checker)
//   param_c[3:0]: Scroll speed (0..15 -> 0..2)
//   param_d: Phase (0..255 -> 0..1)
//   direction: Orientation (reserved for future expansion)
// ============================================================================

fn get_cyl_uv(dir: vec3f) -> vec2f {
    let u = atan2(dir.x, dir.z) / TAU;        // [-0.5..0.5]
    let v = dir.y * 0.5 + 0.5;                // [0..1]
    return vec2f(u, v);
}

fn eval_grid(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let scale = mix(1.0, 64.0, u8_to_01(instr_a(instr)));
    let thickness = mix(0.001, 0.1, u8_to_01(instr_b(instr)));

    let pc = instr_c(instr);
    let pattern_type = (pc >> 4u) & 0xFu;
    let scroll_q = pc & 0xFu;
    let scroll_speed = f32(scroll_q) / 15.0 * 2.0;
    let scroll = u8_to_01(instr_d(instr)) * scroll_speed;

    let uv0 = get_cyl_uv(dir);
    let uv = vec2f(uv0.x + scroll, uv0.y);

    // color_a = primary line color
    let line_rgb = instr_color_a(instr);

    var rgb = vec3f(0.0);
    var pat: f32 = 0.0;

    switch pattern_type {
        case 0u: { // STRIPES
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
        case 1u: { // GRID
            let fx = abs(fract(uv.x * scale) - 0.5);
            let fy = abs(fract(uv.y * scale) - 0.5);
            let h_line = 1.0 - step(thickness, fx);
            let v_line = 1.0 - step(thickness, fy);
            rgb = line_rgb;
            pat = max(h_line, v_line);
        }
        case 2u: { // CHECKER
            let cell = floor(vec2f(uv.x, uv.y) * scale);
            let checker = f32((i32(cell.x) + i32(cell.y)) & 1);
            pat = 1.0;
            rgb = mix(line_rgb * 0.6, line_rgb, checker);
        }
        default: {
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
    }

    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr);
    return LayerSample(rgb, pat * intensity * alpha * region_w);
}
