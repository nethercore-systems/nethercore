// ============================================================================
// CELL - Voronoi Cell Enclosure Source (0x05)
// Tessellates the sphere into discrete cells for mosaic/crystalline enclosures.
// 128-bit packed fields:
//   color_a: Sky (gap) base color (RGB24)
//   color_b: Wall (solid) base color (RGB24)
//   intensity: Outline brightness (0..255 -> 0.0..1.0)
//   param_a: Cell density (0..255 -> 4..64 cells per unit)
//   param_b: Fill ratio (0..255 -> 0.0..1.0, fraction of solid cells)
//   param_c: Gap width (0..255 -> 0.0..0.2)
//   param_d: Seed for randomization (0..255)
//   direction: Alignment axis (oct-u16)
//   alpha_a: Gap alpha (0..15 -> 0.0..1.0)
//   alpha_b: Outline alpha (0..15 -> 0.0..1.0)
//   variant_id: 0=GRID, 1=HEX, 2=VORONOI, 3=RADIAL, 4=SHATTER, 5=BRICK
// ============================================================================

// Hash function for cell id (deterministic, rollback-safe)
fn cell_hash2(cell: vec2f, seed: f32) -> f32 {
    let p = cell + vec2f(seed * 17.3, seed * 31.7);
    return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.5453123);
}

// 2D hash returning vec2 for Voronoi jitter
fn cell_hash2_vec2(cell: vec2f, seed: f32) -> vec2f {
    let p = cell + vec2f(seed * 17.3, seed * 31.7);
    let h = vec2f(
        dot(p, vec2f(127.1, 311.7)),
        dot(p, vec2f(269.5, 183.3))
    );
    return fract(sin(h) * 43758.5453123);
}

// Compute axis-cylinder UV from direction and alignment axis
fn cell_axis_cylinder_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build orthonormal basis around axis
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, axis));
    let b_axis = normalize(cross(axis, t_axis));

    // Project direction
    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);

    // Azimuth u01 in [0, 1), elevation v01 in [0, 1]
    let u01 = fract(atan2(b_proj, t_proj) / TAU + 0.5);
    let v01 = dot(dir, axis) * 0.5 + 0.5;

    return vec2f(u01, v01);
}

// GRID variant: regular rectangular cells
fn cell_grid(uv: vec2f, density: f32) -> vec3f {
    let scaled = uv * density;
    let cell_id = floor(scaled);
    let cell_fract = fract(scaled);

    // Distance to nearest edge
    let d_edge = min(
        min(cell_fract.x, 1.0 - cell_fract.x),
        min(cell_fract.y, 1.0 - cell_fract.y)
    );

    return vec3f(cell_id, d_edge);
}

// HEX variant: hexagonal cells with offset rows
fn cell_hex(uv: vec2f, density: f32) -> vec3f {
    let scaled = uv * density;

    // Offset even rows by 0.5 for hex pattern
    let row = floor(scaled.y);
    var offset_x = scaled.x;
    if fract(row * 0.5) < 0.25 {
        offset_x += 0.5;
    }

    let cell_id = vec2f(floor(offset_x), row);
    let cell_fract = vec2f(fract(offset_x), fract(scaled.y));

    // Approximate hex distance (simplified)
    let center = cell_fract - vec2f(0.5, 0.5);
    let hex_dist = max(abs(center.x), abs(center.y) * 0.866 + abs(center.x) * 0.5);
    let d_edge = 0.5 - hex_dist;

    return vec3f(cell_id, d_edge);
}

// BRICK variant: offset alternating rows
fn cell_brick(uv: vec2f, density: f32) -> vec3f {
    let scaled = uv * vec2f(density, density * 0.5);

    let row = floor(scaled.y);
    var offset_x = scaled.x;
    if fract(row * 0.5) < 0.25 {
        offset_x += 0.5;
    }

    let cell_id = vec2f(floor(offset_x), row);
    let cell_fract = vec2f(fract(offset_x), fract(scaled.y));

    // Distance to edge
    let d_edge = min(
        min(cell_fract.x, 1.0 - cell_fract.x),
        min(cell_fract.y, 1.0 - cell_fract.y)
    );

    return vec3f(cell_id, d_edge);
}

// VORONOI variant: find nearest point from jittered grid
fn cell_voronoi(uv: vec2f, density: f32, seed: f32) -> vec3f {
    let scaled = uv * density;
    let base_cell = floor(scaled);

    var min_dist = 100.0;
    var min_dist2 = 100.0;
    var closest_cell = base_cell;

    // Search 3x3 neighborhood
    for (var dy = -1.0; dy <= 1.0; dy += 1.0) {
        for (var dx = -1.0; dx <= 1.0; dx += 1.0) {
            let neighbor = base_cell + vec2f(dx, dy);
            // Handle wrap in u (azimuth)
            var wrapped = neighbor;
            wrapped.x = fract(wrapped.x / density) * density;

            let jitter = cell_hash2_vec2(neighbor, seed);
            let point = neighbor + jitter * 0.8 + vec2f(0.1);
            let d = length(scaled - point);

            if d < min_dist {
                min_dist2 = min_dist;
                min_dist = d;
                closest_cell = neighbor;
            } else if d < min_dist2 {
                min_dist2 = d;
            }
        }
    }

    // Edge distance approximation (distance to second-nearest minus nearest)
    let d_edge = (min_dist2 - min_dist) * 0.5;

    return vec3f(closest_cell, d_edge);
}

// SHATTER variant: voronoi with higher jitter
fn cell_shatter(uv: vec2f, density: f32, seed: f32) -> vec3f {
    let scaled = uv * density;
    let base_cell = floor(scaled);

    var min_dist = 100.0;
    var min_dist2 = 100.0;
    var closest_cell = base_cell;

    // Search 3x3 neighborhood with more aggressive jitter
    for (var dy = -1.0; dy <= 1.0; dy += 1.0) {
        for (var dx = -1.0; dx <= 1.0; dx += 1.0) {
            let neighbor = base_cell + vec2f(dx, dy);
            let jitter = cell_hash2_vec2(neighbor, seed + 42.0);
            // Full cell jitter for shattered look
            let point = neighbor + jitter;
            let d = length(scaled - point);

            if d < min_dist {
                min_dist2 = min_dist;
                min_dist = d;
                closest_cell = neighbor;
            } else if d < min_dist2 {
                min_dist2 = d;
            }
        }
    }

    let d_edge = (min_dist2 - min_dist) * 0.5;

    return vec3f(closest_cell, d_edge);
}

// RADIAL variant: starburst pattern with rings and spokes
fn cell_radial(uv: vec2f, density: f32, axis: vec3f, dir: vec3f) -> vec3f {
    // Use angle and radius from axis
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, axis));
    let b_axis = normalize(cross(axis, t_axis));

    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);
    let z_proj = dot(dir, axis);

    let angle01 = fract(atan2(b_proj, t_proj) / TAU + 0.5);
    let radius01 = sqrt(max(0.0, 1.0 - z_proj * z_proj));

    // Ring and spoke counts based on density
    let ring_count = density * 0.5;
    let spoke_count = density;

    let ring_id = floor(radius01 * ring_count);
    let spoke_id = floor(angle01 * spoke_count);
    let cell_id = vec2f(spoke_id, ring_id);

    let ring_fract = fract(radius01 * ring_count);
    let spoke_fract = fract(angle01 * spoke_count);

    let d_edge = min(
        min(ring_fract, 1.0 - ring_fract),
        min(spoke_fract, 1.0 - spoke_fract)
    );

    return vec3f(cell_id, d_edge);
}

fn eval_cell(
    dir: vec3f,
    instr: vec4u,
    base_regions: RegionWeights,
) -> BoundsResult {
    // Decode axis from direction field
    let axis = decode_dir16(instr_dir16(instr));

    // Extract parameters
    let outline_brightness = u8_to_01(instr_intensity(instr));
    let density = mix(4.0, 64.0, u8_to_01(instr_a(instr)));
    let fill_ratio = u8_to_01(instr_b(instr));
    let gap_width = u8_to_01(instr_c(instr)) * 0.2;
    let seed = f32(instr_d(instr));
    let gap_alpha = instr_alpha_a_f32(instr);
    let outline_alpha = instr_alpha_b_f32(instr);
    let variant = instr_variant_id(instr);

    // Get UV coordinates in axis-cylinder space
    let uv = cell_axis_cylinder_uv(dir, axis);

    // Compute cell info based on variant: vec3(cell_id.xy, d_edge)
    var cell_info: vec3f;
    switch variant {
        case 0u: { cell_info = cell_grid(uv, density); }      // GRID
        case 1u: { cell_info = cell_hex(uv, density); }       // HEX
        case 2u: { cell_info = cell_voronoi(uv, density, seed); }  // VORONOI
        case 3u: { cell_info = cell_radial(uv, density, axis, dir); }  // RADIAL
        case 4u: { cell_info = cell_shatter(uv, density, seed); }  // SHATTER
        case 5u: { cell_info = cell_brick(uv, density); }     // BRICK
        default: { cell_info = cell_grid(uv, density); }
    }

    let cell_id = cell_info.xy;
    let d_edge = cell_info.z;

    // Determine if cell is solid based on hash and fill ratio
    let cell_value = cell_hash2(cell_id, seed);
    let is_solid = cell_value < fill_ratio;

    // Compute regions from cell geometry
    // CELL defines its own 3 regions:
    //   - Sky: gaps/openings between cells (non-solid cells or gap areas)
    //   - Wall: cell boundaries/outlines
    //   - Floor: cell interiors (solid cells)
    var output_regions: RegionWeights;

    // Radiance weights (can be alpha-scaled for blending)
    var w_sky: f32;
    var w_wall: f32;
    var w_floor: f32;

    if !is_solid {
        // Non-solid cell: all sky (opening)
        output_regions = RegionWeights(1.0, 0.0, 0.0);
        w_sky = 1.0;
        w_wall = 0.0;
        w_floor = 0.0;
    } else {
        // Solid cell: compute regions from edge distance
        // d_edge is distance to cell edge (positive = inside cell)
        // d: negative in gap (sky), positive in cell interior (floor)
        let d = d_edge - gap_width;
        let bw = max(0.005, gap_width * 0.5);
        output_regions = regions_from_signed_distance(d, bw);

        // Radiance weights with alpha scaling for sky (gap)
        w_sky = output_regions.sky * gap_alpha;
        w_wall = output_regions.wall;
        w_floor = output_regions.floor;
    }

    // Add outline effect at gap boundary (only for solid cells)
    let outline_dist = abs(d_edge - gap_width);
    let outline_width = gap_width * 0.3;
    let outline = smoothstep(outline_width, 0.0, outline_dist) * outline_alpha * outline_brightness * select(0.0, 1.0, is_solid);

    // Get colors
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);
    let floor_color = wall_color * 0.5;

    // Blend colors based on weights
    let base_rgb = sky_color * w_sky + wall_color * w_wall + floor_color * w_floor;

    // Outline tinted by the brighter of the two cell colors (not hardcoded white)
    let outline_color = max(sky_color, wall_color);
    let rgb = base_rgb + outline_color * outline;

    // Total weight
    let w = w_sky + w_wall + w_floor;

    // CELL is an enclosure source: return radiance sample + output regions.
    return BoundsResult(LayerSample(rgb, epu_saturate(w)), output_regions);
}
