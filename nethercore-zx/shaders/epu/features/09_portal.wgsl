// ============================================================================
// PORTAL - Swirling Vortex/Portal Effect
// Opcode: 0x11
// Role: Radiance (additive feature layer)
//
// Creates surreal holes, tears, rifts, and vortexes in the sky using
// tangent-local SDF shapes. Used for horror voids, magical portals,
// dimensional cracks, and otherworldly effects.
//
// Packed fields:
//   color_a: Interior/void color (RGB24)
//   color_b: Edge glow color (RGB24)
//   intensity: Edge glow brightness (0..255 -> 0..2)
//   param_a: Size (0..255 -> 0.05..0.8 tangent units)
//   param_b: Edge glow width (0..255 -> 0.01..0.3)
//   param_c: Edge roughness for TEAR/CRACK/RIFT (0..255 -> 0..1)
//   param_d: Rotation speed for VORTEX (0..255 -> 0..2)
//   direction: Portal center (oct-u16)
//   alpha_a: Interior alpha (0..15 -> 0..1)
//   alpha_b: Edge glow alpha (0..15 -> 0..1)
//
// Meta (via meta5):
//   domain_id: 3 TANGENT_LOCAL (fixed; always uses tangent projection)
//   variant_id: 0 CIRCLE, 1 RECT, 2 TEAR, 3 VORTEX, 4 CRACK, 5 RIFT
// ============================================================================

// Variant IDs for PORTAL
const PORTAL_VARIANT_CIRCLE: u32 = 0u;   // Simple circular portal
const PORTAL_VARIANT_RECT: u32 = 1u;     // Rectangular portal
const PORTAL_VARIANT_TEAR: u32 = 2u;     // Circle with noise-displaced edge
const PORTAL_VARIANT_VORTEX: u32 = 3u;   // Circle with spiral warp animation
const PORTAL_VARIANT_CRACK: u32 = 4u;    // Elongated vertical jagged line
const PORTAL_VARIANT_RIFT: u32 = 5u;     // Horizontal tear with ragged edges

// Deterministic hash for portal noise (2D -> 1D)
fn portal_hash21(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.xyx) * 0.1031);
    let d = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z + d);
}

// 2D value noise for edge roughness
fn portal_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = portal_hash21(i);
    let b = portal_hash21(i + vec2f(1.0, 0.0));
    let c = portal_hash21(i + vec2f(0.0, 1.0));
    let d = portal_hash21(i + vec2f(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// CIRCLE variant: Simple circular portal
fn portal_sdf_circle(uv: vec2f, size: f32) -> f32 {
    return length(uv) - size;
}

// RECT variant: Rectangular portal
fn portal_sdf_rect(uv: vec2f, size: f32) -> f32 {
    // Aspect ratio 1:0.6 (wider than tall)
    return max(abs(uv.x) - size, abs(uv.y) - size * 0.6);
}

// TEAR variant: Circle with noise-displaced edge
fn portal_sdf_tear(uv: vec2f, size: f32, roughness: f32) -> f32 {
    let noise_val = portal_noise(uv * 8.0);
    return length(uv) - size + noise_val * roughness * 0.2;
}

// VORTEX variant: Circle SDF (spiral warp applied externally to UV)
fn portal_sdf_vortex(uv: vec2f, size: f32) -> f32 {
    return length(uv) - size;
}

// CRACK variant: Elongated vertical line with jagged edges
fn portal_sdf_crack(uv: vec2f, size: f32, roughness: f32) -> f32 {
    let noise_val = portal_noise(vec2f(uv.y * 16.0, 0.0));
    return abs(uv.x) - size * 0.1 + noise_val * roughness * 0.1;
}

// RIFT variant: Horizontal tear with ragged edges
fn portal_sdf_rift(uv: vec2f, size: f32, roughness: f32) -> f32 {
    let noise_val = portal_noise(vec2f(uv.x * 12.0, 0.0));
    return abs(uv.y) - size * 0.2 + noise_val * roughness * 0.15;
}

// Apply spiral warp for VORTEX variant
fn portal_apply_vortex_warp(uv: vec2f, rotation_speed: f32, time: f32) -> vec2f {
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);
    let warped_angle = angle + radius * rotation_speed * time;
    return vec2f(cos(warped_angle), sin(warped_angle)) * radius;
}

fn eval_portal(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant (domain_id is fixed to TANGENT_LOCAL for PORTAL)
    let variant_id = instr_variant_id(instr);

    // Decode portal center direction
    let center = decode_dir16(instr_dir16(instr));

    // Compute dot product with portal center
    let d = dot(dir, center);

    // Reject if behind the portal center (d <= 0)
    if d <= 0.0 { return LayerSample(vec3f(0.0), 0.0); }

    // Project onto tangent plane: gnomonic projection (tangent-local UV).
    // Divide by `d` so UV is unbounded as d -> 0 (approaching 90° from center).
    // (This avoids "portal rotates with world axes" artifacts.)
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));
    let uv = vec2f(dot(dir, t) / d, dot(dir, b) / d);

    // Compute grazing fade to prevent edge artifacts
    let grazing_w = smoothstep(0.1, 0.3, d);

    // Extract parameters
    // param_a: Size (0..255 -> 0.05..0.8)
    let size = mix(0.05, 0.8, u8_to_01(instr_a(instr)));

    // param_b: Edge glow width (0..255 -> 0.01..0.3)
    let edge_width = mix(0.01, 0.3, u8_to_01(instr_b(instr)));

    // param_c: Edge roughness (0..255 -> 0.0..1.0)
    let roughness = u8_to_01(instr_c(instr));

    // param_d: Rotation speed for VORTEX (0..255 -> 0.0..2.0)
    let rotation_speed = u8_to_01(instr_d(instr)) * 2.0;

    // Apply VORTEX warp if needed (before SDF evaluation)
    var warped_uv = uv;
    if variant_id == PORTAL_VARIANT_VORTEX {
        warped_uv = portal_apply_vortex_warp(uv, rotation_speed, time);
    }

    // Evaluate shape SDF by variant
    var sdf: f32;
    switch variant_id {
        case PORTAL_VARIANT_CIRCLE: {
            sdf = portal_sdf_circle(warped_uv, size);
        }
        case PORTAL_VARIANT_RECT: {
            sdf = portal_sdf_rect(warped_uv, size);
        }
        case PORTAL_VARIANT_TEAR: {
            sdf = portal_sdf_tear(warped_uv, size, roughness);
        }
        case PORTAL_VARIANT_VORTEX: {
            sdf = portal_sdf_vortex(warped_uv, size);
        }
        case PORTAL_VARIANT_CRACK: {
            sdf = portal_sdf_crack(warped_uv, size, roughness);
        }
        case PORTAL_VARIANT_RIFT: {
            sdf = portal_sdf_rift(warped_uv, size, roughness);
        }
        default: {
            // Fallback to CIRCLE for reserved variants
            sdf = portal_sdf_circle(warped_uv, size);
        }
    }

    // Compute AA width based on projected pixel size
    // Use a fixed AA width since fwidth is not available in compute shaders
    let aa_width = 0.01;

    // Compute interior mask with smooth anti-aliasing
    let interior = smoothstep(aa_width, -aa_width, sdf);

    // Compute edge glow: visible outside the shape, fading with distance
    let edge = smoothstep(edge_width, 0.0, sdf) * (1.0 - interior);

    // Extract colors
    let color_a = instr_color_a(instr);  // Interior/void color
    let color_b = instr_color_b(instr);  // Edge glow color
    let intensity = u8_to_01(instr_intensity(instr)) * 2.0; // 0..2 range
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // Blend colors: interior + edge glow
    let rgb = color_a * interior + color_b * edge * intensity;

    // Compute final weight
    let w = (interior * alpha_a + edge * alpha_b) * grazing_w * region_w;

    return LayerSample(rgb, w);
}
