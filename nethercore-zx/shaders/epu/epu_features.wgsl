// ============================================================================
// EPU FEATURE OPCODES (v2 - 128-bit instructions)
// High-frequency motifs: DECAL, GRID, SCATTER, FLOW
// All features are evaluated "sharp" and multiplied by a region mask weight.
//
// EPU v2 128-bit format for feature opcodes:
// | Opcode | color_a | color_b | intensity | param_a | param_b | param_c | param_d | direction |
// |--------|---------|---------|-----------|---------|---------|---------|---------|-----------|
// | DECAL  | Shape   | Glow    | Brightness| Shape   | Size    | Softness| Pulse   | Center    |
// | GRID   | Line    | Cross   | Brightness| Scale   | Width   | Rotation| --      | Normal    |
// | SCATTER| Point   | Variation| Brightness| Density | Size    | Twinkle | Seed    | --        |
// | FLOW   | Flow    | Secondary| Brightness| Scale   | Speed   | Octaves | Turb    | Flow dir  |
// ============================================================================

// ============================================================================
// DECAL - Sharp SDF Shape
// Packed fields (v2):
//   color_a: Shape/fill color (RGB24)
//   color_b: Glow/outline color (RGB24)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a[7:4]: Shape type (0=disk, 1=ring, 2=rect, 3=line)
//   param_a[3:0]: Edge softness (0..15 -> 0.001..0.05 rad)
//   param_b: Size (0..255 -> 0..0.5 rad)
//   param_c: Softness for glow (0..255 -> 0..0.2)
//   param_d: Pulse speed (0..255 -> 0..10)
//   direction: Shape center direction (oct-u16)
// ============================================================================

fn project_to_tangent(dir: vec3f, center: vec3f) -> vec2f {
    // Build an arbitrary tangent basis around center.
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));
    return vec2f(dot(dir, t), dot(dir, b));
}

fn box_sdf(p: vec2f, half_extents: vec2f) -> f32 {
    let d = abs(p) - half_extents;
    return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

fn eval_decal(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let center = decode_dir16(instr_dir16(instr));
    let angle = acos(epu_saturate(dot(dir, center)));

    let pa = instr_a(instr);
    let shape_type = (pa >> 4u) & 0xFu;
    let soft_q = pa & 0xFu;
    let softness = mix(0.001, 0.05, f32(soft_q) / 15.0);

    let size = u8_to_01(instr_b(instr)) * 0.5;

    var sdf: f32 = 0.0;
    switch shape_type {
        case 0u: { // DISK
            sdf = angle - size;
        }
        case 1u: { // RING
            sdf = abs(angle - size) - size * 0.2;
        }
        case 2u: { // RECT (on tangent plane)
            let uv = project_to_tangent(dir, center);
            sdf = box_sdf(uv, vec2f(size, size));
        }
        case 3u: { // LINE (tangent-plane vertical line)
            let uv = project_to_tangent(dir, center);
            sdf = abs(uv.x) - size * 0.1;
        }
        default: {
            sdf = angle - size;
        }
    }

    let edge = 1.0 - smoothstep(-softness, softness, sdf);

    // Glow effect using color_b and param_c for glow softness
    let glow_softness = u8_to_01(instr_c(instr)) * 0.2;
    let glow = smoothstep(glow_softness + softness, softness, sdf) * (1.0 - edge);

    // Pulse animation using param_d
    let speed = u8_to_01(instr_d(instr)) * 10.0;
    let anim = select(1.0, 0.6 + 0.4 * sin(time * speed), speed > 0.0);

    // color_a = shape/fill color, color_b = glow/outline color
    let fill_rgb = instr_color_a(instr);
    let glow_rgb = instr_color_b(instr);
    let rgb = fill_rgb * edge + glow_rgb * glow;

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, (edge + glow) * intensity * anim * region_w);
}

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
//   direction: Drift direction (reserved for future; current impl static)
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

    // Cell on direction sphere (cheap hash distribution).
    let cell = floor(dir * density);
    let h = hash3(cell + vec3f(seed));
    let point_offset = h.xyz * 2.0 - 1.0;
    var v = cell + point_offset * 0.5;
    if length(v) < 1e-5 {
        v = vec3f(1.0, 0.0, 0.0);
    }
    let point_dir = normalize(v);

    let dist = acos(epu_saturate(dot(dir, point_dir)));
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

// ============================================================================
// FLOW - Animated Noise / Streaks / Caustics
// Packed fields (v2):
//   color_a: Primary flow color (RGB24)
//   color_b: Secondary flow color (RGB24) - mixed based on pattern
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..16)
//   param_b: Speed (0..255 -> 0..2)
//   param_c[7:4]: Octaves (0..4)
//   param_c[3:0]: Pattern (0=noise, 1=streaks, 2=caustic)
//   param_d: Turbulence amount (0..255 -> 0..1)
//   direction: Flow direction (oct-u16)
// ============================================================================

fn epu_hash21(p: vec2f) -> f32 {
    let h = dot(p, vec2f(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn value_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = epu_hash21(i + vec2f(0.0, 0.0));
    let b = epu_hash21(i + vec2f(1.0, 0.0));
    let c = epu_hash21(i + vec2f(0.0, 1.0));
    let d = epu_hash21(i + vec2f(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f); // smoothstep
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y) * 2.0 - 1.0;
}

fn eval_flow(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let flow_dir = decode_dir16(instr_dir16(instr));
    let scale = mix(1.0, 16.0, u8_to_01(instr_a(instr)));
    let speed = u8_to_01(instr_b(instr)) * 2.0;

    let pc = instr_c(instr);
    let octaves = min((pc >> 4u) & 0xFu, 4u);
    let pattern_type = pc & 0xFu;

    // Turbulence from param_d - adds noise-based distortion to UV
    let turbulence = u8_to_01(instr_d(instr));

    // Base UV: cheap mapping (cylindrical).
    let uv0 = get_cyl_uv(dir);

    // Apply turbulence distortion
    var uv = uv0 * scale + flow_dir.xy * (time * speed);
    if turbulence > 0.001 {
        let turb_offset = vec2f(
            value_noise(uv0 * 4.0 + time * 0.5),
            value_noise(uv0 * 4.0 + vec2f(17.3, 31.7) + time * 0.5)
        ) * turbulence * 0.5;
        uv += turb_offset;
    }

    var pat: f32 = 0.0;
    var color_mix: f32 = 0.0; // For blending between color_a and color_b

    switch pattern_type {
        case 0u: { // NOISE
            var amp = 1.0;
            var freq = 1.0;
            for (var i = 0u; i < octaves; i++) {
                pat += value_noise(uv * freq) * amp;
                freq *= 2.0;
                amp *= 0.5;
            }
            pat = pat * 0.5 + 0.5;
            color_mix = pat;
        }
        case 1u: { // STREAKS
            let d = normalize(flow_dir);
            let streak_coord = dot(dir, d) * scale + time * speed;
            let perp = length(dir - d * dot(dir, d));
            pat = fract(streak_coord) * smoothstep(0.1, 0.0, perp);
            color_mix = fract(streak_coord * 0.5);
        }
        case 2u: { // CAUSTIC
            let p1 = sin(uv.x * 5.0 + time) * cos(uv.y * 5.0 + time * 0.7);
            let p2 = sin(uv.x * 7.0 - time * 0.8) * cos(uv.y * 6.0 + time * 0.5);
            pat = (p1 + p2) * 0.25 + 0.5;
            pat = smoothstep(0.4, 0.6, pat);
            color_mix = (p1 * 0.5 + 0.5);
        }
        default: {
            pat = value_noise(uv) * 0.5 + 0.5;
            color_mix = pat;
        }
    }

    // color_a = primary flow color, color_b = secondary flow color
    let flow_rgb = instr_color_a(instr);
    let secondary_rgb = instr_color_b(instr);
    let rgb = mix(flow_rgb, secondary_rgb, color_mix);

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, pat * intensity * region_w);
}

// ============================================================================
// LAYER DISPATCH
// ============================================================================

fn evaluate_layer(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
    regions: RegionWeights,
    time: f32
) -> LayerSample {
    let opcode = instr_opcode(instr);
    let region_mask = instr_region(instr);

    let is_feature = opcode >= OP_DECAL;
    let region_w = select(1.0, region_weight(regions, region_mask), is_feature);

    switch opcode {
        case OP_RAMP: { return eval_ramp(dir, instr, enc); }
        case OP_LOBE: { return eval_lobe(dir, instr, time); }
        case OP_BAND: { return eval_band(dir, instr, time); }
        case OP_FOG:  { return eval_fog(dir, instr); }
        case OP_DECAL:   { return eval_decal(dir, instr, region_w, time); }
        case OP_GRID:    { return eval_grid(dir, instr, region_w, time); }
        case OP_SCATTER: { return eval_scatter(dir, instr, region_w, time); }
        case OP_FLOW:    { return eval_flow(dir, instr, region_w, time); }
        default: { return LayerSample(vec3f(0.0), 0.0); }
    }
}
