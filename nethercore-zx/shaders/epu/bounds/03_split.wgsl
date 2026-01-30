// ============================================================================
// SPLIT - Planar Cut Enclosure Source (0x04)
// Divides the sphere using one or more planar cuts for geometric divisions.
// 128-bit packed fields:
//   color_a: Sky region base color (RGB24)
//   color_b: Wall region base color (RGB24)
//   param_a: Blend width (0..255 -> 0.0..1.0)
//   param_b: Wedge angle (0..255 -> 0..180 degrees) - WEDGE variant
//   param_c: Band count (0..255 -> 2..16) - BANDS; side count - PRISM
//   param_d: Band offset (0..255 -> 0.0..1.0) - BANDS; rotation - PRISM
//   direction: Cut plane normal / primary axis (oct-u16)
//   variant_id: 0=HALF, 1=WEDGE, 2=CORNER, 3=BANDS, 4=CROSS, 5=PRISM
// ============================================================================

// Build orthonormal basis around a given axis
fn split_build_basis(n0: vec3f) -> mat3x3f {
    // Choose reference vector that is not parallel to n0
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(n0.y) > 0.9);
    let t = normalize(cross(ref_vec, n0));
    let b = normalize(cross(n0, t));
    return mat3x3f(t, b, n0);
}

// Rotate vector around axis by angle (radians)
fn split_rotate_around_axis(v: vec3f, axis: vec3f, angle: f32) -> vec3f {
    let c = cos(angle);
    let s = sin(angle);
    let k = axis;
    // Rodrigues rotation formula
    return v * c + cross(k, v) * s + k * dot(k, v) * (1.0 - c);
}

// HALF variant (0): Single plane dividing space in two
fn split_half(dir: vec3f, n0: vec3f, bw: f32) -> RegionWeights {
    let d = -dot(dir, n0);  // Negative on positive side of plane (sky)
    return regions_from_signed_distance(d, bw);
}

// WEDGE variant (1): Two planes at configurable angle creating a wedge
fn split_wedge(dir: vec3f, n0: vec3f, basis: mat3x3f, wedge_angle: f32, bw: f32) -> RegionWeights {
    let b_axis = basis[1];
    let n1 = split_rotate_around_axis(n0, b_axis, wedge_angle);

    let d0 = dot(dir, n0);
    let d1 = -dot(dir, n1);  // Negative for inside

    // Signed distance: negative inside wedge, positive outside
    let inside = min(d0, d1);
    return regions_from_signed_distance(-inside, bw);
}

// CORNER variant (2): Three orthogonal planes creating an octant region
fn split_corner(dir: vec3f, n0: vec3f, basis: mat3x3f, bw: f32) -> vec3f {
    let t_axis = basis[0];
    let b_axis = basis[1];

    // Signed distances to three orthogonal planes
    let d0 = dot(dir, n0);
    let d1 = dot(dir, t_axis);
    let d2 = dot(dir, b_axis);

    // Compute smooth octant membership
    let s0 = smoothstep(-bw, bw, d0);
    let s1 = smoothstep(-bw, bw, d1);
    let s2 = smoothstep(-bw, bw, d2);

    // Primary octant (all positive): sky
    // Adjacent octants (one negative): wall
    // Opposite octant (all negative): floor
    let primary = s0 * s1 * s2;
    let opposite = (1.0 - s0) * (1.0 - s1) * (1.0 - s2);
    let adjacent = 1.0 - primary - opposite;

    return vec3f(primary, max(0.0, adjacent), opposite);
}

// BANDS variant (3): Repeating parallel planes creating stripes
fn split_bands(dir: vec3f, n0: vec3f, band_count: f32, band_offset: f32, bw: f32) -> RegionWeights {
    let u = dot(dir, n0) * 0.5 + 0.5 + band_offset;
    let stripe = fract(u * band_count);
    // Distance from center of stripe (0.5)
    let d = stripe - 0.5;  // Ranges -0.5 to 0.5
    return regions_from_signed_distance(d, bw * band_count);
}

// CROSS variant (4): Two perpendicular planes creating quadrants
fn split_cross(dir: vec3f, n0: vec3f, basis: mat3x3f, bw: f32) -> RegionWeights {
    let t_axis = basis[0];
    let q0 = dot(dir, n0);
    let q1 = dot(dir, t_axis);

    // Product is positive when same sign (diagonal quadrants)
    let d = -q0 * q1;  // Negative for "same sign" quadrants (sky)
    return regions_from_signed_distance(d, bw);
}

// PRISM variant (5): Three planes at 120 degrees creating triangular prism regions
fn split_prism(dir: vec3f, n0: vec3f, basis: mat3x3f, side_count: f32, rotation: f32, bw: f32) -> vec3f {
    let t_axis = basis[0];
    let b_axis = basis[1];

    // Project direction onto the plane perpendicular to n0
    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);
    let z_proj = dot(dir, n0);

    // Compute angle around n0 axis
    let angle01 = fract(atan2(b_proj, t_proj) / TAU + 0.5 + rotation);

    // Quantize into sectors based on side_count (clamped to 3..16)
    let sectors = clamp(side_count, 3.0, 16.0);
    let sector_angle = 1.0 / sectors;
    let sector_id = floor(angle01 / sector_angle);
    let sector_fract = fract(angle01 / sector_angle);

    // Distance to sector edge (for AA)
    let d_sector_edge = min(sector_fract, 1.0 - sector_fract);
    let sector_blend = smoothstep(0.0, bw * sectors, d_sector_edge);

    // Determine cap vs side based on z projection
    let cap_threshold = 0.95;  // Above this is ceiling cap, below -threshold is floor cap
    let ceiling_blend = smoothstep(cap_threshold - bw, cap_threshold + bw, z_proj);
    let floor_blend = smoothstep(-cap_threshold + bw, -cap_threshold - bw, z_proj);

    // Assign regions:
    // w_sky = ceiling cap
    // w_wall = side faces (middle band)
    // w_floor = floor cap
    let w_sky = ceiling_blend * sector_blend;
    let w_floor = floor_blend * sector_blend;
    let w_wall = max(0.0, 1.0 - w_sky - w_floor) * sector_blend;

    return vec3f(w_sky, w_wall, w_floor);
}

fn eval_split(
    dir: vec3f,
    instr: vec4u,
    base_regions: RegionWeights,
) -> BoundsResult {
    // Decode plane normal from direction field
    let n0 = decode_dir16(instr_dir16(instr));

    // Build orthonormal basis around n0
    let basis = split_build_basis(n0);

    // Extract parameters
    let pa = instr_a(instr);
    let pb = instr_b(instr);
    let pc = instr_c(instr);
    let pd = instr_d(instr);

    // blend_width: 0..255 -> 0.0..0.2
    let blend_width = u8_to_01(pa) * 1.0;
    // Minimum blend width for AA stability
    let bw = max(blend_width, 0.001);

    // Wedge angle: 0..255 -> 0..PI radians (0..180 degrees)
    let wedge_angle = u8_to_01(pb) * PI;

    // Band count: 0..255 -> 2..16
    let band_count = mix(2.0, 16.0, u8_to_01(pc));

    // Band offset / rotation: 0..255 -> 0.0..1.0
    let band_offset = u8_to_01(pd);

    // Extract variant from meta5
    let variant = instr_variant_id(instr);

    // Get colors
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);

    // Compute region weights based on variant
    var w_sky: f32 = 0.0;
    var w_wall: f32 = 0.0;
    var w_floor: f32 = 0.0;

    switch variant {
        case 0u: {
            // HALF: Single plane dividing space in two
            let regions = split_half(dir, n0, bw);
            w_sky = regions.sky;
            w_wall = regions.wall;
            w_floor = regions.floor;
        }
        case 1u: {
            // WEDGE: Two planes creating a wedge/slice
            let regions = split_wedge(dir, n0, basis, wedge_angle, bw);
            w_sky = regions.sky;
            w_wall = regions.wall;
            w_floor = regions.floor;
        }
        case 2u: {
            // CORNER: Three planes creating octant regions
            let weights = split_corner(dir, n0, basis, bw);
            w_sky = weights.x;
            w_wall = weights.y;
            w_floor = weights.z;
        }
        case 3u: {
            // BANDS: Repeating parallel planes creating stripes
            let regions = split_bands(dir, n0, band_count, band_offset, bw);
            w_sky = regions.sky;
            w_wall = regions.wall;
            w_floor = regions.floor;
        }
        case 4u: {
            // CROSS: Two perpendicular planes creating quadrants
            let regions = split_cross(dir, n0, basis, bw);
            w_sky = regions.sky;
            w_wall = regions.wall;
            w_floor = regions.floor;
        }
        case 5u: {
            // PRISM: Three planes at 120 degrees creating triangular prism
            let weights = split_prism(dir, n0, basis, band_count, band_offset, bw);
            w_sky = weights.x;
            w_wall = weights.y;
            w_floor = weights.z;
        }
        default: {
            // Reserved variants (6..7) default to HALF behavior
            let regions = split_half(dir, n0, bw);
            w_sky = regions.sky;
            w_wall = regions.wall;
            w_floor = regions.floor;
        }
    }

    // For variants with floor region, we need a floor color.
    // Use a darkened wall color for floor (spec says floor is unused for most variants)
    let floor_color = wall_color * 0.5;

    // Blend colors based on region weights
    let rgb = sky_color * w_sky + wall_color * w_wall + floor_color * w_floor;

    // SPLIT defines its own regions
    let output_regions = RegionWeights(w_sky, w_wall, w_floor);
    return BoundsResult(LayerSample(rgb, 1.0), output_regions);
}
