// ============================================================================
// EPU COMMON TYPES, DECODING, AND HELPERS
// Environment Processing Unit - shared definitions
// EPU: 128-bit instructions with direct RGB24 colors
// ============================================================================

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;

// ============================================================================
// OPCODE CONSTANTS (5-bit, 32 possible)
// ============================================================================

// Opcode ranges:
// - 0x00: NOP (universal)
// - 0x01..0x07: bounds ops (low-frequency / enclosure)
// - 0x08..0x1F: feature ops (high-frequency motifs)
const OP_NOP: u32 = 0x00u;
const OP_RAMP: u32 = 0x01u;

// Enclosure opcodes (0x02-0x07).
const OP_SECTOR: u32 = 0x02u;       // Azimuthal opening wedge modifier
const OP_SILHOUETTE: u32 = 0x03u;   // Skyline/horizon cutout modifier
const OP_SPLIT: u32 = 0x04u;        // Planar cut enclosure source
const OP_CELL: u32 = 0x05u;         // Voronoi cell partitioning
const OP_PATCHES: u32 = 0x06u;      // Noise-based patches
const OP_APERTURE: u32 = 0x07u;     // Shaped opening/vignette enclosure modifier

const OP_FEATURE_MIN: u32 = 0x08u;
const OP_DECAL: u32 = 0x08u;
const OP_GRID: u32 = 0x09u;
const OP_SCATTER: u32 = 0x0Au;
const OP_FLOW: u32 = 0x0Bu;

// Feature opcodes (0x0C-0x14+)
const OP_TRACE: u32 = 0x0Cu;        // Procedural line/crack patterns (lightning, cracks, lead lines, filaments)
const OP_VEIL: u32 = 0x0Du;         // Curtain/ribbon effects (curtains, pillars, laser bars, rain wall, shards)
const OP_ATMOSPHERE: u32 = 0x0Eu;   // Atmospheric absorption + scattering
const OP_PLANE: u32 = 0x0Fu;        // Ground plane textures
const OP_CELESTIAL: u32 = 0x10u;    // Moon/sun/planet bodies
const OP_PORTAL: u32 = 0x11u;       // Swirling vortex/portal effect
const OP_LOBE_RADIANCE: u32 = 0x12u; // Region-masked directional glow
const OP_BAND_RADIANCE: u32 = 0x13u; // Region-masked horizon band
const OP_MOTTLE: u32 = 0x14u;       // Abstract texture breakup / base variation
const OP_ADVECT: u32 = 0x15u;       // Broad transport / sheet motion carrier
const OP_SURFACE: u32 = 0x16u;      // Broad material / surface response carrier
const OP_MASS: u32 = 0x17u;         // Broad scene-owning body carrier
// 0x18..0x1F reserved for future feature ops

// ============================================================================
// REGION MASK CONSTANTS (3-bit bitfield)
// SKY=0b100, WALLS=0b010, FLOOR=0b001
// ============================================================================

const REGION_NONE: u32 = 0u;        // 0b000 - layer disabled
const REGION_FLOOR: u32 = 1u;       // 0b001
const REGION_WALLS: u32 = 2u;       // 0b010
const REGION_WALLS_FLOOR: u32 = 3u; // 0b011
const REGION_SKY: u32 = 4u;         // 0b100
const REGION_SKY_FLOOR: u32 = 5u;   // 0b101
const REGION_SKY_WALLS: u32 = 6u;   // 0b110
const REGION_ALL: u32 = 7u;         // 0b111

// ============================================================================
// BLEND MODE CONSTANTS (3-bit, 8 modes)
// ============================================================================

const BLEND_ADD: u32 = 0u;
const BLEND_MULTIPLY: u32 = 1u;
const BLEND_MAX: u32 = 2u;
const BLEND_LERP: u32 = 3u;
const BLEND_SCREEN: u32 = 4u;
const BLEND_HSV_MOD: u32 = 5u;
const BLEND_MIN: u32 = 6u;
const BLEND_OVERLAY: u32 = 7u;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn epu_saturate(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }
fn epu_saturate3(v: vec3f) -> vec3f { return clamp(v, vec3f(0.0), vec3f(1.0)); }
fn u8_to_01(x: u32) -> f32 { return f32(x) / 255.0; }
fn u4_to_01(x: u32) -> f32 { return f32(x) / 15.0; }

// Center a periodic coordinate into [-0.5, 0.5) so repeated patterns can be
// shaped without hard phase seams at the tile boundary.
fn epu_periodic_centered(x: f32) -> f32 {
    return fract(x + 0.5) - 0.5;
}

// Distance from a periodic coordinate to its nearest tile edge in [0, 0.5].
fn epu_periodic_edge_distance(x: f32) -> f32 {
    return 0.5 - abs(epu_periodic_centered(x));
}

// Bounded pseudo-organic relief for de-correlating repeated technical edges.
// This stays deterministic and loops cleanly when fed periodic coordinates.
fn epu_relief_wave(p: vec2f, phase: f32) -> f32 {
    let q0 = dot(p, vec2f(2.73, -4.11)) + phase * TAU;
    let q1 = dot(p, vec2f(-5.27, -1.93)) - phase * PI;
    let q2 = dot(p, vec2f(3.17, 6.21)) + phase * TAU * 0.61803398875;
    return (sin(q0) + sin(q1) + sin(q2)) * (1.0 / 3.0);
}

// Phase values used for authored motion should loop over [0, 1) rather than
// duplicating both 0.0 and 1.0. That keeps the final pre-wrap sample adjacent
// to the first sample instead of forcing a duplicated endpoint.
fn epu_loop_phase01(raw: u32) -> f32 {
    return f32(raw & 0xFFu) * (1.0 / 256.0);
}

fn epu_phase_circle(phase01: f32) -> vec2f {
    let theta = fract(phase01) * TAU;
    return vec2f(cos(theta), sin(theta));
}

// Loop-safe hash that embeds phase on the unit circle so seeded offsets remain
// identical at the wrap boundary instead of jumping when phase returns to 0.
fn epu_phase_hash11(seed: f32, phase01: f32) -> f32 {
    let circle = epu_phase_circle(phase01);
    let q = vec3f(seed * 0.1031 + 17.3, circle.x * 0.5 + 0.5, circle.y * 0.5 + 0.5);
    return fract(sin(dot(q, vec3f(127.1, 311.7, 74.7))) * 43758.5453123);
}

fn epu_axis_basis(axis: vec3f) -> mat3x3f {
    let n = normalize(axis);
    let ref_vec = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(n.y) > 0.9);
    let t = normalize(cross(ref_vec, n));
    let b = normalize(cross(n, t));
    return mat3x3f(t, n, b);
}

fn epu_phase_orbit3(axis: vec3f, phase01: f32, radius_t: f32, radius_b: f32) -> vec3f {
    let basis = epu_axis_basis(axis);
    let circle = epu_phase_circle(phase01);
    return basis[0] * (circle.x * radius_t) + basis[2] * (circle.y * radius_b);
}

// Bell-shaped envelope in [0, 1] that suppresses relief near the ends of a
// range, useful for keeping center poles / hard outer caps stable.
fn epu_relief_envelope(x: f32, lo0: f32, lo1: f32, hi0: f32, hi1: f32) -> f32 {
    return smoothstep(lo0, lo1, x) * (1.0 - smoothstep(hi0, hi1, x));
}

// Bend a nominal xyz body frame into a slightly curved local space so layers
// that use x/y/z masks do not collapse back into obvious slab/panel guides.
fn epu_body_curve_coords(p: vec3f, breakup: f32, phase: f32) -> vec3f {
    let amt = breakup * breakup;
    if amt <= 1e-5 {
        return p;
    }

    let bend0 = epu_relief_wave(vec2f(p.y * 0.43, p.z * 0.31), phase + p.x * 0.19);
    let bend1 = epu_relief_wave(vec2f(p.z * 0.37, p.x * 0.29), phase * 0.73 + p.y * 0.23);
    let bend2 = epu_relief_wave(vec2f(p.x * 0.34, p.y * 0.27), phase * 1.21 - p.z * 0.17);
    let bend_amt = mix(0.0, 0.34, amt);

    return vec3f(
        p.x + (p.y * 0.22 + bend0 * 0.58 + bend2 * 0.16) * bend_amt,
        p.y + (p.z * -0.12 + bend1 * 0.44 + bend0 * 0.09) * bend_amt,
        p.z + (p.x * 0.19 + bend1 * 0.31 + bend2 * 0.23) * bend_amt
    );
}

// Build a slightly irregular wrapped lattice position while keeping ordering
// stable. This is useful for curtains/bars that should stay repeated, but not
// lock into ruler-straight rails.
fn epu_staggered_lattice_phase(index: f32, count: f32, phase: f32, amount: f32) -> f32 {
    let inv_count = 1.0 / max(count, 1.0);
    let base = (index + 0.5) * inv_count;
    let phase01 = fract(phase);
    let h0 = epu_phase_hash11(index * 17.13 + count * 1.7, phase01);
    let h1 = epu_phase_hash11(index * 37.31 + count * 0.9, phase01 + 0.37);
    let wave = sin((index + 1.0) * 2.39996323 + phase01 * TAU) * 0.5;
    let offset = ((h0 - 0.5) * 0.72 + (h1 - 0.5) * 0.28 + wave * 0.35) * amount * inv_count;
    return fract(base + offset);
}

// Apply small deterministic relief to a wrapped UV chart. The u channel stays
// periodic, so cylindrical domains keep looping cleanly while avoiding visible
// seam locks and ruler-straight repeats.
fn epu_wrapped_relief_uv(uv: vec2f, phase: f32, u_amount: f32, v_amount: f32) -> vec2f {
    let uc = epu_periodic_centered(uv.x);
    let seam_gate = smoothstep(0.025, 0.12, epu_periodic_edge_distance(uv.x));
    let v_edge = min(uv.y, 1.0 - uv.y);
    let v_gate = smoothstep(0.03, 0.14, v_edge);
    let relief_gate = seam_gate * v_gate;
    let gated_u_amount = u_amount * relief_gate;
    let gated_v_amount = v_amount * relief_gate;
    let p0 = vec2f(uc * 2.3, uv.y * 1.7);
    let p1 = vec2f(uc * -3.1 + uv.y * 0.38, uv.y * 1.29 - uc * 0.62);
    let wave0 = epu_relief_wave(p0, phase);
    let wave1 = epu_relief_wave(p1, phase + 0.37);
    let u = fract(uv.x + wave0 * gated_u_amount + wave1 * gated_u_amount * 0.45);
    let v = uv.y + wave1 * gated_v_amount + wave0 * gated_v_amount * 0.22;
    return vec2f(u, v);
}

// Deterministic 1D hash: f32 -> f32 in [0, 1).
fn epu_hash11(x: f32) -> f32 {
    return fract(sin(x * 127.1) * 43758.5453123);
}

// 3D hash with lower directional correlation than the older sin(dot()) form.
// This is used by shared noise carriers that were exposing faint guide planes
// in direct background views.
fn epu_noise_hash31(p: vec3f) -> f32 {
    var q = fract(p * vec3f(0.1031, 0.1030, 0.0973));
    q += dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
}

// Fixed orthonormal basis for sampling shared value-noise fields away from the
// world axes without changing their character every octave.
fn epu_noise_rotate3(p: vec3f) -> vec3f {
    return vec3f(
        dot(p, vec3f(0.0, 0.8, 0.6)),
        dot(p, vec3f(-0.8, 0.36, -0.48)),
        dot(p, vec3f(-0.6, -0.48, 0.64))
    );
}

fn epu_value_noise3(p: vec3f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = epu_noise_hash31(i + vec3f(0.0, 0.0, 0.0));
    let b = epu_noise_hash31(i + vec3f(1.0, 0.0, 0.0));
    let c = epu_noise_hash31(i + vec3f(0.0, 1.0, 0.0));
    let d = epu_noise_hash31(i + vec3f(1.0, 1.0, 0.0));
    let e = epu_noise_hash31(i + vec3f(0.0, 0.0, 1.0));
    let f1 = epu_noise_hash31(i + vec3f(1.0, 0.0, 1.0));
    let g = epu_noise_hash31(i + vec3f(0.0, 1.0, 1.0));
    let h = epu_noise_hash31(i + vec3f(1.0, 1.0, 1.0));

    let ab = mix(a, b, u.x);
    let cd = mix(c, d, u.x);
    let ef = mix(e, f1, u.x);
    let gh = mix(g, h, u.x);
    let abcd = mix(ab, cd, u.y);
    let efgh = mix(ef, gh, u.y);

    return mix(abcd, efgh, u.z) * 2.0 - 1.0;
}

fn epu_fbm3(p: vec3f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pp = epu_noise_rotate3(p);

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * epu_value_noise3(pp);
        amplitude *= 0.5;
        pp = pp * 2.01 + vec3f(17.3, 31.7, 11.9);
    }

    return value;
}

fn nibble_to_signed_1(v4: u32) -> f32 {
    // 0..15 -> -1..1
    return (f32(v4) / 15.0) * 2.0 - 1.0;
}

// ============================================================================
// OCTAHEDRAL ENCODING
// ============================================================================

// WGSL `sign()` returns 0 for 0 inputs, which breaks octahedral fold math on the
// axes (producing visible "plus" seams). Use a non-zero sign instead.
fn epu_sign_not_zero(v: vec2f) -> vec2f {
    return vec2f(select(-1.0, 1.0, v.x >= 0.0), select(-1.0, 1.0, v.y >= 0.0));
}

// Encode unit direction to octahedral [-1, 1]^2 coordinates.
fn octahedral_encode(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * epu_sign_not_zero(n.xy);
    }
    return n.xy;
}

// Decode octahedral [-1, 1]^2 coordinates to unit direction.
fn octahedral_decode(oct: vec2f) -> vec3f {
    var n = vec3f(oct.xy, 1.0 - abs(oct.x) - abs(oct.y));
    if n.z < 0.0 {
        n = vec3f((1.0 - abs(n.yx)) * epu_sign_not_zero(n.xy), n.z);
    }
    return normalize(n);
}

fn decode_dir16(encoded: u32) -> vec3f {
    let u = f32(encoded & 0xFFu) / 255.0 * 2.0 - 1.0;
    let v = f32((encoded >> 8u) & 0xFFu) / 255.0 * 2.0 - 1.0;
    return octahedral_decode(vec2f(u, v));
}

// ============================================================================
// INSTRUCTION FIELD EXTRACTION (128-bit)
//
// 128-bit instruction stored as vec4<u32>: [w0, w1, w2, w3]
//   w3 = bits 127..96 (hi.hi)
//   w2 = bits  95..64 (hi.lo)
//   w1 = bits  63..32 (lo.hi)
//   w0 = bits  31..0  (lo.lo)
//
// u64 hi [bits 127..64]:
//   bits 127..123: opcode     (5)  - 32 opcodes
//   bits 122..120: region     (3)  - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
//   bits 119..117: blend      (3)  - 8 blend modes
//   bits 116..112: meta5      (5)  - (domain_id<<3)|variant_id
//   bits 111..88:  color_a    (24) - RGB24 primary color
//   bits 87..64:   color_b    (24) - RGB24 secondary color
//
// u64 lo [bits 63..0]:
//   bits 63..56:   intensity  (8)
//   bits 55..48:   param_a    (8)
//   bits 47..40:   param_b    (8)
//   bits 39..32:   param_c    (8)
//   bits 31..24:   param_d    (8)
//   bits 23..8:    direction  (16)
//   bits 7..4:     alpha_a    (4)
//   bits 3..0:     alpha_b    (4)
// ============================================================================

// Extract 5-bit opcode from bits 127..123 (w3 bits 31..27)
fn instr_opcode(instr: vec4u) -> u32 {
    return (instr.w >> 27u) & 0x1Fu;
}

// Extract 3-bit region mask from bits 122..120 (w3 bits 26..24)
fn instr_region(instr: vec4u) -> u32 {
    return (instr.w >> 24u) & 0x7u;
}

// Extract 3-bit blend mode from bits 119..117 (w3 bits 23..21)
fn instr_blend(instr: vec4u) -> u32 {
    return (instr.w >> 21u) & 0x7u;
}

// Extract 5-bit meta5 from bits 116..112 (w3 bits 20..16)
// meta5 encodes domain_id (top 2 bits) and variant_id (bottom 3 bits)
fn instr_meta5(instr: vec4u) -> u32 {
    return (instr.w >> 16u) & 0x1Fu;
}

// Extract 2-bit domain_id from meta5 (top 2 bits)
// Domain IDs: 0=DIRECT3D, 1=AXIS_Y, 2=AXIS_Z, 3=TANGENT_LOCAL
fn instr_domain_id(instr: vec4u) -> u32 {
    return (instr_meta5(instr) >> 3u) & 0x3u;
}

// Extract 3-bit variant_id from meta5 (bottom 3 bits)
// Variant meaning is opcode-specific (e.g., SCATTER: 0=STARS, 1=DUST, etc.)
fn instr_variant_id(instr: vec4u) -> u32 {
    return instr_meta5(instr) & 0x7u;
}

// Extract 24-bit color_a from bits 111..88 (w3 bits 15..0 + w2 bits 31..24)
// color_a spans: w3[15:0] (16 bits) and w2[31:24] (8 bits)
// Actually: bits 111..88 = 24 bits
//   bit 111 = w3 bit 15
//   bit 88  = w2 bit 24
// So: w3[15:0] gives bits 111..96 (16 bits), w2[31:24] gives bits 95..88 (8 bits)
fn instr_color_a_raw(instr: vec4u) -> u32 {
    let hi_part = (instr.w & 0xFFFFu) << 8u;  // bits 111..96 shifted to 23..8
    let lo_part = (instr.z >> 24u) & 0xFFu;   // bits 95..88 as 7..0
    return hi_part | lo_part;
}

// Extract 24-bit color_b from bits 87..64 (w2 bits 23..0)
fn instr_color_b_raw(instr: vec4u) -> u32 {
    return instr.z & 0xFFFFFFu;
}

// Extract 8-bit intensity from bits 63..56 (w1 bits 31..24)
fn instr_intensity(instr: vec4u) -> u32 {
    return (instr.y >> 24u) & 0xFFu;
}

// Extract 8-bit param_a from bits 55..48 (w1 bits 23..16)
fn instr_a(instr: vec4u) -> u32 {
    return (instr.y >> 16u) & 0xFFu;
}

// Extract 8-bit param_b from bits 47..40 (w1 bits 15..8)
fn instr_b(instr: vec4u) -> u32 {
    return (instr.y >> 8u) & 0xFFu;
}

// Extract 8-bit param_c from bits 39..32 (w1 bits 7..0)
fn instr_c(instr: vec4u) -> u32 {
    return instr.y & 0xFFu;
}

// Extract 8-bit param_d from bits 31..24 (w0 bits 31..24)
fn instr_d(instr: vec4u) -> u32 {
    return (instr.x >> 24u) & 0xFFu;
}

// Extract 16-bit direction from bits 23..8 (w0 bits 23..8)
fn instr_dir16(instr: vec4u) -> u32 {
    return (instr.x >> 8u) & 0xFFFFu;
}

// Extract 4-bit alpha_a from bits 7..4 (w0 bits 7..4)
fn instr_alpha_a(instr: vec4u) -> u32 {
    return (instr.x >> 4u) & 0xFu;
}

// Extract 4-bit alpha_b from bits 3..0 (w0 bits 3..0)
fn instr_alpha_b(instr: vec4u) -> u32 {
    return instr.x & 0xFu;
}

// ============================================================================
// RGB24 COLOR EXTRACTION HELPERS
// ============================================================================

// Extract RGB24 as vec3<f32> in 0-1 range
// RGB24 layout: R[23:16], G[15:8], B[7:0]
fn rgb24_to_vec3f(rgb24: u32) -> vec3f {
    let r = f32((rgb24 >> 16u) & 0xFFu) / 255.0;
    let g = f32((rgb24 >> 8u) & 0xFFu) / 255.0;
    let b = f32(rgb24 & 0xFFu) / 255.0;
    return vec3f(r, g, b);
}

// Get color_a as vec3f (0-1 range)
fn instr_color_a(instr: vec4u) -> vec3f {
    return rgb24_to_vec3f(instr_color_a_raw(instr));
}

// Get color_b as vec3f (0-1 range)
fn instr_color_b(instr: vec4u) -> vec3f {
    return rgb24_to_vec3f(instr_color_b_raw(instr));
}

// Get alpha_a as f32 (0-1 range)
fn instr_alpha_a_f32(instr: vec4u) -> f32 {
    return u4_to_01(instr_alpha_a(instr));
}

// Get alpha_b as f32 (0-1 range)
fn instr_alpha_b_f32(instr: vec4u) -> f32 {
    return u4_to_01(instr_alpha_b(instr));
}

// ============================================================================
// REGION WEIGHTS AND BOUNDS DIRECTION
// ============================================================================

struct RegionWeights {
    sky: f32,
    wall: f32,
    floor: f32,
}

fn normalize_region_weights(weights: RegionWeights) -> RegionWeights {
    let total = max(weights.sky + weights.wall + weights.floor, 0.0001);
    return RegionWeights(weights.sky / total, weights.wall / total, weights.floor / total);
}

fn sharpen_region_weights(weights: RegionWeights, exponent: f32) -> RegionWeights {
    return normalize_region_weights(RegionWeights(
        pow(max(weights.sky, 0.0), exponent),
        pow(max(weights.wall, 0.0), exponent),
        pow(max(weights.floor, 0.0), exponent)
    ));
}

// Compose sequential bounds passes without letting a later organizer erase the
// floor/sky ownership established by an earlier one. This keeps multi-bounds
// authoring usable for outdoor scenes where one layer sets a horizon contract
// and another adds secondary structure.
fn compose_bounds_regions(base: RegionWeights, next: RegionWeights, amount: f32) -> RegionWeights {
    let preserved = RegionWeights(
        max(base.sky, next.sky),
        max(base.wall, next.wall),
        max(base.floor, next.floor)
    );
    let composed = sharpen_region_weights(normalize_region_weights(preserved), 2.25);
    let t = epu_saturate(amount);
    return normalize_region_weights(RegionWeights(
        mix(base.sky, composed.sky, t),
        mix(base.wall, composed.wall, t),
        mix(base.floor, composed.floor, t)
    ));
}

// Extract bounds direction from a layer's instruction.
// Bounds layers that define a direction will update bounds_dir for subsequent features.
fn bounds_dir_from_layer(instr: vec4u, opcode: u32, prev_dir: vec3f) -> vec3f {
    switch opcode {
        case OP_RAMP, OP_SECTOR, OP_SILHOUETTE, OP_SPLIT, OP_CELL, OP_PATCHES, OP_APERTURE: {
            return decode_dir16(instr_dir16(instr));
        }
        default: { return prev_dir; }
    }
}

// Compute region weight from bitfield mask
// mask is a 3-bit bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
fn region_weight(weights: RegionWeights, mask: u32) -> f32 {
    var w = 0.0;
    var bits = 0u;
    if (mask & REGION_SKY) != 0u {
        w += weights.sky;
        bits += 1u;
    }
    if (mask & REGION_WALLS) != 0u {
        w += weights.wall;
        bits += 1u;
    }
    if (mask & REGION_FLOOR) != 0u {
        w += weights.floor;
        bits += 1u;
    }

    // Dedicated single-region features should remain readable even when bounds
    // composition softens ownership nearby. Multi-region masks already have
    // enough coverage, so only boost the focused one-region case.
    if bits == 1u {
        return pow(epu_saturate(w), 0.72);
    }
    return epu_saturate(w);
}

// Convert signed distance to region weights.
// d: signed distance (negative = inside/sky, positive = outside/floor)
// bw: band width for smooth transitions
fn regions_from_signed_distance(d: f32, bw: f32) -> RegionWeights {
    let w_sky = smoothstep(0.0, bw, -d);
    let w_floor = smoothstep(0.0, bw, d);
    let w_wall = max(0.0, 1.0 - w_sky - w_floor);
    return RegionWeights(w_sky, w_wall, w_floor);
}

// ============================================================================
// LAYER SAMPLE AND BLEND
// ============================================================================

struct LayerSample {
    rgb: vec3f,
    w: f32,
}

// Result from bounds opcodes - includes rendering sample and output regions for subsequent features
struct BoundsResult {
    sample: LayerSample,
    regions: RegionWeights,  // The regions to use for all subsequent features
    region_mix: f32,         // How strongly the new regions retag subsequent features
}

fn apply_blend(dst: vec3f, s: LayerSample, blend: u32) -> vec3f {
    let src = s.rgb;
    let a = epu_saturate(s.w);
    switch blend {
        case BLEND_ADD: {
            // Clamp to prevent washed-out colors from additive accumulation
            return epu_saturate3(dst + src * a);
        }
        case BLEND_MULTIPLY: {
            // Absorption/tint: lerp towards multiplying by src.
            return epu_saturate3(dst * mix(vec3f(1.0), src, a));
        }
        case BLEND_MAX: {
            return epu_saturate3(max(dst, src * a));
        }
        case BLEND_LERP: {
            // Lerp directly to src (not premultiplied).
            return epu_saturate3(mix(dst, src, a));
        }
        case BLEND_SCREEN: {
            // Screen blend: 1 - (1-dst)*(1-src*a)
            // Clamp to handle edge cases with extreme values
            return epu_saturate3(vec3f(1.0) - (vec3f(1.0) - dst) * (vec3f(1.0) - src * a));
        }
        case BLEND_HSV_MOD: {
            // HSV modulation placeholder - shifts hue/sat/val by src
            // For now, approximate with additive hue shift via color rotation
            // Full HSV would require rgb<->hsv conversion
            let shifted = dst + (src - vec3f(0.5)) * a * 2.0;
            return epu_saturate3(shifted);
        }
        case BLEND_MIN: {
            return epu_saturate3(min(dst, mix(vec3f(1.0), src, a)));
        }
        case BLEND_OVERLAY: {
            // Overlay: 2*dst*src if dst<0.5, else 1-2*(1-dst)*(1-src)
            let lo = 2.0 * dst * src;
            let hi = vec3f(1.0) - 2.0 * (vec3f(1.0) - dst) * (vec3f(1.0) - src);
            let overlay = select(hi, lo, dst < vec3f(0.5));
            // Clamp to handle edge cases
            return epu_saturate3(mix(dst, overlay, a));
        }
        default: {
            // Clamp default additive blend
            return epu_saturate3(dst + src * a);
        }
    }
}
