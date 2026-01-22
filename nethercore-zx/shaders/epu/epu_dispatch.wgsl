// ============================================================================
// EPU LAYER DISPATCH
// Environment Processing Unit - dispatch logic for all opcodes
// ============================================================================

fn evaluate_layer(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
    regions: RegionWeights,
    time: f32
) -> LayerSample {
    let opcode = instr_opcode(instr);
    let region_mask = instr_region(instr);

    let is_feature = opcode >= OP_FEATURE_MIN;
    let region_w = select(1.0, region_weight(regions, region_mask), is_feature);

    switch opcode {
        // ====================================================================
        // Enclosure opcodes (0x01-0x07)
        // ====================================================================
        case OP_RAMP: { return eval_ramp(dir, instr, enc); }

        // SECTOR (0x02) - vNext enclosure modifier: azimuthal opening wedge
        // Replaces v2 LOBE at this opcode slot. Uses meta5 to distinguish:
        // - meta5 == 0: v2 LOBE behavior (directional glow)
        // - meta5 != 0: vNext SECTOR behavior (angular wedge enclosure modifier)
        case OP_SECTOR: {
            let meta5 = instr_meta5(instr);
            if meta5 == 0u {
                // v2 compatibility: treat as LOBE (directional glow)
                return eval_lobe(dir, instr, time);
            } else {
                // vNext: SECTOR angular wedge modifier
                return eval_sector(dir, instr, enc);
            }
        }
        // SILHOUETTE (0x03) - vNext enclosure modifier: skyline/horizon cutout
        // Replaces v2 BAND at this opcode slot. Uses meta5 to distinguish:
        // - meta5 == 0: v2 BAND behavior (horizon ring)
        // - meta5 != 0: vNext SILHOUETTE behavior (skyline cutout)
        case OP_SILHOUETTE: {
            let meta5 = instr_meta5(instr);
            if meta5 == 0u {
                // v2 compatibility: treat as BAND (horizon ring)
                return eval_band(dir, instr, time);
            } else {
                // vNext: SILHOUETTE skyline/horizon cutout
                return eval_silhouette(dir, instr, enc, time);
            }
        }

        // SPLIT (0x04) - vNext enclosure source: planar cut dividing space
        // Replaces v2 FOG at this opcode slot. v2 FOG moves to ATMOSPHERE (0x0E).
        case OP_SPLIT: { return eval_split(dir, instr); }

        // vNext enclosure opcodes (new slots)
        case OP_CELL: { return eval_cell(dir, instr); }
        case OP_PATCHES: { return eval_patches(dir, instr); }
        case OP_APERTURE: { return eval_aperture(dir, instr, enc); }

        // ====================================================================
        // Radiance opcodes (0x08-0x13)
        // ====================================================================
        case OP_DECAL:   { return eval_decal(dir, instr, region_w, time); }
        case OP_GRID:    { return eval_grid(dir, instr, region_w, time); }
        case OP_SCATTER: { return eval_scatter(dir, instr, region_w, time); }
        case OP_FLOW:    { return eval_flow(dir, instr, region_w, time); }

        // vNext radiance opcodes (new slots)
        case OP_TRACE: { return eval_trace(dir, instr, region_w, time); }
        case OP_VEIL: { return eval_veil(dir, instr, region_w, time); }
        case OP_ATMOSPHERE: {
            return eval_atmosphere(dir, instr, enc, region_w, time);
        }
        case OP_PLANE: {
            return eval_plane(dir, instr, region_w, time);
        }
        case OP_CELESTIAL: {
            return eval_celestial(dir, instr, region_w, time);
        }
        case OP_PORTAL: {
            return eval_portal(dir, instr, region_w, time);
        }
        case OP_LOBE_V2: {
            return eval_lobe_v2(dir, instr, region_w, time);
        }
        case OP_BAND_V2: {
            return eval_band_v2(dir, instr, region_w, time);
        }

        default: { return LayerSample(vec3f(0.0), 0.0); }
    }
}
