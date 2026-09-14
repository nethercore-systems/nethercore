// @epu_meta_begin
// opcode = 0x09
// name = GRID
// kind = radiance
// variants = []
// domains = []
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="repeat target (rounded)", map="u8_lerp", min=1.0, max=64.0 }
// field param_b = { label="thickness", map="u8_lerp", min=0.001, max=0.1 }
// field param_c = { label="pattern+cycles (packed)", map="u8_lerp", min=0.0, max=255.0 }
// field param_d = { label="phase byte", map="u8_lerp", min=0.0, max=255.0 }
// @epu_meta_end

// ============================================================================
// GRID - Repeating Lines / Panels
// Packed fields:
//   color_a: Primary line color (RGB24)
//   color_b: Reserved (set to 0)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Nearest whole repeat count (1..64); checker uses nearest even count (2..64)
//   param_b: Thickness (0..255 -> 0.001..0.1)
//   param_c[7:4]: Pattern (0=stripes, 1=grid, 2=checker)
//   param_c[3:0]: Whole pattern cycles per phase loop (0..15; zero is stationary)
//   param_d: Guest phase (0..255 -> raw/256 of a loop, with no duplicated endpoint)
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

    let thickness = mix(0.001, 0.1, u8_to_01(instr_b(instr)));

    let pc = instr_c(instr);
    let pattern_type = (pc >> 4u) & 0xFu;
    // Integral circumferences close the chart; checker parity requires an even count.
    // Integer half-up rounding avoids backend-dependent floating-point ties.
    let count = select((382u + 63u * instr_a(instr)) / 255u,
        2u * ((510u + 63u * instr_a(instr)) / 510u), pattern_type == 2u);
    let scale = f32(count);
    let scroll_q = pc & 0xFu;
    let phase01 = epu_loop_phase01(instr_d(instr));
    // One checker cycle spans two cells; all rates close after 256 guest phase steps.
    let scroll = phase01 * f32(scroll_q) * select(1.0, 2.0, pattern_type == 2u);

    let uv0 = get_cyl_uv(dir);
    let uv = vec2f(uv0.x * scale + scroll, uv0.y * scale);

    // color_a = primary line color
    let line_rgb = instr_color_a(instr);

    var rgb = vec3f(0.0);
    var pat: f32 = 0.0;

    switch pattern_type {
        case 0u: { // STRIPES
            let fx = abs(fract(uv.x) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
        case 1u: { // GRID
            let fx = abs(fract(uv.x) - 0.5);
            let fy = abs(fract(uv.y) - 0.5);
            let h_line = 1.0 - step(thickness, fx);
            let v_line = 1.0 - step(thickness, fy);
            rgb = line_rgb;
            pat = max(h_line, v_line);
        }
        case 2u: { // CHECKER
            let cell = floor(uv);
            let checker = f32((i32(cell.x) + i32(cell.y)) & 1);
            pat = 1.0;
            rgb = mix(line_rgb * 0.6, line_rgb, checker);
        }
        default: {
            let fx = abs(fract(uv.x) - 0.5);
            pat = 1.0 - step(thickness, fx);
            rgb = line_rgb;
        }
    }

    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr);
    return LayerSample(rgb, pat * intensity * alpha * region_w);
}
