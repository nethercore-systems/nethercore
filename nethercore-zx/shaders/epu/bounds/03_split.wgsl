// @epu_meta_begin
// opcode = 0x04
// name = SPLIT
// kind = bounds
// variants = [HALF, WEDGE, CORNER, BANDS, CROSS, PRISM, TIER, FACE]
// domains = []
// field intensity = { label="-", map="u8_01" }
// field param_a = { label="blend_width", map="u8_01" }
// field param_b = { label="angle", map="u8_lerp", min=0.0, max=180.0, unit="deg" }
// field param_c = { label="sides", map="u8_lerp", min=2.0, max=16.0 }
// field param_d = { label="offset", map="u8_01" }
// @epu_meta_end

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
//   variant_id: 0=HALF, 1=WEDGE, 2=CORNER, 3=BANDS, 4=CROSS, 5=PRISM, 6=TIER, 7=FACE
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
fn split_bands(dir: vec3f, n0: vec3f, basis: mat3x3f, band_count: f32, band_offset: f32, bw: f32) -> RegionWeights {
    let u = dot(dir, n0) * 0.5 + 0.5 + band_offset;
    let cross_phase = vec2f(dot(dir, basis[0]), dot(dir, basis[1]));
    let sweep = dot(cross_phase, vec2f(0.63, -0.77));
    let relief = epu_relief_wave(cross_phase * vec2f(1.35, 1.05), band_offset + band_count * 0.031);
    let secondary = epu_relief_wave(cross_phase.yx * vec2f(0.82, 1.41), band_offset + 0.37);
    let warp = (relief * 0.075 + secondary * 0.04 + sweep * 0.085) / max(band_count, 1.0);
    let local_bw = bw * band_count * mix(0.84, 1.2, relief * 0.5 + 0.5);
    let d = epu_periodic_centered(u * band_count + warp);
    return regions_from_signed_distance(d, local_bw);
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
    let phase_warp = (
        sin(dot(vec2f(t_proj, b_proj), vec2f(4.1, -3.7)) + rotation * TAU)
        + sin(dot(vec2f(t_proj, b_proj), vec2f(2.3, 5.9)) - rotation * PI)
    ) * (0.04 / sectors);
    let relief = epu_relief_wave(
        vec2f(t_proj, b_proj) * vec2f(1.9, 1.35) + vec2f(z_proj * 0.75, 0.0),
        rotation + sectors * 0.021
    );
    let height_shear = z_proj * mix(-0.18, 0.18, epu_hash11(rotation * 97.0 + sectors * 13.0));
    let sector_phase = angle01 * sectors + phase_warp + height_shear + relief * (0.11 / sectors);
    let d_sector_edge = epu_periodic_edge_distance(sector_phase);
    let sector_blend = smoothstep(0.0, bw * sectors * mix(0.88, 1.18, relief * 0.5 + 0.5), d_sector_edge);

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

// TIER variant (6): Stepped structural split with a broad middle shelf band.
fn split_tier(
    dir: vec3f,
    n0: vec3f,
    basis: mat3x3f,
    tilt_angle: f32,
    tier_span: f32,
    tier_center: f32,
    bw: f32,
) -> vec3f {
    let tilt = mix(-0.35, 0.35, tilt_angle / PI);
    let profile = dot(dir, n0) + dot(dir, basis[1]) * tilt;
    let half_span = max(tier_span * 0.5, 0.02);

    let sky = smoothstep(tier_center + half_span - bw, tier_center + half_span + bw, profile);
    let floor = 1.0 - smoothstep(tier_center - half_span - bw, tier_center - half_span + bw, profile);
    let wall = max(0.0, 1.0 - sky - floor);

    return vec3f(sky, wall, floor);
}

// FACE variant (7): One dominant structural face with sky/floor compressed to either side.
fn split_face(
    dir: vec3f,
    n0: vec3f,
    basis: mat3x3f,
    tilt_angle: f32,
    face_width: f32,
    face_center: f32,
    bw: f32,
) -> vec3f {
    let tilt = mix(-0.25, 0.25, tilt_angle / PI);
    let profile = dot(dir, n0) + dot(dir, basis[1]) * tilt - face_center;
    let width = max(face_width, 0.03);

    let wall_raw = 1.0 - smoothstep(width, width + bw, abs(profile));
    let sky_raw = smoothstep(width * 0.4, width + bw, profile);
    let floor_raw = smoothstep(width * 0.4, width + bw, -profile);
    let total = max(wall_raw + sky_raw + floor_raw, 0.0001);

    return vec3f(sky_raw, wall_raw, floor_raw) / total;
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
    let blend_width = u8_to_01(pa) * 0.2;
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
            let regions = split_bands(dir, n0, basis, band_count, band_offset, bw);
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
        case 6u: {
            // TIER: Broad stepped middle band for shelf / terrace / layered structure
            let weights = split_tier(dir, n0, basis, wedge_angle, u8_to_01(pc), band_offset * 2.0 - 1.0, bw);
            w_sky = weights.x;
            w_wall = weights.y;
            w_floor = weights.z;
        }
        case 7u: {
            // FACE: One dominant wall/far-field face with compressed sky/floor edges
            let weights = split_face(dir, n0, basis, wedge_angle, mix(0.08, 0.45, u8_to_01(pc)), band_offset * 1.2 - 0.6, bw);
            w_sky = weights.x;
            w_wall = weights.y;
            w_floor = weights.z;
        }
        default: {
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
    let paint_alpha = instr_alpha_a_f32(instr);

    // SPLIT defines its own regions
    let output_regions = RegionWeights(w_sky, w_wall, w_floor);
    return BoundsResult(LayerSample(rgb, paint_alpha), output_regions, 1.0);
}
