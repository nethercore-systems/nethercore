// ============================================================================
// SECTOR - Angular Wedge Enclosure Modifier (0x02)
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
// SECTOR creates an azimuthal opening in the enclosure, promoting wall regions
// to sky within the opening sector. This signals "not a perfect sphere" for
// interiors, tunnels, and cave mouths.
// ============================================================================

fn eval_sector(
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

    // Compute opening weight with smoothstep falloff
    // smoothstep(width, 0, dist) = 1 at center, 0 at edge
    let half_width = width * 0.5;
    let open_base = smoothstep(half_width, 0.0, dist) * intensity;

    // Use baseline region weights passed in
    let baseline = base_regions;

    // Apply variant shaping
    let variant = instr_variant_id(instr);
    var open = open_base;

    switch variant {
        case 0u: {
            // BOX: uniform opening (no vertical modulation)
        }
        case 1u: {
            // TUNNEL: boost opening in wall region
            open = open_base * (1.0 + baseline.wall * 0.5);
        }
        case 2u: {
            // CAVE: opening biased downward
            let y = dot(dir, up);
            let down_bias = smoothstep(0.5, -0.5, y);
            open = open_base * down_bias;
        }
        default: {}
    }

    // Clamp open to [0,1] to prevent negative wall weight (TUNNEL can exceed 1.0)
    open = min(open, 1.0);

    // Compute modified region weights (wall -> sky in opening)
    let opening_mask = open * baseline.wall;
    let modified_regions = RegionWeights(
        baseline.sky + opening_mask,
        baseline.wall - opening_mask,
        baseline.floor
    );

    // Get colors and render
    let sky_color = instr_color_a(instr);
    let wall_color = instr_color_b(instr);
    let rgb = sky_color * modified_regions.sky + wall_color * modified_regions.wall + wall_color * 0.5 * modified_regions.floor;

    return BoundsResult(LayerSample(rgb, 1.0), modified_regions);
}
