// @epu_meta_begin
// opcode = 0x15
// name = ADVECT
// kind = feature
// variants = [SHEET, SPINDRIFT, SQUALL, MIST, BANK, FRONT]
// domains = [DIRECT3D, AXIS_CYL, AXIS_POLAR]
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=0.5, max=12.0, unit="x" }
// field param_b = { label="coverage", map="u8_01" }
// field param_c = { label="breakup", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// ADVECT - Broad Transport / Sheet Motion Carrier
// Opcode: 0x15
// Role: Feature
//
// Purpose:
//   A reusable broad transport carrier for weather sheets, fog banks,
//   spindrift, and other direct-view moving masses that should read as a
//   coherent world-space event rather than as thin bars or particle shimmer.
//
// Packed fields:
//   color_a: leading-edge / bright tint
//   color_b: recess / interior tint
//   intensity: brightness / density
//   param_a: scale (0..255 -> 0.5..12.0)
//   param_b: coverage / slab width
//   param_c: breakup / irregularity
//   param_d: loop phase
//   direction: prevailing travel direction / wind axis
//   alpha_a: layer alpha
//   alpha_b: unused
//
// Variants:
//   0 SHEET     - broad drifting density slab
//   1 SPINDRIFT - thinner broken transport with cold lifted edges
//   2 SQUALL    - denser storm body with embedded streak structure
//   3 MIST      - soft low-contrast suspended haze
//   4 BANK      - forceful wall-attached mass / front body
//   5 FRONT     - broad dominant storm shelf / front wall
//
// Domains:
//   0 DIRECT3D  - world-space directional basis
//   1 AXIS_CYL  - wrapped cylindrical sheet
//   2 AXIS_POLAR - radial / orbital transport field
// ============================================================================

const ADVECT_DOMAIN_DIRECT3D: u32 = 0u;
const ADVECT_DOMAIN_AXIS_CYL: u32 = 1u;
const ADVECT_DOMAIN_AXIS_POLAR: u32 = 2u;

const ADVECT_VARIANT_SHEET: u32 = 0u;
const ADVECT_VARIANT_SPINDRIFT: u32 = 1u;
const ADVECT_VARIANT_SQUALL: u32 = 2u;
const ADVECT_VARIANT_MIST: u32 = 3u;
const ADVECT_VARIANT_BANK: u32 = 4u;
const ADVECT_VARIANT_FRONT: u32 = 5u;

fn advect_hash31(p: vec3f) -> f32 {
    let h = dot(p, vec3f(157.1, 311.7, 73.7));
    return fract(sin(h) * 43758.5453123);
}

fn advect_value_noise3(p: vec3f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = advect_hash31(i + vec3f(0.0, 0.0, 0.0));
    let b = advect_hash31(i + vec3f(1.0, 0.0, 0.0));
    let c = advect_hash31(i + vec3f(0.0, 1.0, 0.0));
    let d = advect_hash31(i + vec3f(1.0, 1.0, 0.0));
    let e = advect_hash31(i + vec3f(0.0, 0.0, 1.0));
    let f1 = advect_hash31(i + vec3f(1.0, 0.0, 1.0));
    let g = advect_hash31(i + vec3f(0.0, 1.0, 1.0));
    let h = advect_hash31(i + vec3f(1.0, 1.0, 1.0));

    let ab = mix(a, b, u.x);
    let cd = mix(c, d, u.x);
    let ef = mix(e, f1, u.x);
    let gh = mix(g, h, u.x);
    let abcd = mix(ab, cd, u.y);
    let efgh = mix(ef, gh, u.y);

    return mix(abcd, efgh, u.z) * 2.0 - 1.0;
}

fn advect_fbm3(p: vec3f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * advect_value_noise3(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

fn advect_build_basis(axis: vec3f) -> mat3x3f {
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(ref_vec, axis));
    let b = normalize(cross(axis, t));
    return mat3x3f(t, axis, b);
}

fn advect_coords_direct3d(dir: vec3f, axis: vec3f) -> vec3f {
    let basis = advect_build_basis(axis);
    return vec3f(dot(dir, basis[0]), dot(dir, basis[1]), dot(dir, basis[2]));
}

fn advect_coords_cyl(dir: vec3f, axis: vec3f) -> vec3f {
    let basis = advect_build_basis(axis);
    let x = dot(dir, basis[0]);
    let y = dot(dir, basis[1]);
    let z = dot(dir, basis[2]);
    let angle = atan2(x, z);
    return vec3f(cos(angle), sin(angle), y);
}

fn advect_coords_polar(dir: vec3f, axis: vec3f) -> vec3f {
    let basis = advect_build_basis(axis);
    let x = dot(dir, basis[0]);
    let y = dot(dir, basis[1]);
    let z = dot(dir, basis[2]);
    let angle = atan2(x, z);
    let radius = acos(clamp(y, -1.0, 1.0)) / PI;
    return vec3f(cos(angle) * radius, sin(angle) * radius, radius * 2.0 - 1.0);
}

fn eval_advect(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let variant = instr_variant_id(instr);
    let domain = instr_domain_id(instr);
    let axis16 = instr_dir16(instr);
    let axis = normalize(select(vec3f(-1.0, 0.08, 0.0), decode_dir16(axis16), axis16 != 0u));

    let scale = mix(0.5, 12.0, u8_to_01(instr_a(instr)));
    let coverage = u8_to_01(instr_b(instr));
    let breakup = u8_to_01(instr_c(instr));
    let phase01 = u8_to_01(instr_d(instr));
    let travel = phase01 * mix(0.6, 2.4, breakup);

    var coords = advect_coords_direct3d(dir, axis);
    switch domain {
        case ADVECT_DOMAIN_DIRECT3D: {
            coords = advect_coords_direct3d(dir, axis);
        }
        case ADVECT_DOMAIN_AXIS_CYL: {
            coords = advect_coords_cyl(dir, axis);
        }
        case ADVECT_DOMAIN_AXIS_POLAR: {
            coords = advect_coords_polar(dir, axis);
        }
        default: {
            coords = advect_coords_direct3d(dir, axis);
        }
    }

    let q = coords * scale;
    let body_noise = advect_fbm3(q * 0.85 + axis * travel * 0.7, 3u) * 0.5 + 0.5;
    let breakup_noise = advect_fbm3(q * 1.7 - axis * travel * 1.3 + vec3f(13.0, 7.0, 19.0), 2u) * 0.5 + 0.5;
    let sheet_noise = advect_fbm3(
        vec3f(q.y * 0.75 + travel * 0.2, q.z * 0.75 - travel * 0.15, q.x * 0.35 + 9.0),
        3u
    ) * 0.5 + 0.5;

    let half_width = mix(0.12, 1.1, coverage);
    let soft_width = mix(0.08, 0.32, breakup);
    let slab_warp = mix(0.04, 0.45, breakup) * (body_noise * 2.0 - 1.0);
    let edge_warp = (sheet_noise * 2.0 - 1.0) * mix(0.06, 0.7, breakup);
    let slab_coord = q.x + travel * mix(0.35, 1.1, breakup) + slab_warp + edge_warp;
    let slab = 1.0 - smoothstep(half_width, half_width + soft_width, abs(slab_coord));
    let front = 1.0 - smoothstep(
        -half_width * 0.25 - soft_width,
        half_width + soft_width * 1.6,
        slab_coord - (breakup_noise * 2.0 - 1.0) * mix(0.05, 0.65, breakup)
    );

    var density = slab;
    var bank_core = 0.0;
    var bank_rim = 0.0;
    var bank_veining = 0.0;
    var front_core = 0.0;
    var front_rim = 0.0;
    var front_erosion = 0.0;
    var front_crest = 0.0;
    var front_billow = 0.0;
    var front_event = 0.0;
    switch variant {
        case ADVECT_VARIANT_SHEET: {
            let body = smoothstep(0.28, 0.78, body_noise * mix(0.75, 1.15, coverage));
            density = slab * body;
        }
        case ADVECT_VARIANT_SPINDRIFT: {
            let streaks = advect_value_noise3(vec3f(q.y * 2.6 - travel * 2.4, q.z * 1.7, q.x * 0.8)) * 0.5 + 0.5;
            let horizon_band = 1.0 - smoothstep(0.04, 0.56, abs(dir.y + 0.03));
            let lift = smoothstep(0.28, 0.82, body_noise * 0.7 + breakup_noise * 0.3);
            density = front * lift * mix(0.3, 1.0, streaks) * horizon_band;
        }
        case ADVECT_VARIANT_SQUALL: {
            let streaks = advect_value_noise3(vec3f(q.y * 3.4 - travel * 3.0, q.z * 0.7 + travel, q.x * 1.1)) * 0.5 + 0.5;
            let horizon_band = 1.0 - smoothstep(0.16, 0.96, abs(dir.y + 0.02));
            let belly = smoothstep(0.16, 0.72, body_noise * 0.6 + breakup_noise * 0.4);
            let erosion = smoothstep(
                0.2,
                0.8,
                advect_fbm3(q * vec3f(0.45, 1.1, 0.9) + vec3f(-travel * 0.8, travel * 0.35, travel * 0.2), 3u) * 0.5 + 0.5
            );
            let curtain = smoothstep(
                0.18,
                0.82,
                advect_fbm3(q * vec3f(0.55, 1.35, 0.85) + vec3f(17.0, -9.0, travel * 0.4), 3u) * 0.5 + 0.5
            );
            let depth_fade = smoothstep(0.1, 0.86, 1.0 - abs(q.z + (sheet_noise * 2.0 - 1.0) * 0.45));
            let storm_body = front * mix(0.42, 1.0, belly) * horizon_band * depth_fade;
            let detail = mix(0.68, 1.0, streaks) * mix(0.72, 1.0, erosion) * mix(0.76, 1.0, curtain);
            density = storm_body * detail;
        }
        case ADVECT_VARIANT_MIST: {
            let fog = smoothstep(0.18, 0.72, advect_fbm3(q * 0.6 + axis * travel * 0.25, 3u) * 0.5 + 0.5);
            density = mix(fog, slab * fog, 0.45 + coverage * 0.35);
        }
        case ADVECT_VARIANT_BANK: {
            let horizon_band = 1.0 - smoothstep(0.12, 0.82, abs(dir.y + 0.01));
            let mass = smoothstep(0.18, 0.8, body_noise * 0.78 + breakup_noise * 0.22);
            let wall_front = max(front, slab * 0.82);
            let depth_coord = coords.z * 0.7 + (sheet_noise * 2.0 - 1.0) * 0.12;
            let wall_depth = smoothstep(0.02, 0.98, 1.0 - abs(depth_coord));
            let veining = advect_value_noise3(vec3f(q.y * 2.8 - travel * 2.2, q.z * 0.55, q.x * 0.7 + 11.0)) * 0.5 + 0.5;
            bank_core = wall_front
                * mix(0.68, 1.0, mass)
                * horizon_band
                * wall_depth;
            bank_rim = smoothstep(0.34, 0.94, front) * horizon_band;
            bank_veining = veining;
            density = max(
                bank_core * mix(0.88, 1.0, veining),
                slab * horizon_band * wall_depth * 0.55
            );
        }
        case ADVECT_VARIANT_FRONT: {
            let front_shift = travel * mix(2.8, 6.4, breakup);
            let horizon_band = 1.0 - smoothstep(0.18, 0.92, abs(dir.y + 0.02));
            let shelf_shape = smoothstep(
                -half_width * 0.22 - soft_width * 0.8,
                half_width + soft_width * 2.2,
                (slab_coord + front_shift * 0.45) - (breakup_noise * 2.0 - 1.0) * mix(0.08, 0.72, breakup)
            );
            let occupied = 1.0 - shelf_shape;
            let body_fill = smoothstep(0.16, 0.76, body_noise * 0.74 + breakup_noise * 0.26);
            let roofline = smoothstep(
                0.06,
                0.86,
                advect_fbm3(
                    q * vec3f(0.26, 0.72, 0.48) + vec3f(-front_shift * 0.6, front_shift * 0.28, 23.0),
                    3u
                ) * 0.5 + 0.5
            );
            let wall_depth = smoothstep(
                0.02,
                0.96,
                1.0 - abs(coords.z * 0.46 + (sheet_noise * 2.0 - 1.0) * mix(0.06, 0.28, breakup))
            );
            let ribbing = advect_value_noise3(vec3f(q.y * 2.2 - front_shift * 2.6, q.z * 0.42, q.x * 0.55 + 21.0)) * 0.5 + 0.5;
            let erosion = smoothstep(
                0.16,
                0.86,
                advect_fbm3(
                    q * vec3f(0.42, 1.04, 0.62) + vec3f(17.0, -11.0, front_shift * 0.8),
                    3u
                ) * 0.5 + 0.5
            );
            let surge = smoothstep(
                0.18,
                0.88,
                advect_fbm3(
                    q * vec3f(0.34, 1.58, 0.44) + vec3f(-front_shift * 2.4, front_shift * 0.52, 37.0),
                    3u
                ) * 0.5 + 0.5
            );
            let crest = smoothstep(
                0.18,
                0.88,
                advect_fbm3(
                    q * vec3f(0.2, 1.96, 0.32) + vec3f(-front_shift * 3.4, front_shift * 0.78, 41.0),
                    3u
                ) * 0.5 + 0.5
            );
            let billow = smoothstep(
                0.14,
                0.84,
                advect_fbm3(
                    q * vec3f(0.16, 0.92, 0.24) + vec3f(-front_shift * 1.9, -front_shift * 0.28, 53.0),
                    3u
                ) * 0.5 + 0.5
            );
            let shear = advect_value_noise3(vec3f(q.y * 3.1 - front_shift * 3.3, q.x * 0.72 + 5.0, q.z * 0.18)) * 0.5 + 0.5;
            let pulse = smoothstep(
                0.24,
                0.9,
                advect_fbm3(
                    q * vec3f(0.24, 2.24, 0.28) + vec3f(-front_shift * 4.6, front_shift * 0.94, 67.0),
                    3u
                ) * 0.5 + 0.5
            );
            let shelf_mass = smoothstep(0.18, 0.84, occupied * 0.7 + front * 0.3);
            let wall_hold = smoothstep(0.16, 0.82, wall_depth * 0.82 + slab * 0.18);
            let leading_edge = smoothstep(
                0.26,
                0.92,
                1.0 - smoothstep(
                    -half_width * 0.06 - soft_width * 0.45,
                    half_width * 0.52 + soft_width * 1.65,
                    slab_coord
                        + front_shift * 0.96
                        + (crest * 2.0 - 1.0) * mix(0.05, 0.28, breakup)
                )
            );
            let event_band = occupied
                * horizon_band
                * wall_hold
                * mix(0.32, 0.88, surge)
                * mix(0.62, 1.0, leading_edge)
                * mix(0.56, 1.0, pulse)
                * mix(0.72, 1.0, billow);
            front_core = shelf_mass
                * mix(0.78, 1.0, body_fill)
                * mix(0.82, 1.0, roofline)
                * mix(0.72, 1.0, ribbing)
                * mix(0.54, 1.0, surge)
                * mix(0.52, 1.0, crest)
                * mix(0.62, 1.0, billow)
                * mix(0.74, 1.0, shear)
                * horizon_band
                * wall_hold;
            front_rim = smoothstep(0.34, 0.92, occupied) * horizon_band * wall_hold * mix(0.65, 1.0, leading_edge);
            front_erosion = erosion * mix(0.64, 1.0, billow);
            front_crest = crest;
            front_billow = billow;
            front_event = epu_saturate(pulse * 0.56 + surge * 0.34 + leading_edge * 0.24 - 0.22);
            density = max(
                front_core * mix(0.56, 1.0, erosion) * mix(0.58, 1.0, crest),
                event_band * mix(0.58, 1.0, crest) * 1.18
            );
        }
        default: {
            let body = smoothstep(0.28, 0.78, body_noise);
            density = slab * body;
        }
    }

    density = epu_saturate(density);

    var mix_w = epu_saturate(density * (0.65 + 0.35 * body_noise));
    var alpha_scale = 1.0;
    if (variant == ADVECT_VARIANT_BANK) {
        // BANK should read as a dense dark weather mass with a restrained lit rim,
        // not as another bright translucent sheet.
        mix_w = epu_saturate(0.06 + bank_rim * 0.34 + bank_veining * 0.12 - bank_core * 0.14);
        alpha_scale = mix(1.08, 1.28, epu_saturate(bank_core));
    } else if (variant == ADVECT_VARIANT_FRONT) {
        // FRONT should carry visible transported energy inside a larger wall mass,
        // without trying to become the primary body on its own.
        mix_w = epu_saturate(
            0.012
            + front_rim * 0.12
            + front_erosion * 0.04
            + front_crest * 0.11
            + front_event * 0.12
            - front_core * 0.11
        );
        alpha_scale = mix(1.16, 1.82, epu_saturate(front_core * 0.4 + front_billow * 0.18 + front_event * 0.42));
    }
    let rgb = mix(instr_color_b(instr), instr_color_a(instr), mix_w);
    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr) * intensity * region_w;
    return LayerSample(rgb, alpha * alpha_scale * density);
}
