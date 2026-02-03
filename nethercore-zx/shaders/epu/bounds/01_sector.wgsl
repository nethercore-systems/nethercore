// @epu_meta_begin
// opcode = 0x02
// name = SECTOR
// kind = bounds
// variants = [BOX, TUNNEL, CAVE]
// domains = []
// field intensity = { label="opening", map="u8_01" }
// field param_a = { label="azimuth", map="u8_01" }
// field param_b = { label="width", map="u8_01" }
// field param_c = { label="-", map="u8_01" }
// field param_d = { label="-", map="u8_01" }
// @epu_meta_end

// ============================================================================
// SECTOR - Angular Wedge Enclosure (0x02)
// 128-bit packed fields:
//   color_a: Sky/opening color (RGB24)
//   color_b: Wall color (RGB24)
//   intensity: Opening strength (0..255 -> 0.0..1.0)
//   param_a: Opening center azimuth (0..255 -> 0.0..1.0)
//   param_b: Opening width (0..255 -> 0.0..1.0)
//   param_c: Reserved (set to 0)
//   param_d: Reserved (set to 0)
//   direction: Up axis (oct-u16), should match RAMP.up
//   variant_id: 0 BOX, 1 TUNNEL, 2 CAVE (from meta5)
//
// SECTOR defines a 3-band sky/wall/floor split based on wedge geometry:
//   - Sky (opening): inside the wedge
//   - Wall (edge): the wedge boundary band
//   - Floor (outside): outside the wedge
// ============================================================================

fn eval_sector(
    dir: vec3f,
    instr: vec4u,
    base_regions: RegionWeights,  // Kept for interface compatibility, but unused
) -> BoundsResult {
    // Decode up axis from direction field
    let up = decode_dir16(instr_dir16(instr));

    // Build axis-cylinder basis around up vector
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(up.y) > 0.9);
    let t_axis = normalize(cross(ref_vec, up));
    let b_axis = normalize(cross(up, t_axis));

    // Project direction onto horizontal plane (perpendicular to up)
    let t_proj = dot(dir, t_axis);
    let b_proj = dot(dir, b_axis);

    // Compute wrap-around azimuth u01 in [0, 1)
    let u01 = atan2(b_proj, t_proj) / TAU + 0.5;

    // Parameters
    let intensity = u8_to_01(instr_intensity(instr));
    let center_u01 = u8_to_01(instr_a(instr));
    let width = u8_to_01(instr_b(instr));

    // Compute circular distance (handles wrap-around at u=0/1)
    let d0 = abs(u01 - center_u01);
    let dist = min(d0, 1.0 - d0);

    // Half-width of the opening wedge
    let half_width = width * 0.5;

    // Compute signed distance: negative inside wedge, positive outside
    let d = dist - half_width;

    // Band width for smooth transitions
    let bw = max(0.01, half_width * 0.25);

    // Compute base regions from signed distance (independent of base_regions)
    let regions_open = regions_from_signed_distance(d, bw);

    // Blend "no opening" -> "full opening" using intensity.
    // When intensity=0: all floor (closed)
    // When intensity=1: regions_open (full opening)
    let regions_closed = RegionWeights(0.0, 0.0, 1.0);
    let regions = RegionWeights(
        mix(regions_closed.sky, regions_open.sky, intensity),
        mix(regions_closed.wall, regions_open.wall, intensity),
        mix(regions_closed.floor, regions_open.floor, intensity)
    );

    // Apply variant shaping to the region weights
    let variant = instr_variant_id(instr);
    var output_regions = regions;

    switch variant {
        case 0u: {
            // BOX: uniform opening (no modification needed)
        }
        case 1u: {
            // TUNNEL: enhanced opening in wall-like areas (horizontal directions)
            let y = abs(dot(dir, up));
            let horiz_bias = 1.0 - y; // More horizontal = more opening
            let boost = horiz_bias * 0.5;
            output_regions = RegionWeights(
                min(1.0, regions.sky * (1.0 + boost)),
                max(0.0, regions.wall - regions.sky * boost),
                regions.floor
            );
        }
        case 2u: {
            // CAVE: opening biased downward
            let y = dot(dir, up);
            let down_bias = smoothstep(0.5, -0.5, y);
            output_regions = RegionWeights(
                regions.sky * down_bias,
                regions.wall * down_bias,
                regions.floor + (regions.sky + regions.wall) * (1.0 - down_bias)
            );
        }
        default: {}
    }

    // Get colors and render with 3-band split
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);
    let floor_color = wall_color * 0.5;
    let rgb = sky_color * output_regions.sky + wall_color * output_regions.wall + floor_color * output_regions.floor;

    return BoundsResult(LayerSample(rgb, 1.0), output_regions);
}
