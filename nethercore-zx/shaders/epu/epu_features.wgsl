// ============================================================================
// EPU FEATURE OPCODES
// High-frequency motifs: DECAL, GRID, SCATTER, FLOW
// All features are evaluated "sharp" and multiplied by a region mask weight.
// ============================================================================

// ============================================================================
// DECAL - Sharp SDF Shape
// Packed fields:
//   color_index: Shape color
//   intensity: Brightness (0..255 -> 0..1)
//   param_a[7:4]: Shape type (0=disk, 1=ring, 2=rect, 3=line)
//   param_a[3:0]: Edge softness (0..15 -> 0.001..0.05 rad)
//   param_b: Size (0..255 -> 0..0.5 rad)
//   param_c: Pulse speed (0..255 -> 0..10)
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
    lo: u32,
    hi: u32,
    region_w: f32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let center = decode_dir16(instr_dir16(lo, hi));
    let angle = acos(saturate(dot(dir, center)));

    let pa = instr_a(lo, hi);
    let shape_type = (pa >> 4u) & 0xFu;
    let soft_q = pa & 0xFu;
    let softness = mix(0.001, 0.05, f32(soft_q) / 15.0);

    let size = u8_to_01(instr_b(lo, hi)) * 0.5;

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

    let speed = u8_to_01(instr_c(lo, hi)) * 10.0;
    let anim = select(1.0, 0.6 + 0.4 * sin(time * speed), speed > 0.0);

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, edge * intensity * anim * region_w);
}

// ============================================================================
// GRID - Repeating Lines / Panels
// Packed fields:
//   color_index: Line color
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..64)
//   param_b: Thickness (0..255 -> 0.001..0.1)
//   param_c[7:4]: Pattern (0=stripes, 1=grid, 2=checker)
//   param_c[3:0]: Scroll speed (0..15 -> 0..2)
//   direction: Orientation (reserved for future expansion)
// ============================================================================

fn get_cyl_uv(dir: vec3f) -> vec2f {
    let u = atan2(dir.x, dir.z) / TAU;        // [-0.5..0.5]
    let v = dir.y * 0.5 + 0.5;                // [0..1]
    return vec2f(u, v);
}

fn eval_grid(
    dir: vec3f,
    lo: u32,
    hi: u32,
    region_w: f32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let scale = mix(1.0, 64.0, u8_to_01(instr_a(lo, hi)));
    let thickness = mix(0.001, 0.1, u8_to_01(instr_b(lo, hi)));

    let pc = instr_c(lo, hi);
    let pattern_type = (pc >> 4u) & 0xFu;
    let scroll_q = pc & 0xFu;
    let scroll_speed = (f32(scroll_q) / 15.0) * 2.0;

    let uv0 = get_cyl_uv(dir);
    let uv = uv0 + vec2f(time * scroll_speed, 0.0);

    var pat: f32 = 0.0;
    switch pattern_type {
        case 0u: { // STRIPES
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
        }
        case 1u: { // GRID
            let fx = abs(fract(uv.x * scale) - 0.5);
            let fy = abs(fract(uv.y * scale) - 0.5);
            pat = 1.0 - step(thickness, fx) * step(thickness, fy);
        }
        case 2u: { // CHECKER
            let cell = floor(vec2f(uv.x, uv.y) * scale);
            pat = fract((cell.x + cell.y) * 0.5) * 2.0;
        }
        default: {
            let fx = abs(fract(uv.x * scale) - 0.5);
            pat = 1.0 - step(thickness, fx);
        }
    }

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, pat * intensity * region_w);
}

// ============================================================================
// SCATTER - Point Field (Stars / Dust / Windows)
// Packed fields:
//   color_index: Point color
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Density (0..255 -> 1..256)
//   param_b: Point size (0..255 -> 0.001..0.05 rad)
//   param_c[7:4]: Twinkle amount (0..15 -> 0..1)
//   param_c[3:0]: Seed (0..15)
//   direction: Drift direction (reserved for future; current impl static)
// ============================================================================

fn hash3(p: vec3f) -> vec4f {
    var p4 = fract(vec4f(p.xyzx) * vec4f(0.1031, 0.1030, 0.0973, 0.1099));
    p4 += dot(p4, p4.wzxy + 33.33);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

fn eval_scatter(
    dir: vec3f,
    lo: u32,
    hi: u32,
    region_w: f32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let density = mix(1.0, 256.0, u8_to_01(instr_a(lo, hi)));
    let size = mix(0.001, 0.05, u8_to_01(instr_b(lo, hi)));

    let pc = instr_c(lo, hi);
    let twinkle = f32((pc >> 4u) & 0xFu) / 15.0;
    let seed = f32(pc & 0xFu);

    // Cell on direction sphere (cheap hash distribution).
    let cell = floor(dir * density);
    let h = hash3(cell + vec3f(seed));
    let point_offset = h.xyz * 2.0 - 1.0;
    var v = cell + point_offset * 0.5;
    if length(v) < 1e-5 {
        v = vec3f(1.0, 0.0, 0.0);
    }
    let point_dir = normalize(v);

    let dist = acos(saturate(dot(dir, point_dir)));
    let point = smoothstep(size, size * 0.3, dist);

    let tw = select(1.0, (0.5 + 0.5 * sin(h.w * TAU + time * 3.0)), twinkle > 0.001);

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, point * intensity * mix(1.0, tw, twinkle) * region_w);
}

// ============================================================================
// FLOW - Animated Noise / Streaks / Caustics
// Packed fields:
//   color_index: Flow color
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..16)
//   param_b: Speed (0..255 -> 0..2)
//   param_c[7:4]: Octaves (0..4)
//   param_c[3:0]: Pattern (0=noise, 1=streaks, 2=caustic)
//   direction: Flow direction (oct-u16)
// ============================================================================

fn hash21(p: vec2f) -> f32 {
    let h = dot(p, vec2f(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn value_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash21(i + vec2f(0.0, 0.0));
    let b = hash21(i + vec2f(1.0, 0.0));
    let c = hash21(i + vec2f(0.0, 1.0));
    let d = hash21(i + vec2f(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f); // smoothstep
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y) * 2.0 - 1.0;
}

fn eval_flow(
    dir: vec3f,
    lo: u32,
    hi: u32,
    region_w: f32,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let flow_dir = decode_dir16(instr_dir16(lo, hi));
    let scale = mix(1.0, 16.0, u8_to_01(instr_a(lo, hi)));
    let speed = u8_to_01(instr_b(lo, hi)) * 2.0;

    let pc = instr_c(lo, hi);
    let octaves = min((pc >> 4u) & 0xFu, 4u);
    let pattern_type = pc & 0xFu;

    // Base UV: cheap mapping (cylindrical).
    let uv0 = get_cyl_uv(dir);
    let uv = uv0 * scale + flow_dir.xy * (time * speed);

    var pat: f32 = 0.0;
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
        }
        case 1u: { // STREAKS
            let d = normalize(flow_dir);
            let streak_coord = dot(dir, d) * scale + time * speed;
            let perp = length(dir - d * dot(dir, d));
            pat = fract(streak_coord) * smoothstep(0.1, 0.0, perp);
        }
        case 2u: { // CAUSTIC
            let p1 = sin(uv.x * 5.0 + time) * cos(uv.y * 5.0 + time * 0.7);
            let p2 = sin(uv.x * 7.0 - time * 0.8) * cos(uv.y * 6.0 + time * 0.5);
            pat = (p1 + p2) * 0.25 + 0.5;
            pat = smoothstep(0.4, 0.6, pat);
        }
        default: {
            pat = value_noise(uv) * 0.5 + 0.5;
        }
    }

    let rgb = palette_lookup(palette, instr_color(lo, hi));
    let intensity = u8_to_01(instr_intensity(lo, hi));
    return LayerSample(rgb, pat * intensity * region_w);
}

// ============================================================================
// LAYER DISPATCH
// ============================================================================

fn evaluate_layer(
    dir: vec3f,
    lo: u32,
    hi: u32,
    enc: EnclosureConfig,
    regions: RegionWeights,
    time: f32,
    palette: ptr<storage, array<vec4f>, read>
) -> LayerSample {
    let opcode = instr_opcode(lo, hi);
    let region_mask = instr_region(lo, hi);

    let is_feature = opcode >= OP_DECAL;
    let region_w = select(1.0, region_weight(regions, region_mask), is_feature);

    switch opcode {
        case OP_RAMP: { return eval_ramp(dir, lo, hi, enc, palette); }
        case OP_LOBE: { return eval_lobe(dir, lo, hi, time, palette); }
        case OP_BAND: { return eval_band(dir, lo, hi, time, palette); }
        case OP_FOG:  { return eval_fog(dir, lo, hi, palette); }
        case OP_DECAL:   { return eval_decal(dir, lo, hi, region_w, time, palette); }
        case OP_GRID:    { return eval_grid(dir, lo, hi, region_w, time, palette); }
        case OP_SCATTER: { return eval_scatter(dir, lo, hi, region_w, time, palette); }
        case OP_FLOW:    { return eval_flow(dir, lo, hi, region_w, time, palette); }
        default: { return LayerSample(vec3f(0.0), 0.0); }
    }
}
