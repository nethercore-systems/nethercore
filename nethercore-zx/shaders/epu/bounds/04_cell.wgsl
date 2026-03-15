// @epu_meta_begin
// opcode = 0x05
// name = CELL
// kind = bounds
// variants = [GRID, HEX, VORONOI, RADIAL, SHATTER, BRICK]
// domains = []
// field intensity = { label="outline", map="u8_01" }
// field param_a = { label="density", map="u8_lerp", min=4.0, max=64.0 }
// field param_b = { label="fill", map="u8_01" }
// field param_c = { label="gap_width", map="u8_lerp", min=0.0, max=0.2 }
// field param_d = { label="seed", map="u8_01" }
// @epu_meta_end

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

fn wrap_periodic_x(x: f32, period: f32) -> f32 {
    return fract(x / period) * period;
}

fn shortest_periodic_delta(x: f32, center: f32, period: f32) -> f32 {
    return x - center - round((x - center) / period) * period;
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
fn cell_brick(uv: vec2f, density: f32, seed: f32) -> vec3f {
    let scaled = uv * vec2f(density, density * 0.5);

    let row = floor(scaled.y);
    let row_seed = cell_hash2(vec2f(row, density), seed + 11.0);
    var offset_x = scaled.x;
    if fract(row * 0.5) < 0.25 {
        offset_x += 0.5;
    }
    // Break perfect repeated mortar columns. A tiny row-specific shift keeps the
    // brick read, but stops the brightest seams from marching as rigid guide bars.
    offset_x += (row_seed - 0.5) * 0.16;

    let cell_id = vec2f(floor(offset_x), row);
    let brick_seed = cell_hash2(cell_id, seed + 29.0);
    let row_phase = fract(scaled.y);
    let warped_x = fract(offset_x + (row_phase - 0.5) * mix(-0.08, 0.08, brick_seed));
    let warped_y = fract(scaled.y + (fract(offset_x) - 0.5) * mix(-0.035, 0.035, row_seed));
    let cell_fract = vec2f(warped_x, warped_y);

    let vertical_wave = epu_relief_wave(
        vec2f(cell_id.x * 0.23 + cell_fract.y * 0.91, cell_id.y * 0.37 + cell_fract.x * 0.19),
        seed * 0.013 + brick_seed * 0.71 + 0.11
    );
    let horizontal_wave = epu_relief_wave(
        vec2f(cell_id.y * 0.29 + cell_fract.x * 0.83, cell_id.x * -0.17 + cell_fract.y * 0.27),
        seed * 0.017 + row_seed * 0.63 + 0.43
    );
    let vertical_gate = smoothstep(-0.2, 0.72, vertical_wave);
    let horizontal_gate = smoothstep(-0.24, 0.68, horizontal_wave);

    // Break mortar continuity before compositing. The line family still reads as
    // brick, but low-frequency gating closes sections of the vertical/horizontal
    // seams so they cannot survive as full-chart tracery or seam rails.
    let x_edge = min(cell_fract.x, 1.0 - cell_fract.x);
    let y_edge = min(cell_fract.y, 1.0 - cell_fract.y);
    let joint_taper = smoothstep(0.0, 0.22, y_edge);
    let vertical_lane = smoothstep(0.04, 0.26, y_edge);
    let horizontal_lane = smoothstep(0.04, 0.22, x_edge);
    let x_mortar = x_edge
        * mix(0.8, 1.0, joint_taper)
        * (1.0 + (1.0 - vertical_gate) * vertical_lane * 0.9);
    let y_mortar = y_edge * (1.0 + (1.0 - horizontal_gate) * horizontal_lane * 0.75);
    let d_edge = min(x_mortar, y_mortar);

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
            let wrapped = vec2f(wrap_periodic_x(neighbor.x, density), neighbor.y);
            let jitter = cell_hash2_vec2(neighbor, seed);
            let point = wrapped + jitter * 0.8 + vec2f(0.1);
            let delta = vec2f(
                shortest_periodic_delta(scaled.x, point.x, density),
                scaled.y - point.y
            );
            let d = length(delta);

            if d < min_dist {
                min_dist2 = min_dist;
                min_dist = d;
                closest_cell = wrapped;
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
            let wrapped = vec2f(wrap_periodic_x(neighbor.x, density), neighbor.y);
            let jitter = cell_hash2_vec2(neighbor, seed + 42.0);
            // Full cell jitter for shattered look
            let point = wrapped + jitter;
            let delta = vec2f(
                shortest_periodic_delta(scaled.x, point.x, density),
                scaled.y - point.y
            );
            let d = length(delta);

            if d < min_dist {
                min_dist2 = min_dist;
                min_dist = d;
                closest_cell = wrapped;
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
    let radial_uv = epu_wrapped_relief_uv(
        vec2f(angle01, radius01),
        density * 0.019 + z_proj * 0.41,
        0.038 * smoothstep(0.1, 0.42, radius01),
        0.032
    );
    let angle_phase = radial_uv.x;
    let radius_phase = clamp(radial_uv.y, 0.0, 0.9999);

    // Ring and spoke counts based on density
    let ring_count = max(density * 0.5, 1.0);
    let spoke_count = max(density, 1.0);
    let base_ring_seed = floor(radius_phase * ring_count);
    let base_spoke_seed = floor(angle_phase * spoke_count);
    let center_relief = smoothstep(0.08, 0.34, radius_phase);
    let ring_relief = epu_relief_envelope(radius_phase, 0.06, 0.22, 0.82, 0.99);
    let angular_relief = epu_relief_wave(vec2f(t_proj, b_proj) * vec2f(2.4, 2.1), density * 0.017);
    let radial_relief = epu_relief_wave(
        vec2f(angle_phase * spoke_count, radius_phase * ring_count),
        density * 0.043 + z_proj * 0.5
    );
    let ring_warp = (cell_hash2(vec2f(base_spoke_seed, base_ring_seed), density * 0.37) - 0.5)
        * mix(0.0, 0.16 / ring_count, ring_relief);
    let spoke_warp = (cell_hash2(vec2f(base_ring_seed, base_spoke_seed), density * 0.73) - 0.5)
        * (0.14 / spoke_count)
        * center_relief;
    let ring_count_local = ring_count * mix(1.0, mix(0.9, 1.18, angular_relief * 0.5 + 0.5), ring_relief);
    let spoke_count_local = spoke_count * mix(1.0, mix(0.92, 1.12, radial_relief * 0.5 + 0.5), center_relief);
    let ring_phase = clamp(
        radius_phase + ring_warp + angular_relief * (0.11 / ring_count) * ring_relief,
        0.0,
        0.9999
    ) * ring_count_local;
    let spoke_phase = fract(angle_phase + spoke_warp + radial_relief * 0.085 * center_relief) * spoke_count_local;
    let ring_id = floor(ring_phase);
    let spoke_id = floor(spoke_phase);
    let cell_id = vec2f(spoke_id, ring_id);
    let edge_relief = epu_relief_wave(vec2f(radius_phase * ring_count, angle_phase * spoke_count), density * 0.061 + 0.23);
    let ring_edge = mix(
        0.5,
        epu_periodic_edge_distance(ring_phase + edge_relief * 0.08 * ring_relief),
        ring_relief
    );
    let spoke_edge = mix(
        0.5,
        epu_periodic_edge_distance(spoke_phase + edge_relief * 0.06 * center_relief),
        center_relief
    );
    let d_edge = min(ring_edge, spoke_edge);

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
    var brick_stress = 0.0;
    switch variant {
        case 0u: { cell_info = cell_grid(uv, density); }      // GRID
        case 1u: { cell_info = cell_hex(uv, density); }       // HEX
        case 2u: { cell_info = cell_voronoi(uv, density, seed); }  // VORONOI
        case 3u: { cell_info = cell_radial(uv, density, axis, dir); }  // RADIAL
        case 4u: { cell_info = cell_shatter(uv, density, seed); }  // SHATTER
        case 5u: { // BRICK
            cell_info = cell_brick(uv, density, seed);
            let seam_band = 1.0 - smoothstep(0.03, 0.11, epu_periodic_edge_distance(uv.x));
            let pole_band = 1.0 - smoothstep(0.05, 0.18, min(uv.y, 1.0 - uv.y));
            let horizon_band = 1.0 - smoothstep(0.045, 0.16, abs(uv.y - 0.5));
            brick_stress = max(seam_band, max(pole_band, horizon_band));
        }
        default: { cell_info = cell_grid(uv, density); }
    }

    let cell_id = cell_info.xy;
    let d_edge = cell_info.z;

    // Determine if cell is solid based on hash and fill ratio
    let cell_value = cell_hash2(cell_id, seed);
    let is_solid = cell_value < fill_ratio;
    var solid_w = select(0.0, 1.0, is_solid);

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
    var outline_dist = abs(d_edge - gap_width);

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
    let outline_width = gap_width * 0.3;
    var outline_taper = 1.0;
    switch variant {
        case 3u: { // RADIAL
            let radial_center = epu_relief_envelope(uv.y, 0.08, 0.26, 0.78, 0.98);
            let radial_gate = smoothstep(
                -0.25,
                0.7,
                epu_relief_wave(vec2f(cell_id.x * 0.37, uv.y * density * 0.91 + cell_id.y * 0.11), seed * 0.017)
            );
            outline_taper = mix(1.0, mix(0.64, 1.0, radial_gate), radial_center);
        }
        case 5u: { // BRICK
            let brick_gate = smoothstep(
                -0.22,
                0.7,
                epu_relief_wave(vec2f(cell_id.y * 0.41 + fract(uv.x * density), uv.y * density * 1.27), seed * 0.013)
            );
            outline_taper = mix(0.74, 1.0, brick_gate);
        }
        default: {}
    }
    var chart_outline_gate = 1.0;
    switch variant {
        case 5u: { // BRICK
            let seam_gate = smoothstep(0.025, 0.12, epu_periodic_edge_distance(uv.x));
            let pole_gate = smoothstep(0.04, 0.18, min(uv.y, 1.0 - uv.y));
            let horizon_gate = smoothstep(0.04, 0.16, abs(uv.y - 0.5));
            chart_outline_gate = seam_gate * mix(0.32, 1.0, pole_gate) * mix(0.25, 1.0, horizon_gate);
        }
        default: {}
    }
    let outline = smoothstep(outline_width, 0.0, outline_dist)
        * outline_alpha
        * outline_brightness
        * outline_taper
        * chart_outline_gate
        * solid_w;

    // Get colors
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);
    var floor_color = wall_color * 0.5;
    if variant == 5u {
        // In stressed cylindrical zones, keep BRICK region structure but
        // compress wall/floor contrast so panel and ring reads soften.
        floor_color = mix(floor_color, wall_color, brick_stress * 0.8);
    }

    // Blend colors based on weights
    let base_rgb = sky_color * w_sky + wall_color * w_wall + floor_color * w_floor;

    // Outline tinted by the brighter of the two cell colors (not hardcoded white)
    var outline_color = max(sky_color, wall_color);
    if variant == 5u {
        outline_color = mix(outline_color, mix(wall_color, base_rgb, 0.7), brick_stress * 0.85);
    }
    let rgb = base_rgb + outline_color * outline;

    // Total weight
    let w = w_sky + w_wall + w_floor;

    // CELL is an enclosure source: return radiance sample + output regions.
    return BoundsResult(LayerSample(rgb, epu_saturate(w)), output_regions, 1.0);
}
