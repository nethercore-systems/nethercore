// ============================================================================
// EPU LAYER DISPATCH
// Environment Processing Unit - dispatch logic for all opcodes
// ============================================================================

// Evaluate bounds opcode - returns sample + modified regions for subsequent features
fn evaluate_bounds_layer(
    dir: vec3f,
    instr: vec4u,
    opcode: u32,
    bounds_dir: vec3f,
    base_regions: RegionWeights
) -> BoundsResult {
    switch opcode {
        case OP_RAMP: {
            return eval_ramp(dir, instr);
        }
        case OP_SECTOR: {
            return eval_sector(dir, instr, base_regions);
        }
        case OP_SILHOUETTE: {
            return eval_silhouette(dir, instr, base_regions);
        }
        case OP_SPLIT: {
            return eval_split(dir, instr, base_regions);
        }
        case OP_CELL: {
            return eval_cell(dir, instr, base_regions);
        }
        case OP_PATCHES: {
            return eval_patches(dir, instr, base_regions);
        }
        case OP_APERTURE: {
            return eval_aperture(dir, instr, base_regions);
        }
        default: {
            return BoundsResult(LayerSample(vec3f(0.0), 0.0), base_regions);
        }
    }
}

fn evaluate_layer(
    dir: vec3f,
    instr: vec4u,
    bounds_dir: vec3f,
    regions: RegionWeights
) -> LayerSample {
    let opcode = instr_opcode(instr);
    let region_mask = instr_region(instr);

    // Always compute region weight from mask - both bounds and features can use it
    let region_w = region_weight(regions, region_mask);

    switch opcode {
        // ====================================================================
        // Feature opcodes (0x08+) - bounds handled by evaluate_bounds_layer
        // ====================================================================
        case OP_DECAL:   { return eval_decal(dir, instr, region_w); }
        case OP_GRID:    { return eval_grid(dir, instr, region_w); }
        case OP_SCATTER: { return eval_scatter(dir, instr, region_w); }
        case OP_FLOW:    { return eval_flow(dir, instr, region_w); }

        // Additional radiance opcodes (0x0C..0x13)
        case OP_TRACE: { return eval_trace(dir, instr, region_w); }
        case OP_VEIL: { return eval_veil(dir, instr, region_w); }
        case OP_ATMOSPHERE: {
            return eval_atmosphere(dir, instr, bounds_dir, region_w);
        }
        case OP_PLANE: {
            return eval_plane(dir, instr, region_w);
        }
        case OP_CELESTIAL: {
            return eval_celestial(dir, instr, region_w);
        }
        case OP_PORTAL: {
            return eval_portal(dir, instr, region_w);
        }
        case OP_LOBE_RADIANCE: {
            return eval_lobe_radiance(dir, instr, region_w);
        }
        case OP_BAND_RADIANCE: {
            return eval_band_radiance(dir, instr, region_w);
        }

        default: { return LayerSample(vec3f(0.0), 0.0); }
    }
}
