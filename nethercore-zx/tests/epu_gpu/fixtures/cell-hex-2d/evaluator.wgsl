fn eval_cell_2d(
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
        case 1u: { cell_info = cell_hex_2d(uv, density); }       // HEX
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
            var outline_x = fract(uv.x * density);
            if abs(density - round(density)) <= 0.00001 {
                // Keep relief in this offset cell's chart, not the unshifted
                // lattice chart whose fract cut can run through its interior.
                outline_x = shortest_periodic_delta(uv.x * density, cell_id.x, round(density));
            }
            let brick_gate = smoothstep(
                -0.22,
                0.7,
                epu_relief_wave(vec2f(cell_id.y * 0.41 + outline_x, uv.y * density * 1.27), seed * 0.013)
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
