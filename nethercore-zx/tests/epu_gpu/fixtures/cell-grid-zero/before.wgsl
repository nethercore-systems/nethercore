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
// CELL - Tiled Cell Enclosure Source (0x05)
// Tessellates the sphere into discrete cells for mosaic/crystalline enclosures.
// 128-bit packed fields:
//   color_a: Sky (gap) base color (RGB24)
//   color_b: Wall (solid) base color (RGB24)
//   intensity: Outline brightness (0..255 -> 0.0..1.0)
//   param_a: Cell density (0..255 -> 4..64); HEX/BRICK/VORONOI/SHATTER use whole columns
//   param_b: Fill ratio (0..255 -> 0.0..1.0, fraction of solid cells)
//   param_c: Gap width (0..255 -> 0.0..0.2)
//   param_d: Seed for randomization (0..255)
//   direction: Alignment axis (oct-u16)
//   alpha_a: Opacity of all gaps and unfilled cells (0..15 -> 0.0..1.0)
//   alpha_b: Outline alpha (0..15 -> 0.0..1.0)
//   variant_id: 0=GRID, 1=HEX, 2=VORONOI, 3=RADIAL, 4=SHATTER, 5=BRICK
// ============================================================================

// Hash all input bits, preserving fractional seeds/density and canonicalizing zero.
// Integer mixing avoids constant/dynamic sine approximations changing occupancy.
fn cell_hash_bits(cell: vec2f, seed: f32, salt: u32) -> u32 {
    let c = bitcast<vec2u>(select(cell, vec2f(0.0), cell == vec2f(0.0)));
    let s = bitcast<u32>(select(seed, 0.0, seed == 0.0));
    var h = c.x * 0x9e3779b9u + c.y * 0x85ebca6bu + s * 0xc2b2ae35u + salt;
    h ^= h >> 16u;
    h *= 0x7feb352du;
    h ^= h >> 15u;
    h *= 0x846ca68bu;
    return h ^ (h >> 16u);
}

fn cell_hash2(cell: vec2f, seed: f32) -> f32 {
    return f32(cell_hash_bits(cell, seed, 0u) >> 8u) / 16777216.0;
}

fn cell_hash2_vec2(cell: vec2f, seed: f32) -> vec2f {
    return vec2f(
        f32(cell_hash_bits(cell, seed, 0u) >> 8u),
        f32(cell_hash_bits(cell, seed, 0x9e3779b9u) >> 8u)
    ) / 16777216.0;
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

// GRID variant: rectangular cells; fractional density leaves a partial last column.
// Callers supply canonical cylinder u in [0,1); close that column at scaled.x=density.
fn cell_grid(uv: vec2f, density: f32) -> vec3f {
    let scaled = uv * density;
    let cell_id = floor(scaled);
    let cell_fract = fract(scaled);

    // Distance to nearest edge
    let d_edge = min(
        min(cell_fract.x, 1.0 - cell_fract.x),
        min(cell_fract.y, 1.0 - cell_fract.y)
    );

    return vec3f(cell_id, min(d_edge, density - scaled.x));
}

// Offset cells straddling an integral circumference share one identity.
// Fractional inputs pass through; periodic production callers use whole columns.
// Wrap the integer ID, not local geometry or fractional hash/seed inputs.
fn cell_offset_id_x(x: f32, density: f32) -> f32 {
    // Only the identity period is integral; keep density/row seeds untouched.
    // Allow f32 decode error, far below one packed fractional-density step.
    let period = round(density);
    if abs(density - period) <= 0.00001 {
        // Keep remainder operands nonnegative: the tested GPU path mishandles
        // signed negative remainders (for example -1 at period 34).
        let n = u32(period);
        let remainder = u32(abs(x)) % n;
        return f32(select(remainder, n - remainder, x < 0.0 && remainder != 0u));
    }
    return x;
}

// HEX: whole cells on a periodic triangular lattice; no partial last column.
// Density selects 4..64 whole columns. The nearest 3x3 sites cover every
// hexagon that can affect a sample, including its existing edge band.
// Filled-cell distance is independent of which site owns the sample.
fn cell_hex_fields(uv: vec2f, density: f32, seed: f32, fill: f32) -> vec4f {
    let count = round(density);
    let row_height = 0.8660254037844386;
    let p = uv * count;
    let base_row = i32(floor(p.y / row_height));
    var nearest = 1e20;
    var owner = vec2f(0.0);
    var all_edge = -100.0;
    var solid_edge = -100.0;
    for (var dy = -1; dy <= 1; dy += 1) {
        let row = base_row + dy;
        let shift = select(0.0, 0.5, (row & 1) == 0);
        let base_col = i32(floor(p.x + shift));
        for (var dx = -1; dx <= 1; dx += 1) {
            let col = base_col + dx;
            let center = vec2f(f32(col) + 0.5 - shift, (f32(row) + 0.5) * row_height);
            let delta = p - center;
            let id = vec2f(cell_offset_id_x(f32(col), count), f32(row));
            let distance = dot(delta, delta);
            if distance < nearest { nearest = distance; owner = id; }
            let q = abs(delta);
            let edge = 0.5 - max(q.x, 0.5 * q.x + row_height * q.y);
            all_edge = max(all_edge, edge);
            if cell_hash2(id, seed) < fill { solid_edge = max(solid_edge, edge); }
        }
    }
    return vec4f(owner, all_edge, solid_edge);
}
fn cell_hex(uv: vec2f, density: f32) -> vec3f {
    return cell_hex_fields(uv, density, 0.0, 1.0).xyz;
}

// BRICK: ordinary staggered rectangles; whole periodic columns, no hidden relief.
fn cell_brick(uv: vec2f, density: f32, seed: f32) -> vec3f {
    let count = round(density);
    let scaled = uv * vec2f(count, count * 0.5);
    let row = floor(scaled.y);
    let shift = select(0.0, 0.5, fract(row * 0.5) < 0.25);
    let p = scaled + vec2f(shift, 0.0);
    let id = vec2f(cell_offset_id_x(floor(p.x), count), row);
    let local = fract(p);
    let edge = min(min(local.x, 1.0 - local.x), min(local.y, 1.0 - local.y));
    return vec3f(id, edge);
}

// Filled rectangles retain their edge support across an empty-cell owner.
// GRID preserves its fractional terminal rectangle; BRICK uses whole columns.
fn cell_rect_fields(uv: vec2f, density: f32, seed: f32, fill: f32, brick: bool) -> vec4f {
    let count = select(density, round(density), brick);
    let height = select(count, count * 0.5, brick);
    var info = cell_grid(uv, density);
    if brick { info = cell_brick(uv, density, seed); }
    let p = uv * vec2f(count, height);
    let columns = i32(ceil(count));
    let base_row = i32(floor(p.y));
    var solid_edge = -100.0;
    for (var dy = -1; dy <= 1; dy += 1) {
        let row = base_row + dy;
        let shift = select(0.0, 0.5, brick && (row & 1) == 0);
        let base_col = i32(floor(p.x + shift));
        for (var dx = -1; dx <= 1; dx += 1) {
            let raw = base_col + dx;
            // UV is in one chart; these neighbours need at most one wrap.
            let cycle = select(0.0, 1.0, raw >= columns) - select(0.0, 1.0, raw < 0);
            let col = raw - i32(cycle) * columns;
            let id = vec2f(f32(col), f32(row));
            let left = cycle * count + f32(col) - shift;
            let right = cycle * count + min(f32(col) + 1.0, count) - shift;
            let bottom = f32(row);
            let edge = min(min(p.x - left, right - p.x), min(p.y - bottom, bottom + 1.0 - p.y));
            if cell_hash2(id, seed) < fill { solid_edge = max(solid_edge, edge); }
        }
    }
    return vec4f(info, solid_edge);
}

fn cell_grid_fields(uv: vec2f, density: f32, seed: f32, fill: f32) -> vec4f {
    return cell_rect_fields(uv, density, seed, fill, false);
}

fn cell_brick_fields(uv: vec2f, density: f32, seed: f32, fill: f32) -> vec4f {
    return cell_rect_fields(uv, density, seed, fill, true);
}

// VORONOI variant: find nearest point from jittered grid
// Radius two covers both nearest sites: two sites are within sqrt(3.25),
// while a site outside this window is at least two cell units away.
fn cell_site_fields(uv: vec2f, period: f32, seed: f32, jitter_scale: f32, fill_seed: f32, fill: f32) -> vec4f {
    let scaled = uv * period;
    let base = vec2i(floor(scaled));
    let n = i32(period);
    var first = 100.0;
    var second = 100.0;
    var owner = vec2f(0.0);
    var nearest_filled = 100.0;
    for (var dy = -2; dy <= 2; dy += 1) {
        for (var dx = -2; dx <= 2; dx += 1) {
            // n=4 aliases the two outer columns: never count a site twice.
            if n == 4 && dx == 2 { continue; }
            let raw_x = base.x + dx;
            let id = vec2f(vec2i((raw_x + n) % n, base.y + dy));
            let jitter = cell_hash2_vec2(id, seed);
            let point = id + jitter * jitter_scale + vec2f(select(0.1, 0.0, jitter_scale == 1.0));
            let d = length(vec2f(shortest_periodic_delta(scaled.x, point.x, period), scaled.y - point.y));
            if cell_hash2(id, fill_seed) < fill { nearest_filled = min(nearest_filled, d); }
            if d < first {
                second = first;
                first = d;
                owner = id;
            } else if d < second {
                second = d;
            }
        }
    }
    let edge = (second - first) * 0.5;
    // Extend filled-site support into empty ownership, rather than dropping
    // the edge band at the bisector. All-site geometry/identity stays unchanged.
    // Omitted sites are too distant to affect the existing 0.005 edge band.
    var solid_edge = -100.0;
    if cell_hash2(owner, fill_seed) < fill {
        solid_edge = edge;
    } else if nearest_filled < 100.0 {
        solid_edge = (first - nearest_filled) * 0.5;
    }
    return vec4f(owner, edge, solid_edge);
}

fn cell_periodic_candidates(uv: vec2f, period: f32, seed: f32, jitter_scale: f32) -> vec3f {
    return cell_site_fields(uv, period, seed, jitter_scale, seed, 0.0).xyz;
}

fn cell_voronoi(uv: vec2f, density: f32, seed: f32) -> vec3f {
    return cell_periodic_candidates(uv, round(density), seed, 0.8);
}

// SHATTER variant: voronoi with higher jitter
fn cell_shatter(uv: vec2f, density: f32, seed: f32) -> vec3f {
    return cell_periodic_candidates(uv, round(density), seed + 42.0, 1.0);
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
    switch variant {
        case 0u: { let fields = cell_grid_fields(uv, density, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); } // GRID
        case 1u: { let fields = cell_hex_fields(uv, density, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); }       // HEX
        case 2u: { let fields = cell_site_fields(uv, round(density), seed, 0.8, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); }  // VORONOI
        case 3u: { cell_info = cell_radial(uv, density, axis, dir); }  // RADIAL
        case 4u: { let fields = cell_site_fields(uv, round(density), seed + 42.0, 1.0, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); }  // SHATTER
        case 5u: { let fields = cell_brick_fields(uv, density, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); } // BRICK
        default: { let fields = cell_grid_fields(uv, density, seed, fill_ratio); cell_info = vec3f(fields.xy, fields.w); }
    }

    let cell_id = cell_info.xy;
    let d_edge = cell_info.z;

    // Determine if cell is solid based on hash and fill ratio
    let cell_value = cell_hash2(cell_id, seed);
    let is_solid = select(cell_value < fill_ratio, cell_info.z > -99.0, variant != 3u);
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
        w_sky = gap_alpha;
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
        default: {}
    }
    let outline = smoothstep(outline_width, 0.0, outline_dist)
        * outline_alpha
        * outline_brightness
        * outline_taper
        * solid_w;

    // Get colors
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);
    let floor_color = wall_color * 0.5;

    // Blend colors based on weights
    let base_rgb = sky_color * w_sky + wall_color * w_wall + floor_color * w_floor;

    // Outline tinted by the brighter of the two cell colors (not hardcoded white)
    let outline_color = max(sky_color, wall_color);
    var rgb = base_rgb + outline_color * outline;

    // Total weight
    let w = w_sky + w_wall + w_floor;
    // LayerSample carries straight color; apply_blend applies the weight once.
    if w > 0.0 { rgb /= w; } else { rgb = vec3f(0.0); }

    // CELL is an enclosure source: return radiance sample + output regions.
    return BoundsResult(LayerSample(rgb, epu_saturate(w)), output_regions, 1.0);
}
