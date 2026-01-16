// ============================================================================
// Mode 7: Rings (Portals / Tunnels / Vortex / Radar)
// ============================================================================
// w0: ring_count:u8 | thickness:u8 | center_falloff:u8 | family:u8
// w1: color_a (RGBA8)
// w2: color_b (RGBA8)
// w3: center_color (RGBA8)
// w4: spiral_twist:f16 (low16) | axis_oct16:u16 (high16)
// w5: phase:u16 (low16) | wobble:u16 (high16)
// w6: noise:u8 | dash:u8 | glow:u8 | seed:u8
fn sample_rings(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let ring_count_u8 = w0 & 0xFFu;
    let thickness_u8 = (w0 >> 8u) & 0xFFu;
    let center_falloff_u8 = (w0 >> 16u) & 0xFFu;
    let family = (w0 >> 24u) & 0xFFu;

    let ring_count = max(1u, ring_count_u8);

    let color_a = unpack_rgba8(data[offset + 1u]);
    let color_b = unpack_rgba8(data[offset + 2u]);
    let center_color = unpack_rgba8(data[offset + 3u]);

    let w4 = data[offset + 4u];
    let spiral_twist_deg = unpack2x16float(w4 & 0xFFFFu).x;
    let axis = unpack_octahedral_u16(w4 >> 16u);

    let w5 = data[offset + 5u];
    let phase01 = f32(w5 & 0xFFFFu) / 65536.0;
    let wobble01 = f32((w5 >> 16u) & 0xFFFFu) / 65535.0;

    let w6 = data[offset + 6u];
    let noise01 = f32(w6 & 0xFFu) / 255.0;
    let dash01 = f32((w6 >> 8u) & 0xFFu) / 255.0;
    let glow01 = f32((w6 >> 16u) & 0xFFu) / 255.0;
    let seed_u8 = (w6 >> 24u) & 0xFFu;

    // Seed derivation must not depend on phase (stability + loopability).
    let w5_no_phase = w5 & 0xFFFF0000u;
    let base_seed = hash_u32(w0 ^ w4 ^ w5_no_phase ^ data[offset + 1u] ^ data[offset + 2u] ^ data[offset + 3u]);
    let seed = select(hash_u32(base_seed ^ (seed_u8 * 0x9e3779b9u)), base_seed, seed_u8 == 0u);
    let seed01 = hash01_u32(seed ^ 0x243f6a88u);

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Radial distance proxy from axis: r = sin(theta/2) (monotonic in angle).
    let dot_axis = clamp(dot(axis, dir), -1.0, 1.0);
    var r = sqrt(max(0.0, 0.5 * (1.0 - dot_axis)));

    // Trig-free azimuth around axis (diamond angle).
    let b = basis_from_axis(axis);
    let p = vec2<f32>(dot(dir, b.t), dot(dir, b.b));
    let az = pseudo_angle01(p);
    let az_rot = fract(az + phase01);

    // Azimuth is ill-defined at the ring center (p≈0, r≈0). Fade azimuth-driven modulation
    // near the center pole to avoid visible "merging" artifacts.
    let az_w = smoothstep(0.02, 0.12, r);
    let az_rot_s = mix(seed01, az_rot, az_w);

    // Secondary domain warp (stable, loopable).
    if (wobble01 > 0.0) {
        let w = tri(az_rot_s * 2.0 + phase01 + seed01 * 3.0);
        r = clamp(r + w * wobble01 * 0.08 * (0.25 + 0.75 * r), 0.0, 1.0);
    }
    if (noise01 > 0.0) {
        let n = value_noise2(vec2<f32>(az_rot_s * 8.0, r * 8.0) + vec2<f32>(seed01 * 17.0, seed01 * 31.0));
        r = clamp(r + (n - 0.5) * noise01 * 0.08, 0.0, 1.0);
    }

    let twist_turns = spiral_twist_deg * (1.0 / 360.0);

    // Ring-space coordinate.
    var u = r * f32(ring_count);
    u = u + az_rot_s * twist_turns * f32(ring_count);

    // Family-specific motion.
    let travel = phase01 * f32(ring_count);
    if (family == 1u) {
        // Tunnel travel: rings move outward as phase increases.
        u = u - travel;
    } else if (family == 2u) {
        // Hypnotic: gentle breathing of ring position (loopable).
        u = u + tri(phase01 + seed01) * 0.25;
    }

    // Ring band (AA'd).
    let ring_i = i32(floor(u));
    let fu = fract(u);
    let d = abs(fu - 0.5);

    let t01 = f32(thickness_u8) / 255.0;
    var halfw = mix(0.006, 0.14, t01);
    if (family == 2u) {
        // Breathing thickness for op-art.
        let breathe = 0.85 + 0.15 * (0.5 + 0.5 * tri(phase01 + seed01));
        halfw = halfw * breathe;
    }

    let aa = fwidth(u) + 1e-6;
    let ring_mask = 1.0 - smoothstep(halfw, halfw + aa, d);

    // Alternate palette by ring ID (stable even for negative u).
    let mod2 = ((ring_i % 2) + 2) % 2;
    let is_a = mod2 == 0;
    let ring_color = select(color_b, color_a, is_a);

    var a = ring_mask * ring_color.a;
    var rgb = ring_color.rgb * a;

    // Dash/segmentation around azimuth.
    if (dash01 > 0.0) {
        let segs = mix(4.0, 64.0, dash01);
        let sd = abs(fract(az_rot_s * segs + seed01) - 0.5);
        let seg_aa = fwidth(az_rot_s * segs) + 1e-6;
        let w = mix(0.5, 0.06, dash01);
        let dash_mask = 1.0 - smoothstep(w, w + seg_aa, sd);
        let dash_mask_w = mix(1.0, dash_mask, az_w);
        a = a * dash_mask_w;
        rgb = rgb * dash_mask_w;
    }

    // Radar sweep (family 3): bright wedge driven by az vs phase.
    if (family == 3u) {
        let sweep_d = wrap_dist01(az, phase01);
        let sweep_w = 0.03;
        let sweep_aa = fwidth(sweep_d) + 1e-6;
        let sweep = (1.0 - smoothstep(sweep_w, sweep_w + sweep_aa, sweep_d)) * az_w;
        rgb = rgb * (0.6 + 1.8 * sweep);
    }

    // Center glow (uses alpha as coverage so it works under Alpha blend).
    if (center_falloff_u8 != 0u) {
        let fall = f32(center_falloff_u8) / 255.0;
        let radius = mix(0.02, 0.7, fall);
        let ra = fwidth(r) + 1e-6;
        var c = 1.0 - smoothstep(radius, radius + ra, r);
        if (family == 0u) {
            c = min(1.0, c * 1.35);
        }
        let ca = c * center_color.a;
        rgb = rgb + center_color.rgb * ca;
        a = max(a, ca * 0.75);
    }

    // Glow boosts RGB energy (not alpha).
    rgb = rgb * (1.0 + glow01 * 4.0);
    return vec4<f32>(rgb, a);
}
