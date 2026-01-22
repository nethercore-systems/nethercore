// ============================================================================
// RAMP - Enclosure Gradient
// 128-bit packed fields:
//   color_a: Sky/ceiling color (RGB24)
//   color_b: Floor/ground color (RGB24)
//   param_a: Wall/horizon color R (0..255)
//   param_b: Wall/horizon color G (0..255)
//   param_c: Wall/horizon color B (0..255)
//   param_d[7:4]: ceil_y threshold (0..15 -> -1..1)
//   param_d[3:0]: floor_y threshold (0..15 -> -1..1)
//   intensity: Softness (0..255 -> 0.01..0.5)
//   direction: Up vector (oct-u16)
// ============================================================================

fn eval_ramp(
    dir: vec3f,
    instr: vec4u,
    enc: EnclosureConfig,
) -> LayerSample {
    // v2 packing matches `EpuBuilder::ramp_enclosure()`:
    // - color_a: sky/ceiling
    // - color_b: floor/ground
    // - param_a/b/c: wall/horizon
    let sky = instr_color_a(instr);
    let floor = instr_color_b(instr);

    // Wall/horizon color from param_a/b/c
    let wall = vec3f(
        u8_to_01(instr_a(instr)),
        u8_to_01(instr_b(instr)),
        u8_to_01(instr_c(instr))
    );

    let weights = compute_region_weights(dir, enc);
    let rgb = sky * weights.sky + wall * weights.wall + floor * weights.floor;

    // RAMP is a base layer: treat as fully weighted (w=1).
    return LayerSample(rgb, 1.0);
}
