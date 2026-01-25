// ============================================================================
// DECAL - Sharp SDF Shape
// Packed fields:
//   color_a: Shape/fill color (RGB24)
//   color_b: Glow/outline color (RGB24)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a[7:4]: Shape type (0=disk, 1=ring, 2=rect, 3=line)
//   param_a[3:0]: Edge softness (0..15 -> 0.001..0.05 rad)
//   param_b: Size (0..255 -> 0..0.5 rad)
//   param_c: Softness for glow (0..255 -> 0..0.2)
//   param_d: Phase (0..255 -> 0..1)
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
    region_w: f32
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
    let glow_softness = max(u8_to_01(instr_c(instr)) * 0.2, 0.0005);
    let glow = smoothstep(glow_softness + softness, softness, sdf) * (1.0 - edge);

    let phase = u8_to_01(instr_d(instr));
    let anim = 1.0 + 0.25 * sin(phase * TAU);

    // color_a = shape/fill color, color_b = glow/outline color
    // alpha_a = fill alpha, alpha_b = glow alpha
    let fill_rgb = instr_color_a(instr);
    let glow_rgb = instr_color_b(instr);
    let fill_alpha = instr_alpha_a_f32(instr);
    let glow_alpha = instr_alpha_b_f32(instr);
    let rgb = fill_rgb * edge * fill_alpha + glow_rgb * glow * glow_alpha;

    let intensity = u8_to_01(instr_intensity(instr));
    return LayerSample(rgb, (edge * fill_alpha + glow * glow_alpha) * intensity * anim * region_w);
}
