// ============================================================================
// Mode 3: Shared Helpers
// ============================================================================

// Looping value noise for stable, loopable 1D shaping (used by silhouette).
fn looping_value_noise(t: f32, period: u32, seed: u32) -> f32 {
    let scaled = t * f32(period);
    let i = u32(floor(scaled)) % period;
    let i_next = (i + 1u) % period;
    let f = fract(scaled);

    let seed_offset = seed * 7919u;
    let a = hash11(i + seed_offset);
    let b = hash11(i_next + seed_offset);
    let smooth_t = f * f * (3.0 - 2.0 * f);  // smoothstep interpolation
    return mix(a, b, smooth_t);
}

// ============================================================================
// Mode 3: Silhouette (Mountains / City / Forest / Waves)
// ============================================================================
// w0: family:u8 | jaggedness:u8 | layer_count:u8 | parallax_rate:u8
// w1: color_near (RGBA8)
// w2: color_far (RGBA8)
// w3: sky_zenith (RGBA8)
// w4: sky_horizon (RGBA8)
// w5: seed:u32 (0 = derive from packed words)
// w6: phase:u16 (low) | fog:u8 | wind:u8
fn sample_silhouette(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let family = w0 & 0xFFu;
    let jag = f32((w0 >> 8u) & 0xFFu) / 255.0;
    let layer_count = clamp((w0 >> 16u) & 0xFFu, 1u, 3u);
    let parallax_rate = f32((w0 >> 24u) & 0xFFu) / 255.0;

    let color_near = unpack_rgba8(data[offset + 1u]);
    let color_far = unpack_rgba8(data[offset + 2u]);
    let sky_zenith = unpack_rgba8(data[offset + 3u]);
    let sky_horizon = unpack_rgba8(data[offset + 4u]);

    let seed_in = data[offset + 5u];
    let w6 = data[offset + 6u];
    // Seed derivation must not depend on phase (stability + loopability).
    let w6_no_phase = w6 & 0xFFFF0000u;
    let seed = select(
        hash_u32(w0 ^ data[offset + 1u] ^ data[offset + 2u] ^ data[offset + 3u] ^ data[offset + 4u] ^ w6_no_phase),
        seed_in,
        seed_in != 0u,
    );

    let phase01 = f32(w6 & 0xFFFFu) / 65536.0;
    let fog_base = f32((w6 >> 16u) & 0xFFu) / 255.0;
    let wind = f32((w6 >> 24u) & 0xFFu) / 255.0;

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Sky gradient backdrop.
    let sky_t = saturate(dir.y * 0.5 + 0.5);
    let sky = mix(sky_horizon, sky_zenith, sky_t);

    // Seam-free azimuth parameter from xz (no atan2).
    let u0 = pseudo_angle01(vec2<f32>(dir.x, dir.z));

    // Loopable haze breathing.
    let fog = clamp(fog_base * (0.9 + 0.1 * tri(phase01 + 0.17)), 0.0, 1.0);

    var out_rgb = sky.rgb;
    var out_a = sky.a;

    // Composite internal depth layers from far -> near (bounded ≤ 3).
    for (var idx: i32 = 2; idx >= 0; idx = idx - 1) {
        if (u32(idx) >= layer_count) {
            continue;
        }

        let denom = max(1.0, f32(layer_count - 1u));
        let depth = f32(idx) / denom; // 0 near, 1 far

        // Loopable drift per layer (phase-driven; no shimmer).
        let drift = tri(phase01 + f32(idx) * 0.13) * (0.015 + 0.02 * f32(idx)) * parallax_rate;
        var u = fract(u0 + drift);

        // Optional wind sway (family-specific).
        if (family == 2u || family == 3u) {
            u = fract(u + tri(phase01 + u0 * 2.0 + f32(idx) * 0.31) * wind * 0.02);
        }

        // Base horizon height for this depth slice.
        let base = -0.05 - depth * 0.22 * parallax_rate;

        var h = 0.0;

        if (family == 1u) {
            // City skyline: quantized block heights + spires at high jaggedness.
            let blocks = u32(mix(12.0, 72.0, jag));
            let s = u * f32(blocks);
            let bid = i32(floor(s));
            let bseed = hash_u32(bitcast<u32>(bid) ^ seed ^ (u32(idx) * 0x9e3779b9u));
            let raw = hash01_u32(bseed);
            let steps = mix(3.0, 12.0, jag);
            let q = floor(raw * steps) / steps;
            let spire = smoothstep(0.82, 1.0, raw) * smoothstep(0.6, 1.0, jag);
            let amp = mix(0.10, 0.32, jag) * (1.0 - depth * 0.35);
            h = base + (q + spire * 0.35) * amp;
        } else if (family == 2u) {
            // Forest canopy: smooth bumps + toothy spikes at high jaggedness.
            let period = u32(mix(8.0, 36.0, jag)) + u32(idx) * 3u;
            let n0 = looping_value_noise(u, period, seed + u32(idx) * 1234u);
            let tooth = tri(u * f32(period) * 2.0 + hash01_u32(seed) * 4.0) * 0.5 + 0.5;
            let spikes = mix(0.0, tooth, jag);
            let amp = mix(0.10, 0.28, jag) * (1.0 - depth * 0.35);
            h = base + (n0 * 0.6 + spikes * 0.4) * amp;
        } else if (family == 3u) {
            // Waves / coral: triangle crests + seeded distortion.
            let freq = mix(4.0, 24.0, jag) + f32(idx) * 2.0;
            let w = tri(u * freq + phase01 * 0.35) * 0.5 + 0.5;
            let cusp = mix(w, w * w, jag);
            let amp = mix(0.08, 0.22, jag) * (1.0 - depth * 0.35);
            h = base + cusp * amp;
        } else {
            // Mountains: ridged value-noise with peak sharpening.
            let period = u32(mix(8.0, 48.0, jag)) + u32(idx) * 4u;
            let n0 = looping_value_noise(u, period, seed + u32(idx) * 7919u);
            let n1 = looping_value_noise(u * 2.0, period * 2u, seed + u32(idx) * 7919u + 1000u);
            let n = n0 * 0.65 + n1 * 0.35;
            let ridged = 1.0 - abs(n * 2.0 - 1.0);
            let peaks = mix(ridged, ridged * ridged, jag);
            let amp = mix(0.10, 0.34, jag) * (1.0 - depth * 0.35);
            h = base + peaks * amp;
        }

        // Coverage mask with derivative AA.
        let edge = dir.y - h;
        // Only AA in the vertical dimension; horizontal discontinuities in `h` (e.g. city block
        // steps) can otherwise produce faint vertical "bands" above silhouettes.
        let aa = fwidth(dir.y) + 1e-5;
        let mask = 1.0 - smoothstep(0.0, aa, edge);

        var layer_col = mix(color_near, color_far, depth);

        // Fog: fade far layers toward sky, reduce alpha.
        let fog_layer = fog * depth;
        layer_col = vec4<f32>(mix(layer_col.rgb, sky.rgb, fog_layer), layer_col.a);
        let layer_a = mask * layer_col.a * (1.0 - fog_layer * 0.85);

        // Lighting safety: always leak a tiny bit of sky into very dark silhouettes at high fog.
        layer_col = vec4<f32>(mix(layer_col.rgb, sky.rgb, fog * 0.15), layer_col.a);

        out_rgb = mix(out_rgb, layer_col.rgb, layer_a);
        out_a = out_a + layer_a * (1.0 - out_a);
    }

    return vec4<f32>(out_rgb, out_a);
}
