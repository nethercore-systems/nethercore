// ============================================================================
// PATCHES - Noise-Based Patch Enclosure Source (0x06)
// Distributes organic patches of wall across a sky background using noise.
// 128-bit packed fields:
//   color_a: Sky base color (RGB24)
//   color_b: Wall (patch) base color (RGB24)
//   intensity: Unused (set to 0)
//   param_a: Scale/frequency (0..255 -> 1.0..16.0)
//   param_b: Coverage/threshold (0..255 -> 0.0..1.0)
//   param_c: Sharpness (0..255 -> 0.0..0.5) - 0=soft edges, 255=hard edges
//   param_d: Seed for randomization (0..255)
//   direction: Axis (oct-u16) for AXIS_CYL/AXIS_POLAR or stretch dir for STREAKS
//   alpha_a: Sky alpha (0..15 -> 0.0..1.0)
//   alpha_b: Wall (patch) alpha (0..15 -> 0.0..1.0)
//   domain_id (meta5 bits 4..3): 0 DIRECT3D, 1 AXIS_CYL, 2 AXIS_POLAR
//   variant_id (meta5 bits 2..0): 0 BLOBS, 1 ISLANDS, 2 DEBRIS, 3 MEMBRANE, 4 STATIC, 5 STREAKS
// ============================================================================

// 3D hash for noise functions (deterministic, rollback-safe)
fn patches_hash31(p: vec3f) -> f32 {
    let h = dot(p, vec3f(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

// 3D value noise for PATCHES (trilinear interpolation)
// Returns value in [-1, 1] range
fn patches_value_noise3(p: vec3f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep interpolation

    let a = patches_hash31(i + vec3f(0.0, 0.0, 0.0));
    let b = patches_hash31(i + vec3f(1.0, 0.0, 0.0));
    let c = patches_hash31(i + vec3f(0.0, 1.0, 0.0));
    let d = patches_hash31(i + vec3f(1.0, 1.0, 0.0));
    let e = patches_hash31(i + vec3f(0.0, 0.0, 1.0));
    let f1 = patches_hash31(i + vec3f(1.0, 0.0, 1.0));
    let g = patches_hash31(i + vec3f(0.0, 1.0, 1.0));
    let h = patches_hash31(i + vec3f(1.0, 1.0, 1.0));

    let ab = mix(a, b, u.x);
    let cd = mix(c, d, u.x);
    let ef = mix(e, f1, u.x);
    let gh = mix(g, h, u.x);
    let abcd = mix(ab, cd, u.y);
    let efgh = mix(ef, gh, u.y);

    return mix(abcd, efgh, u.z) * 2.0 - 1.0;
}

// FBM noise for BLOBS variant (smooth, rounded shapes)
// max_octaves capped at 4 for performance
fn patches_fbm_blobs(p: vec3f, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    var norm = 0.0;
    let oct = min(octaves, 4u);
    for (var i = 0u; i < oct; i++) {
        sum += patches_value_noise3(p * freq) * amp;
        norm += amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    return sum / max(norm, 1e-6);
}

// FBM noise for ISLANDS variant (higher persistence, more defined edges)
fn patches_fbm_islands(p: vec3f, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    var norm = 0.0;
    let oct = min(octaves, 4u);
    for (var i = 0u; i < oct; i++) {
        sum += patches_value_noise3(p * freq) * amp;
        norm += amp;
        freq *= 2.0;
        amp *= 0.65; // Higher persistence for sharper features
    }
    return sum / max(norm, 1e-6);
}

// Turbulence noise for DEBRIS variant (absolute value, fragmented look)
fn patches_fbm_debris(p: vec3f, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    var norm = 0.0;
    let oct = min(octaves, 4u);
    for (var i = 0u; i < oct; i++) {
        sum += abs(patches_value_noise3(p * freq)) * amp;
        norm += amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    return (sum / max(norm, 1e-6)) * 2.0 - 1.0; // Remap to [-1, 1]
}

// High-frequency hash noise for STATIC variant (grainy texture)
fn patches_static(p: vec3f) -> f32 {
    return patches_hash31(floor(p * 32.0)) * 2.0 - 1.0;
}

// Anisotropic noise for STREAKS variant (stretched along direction)
fn patches_streaks(p: vec3f, stretch_dir: vec3f, octaves: u32) -> f32 {
    // Build basis to stretch along the given direction
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(stretch_dir.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, stretch_dir));
    let b_axis = normalize(cross(stretch_dir, t_axis));

    // Project p onto the basis, then stretch along the stretch direction
    let along = dot(p, stretch_dir);
    let across_t = dot(p, t_axis);
    let across_b = dot(p, b_axis);

    // Stretch factor: compress along stretch_dir, expand perpendicular
    let stretched = vec3f(across_t * 4.0, across_b * 4.0, along * 0.25);

    return patches_fbm_blobs(stretched, octaves);
}

// Compute axis-polar UV from direction and axis
fn patches_axis_polar_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build orthonormal basis around axis
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, axis));
    let b_axis = normalize(cross(axis, t_axis));

    // Project direction
    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);
    let z_proj = dot(dir, axis);

    // Polar coordinates: radius from pole, angle around axis
    let radius01 = acos(clamp(z_proj, -1.0, 1.0)) / PI; // 0 at +axis, 1 at -axis
    let angle01 = fract(atan2(b_proj, t_proj) / TAU + 0.5);

    return vec2f(angle01, radius01);
}

fn eval_patches(
    dir: vec3f,
    instr: vec4u,
) -> LayerSample {
    // Decode axis from direction field
    let axis = decode_dir16(instr_dir16(instr));

    // Extract parameters
    let scale = mix(1.0, 16.0, u8_to_01(instr_a(instr)));
    let coverage = u8_to_01(instr_b(instr));
    let sharpness = u8_to_01(instr_c(instr)) * 0.5;
    let seed = f32(instr_d(instr));
    let sky_alpha = instr_alpha_a_f32(instr);
    let wall_alpha = instr_alpha_b_f32(instr);
    let domain_id = instr_domain_id(instr);
    let variant = instr_variant_id(instr);

    // Build noise coordinates based on domain_id
    var p: vec3f;
    switch domain_id {
        case 0u: {
            // DIRECT3D: triplanar blending to avoid octahedral seams
            let w = abs(dir);
            let w3 = w * w * w;
            let wn = w3 / (w3.x + w3.y + w3.z);
            let px = vec3f(dir.y, dir.z, 0.0) * scale;
            let py = vec3f(dir.x, dir.z, 0.0) * scale;
            let pz = vec3f(dir.x, dir.y, 0.0) * scale;
            p = px * wn.x + py * wn.y + pz * wn.z;
        }
        case 1u: {
            // AXIS_CYL: cylindrical UV mapping around axis
            let uv = cell_axis_cylinder_uv(dir, axis);
            p = vec3f(uv * scale, 0.0);
        }
        case 2u: {
            // AXIS_POLAR: polar UV mapping on sphere
            let uv = patches_axis_polar_uv(dir, axis);
            p = vec3f(uv * scale, 0.0);
        }
        default: {
            p = dir * scale;
        }
    }

    // Add seed offset for variation
    p += vec3f(seed * 0.1, seed * 0.17, seed * 0.31);

    // Evaluate variant noise (band-limited, max 4 octaves)
    let octaves = 3u;
    var noise_val: f32;
    switch variant {
        case 0u: {
            // BLOBS: standard fbm, smooth rounded shapes
            noise_val = patches_fbm_blobs(p, octaves);
        }
        case 1u: {
            // ISLANDS: fbm with higher persistence, more defined edges
            noise_val = patches_fbm_islands(p, octaves);
        }
        case 2u: {
            // DEBRIS: fbm with turbulence (absolute value), fragmented look
            noise_val = patches_fbm_debris(p, octaves);
        }
        case 3u: {
            // MEMBRANE: inverted BLOBS (swap sky/wall interpretation)
            noise_val = patches_fbm_blobs(p, octaves);
        }
        case 4u: {
            // STATIC: high-frequency hash noise, grainy texture
            noise_val = patches_static(p);
        }
        case 5u: {
            // STREAKS: anisotropic noise stretched along direction
            noise_val = patches_streaks(p, axis, octaves);
        }
        default: {
            noise_val = patches_fbm_blobs(p, octaves);
        }
    }

    // Compute threshold: higher coverage = lower threshold = more patches
    let threshold = 1.0 - coverage;

    // Compute boundary width: (1.0 - sharpness) * 0.5
    // sharpness=0 -> bw=0.5 (soft), sharpness=0.5 -> bw=0.25 (medium-soft)
    let bw = (1.0 - sharpness * 2.0) * 0.5;

    // Compute weight: smoothstep from threshold-bw to threshold+bw
    // noise_val is in [-1, 1], remap to [0, 1] for threshold comparison
    let noise_01 = noise_val * 0.5 + 0.5;
    var w = smoothstep(threshold - bw, threshold + bw, noise_01);

    // For MEMBRANE variant: invert (swap sky/wall interpretation)
    if variant == 3u {
        w = 1.0 - w;
    }

    // Compute region weights: w_sky = 1 - w, w_wall = w, w_floor = 0
    let w_sky = (1.0 - w) * sky_alpha;
    let w_wall = w * wall_alpha;

    // Get colors
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);

    // Blend colors based on weights
    let total_w = w_sky + w_wall;
    var rgb: vec3f;
    if total_w > 0.001 {
        rgb = (sky_color * w_sky + wall_color * w_wall) / total_w;
    } else {
        rgb = sky_color;
    }

    // PATCHES is an enclosure source: return blended result
    return LayerSample(rgb, epu_saturate(total_w));
}
