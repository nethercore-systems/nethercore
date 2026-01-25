// ============================================================================
// SCATTER - Point Field (Stars / Dust / Windows)
// Packed fields:
//   color_a: Primary point color (RGB24)
//   color_b: Color variation (RGB24) - points randomly vary between a and b
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Density (0..255 -> 1..256)
//   param_b: Point size (0..255 -> 0.001..0.05 rad)
//   param_c[7:4]: Twinkle amount (0..15 -> 0..1)
//   param_c[3:0]: Reserved (set to 0)
//   param_d: Seed for randomization (0..255)
//   direction: Axis direction (oct-u16) for AXIS_CYL/AXIS_POLAR domains (set to 0 for DIRECT3D)
//   meta5[4:3]: Domain ID (coordinate system)
//   meta5[2:0]: Variant ID (visual style)
// ============================================================================

// Domain IDs - coordinate system for point distribution
const SCATTER_DOMAIN_DIRECT3D: u32 = 0u;     // Direction sphere (default)
const SCATTER_DOMAIN_AXIS_CYL: u32 = 1u;     // Cylindrical (azimuth, height)
const SCATTER_DOMAIN_AXIS_POLAR: u32 = 2u;   // Polar (angle, radius from axis)
const SCATTER_DOMAIN_TANGENT_LOCAL: u32 = 3u; // Tangent plane at direction

// Variant IDs - visual style variations
const SCATTER_VARIANT_STARS: u32 = 0u;       // Bright pinpoints, twinkling
const SCATTER_VARIANT_DUST: u32 = 1u;        // Larger, dimmer, slower motion
const SCATTER_VARIANT_WINDOWS: u32 = 2u;     // Rectangular, no twinkle
const SCATTER_VARIANT_BUBBLES: u32 = 3u;     // Round, slow drift up
const SCATTER_VARIANT_EMBERS: u32 = 4u;      // Elongated, upward drift, fade
const SCATTER_VARIANT_RAIN: u32 = 5u;        // Vertical streaks, fast
const SCATTER_VARIANT_SNOW: u32 = 6u;        // Soft, slow, drift

fn hash3(p: vec3f) -> vec4f {
    var p4 = fract(vec4f(p.xyzx) * vec4f(0.1031, 0.1030, 0.0973, 0.1099));
    p4 += dot(p4, p4.wzxy + 33.33);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

// Cylindrical UV mapping for scatter domain
fn scatter_cyl_uv(dir: vec3f, axis: vec3f) -> vec2f {
    let up = normalize(axis);
    let v = dot(dir, up);
    let proj = normalize(dir - up * v);
    // Reference for azimuth
    var right = cross(up, vec3f(0.0, 0.0, 1.0));
    if length(right) < 0.01 {
        right = cross(up, vec3f(1.0, 0.0, 0.0));
    }
    right = normalize(right);
    let fwd = cross(right, up);
    let u = atan2(dot(proj, fwd), dot(proj, right)) / TAU;
    return vec2f(u, v);
}

// Polar UV mapping for scatter domain
fn scatter_polar_uv(dir: vec3f, axis: vec3f) -> vec2f {
    let up = normalize(axis);
    let v = dot(dir, up);
    let rad = sqrt(max(0.0, 1.0 - v * v));
    let proj = normalize(dir - up * v);
    var right = cross(up, vec3f(0.0, 0.0, 1.0));
    if length(right) < 0.01 {
        right = cross(up, vec3f(1.0, 0.0, 0.0));
    }
    right = normalize(right);
    let fwd = cross(right, up);
    let angle = atan2(dot(proj, fwd), dot(proj, right)) / TAU;
    return vec2f(angle, rad);
}

// Variant-specific size multiplier
fn scatter_size_mult(variant: u32) -> f32 {
    switch variant {
        case SCATTER_VARIANT_DUST: { return 2.0; }      // Larger particles
        case SCATTER_VARIANT_WINDOWS: { return 1.5; }   // Medium-large
        case SCATTER_VARIANT_BUBBLES: { return 2.5; }   // Large bubbles
        case SCATTER_VARIANT_EMBERS: { return 1.2; }    // Slightly larger
        case SCATTER_VARIANT_RAIN: { return 0.3; }      // Very thin
        case SCATTER_VARIANT_SNOW: { return 1.8; }      // Soft/large
        default: { return 1.0; }                         // STARS: default
    }
}

// Variant-specific twinkle modulation
fn scatter_twinkle_mod(variant: u32, twinkle_amount: f32, h: f32) -> f32 {
    if twinkle_amount <= 0.001 {
        return 1.0;
    }

    switch variant {
        case SCATTER_VARIANT_WINDOWS: {
            // No twinkle for windows
            return 1.0;
        }
        case SCATTER_VARIANT_DUST: {
            // Subtle twinkle
            let tw = 0.8 + 0.2 * sin(h * TAU);
            return mix(1.0, tw, twinkle_amount * 0.5);
        }
        case SCATTER_VARIANT_BUBBLES: {
            // Gentle shimmer
            let tw = 0.7 + 0.3 * sin(h * TAU);
            return mix(1.0, tw, twinkle_amount * 0.7);
        }
        case SCATTER_VARIANT_EMBERS: {
            // Flickering fade
            let tw = 0.3 + 0.7 * sin(h * TAU);
            return mix(1.0, tw * tw, twinkle_amount);
        }
        case SCATTER_VARIANT_RAIN: {
            // Slight variation
            return mix(1.0, 0.9 + 0.1 * sin(h * TAU), twinkle_amount);
        }
        case SCATTER_VARIANT_SNOW: {
            // Very subtle shimmer
            let tw = 0.9 + 0.1 * sin(h * TAU);
            return mix(1.0, tw, twinkle_amount * 0.3);
        }
        default: {
            // STARS: standard twinkle
            let tw = 0.5 + 0.5 * sin(h * TAU);
            return mix(1.0, tw, twinkle_amount);
        }
    }
}

// Variant-specific point shape (returns distance field modifier)
fn scatter_point_shape(variant: u32, dist: f32, size: f32, h: vec4f) -> f32 {
    switch variant {
        case SCATTER_VARIANT_WINDOWS: {
            // Rectangular shape approximation
            let rect_dist = max(abs(dist), abs(dist * (0.7 + h.y * 0.3)));
            return smoothstep(size, size * 0.1, rect_dist);
        }
        case SCATTER_VARIANT_RAIN: {
            // Elongated vertical streak
            let streak = smoothstep(size * 3.0, size * 0.1, dist);
            return streak * smoothstep(0.0, size * 0.5, dist + size * 0.5);
        }
        case SCATTER_VARIANT_EMBERS: {
            // Elongated with soft glow
            let core = smoothstep(size, size * 0.2, dist);
            let glow = smoothstep(size * 2.0, size * 0.5, dist) * 0.3;
            return core + glow;
        }
        case SCATTER_VARIANT_BUBBLES: {
            // Soft edge with highlight
            let soft = smoothstep(size, size * 0.6, dist);
            let highlight = smoothstep(size * 0.3, size * 0.1, dist) * 0.5;
            return soft * (1.0 + highlight);
        }
        default: {
            // Standard circular point (STARS, DUST, SNOW)
            return smoothstep(size, size * 0.3, dist);
        }
    }
}

fn eval_scatter(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract domain and variant from meta5
    let domain_id = instr_domain_id(instr);
    let variant_id = instr_variant_id(instr);

    // Reserved/unknown variants: no output.
    if variant_id > SCATTER_VARIANT_SNOW {
        return LayerSample(vec3f(0.0), 0.0);
    }

    let density = mix(1.0, 256.0, u8_to_01(instr_a(instr)));
    let base_size = mix(0.001, 0.05, u8_to_01(instr_b(instr)));
    let size = base_size * scatter_size_mult(variant_id);

    let pc = instr_c(instr);
    let twinkle = f32((pc >> 4u) & 0xFu) / 15.0;

    // Seed from param_d
    let seed = f32(instr_d(instr));

    // Decode axis direction for domain transforms
    let axis = decode_dir16(instr_dir16(instr));
    var domain_w = 1.0;

    var dir_s = dir;

    // Apply domain transform for point distribution
    var sample_coords = dir_s * density;

    switch domain_id {
        case SCATTER_DOMAIN_AXIS_CYL: {
            // Cylindrical: use UV coordinates for 2D cell distribution
            let uv = scatter_cyl_uv(dir_s, axis);
            sample_coords = vec3f(uv.x * density, uv.y * density * 0.5, seed);
            // Pole fade
            domain_w = smoothstep(0.95, 0.8, abs(uv.y));
        }
        case SCATTER_DOMAIN_AXIS_POLAR: {
            // Polar: use angle and radius for 2D cell distribution
            let uv = scatter_polar_uv(dir_s, axis);
            sample_coords = vec3f(uv.x * density, uv.y * density, seed);
            // Center fade
            domain_w = smoothstep(0.05, 0.2, uv.y);
        }
        case SCATTER_DOMAIN_TANGENT_LOCAL: {
            // Tangent plane: project onto plane perpendicular to axis
            let proj = dir_s - axis * dot(dir_s, axis);
            sample_coords = proj * density + vec3f(seed);
            // Fade near axis direction
            domain_w = smoothstep(0.95, 0.8, abs(dot(dir_s, axis)));
        }
        default: {
            // DIRECT3D: direction sphere (default behavior)
            sample_coords = dir_s * density;
        }
    }

    // Cell on direction sphere (cheap hash distribution).
    let cell = floor(sample_coords);
    let h = hash3(cell + vec3f(seed));
    let point_offset = h.xyz * 2.0 - 1.0;
    var v = cell + point_offset * 0.5;
    if length(v) < 1e-5 {
        v = vec3f(1.0, 0.0, 0.0);
    }
    let point_dir = normalize(v);

    let dist = acos(epu_saturate(dot(dir_s, point_dir)));

    // Variant-specific point shape
    let point = scatter_point_shape(variant_id, dist, size, h);

    // Variant-specific twinkle
    let tw = scatter_twinkle_mod(variant_id, twinkle, h.w);

    // color_a = primary point color, color_b = variation color
    // Mix between colors based on hash for per-point variation
    let point_rgb = instr_color_a(instr);
    let var_rgb = instr_color_b(instr);
    let rgb = mix(point_rgb, var_rgb, h.x);

    let intensity = u8_to_01(instr_intensity(instr));
    let alpha = instr_alpha_a_f32(instr);
    return LayerSample(rgb, point * intensity * tw * alpha * region_w * domain_w);
}
