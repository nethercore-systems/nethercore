// ============================================================================
// SILHOUETTE - Skyline/Horizon Cutout (0x03)
// Creates environmental silhouettes that reshape the sky/wall boundary.
// 128-bit packed fields:
//   color_a: Silhouette color (RGB24)
//   color_b: Background/sky color (RGB24)
//   intensity: Edge softness (0..255 -> 0.005..0.1)
//   param_a: Horizon height bias (0..255 -> -0.3..0.5)
//   param_b: Roughness/amplitude (0..255 -> 0.1..1.0)
//   param_c[7:4]: Layer count / octaves (0..15 -> 1..8)
//   param_c[3:0]: Reserved (set to 0)
//   param_d: Reserved (set to 0)
//   direction: Up axis (oct-u16)
//   alpha_a: Strength (0..15 -> 0.0..1.0)
//   variant_id: 0=MOUNTAINS, 1=CITY, 2=FOREST, 3=DUNES, 4=WAVES, 5=RUINS, 6=INDUSTRIAL, 7=SPIRES (from meta5)
// ============================================================================

// Periodic hash for seamless azimuthal wrap
fn silhouette_hash(x: f32, seed: f32) -> f32 {
    return fract(sin(x * 127.1 + seed * 311.7) * 43758.5453123);
}

// 1D periodic value noise for seamless height functions
fn silhouette_noise(u: f32, freq: f32, seed: f32) -> f32 {
    let p = u * freq;
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f); // smoothstep interpolation
    let a = silhouette_hash(i, seed);
    let b = silhouette_hash(i + 1.0, seed);
    return mix(a, b, s) * 2.0 - 1.0;
}

// FBM for MOUNTAINS variant
fn silhouette_fbm(u: f32, octaves: u32, seed: f32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 4.0;
    for (var i = 0u; i < octaves; i++) {
        value += amplitude * silhouette_noise(u, frequency, seed + f32(i) * 17.3);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value;
}

// CITY variant: rectangular blocks with varying heights
fn silhouette_city(u: f32, seed: f32) -> f32 {
    let block_freq = 16.0;
    let block_id = floor(u * block_freq);
    let block_fract = fract(u * block_freq);
    let h = silhouette_hash(block_id, seed);
    // Sharper edges for buildings
    let edge = smoothstep(0.0, 0.1, block_fract) * smoothstep(1.0, 0.9, block_fract);
    return h * edge;
}

// FOREST variant: triangular/conical tree shapes
fn silhouette_forest(u: f32, seed: f32) -> f32 {
    let tree_freq = 12.0;
    let tree_id = floor(u * tree_freq);
    let tree_fract = fract(u * tree_freq);
    let h = silhouette_hash(tree_id, seed);
    // Triangular shape: peak at center
    let tree = 1.0 - 2.0 * abs(tree_fract - 0.5);
    return h * tree;
}

// DUNES variant: smooth sinusoidal waves
fn silhouette_dunes(u: f32, seed: f32) -> f32 {
    let phase = silhouette_hash(0.0, seed) * TAU;
    return 0.5 * sin(u * TAU * 3.0 + phase) + 0.3 * sin(u * TAU * 7.0 + phase * 1.7);
}

// WAVES variant: ocean waves (periodic)
fn silhouette_waves(u: f32, seed: f32) -> f32 {
    let wave1 = sin(u * TAU * 4.0 + seed);
    let wave2 = sin(u * TAU * 9.0 + seed * 2.3) * 0.5;
    return (wave1 + wave2) * 0.4;
}

// RUINS variant: broken blocks with gaps
fn silhouette_ruins(u: f32, seed: f32) -> f32 {
    let block_freq = 10.0;
    let block_id = floor(u * block_freq);
    let block_fract = fract(u * block_freq);
    let h = silhouette_hash(block_id, seed);
    let gap = step(0.3, silhouette_hash(block_id + 100.0, seed));
    let edge = smoothstep(0.0, 0.15, block_fract) * smoothstep(1.0, 0.85, block_fract);
    return h * edge * gap;
}

// INDUSTRIAL variant: tall stacks and horizontal elements
fn silhouette_industrial(u: f32, seed: f32) -> f32 {
    let stack_freq = 8.0;
    let stack_id = floor(u * stack_freq);
    let stack_fract = fract(u * stack_freq);
    let h = silhouette_hash(stack_id, seed);
    // Thin vertical stacks
    let stack_width = 0.2 + 0.1 * silhouette_hash(stack_id + 50.0, seed);
    let stack = smoothstep(0.5 - stack_width, 0.5 - stack_width + 0.05, stack_fract)
              * smoothstep(0.5 + stack_width, 0.5 + stack_width - 0.05, stack_fract);
    return h * stack;
}

// SPIRES variant: gaussian peaks with taper
fn silhouette_spires(u: f32, seed: f32) -> f32 {
    var value = 0.0;
    let spire_count = 6.0;
    for (var i = 0.0; i < spire_count; i += 1.0) {
        let pos = silhouette_hash(i, seed);
        let h = 0.5 + 0.5 * silhouette_hash(i + 10.0, seed);
        let width = 0.02 + 0.03 * silhouette_hash(i + 20.0, seed);
        // Wrap-aware distance
        var dist = abs(u - pos);
        dist = min(dist, 1.0 - dist);
        let spike = exp(-dist * dist / (width * width)) * h;
        value = max(value, spike);
    }
    return value;
}

// Compute height function based on variant
fn silhouette_height(u: f32, variant: u32, octaves: u32, seed: f32) -> f32 {
    switch variant {
        case 0u: { return silhouette_fbm(u, octaves, seed); }         // MOUNTAINS
        case 1u: { return silhouette_city(u, seed); }                  // CITY
        case 2u: { return silhouette_forest(u, seed); }                // FOREST
        case 3u: { return silhouette_dunes(u, seed); }                 // DUNES
        case 4u: { return silhouette_waves(u, seed); }                 // WAVES
        case 5u: { return silhouette_ruins(u, seed); }                 // RUINS
        case 6u: { return silhouette_industrial(u, seed); }            // INDUSTRIAL
        case 7u: { return silhouette_spires(u, seed); }                // SPIRES
        default: { return silhouette_fbm(u, octaves, seed); }
    }
}

fn eval_silhouette(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
    base_regions: RegionWeights,
) -> BoundsResult {
    // Decode up axis from direction field
    let up = decode_dir16(instr_dir16(instr));

    // Build axis-cylinder basis around up vector
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(up.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, up));
    let b_axis = normalize(cross(up, t_axis));

    // Project direction onto the horizontal plane and compute azimuth
    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);

    // Azimuth u01: 0..1 wrapping
    let u01 = fract(atan2(b_proj, t_proj) / TAU + 0.5);

    // Elevation v01: dot with up, mapped from [-1,1] to [0,1]
    let y = dot(dir, up);
    let v01 = y * 0.5 + 0.5;

    // Extract parameters
    let softness = mix(0.005, 0.1, u8_to_01(instr_intensity(instr)));
    let height_bias = mix(-0.3, 0.5, u8_to_01(instr_a(instr)));
    let roughness = mix(0.1, 1.0, u8_to_01(instr_b(instr)));

    let pc = instr_c(instr);
    let octaves = 1u + ((pc >> 4u) & 0xFu) / 2u;  // 1..8 octaves
    let strength = instr_alpha_a_f32(instr);
    let variant = instr_variant_id(instr);

    // Use a deterministic seed based on variant
    let seed = f32(variant) * 13.7 + 42.0;

    let u_shifted = u01;

    // Compute height function
    let raw_height = silhouette_height(u_shifted, variant, octaves, seed);

    // Scale by roughness and apply height bias
    let h = height_bias + raw_height * roughness * 0.5;

    // Convert v01 to y-threshold space
    let y_equiv = v01 * 2.0 - 1.0;

    // Silhouette mask: 1.0 where below silhouette line (sky becomes wall)
    let wall_from_sky = smoothstep(h + softness, h - softness, y_equiv);

    // Apply strength
    let effect = wall_from_sky * strength;

    // Modify regions: silhouette converts sky->wall below the horizon line
    let sky_to_wall = effect * base_regions.sky;
    let modified_regions = RegionWeights(
        base_regions.sky - sky_to_wall,
        base_regions.wall + sky_to_wall,
        base_regions.floor
    );

    // Get colors and render
    let silhouette_color = instr_color_a(instr);
    let background_color = instr_color_b(instr);
    let rgb = mix(background_color, silhouette_color, effect);

    return BoundsResult(LayerSample(rgb, 1.0), modified_regions);
}
