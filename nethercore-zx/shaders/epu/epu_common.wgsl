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

// Radiance opcodes (0x0C-0x13)
const OP_TRACE: u32 = 0x0Cu;        // Procedural line/crack patterns (lightning, cracks, lead lines, filaments)
const OP_VEIL: u32 = 0x0Du;         // Curtain/ribbon effects (curtains, pillars, laser bars, rain wall, shards)
const OP_ATMOSPHERE: u32 = 0x0Eu;   // Atmospheric absorption + scattering
const OP_PLANE: u32 = 0x0Fu;        // Ground plane textures
const OP_CELESTIAL: u32 = 0x10u;    // Moon/sun/planet bodies
const OP_PORTAL: u32 = 0x11u;       // Swirling vortex/portal effect
const OP_LOBE_RADIANCE: u32 = 0x12u; // Region-masked directional glow
const OP_BAND_RADIANCE: u32 = 0x13u; // Region-masked horizon band
// 0x14..0x1F reserved for future radiance ops

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

// Deterministic 1D hash: f32 -> f32 in [0, 1).
fn epu_hash11(x: f32) -> f32 {
    return fract(sin(x * 127.1) * 43758.5453123);
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
    if (mask & REGION_SKY) != 0u {
        w += weights.sky;
    }
    if (mask & REGION_WALLS) != 0u {
        w += weights.wall;
    }
    if (mask & REGION_FLOOR) != 0u {
        w += weights.floor;
    }
    return w;
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
            return dst * mix(vec3f(1.0), src, a);
        }
        case BLEND_MAX: {
            return max(dst, src * a);
        }
        case BLEND_LERP: {
            // Lerp directly to src (not premultiplied).
            return mix(dst, src, a);
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
            return min(dst, mix(vec3f(1.0), src, a));
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
