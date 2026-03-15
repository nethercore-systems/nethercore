// @epu_meta_begin
// opcode = 0x09
// name = GRID
// kind = radiance
// variants = []
// domains = []
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=1.0, max=64.0 }
// field param_b = { label="thickness", map="u8_lerp", min=0.001, max=0.1 }
// field param_c = { label="pat+scroll", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

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
    let phase01 = epu_loop_phase01(instr_d(instr));
    let phase_circle = epu_phase_circle(phase01);
    let scroll = phase_circle.x * (scroll_speed * 0.18) + phase_circle.y * (scroll_speed * 0.08);

    let uv0 = get_cyl_uv(dir);
    let phase_seed = fract(phase01 + f32(pattern_type) * 0.17);
    let seam_w = smoothstep(0.04, 0.24, epu_periodic_edge_distance(uv0.x));
    let uv_relief = epu_wrapped_relief_uv(
        vec2f(uv0.x + scroll, uv0.y),
        phase_seed + scroll * 0.37,
        mix(0.0, 0.055, seam_w),
        0.045
    );
    let uv = uv_relief;
    let scaled = uv * scale;
    let row = floor(scaled.y);
    let col = floor(scaled.x);
    let row_offset = (epu_phase_hash11(row * 17.3, phase_seed) - 0.5) * 0.32;
    let col_offset = (epu_phase_hash11(col * 23.7, phase_seed + 0.19) - 0.5) * 0.24;
    let wave_x = epu_relief_wave(vec2f(uv.y * 2.1, uv.x * 1.4), phase_seed) * 0.08;
    let wave_y = epu_relief_wave(vec2f(uv.x * 1.7, uv.y * 1.9), phase_seed + 0.31) * 0.05;
    let cell_relief = epu_relief_wave(vec2f(fract(scaled.x), fract(scaled.y)) * vec2f(2.7, 2.2), phase_seed + 0.53);
    let x_phase = scaled.x + row_offset + wave_x + cell_relief * 0.09;
    let y_phase = scaled.y + col_offset + wave_y + cell_relief * 0.06;
    let thickness_x = thickness * mix(0.9, 1.12, epu_phase_hash11(row * 11.9, phase_seed + 0.31));
    let thickness_y = thickness * mix(0.88, 1.1, epu_phase_hash11(col * 7.7, phase_seed + 0.53));
    let line_gate_x = mix(
        0.62,
        1.0,
        smoothstep(
            -0.3,
            0.72,
            epu_relief_wave(vec2f(y_phase * 0.16, row * 0.37 + fract(scaled.x) * 0.42), phase_seed + 0.13)
        )
    );
    let line_gate_y = mix(
        0.62,
        1.0,
        smoothstep(
            -0.28,
            0.74,
            epu_relief_wave(vec2f(x_phase * 0.18, col * 0.29 + fract(scaled.y) * 0.47), phase_seed + 0.39)
        )
    );

    // color_a = primary line color
    let line_rgb = instr_color_a(instr);

    var rgb = vec3f(0.0);
    var pat: f32 = 0.0;

    switch pattern_type {
        case 0u: { // STRIPES
            let fx = abs(epu_periodic_centered(x_phase));
            pat = (1.0 - step(thickness_x, fx)) * line_gate_x;
            rgb = line_rgb;
        }
        case 1u: { // GRID
            let fx = abs(epu_periodic_centered(x_phase));
            let fy = abs(epu_periodic_centered(y_phase));
            let h_line = (1.0 - step(thickness_x, fx)) * line_gate_x;
            let v_line = (1.0 - step(thickness_y, fy)) * line_gate_y;
            rgb = line_rgb;
            pat = max(h_line, v_line);
        }
        case 2u: { // CHECKER
            let cell = floor(vec2f(x_phase, y_phase));
            let checker = f32((i32(cell.x) + i32(cell.y)) & 1);
            pat = 1.0;
            rgb = mix(line_rgb * 0.6, line_rgb, checker);
        }
        default: {
            let fx = abs(epu_periodic_centered(x_phase));
            pat = (1.0 - step(thickness_x, fx)) * line_gate_x;
            rgb = line_rgb;
        }
    }

    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr);
    var chart_gate = 1.0;
    if pattern_type == 1u || pattern_type == 2u {
        let seam_gate = smoothstep(0.025, 0.12, epu_periodic_edge_distance(uv0.x));
        let cap_gate = smoothstep(0.78, 0.5, abs(dir.y));
        chart_gate = seam_gate * cap_gate;
    }
    return LayerSample(rgb, pat * chart_gate * intensity * alpha * region_w);
}
