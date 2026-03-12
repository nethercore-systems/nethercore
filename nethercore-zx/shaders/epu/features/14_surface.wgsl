// @epu_meta_begin
// opcode = 0x16
// name = SURFACE
// kind = feature
// variants = [GLAZE, CRUST, FACET, DUSTED]
// domains = []
// field intensity = { label="contrast", map="u8_01" }
// field param_a = { label="scale", map="u8_lerp", min=0.5, max=16.0, unit="x" }
// field param_b = { label="fracture", map="u8_01" }
// field param_c = { label="sheen", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// SURFACE - Broad Material / Surface Response Carrier
// Opcode: 0x16
// Role: Feature
//
// Purpose:
//   A reusable material-response carrier for broad surface identity. It covers
//   glazed, crusted, faceted, or dusted beds without collapsing back into
//   liquid-water reads or literal scene nouns.
// ============================================================================

const SURFACE_VARIANT_GLAZE: u32 = 0u;
const SURFACE_VARIANT_CRUST: u32 = 1u;
const SURFACE_VARIANT_FACET: u32 = 2u;
const SURFACE_VARIANT_DUSTED: u32 = 3u;

fn surface_hash21(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.xyx) * 0.1031);
    let d = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z + d);
}

fn surface_hash22(p: vec2f) -> vec2f {
    var p3 = fract(vec3f(p.xyx) * vec3f(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn surface_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = surface_hash21(i);
    let b = surface_hash21(i + vec2f(1.0, 0.0));
    let c = surface_hash21(i + vec2f(0.0, 1.0));
    let d = surface_hash21(i + vec2f(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn surface_fbm(p: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * surface_noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

fn surface_voronoi(p: vec2f) -> vec3f {
    let cell = floor(p);
    let f = fract(p);

    var min_dist = 8.0;
    var second_dist = 8.0;
    var cell_hash = 0.0;

    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let neighbor = vec2f(f32(i), f32(j));
            let offset = surface_hash22(cell + neighbor);
            let point = neighbor + offset - f;
            let d = dot(point, point);

            if d < min_dist {
                second_dist = min_dist;
                min_dist = d;
                cell_hash = surface_hash21(cell + neighbor);
            } else if d < second_dist {
                second_dist = d;
            }
        }
    }

    return vec3f(sqrt(min_dist), second_dist - min_dist, cell_hash);
}

fn surface_build_basis(axis: vec3f) -> mat3x3f {
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(ref_vec, axis));
    let b = normalize(cross(axis, t));
    return mat3x3f(t, axis, b);
}

fn surface_apply_contrast(x: f32, contrast: f32) -> f32 {
    let gain = 1.0 + contrast * 3.0;
    return epu_saturate((x - 0.5) * gain + 0.5);
}

fn eval_surface(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    let variant = instr_variant_id(instr);
    let axis16 = instr_dir16(instr);
    let axis = normalize(select(vec3f(0.0, 1.0, 0.0), decode_dir16(axis16), axis16 != 0u));
    let basis = surface_build_basis(axis);

    let contrast = u8_to_01(instr_intensity(instr));
    let scale = mix(0.5, 16.0, u8_to_01(instr_a(instr)));
    let fracture = u8_to_01(instr_b(instr));
    let sheen = u8_to_01(instr_c(instr));
    let phase = u8_to_01(instr_d(instr)) * TAU;

    let drift = vec2f(cos(phase), sin(phase)) * mix(0.015, 0.12, sheen);
    let base_uv = vec2f(dot(dir, basis[0]), dot(dir, basis[2])) * scale + drift;
    let warp = vec2f(
        surface_noise(base_uv * 0.85 + vec2f(7.0, 13.0)) - 0.5,
        surface_noise(base_uv * 0.85 + vec2f(19.0, -5.0)) - 0.5
    ) * mix(0.03, 0.28, fracture);
    let uv = base_uv + warp;

    let graze = epu_saturate(1.0 - abs(dot(dir, axis)));

    var base = 0.5;
    var highlight = 0.0;
    var coverage = 1.0;

    switch variant {
        case SURFACE_VARIANT_GLAZE: {
            let low = surface_fbm(uv * 0.55, 3u);
            let veins = 1.0 - abs(surface_fbm(uv * 1.35 + vec2f(11.0, -3.0), 3u));
            base = mix(low, veins, fracture * 0.4);
            highlight = smoothstep(0.55, 0.98, graze + veins * 0.25) * (0.2 + sheen * 0.8);
            coverage = smoothstep(0.18, 0.86, low);
        }
        case SURFACE_VARIANT_CRUST: {
            let vor = surface_voronoi(uv * mix(0.8, 2.0, fracture));
            let plates = smoothstep(0.05, 0.26, vor.y + fracture * 0.12);
            let powder = surface_fbm(uv * 1.3 + vec2f(-5.0, 9.0), 2u);
            base = mix(powder, vor.z, 0.35);
            highlight = smoothstep(0.72, 0.98, graze) * plates * sheen * 0.45;
            coverage = plates * (0.7 + powder * 0.3);
        }
        case SURFACE_VARIANT_FACET: {
            let ridge = 1.0 - abs(surface_fbm(uv * mix(1.0, 3.0, fracture) + vec2f(13.0, 5.0), 3u));
            let shard = surface_noise(uv * 2.4 + vec2f(-9.0, 17.0));
            base = mix(ridge, shard, 0.3);
            highlight = pow(epu_saturate(graze + ridge * 0.45), mix(4.0, 18.0, sheen)) * (0.3 + fracture * 0.7);
            coverage = smoothstep(0.2, 0.84, ridge * 0.8 + shard * 0.2);
        }
        case SURFACE_VARIANT_DUSTED: {
            let frost = surface_fbm(uv * 0.6 + vec2f(3.0, -7.0), 3u);
            let streak = surface_noise(uv * 2.2 + vec2f(-phase * 0.08, phase * 0.06));
            base = mix(frost, streak, fracture * 0.2);
            highlight = smoothstep(0.78, 0.98, graze) * sheen * 0.22;
            coverage = smoothstep(0.16, 0.82, frost);
        }
        default: {
            let low = surface_fbm(uv * 0.55, 3u);
            base = low;
            highlight = smoothstep(0.55, 0.98, graze) * sheen * 0.4;
            coverage = smoothstep(0.18, 0.86, low);
        }
    }

    base = surface_apply_contrast(base, contrast);
    coverage = epu_saturate(coverage);
    highlight = epu_saturate(highlight);

    let base_rgb = mix(instr_color_b(instr), instr_color_a(instr), base);
    let highlight_tint = mix(base_rgb, instr_color_a(instr), 0.55);
    let highlight_rgb = mix(base_rgb, highlight_tint, 0.12 + sheen * 0.24);
    let rgb = mix(base_rgb, highlight_rgb, highlight);
    let alpha = instr_alpha_a_f32(instr) * region_w * mix(0.35, 1.0, coverage);
    return LayerSample(rgb, alpha);
}
