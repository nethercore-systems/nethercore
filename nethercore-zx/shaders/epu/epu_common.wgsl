// ============================================================================
// EPU COMMON TYPES, DECODING, AND HELPERS
// Environment Processing Unit - shared definitions
// ============================================================================

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;

const OP_NOP: u32 = 0x0u;
const OP_RAMP: u32 = 0x1u;
const OP_LOBE: u32 = 0x2u;
const OP_BAND: u32 = 0x3u;
const OP_FOG:  u32 = 0x4u;
const OP_DECAL:   u32 = 0x5u;
const OP_GRID:    u32 = 0x6u;
const OP_SCATTER: u32 = 0x7u;
const OP_FLOW:    u32 = 0x8u;

const REGION_ALL:   u32 = 0u;
const REGION_SKY:   u32 = 1u;
const REGION_WALLS: u32 = 2u;
const REGION_FLOOR: u32 = 3u;

const BLEND_ADD:      u32 = 0u;
const BLEND_MULTIPLY: u32 = 1u;
const BLEND_MAX:      u32 = 2u;
const BLEND_LERP:     u32 = 3u;

fn saturate(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }
fn u8_to_01(x: u32) -> f32 { return f32(x) / 255.0; }

fn nibble_to_signed_1(v4: u32) -> f32 {
    // 0..15 -> -1..1
    return (f32(v4) / 15.0) * 2.0 - 1.0;
}

// ============================================================================
// OCTAHEDRAL ENCODING
// ============================================================================

// Encode unit direction to octahedral [-1, 1]^2 coordinates.
fn octahedral_encode(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign(n.xy);
    }
    return n.xy;
}

// Decode octahedral [-1, 1]^2 coordinates to unit direction.
fn octahedral_decode(oct: vec2f) -> vec3f {
    var n = vec3f(oct.xy, 1.0 - abs(oct.x) - abs(oct.y));
    if n.z < 0.0 {
        n = vec3f((1.0 - abs(n.yx)) * sign(n.xy), n.z);
    }
    return normalize(n);
}

fn decode_dir16(encoded: u32) -> vec3f {
    let u = f32(encoded & 0xFFu) / 255.0 * 2.0 - 1.0;
    let v = f32((encoded >> 8u) & 0xFFu) / 255.0 * 2.0 - 1.0;
    return octahedral_decode(vec2f(u, v));
}

// ============================================================================
// INSTRUCTION FIELD EXTRACTION
// Packed 64-bit instruction layout (stored as vec2u: lo, hi):
//   63..60  opcode        (4)
//   59..58  region_mask   (2)
//   57..56  blend_mode    (2)
//   55..48  color_index   (8)
//   47..40  intensity     (8)
//   39..32  param_a       (8)
//   31..24  param_b       (8)
//   23..16  param_c       (8)
//   15..0   direction     (16)
// ============================================================================

// Extract fields from packed instruction (lo = bits 0..31, hi = bits 32..63)
fn instr_opcode(lo: u32, hi: u32) -> u32 { return (hi >> 28u) & 0xFu; }
fn instr_region(lo: u32, hi: u32) -> u32 { return (hi >> 26u) & 0x3u; }
fn instr_blend(lo: u32, hi: u32) -> u32 { return (hi >> 24u) & 0x3u; }
fn instr_color(lo: u32, hi: u32) -> u32 { return (hi >> 16u) & 0xFFu; }
fn instr_intensity(lo: u32, hi: u32) -> u32 { return (hi >> 8u) & 0xFFu; }
fn instr_a(lo: u32, hi: u32) -> u32 { return hi & 0xFFu; }
fn instr_b(lo: u32, hi: u32) -> u32 { return (lo >> 24u) & 0xFFu; }
fn instr_c(lo: u32, hi: u32) -> u32 { return (lo >> 16u) & 0xFFu; }
fn instr_dir16(lo: u32, hi: u32) -> u32 { return lo & 0xFFFFu; }

// ============================================================================
// REGION WEIGHTS AND ENCLOSURE
// ============================================================================

struct RegionWeights {
    sky: f32,
    wall: f32,
    floor: f32,
}

struct EnclosureConfig {
    up: vec3f,
    ceil_y: f32,
    floor_y: f32,
    soft: f32,
}

fn enclosure_from_ramp(lo: u32, hi: u32) -> EnclosureConfig {
    let up = decode_dir16(instr_dir16(lo, hi));

    let pc = instr_c(lo, hi);
    let ceil_q = (pc >> 4u) & 0xFu;
    let floor_q = pc & 0xFu;

    // Soften to a small minimum to avoid hard banding.
    let soft = mix(0.01, 0.5, u8_to_01(instr_intensity(lo, hi)));

    var ceil_y = nibble_to_signed_1(ceil_q);
    var floor_y = nibble_to_signed_1(floor_q);
    if floor_y > ceil_y {
        // Ensure a valid ordering; swap if authored incorrectly.
        let t = floor_y;
        floor_y = ceil_y;
        ceil_y = t;
    }

    return EnclosureConfig(up, ceil_y, floor_y, soft);
}

fn compute_region_weights(dir: vec3f, enc: EnclosureConfig) -> RegionWeights {
    let y = dot(dir, enc.up);

    let w_sky = smoothstep(enc.ceil_y - enc.soft, enc.ceil_y + enc.soft, y);
    let w_floor = smoothstep(enc.floor_y + enc.soft, enc.floor_y - enc.soft, y);
    let w_wall = 1.0 - w_sky - w_floor;

    return RegionWeights(w_sky, w_wall, w_floor);
}

fn region_weight(weights: RegionWeights, mask: u32) -> f32 {
    switch mask {
        case REGION_ALL:   { return 1.0; }
        case REGION_SKY:   { return weights.sky; }
        case REGION_WALLS: { return weights.wall; }
        case REGION_FLOOR: { return weights.floor; }
        default:           { return 1.0; }
    }
}

// ============================================================================
// LAYER SAMPLE AND BLEND
// ============================================================================

struct LayerSample {
    rgb: vec3f,
    w: f32,
}

fn apply_blend(dst: vec3f, s: LayerSample, blend: u32) -> vec3f {
    let src = s.rgb;
    let a = saturate(s.w);
    switch blend {
        case BLEND_ADD: {
            return dst + src * a;
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
        default: {
            return dst + src * a;
        }
    }
}

// ============================================================================
// PALETTE LOOKUP
// Palette: storage buffer of 256 RGB values in linear space.
// ============================================================================

fn palette_lookup(palette: ptr<storage, array<vec4f>, read>, idx: u32) -> vec3f {
    return (*palette)[idx & 255u].rgb;
}
