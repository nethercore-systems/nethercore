// @epu_meta_begin
// opcode = 0x18
// name = SCATTER_PHASED
// kind = radiance
// variants = [STARS, DUST, WINDOWS, BUBBLES, EMBERS, RAIN, SNOW]
// domains = [DIRECT3D, AXIS_CYL, AXIS_POLAR, TANGENT_LOCAL]
// field intensity = { label="brightness", map="u8_01" }
// field param_a = { label="density", map="u8_lerp", min=1.0, max=256.0 }
// field param_b = { label="size (advanced raw: radius depends on density and variant)", map="u8_lerp", min=0.0, max=255.0 }
// field param_c = { label="phase (turns; guest-owned, wraps at 1)", map="u8_lerp", min=0.0, max=0.99609375 }
// field param_d = { label="seed (raw byte)", map="u8_lerp", min=0.0, max=255.0 }
// @epu_meta_end

// Opt-in SCATTER geometry/identity; no clock, movement, or seed mutation.
// param_c = guest-owned phase / 256 turns; alpha_b = modulation depth / 15.
// Depth 0 is steady; depth 15 reaches fully off/on. alpha_a remains layer opacity.
// Integer per-point rates close at phase wrap; speed/direction/holds are guest-owned.
fn scatter_phased_mod(h: vec4f, phase: f32) -> f32 {
    let rate = 1.0 + floor(h.z * 4.0);
    let wave = 0.5 + 0.5 * cos(TAU * (phase * rate + h.w));
    // Short dark/bright holds retain actual endpoints at the accepted +4 stride.
    return smoothstep(0.05, 0.95, wave);
}
