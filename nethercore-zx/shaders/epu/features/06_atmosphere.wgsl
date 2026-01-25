// ============================================================================
// ATMOSPHERE - Advanced Fog with Scattering
// Opcode: 0x0E
// Role: Radiance (additive feature layer)
//
// Packed fields:
//   color_a: Zenith/overhead tint (RGB24)
//   color_b: Horizon tint (RGB24)
//   intensity: Overall strength (0..255 -> 0..1)
//   param_a: Falloff exponent (0..255 -> 0.5..8.0)
//   param_b: Horizon Y threshold (0..255 -> -1..1)
//   param_c: Mie concentration (0..255 -> 0..2) - variants 2,3
//   param_d: Mie exponent (0..255 -> 4..128) - variants 2,3
//   direction: Sun direction (oct-u16) - variants 2,3
//   alpha_a: Coverage alpha (0..15 -> 0..1)
//   alpha_b: Unused (set to 0)
//
// Meta (via meta5):
//   domain_id: Ignored (always uses enclosure up vector)
//   variant_id: 0 ABSORPTION, 1 RAYLEIGH, 2 MIE, 3 FULL, 4 ALIEN
//
// ============================================================================

// Variant IDs for ATMOSPHERE
const ATMOSPHERE_VARIANT_ABSORPTION: u32 = 0u;  // Multiplicative fog
const ATMOSPHERE_VARIANT_RAYLEIGH: u32 = 1u;    // Blue sky gradient
const ATMOSPHERE_VARIANT_MIE: u32 = 2u;         // Sun halo/glow
const ATMOSPHERE_VARIANT_FULL: u32 = 3u;        // Combined Rayleigh + Mie
const ATMOSPHERE_VARIANT_ALIEN: u32 = 4u;       // Non-physical sin-based gradient

// ABSORPTION variant: Multiplicative fog
// factor = 1 - (1 - t) * intensity
// Used with BLEND_MULTIPLY for absorption/darkening effect
fn eval_atmosphere_absorption(
    t: f32,
    intensity: f32,
    color_a: vec3f,
    color_b: vec3f
) -> LayerSample {
    // Blend between horizon (color_b) and zenith (color_a) based on altitude
    let gradient = mix(color_b, color_a, t);

    // Absorption factor: less absorption at zenith (t=1), more at horizon (t=0)
    let factor = 1.0 - (1.0 - t) * intensity;

    // Output gradient color with absorption weight
    // Caller should use BLEND_MULTIPLY for proper absorption
    return LayerSample(gradient, 1.0 - factor);
}

// RAYLEIGH variant: Blue sky gradient (additive tint)
// rgb = rayleigh * intensity
fn eval_atmosphere_rayleigh(
    t: f32,
    intensity: f32,
    color_a: vec3f,
    color_b: vec3f
) -> LayerSample {
    // Rayleigh tint: horizon color blends to zenith color based on altitude
    let rayleigh = mix(color_b, color_a, t);

    // Output as additive tint
    return LayerSample(rayleigh, intensity);
}

// MIE variant: Sun halo/glow (additive)
// rgb = color_a * mie * intensity
fn eval_atmosphere_mie(
    dir: vec3f,
    t: f32,
    intensity: f32,
    color_a: vec3f,
    sun_dir: vec3f,
    mie_concentration: f32,
    mie_exponent: f32
) -> LayerSample {
    // Compute sun alignment
    let sun_dot = epu_saturate(dot(dir, sun_dir));

    // Mie halo: exponential falloff from sun direction
    let mie = pow(sun_dot, mie_exponent) * mie_concentration;

    // Output sun glow color
    return LayerSample(color_a, mie * intensity);
}

// FULL variant: Combined Rayleigh + Mie
// rgb = rayleigh * intensity + color_a * mie * intensity * 0.5
fn eval_atmosphere_full(
    dir: vec3f,
    t: f32,
    intensity: f32,
    color_a: vec3f,
    color_b: vec3f,
    sun_dir: vec3f,
    mie_concentration: f32,
    mie_exponent: f32
) -> LayerSample {
    // Rayleigh component
    let rayleigh = mix(color_b, color_a, t);

    // Mie component
    let sun_dot = epu_saturate(dot(dir, sun_dir));
    let mie = pow(sun_dot, mie_exponent) * mie_concentration;

    // Combined: Rayleigh + attenuated Mie (0.5 factor to balance)
    let rgb = rayleigh + color_a * mie * 0.5;

    return LayerSample(rgb, intensity);
}

// ALIEN variant: Non-physical gradient with sin cycling
// rgb = mix(color_b, color_a, sin(t * PI) * 0.5 + 0.5) * intensity
fn eval_atmosphere_alien(
    t: f32,
    intensity: f32,
    color_a: vec3f,
    color_b: vec3f
) -> LayerSample {
    // Sin-based oscillation: creates color band that peaks at mid-altitude
    let sin_factor = sin(t * PI) * 0.5 + 0.5;

    // Blend colors with oscillating factor
    let rgb = mix(color_b, color_a, sin_factor);

    return LayerSample(rgb, intensity);
}

fn eval_atmosphere(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant from meta5 (domain_id is ignored)
    let variant_id = instr_variant_id(instr);

    // Extract common parameters
    let color_a = instr_color_a(instr);  // Zenith color
    let color_b = instr_color_b(instr);  // Horizon color
    let intensity = u8_to_01(instr_intensity(instr));
    let alpha_a = instr_alpha_a_f32(instr);

    // param_a: Falloff exponent (0..255 -> 0.5..8.0)
    let falloff = mix(0.5, 8.0, u8_to_01(instr_a(instr)));

    // param_b: Horizon Y threshold (0..255 -> -1.0..1.0)
    let horizon_y = mix(-1.0, 1.0, u8_to_01(instr_b(instr)));

    // Get up vector from enclosure state
    let up = enc.up;

    // Compute altitude: y = dot(dir, up)
    let y = dot(dir, up);

    // Compute altitude blend: t = pow(saturate((y - horizon_y + 1) * 0.5), falloff)
    // This maps y from [-1, 1] relative to horizon_y to [0, 1], then applies falloff curve
    let raw_t = epu_saturate((y - horizon_y + 1.0) * 0.5);
    let t = pow(raw_t, falloff);

    // Evaluate variant-specific atmosphere
    var sample = LayerSample(vec3f(0.0), 0.0);

    switch variant_id {
        case ATMOSPHERE_VARIANT_ABSORPTION: {
            sample = eval_atmosphere_absorption(t, intensity, color_a, color_b);
        }
        case ATMOSPHERE_VARIANT_RAYLEIGH: {
            sample = eval_atmosphere_rayleigh(t, intensity, color_a, color_b);
        }
        case ATMOSPHERE_VARIANT_MIE: {
            // Extract Mie-specific parameters
            // param_c: Mie concentration (0..255 -> 0..2)
            let mie_concentration = u8_to_01(instr_c(instr)) * 2.0;
            // param_d: Mie exponent (0..255 -> 4..128)
            let mie_exponent = mix(4.0, 128.0, u8_to_01(instr_d(instr)));
            // direction: Sun direction
            let sun_dir = decode_dir16(instr_dir16(instr));

            sample = eval_atmosphere_mie(dir, t, intensity, color_a, sun_dir, mie_concentration, mie_exponent);
        }
        case ATMOSPHERE_VARIANT_FULL: {
            // Extract Mie-specific parameters
            let mie_concentration = u8_to_01(instr_c(instr)) * 2.0;
            let mie_exponent = mix(4.0, 128.0, u8_to_01(instr_d(instr)));
            let sun_dir = decode_dir16(instr_dir16(instr));

            sample = eval_atmosphere_full(dir, t, intensity, color_a, color_b, sun_dir, mie_concentration, mie_exponent);
        }
        case ATMOSPHERE_VARIANT_ALIEN: {
            sample = eval_atmosphere_alien(t, intensity, color_a, color_b);
        }
        default: {
            // Reserved/unknown variants: no output.
            sample = LayerSample(vec3f(0.0), 0.0);
        }
    }

    // Apply alpha and region weight
    let final_w = sample.w * alpha_a * region_w;

    return LayerSample(sample.rgb, final_w);
}
