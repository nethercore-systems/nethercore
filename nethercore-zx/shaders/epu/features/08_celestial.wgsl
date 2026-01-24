// ============================================================================
// CELESTIAL - Moon/Sun/Planet Bodies
// Opcode: 0x10
// Role: Radiance (additive feature layer)
//
// Packed fields:
//   color_a: Body surface color (RGB24)
//   color_b: Atmosphere/corona/ring color (RGB24)
//   intensity: Overall brightness (0..255 -> 0..2) - >1 for suns
//   param_a: Angular size (0..255 -> 0.5..45 degrees)
//   param_b: Limb darkening exponent (0..255 -> 0.5..4.0)
//   param_c: Phase angle (0..255 -> 0..360 degrees) - MOON/PLANET
//   param_d: Variant-specific (corona extent, band count, ring tilt, etc.)
//   direction: Body center (oct-u16)
//   alpha_a: Body alpha (0..15 -> 0..1)
//   alpha_b: Atmosphere/ring alpha (0..15 -> 0..1)
//
// Meta (via meta5):
//   domain_id: Ignored (always DIRECT3D spherical)
//   variant_id: 0 MOON, 1 SUN, 2 PLANET, 3 GAS_GIANT, 4 RINGED, 5 BINARY, 6 ECLIPSE
// ============================================================================

// Variant IDs for CELESTIAL
const CELESTIAL_VARIANT_MOON: u32 = 0u;       // Cratered surface, phase illumination
const CELESTIAL_VARIANT_SUN: u32 = 1u;        // Corona/glow, limb darkening
const CELESTIAL_VARIANT_PLANET: u32 = 2u;     // Solid body with optional cloud bands
const CELESTIAL_VARIANT_GAS_GIANT: u32 = 3u;  // Jupiter-like horizontal stripes
const CELESTIAL_VARIANT_RINGED: u32 = 4u;     // Saturn-like with tilted rings
const CELESTIAL_VARIANT_BINARY: u32 = 5u;     // Two bodies (primary + secondary)
const CELESTIAL_VARIANT_ECLIPSE: u32 = 6u;    // Solar eclipse with corona

// Deterministic hash for celestial noise (2D -> 1D)
fn celestial_hash21(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.xyx) * 0.1031);
    let d = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z + d);
}

// 2D value noise for crater/surface detail
fn celestial_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = celestial_hash21(i);
    let b = celestial_hash21(i + vec2f(1.0, 0.0));
    let c = celestial_hash21(i + vec2f(0.0, 1.0));
    let d = celestial_hash21(i + vec2f(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// FBM noise for surface detail (2 octaves for mip stability)
fn celestial_fbm(p: vec2f) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pp = p;

    for (var i = 0u; i < 2u; i++) {
        value += amplitude * celestial_noise(pp);
        amplitude *= 0.5;
        pp = pp * 2.0 + vec2f(17.3, 31.7);
    }

    return value;
}

// Compute surface UV from direction and body center
// Returns UV coordinates on the spherical disk surface
fn celestial_surface_uv(dir: vec3f, body_dir: vec3f, r: f32) -> vec2f {
    // Build tangent frame around body direction
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let tangent = normalize(cross(hint, body_dir));
    let bitangent = cross(body_dir, tangent);

    // Project direction onto tangent plane
    let local_dir = dir - body_dir * dot(dir, body_dir);
    let u = dot(local_dir, tangent);
    let v = dot(local_dir, bitangent);

    // Scale by radius for consistent surface mapping
    return vec2f(u, v) / max(r, 0.001);
}

// MOON variant: Cratered surface with phase illumination
fn eval_celestial_moon(
    r: f32,
    surface_uv: vec2f,
    phase_factor: f32,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    alpha_b: f32
) -> LayerSample {
    // Crater noise: low-frequency bumps for craters
    let crater_noise = celestial_fbm(surface_uv * 8.0);
    let crater_dark = 1.0 - crater_noise * 0.3;

    // Surface shading: limb darkening * phase * crater detail
    let surface = limb * phase_factor * crater_dark;

    // Disk mask with soft edge
    let disk = smoothstep(1.05, 0.95, r);

    // Subtle atmosphere glow beyond disk
    let atmo_mask = smoothstep(1.3, 1.0, r) * (1.0 - disk);
    let atmo = color_b * atmo_mask * alpha_b * 0.5;

    let rgb = color_a * surface * disk + atmo;
    let w = disk + atmo_mask * alpha_b;

    return LayerSample(rgb, w);
}

// SUN variant: Corona/glow with limb darkening
fn eval_celestial_sun(
    r: f32,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    intensity: f32,
    corona_extent: f32,
    alpha_b: f32
) -> LayerSample {
    // Disk with limb darkening (suns have pronounced limb darkening)
    let disk = smoothstep(1.02, 0.98, r);
    let surface = limb * intensity;

    // Corona glow beyond disk: soft exponential falloff
    let corona_r = r / corona_extent;
    let corona = exp(-corona_r * 2.0) * (1.0 - disk * 0.8);
    let corona_color = color_b * corona * alpha_b * intensity;

    let rgb = color_a * surface * disk + corona_color;
    let w = disk + corona * alpha_b;

    return LayerSample(rgb, w);
}

// PLANET variant: Solid body with optional cloud bands
fn eval_celestial_planet(
    r: f32,
    surface_uv: vec2f,
    phase_factor: f32,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    band_count: f32,
    alpha_b: f32
) -> LayerSample {
    // Cloud bands: horizontal stripes based on latitude (uv.y)
    let latitude = surface_uv.y * 0.5 + 0.5; // Normalize to 0..1
    var band_factor = 1.0;
    if band_count > 0.5 {
        let band = sin(latitude * band_count * PI) * 0.5 + 0.5;
        band_factor = mix(0.9, 1.0, band);
    }

    // Surface noise for terrain variation
    let terrain = celestial_fbm(surface_uv * 4.0);
    let terrain_factor = 0.85 + terrain * 0.3;

    // Surface shading
    let surface = limb * phase_factor * terrain_factor * band_factor;

    // Disk mask
    let disk = smoothstep(1.05, 0.95, r);

    // Atmosphere glow
    let atmo_mask = smoothstep(1.25, 1.0, r) * (1.0 - disk);
    let atmo = color_b * atmo_mask * alpha_b;

    let rgb = color_a * surface * disk + atmo;
    let w = disk + atmo_mask * alpha_b;

    return LayerSample(rgb, w);
}

// GAS_GIANT variant: Jupiter-like horizontal stripes
fn eval_celestial_gas_giant(
    r: f32,
    surface_uv: vec2f,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    band_count: f32,
    alpha_b: f32
) -> LayerSample {
    // Prominent horizontal bands based on latitude
    let latitude = surface_uv.y;
    let band = sin(latitude * band_count * PI) * 0.5 + 0.5;

    // Mix between two colors for band pattern
    let band_color = mix(color_a, color_b, band);

    // Add turbulence noise to band edges
    let turbulence = celestial_noise(surface_uv * 12.0 + vec2f(latitude * 4.0, 0.0)) * 0.15;
    let disturbed_band = mix(color_a, color_b, epu_saturate(band + turbulence - 0.075));

    // Surface shading with limb darkening
    let surface = disturbed_band * limb;

    // Disk mask
    let disk = smoothstep(1.05, 0.95, r);

    // Subtle atmosphere
    let atmo_mask = smoothstep(1.15, 1.0, r) * (1.0 - disk);
    let atmo_color = mix(color_a, color_b, 0.5) * atmo_mask * alpha_b * 0.3;

    let rgb = surface * disk + atmo_color;
    let w = disk + atmo_mask * alpha_b * 0.3;

    return LayerSample(rgb, w);
}

// RINGED variant: Saturn-like with tilted rings
fn eval_celestial_ringed(
    dir: vec3f,
    body_dir: vec3f,
    r: f32,
    angular_size_rad: f32,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    ring_tilt_deg: f32,
    alpha_a: f32,
    alpha_b: f32
) -> LayerSample {
    // Ring geometry: rings are in a plane tilted from viewer
    // Ring tilt: 0 = edge-on, 90 = face-on
    let ring_tilt_rad = ring_tilt_deg * PI / 180.0;
    let tilt_factor = sin(ring_tilt_rad);

    // Build ring plane normal (tilted from body direction)
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let ring_axis = normalize(cross(hint, body_dir));
    // Rotate body_dir around ring_axis by tilt angle
    let ring_normal = body_dir * cos(ring_tilt_rad) + cross(ring_axis, body_dir) * sin(ring_tilt_rad);

    // Compute angular distance from body center
    let body_dot = epu_saturate(dot(dir, body_dir));
    let angle = acos(body_dot);

    // Ring radii in angular units (inner at 1.5x disk, outer at 2.5x disk)
    let inner_r = 1.5;
    let outer_r = 2.5;

    // Distance from ring plane
    let ring_plane_dist = abs(dot(dir - body_dir * body_dot, ring_normal));
    let in_ring_plane = ring_plane_dist < angular_size_rad * 0.3 * tilt_factor;

    // Ring mask: annular region
    let ring_mask = smoothstep(inner_r - 0.1, inner_r + 0.1, r) *
                    smoothstep(outer_r + 0.1, outer_r - 0.1, r);
    let ring_visible = select(0.0, ring_mask, in_ring_plane && r > 1.05);

    // Ring brightness varies with radius (Cassini division effect)
    let ring_r_norm = (r - inner_r) / (outer_r - inner_r);
    let ring_bands = sin(ring_r_norm * 8.0 * PI) * 0.3 + 0.7;
    // Gap at ~0.5 for Cassini-like division
    let cassini_gap = smoothstep(0.45, 0.5, ring_r_norm) * smoothstep(0.55, 0.5, ring_r_norm);
    let ring_brightness = ring_bands * (1.0 - cassini_gap * 0.8);

    // Planet disk (behind rings at some angles)
    let disk = smoothstep(1.05, 0.95, r);
    let planet_surface = color_a * limb * disk;

    // Ring color
    let ring_color = color_b * ring_visible * ring_brightness * tilt_factor;

    // Combine: rings can be in front of or behind the planet depending on geometry
    // Simplified: rings always rendered on top for visual clarity
    let rgb = planet_surface * alpha_a + ring_color * alpha_b;
    let w = disk * alpha_a + ring_visible * alpha_b;

    return LayerSample(rgb, w);
}

// BINARY variant: Two bodies (primary + secondary)
fn eval_celestial_binary(
    dir: vec3f,
    body_dir: vec3f,
    angular_size_rad: f32,
    limb_exp: f32,
    color_a: vec3f,
    color_b: vec3f,
    size_ratio: f32,
    alpha_a: f32,
    alpha_b: f32
) -> LayerSample {
    // Offset secondary body by small angle (1.5x angular size)
    let offset_angle = angular_size_rad * 1.5;
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let offset_axis = normalize(cross(hint, body_dir));

    // Secondary body direction (offset from primary)
    let secondary_dir = normalize(body_dir + offset_axis * sin(offset_angle));

    // Primary body
    let primary_dot = epu_saturate(dot(dir, body_dir));
    let primary_angle = acos(primary_dot);
    let primary_r = primary_angle / angular_size_rad;
    let primary_disk = smoothstep(1.05, 0.95, primary_r);
    let primary_limb = pow(epu_saturate(1.0 - primary_r), limb_exp);

    // Secondary body (smaller based on size_ratio)
    let secondary_size = angular_size_rad * size_ratio;
    let secondary_dot = epu_saturate(dot(dir, secondary_dir));
    let secondary_angle = acos(secondary_dot);
    let secondary_r = secondary_angle / secondary_size;
    let secondary_disk = smoothstep(1.05, 0.95, secondary_r);
    let secondary_limb = pow(epu_saturate(1.0 - secondary_r), limb_exp);

    // Combine both bodies
    let primary_rgb = color_a * primary_limb * primary_disk;
    let secondary_rgb = color_b * secondary_limb * secondary_disk;

    // Secondary is behind primary if overlapping (occlusion)
    let occluded = primary_disk > 0.5 && secondary_disk > 0.5;
    let final_secondary = select(secondary_rgb * alpha_b, vec3f(0.0), occluded);

    let rgb = primary_rgb * alpha_a + final_secondary;
    let w = primary_disk * alpha_a + secondary_disk * alpha_b * select(1.0, 0.0, occluded);

    return LayerSample(rgb, w);
}

// ECLIPSE variant: Solar eclipse with corona
fn eval_celestial_eclipse(
    r: f32,
    color_a: vec3f,
    color_b: vec3f,
    corona_brightness: f32,
    alpha_a: f32,
    alpha_b: f32
) -> LayerSample {
    // Dark disk (moon blocking sun)
    let disk = smoothstep(1.02, 0.98, r);

    // Corona glow: bright ring just outside disk
    let corona_inner = 1.0;
    let corona_outer = 2.5;
    let corona_r = (r - corona_inner) / (corona_outer - corona_inner);
    let corona = exp(-corona_r * 3.0) * step(corona_inner, r) * corona_brightness;

    // Diamond ring effect: bright point at edge
    let diamond = exp(-pow((r - 1.0) * 20.0, 2.0)) * 2.0;

    // Streamers: radial patterns in corona (simplified)
    let streamer = (1.0 - disk) * corona * 0.5;

    // Dark disk interior with very faint color_a (umbra not pure black)
    let umbra_color = color_a * 0.05 * disk;

    // Corona color
    let corona_color = color_b * (corona + diamond + streamer);

    let rgb = umbra_color * alpha_a + corona_color * alpha_b;
    let w = disk * alpha_a * 0.1 + (corona + diamond) * alpha_b;

    return LayerSample(rgb, w);
}

fn eval_celestial(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant (domain_id is ignored for CELESTIAL)
    let variant_id = instr_variant_id(instr);

    // Decode body direction
    let body_dir = decode_dir16(instr_dir16(instr));

    // Compute angular distance from body center
    let body_dot = epu_saturate(dot(dir, body_dir));
    let angle = acos(body_dot);

    // Extract parameters
    // param_a: Angular size (0..255 -> 0.5..45 degrees)
    let angular_size_deg = mix(0.5, 45.0, u8_to_01(instr_a(instr)));
    let angular_size_rad = angular_size_deg * PI / 180.0;

    // param_b: Limb darkening exponent (0..255 -> 0.5..4.0)
    let limb_exp = mix(0.5, 4.0, u8_to_01(instr_b(instr)));

    // param_c: Phase angle (0..255 -> 0..360 degrees)
    let phase_deg = u8_to_01(instr_c(instr)) * 360.0;
    let phase_rad = phase_deg * PI / 180.0;

    // param_d: Variant-specific parameter
    let param_d_raw = u8_to_01(instr_d(instr));

    // Convert angle to radii (normalized disk distance)
    let r = angle / angular_size_rad;

    // Compute limb darkening: pow(1 - r, exponent) for r < 1
    let limb = pow(epu_saturate(1.0 - r), limb_exp);

    // Compute phase illumination (for MOON/PLANET)
    // Phase: 0 = full, 90 = half, 180 = new
    // Sun direction rotated from body_dir by phase angle
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let phase_axis = normalize(cross(hint, body_dir));
    let sun_dir = normalize(body_dir * cos(phase_rad) + cross(phase_axis, body_dir) * sin(phase_rad));

    // Local surface normal approximation (points outward from disk center)
    let surface_normal = normalize(dir - body_dir * body_dot + body_dir * 0.3);
    let phase_factor = epu_saturate(dot(surface_normal, sun_dir) * 0.5 + 0.5);

    // Compute surface UV for detail texturing
    let surface_uv = celestial_surface_uv(dir, body_dir, r);

    // Extract colors
    let color_a = instr_color_a(instr);  // Body surface color
    let color_b = instr_color_b(instr);  // Atmosphere/corona/ring color
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0; // 0..2 range
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // Evaluate variant-specific rendering
    var sample = LayerSample(vec3f(0.0), 0.0);

    switch variant_id {
        case CELESTIAL_VARIANT_MOON: {
            sample = eval_celestial_moon(r, surface_uv, phase_factor, limb, color_a, color_b, alpha_b);
        }
        case CELESTIAL_VARIANT_SUN: {
            // param_d: Corona extent (0..255 -> 1.0..3.0)
            let corona_extent = mix(1.0, 3.0, param_d_raw);
            sample = eval_celestial_sun(r, limb, color_a, color_b, intensity, corona_extent, alpha_b);
        }
        case CELESTIAL_VARIANT_PLANET: {
            // param_d: Cloud band count (0..255 -> 0..8)
            let band_count = param_d_raw * 8.0;
            sample = eval_celestial_planet(r, surface_uv, phase_factor, limb, color_a, color_b, band_count, alpha_b);
        }
        case CELESTIAL_VARIANT_GAS_GIANT: {
            // param_d: Horizontal band count (0..255 -> 2..16)
            let band_count = mix(2.0, 16.0, param_d_raw);
            sample = eval_celestial_gas_giant(r, surface_uv, limb, color_a, color_b, band_count, alpha_b);
        }
        case CELESTIAL_VARIANT_RINGED: {
            // param_d: Ring tilt (0..255 -> 0..90 degrees)
            let ring_tilt = param_d_raw * 90.0;
            sample = eval_celestial_ringed(dir, body_dir, r, angular_size_rad, limb, color_a, color_b, ring_tilt, alpha_a, alpha_b);
        }
        case CELESTIAL_VARIANT_BINARY: {
            // param_d: Secondary size ratio (0..255 -> 0.2..2.0)
            let size_ratio = mix(0.2, 2.0, param_d_raw);
            sample = eval_celestial_binary(dir, body_dir, angular_size_rad, limb_exp, color_a, color_b, size_ratio, alpha_a, alpha_b);
        }
        case CELESTIAL_VARIANT_ECLIPSE: {
            // param_d: Corona brightness (0..255 -> 1.0..2.5)
            let corona_brightness = mix(1.0, 2.5, param_d_raw);
            sample = eval_celestial_eclipse(r, color_a, color_b, corona_brightness, alpha_a, alpha_b);
        }
        default: {
            // Fallback to MOON for reserved variants
            sample = eval_celestial_moon(r, surface_uv, phase_factor, limb, color_a, color_b, alpha_b);
        }
    }

    // Apply intensity and region weight
    let final_w = sample.w * intensity * alpha_a * region_w;

    return LayerSample(sample.rgb * intensity, final_w);
}
