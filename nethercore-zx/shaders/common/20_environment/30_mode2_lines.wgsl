// ============================================================================
// Mode 2: Lines (Grid / Lanes / Scanlines / Caustic Bands)
// ============================================================================
// w0: variant:u2 | line_type:u2 | thickness:u8 | accent_every:u8 | parallax:u8 | reserved:u4
// w1: spacing:f16 (low16) | fade_distance:f16 (high16)
// w2: color_primary (RGBA8)
// w3: color_accent (RGBA8)
// w4: phase:u16 (low16) | axis_oct16:u16 (high16)
// w5: warp:u8 | glow:u8 | wobble:u8 | profile:u8
// w6: seed:u32 (0 = derive from packed words)
fn sample_lines_layer(
    variant: u32,
    line_type: u32,
    thickness_u8: u32,
    accent_every_u32: u32,
    spacing: f32,
    fade_distance: f32,
    color_primary: vec4<f32>,
    color_accent: vec4<f32>,
    phase01_in: f32,
    axis: vec3<f32>,
    warp01: f32,
    glow01: f32,
    wobble01: f32,
    profile: u32,
    seed01: f32,
    dir: vec3<f32>,
    parallax01: f32,
    depth01: f32,
    slice_index: u32,
    weight: f32
) -> vec4<f32> {
    if (weight <= 0.0) {
        return vec4<f32>(0.0);
    }

    let parallax_layer = parallax01 * mix(1.0, 0.35, depth01);
    let warp_l = warp01 * mix(1.0, 0.55, depth01);
    let wobble_l = wobble01 * mix(1.0, 0.55, depth01);

    let phase01 = phase01_in;
    var uv = vec2<f32>(0.0);
    var fade = 1.0;

    if (variant == 0u || variant == 1u) {
        // Floor/Ceiling: ray-plane projection in world-ish units.
        let plane_y = select(-1.0, 1.0, variant == 1u);
        if (dir.y * plane_y <= 0.001) {
            return vec4<f32>(0.0);
        }

        let t = plane_y / dir.y;
        let p = dir.xz * t;
        uv = p;
        fade = 1.0 - smoothstep(0.0, fade_distance, length(p));

        // Scroll direction: project axis onto XZ.
        let scroll_dir = safe_normalize2(vec2<f32>(axis.x, axis.z), vec2<f32>(0.0, 1.0));
        uv = uv + scroll_dir * (phase01 * spacing);

        // Parallax: horizon-proportional perspective bias (optional; parallax=0 disables).
        if (parallax_layer > 0.0) {
            let horizon = 1.0 - abs(dir.y);
            let boost = 1.0 + parallax_layer * horizon * 1.25;
            uv = uv * boost;
        }
    } else {
        // Sphere: axis-oriented oct mapping (no trig).
        let b = basis_from_axis(axis);
        let local = vec3<f32>(dot(dir, b.t), dot(dir, b.n), dot(dir, b.b));
        let uv_oct = dir_to_oct_uv(local); // [-1,1]
        let r = length(uv_oct);
        fade = 1.0 - smoothstep(fade_distance, 1.41421356, r);
        uv = uv_oct + vec2<f32>(0.0, 1.0) * (phase01 * spacing);

        // Parallax: bias density near the local horizon band (optional; parallax=0 disables).
        if (parallax_layer > 0.0) {
            let horizon = 1.0 - abs(local.y);
            let boost = 1.0 + parallax_layer * horizon * 1.25;
            uv = uv * boost;
        }
    }

    // Work in line-space (period 1), apply warp/wobble in a stable, loopable way.
    var s = uv / spacing;

    // Stagger internal slices so the union reads as depth layers rather than a single grid.
    if (slice_index > 0u) {
        let ofs = vec2<f32>(0.33, 0.17) * f32(slice_index);
        s = s + ofs;
    }

    if (warp_l > 0.0 || wobble_l > 0.0) {
        let w = tri(s.x * 0.35 + s.y * 0.27 + seed01 * 4.0);
        let w2 = tri(s.y * 0.41 + seed01 * 7.3);
        s = s + vec2<f32>(w, w2) * (warp_l * 0.35);

        let wob = vec2<f32>(tri(s.y * 0.65 + phase01), tri(s.x * 0.65 + phase01 + 0.25));
        s = s + wob * (wobble_l * 0.25);
    }

    var thick = max(0.0005, f32(thickness_u8) / 255.0 * 0.5);
    if (profile == 1u) { // Lanes
        thick = thick * 2.2;
    } else if (profile == 2u) { // Scanlines
        thick = thick * 1.1;
    } else if (profile == 3u) { // Caustic bands
        thick = thick * 4.0;
    }

    // Far slices read finer and less dominant.
    thick = thick * mix(1.0, 0.82, depth01);

    let aa_x = fwidth(s.x) + 1e-6;
    let aa_y = fwidth(s.y) + 1e-6;

    var h_line = 0.0;
    var v_line = 0.0;

    if (line_type == 0u || line_type == 2u) {
        let d = abs(fract(s.y) - 0.5);
        h_line = 1.0 - smoothstep(thick, thick + aa_y, d);
    }
    if (line_type == 1u || line_type == 2u) {
        let d = abs(fract(s.x) - 0.5);
        v_line = 1.0 - smoothstep(thick, thick + aa_x, d);
    }

    // Union for grid (crisper than max for thin lines).
    let line_intensity = select(max(h_line, v_line), 1.0 - (1.0 - h_line) * (1.0 - v_line), line_type == 2u);

    // Accent cadence from integer line IDs (stable; no screen-space hashing).
    let ae = i32(accent_every_u32);
    let id_x = i32(floor(s.x));
    let id_y = i32(floor(s.y));
    let mod_x = ((id_x % ae) + ae) % ae;
    let mod_y = ((id_y % ae) + ae) % ae;
    let accent_x = mod_x == 0;
    let accent_y = mod_y == 0;

    // Choose the dominant component for accent selection.
    let use_h = h_line >= v_line;
    let is_accent = select(accent_x, accent_y, use_h);

    // Profile-specific shaping.
    var shaped = line_intensity;
    if (profile == 2u) { // Scanlines: softer bands
        shaped = shaped * shaped;
    } else if (profile == 3u) { // Caustic: add gentle phase-driven modulation
        let wob = 0.5 + 0.5 * tri(phase01 + seed01 + s.x * 0.15);
        shaped = shaped * mix(0.65, 1.35, wob);
    }

    // Accent emphasis for lanes.
    if (profile == 1u && is_accent) {
        shaped = min(1.0, shaped * 1.35);
    }

    let line_color = select(color_primary, color_accent, is_accent);
    let a = clamp(line_color.a * shaped * fade * weight * mix(1.0, 0.75, depth01), 0.0, 1.0);
    let rgb = line_color.rgb * a * (1.0 + glow01 * 4.0) * mix(1.0, 0.8, depth01);
    return vec4<f32>(rgb, a);
}

fn sample_lines(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let w0 = data[offset];
    let variant = w0 & 0x3u;           // 0=Floor, 1=Ceiling, 2=Sphere
    let line_type = (w0 >> 2u) & 0x3u; // 0=Horizontal, 1=Vertical, 2=Grid
    let thickness_u8 = (w0 >> 4u) & 0xFFu;
    let accent_every_u32 = max(1u, (w0 >> 12u) & 0xFFu);
    let parallax_u8 = (w0 >> 20u) & 0xFFu;
    let parallax01 = f32(parallax_u8) / 255.0;

    let spacing_fade = unpack2x16float(data[offset + 1u]);
    let spacing = max(1e-4, spacing_fade.x);
    let fade_distance = max(1e-4, spacing_fade.y);

    let color_primary = unpack_rgba8(data[offset + 2u]);
    let color_accent = unpack_rgba8(data[offset + 3u]);

    let w4 = data[offset + 4u];
    let phase01 = f32(w4 & 0xFFFFu) / 65536.0;
    let axis = unpack_octahedral_u16(w4 >> 16u);

    let w5 = data[offset + 5u];
    let warp01 = f32(w5 & 0xFFu) / 255.0;
    let glow01 = f32((w5 >> 8u) & 0xFFu) / 255.0;
    let wobble01 = f32((w5 >> 16u) & 0xFFu) / 255.0;
    let profile = (w5 >> 24u) & 0x3u;

    let seed_in = data[offset + 6u];
    // Seed derivation must not depend on phase (stability + loopability).
    let w4_no_phase = w4 & 0xFFFF0000u;
    let seed = select(hash_u32(w0 ^ data[offset + 1u] ^ data[offset + 2u] ^ data[offset + 3u] ^ w4_no_phase ^ w5), seed_in, seed_in != 0u);
    let seed01 = hash01_u32(seed ^ 0x9e3779b9u);

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    let slice_count = select(1u, select(2u, 3u, parallax_u8 >= 192u), parallax_u8 >= 96u);

    // Slice 0 (nearest)
    let l0 = sample_lines_layer(
        variant,
        line_type,
        thickness_u8,
        accent_every_u32,
        spacing,
        fade_distance,
        color_primary,
        color_accent,
        phase01,
        axis,
        warp01,
        glow01,
        wobble01,
        profile,
        seed01,
        dir,
        parallax01,
        0.0,
        0u,
        1.0
    );
    var accum = l0;

    if (slice_count >= 2u) {
        let depth1 = select(1.0, 0.5, slice_count == 3u);
        let seed01_1 = hash01_u32((seed ^ 0x85ebca6bu) ^ 0x9e3779b9u);
        let l1 = sample_lines_layer(
            variant,
            line_type,
            thickness_u8,
            accent_every_u32,
            spacing,
            fade_distance,
            color_primary,
            color_accent,
            phase01 + parallax01 * 0.37,
            axis,
            warp01,
            glow01,
            wobble01,
            profile,
            seed01_1,
            dir,
            parallax01,
            depth1,
            1u,
            parallax01
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + l1.rgb * inv_a;
        let a_new = accum.a + l1.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    if (slice_count >= 3u) {
        let seed01_2 = hash01_u32((seed ^ 0xc2b2ae35u) ^ 0x9e3779b9u);
        let l2 = sample_lines_layer(
            variant,
            line_type,
            thickness_u8,
            accent_every_u32,
            spacing,
            fade_distance,
            color_primary,
            color_accent,
            phase01 + parallax01 * 0.74,
            axis,
            warp01,
            glow01,
            wobble01,
            profile,
            seed01_2,
            dir,
            parallax01,
            1.0,
            2u,
            parallax01
        );
        let inv_a = 1.0 - accum.a;
        let rgb_new = accum.rgb + l2.rgb * inv_a;
        let a_new = accum.a + l2.a * inv_a;
        accum = vec4<f32>(rgb_new, a_new);
    }

    return accum;
}
