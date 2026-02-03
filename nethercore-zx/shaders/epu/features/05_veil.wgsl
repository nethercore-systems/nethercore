// @epu_meta_begin
// opcode = 0x0D
// name = VEIL
// kind = radiance
// variants = [CURTAINS, PILLARS, LASER_BARS, RAIN_WALL, SHARDS]
// domains = [DIRECT3D, AXIS_CYL, AXIS_POLAR, TANGENT_LOCAL]
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="count", map="u8_lerp", min=2.0, max=32.0 }
// field param_b = { label="thickness", map="u8_lerp", min=0.002, max=0.5 }
// field param_c = { label="sway", map="u8_01" }
// field param_d = { label="phase", map="u8_01" }
// @epu_meta_end

// ============================================================================
// VEIL - Curtain/Ribbon Effects (Curtains, Pillars, Laser Bars, Rain Wall, Shards)
// Opcode: 0x0D
// Role: Radiance (additive feature layer)
//
// Packed fields:
//   color_a: Ribbon/bar color (RGB24)
//   color_b: Edge/glow color (RGB24)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Ribbon count (0..255 -> 2..32)
//   param_b: Thickness (0..255 -> 0.002..0.5/ribbon_count, scales with spacing)
//   param_c: Curvature/sway amplitude (0..255 -> 0..1)
//   param_d: Phase (0..255 -> 0..1, used by RAIN_WALL for animation)
//   direction: Cylinder/polar axis (oct-u16)
//   alpha_a: Ribbon alpha (0..15 -> 0..1)
//   alpha_b: Glow alpha (0..15 -> 0..1)
//
// Meta (via meta5):
//   domain_id: 1 AXIS_CYL, 2 AXIS_POLAR, 3 TANGENT_LOCAL
//   variant_id: 0 CURTAINS, 1 PILLARS, 2 LASER_BARS, 3 RAIN_WALL, 4 SHARDS
// ============================================================================

// Domain IDs for VEIL
const VEIL_DOMAIN_AXIS_CYL: u32 = 1u;      // Cylindrical (azimuth, height)
const VEIL_DOMAIN_AXIS_POLAR: u32 = 2u;    // Polar (angle, radius from axis)
const VEIL_DOMAIN_TANGENT_LOCAL: u32 = 3u; // Tangent-local (gnomonic projection)

// Variant IDs for VEIL
const VEIL_VARIANT_CURTAINS: u32 = 0u;     // Soft edges, variable thickness, transparency gradient
const VEIL_VARIANT_PILLARS: u32 = 1u;      // Hard edges, uniform thickness, pole fade
const VEIL_VARIANT_LASER_BARS: u32 = 2u;   // Very thin, bright core with wide glow
const VEIL_VARIANT_RAIN_WALL: u32 = 3u;    // Many thin bars with v-scroll, flicker
const VEIL_VARIANT_SHARDS: u32 = 4u;       // Irregular thickness, sharp edges, crystalline

// Deterministic hash for ribbon variation (2D -> 3D)
fn veil_hash23(p: vec2f) -> vec3f {
    var p3 = fract(vec3f(p.xyx) * vec3f(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yzz) * p3.zyx);
}

// Deterministic hash for single value (1D -> 1D)
fn veil_hash11(p: f32) -> f32 {
    var p2 = fract(p * 0.1031);
    p2 *= p2 + 33.33;
    p2 *= p2 + p2;
    return fract(p2);
}

// Map direction to cylindrical UV with axis
fn veil_cyl_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build tangent basis perpendicular to axis
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(up, axis));
    let b = normalize(cross(axis, t));

    // Project dir onto the plane perpendicular to axis
    let x = dot(dir, t);
    let z = dot(dir, b);
    let y = dot(dir, axis);

    // Azimuth angle [0, 1] and height [-1, 1]
    let u = atan2(x, z) / TAU + 0.5;
    let v = y;
    return vec2f(u, v);
}

// Map direction to polar UV with axis
fn veil_polar_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build tangent basis perpendicular to axis
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(up, axis));
    let b = normalize(cross(axis, t));

    // Project dir onto the plane perpendicular to axis
    let x = dot(dir, t);
    let z = dot(dir, b);
    let y = dot(dir, axis);

    // Angle around axis [0, 1] and radial distance from axis [0, 1]
    let angle = atan2(x, z) / TAU + 0.5;
    let rad = acos(clamp(y, -1.0, 1.0)) / PI; // 0 at axis, 1 at opposite
    return vec2f(angle, rad);
}

// Map direction to tangent-local UV (gnomonic projection)
fn veil_tangent_uv(dir: vec3f, center: vec3f) -> vec3f {
    // Returns (u, v, visibility_weight)
    let d = dot(dir, center);
    if d <= 0.0 {
        return vec3f(0.0, 0.0, 0.0);
    }

    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));

    let proj = dir - center * d;
    let u = dot(proj, t) / d;
    let v = dot(proj, b) / d;

    let grazing_w = smoothstep(0.1, 0.3, d);
    return vec3f(u, v, grazing_w);
}

// Compute ribbon distance with wrapping (handles seam correctly)
fn ribbon_dist_wrapped(u: f32, center_u: f32) -> f32 {
    // Distance with wrap-around (both u and center_u in [0, 1])
    let d = abs(fract(u - center_u + 0.5) - 0.5);
    return d;
}

// CURTAINS variant: soft edges, variable thickness per-ribbon, slight transparency gradient along v
fn eval_veil_curtains(
    u: f32,
    v: f32,
    ribbon_count: u32,
    base_thickness: f32,
    curvature: f32,
) -> vec3f {
    // Returns (min_dist, ribbon_mask_contribution, glow_factor)
    var min_dist = 1000.0;
    var best_hash_x = 0.0;

    let u_scrolled = fract(u);

    for (var i = 0u; i < ribbon_count; i++) {
        let fi = f32(i);
        let center_u = (fi + 0.5) / f32(ribbon_count);
        let h = veil_hash23(vec2f(fi * 7.3, 13.7));

        // Variable thickness per ribbon (0.7x to 1.3x base)
        let thickness_var = base_thickness * (0.7 + h.y * 0.6);

        // Curvature/sway offset: sin wave along v with per-ribbon phase
        let sway_offset = curvature * 0.1 * sin(v * PI * 2.0 + h.x * TAU);

        let d = ribbon_dist_wrapped(u_scrolled, center_u + sway_offset);

        if d < min_dist {
            min_dist = d;
            best_hash_x = h.x;
        }
    }

    // Soft edges using larger smoothstep range
    let aa_width = 0.005;
    let thickness = base_thickness;
    let ribbon_mask = 1.0 - smoothstep(thickness * 0.5 - aa_width, thickness + aa_width, min_dist);

    // Transparency gradient along v (slightly more transparent at top and bottom)
    let v_fade = 1.0 - 0.3 * pow(abs(v), 2.0);

    // Glow extends beyond ribbon
    let glow = smoothstep(thickness * 3.0, thickness * 0.8, min_dist) * (1.0 - ribbon_mask);

    return vec3f(min_dist, ribbon_mask * v_fade, glow);
}

// PILLARS variant: hard edges, uniform thickness, pole fade
fn eval_veil_pillars(
    u: f32,
    v: f32,
    ribbon_count: u32,
    thickness: f32,
    curvature: f32
) -> vec3f {
    var min_dist = 1000.0;

    let u_scrolled = fract(u);

    for (var i = 0u; i < ribbon_count; i++) {
        let fi = f32(i);
        let center_u = (fi + 0.5) / f32(ribbon_count);

        // No curvature for pillars (rigid vertical)
        let d = ribbon_dist_wrapped(u_scrolled, center_u);
        min_dist = min(min_dist, d);
    }

    // Hard edges with minimal AA
    let aa_width = 0.002;
    let ribbon_mask = 1.0 - smoothstep(thickness - aa_width, thickness + aa_width, min_dist);

    // No glow for pillars (architectural, solid appearance)
    let glow = 0.0;

    return vec3f(min_dist, ribbon_mask, glow);
}

// LASER_BARS variant: very thin, bright core with wide glow, no curvature
fn eval_veil_laser_bars(
    u: f32,
    v: f32,
    ribbon_count: u32,
    thickness: f32,
    curvature: f32
) -> vec3f {
    var min_dist = 1000.0;

    let u_scrolled = fract(u);

    // Laser bars are very thin (use 1/3 of base thickness for core)
    let core_thickness = thickness * 0.33;

    for (var i = 0u; i < ribbon_count; i++) {
        let fi = f32(i);
        let center_u = (fi + 0.5) / f32(ribbon_count);

        // No curvature for laser bars (straight lines)
        let d = ribbon_dist_wrapped(u_scrolled, center_u);
        min_dist = min(min_dist, d);
    }

    // Very sharp core
    let aa_width = 0.001;
    let core_mask = 1.0 - smoothstep(core_thickness - aa_width, core_thickness + aa_width, min_dist);

    // Wide glow around the core (extends to 5x core thickness)
    let glow_radius = core_thickness * 5.0;
    let glow = smoothstep(glow_radius, core_thickness, min_dist) * (1.0 - core_mask);

    // Bright core (intensity boost)
    let ribbon_mask = core_mask * 1.5;

    return vec3f(min_dist, ribbon_mask, glow);
}

// RAIN_WALL variant: many thin bars with per-bar phase offsets, slight flicker
fn eval_veil_rain_wall(
    u: f32,
    v: f32,
    ribbon_count: u32,
    thickness: f32,
    curvature: f32,
    phase: f32
) -> vec3f {
    var min_dist = 1000.0;
    var best_flicker = 1.0;

    // Rain wall uses more bars (multiply count by 2 for denser rain)
    let actual_count = ribbon_count * 2u;

    // Thin bars (half thickness)
    let bar_thickness = thickness * 0.4;

    let u_scrolled = fract(u);

    // Wind slant from curvature parameter
    let wind = (curvature - 0.5) * 0.15;

    for (var i = 0u; i < actual_count; i++) {
        let fi = f32(i);
        let center_u = (fi + 0.5) / f32(actual_count);
        let h = veil_hash23(vec2f(fi * 11.3, 17.7));

        // Per-bar v-position using deterministic phase offset
        let v01 = v * 0.5 + 0.5;

        // Each bar has different fall speed and starting offset
        let fall_speed = 0.3 + h.x * 1.4;
        let drop_pos = fract(h.y + phase * fall_speed);

        // Short streak segments
        let half_len = 0.02 + h.z * 0.06;
        let dv = abs(v01 - drop_pos);
        let bar_visible = 1.0 - smoothstep(half_len, half_len * 1.6, dv);

        let d = ribbon_dist_wrapped(u_scrolled + v * wind, center_u);

        if d < min_dist && bar_visible > 0.5 {
            min_dist = d;
            // Per-bar intensity variation (deterministic)
            best_flicker = 0.7 + 0.3 * sin(h.z * TAU);
        }
    }

    let aa_width = 0.002;
    let ribbon_mask = (1.0 - smoothstep(bar_thickness - aa_width, bar_thickness + aa_width, min_dist)) * best_flicker;

    // Subtle glow
    let glow = smoothstep(bar_thickness * 2.5, bar_thickness, min_dist) * (1.0 - clamp(ribbon_mask, 0.0, 1.0)) * 0.5;

    return vec3f(min_dist, ribbon_mask, glow);
}

// SHARDS variant: irregular thickness, sharp edges, crystalline color variation
fn eval_veil_shards(
    u: f32,
    v: f32,
    ribbon_count: u32,
    base_thickness: f32,
    curvature: f32
) -> vec3f {
    var min_dist = 1000.0;
    var best_hash = vec3f(0.0);

    let u_scrolled = fract(u);

    for (var i = 0u; i < ribbon_count; i++) {
        let fi = f32(i);
        let h = veil_hash23(vec2f(fi * 13.7, 19.3));

        // Irregular positioning (not evenly spaced)
        let center_u = fract(h.x + fi * 0.618033988749); // Golden ratio distribution

        // Highly variable thickness (0.3x to 2.0x base)
        let thickness_var = base_thickness * (0.3 + h.y * 1.7);

        // Angular offset (shards are tilted)
        let tilt = (h.z - 0.5) * curvature * 0.2;
        let tilted_u = u_scrolled + v * tilt;

        let d = ribbon_dist_wrapped(tilted_u, center_u);

        if d < min_dist {
            min_dist = d;
            best_hash = h;
        }
    }

    // Sharp edges (very small AA)
    let aa_width = 0.001;
    let thickness = base_thickness * (0.3 + best_hash.y * 1.7);
    let ribbon_mask = 1.0 - smoothstep(thickness - aa_width, thickness + aa_width, min_dist);

    // Crystalline glow with color variation factor stored in z
    let glow = smoothstep(thickness * 2.0, thickness * 0.5, min_dist) * (1.0 - ribbon_mask);

    // Store hash for color variation (will be used by caller)
    // We encode it in the glow value by scaling
    let color_var = best_hash.z;

    return vec3f(color_var, ribbon_mask, glow);
}

fn eval_veil(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract domain and variant from meta5
    let domain_id = instr_domain_id(instr);
    let variant_id = instr_variant_id(instr);

    // Extract parameters
    // param_a: Ribbon count (0..255 -> 2..32)
    let ribbon_count = 2u + (instr_a(instr) * 30u) / 255u;
    // param_b: Thickness (scales with ribbon spacing)
    let spacing = 1.0 / f32(ribbon_count);
    let max_thickness = max(0.05, spacing * 0.5);
    let thickness = mix(0.002, max_thickness, u8_to_01(instr_b(instr)));
    // param_c: Curvature/sway (0..255 -> 0..1)
    let curvature = u8_to_01(instr_c(instr));
    // param_d: Phase (0..255 -> 0..1, used by RAIN_WALL)
    let phase = u8_to_01(instr_d(instr));

    // Decode axis direction
    let axis = decode_dir16(instr_dir16(instr));

    // Map to 2D chart based on domain
    var uv = vec2f(0.0);
    var domain_w = 1.0;

    switch domain_id {
        case VEIL_DOMAIN_AXIS_CYL: {
            uv = veil_cyl_uv(dir, axis);
            // Pole fade at v near -1 or 1 (poles of cylinder)
            domain_w = smoothstep(0.95, 0.8, abs(uv.y));
        }
        case VEIL_DOMAIN_AXIS_POLAR: {
            uv = veil_polar_uv(dir, axis);
            // Axis fade near center (rad near 0)
            domain_w = smoothstep(0.05, 0.2, uv.y);
        }
        case VEIL_DOMAIN_TANGENT_LOCAL: {
            let result = veil_tangent_uv(dir, axis);
            uv = vec2f(result.x * 0.5 + 0.5, clamp(result.y, -1.0, 1.0));
            domain_w = result.z;
        }
        default: {
            // Default to AXIS_CYL with Y-up
            uv = veil_cyl_uv(dir, vec3f(0.0, 1.0, 0.0));
            domain_w = smoothstep(0.95, 0.8, abs(uv.y));
        }
    }

    let u = uv.x;
    let v = uv.y;

    // Evaluate variant-specific ribbon pattern
    var result = vec3f(0.0);

    switch variant_id {
        case VEIL_VARIANT_CURTAINS: {
            result = eval_veil_curtains(u, v, ribbon_count, thickness, curvature);
        }
        case VEIL_VARIANT_PILLARS: {
            result = eval_veil_pillars(u, v, ribbon_count, thickness, curvature);
        }
        case VEIL_VARIANT_LASER_BARS: {
            result = eval_veil_laser_bars(u, v, ribbon_count, thickness, curvature);
        }
        case VEIL_VARIANT_RAIN_WALL: {
            result = eval_veil_rain_wall(u, v, ribbon_count, thickness, curvature, phase);
        }
        case VEIL_VARIANT_SHARDS: {
            result = eval_veil_shards(u, v, ribbon_count, thickness, curvature);
        }
        default: {
            // Default to curtains
            result = eval_veil_curtains(u, v, ribbon_count, thickness, curvature);
        }
    }

    // result.x = variant-specific (distance or color_var for SHARDS)
    // result.y = ribbon_mask
    // result.z = glow

    let ribbon_mask = result.y;
    let glow = result.z;

    // Extract colors and alphas
    let ribbon_rgb = instr_color_a(instr);
    let glow_rgb = instr_color_b(instr);
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // For SHARDS variant, apply crystalline color variation
    var final_ribbon_rgb = ribbon_rgb;
    var final_glow_rgb = glow_rgb;
    if variant_id == VEIL_VARIANT_SHARDS {
        let color_var = result.x;
        // Shift hue slightly based on hash
        final_ribbon_rgb = mix(ribbon_rgb, glow_rgb, color_var * 0.5);
        final_glow_rgb = mix(glow_rgb, ribbon_rgb, color_var * 0.3);
    }

    // Blend colors
    let rgb = final_ribbon_rgb * clamp(ribbon_mask, 0.0, 1.0) + final_glow_rgb * glow;

    // Compute final weight
    let intensity = u8_to_01(instr_intensity(instr));
    let w = (clamp(ribbon_mask, 0.0, 1.0) * alpha_a + glow * alpha_b) * intensity * domain_w * region_w;

    return LayerSample(rgb, w);
}
