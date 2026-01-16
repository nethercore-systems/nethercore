// ============================================================================
// Mode 4: Nebula (Fog / Clouds / Aurora / Ink / Plasma / Kaleido)
// ============================================================================
// w0: family:u8 | coverage:u8 | softness:u8 | intensity:u8
// w1: scale:u8 | detail:u8 | warp:u8 | flow:u8
// w2: color_a (RGBA8)
// w3: color_b (RGBA8)
// w4: height_bias:u8 | contrast:u8 | parallax:u8 | reserved:u8
// w5: axis_oct16 (low16) | phase:u16 (high16)
// w6: seed:u32 (0 = derive from packed words)
fn sample_nebula_layer(
    family: u32,
    coverage: f32,
    softness: f32,
    intensity: f32,
    scale01: f32,
    detail01: f32,
    warp01: f32,
    flow01: f32,
    height_bias01: f32,
    contrast01: f32,
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    axis: vec3<f32>,
    dir: vec3<f32>,
    local: vec3<f32>,
    uv_base: vec2<f32>,
    phase01_in: f32,
    seed: u32,
    parallax01: f32,
    depth01: f32,
    slice_index: u32,
    weight: f32
) -> vec4<f32> {
    if (coverage <= 0.0 || weight <= 0.0) {
        return vec4<f32>(0.0);
    }

    let seed01 = hash01_u32(seed ^ 0x6a09e667u);
    let parallax_layer = parallax01 * mix(1.0, 0.35, depth01);

    // Depth shaping: far slices are calmer + less emissive.
    let flow_l = flow01 * mix(1.0, 0.65, depth01);
    let warp_l = warp01 * mix(1.0, 0.55, depth01);
    let energy = (1.0 + intensity * 4.0) * mix(1.0, 0.65, depth01);

    // Start from axis-oriented oct UV (no trig).
    var uv = uv_base;

    // Loopable figure-eight domain offset (triangle wave + soft S-curve).
    let a = tri(phase01_in);
    let c = tri(phase01_in + 0.25);
    let a_s = a * (1.0 - 0.333 * a * a);
    let c_s = c * (1.0 - 0.333 * c * c);
    uv = uv + vec2<f32>(a_s, c_s) * (flow_l * 0.55);

    // Parallax: bias UV density near the local horizon band (optional; parallax=0 disables).
    if (parallax_layer > 0.0) {
        let horizon = 1.0 - abs(local.y);
        let boost = 1.0 + parallax_layer * horizon * 1.25;
        uv = uv * boost;
    }

    // Kaleido fold (bounded, unrolled).
    var uv_k = uv;
    if (family == 5u) {
        let uv_k_yx = uv_k.yx;
        uv_k = abs(uv_k);
        uv_k = select(uv_k, uv_k_yx, uv_k.x < uv_k.y);
        uv_k = abs(uv_k - vec2<f32>(0.55, 0.25));
        uv_k = abs(uv_k);
        uv_k = select(uv_k, uv_k_yx, uv_k.x < uv_k.y);
    }

    // Base frequency: higher scale => larger features (lower frequency).
    let freq = mix(12.0, 2.0, scale01) + f32(slice_index) * 0.9;
    var p = uv_k * freq + vec2<f32>(seed01 * 13.0, seed01 * 29.0) + vec2<f32>(f32(slice_index) * 7.0, f32(slice_index) * 11.0);

    // Domain warp (bounded, stable).
    if (warp_l > 0.0) {
        let wv = vec2<f32>(
            value_noise2(p + vec2<f32>(3.1, 7.7)),
            value_noise2(p + vec2<f32>(11.3, 2.9)),
        ) - vec2<f32>(0.5);
        p = p + wv * (warp_l * 2.0);
    }

    // Family-specific shaping before noise eval.
    if (family == 3u) { // Ink: emphasize swirl-like warp (linear mix; no trig)
        let k = warp_l * 1.5;
        p = vec2<f32>(p.x + p.y * k, p.y - p.x * k);
    }

    // Bounded noise (≤ 2 octaves).
    let n0 = value_noise2(p);
    let n1 = value_noise2(p * 2.0 + vec2<f32>(17.0, 31.0));
    var n = clamp(n0 + (n1 - 0.5) * detail01 * 0.6, 0.0, 1.0);

    // Height bias shaping along axis.
    let h01 = saturate(dot(dir, axis) * 0.5 + 0.5);
    let hb = saturate(1.0 - abs(h01 - height_bias01) * 2.0);
    let hb2 = hb * hb;
    n = n * (0.4 + 0.6 * hb2);

    // Family "language" adjustments.
    if (family == 0u) { // Fog: flatten contrast and bias toward macro haze
        n = mix(n, hb2, 0.35);
        n = mix(n, n * n * (3.0 - 2.0 * n), contrast01 * 0.35);
    } else if (family == 1u) { // Clouds: billowy structure
        n = mix(n, n * n * (3.0 - 2.0 * n), contrast01);
    } else if (family == 2u) { // Aurora: directional ribbons/curtains
        let bands = tri(uv_k.x * mix(2.0, 9.0, detail01) + seed01 * 3.0) * 0.5 + 0.5;
        let ribbon = smoothstep(0.35, 1.0, bands);
        n = mix(n, n * ribbon, 0.75);
        n = mix(n, n * n * (3.0 - 2.0 * n), contrast01);
    } else if (family == 4u) { // Plasma/Blobs: encourage large smooth islands
        n = mix(n, n * n, 0.6);
        n = mix(n, n * n * (3.0 - 2.0 * n), contrast01);
    } else { // Ink/Kaleido: keep contrast but allow warp/detail
        n = mix(n, n * n * (3.0 - 2.0 * n), contrast01);
    }

    // Coverage + AA shaping.
    let threshold = 1.0 - coverage;
    let aa = (fwidth(n) + 1e-4) * (1.0 + softness * 4.0);
    let mask = smoothstep(threshold - aa, threshold + aa, n);

    let a_out = clamp(mask * coverage * max(color_a.a, color_b.a) * weight * mix(1.0, 0.75, depth01), 0.0, 1.0);
    let rgb = mix(color_a.rgb, color_b.rgb, n) * a_out * energy;
    return vec4<f32>(rgb, a_out);
}

fn sample_nebula(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let family = w0 & 0xFFu;
    let coverage = f32((w0 >> 8u) & 0xFFu) / 255.0;
    let softness = f32((w0 >> 16u) & 0xFFu) / 255.0;
    let intensity = f32((w0 >> 24u) & 0xFFu) / 255.0;

    let w1 = data[offset + 1u];
    let scale01 = f32(w1 & 0xFFu) / 255.0;
    let detail01 = f32((w1 >> 8u) & 0xFFu) / 255.0;
    let warp01 = f32((w1 >> 16u) & 0xFFu) / 255.0;
    let flow01 = f32((w1 >> 24u) & 0xFFu) / 255.0;

    if (coverage <= 0.0) {
        return vec4<f32>(0.0);
    }

    let color_a = unpack_rgba8(data[offset + 2u]);
    let color_b = unpack_rgba8(data[offset + 3u]);

    let w4 = data[offset + 4u];
    let height_bias01 = f32(w4 & 0xFFu) / 255.0;
    let contrast01 = f32((w4 >> 8u) & 0xFFu) / 255.0;
    let parallax_u8 = (w4 >> 16u) & 0xFFu;
    let parallax01 = f32(parallax_u8) / 255.0;

    let w5 = data[offset + 5u];
    let axis = unpack_octahedral_u16(w5 & 0xFFFFu);
    let phase01 = f32((w5 >> 16u) & 0xFFFFu) / 65536.0;

    let seed_in = data[offset + 6u];
    // Seed derivation must not depend on phase (stability + loopability).
    let w5_no_phase = w5 & 0xFFFFu;
    let seed = select(hash_u32(w0 ^ w1 ^ data[offset + 2u] ^ data[offset + 3u] ^ w4 ^ w5_no_phase), seed_in, seed_in != 0u);

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Axis-oriented oct mapping (no trig).
    let b = basis_from_axis(axis);
    let local = vec3<f32>(dot(dir, b.t), dot(dir, b.n), dot(dir, b.b));
    let uv_base = dir_to_oct_uv(local); // [-1,1]

    let slice_count = select(1u, select(2u, 3u, parallax_u8 >= 192u), parallax_u8 >= 96u);

    // Slice 0 (nearest)
    let n0 = sample_nebula_layer(
        family,
        coverage,
        softness,
        intensity,
        scale01,
        detail01,
        warp01,
        flow01,
        height_bias01,
        contrast01,
        color_a,
        color_b,
        axis,
        dir,
        local,
        uv_base,
        phase01,
        seed,
        parallax01,
        0.0,
        0u,
        1.0
    );
    var accum = n0;

    if (slice_count >= 2u) {
        let depth1 = select(1.0, 0.5, slice_count == 3u);
        let n1 = sample_nebula_layer(
            family,
            coverage,
            softness,
            intensity,
            scale01,
            detail01,
            warp01,
            flow01,
            height_bias01,
            contrast01,
            color_a,
            color_b,
            axis,
            dir,
            local,
            uv_base,
            phase01 + parallax01 * 0.37,
            seed ^ 0x85ebca6bu,
            parallax01,
            depth1,
            1u,
            parallax01
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + n1.rgb * inv_a;
        let a_new = accum.a + n1.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    if (slice_count >= 3u) {
        let n2 = sample_nebula_layer(
            family,
            coverage,
            softness,
            intensity,
            scale01,
            detail01,
            warp01,
            flow01,
            height_bias01,
            contrast01,
            color_a,
            color_b,
            axis,
            dir,
            local,
            uv_base,
            phase01 + parallax01 * 0.74,
            seed ^ 0xc2b2ae35u,
            parallax01,
            1.0,
            2u,
            parallax01
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + n2.rgb * inv_a;
        let a_new = accum.a + n2.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    return accum;
}
