// @epu_meta_begin
// opcode = 0x0B
// name = FLOW
// kind = radiance
// variants = []
// domains = []
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=1.0, max=16.0 }
// field param_b = { label="turbulence", map="u8_01" }
// field param_c = { label="oct+pat", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// FLOW - Animated Noise / Streaks / Caustics
// Packed fields:
//   color_a: Primary flow color (RGB24)
//   color_b: Secondary flow color (RGB24) - mixed based on pattern
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Scale (0..255 -> 1..16)
//   param_b: Turbulence amount (0..255 -> 0..1)
//   param_c[7:4]: Octaves (0..4)
//   param_c[3:0]: Pattern (0=noise, 1=streaks, 2=caustic)
//   param_d: Phase (0..255 -> 0..1)
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
    bounds_dir: vec3f,
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

    // Turbulence from param_b - adds noise-based distortion to UV
    let turbulence = u8_to_01(instr_b(instr));

    // NOTE: FLOW previously used 2D UV parameterizations (cylindrical / octahedral),
    // which necessarily introduce seams. Those seams become very noticeable once the
    // pattern is animated, especially for smooth trig patterns like caustics.
    //
    // Use a 3D domain based on the direction vector instead. This is continuous on the sphere,
    // so it eliminates hard seams for animated environments.
    var p = dir * scale;
    let phase01 = epu_loop_phase01(instr_d(instr));
    let t = phase01 * TAU;
    let phase_circle = epu_phase_circle(phase01);

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
            // Animate by scrolling the 3D sample position along the flow direction.
            let flow_basis = epu_axis_basis(flow_dir);
            let p_anim = p
                + flow_basis[1] * (phase_circle.x * 0.22)
                + flow_basis[0] * (phase_circle.y * 0.1);
            var amp = 1.0;
            var freq = 1.0;
            var sum = 0.0;
            var norm = 0.0;
            for (var i = 0u; i < octaves; i++) {
                sum += value_noise3(p_anim * freq) * amp;
                norm += amp;
                freq *= 2.0;
                amp *= 0.5;
            }
            let n = sum / max(norm, 1e-6);
            pat = n * 0.5 + 0.5;
            color_mix = pat;
        }
        case 1u: { // STREAKS
            // Project streak coordinates onto a floor plane derived from the active bounds up
            // vector. This keeps floor lanes straight in local space instead of wrapping into
            // spherical contour arcs on the environment map.
            let bounds_len2 = dot(bounds_dir, bounds_dir);
            let floor_up = normalize(select(vec3f(0.0, 1.0, 0.0), bounds_dir, bounds_len2 > 1e-6));
            let basis_hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(floor_up.y) > 0.9);
            let floor_right = normalize(cross(basis_hint, floor_up));
            let floor_forward = cross(floor_up, floor_right);

            let flow_flat = flow_dir - floor_up * dot(flow_dir, floor_up);
            let flow_flat_len2 = dot(flow_flat, flow_flat);
            let along_axis = normalize(select(floor_forward, flow_flat, flow_flat_len2 > 1e-4));
            let across_axis = cross(floor_up, along_axis);

            let floor_view = max(-dot(dir, floor_up), 0.0);
            let floor_proj = max(floor_view, 0.12);
            let floor_fade = smoothstep(0.02, 0.12, floor_view);

            var floor_uv = vec2f(
                dot(dir, across_axis),
                dot(dir, along_axis)
            ) / floor_proj;

            if turbulence > 0.001 {
                let turb_freq = 1.5 + scale * 0.2;
                let turb_uv = floor_uv * turb_freq;
                let turb = vec2f(
                    value_noise(turb_uv + vec2f(13.7, 7.1)),
                    value_noise(turb_uv + vec2f(29.3, 17.9))
                );
                floor_uv += turb * turbulence * 0.18;
            }

            let lane_freq = 6.0 + scale * 1.5;
            // Lower base frequency + per-lane variation keeps rain from collapsing into a
            // perfectly regular dot-grid pattern.
            let seg_freq = 1.8 + scale * 0.55;

            let across = floor_uv.x * lane_freq;
            let lane = floor(across);

            // Per-lane variation (deterministic, cheap).
            let h0 = epu_hash21(vec2f(lane, f32(scale_i) * 17.0));
            let h1 = epu_hash21(vec2f(lane, f32(octaves) * 23.0));

            // Thin line mask (lanes).
            let width = mix(0.05, 0.11, h0);
            let xf = abs(fract(across + h0) - 0.5);
            let line = 1.0 - smoothstep(width, width * 1.7, xf);

            // Periodic droplet modulation along the projected flow axis. Use a cosine bump so
            // the wrap boundary is always zero (no visible "cut off" points).
            let seg_freq_lane = seg_freq * mix(0.65, 1.35, h1);
            let seg_cycles = 1.0 + floor(h0 * 3.0);
            let along = floor_uv.y * seg_freq_lane - phase01 * seg_cycles + h1;
            let phase = fract(along);
            let bump = 0.5 - 0.5 * cos(phase * TAU);
            // Wider bumps read as streaks instead of pinpoint dots.
            let seg = pow(bump, mix(0.85, 2.1, h1));

            // Slight lateral wobble so streaks aren't perfectly rigid.
            let wobble = floor_uv.x * (1.5 + h1 * 4.0) + floor_uv.y * 0.35;
            pat = line * seg * (0.88 + 0.12 * sin(wobble + t * 2.0)) * floor_fade;
            color_mix = h0;
        }
        case 2u: { // CAUSTIC
            // Build a basis around flow_dir so caustics don't lock to world axes.
            // This avoids mirrored vertical banding on the reflection sphere.
            let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(flow_dir.y) > 0.9);
            let t_axis = normalize(cross(up, flow_dir));
            let b_axis = normalize(cross(flow_dir, t_axis));
            let q = vec3f(dot(p, t_axis), dot(p, b_axis), dot(p, flow_dir)) * 2.0;
            let p1 = sin(q.x * 1.7 + t) * cos(q.z * 1.9 + t * 2.0);
            let p2 = sin(q.x * 2.3 - t * 3.0) * cos(q.y * 2.0 + t);
            pat = (p1 + p2) * 0.25 + 0.5;
            // Keep only higher-energy crest events so CAUSTIC doesn't collapse
            // into a broad midtone slab or shell.
            pat = smoothstep(0.62, 0.82, pat);
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
