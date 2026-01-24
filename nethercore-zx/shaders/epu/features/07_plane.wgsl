// ============================================================================
// PLANE - Ground Plane Textures
// Opcode: 0x0F
// Role: Radiance (additive feature layer)
//
// Packed fields:
//   color_a: Primary surface color (RGB24)
//   color_b: Secondary/grout/gap color (RGB24)
//   intensity: Pattern contrast/brightness (0..255 -> 0..1)
//   param_a: Pattern scale (0..255 -> 0.5..16.0)
//   param_b: Gap width (0..255 -> 0..0.2)
//   param_c: Roughness (0..255 -> 0..1)
//   param_d: Reserved (set to 0)
//   direction: Plane normal (oct-u16)
//   alpha_a: Pattern alpha (0..15 -> 0..1)
//   alpha_b: Unused (set to 0)
//
// Meta (via meta5):
//   domain_id: Ignored (always planar projection)
//   variant_id: 0 TILES, 1 HEX, 2 STONE, 3 SAND, 4 WATER, 5 GRATING, 6 GRASS, 7 PAVEMENT
// ============================================================================

// Variant IDs for PLANE
const PLANE_VARIANT_TILES: u32 = 0u;      // Regular grid with grout
const PLANE_VARIANT_HEX: u32 = 1u;        // Hexagonal cells with edges
const PLANE_VARIANT_STONE: u32 = 2u;      // Irregular stone with noise displacement
const PLANE_VARIANT_SAND: u32 = 3u;       // Low-freq noise + grain, optional drift
const PLANE_VARIANT_WATER: u32 = 4u;      // Layered sinusoidal ripples
const PLANE_VARIANT_GRATING: u32 = 5u;    // Parallel bars with gaps
const PLANE_VARIANT_GRASS: u32 = 6u;      // Noise-based with directional streaks
const PLANE_VARIANT_PAVEMENT: u32 = 7u;   // Irregular Voronoi tiles with grout

// Deterministic hash for plane patterns (2D -> 2D)
fn plane_hash22(p: vec2f) -> vec2f {
    var p3 = fract(vec3f(p.xyx) * vec3f(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

// Deterministic hash for plane patterns (2D -> 1D)
fn plane_hash21(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.xyx) * 0.1031);
    let d = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z + d);
}

// Deterministic hash for plane patterns (2D -> 3D)
fn plane_hash23(p: vec2f) -> vec3f {
    var p3 = fract(vec3f(p.xyx) * vec3f(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yzz) * p3.zyx);
}

// 2D value noise for STONE/SAND/GRASS patterns
fn plane_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = plane_hash21(i);
    let b = plane_hash21(i + vec2f(1.0, 0.0));
    let c = plane_hash21(i + vec2f(0.0, 1.0));
    let d = plane_hash21(i + vec2f(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// FBM noise (3 octaves, band-limited for mip stability)
fn plane_fbm(p: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pp = p;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * plane_noise(pp * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
        pp = pp * 2.0 + vec2f(17.3, 31.7);
    }

    return value;
}

// Voronoi cell distance for PAVEMENT variant
fn plane_voronoi(p: vec2f) -> vec3f {
    // Returns (cell_distance, edge_distance, cell_id_hash)
    let cell = floor(p);
    let f = fract(p);

    var min_dist = 8.0;
    var second_dist = 8.0;
    var cell_id = 0.0;

    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let neighbor = vec2f(f32(i), f32(j));
            let offset = plane_hash22(cell + neighbor);
            let point = neighbor + offset - f;
            let d = dot(point, point);

            if d < min_dist {
                second_dist = min_dist;
                min_dist = d;
                cell_id = plane_hash21(cell + neighbor);
            } else if d < second_dist {
                second_dist = d;
            }
        }
    }

    let edge_dist = second_dist - min_dist;
    return vec3f(sqrt(min_dist), edge_dist, cell_id);
}

// TILES variant: regular grid with grout
fn eval_plane_tiles(
    uv: vec2f,
    gap: f32
) -> vec2f {
    // Returns (pattern_mask, grout_mask) where 1.0 = tile surface, 0.0 = grout
    let cell = floor(uv);
    let f = fract(uv);

    // Distance to nearest edge
    let edge_dist = min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y));

    // AA width based on UV derivatives approximation
    let aa_width = 0.01;
    let grout_mask = smoothstep(gap - aa_width, gap + aa_width, edge_dist);

    // Per-tile color variation
    let tile_hash = plane_hash21(cell);

    return vec2f(grout_mask, tile_hash);
}

// HEX variant: hexagonal cells with edges
fn eval_plane_hex(
    uv: vec2f,
    gap: f32
) -> vec2f {
    // Returns (pattern_mask, cell_hash)
    // Hex grid constants
    let hex_scale = vec2f(1.0, 1.7320508); // 1, sqrt(3)
    let hex_half = hex_scale * 0.5;

    // Two offset grids
    let a = (uv / hex_scale) - floor(uv / hex_scale + 0.5);
    let b = ((uv + hex_half) / hex_scale) - floor((uv + hex_half) / hex_scale + 0.5);

    // Pick closer cell
    let use_b = length(a) > length(b);
    let hex_uv = select(a, b, use_b);
    let hex_cell = select(floor(uv / hex_scale + 0.5), floor((uv + hex_half) / hex_scale + 0.5), use_b);

    // Hexagonal distance (approximate)
    let hex_dist = max(abs(hex_uv.x), abs(hex_uv.y) * 0.5 + abs(hex_uv.x) * 0.5);

    // Edge mask
    let edge_threshold = 0.5 - gap;
    let aa_width = 0.01;
    let edge_mask = smoothstep(edge_threshold + aa_width, edge_threshold - aa_width, hex_dist);

    // Cell hash for color variation
    let cell_hash = plane_hash21(hex_cell);

    return vec2f(1.0 - edge_mask, cell_hash);
}

// STONE variant: irregular stone with noise displacement
fn eval_plane_stone(
    uv: vec2f,
    gap: f32,
    roughness: f32
) -> vec2f {
    // Returns (pattern_mask, color_variation)
    // Irregular cell grid
    let cell = floor(uv);
    let f = fract(uv);

    // Noise-based edge displacement
    let noise_offset = plane_fbm(uv * 4.0, 2u) * roughness * 0.3;

    // Perturbed edge distance
    let edge_x = min(f.x, 1.0 - f.x) + (plane_noise(uv * 8.0 + vec2f(7.3, 0.0)) - 0.5) * roughness * 0.2;
    let edge_y = min(f.y, 1.0 - f.y) + (plane_noise(uv * 8.0 + vec2f(0.0, 11.7)) - 0.5) * roughness * 0.2;
    let edge_dist = min(edge_x, edge_y) + noise_offset;

    // Grout mask
    let aa_width = 0.015;
    let grout_mask = smoothstep(gap - aa_width, gap + aa_width * 2.0, edge_dist);

    // Surface color variation (per-stone + noise)
    let stone_hash = plane_hash21(cell);
    let surface_noise = plane_fbm(uv * 6.0, 3u) * 0.3;
    let color_var = stone_hash * 0.5 + surface_noise;

    return vec2f(grout_mask, color_var);
}

// SAND variant: low-freq noise + grain, optional drift
fn eval_plane_sand(
    uv: vec2f,
    roughness: f32
) -> vec2f {
    // Returns (brightness_variation, grain_detail)
    let drifted_uv = uv;

    // Low-frequency dunes
    let dunes = plane_fbm(drifted_uv * 0.5, 2u);

    // Mid-frequency ripples
    let ripples = plane_fbm(drifted_uv * 2.0 + vec2f(17.3, 23.7), 2u) * 0.5;

    // High-frequency grain
    let grain = plane_noise(uv * 16.0) * roughness * 0.3;

    // Combined brightness variation
    let brightness = dunes * 0.4 + ripples * 0.3 + 0.3;

    return vec2f(brightness, grain);
}

// WATER variant: layered sinusoidal ripples
fn eval_plane_water(
    uv: vec2f,
) -> vec2f {
    // Returns (ripple_brightness, specular_highlight)
    let t = 0.0;

    // Multiple overlapping ripple layers
    let center1 = vec2f(0.3, 0.4);
    let center2 = vec2f(0.7, 0.6);
    let center3 = vec2f(0.5, 0.2);

    let d1 = length(fract(uv) - center1);
    let d2 = length(fract(uv * 0.7 + vec2f(0.2, 0.3)) - center2);
    let d3 = length(fract(uv * 1.3 + vec2f(0.5, 0.1)) - center3);

    // Sinusoidal ripples
    let ripple1 = sin(d1 * 20.0 - t * 3.0) * 0.5 + 0.5;
    let ripple2 = sin(d2 * 15.0 - t * 2.5 + 1.0) * 0.5 + 0.5;
    let ripple3 = sin(d3 * 25.0 - t * 3.5 + 2.0) * 0.5 + 0.5;

    // Combine ripples
    let combined = (ripple1 + ripple2 + ripple3) / 3.0;

    // Subtle caustic-like pattern
    let caustic = plane_noise(uv * 8.0 + vec2f(0.0, 0.0)) * 0.2;

    // Specular highlight approximation
    let specular = pow(combined, 4.0) * 0.5;

    return vec2f(combined * 0.6 + 0.4 + caustic, specular);
}

// GRATING variant: parallel bars with gaps
fn eval_plane_grating(
    uv: vec2f,
    gap: f32
) -> vec2f {
    // Returns (bar_mask, bar_index_hash)
    // Compute bar position
    let bar_width = 1.0 - gap * 5.0; // Invert gap to bar ratio
    let bar_period = 1.0;

    let u_fract = fract(uv.x);

    // Distance to bar center
    let bar_mask = smoothstep(0.5 - bar_width * 0.5 - 0.01, 0.5 - bar_width * 0.5 + 0.01, abs(u_fract - 0.5));
    let final_mask = 1.0 - bar_mask;

    // Bar index for variation
    let bar_index = floor(uv.x);
    let bar_hash = plane_hash21(vec2f(bar_index, 0.0));

    return vec2f(final_mask, bar_hash);
}

// GRASS variant: noise-based with directional streaks
fn eval_plane_grass(
    uv: vec2f,
    roughness: f32
) -> vec2f {
    // Returns (grass_density, color_variation)
    let swayed_uv = uv;

    // Base grass density (large patches)
    let density_base = plane_fbm(swayed_uv * 0.8, 2u);

    // Directional streaks (blade-like patterns)
    let streak_uv = vec2f(swayed_uv.x * 4.0, swayed_uv.y * 0.5);
    let streaks = plane_noise(streak_uv) * roughness;

    // Fine grass detail
    let detail = plane_noise(uv * 20.0) * 0.15;

    // Combined density
    let density = density_base * 0.5 + streaks * 0.3 + detail + 0.2;

    // Color variation (some blades lighter/darker)
    let color_var = plane_fbm(uv * 3.0, 2u);

    return vec2f(density, color_var);
}

// PAVEMENT variant: irregular Voronoi tiles with grout
fn eval_plane_pavement(
    uv: vec2f,
    gap: f32,
    roughness: f32
) -> vec2f {
    // Returns (grout_mask, tile_color_var)
    // Voronoi-based irregular tiles
    let vor = plane_voronoi(uv);
    let edge_dist = vor.y; // Distance between closest and second closest

    // Grout in the cracks between cells
    let grout_width = gap * 2.0;
    let aa_width = 0.02;
    let grout_mask = smoothstep(grout_width - aa_width, grout_width + aa_width, edge_dist);

    // Per-tile color variation with surface wear
    let tile_hash = vor.z;
    let wear = plane_fbm(uv * 4.0, 2u) * roughness * 0.3;
    let color_var = tile_hash * 0.5 + wear;

    return vec2f(grout_mask, color_var);
}

fn eval_plane(
    dir: vec3f,
    instr: vec4u,
    region_w: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract variant (domain_id is ignored for PLANE)
    let variant_id = instr_variant_id(instr);

    // Extract parameters
    // param_a: Pattern scale (0..255 -> 0.5..16.0)
    let scale = mix(0.5, 16.0, u8_to_01(instr_a(instr)));
    // param_b: Gap width (0..255 -> 0..0.2)
    let gap = u8_to_01(instr_b(instr)) * 0.2;
    // param_c: Roughness (0..255 -> 0..1)
    let roughness = u8_to_01(instr_c(instr));

    // Decode plane normal
    let plane_normal = decode_dir16(instr_dir16(instr));

    // Build orthonormal basis for plane
    // Choose a hint vector that is not parallel to the normal
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(plane_normal.y) > 0.9);
    let plane_right = normalize(cross(hint, plane_normal));
    let plane_forward = cross(plane_normal, plane_right);

    // Compute plane intersection
    // d = dot(dir, n) is the alignment with plane normal
    // Positive d means looking toward the plane
    let d = dot(dir, plane_normal);

    // Reject if behind plane or near-parallel (d <= 0.05)
    if d <= 0.05 {
        return LayerSample(vec3f(0.0), 0.0);
    }

    // Compute hit distance: t = 1.0 / d
    // This is a simplified projection assuming plane at origin
    let t = 1.0 / d;

    // Compute hit point on plane
    let hit = dir * t;

    // Project hit point to 2D UV coordinates
    let u_coord = dot(hit, plane_right) * scale;
    let v_coord = dot(hit, plane_forward) * scale;
    let uv = vec2f(u_coord, v_coord);

    // Compute grazing fade to prevent aliasing at shallow angles
    let grazing_w = smoothstep(0.05, 0.2, d);

    // Evaluate pattern by variant
    var pattern_mask = 1.0;
    var color_variation = 0.0;
    var specular_term = 0.0;

    switch variant_id {
        case PLANE_VARIANT_TILES: {
            let result = eval_plane_tiles(uv, gap);
            pattern_mask = result.x;
            color_variation = result.y * 0.2;
        }
        case PLANE_VARIANT_HEX: {
            let result = eval_plane_hex(uv, gap);
            pattern_mask = result.x;
            color_variation = result.y * 0.2;
        }
        case PLANE_VARIANT_STONE: {
            let result = eval_plane_stone(uv, gap, roughness);
            pattern_mask = result.x;
            color_variation = result.y;
        }
        case PLANE_VARIANT_SAND: {
            let result = eval_plane_sand(uv, roughness);
            pattern_mask = 1.0;
            color_variation = result.x - 0.5; // Center around 0
            specular_term = result.y * 0.1;
        }
        case PLANE_VARIANT_WATER: {
            let result = eval_plane_water(uv);
            pattern_mask = 1.0;
            color_variation = result.x - 0.5;
            specular_term = result.y;
        }
        case PLANE_VARIANT_GRATING: {
            let result = eval_plane_grating(uv, gap);
            pattern_mask = result.x;
            color_variation = result.y * 0.1;
        }
        case PLANE_VARIANT_GRASS: {
            let result = eval_plane_grass(uv, roughness);
            pattern_mask = 1.0;
            color_variation = (result.x - 0.5) * 0.5 + (result.y - 0.5) * 0.3;
        }
        case PLANE_VARIANT_PAVEMENT: {
            let result = eval_plane_pavement(uv, gap, roughness);
            pattern_mask = result.x;
            color_variation = result.y;
        }
        default: {
            // Fallback to TILES
            let result = eval_plane_tiles(uv, gap);
            pattern_mask = result.x;
            color_variation = result.y * 0.2;
        }
    }

    // Extract colors
    let color_a = instr_color_a(instr);  // Primary surface color
    let color_b = instr_color_b(instr);  // Secondary/grout color
    let intensity = u8_to_01(instr_intensity(instr));
    let alpha_a = instr_alpha_a_f32(instr);

    // Blend colors based on pattern
    // pattern_mask: 1.0 = primary surface, 0.0 = grout/gap
    // color_variation: shifts the primary color slightly
    let varied_color_a = color_a * (1.0 + color_variation * 0.5);
    let surface_color = mix(color_b, varied_color_a, pattern_mask);

    // Add specular highlight for water/reflective surfaces
    let final_color = surface_color + vec3f(specular_term);

    // Compute final weight
    let w = intensity * grazing_w * alpha_a * region_w;

    return LayerSample(final_color, w);
}
