// ============================================================================
// Mode 6: Veil (Forest / Pillars / Drapes / Shards)
// ============================================================================
// w0: family:u8 | density:u8 | width:u8 | taper:u8
// w1: curvature:u8 | edge_soft:u8 | height_min:u8 | height_max:u8
// w2: color_near (RGBA8)
// w3: color_far (RGBA8)
// w4: glow:u8 | parallax:u8 | reserved:u16
// w5: axis_oct16 (low16) | phase:u16 (high16)
// w6: seed:u32 (0 = derive from packed words)
fn veil_eval_slice(
    family: u32,
    uv: vec2<f32>,
    freq: f32,
    scroll: f32,
    width_u8: u32,
    taper_mul: f32,
    curv_amp: f32,
    phase01: f32,
    seed: u32,
    edge_soft01: f32,
    height_gate: f32,
    color_near: vec4<f32>,
    color_far: vec4<f32>,
    depth01: f32,
    weight: f32,
) -> vec4<f32> {
    let stripe_u = uv.x;
    let u01 = fract(stripe_u + scroll);
    let s0 = u01 * freq;
    let i0 = floor(s0);
    let i1 = i0 + 1.0;

    // Per-stripe offsets (stable, anchored to stripe IDs).
    let freq_u = u32(freq);
    let stripe_id0 = u32(i0) % freq_u;
    let stripe_id1 = u32(i1) % freq_u;
    let off0 = hash01_u32(hash_u32(stripe_id0 ^ seed ^ 0x9e3779b9u));
    let off1 = hash01_u32(hash_u32(stripe_id1 ^ seed ^ 0x9e3779b9u));

    let choose1 = fract(s0) >= 0.5;
    let off = select(off0, off1, choose1);

    let sway = tri(phase01 + off);
    let static_bend = off - 0.5;
    let sway_bias = select(0.55, 0.8, family == 1u || family == 3u);

    // Curvature displacement uses the secondary coordinate (uv.y).
    let y = uv.y * 2.0 - 1.0;
    let s = s0 + curv_amp * y * mix(static_bend, sway, sway_bias);

    // Distance to stripe center in stripe units.
    var d = abs(fract(s) - 0.5);

    // Optional center jitter (Families 0 & 2 only; bounded 2-center eval).
    if (family == 0u || family == 2u) {
        let jitter_amp = select(0.06, 0.10, family == 2u);
        let j0 = (off0 - 0.5) * jitter_amp;
        let j1 = (off1 - 0.5) * jitter_amp;
        let c0 = i0 + 0.5 + j0;
        let c1 = i1 + 0.5 + j1;
        d = min(abs(s - c0), abs(s - c1));
    }

    // Width in stripe units: u8 -> [~0.005..0.22] then taper.
    let w01 = f32(width_u8) / 255.0;
    var halfw = mix(0.005, 0.22, w01) * taper_mul;
    halfw = max(0.002, halfw);

    // Edge AA bias.
    let aa = (fwidth(d) + 1e-5) * (1.0 + edge_soft01 * 4.0);

    // Coverage.
    var a = 1.0 - smoothstep(halfw, halfw + aa, d);

    // Family 2: pointier profile.
    if (family == 2u) {
        a = a * a;
    }

    // Family 3: softer + lighter (target ~0.6× coverage).
    if (family == 3u) {
        a = a * 0.6;
        let shimmer = 0.75 + 0.25 * tri(phase01 + y * 0.25 + off);
        a = a * shimmer;
    }

    // Depth palette and weight.
    let col = mix(color_near, color_far, depth01);
    a = clamp(a * col.a * height_gate * weight, 0.0, 1.0);
    let rgb = col.rgb * a;
    return vec4<f32>(rgb, a);
}

fn sample_veil(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let family = w0 & 0xFFu;
    let density_u8 = (w0 >> 8u) & 0xFFu;
    let width_u8 = (w0 >> 16u) & 0xFFu;
    let taper_u8 = (w0 >> 24u) & 0xFFu;

    if (density_u8 == 0u) {
        return vec4<f32>(0.0);
    }

    let w1 = data[offset + 1u];
    let curvature_u8 = w1 & 0xFFu;
    let edge_soft_u8 = (w1 >> 8u) & 0xFFu;
    let height_min_u8 = (w1 >> 16u) & 0xFFu;
    let height_max_u8 = (w1 >> 24u) & 0xFFu;

    let color_near = unpack_rgba8(data[offset + 2u]);
    let color_far = unpack_rgba8(data[offset + 3u]);

    let w4 = data[offset + 4u];
    let glow01 = f32(w4 & 0xFFu) / 255.0;
    let parallax_u8 = (w4 >> 8u) & 0xFFu;
    let parallax01 = f32(parallax_u8) / 255.0;

    let w5 = data[offset + 5u];
    let axis = unpack_octahedral_u16(w5 & 0xFFFFu);
    let phase01 = f32((w5 >> 16u) & 0xFFFFu) / 65536.0;

    let seed_in = data[offset + 6u];
    // Seed derivation must not depend on phase (stability + loopability).
    let w5_no_phase = w5 & 0xFFFFu;
    let seed = select(hash_u32(w0 ^ w1 ^ data[offset + 2u] ^ data[offset + 3u] ^ w4 ^ w5_no_phase), seed_in, seed_in != 0u);

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Dot-height gating in 0..255.
    let h01 = saturate(dot(dir, axis) * 0.5 + 0.5);
    let h255 = h01 * 255.0;
    let hmin = f32(min(height_min_u8, height_max_u8));
    let hmax = f32(max(height_min_u8, height_max_u8));
    let fade_w = 8.0;
    let gate_in = smoothstep(hmin, hmin + fade_w, h255);
    let gate_out = 1.0 - smoothstep(hmax - fade_w, hmax, h255);
    let height_gate = gate_in * gate_out;
    if (height_gate <= 0.0) {
        return vec4<f32>(0.0);
    }

    // Locked density→frequency mapping: 2..64 stripes.
    let d = density_u8;
    let freq_u = 2u + (((d - 1u) * 62u + 127u) / 254u);
    let freq0 = f32(freq_u);

    // Trig-free azimuth around axis (diamond angle).
    let b = basis_from_axis(axis);
    let p = vec2<f32>(dot(dir, b.t), dot(dir, b.b));
    let u01 = pseudo_angle01(p);
    let uv = vec2<f32>(u01, h01);

    // Taper: modulate width across the active height span.
    let taper01 = f32(taper_u8) / 255.0;
    let ht = saturate((h255 - hmin) / max(1.0, hmax - hmin));
    let taper_mul = mix(1.0 + taper01 * 0.7, 1.0 - taper01 * 0.7, ht);

    // Curvature: static bend + phase-driven sway (loopable).
    let curv01 = f32(curvature_u8) / 255.0;
    let curv_amp = curv01 * 0.12;

    // Slice count from parallax.
    let slice_count = select(1u, select(2u, 3u, parallax_u8 >= 192u), parallax_u8 >= 96u);

    // Soft-veils rule: enforce extra softness and reduce coverage.
    let soft_family = family == 3u;
    let edge_soft_clamped = select(edge_soft_u8, max(edge_soft_u8, 160u), soft_family);
    let edge_soft01 = f32(edge_soft_clamped) / 255.0;

    // Unrolled 1–3 slices. Weight rule: i==0 => 1, else parallax01.
    let c0 = color_near;
    let c1 = color_far;

    // Slice 0
    let s0 = veil_eval_slice(
        family,
        uv,
        freq0,
        phase01,
        width_u8,
        taper_mul,
        curv_amp,
        phase01,
        seed,
        edge_soft01,
        height_gate,
        c0,
        c1,
        0.0,
        1.0,
    );
    var accum = s0;

    if (slice_count >= 2u) {
        let depth1 = select(1.0, 0.5, slice_count == 3u);
        let s1 = veil_eval_slice(
            family,
            uv,
            freq0 + 1.0,
            phase01 + parallax01 * 0.37,
            max(1u, u32(f32(width_u8) * (1.0 - 0.22))),
            taper_mul,
            curv_amp,
            phase01,
            seed ^ 0x85ebca6bu,
            edge_soft01,
            height_gate,
            c0,
            c1,
            depth1,
            parallax01,
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + s1.rgb * inv_a;
        let a_new = accum.a + s1.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    if (slice_count >= 3u) {
        let s2 = veil_eval_slice(
            family,
            uv,
            freq0 + 2.0,
            phase01 + parallax01 * 0.74,
            max(1u, u32(f32(width_u8) * (1.0 - 0.44))),
            taper_mul,
            curv_amp,
            phase01,
            seed ^ 0xc2b2ae35u,
            edge_soft01,
            height_gate,
            c0,
            c1,
            1.0,
            parallax01,
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + s2.rgb * inv_a;
        let a_new = accum.a + s2.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    // Glow boosts RGB (not alpha).
    let glow = 1.0 + glow01 * 4.0;
    accum = vec4<f32>(accum.rgb * glow, accum.a);
    return accum;
}
