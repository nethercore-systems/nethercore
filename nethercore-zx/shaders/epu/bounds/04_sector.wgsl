// ============================================================================
// SECTOR - Angular Wedge Enclosure Modifier (vNext 0x02)
// 128-bit packed fields:
//   color_a: Unused (set to 0)
//   color_b: Unused (set to 0)
//   intensity: Opening strength (0..255 -> 0.0..1.0)
//   param_a: Opening center azimuth (0..255 -> 0.0..1.0)
//   param_b: Opening width (0..255 -> 0.0..1.0)
//   param_c: Reserved (set to 0)
//   param_d: Reserved (set to 0)
//   direction: Up axis (oct-u16), should match RAMP.up
//   variant_id: 0 BOX, 1 TUNNEL, 2 CAVE
//
// SECTOR creates an azimuthal opening in the enclosure, promoting wall regions
// to sky within the opening sector. This signals "not a perfect sphere" for
// interiors, tunnels, and cave mouths.
// ============================================================================

fn eval_sector(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
) -> LayerSample {
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

    // Get baseline region weights from enclosure config
    let baseline = compute_region_weights(dir, enc);

    // Apply variant shaping
    let variant = instr_variant_id(instr);
    var open = open_base;

    switch variant {
        case 0u: {
            // BOX: uniform opening (no vertical modulation)
            // Opening applies uniformly across all vertical angles
        }
        case 1u: {
            // TUNNEL: opening extends vertically (no floor/ceiling cap)
            // Full opening strength regardless of vertical position
            // Boost opening in wall region
            open = open_base * (1.0 + baseline.wall * 0.5);
        }
        case 2u: {
            // CAVE: opening biased downward (floor visible, ceiling blocked)
            // Stronger opening near floor, weaker near ceiling
            let y = dot(dir, up);
            let down_bias = smoothstep(0.5, -0.5, y);
            open = open_base * down_bias;
        }
        default: {
            // Default to BOX behavior
        }
    }

    // Promote sky into opening sector:
    // w_sky += open * w_wall; w_wall -= open * w_wall
    // This shifts wall region toward sky within the opening.
    //
    // Compute modified region weights for the opening effect.
    // The opening mask represents where wall becomes sky.
    let opening_mask = open * baseline.wall;

    // Compute the modified region weights
    let new_sky = baseline.sky + opening_mask;
    let new_wall = baseline.wall - opening_mask;
    let new_floor = baseline.floor;

    // SECTOR is an enclosure modifier that changes the perceived region weights.
    // Since color_a/color_b are unused, we output using the RAMP's region colors
    // weighted by the modified weights. However, for proper layering, we output
    // a blend factor that represents the opening contribution.
    //
    // For enclosure-aware blending: output RGB=0 with the sky-promotion weight.
    // When used with appropriate blend modes, this modifies the perception.
    // Full enclosure modifier semantics would require architectural changes.
    return LayerSample(vec3f(0.0), opening_mask);
}
