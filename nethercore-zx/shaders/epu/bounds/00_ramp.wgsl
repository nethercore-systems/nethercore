// @epu_meta_begin
// opcode = 0x01
// name = RAMP
// kind = bounds
// variants = []
// domains = []
// field intensity = { label="softness", map="u8_lerp", min=0.01, max=0.5 }
// field param_a = { label="wall_r", map="u8_01" }
// field param_b = { label="wall_g", map="u8_01" }
// field param_c = { label="wall_b", map="u8_01" }
// field param_d = { label="thresholds", map="u8_01" }
// @epu_meta_end

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
) -> BoundsResult {
    // Packing:
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

    // Heights are packed in param_d
    let pd = instr_d(instr);
    let ceil_q = (pd >> 4u) & 0xFu;
    let floor_q = pd & 0xFu;

    // Soften to a small minimum to avoid hard banding.
    let soft = mix(0.01, 0.5, u8_to_01(instr_intensity(instr)));

    var ceil_y = nibble_to_signed_1(ceil_q);
    var floor_y = nibble_to_signed_1(floor_q);
    if floor_y > ceil_y {
        // Ensure a valid ordering; swap if authored incorrectly.
        let t = floor_y;
        floor_y = ceil_y;
        ceil_y = t;
    }

    // Decode the up vector from the instruction
    let up = decode_dir16(instr_dir16(instr));

    // Compute region weights from up vector and thresholds
    let y = dot(dir, up);
    let w_sky = smoothstep(ceil_y - soft, ceil_y + soft, y);
    let w_floor = smoothstep(floor_y + soft, floor_y - soft, y);
    let w_wall = 1.0 - w_sky - w_floor;

    let rgb = sky * w_sky + wall * w_wall + floor * w_floor;

    // RAMP is a base layer: treat as fully weighted (w=1).
    let regions = RegionWeights(w_sky, w_wall, w_floor);
    return BoundsResult(LayerSample(rgb, 1.0), regions);
}
