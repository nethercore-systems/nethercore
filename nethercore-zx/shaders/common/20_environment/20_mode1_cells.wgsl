// ============================================================================
// Mode 1: Cells (Particles / Tiles / Lights)
// ============================================================================
// w0: family:u8 | variant:u8 | density:u8 | intensity:u8
// w1: size_min:u8 | size_max:u8 | shape:u8 | motion:u8
// w2: color_a (RGBA8)
// w3: color_b (RGBA8)
// w4: parallax:u8 | reserved:u8 | reserved:u8 | flags:u8 (all reserved/flags are 0)
// w5: phase:u16 (low) | height_bias:u8 | clustering:u8
// w6: seed:u32 (0 = derive from packed words)

fn cells_seed_or_derive(seed_in: u32, mixin: u32) -> u32 {
    return select(hash_u32(mixin), seed_in, seed_in != 0u);
}

fn hash_cell_u32(cell: vec2<i32>, seed: u32, salt: u32) -> u32 {
    let x = bitcast<u32>(cell.x);
    let y = bitcast<u32>(cell.y);
    return hash_u32(x * 0x1f123bb5u ^ y * 0x9e3779b9u ^ seed ^ salt);
}

fn sample_cells_particles_layer(
    uv_base: vec2<f32>,
    dir: vec3<f32>,
    variant: u32,
    density: f32,
    size_min01: f32,
    size_max01: f32,
    shape: f32,
    motion: f32,
    parallax: f32,
    height_bias: f32,
    clustering: f32,
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    phase01_in: f32,
    seed: u32,
    depth01: f32,
    weight: f32,
    energy: f32,
) -> vec4<f32> {
    if (density <= 0.0 || weight <= 0.0) {
        return vec4<f32>(0.0);
    }

    // Depth shaping:
    // - depth01=0 is the nearest slice, depth01=1 is the farthest slice.
    // - Far slices are smaller + less parallax-biased.
    let parallax_layer = parallax * mix(1.0, 0.35, depth01);
    let horizon_boost = 1.0 + parallax_layer * (1.0 - abs(dir.y)) * 1.25;
    let phase01 = fract(phase01_in);

    var uv = uv_base;

    // Variant-specific loopable motion (must wrap cleanly at phase).
    if (variant == 1u) { // Fall (rain/snow)
        // Wrap within the oct domain for seamless looping.
        uv.y = fract(uv.y + phase01);
        uv.x = uv.x + tri(uv.y * 2.0 + phase01) * motion * 0.015;
    } else if (variant == 2u) { // Drift (embers/dust/bubbles)
        let drift = vec2<f32>(tri(phase01), tri(phase01 + 0.25)) * motion * 0.04;
        uv = uv + drift;
    } else if (variant == 3u) { // Warp (hyperspace/burst)
        let p = (uv - vec2<f32>(0.5)) * horizon_boost;
        let az = pseudo_angle01(p);
        let r = saturate(length(p) * 1.41421356); // normalize ~[0,1]
        uv = vec2<f32>(az, fract(r + phase01));
    }

    uv = (uv - vec2<f32>(0.5)) * horizon_boost + vec2<f32>(0.5);

    // Depth-dependent size (farther = smaller).
    let size_mul = mix(1.0, 0.55, depth01);
    let size_min_l = size_min01 * size_mul;
    let size_max_l = size_max01 * size_mul;

    // Grid frequency: smaller particles -> more cells.
    let size_hi = max(size_min_l, size_max_l);
    let freq = mix(10.0, 90.0, 1.0 - size_hi);

    let p = uv * freq;
    let base_cell = vec2<i32>(i32(floor(p.x)), i32(floor(p.y)));
    let f = fract(p);

    // Height bias shaping:
    // - Stars/Fall/Drift: 0 = zenith-biased, 1 = horizon-biased.
    // - Warp: 0 = edge-biased, 1 = center-biased (radial in oct UV).
    var place = 1.0;
    if (variant == 3u) {
        let rad = saturate(length((uv_base - vec2<f32>(0.5))) * 1.41421356);
        place = mix(rad, 1.0 - rad, height_bias);
    } else {
        let h01 = clamp(dir.y * 0.5 + 0.5, 0.0, 1.0);
        let zenith_w = h01;
        let horizon_w = 1.0 - abs(dir.y);
        place = mix(zenith_w, horizon_w, height_bias);
    }

    // Districting / clustering (0 = uniform, 1 = strongly patchy).
    let district = vec2<i32>(base_cell.x >> 2, base_cell.y >> 2);
    let district_hash = hash01_u32(hash_cell_u32(district, seed, 0x6a09e667u));
    let district_mask = smoothstep(0.25, 0.85, district_hash);
    let density_eff = clamp(density * mix(1.0, district_mask * 1.6, clustering) * mix(0.6, 1.4, place), 0.0, 1.0);

    // Choose 4 quadrant candidates (bounded ≤ 4 evals).
    let sx: i32 = select(1, -1, f.x < 0.5);
    let sy: i32 = select(1, -1, f.y < 0.5);

    let c0 = base_cell;
    let c1 = base_cell + vec2<i32>(sx, 0);
    let c2 = base_cell + vec2<i32>(0, sy);
    let c3 = base_cell + vec2<i32>(sx, sy);

    var best_a = 0.0;
    var best_rgb = vec3<f32>(0.0);

    // Candidate evaluation helper (inline, no unbounded loops).
    {
        let ch = hash_cell_u32(c0, seed, 0x243f6a88u);
        let spawn = hash01_u32(ch) < density_eff;
        if (spawn) {
            let jitter = (hash22_u32(ch ^ 0x9e3779b9u) - vec2<f32>(0.5)) * 0.6;
            let center = vec2<f32>(f32(c0.x), f32(c0.y)) + vec2<f32>(0.5) + jitter;
            let d = p - center;
            let dist = length(d);
            let size_r = mix(size_min_l, size_max_l, hash01_u32(ch ^ 0x85ebca6bu));
            let r = max(0.01, size_r * 0.35);
            let aa = fwidth(dist) + 1e-6;

            var a = 1.0 - smoothstep(r, r + aa, dist);
            var rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(ch ^ 0xc2b2ae35u));

            if (variant == 0u) { // Stars / Fireflies
                let tw = 0.65 + 0.35 * tri(phase01 + hash01_u32(ch ^ 0x27d4eb2fu));
                let glint_w = mix(0.18, 0.04, shape);
                let gx = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x));
                let gy = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.y));
                let g1 = max(gx, gy);
                let g2 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x + d.y));
                let g3 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x - d.y));
                let glint = max(g1, max(g2, g3)) * shape;
                a = max(a, glint * 0.85);
                rgb = rgb * tw;
            } else if (variant == 1u) { // Fall
                let streak = mix(0.25, 0.85, motion) * (1.0 - 0.7 * shape);
                let line = 1.0 - smoothstep(r * 0.5, r * 0.5 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * mix(1.0, line * trail, 1.0 - shape);
            } else if (variant == 2u) { // Drift
                let flicker = 0.7 + 0.3 * tri(phase01 + hash01_u32(ch ^ 0x165667b1u));
                // Bubble-ish ring at high shape.
                let ring = 1.0 - smoothstep(r * 0.65, r * 0.65 + aa, abs(dist - r * 0.55));
                a = mix(a, max(a, ring * 0.9), shape);
                rgb = mix(rgb, mix(color_a.rgb, color_b.rgb, 0.8), shape) * flicker;
            } else { // Warp
                let streak = mix(0.15, 0.9, motion);
                let axis_mask = 1.0 - smoothstep(r * 0.35, r * 0.35 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * axis_mask * trail;
                let pulse = 0.75 + 0.25 * tri(phase01 + hash01_u32(ch ^ 0xd1b54a35u));
                rgb = rgb * pulse;
            }

            if (a > best_a) {
                best_a = a;
                best_rgb = rgb;
            }
        }
    }

    {
        let ch = hash_cell_u32(c1, seed, 0x243f6a88u);
        let spawn = hash01_u32(ch) < density_eff;
        if (spawn) {
            let jitter = (hash22_u32(ch ^ 0x9e3779b9u) - vec2<f32>(0.5)) * 0.6;
            let center = vec2<f32>(f32(c1.x), f32(c1.y)) + vec2<f32>(0.5) + jitter;
            let d = p - center;
            let dist = length(d);
            let size_r = mix(size_min_l, size_max_l, hash01_u32(ch ^ 0x85ebca6bu));
            let r = max(0.01, size_r * 0.35);
            let aa = fwidth(dist) + 1e-6;

            var a = 1.0 - smoothstep(r, r + aa, dist);
            var rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(ch ^ 0xc2b2ae35u));

            if (variant == 0u) {
                let tw = 0.65 + 0.35 * tri(phase01 + hash01_u32(ch ^ 0x27d4eb2fu));
                let glint_w = mix(0.18, 0.04, shape);
                let gx = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x));
                let gy = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.y));
                let g1 = max(gx, gy);
                let g2 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x + d.y));
                let g3 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x - d.y));
                let glint = max(g1, max(g2, g3)) * shape;
                a = max(a, glint * 0.85);
                rgb = rgb * tw;
            } else if (variant == 1u) {
                let streak = mix(0.25, 0.85, motion) * (1.0 - 0.7 * shape);
                let line = 1.0 - smoothstep(r * 0.5, r * 0.5 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * mix(1.0, line * trail, 1.0 - shape);
            } else if (variant == 2u) {
                let flicker = 0.7 + 0.3 * tri(phase01 + hash01_u32(ch ^ 0x165667b1u));
                let ring = 1.0 - smoothstep(r * 0.65, r * 0.65 + aa, abs(dist - r * 0.55));
                a = mix(a, max(a, ring * 0.9), shape);
                rgb = mix(rgb, mix(color_a.rgb, color_b.rgb, 0.8), shape) * flicker;
            } else {
                let streak = mix(0.15, 0.9, motion);
                let axis_mask = 1.0 - smoothstep(r * 0.35, r * 0.35 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * axis_mask * trail;
                let pulse = 0.75 + 0.25 * tri(phase01 + hash01_u32(ch ^ 0xd1b54a35u));
                rgb = rgb * pulse;
            }

            if (a > best_a) {
                best_a = a;
                best_rgb = rgb;
            }
        }
    }

    {
        let ch = hash_cell_u32(c2, seed, 0x243f6a88u);
        let spawn = hash01_u32(ch) < density_eff;
        if (spawn) {
            let jitter = (hash22_u32(ch ^ 0x9e3779b9u) - vec2<f32>(0.5)) * 0.6;
            let center = vec2<f32>(f32(c2.x), f32(c2.y)) + vec2<f32>(0.5) + jitter;
            let d = p - center;
            let dist = length(d);
            let size_r = mix(size_min_l, size_max_l, hash01_u32(ch ^ 0x85ebca6bu));
            let r = max(0.01, size_r * 0.35);
            let aa = fwidth(dist) + 1e-6;

            var a = 1.0 - smoothstep(r, r + aa, dist);
            var rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(ch ^ 0xc2b2ae35u));

            if (variant == 0u) {
                let tw = 0.65 + 0.35 * tri(phase01 + hash01_u32(ch ^ 0x27d4eb2fu));
                let glint_w = mix(0.18, 0.04, shape);
                let gx = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x));
                let gy = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.y));
                let g1 = max(gx, gy);
                let g2 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x + d.y));
                let g3 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x - d.y));
                let glint = max(g1, max(g2, g3)) * shape;
                a = max(a, glint * 0.85);
                rgb = rgb * tw;
            } else if (variant == 1u) {
                let streak = mix(0.25, 0.85, motion) * (1.0 - 0.7 * shape);
                let line = 1.0 - smoothstep(r * 0.5, r * 0.5 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * mix(1.0, line * trail, 1.0 - shape);
            } else if (variant == 2u) {
                let flicker = 0.7 + 0.3 * tri(phase01 + hash01_u32(ch ^ 0x165667b1u));
                let ring = 1.0 - smoothstep(r * 0.65, r * 0.65 + aa, abs(dist - r * 0.55));
                a = mix(a, max(a, ring * 0.9), shape);
                rgb = mix(rgb, mix(color_a.rgb, color_b.rgb, 0.8), shape) * flicker;
            } else {
                let streak = mix(0.15, 0.9, motion);
                let axis_mask = 1.0 - smoothstep(r * 0.35, r * 0.35 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * axis_mask * trail;
                let pulse = 0.75 + 0.25 * tri(phase01 + hash01_u32(ch ^ 0xd1b54a35u));
                rgb = rgb * pulse;
            }

            if (a > best_a) {
                best_a = a;
                best_rgb = rgb;
            }
        }
    }

    {
        let ch = hash_cell_u32(c3, seed, 0x243f6a88u);
        let spawn = hash01_u32(ch) < density_eff;
        if (spawn) {
            let jitter = (hash22_u32(ch ^ 0x9e3779b9u) - vec2<f32>(0.5)) * 0.6;
            let center = vec2<f32>(f32(c3.x), f32(c3.y)) + vec2<f32>(0.5) + jitter;
            let d = p - center;
            let dist = length(d);
            let size_r = mix(size_min_l, size_max_l, hash01_u32(ch ^ 0x85ebca6bu));
            let r = max(0.01, size_r * 0.35);
            let aa = fwidth(dist) + 1e-6;

            var a = 1.0 - smoothstep(r, r + aa, dist);
            var rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(ch ^ 0xc2b2ae35u));

            if (variant == 0u) {
                let tw = 0.65 + 0.35 * tri(phase01 + hash01_u32(ch ^ 0x27d4eb2fu));
                let glint_w = mix(0.18, 0.04, shape);
                let gx = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x));
                let gy = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.y));
                let g1 = max(gx, gy);
                let g2 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x + d.y));
                let g3 = 1.0 - smoothstep(r * glint_w, r * glint_w + aa, abs(d.x - d.y));
                let glint = max(g1, max(g2, g3)) * shape;
                a = max(a, glint * 0.85);
                rgb = rgb * tw;
            } else if (variant == 1u) {
                let streak = mix(0.25, 0.85, motion) * (1.0 - 0.7 * shape);
                let line = 1.0 - smoothstep(r * 0.5, r * 0.5 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * mix(1.0, line * trail, 1.0 - shape);
            } else if (variant == 2u) {
                let flicker = 0.7 + 0.3 * tri(phase01 + hash01_u32(ch ^ 0x165667b1u));
                let ring = 1.0 - smoothstep(r * 0.65, r * 0.65 + aa, abs(dist - r * 0.55));
                a = mix(a, max(a, ring * 0.9), shape);
                rgb = mix(rgb, mix(color_a.rgb, color_b.rgb, 0.8), shape) * flicker;
            } else {
                let streak = mix(0.15, 0.9, motion);
                let axis_mask = 1.0 - smoothstep(r * 0.35, r * 0.35 + aa, abs(d.x));
                let trail = 1.0 - smoothstep(streak, streak + aa, abs(d.y));
                a = a * axis_mask * trail;
                let pulse = 0.75 + 0.25 * tri(phase01 + hash01_u32(ch ^ 0xd1b54a35u));
                rgb = rgb * pulse;
            }

            if (a > best_a) {
                best_a = a;
                best_rgb = rgb;
            }
        }
    }

    let out_a = best_a * max(color_a.a, color_b.a) * weight;
    let out_rgb = best_rgb * out_a * energy * mix(1.0, 0.65, depth01);
    return vec4<f32>(out_rgb, out_a);
}

fn sample_cells(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let w1 = data[offset + 1u];

    let family = w0 & 0xFFu;
    let variant = (w0 >> 8u) & 0x3u;
    let density = f32((w0 >> 16u) & 0xFFu) / 255.0;
    let intensity = f32((w0 >> 24u) & 0xFFu) / 255.0;

    let size_min01 = f32(w1 & 0xFFu) / 255.0;
    let size_max01 = f32((w1 >> 8u) & 0xFFu) / 255.0;
    let shape = f32((w1 >> 16u) & 0xFFu) / 255.0;
    let motion = f32((w1 >> 24u) & 0xFFu) / 255.0;

    let color_a = unpack_rgba8(data[offset + 2u]);
    let color_b = unpack_rgba8(data[offset + 3u]);

    let parallax_u8 = data[offset + 4u] & 0xFFu;
    let parallax = f32(parallax_u8) / 255.0;

    let w5 = data[offset + 5u];
    let phase_u16 = w5 & 0xFFFFu;
    let phase01 = f32(phase_u16) / 65536.0; // [0,1) loop param
    let height_bias = f32((w5 >> 16u) & 0xFFu) / 255.0;
    let clustering = f32((w5 >> 24u) & 0xFFu) / 255.0;

    let seed_in = data[offset + 6u];
    // Seed derivation must not depend on phase (stability + loopability).
    let w5_no_phase = w5 & 0xFFFF0000u;
    let seed = cells_seed_or_derive(
        seed_in,
        w0 ^ (w1 * 0x9e3779b9u) ^ data[offset + 2u] ^ (data[offset + 3u] * 0x85ebca6bu) ^ (data[offset + 4u] * 0xc2b2ae35u) ^ w5_no_phase,
    );

    if (density <= 0.0 || intensity <= 0.0) {
        return vec4<f32>(0.0);
    }

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));
    let h01 = clamp(dir.y * 0.5 + 0.5, 0.0, 1.0);

    // Base oct UV (0..1), with mild parallax scaling near the horizon.
    let uv_base = dir_to_oct_uv01(dir);
    let horizon_boost = 1.0 + parallax * (1.0 - abs(dir.y)) * 1.25;

    // Energy scaling: intensity boosts RGB more than alpha (coverage stays geometric).
    let energy = 1.0 + intensity * 6.0;

    // ------------------------------------------------------------------------
    // Family 0: Particles
    // ------------------------------------------------------------------------
    if (family == 0u) {
        // Parallax selects 1–3 bounded internal depth slices.
        let slice_count = select(1u, select(2u, 3u, parallax_u8 >= 192u), parallax_u8 >= 96u);

        // Slice 0 (nearest)
        let p0 = sample_cells_particles_layer(
            uv_base,
            dir,
            variant,
            density,
            size_min01,
            size_max01,
            shape,
            motion,
            parallax,
            height_bias,
            clustering,
            color_a,
            color_b,
            phase01,
            seed,
            0.0,
            1.0,
            energy,
        );
        var accum = p0;

        // Slice 1
        if (slice_count >= 2u) {
            let depth1 = select(1.0, 0.5, slice_count == 3u);
            let p1 = sample_cells_particles_layer(
                uv_base,
                dir,
                variant,
                density,
                size_min01,
                size_max01,
                shape,
                motion,
                parallax,
                height_bias,
                clustering,
                color_a,
                color_b,
                phase01 + parallax * 0.37,
                seed ^ 0x85ebca6bu,
                depth1,
                parallax,
                energy,
            );
            let inv_a = 1.0 - accum.a;
            let rgb_new = accum.rgb + p1.rgb * inv_a;
            let a_new = accum.a + p1.a * inv_a;
            accum = vec4<f32>(rgb_new, a_new);
        }

        // Slice 2 (farthest)
        if (slice_count >= 3u) {
            let p2 = sample_cells_particles_layer(
                uv_base,
                dir,
                variant,
                density,
                size_min01,
                size_max01,
                shape,
                motion,
                parallax,
                height_bias,
                clustering,
                color_a,
                color_b,
                phase01 + parallax * 0.74,
                seed ^ 0xc2b2ae35u,
                1.0,
                parallax,
                energy,
            );
            let inv_a = 1.0 - accum.a;
            let rgb_new = accum.rgb + p2.rgb * inv_a;
            let a_new = accum.a + p2.a * inv_a;
            accum = vec4<f32>(rgb_new, a_new);
        }

        return accum;
    }

    // ------------------------------------------------------------------------
    // Family 1: Tiles / Lights
    // ------------------------------------------------------------------------
    {
        var uv = (uv_base - vec2<f32>(0.5)) * horizon_boost + vec2<f32>(0.5);

        // Mild "perspective" bias for tile families near the horizon.
        uv.x = (uv.x - 0.5) * (1.0 + parallax * (1.0 - h01) * 1.25) + 0.5;

        let size_hi = max(size_min01, size_max01);
        let freq = mix(3.0, 30.0, 1.0 - size_hi);
        let p = uv * freq;
        let cell = vec2<i32>(i32(floor(p.x)), i32(floor(p.y)));
        let f = fract(p);

        // Zoning and districts.
        let zenith_w = h01;
        let horizon_w = 1.0 - abs(dir.y);
        let zone = mix(zenith_w, horizon_w, height_bias);
        let district = vec2<i32>(cell.x >> 2, cell.y >> 2);
        let district_hash = hash01_u32(hash_cell_u32(district, seed, 0xbb67ae85u));
        let district_mask = smoothstep(0.2, 0.85, district_hash);
        let density_eff = clamp(density * mix(1.0, district_mask * 1.5, clustering) * mix(0.6, 1.4, zone), 0.0, 1.0);

        let cell_h = hash_cell_u32(cell, seed, 0x3c6ef372u);
        if (hash01_u32(cell_h) > density_eff) {
            return vec4<f32>(0.0);
        }

        // Per-cell phase offsets (loopable; no phase as hash).
        let flick = 0.65 + 0.35 * tri(phase01 + hash01_u32(cell_h ^ 0xa54ff53au));
        let accent = 0.65 + 0.35 * tri(phase01 + hash01_u32(cell_h ^ 0x510e527fu) + 0.25);

        var a = 0.0;
        var rgb = color_a.rgb;

        if (variant == 0u) { // Abstract Tiles (Mondrian / Truchet)
            // Mondrian-ish blocks
            let split_x = mix(0.25, 0.75, hash01_u32(cell_h ^ 0x9b05688cu));
            let split_y = mix(0.25, 0.75, hash01_u32(cell_h ^ 0x1f83d9abu));
            let block = select(0.0, 1.0, (f.x < split_x) == (f.y < split_y));

            // Truchet-ish arc tile
            let tile = hash01_u32(cell_h ^ 0x5be0cd19u) > 0.5;
            let p0 = select(vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), tile);
            let p1 = select(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), tile);
            let d0 = abs(length(f - p0) - 0.5);
            let d1 = abs(length(f - p1) - 0.5);
            let d = min(d0, d1);
            let aa = fwidth(d) + 1e-6;
            let arc = 1.0 - smoothstep(0.06, 0.06 + aa, d);

            let tile_mix = shape; // 0 = Mondrian, 1 = Truchet
            a = mix(block, arc, tile_mix);
            rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(cell_h ^ 0x6d2b79f5u));
        } else if (variant == 1u) { // Buildings (windows)
            // Window aspect: 0 = square/soft, 1 = tall/hard.
            let aspect = mix(1.0, 2.8, shape);
            let w = 0.55;
            let h = 0.55 * aspect;
            let dx = abs(f.x - 0.5);
            let dy = abs(f.y - 0.5);
            let aa = max(fwidth(f.x), fwidth(f.y)) + 1e-6;
            let inside = (1.0 - smoothstep(w * 0.5, w * 0.5 + aa, dx)) * (1.0 - smoothstep(h * 0.5, h * 0.5 + aa, dy));

            // Patchy window lighting: more variance when motion is high.
            let light = mix(flick, accent, motion);

            a = inside;
            rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(cell_h ^ 0x94d049bfu)) * light;
        } else if (variant == 2u) { // Bands (signage floors)
            let thick = mix(0.08, 0.35, shape);
            let aa = fwidth(f.y) + 1e-6;
            let band = 1.0 - smoothstep(thick, thick + aa, abs(f.y - 0.5));

            // Segmentation along x.
            let segs = mix(4.0, 24.0, density);
            let sx = fract(f.x * segs);
            let edge = fwidth(f.x * segs) + 1e-6;
            let on = smoothstep(0.0, edge, sx) * (1.0 - smoothstep(0.65, 0.65 + edge, sx));
            a = band * on;
            rgb = mix(color_a.rgb, color_b.rgb, hash01_u32(cell_h ^ 0x2f6a3b55u)) * mix(flick, accent, motion);
        } else { // Panels (UI grids)
            let seam = 1.0 - smoothstep(0.08, 0.08 + fwidth(f.x) + 1e-6, min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y)));
            let dot_d = length(f - vec2<f32>(0.5));
            let dot_aa = fwidth(dot_d) + 1e-6;
            let dot = 1.0 - smoothstep(0.12, 0.12 + dot_aa, dot_d);
            let scan = 0.5 + 0.5 * tri(phase01 + hash01_u32(cell_h ^ 0x4cf5ad43u));

            a = max(seam * mix(0.5, 1.0, shape), dot);
            rgb = mix(color_a.rgb, color_b.rgb, scan) * mix(flick, accent, motion);
        }

        let out_a = clamp(a * max(color_a.a, color_b.a), 0.0, 1.0);
        let out_rgb = rgb * out_a * energy;
        return vec4<f32>(out_rgb, out_a);
    }
}
