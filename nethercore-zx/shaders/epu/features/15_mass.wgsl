// @epu_meta_begin
// opcode = 0x17
// name = MASS
// kind = feature
// variants = [BANK, SHELF, PLUME, VEIL]
// domains = [DIRECT3D, AXIS_CYL, AXIS_POLAR]
// field intensity = { label="density", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=0.5, max=10.0, unit="x" }
// field param_b = { label="coverage", map="u8_01" }
// field param_c = { label="breakup", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// MASS - Broad Scene-Owning Body Carrier
// Opcode: 0x17
// Role: Feature
//
// Purpose:
//   A reusable world-space body carrier for dominant fronts, shelves, plumes,
//   and diffuse masses that must read directly before transport/detail layers.
//
// Packed fields:
//   color_a: rim / lit edge tint
//   color_b: core / recess tint
//   intensity: body density / alpha
//   param_a: scale (0..255 -> 0.5..10.0)
//   param_b: coverage / occupancy
//   param_c: breakup / irregularity
//   param_d: slow phase drift
//   direction: preferred shaping / lean axis
//   alpha_a: layer alpha
//   alpha_b: unused
//
// Variants:
//   0 BANK   - dense wall-attached front body
//   1 SHELF  - suspended overhang / storm shelf
//   2 PLUME  - rising or leaning lifted body
//   3 VEIL   - soft diffuse suspended mass
// ============================================================================

const MASS_DOMAIN_DIRECT3D: u32 = 0u;
const MASS_DOMAIN_AXIS_CYL: u32 = 1u;
const MASS_DOMAIN_AXIS_POLAR: u32 = 2u;

const MASS_VARIANT_BANK: u32 = 0u;
const MASS_VARIANT_SHELF: u32 = 1u;
const MASS_VARIANT_PLUME: u32 = 2u;
const MASS_VARIANT_VEIL: u32 = 3u;

fn eval_mass(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if (region_w < 0.001) { return LayerSample(vec3f(0.0), 0.0); }

    let variant = instr_variant_id(instr);
    let domain = instr_domain_id(instr);
    let axis16 = instr_dir16(instr);
    let axis = normalize(select(vec3f(-1.0, 0.08, 0.0), decode_dir16(axis16), axis16 != 0u));

    let scale = mix(0.5, 10.0, u8_to_01(instr_a(instr)));
    let coverage = u8_to_01(instr_b(instr));
    let breakup = u8_to_01(instr_c(instr));
    let phase01 = epu_loop_phase01(instr_d(instr));
    let phase_circle = epu_phase_circle(phase01);
    let drift = phase_circle.x * mix(0.18, 1.05, breakup) + phase_circle.y * mix(0.06, 0.42, breakup);

    var coords = vec3f(0.0);
    var domain_gate = 1.0;
    switch domain {
        case MASS_DOMAIN_DIRECT3D: {
            let projected = advect_coords_direct3d(dir, axis);
            coords = projected.xyz;
            domain_gate = projected.w;
        }
        case MASS_DOMAIN_AXIS_CYL: {
            coords = advect_coords_cyl(dir, axis);
        }
        case MASS_DOMAIN_AXIS_POLAR: {
            coords = advect_coords_polar(dir, axis);
        }
        default: {
            let projected = advect_coords_direct3d(dir, axis);
            coords = projected.xyz;
            domain_gate = projected.w;
        }
    }

    let coords_shaped = epu_body_curve_coords(coords, breakup, phase01 + 0.27);
    let q = coords_shaped * scale;
    let body_noise = advect_fbm3(q * 0.78 + axis * drift * 0.22, 3u) * 0.5 + 0.5;
    let edge_noise = advect_fbm3(
        q * vec3f(0.32, 1.18, 0.52) + vec3f(-drift * 1.25, drift * 0.34, 11.0),
        3u
    ) * 0.5 + 0.5;
    let billow = advect_fbm3(
        q * vec3f(0.16, 0.92, 0.24) + vec3f(-drift * 0.86, drift * 0.18, 23.0),
        3u
    ) * 0.5 + 0.5;
    let erosion = advect_fbm3(
        q * vec3f(0.48, 1.36, 0.66) + vec3f(17.0, -9.0, drift * 0.55),
        3u
    ) * 0.5 + 0.5;
    let ribbing = advect_value_noise3(
        vec3f(
            q.y * 2.5 - drift * 2.1 + (edge_noise * 2.0 - 1.0) * 0.42,
            q.x * 0.46 + q.z * 0.31 + (billow * 2.0 - 1.0) * 0.35 + 7.0,
            q.z * 0.24 + (body_noise * 2.0 - 1.0) * 0.18
        )
    ) * 0.5 + 0.5;
    let ribbing_soft = smoothstep(0.16, 0.84, ribbing);
    let guide_relief = epu_relief_wave(vec2f(q.y * 0.27, q.z * 0.33), phase01 + drift * 0.09);
    let body_curve = breakup * mix(0.55, 1.0, body_noise * 0.52 + billow * 0.48);
    let lateral_relief = epu_relief_wave(
        vec2f(q.x * 0.19 + q.z * 0.31, q.y * 0.27 - q.x * 0.21),
        phase01 + drift * 0.07 + 0.41
    );
    let profile_relief = epu_relief_wave(
        vec2f(q.z * 0.23 - q.y * 0.17, q.x * 0.29 + q.y * 0.33),
        phase01 + 0.63
    );
    let front_axis = q.x
        + q.y * mix(-0.1, 0.18, body_curve)
        + q.z * mix(0.04, 0.14, body_curve)
        + guide_relief * mix(0.03, 0.2, body_curve)
        + lateral_relief * mix(0.01, 0.12, body_curve);
    let vertical_axis = q.y
        + q.x * mix(-0.05, 0.11, body_curve)
        + q.z * mix(-0.05, 0.1, body_curve)
        + guide_relief * mix(-0.02, 0.09, body_curve)
        + profile_relief * mix(-0.01, 0.08, body_curve);
    let depth_axis = coords_shaped.z
        + coords_shaped.x * mix(0.04, 0.2, body_curve)
        + coords_shaped.y * mix(-0.03, 0.09, body_curve)
        + guide_relief * mix(0.01, 0.08, body_curve)
        + lateral_relief * mix(0.0, 0.06, body_curve)
        - profile_relief * mix(0.0, 0.05, body_curve);
    let horizon_coord = dir.y
        + coords_shaped.x * 0.03 * body_curve
        + guide_relief * 0.05 * body_curve
        + profile_relief * 0.03 * body_curve;

    let horizon_band = 1.0 - smoothstep(0.14, 0.92, abs(horizon_coord + 0.01));
    let wall_depth_warp = (edge_noise * 2.0 - 1.0) * mix(0.035, 0.12, breakup)
        + (billow * 2.0 - 1.0) * mix(0.015, 0.075, breakup);
    let wall_depth = smoothstep(
        0.08,
        0.96,
        1.0 - abs(depth_axis * mix(0.42, 0.82, coverage) + wall_depth_warp)
    );
    let occupancy = smoothstep(0.24, 0.78, body_noise * 0.66 + billow * 0.34);

    var density = 0.0;
    var core = 0.0;
    var rim = 0.0;
    var fade = 0.0;
    var mix_w = 0.0;
    var alpha_scale = 1.0;
    switch variant {
        case MASS_VARIANT_BANK: {
            let lip_warp = (edge_noise * 2.0 - 1.0) * mix(0.04, 0.2, breakup);
            let belly_warp = (billow * 2.0 - 1.0) * mix(0.03, 0.18, breakup);
            let bank_face = 1.0 - smoothstep(
                -0.08,
                mix(0.22, 0.72, coverage),
                front_axis + lip_warp
            );
            let inner_face = 1.0 - smoothstep(
                -0.18,
                mix(0.06, 0.52, coverage),
                front_axis + mix(0.14, 0.4, coverage) + belly_warp
            );
            let shoulder = smoothstep(0.18, 0.92, bank_face) * (1.0 - smoothstep(0.14, 0.9, inner_face));
            core = bank_face
                * inner_face
                * occupancy
                * horizon_band
                * wall_depth
                * mix(0.74, 1.0, ribbing_soft);
            rim = smoothstep(0.28, 0.92, bank_face) * horizon_band * wall_depth * mix(0.54, 1.0, shoulder);
            fade = erosion;
            density = max(
                core * mix(0.58, 1.0, fade),
                bank_face
                    * occupancy
                    * horizon_band
                    * wall_depth
                    * mix(0.16, 0.44, inner_face)
                    * mix(0.7, 1.0, shoulder)
            );
            density = smoothstep(0.08, 0.92, density);
            mix_w = epu_saturate(0.005 + rim * 0.04 + shoulder * 0.06 + fade * 0.015 - core * 0.46 - density * 0.16);
            alpha_scale = mix(1.42, 2.72, epu_saturate(core * 0.88 + inner_face * 0.12));
        }
        case MASS_VARIANT_SHELF: {
            let shelf_drop = mix(-0.18, 0.18, coverage);
            let overhang = 1.0 - smoothstep(
                -0.12,
                0.44,
                vertical_axis - shelf_drop + (billow * 2.0 - 1.0) * mix(0.03, 0.16, breakup)
            );
            let shelf_band = smoothstep(
                0.14,
                0.94,
                1.0 - abs(
                    vertical_axis
                    - (shelf_drop - mix(0.06, 0.18, coverage))
                    + (billow * 2.0 - 1.0) * mix(0.02, 0.1, breakup)
                )
            );
            let crown_trim = 1.0 - smoothstep(
                mix(0.14, 0.3, coverage),
                mix(0.44, 0.72, coverage),
                vertical_axis + (edge_noise * 2.0 - 1.0) * mix(0.02, 0.08, breakup)
            );
            let nose = 1.0 - smoothstep(
                -0.08,
                mix(0.28, 0.88, coverage),
                front_axis + drift * 0.48 + (edge_noise * 2.0 - 1.0) * mix(0.05, 0.24, breakup)
            );
            let belly_band = smoothstep(
                0.14,
                0.92,
                1.0 - smoothstep(
                    -0.3,
                    0.16,
                    vertical_axis - shelf_drop + (billow * 2.0 - 1.0) * mix(0.03, 0.12, breakup)
                )
            );
            let underside_hold = smoothstep(
                0.16,
                0.92,
                1.0 - smoothstep(
                    -0.2,
                    0.26,
                    vertical_axis + drift * 0.16 + (edge_noise * 2.0 - 1.0) * mix(0.04, 0.18, breakup)
                )
            );
            let shoulder = smoothstep(0.18, 0.9, overhang) * smoothstep(0.2, 0.92, nose) * underside_hold * shelf_band;
            let body_hold = mix(0.62, 1.0, belly_band) * mix(0.66, 1.0, underside_hold) * mix(0.64, 1.0, shelf_band);
            let recess = smoothstep(0.24, 0.92, nose) * body_hold * mix(0.72, 1.0, shoulder) * mix(0.7, 1.0, crown_trim);
            let shelf_rib_mix = mix(0.86, 1.0, ribbing_soft * 0.22 + billow * 0.36);
            core = overhang
                * shelf_band
                * nose
                * occupancy
                * horizon_band
                * wall_depth
                * shelf_rib_mix
                * body_hold
                * mix(0.76, 1.0, shoulder)
                * mix(0.72, 1.0, crown_trim);
            rim = smoothstep(0.42, 0.96, overhang * shelf_band * nose)
                * horizon_band
                * wall_depth
                * mix(0.18, 0.46, shoulder)
                * crown_trim;
            fade = erosion;
            density = max(
                max(
                    core * mix(0.72, 1.0, fade),
                    overhang
                        * shelf_band
                        * nose
                        * occupancy
                        * horizon_band
                        * wall_depth
                        * mix(0.14, 0.4, billow)
                        * body_hold
                        * mix(0.7, 1.0, shoulder)
                        * mix(0.72, 1.0, crown_trim)
                ),
                overhang * shelf_band * horizon_band * wall_depth * recess * 0.3
            );
            density = smoothstep(0.06, 0.86, density);
            mix_w = epu_saturate(0.003 + rim * 0.016 + fade * 0.012 - core * 0.58 - density * 0.24 - recess * 0.12);
            alpha_scale = mix(1.52, 3.18, epu_saturate(core * 0.76 + recess * 0.24));
        }
        case MASS_VARIANT_PLUME: {
            let spine = smoothstep(
                0.08,
                0.92,
                1.0 - abs(front_axis + drift * 0.32 + (edge_noise * 2.0 - 1.0) * mix(0.04, 0.18, breakup))
            );
            let lift = smoothstep(-0.18, 0.68, vertical_axis + (billow * 2.0 - 1.0) * mix(0.05, 0.16, breakup));
            core = spine * lift * occupancy * wall_depth * mix(0.72, 1.0, ribbing_soft);
            rim = smoothstep(0.26, 0.9, spine * lift) * wall_depth;
            fade = erosion;
            density = max(core * mix(0.56, 1.0, fade), spine * lift * wall_depth * 0.24);
            mix_w = epu_saturate(0.01 + rim * 0.11 + fade * 0.05 - core * 0.18);
            alpha_scale = mix(1.24, 2.2, epu_saturate(core * 0.78 + billow * 0.22));
        }
        case MASS_VARIANT_VEIL: {
            let fog = smoothstep(0.18, 0.78, advect_fbm3(q * 0.55 + axis * drift * 0.12, 3u) * 0.5 + 0.5);
            core = fog * mix(0.44, 1.0, occupancy) * horizon_band;
            rim = smoothstep(0.34, 0.88, fog) * horizon_band;
            fade = edge_noise;
            density = core;
            mix_w = epu_saturate(0.01 + rim * 0.11 + fade * 0.05 - core * 0.18);
            alpha_scale = mix(1.24, 2.2, epu_saturate(core * 0.78 + billow * 0.22));
        }
        default: {
            core = occupancy * horizon_band * wall_depth;
            rim = smoothstep(0.3, 0.9, occupancy) * horizon_band * wall_depth;
            fade = edge_noise;
            density = core;
            mix_w = epu_saturate(0.01 + rim * 0.11 + fade * 0.05 - core * 0.18);
            alpha_scale = mix(1.24, 2.2, epu_saturate(core * 0.78 + billow * 0.22));
        }
    }

    density = epu_saturate(density) * domain_gate;
    let rgb = mix(instr_color_b(instr), instr_color_a(instr), mix_w);
    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr) * intensity * region_w;
    return LayerSample(rgb, alpha * alpha_scale * density);
}
