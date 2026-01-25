// ============================================================================
// FLOW - Animated Noise / Streaks / Caustics
// Packed fields:
//   color_a: Primary flow color (RGB24)
//   color_b: Secondary flow color (RGB24) - mixed based on pattern
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..16)
//   param_b: Phase (0..255 -> 0..1)
//   param_c[7:4]: Octaves (0..4)
//   param_c[3:0]: Pattern (0=noise, 1=streaks, 2=caustic)
//   param_d: Turbulence amount (0..255 -> 0..1)
//   direction: Flow direction (oct-u16)
//   alpha_a: Flow alpha (0..15 -> 0..1)
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

fn epu_hash31(p: vec3f) -> f32 {
    let h = dot(p, vec3f(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

// 3D value noise (trilinear), stable and seam-free for directional domains.
// Returns [-1, 1].
fn value_noise3(p: vec3f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = epu_hash31(i + vec3f(0.0, 0.0, 0.0));
    let b = epu_hash31(i + vec3f(1.0, 0.0, 0.0));
    let c = epu_hash31(i + vec3f(0.0, 1.0, 0.0));
    let d = epu_hash31(i + vec3f(1.0, 1.0, 0.0));
    let e = epu_hash31(i + vec3f(0.0, 0.0, 1.0));
    let f1 = epu_hash31(i + vec3f(1.0, 0.0, 1.0));
    let g = epu_hash31(i + vec3f(0.0, 1.0, 1.0));
    let h = epu_hash31(i + vec3f(1.0, 1.0, 1.0));

    let ab = mix(a, b, u.x);
    let cd = mix(c, d, u.x);
    let ef = mix(e, f1, u.x);
    let gh = mix(g, h, u.x);
    let abcd = mix(ab, cd, u.y);
    let efgh = mix(ef, gh, u.y);

    return mix(abcd, efgh, u.z) * 2.0 - 1.0;
}

fn eval_flow(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let flow_dir16 = instr_dir16(instr);
    let flow_dir = select(vec3f(0.0, -1.0, 0.0), decode_dir16(flow_dir16), flow_dir16 != 0u);

    // Quantize to an integer frequency in [1, 16]. This avoids visible seams when using
    // periodic functions around the azimuth wrap and keeps patterns stable across the sphere.
    let scale_i = 1u + (instr_a(instr) * 15u) / 255u;
    let scale = f32(scale_i);

    let pc = instr_c(instr);
    let octaves = min((pc >> 4u) & 0xFu, 4u);
    let pattern_type = pc & 0xFu;

    // Turbulence from param_d - adds noise-based distortion to UV
    let turbulence = u8_to_01(instr_d(instr));

    // NOTE: FLOW previously used 2D UV parameterizations (cylindrical / octahedral),
    // which necessarily introduce seams. Those seams become very noticeable once the
    // pattern is animated, especially for smooth trig patterns like caustics.
    //
    // Use a 3D domain based on the direction vector instead. This is continuous on the sphere,
    // so it eliminates hard seams for animated environments.
    var p = dir * scale;
    let t = u8_to_01(instr_b(instr)) * TAU;

    // Optional turbulence: add a small vector-valued distortion.
    if turbulence > 0.001 {
        let p2 = p * 2.0;
        let wobble = vec3f(
            value_noise3(p2 + vec3f(0.0, 0.0, 0.0)),
            value_noise3(p2 + vec3f(17.3, 31.7, 0.0)),
            value_noise3(p2 + vec3f(41.0, 12.0, 0.0))
        );
        p += wobble * turbulence * 0.5;
    }

    var pat: f32 = 0.0;
    var color_mix: f32 = 0.0; // For blending between color_a and color_b

    switch pattern_type {
        case 0u: { // NOISE
            var amp = 1.0;
            var freq = 1.0;
            var sum = 0.0;
            var norm = 0.0;
            for (var i = 0u; i < octaves; i++) {
                sum += value_noise3(p * freq) * amp;
                norm += amp;
                freq *= 2.0;
                amp *= 0.5;
            }
            let n = sum / max(norm, 1e-6);
            pat = n * 0.5 + 0.5;
            color_mix = pat;
        }
        case 1u: { // STREAKS
            // Rain / particle streaks across the whole environment.
            //
            // Generate "lanes" in a plane perpendicular to `flow_dir`, and animate droplets
            // along the flow axis using the 3D domain `p` (so motion is seam-free).
            let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(flow_dir.y) > 0.9);
            let t_axis = normalize(cross(up, flow_dir));
            let b_axis = normalize(cross(flow_dir, t_axis));

            let lane_freq = 10.0 + scale * 4.0;
            // Lower base frequency + per-lane variation keeps rain from collapsing into a
            // perfectly regular dot-grid pattern.
            let seg_freq = 3.0 + scale * 1.6;

            let across = dot(dir, t_axis) * lane_freq;
            let lane = floor(across);

            // Per-lane variation (deterministic, cheap).
            let h0 = epu_hash21(vec2f(lane, f32(scale_i) * 17.0));
            let h1 = epu_hash21(vec2f(lane, f32(octaves) * 23.0));

            // Thin line mask (lanes).
            let width = mix(0.035, 0.085, h0);
            let xf = abs(fract(across + h0) - 0.5);
            let line = 1.0 - smoothstep(width, width * 1.8, xf);

            // Periodic droplet modulation along the flow axis. Use a cosine bump so the
            // wrap boundary is always zero (no visible "cut off" points).
            let seg_freq_lane = seg_freq * mix(0.65, 1.35, h1);
            let along = dot(p, flow_dir) * seg_freq_lane + h1;
            let phase = fract(along);
            let bump = 0.5 - 0.5 * cos(phase * TAU);
            // Wider bumps read as streaks instead of pinpoint dots.
            let seg = pow(bump, mix(0.8, 2.2, h1));

            // Slight lateral wobble so streaks aren't perfectly rigid.
            let wobble = dot(dir, b_axis) * (2.0 + h1 * 6.0);
            pat = line * seg * (0.85 + 0.15 * sin(wobble + t * 1.5));
            color_mix = h0;
        }
        case 2u: { // CAUSTIC
            let q = p * 2.0;
            let p1 = sin(q.x * 1.7 + t) * cos(q.z * 1.9 + t * 0.7);
            let p2 = sin(q.x * 2.3 - t * 0.8) * cos(q.y * 2.0 + t * 0.5);
            pat = (p1 + p2) * 0.25 + 0.5;
            pat = smoothstep(0.45, 0.65, pat);
            color_mix = p1 * 0.5 + 0.5;
        }
        default: {
            pat = value_noise3(p) * 0.5 + 0.5;
            color_mix = pat;
        }
    }

    // color_a = primary flow color, color_b = secondary flow color
    let flow_rgb = instr_color_a(instr);
    let secondary_rgb = instr_color_b(instr);
    let rgb = mix(flow_rgb, secondary_rgb, color_mix);

    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr);
    return LayerSample(rgb, pat * intensity * alpha * region_w);
}
