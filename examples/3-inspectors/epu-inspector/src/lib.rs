//! EPU Inspector - Environment Processing Unit Demo
//!
//! Demonstrates the instruction-based EPU system for procedural environment backgrounds.
//! The EPU uses 128-byte configurations (8 x 128-bit layers) to define complex environments
//! with minimal memory and deterministic rendering.
//!
//! # v2 Format (128-bit instructions)
//!
//! Each layer is 128 bits (2 x u64) with the following layout:
//! - hi word: opcode(5), region(3), blend(3), reserved4(4), reserved(1), color_a(24), color_b(24)
//! - lo word: intensity(8), param_a(8), param_b(8), param_c(8), param_d(8), direction(16), alpha_a(4), alpha_b(4)
//!
//! Features:
//! - Multiple preset environments (void+stars, sunny meadow, cyberpunk alley, etc.)
//! - Keyboard/gamepad cycling through presets
//! - Real-time environment background rendering via epu_set() and epu_draw()
//!
//! Controls:
//! - A button: Cycle to next preset
//! - B button: Cycle to previous preset
//! - Left stick: Rotate camera around scene
//!
//! Press F4 to open the debug inspector.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Declarations
// ============================================================================

#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

// =============================================================================
// EPU Preset Configurations (v2 128-bit format)
// =============================================================================
//
// Each 128-bit layer is stored as [hi, lo] u64 pair:
//
// u64 hi [bits 127..64]:
//   bits 63..59: opcode     (5)   - NOP=0, RAMP=1, LOBE=2, BAND=3, FOG=4, DECAL=5, GRID=6, SCATTER=7, FLOW=8
//   bits 58..56: region     (3)   - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001, ALL=0b111
//   bits 55..53: blend      (3)   - ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
//   bits 52..49: reserved   (4)
//   bit  48:     reserved   (1)
//   bits 47..24: color_a    (24)  - RGB24 primary color
//   bits 23..0:  color_b    (24)  - RGB24 secondary color
//
// u64 lo [bits 63..0]:
//   bits 63..56: intensity  (8)
//   bits 55..48: param_a    (8)
//   bits 47..40: param_b    (8)
//   bits 39..32: param_c    (8)
//   bits 31..24: param_d    (8)
//   bits 23..8:  direction  (16)  - Octahedral encoded
//   bits 7..4:   alpha_a    (4)   - color_a alpha (0-15)
//   bits 3..0:   alpha_b    (4)   - color_b alpha (0-15)
//
// Slots 0-3: Bounds layers (RAMP, LOBE, BAND, FOG)
// Slots 4-7: Feature layers (DECAL, GRID, SCATTER, FLOW)

// Helper to build v2 hi word
const fn hi(opcode: u64, region: u64, blend: u64, _reserved4: u64, color_a: u64, color_b: u64) -> u64 {
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        // bits 52..49 reserved
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

// Helper to build v2 lo word
const fn lo(intensity: u64, param_a: u64, param_b: u64, param_c: u64, param_d: u64, direction: u64, alpha_a: u64, alpha_b: u64) -> u64 {
    ((intensity & 0xFF) << 56)
        | ((param_a & 0xFF) << 48)
        | ((param_b & 0xFF) << 40)
        | ((param_c & 0xFF) << 32)
        | ((param_d & 0xFF) << 24)
        | ((direction & 0xFFFF) << 8)
        | ((alpha_a & 0xF) << 4)
        | (alpha_b & 0xF)
}

// Opcodes (v2 spec: 0x00=NOP, 0x01=RAMP, 0x02=LOBE, 0x03=BAND, 0x04=FOG, 0x05=DECAL, 0x06=GRID, 0x07=SCATTER, 0x08=FLOW)
const OP_RAMP: u64 = 0x01;
const OP_LOBE: u64 = 0x02;
const OP_BAND: u64 = 0x03;
const OP_FOG: u64 = 0x04;
const OP_DECAL: u64 = 0x05;
const OP_GRID: u64 = 0x06;
const OP_SCATTER: u64 = 0x07;
const OP_FLOW: u64 = 0x08;

// Regions
const REGION_ALL: u64 = 0b111;
const REGION_SKY: u64 = 0b100;
const REGION_WALLS: u64 = 0b010;
const REGION_FLOOR: u64 = 0b001;

// Blends (ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7)
const BLEND_ADD: u64 = 0;
const BLEND_MULTIPLY: u64 = 1;
const BLEND_MAX: u64 = 2;
const BLEND_LERP: u64 = 3;
const BLEND_SCREEN: u64 = 4;
const BLEND_HSV_MOD: u64 = 5;
const BLEND_MIN: u64 = 6;
const BLEND_OVERLAY: u64 = 7;

// Direction for +Y (up) in octahedral encoding: u=128, v=255
const DIR_UP: u64 = 0x80FF;
// Direction for sun (0.5, 0.7, 0.3 normalized): approximately
const DIR_SUN: u64 = 0xC0A0;

/// NOP layer (disabled)
const NOP_LAYER: [u64; 2] = [0, 0];

// =============================================================================
// 20 Environment Presets (v2 128-bit format)
// =============================================================================

/// 1. Sunny Meadow - Bright blue sky, green grass, warm sun
const PRESET_SUNNY_MEADOW: [[u64; 2]; 8] = [
    // RAMP: sky=blue, floor=green
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x6496DC, 0x508C50), lo(180, 200, 180, 150, 0xA5, DIR_UP, 15, 15)],
    // LOBE: warm sun glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 15, 0xFFF0C8, 0xFFE8B0), lo(180, 32, 0, 0, 0, DIR_SUN, 15, 12)],
    NOP_LAYER,
    NOP_LAYER,
    // DECAL: sun disk
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFDCB4), lo(255, 0x02, 12, 0, 0, DIR_SUN, 15, 15)],
    // FLOW: clouds (LERP)
    [hi(OP_FLOW, REGION_SKY, BLEND_LERP, 0, 0xFFFFFF, 0xE0E8F0), lo(60, 32, 20, 0x20, 0, DIR_UP, 15, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 2. Cyberpunk Alley - Neon-lit urban with fog and rain
const PRESET_CYBERPUNK_ALLEY: [[u64; 2]; 8] = [
    // RAMP: dark walls
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x101020, 0x080810), lo(100, 20, 20, 30, 0xA5, DIR_UP, 15, 15)],
    // LOBE: magenta glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFF00FF, 0xFF44AA), lo(140, 24, 0, 0, 0, 0x30C0, 15, 10)],
    // LOBE: cyan glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0x00FFFF, 0x00AAFF), lo(120, 28, 0, 0, 0, 0xD060, 15, 10)],
    // FOG: purple haze
    [hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x402040, 0x201030), lo(80, 140, 100, 0, 0, DIR_UP, 12, 8)],
    // GRID: walls, cyan accent
    [hi(OP_GRID, REGION_WALLS, BLEND_ADD, 8, 0x00AAAA, 0x006666), lo(80, 48, 12, 0x00, 0, 0, 15, 10)],
    // DECAL: pink sign
    [hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 10, 0xFF88AA, 0xFF4488), lo(200, 0x04, 40, 30, 0, 0x80C0, 15, 12)],
    // FLOW: rain
    [hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x808080, 0x404050), lo(40, 64, 180, 0x11, 0, 0x6980, 10, 6)],
    // SCATTER: warm windows
    [hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 6, 0xFFAA44, 0xFF8822), lo(180, 120, 35, 0x23, 0, 0, 15, 12)],
];

/// 3. Void Stars - Black void with twinkling stars
const PRESET_VOID_STARS: [[u64; 2]; 8] = [
    // RAMP: black everywhere
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000005, 0x000008), lo(10, 0, 0, 0, 0xF0, DIR_UP, 15, 15)],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    // SCATTER: white stars
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 15, 0xFFFFFF, 0xAABBFF), lo(255, 200, 20, 0x83, 0, 0, 15, 10)],
    // SCATTER: blue distant stars
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x8888FF, 0x4444AA), lo(180, 300, 8, 0x41, 0, 0, 12, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 4. Sunset Desert - Orange/red desert at dusk
const PRESET_SUNSET_DESERT: [[u64; 2]; 8] = [
    // RAMP: orange sky, tan sand
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xFF6030, 0xC09060), lo(200, 220, 180, 140, 0xA3, DIR_UP, 15, 15)],
    // LOBE: setting sun glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 14, 0xFFAA40, 0xFF6600), lo(240, 28, 0, 0, 0, 0xE040, 15, 12)],
    // FOG: dust haze
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x804020, 0x603010), lo(60, 100, 80, 0, 0, DIR_UP, 10, 8)],
    NOP_LAYER,
    // DECAL: sun disk near horizon
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFF4400, 0xFF2200), lo(255, 0x04, 22, 0, 0, 0xE040, 15, 15)],
    // FLOW: blowing sand
    [hi(OP_FLOW, REGION_FLOOR, BLEND_LERP, 0, 0xC09060, 0x906040), lo(50, 32, 24, 0x10, 0, 0x4080, 12, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 5. Underwater Cave - Deep blue caustics and bubbles
const PRESET_UNDERWATER_CAVE: [[u64; 2]; 8] = [
    // RAMP: dark teal walls
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x004060, 0x002030), lo(100, 30, 50, 40, 0xBB, DIR_UP, 15, 15)],
    // LOBE: blue-green from above
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0x40A0A0, 0x206060), lo(100, 16, 0, 0, 0, DIR_UP, 15, 10)],
    // FOG: deep blue
    [hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x203050, 0x102030), lo(120, 80, 100, 0, 0, DIR_UP, 12, 8)],
    NOP_LAYER,
    // FLOW: caustics
    [hi(OP_FLOW, REGION_ALL, BLEND_ADD, 4, 0x40C0C0, 0x208080), lo(80, 40, 20, 0x22, 0, 0x9870, 15, 10)],
    // SCATTER: bubbles
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFFFFFF, 0xAADDFF), lo(140, 80, 12, 0x60, 0, 0, 15, 12)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 6. Storm Front - Dark skies, lightning, rain
const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // RAMP: dark gray storm clouds
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x202830, 0x303840), lo(80, 40, 50, 60, 0xC8, DIR_UP, 15, 15)],
    // LOBE: dim ambient from sky
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 4, 0x405060, 0x304050), lo(60, 24, 0, 0, 0, DIR_UP, 12, 10)],
    // BAND: dark cloud band
    [hi(OP_BAND, REGION_SKY, BLEND_MULTIPLY, 0, 0x101820, 0x081018), lo(120, 60, 140, 0, 0, DIR_UP, 15, 12)],
    // FOG: mist
    [hi(OP_FOG, REGION_ALL, BLEND_LERP, 0, 0x404850, 0x303840), lo(100, 120, 80, 0, 0, DIR_UP, 10, 8)],
    // SCATTER: lightning flashes
    [hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xCCDDFF), lo(255, 10, 80, 0xF2, 0, 0, 15, 12)],
    // FLOW: rain
    [hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x606880, 0x404860), lo(60, 80, 200, 0x11, 0, 0x7F80, 12, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 7. Neon City - Futuristic city at night
const PRESET_NEON_CITY: [[u64; 2]; 8] = [
    // RAMP: dark blue night
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x080818, 0x040410), lo(60, 10, 15, 20, 0xA8, DIR_UP, 15, 15)],
    // LOBE: orange street glow from below
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0xFF8844, 0xCC6622), lo(100, 20, 0, 0, 0, 0x8000, 15, 12)],
    // BAND: sky glow pollution
    [hi(OP_BAND, REGION_SKY, BLEND_SCREEN, 2, 0x442244, 0x221122), lo(40, 80, 180, 0, 0, DIR_UP, 10, 8)],
    NOP_LAYER,
    // GRID: building windows
    [hi(OP_GRID, REGION_WALLS, BLEND_ADD, 6, 0xFFCC88, 0xFF8844), lo(120, 32, 20, 0x00, 0, 0, 15, 12)],
    // SCATTER: neon signs
    [hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 10, 0xFF00AA, 0x00FFAA), lo(200, 40, 50, 0x34, 0, 0, 15, 15)],
    // DECAL: billboard
    [hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 12, 0x00FFFF, 0xFF00FF), lo(220, 0x04, 60, 40, 0, 0x90B0, 15, 15)],
    NOP_LAYER,
];

/// 8. Enchanted Forest - Magical woodland with glowing elements
const PRESET_ENCHANTED_FOREST: [[u64; 2]; 8] = [
    // RAMP: deep green-blue
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x102818, 0x081810), lo(80, 40, 60, 50, 0xD4, DIR_UP, 15, 15)],
    // LOBE: moonlight filtering through
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 6, 0xA0C0D0, 0x608090), lo(70, 14, 0, 0, 0, 0x60E0, 15, 10)],
    // FOG: mystical mist
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x204030, 0x102820), lo(80, 100, 80, 0, 0, DIR_UP, 10, 8)],
    NOP_LAYER,
    // SCATTER: fireflies/magical particles
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x80FF60, 0x40FF80), lo(220, 60, 10, 0xC9, 0, 0, 15, 12)],
    // SCATTER: glowing mushrooms
    [hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 8, 0x6060FF, 0x4040AA), lo(160, 30, 15, 0x21, 0, 0, 15, 10)],
    // FLOW: floating pollen
    [hi(OP_FLOW, REGION_ALL, BLEND_ADD, 2, 0xFFFFAA, 0xAAAA66), lo(40, 20, 10, 0x30, 0, 0x6080, 10, 6)],
    NOP_LAYER,
];

/// 9. Arctic Tundra - Cold, snowy, white/blue palette
const PRESET_ARCTIC_TUNDRA: [[u64; 2]; 8] = [
    // RAMP: white snow, pale blue sky
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xC0D0E0, 0xF0F0F0), lo(180, 240, 240, 255, 0xBE, DIR_UP, 15, 15)],
    // LOBE: cold sun
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0xC0D0E0, 0xA0B0C0), lo(100, 16, 0, 0, 0, 0xC0A0, 15, 12)],
    // FOG: white haze
    [hi(OP_FOG, REGION_ALL, BLEND_MIN, 0, 0xF0F0FF, 0xE0E0F0), lo(80, 80, 100, 0, 0, DIR_UP, 12, 10)],
    NOP_LAYER,
    // SCATTER: snowflakes
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 4, 0xFFFFFF, 0xDDEEFF), lo(200, 100, 72, 0x48, 0, 0, 15, 12)],
    // FLOW: blowing snow
    [hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0xFFFFFF, 0xCCDDEE), lo(40, 50, 30, 0x11, 0, 0x40A0, 10, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 10. Neon Arcade - Retro gaming, colorful neon on black
const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // RAMP: black everywhere
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000010, 0x000008), lo(50, 0, 0, 0, 0x00, DIR_UP, 15, 15)],
    // BAND: magenta stripes
    [hi(OP_BAND, REGION_ALL, BLEND_ADD, 10, 0xFF00FF, 0xAA00AA), lo(200, 48, 96, 0, 0, DIR_UP, 15, 12)],
    // BAND: cyan stripes offset
    [hi(OP_BAND, REGION_ALL, BLEND_ADD, 8, 0x00FFFF, 0x00AAAA), lo(160, 48, 144, 0, 0, DIR_UP, 15, 12)],
    NOP_LAYER,
    // GRID: cyan floor tiles
    [hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 8, 0x00FFFF, 0x008888), lo(160, 64, 16, 0x00, 0, 0, 15, 12)],
    // SCATTER: magenta particles
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 8, 0xFF00FF, 0xFF44FF), lo(220, 64, 60, 0x3C, 0, 0, 15, 12)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 11. Desert Dunes - Hot, arid, golden/orange midday
const PRESET_DESERT_DUNES: [[u64; 2]; 8] = [
    // RAMP: golden sand floor, bright sky
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xE0C080, 0xD0A050), lo(160, 220, 200, 180, 0xB2, DIR_UP, 15, 15)],
    // LOBE: blazing sun
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 15, 0xFFFFE0, 0xFFF0C0), lo(255, 32, 0, 0, 0, 0x8090, 15, 12)],
    // FOG: golden haze
    [hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0xE0C080, 0xC0A060), lo(100, 80, 100, 0, 0, DIR_UP, 12, 10)],
    NOP_LAYER,
    // DECAL: sun disk
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFF8E0), lo(255, 0x02, 14, 0, 0, 0x8090, 15, 15)],
    // FLOW: sand drift
    [hi(OP_FLOW, REGION_FLOOR, BLEND_LERP, 0, 0xD0A050, 0xB08040), lo(80, 24, 16, 0x10, 0, DIR_UP, 12, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 12. Alien Planet - Strange colors, foreign atmosphere
const PRESET_ALIEN_PLANET: [[u64; 2]; 8] = [
    // RAMP: purple sky, teal ground
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x602080, 0x208060), lo(140, 160, 140, 120, 0xB6, DIR_UP, 15, 15)],
    // LOBE: twin suns (orange and blue)
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFF8800, 0x0088FF), lo(180, 20, 0, 0, 0, 0xA0C0, 15, 12)],
    // FOG: alien atmosphere
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x402060, 0x204040), lo(80, 100, 80, 0, 0, DIR_UP, 10, 8)],
    NOP_LAYER,
    // SCATTER: floating spores
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFF44FF, 0x44FFFF), lo(160, 80, 20, 0x55, 0, 0, 15, 12)],
    // DECAL: alien sun
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 14, 0xFF6600, 0x0066FF), lo(240, 0x04, 20, 0, 0, 0xA0C0, 15, 15)],
    // FLOW: gas currents
    [hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x804080, 0x408040), lo(50, 40, 25, 0x22, 0, 0x5090, 10, 8)],
    NOP_LAYER,
];

/// 13. Stormy Ocean - Dark seas, crashing waves
const PRESET_STORMY_OCEAN: [[u64; 2]; 8] = [
    // RAMP: dark gray-blue
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x304050, 0x203040), lo(100, 50, 60, 80, 0xC6, DIR_UP, 15, 15)],
    // LOBE: dim overcast light
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 4, 0x506070, 0x405060), lo(60, 20, 0, 0, 0, DIR_UP, 12, 10)],
    // BAND: horizon line
    [hi(OP_BAND, REGION_ALL, BLEND_LERP, 0, 0x405060, 0x304050), lo(80, 40, 128, 0, 0, DIR_UP, 12, 10)],
    // FOG: sea spray
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x405060, 0x304050), lo(100, 120, 80, 0, 0, DIR_UP, 10, 8)],
    // FLOW: waves
    [hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 4, 0x6080A0, 0x406080), lo(100, 60, 40, 0x21, 0, 0x80A0, 15, 10)],
    // SCATTER: foam
    [hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 2, 0xFFFFFF, 0xCCDDEE), lo(180, 60, 30, 0x32, 0, 0, 12, 10)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 14. Crystal Cavern - Glowing crystals, refractions
const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // RAMP: dark purple rock
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x201030, 0x100820), lo(80, 30, 40, 50, 0xD5, DIR_UP, 15, 15)],
    // LOBE: crystal glow from below
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0x8040FF, 0x4020AA), lo(140, 18, 0, 0, 0, 0x8010, 15, 12)],
    // FOG: faint mist
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x302040, 0x201030), lo(60, 80, 100, 0, 0, DIR_UP, 10, 8)],
    NOP_LAYER,
    // SCATTER: glowing crystals (cyan)
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x00FFFF, 0x0088AA), lo(220, 40, 30, 0x24, 0, 0, 15, 12)],
    // SCATTER: glowing crystals (magenta)
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0xFF00FF, 0xAA0088), lo(200, 30, 25, 0x23, 0, 0, 15, 12)],
    // GRID: crystal facets on walls
    [hi(OP_GRID, REGION_WALLS, BLEND_ADD, 6, 0x6040A0, 0x402080), lo(80, 24, 12, 0x00, 0, 0, 12, 10)],
    NOP_LAYER,
];

/// 15. Toxic Wasteland - Green/yellow toxic, industrial decay
const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // RAMP: sickly yellow-green
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x404020, 0x606030), lo(100, 60, 80, 70, 0xC4, DIR_UP, 15, 15)],
    // LOBE: toxic glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0x80FF00, 0x40AA00), lo(120, 16, 0, 0, 0, 0x9060, 15, 12)],
    // FOG: toxic haze
    [hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x605020, 0x403010), lo(140, 100, 80, 0, 0, DIR_UP, 12, 10)],
    NOP_LAYER,
    // SCATTER: toxic bubbles
    [hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 10, 0xAAFF00, 0x66AA00), lo(200, 50, 20, 0x62, 0, 0, 15, 12)],
    // FLOW: toxic sludge
    [hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 6, 0x80C000, 0x408000), lo(120, 30, 20, 0x10, 0, DIR_UP, 12, 10)],
    // GRID: industrial grating
    [hi(OP_GRID, REGION_WALLS, BLEND_ADD, 2, 0x404040, 0x303030), lo(40, 32, 8, 0x00, 0, 0, 10, 8)],
    NOP_LAYER,
];

/// 16. Cherry Blossom - Pink petals, soft spring day
const PRESET_CHERRY_BLOSSOM: [[u64; 2]; 8] = [
    // RAMP: soft blue sky, green grass
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xA0C0E0, 0x60A060), lo(180, 200, 180, 160, 0xA6, DIR_UP, 15, 15)],
    // LOBE: warm spring sun
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFFF0E0, 0xFFE0C0), lo(160, 24, 0, 0, 0, DIR_SUN, 15, 12)],
    // FOG: soft haze
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0xFFE0F0, 0xFFC0E0), lo(40, 100, 80, 0, 0, DIR_UP, 8, 6)],
    NOP_LAYER,
    // SCATTER: pink petals
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 4, 0xFFAACC, 0xFF88AA), lo(200, 80, 40, 0x54, 0, 0, 15, 12)],
    // FLOW: drifting petals
    [hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0xFFCCDD, 0xFFAABB), lo(60, 30, 15, 0x21, 0, 0x40B0, 12, 10)],
    // DECAL: sun through branches
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 10, 0xFFFFE0, 0xFFF0C0), lo(180, 0x02, 16, 0, 0, DIR_SUN, 15, 12)],
    NOP_LAYER,
];

/// 17. Void Dimension - Abstract, surreal void space
const PRESET_VOID_DIMENSION: [[u64; 2]; 8] = [
    // RAMP: deep purple-black
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x100818, 0x080410), lo(40, 0, 0, 0, 0x00, DIR_UP, 15, 15)],
    // LOBE: eerie glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 6, 0x8000FF, 0x400088), lo(80, 12, 0, 0, 0, 0x70C0, 15, 10)],
    // BAND: dimensional rift
    [hi(OP_BAND, REGION_ALL, BLEND_ADD, 12, 0xFF00FF, 0x0000FF), lo(160, 30, 128, 0, 0, 0x5090, 15, 15)],
    // FOG: void mist
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x200830, 0x100418), lo(60, 120, 100, 0, 0, DIR_UP, 8, 6)],
    // SCATTER: floating fragments
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0xAA00FF, 0x6600AA), lo(180, 60, 40, 0x45, 0, 0, 15, 12)],
    // GRID: reality grid
    [hi(OP_GRID, REGION_ALL, BLEND_ADD, 4, 0x400880, 0x200440), lo(60, 48, 8, 0x00, 0, 0, 10, 8)],
    NOP_LAYER,
    NOP_LAYER,
];

/// 18. Retro Synthwave - 80s aesthetic, grid, sunset
const PRESET_RETRO_SYNTHWAVE: [[u64; 2]; 8] = [
    // RAMP: dark blue to magenta gradient
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000040, 0x400040), lo(120, 0, 20, 80, 0xA0, DIR_UP, 15, 15)],
    // LOBE: sunset horizon glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFF4488, 0xFF0044), lo(200, 28, 0, 0, 0, 0xF020, 15, 12)],
    // BAND: sun stripe
    [hi(OP_BAND, REGION_SKY, BLEND_ADD, 14, 0xFF8844, 0xFF4400), lo(220, 20, 140, 0, 0, 0xF020, 15, 15)],
    NOP_LAYER,
    // GRID: perspective floor grid
    [hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 8, 0x00FFFF, 0x0088AA), lo(180, 64, 16, 0x00, 0, 0, 15, 12)],
    // DECAL: setting sun
    [hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFF6600, 0xFF0066), lo(255, 0x04, 24, 0, 0, 0xF020, 15, 15)],
    // SCATTER: stars
    [hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 6, 0xFFFFFF, 0xAABBFF), lo(160, 100, 12, 0x41, 0, 0, 12, 10)],
    NOP_LAYER,
];

/// 19. Ancient Temple - Stone, torchlight, mysterious
const PRESET_ANCIENT_TEMPLE: [[u64; 2]; 8] = [
    // RAMP: dark stone
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x302820, 0x201810), lo(100, 40, 50, 60, 0xC8, DIR_UP, 15, 15)],
    // LOBE: torch glow
    [hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFF8840, 0xCC6620), lo(140, 16, 0, 0, 0, 0x60A0, 15, 12)],
    // FOG: dust and smoke
    [hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x403020, 0x302010), lo(80, 100, 80, 0, 0, DIR_UP, 12, 10)],
    NOP_LAYER,
    // SCATTER: torch flames
    [hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 12, 0xFF6600, 0xFF4400), lo(220, 20, 25, 0x83, 0, 0, 15, 15)],
    // GRID: stone blocks
    [hi(OP_GRID, REGION_WALLS, BLEND_ADD, 2, 0x504030, 0x403020), lo(60, 40, 12, 0x00, 0, 0, 12, 10)],
    // SCATTER: floating dust
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 2, 0xCCBBAA, 0xAA9988), lo(80, 40, 10, 0x30, 0, 0, 10, 8)],
    NOP_LAYER,
];

/// 20. Deep Space Nebula - Colorful cosmic clouds
const PRESET_DEEP_SPACE_NEBULA: [[u64; 2]; 8] = [
    // RAMP: deep space black
    [hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000008, 0x000004), lo(20, 0, 0, 0, 0x00, DIR_UP, 15, 15)],
    // LOBE: nebula glow (purple)
    [hi(OP_LOBE, REGION_ALL, BLEND_SCREEN, 6, 0x8040C0, 0x402080), lo(100, 32, 0, 0, 0, 0x50A0, 15, 10)],
    // LOBE: nebula glow (teal)
    [hi(OP_LOBE, REGION_ALL, BLEND_SCREEN, 5, 0x208080, 0x104040), lo(80, 28, 0, 0, 0, 0xB060, 15, 10)],
    // FOG: cosmic dust
    [hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x201030, 0x100818), lo(40, 150, 100, 0, 0, DIR_UP, 8, 6)],
    // SCATTER: stars
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 14, 0xFFFFFF, 0xAABBFF), lo(255, 180, 16, 0x42, 0, 0, 15, 12)],
    // SCATTER: distant galaxies
    [hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 8, 0xFF88CC, 0x88CCFF), lo(140, 40, 30, 0x33, 0, 0, 12, 10)],
    // FLOW: gas currents
    [hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 2, 0x604080, 0x408060), lo(40, 50, 20, 0x20, 0, 0x6090, 8, 6)],
    NOP_LAYER,
];

/// All presets array (20 presets)
static PRESETS: [[[u64; 2]; 8]; 20] = [
    PRESET_SUNNY_MEADOW,
    PRESET_CYBERPUNK_ALLEY,
    PRESET_VOID_STARS,
    PRESET_SUNSET_DESERT,
    PRESET_UNDERWATER_CAVE,
    PRESET_STORM_FRONT,
    PRESET_NEON_CITY,
    PRESET_ENCHANTED_FOREST,
    PRESET_ARCTIC_TUNDRA,
    PRESET_NEON_ARCADE,
    PRESET_DESERT_DUNES,
    PRESET_ALIEN_PLANET,
    PRESET_STORMY_OCEAN,
    PRESET_CRYSTAL_CAVERN,
    PRESET_TOXIC_WASTELAND,
    PRESET_CHERRY_BLOSSOM,
    PRESET_VOID_DIMENSION,
    PRESET_RETRO_SYNTHWAVE,
    PRESET_ANCIENT_TEMPLE,
    PRESET_DEEP_SPACE_NEBULA,
];

/// Preset names for display
const PRESET_NAMES: [&str; 20] = [
    "Sunny Meadow",
    "Cyberpunk Alley",
    "Void Stars",
    "Sunset Desert",
    "Underwater Cave",
    "Storm Front",
    "Neon City",
    "Enchanted Forest",
    "Arctic Tundra",
    "Neon Arcade",
    "Desert Dunes",
    "Alien Planet",
    "Stormy Ocean",
    "Crystal Cavern",
    "Toxic Wasteland",
    "Cherry Blossom",
    "Void Dimension",
    "Retro Synthwave",
    "Ancient Temple",
    "Deep Space Nebula",
];

const PRESET_COUNT: usize = 20;

// ============================================================================
// Game State
// ============================================================================

static mut PRESET_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 15.0;
static mut SPHERE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut MATERIAL_METALLIC_U8: i32 = 77; // ~0.30 * 255
static mut MATERIAL_ROUGHNESS_U8: i32 = 128; // ~0.50 * 255

const SHAPE_COUNT: i32 = 3;
const SHAPE_NAMES: [&str; 3] = ["Sphere", "Cube", "Torus"];

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);

        // Generate meshes for the scene
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);

        // Set up initial light
        light_set(0, 0.5, -0.7, 0.5);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.0);
        light_enable(0);

        // Set up initial environment
        epu_set(0, PRESETS[0].as_ptr() as *const u64);

        // Register debug values
        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(
            b"index".as_ptr(),
            5,
            &raw const PRESET_INDEX as *const i32 as *const u8,
        );
        debug_group_end();

        debug_group_begin(b"camera".as_ptr(), 6);
        debug_register_f32(
            b"angle".as_ptr(),
            5,
            &raw const CAM_ANGLE as *const f32 as *const u8,
        );
        debug_register_f32(
            b"elevation".as_ptr(),
            9,
            &raw const CAM_ELEVATION as *const f32 as *const u8,
        );
        debug_group_end();

        debug_group_begin(b"shape".as_ptr(), 5);
        debug_register_i32(
            b"index".as_ptr(),
            5,
            &raw const SHAPE_INDEX as *const i32 as *const u8,
        );
        debug_group_end();

        debug_group_begin(b"material".as_ptr(), 8);
        debug_register_i32(
            b"metallic_u8".as_ptr(),
            11,
            &raw const MATERIAL_METALLIC_U8 as *const i32 as *const u8,
        );
        debug_register_i32(
            b"roughness_u8".as_ptr(),
            12,
            &raw const MATERIAL_ROUGHNESS_U8 as *const i32 as *const u8,
        );
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        // Clamp preset index
        if PRESET_INDEX < 0 {
            PRESET_INDEX = 0;
        }
        if PRESET_INDEX >= PRESET_COUNT as i32 {
            PRESET_INDEX = PRESET_COUNT as i32 - 1;
        }

        // Clamp shape index
        if SHAPE_INDEX < 0 {
            SHAPE_INDEX = 0;
        }
        if SHAPE_INDEX >= SHAPE_COUNT {
            SHAPE_INDEX = SHAPE_COUNT - 1;
        }

        // Clamp camera elevation
        CAM_ELEVATION = if CAM_ELEVATION < -60.0 {
            -60.0
        } else if CAM_ELEVATION > 60.0 {
            60.0
        } else {
            CAM_ELEVATION
        };

        // Clamp material parameters (0..255)
        if MATERIAL_METALLIC_U8 < 0 {
            MATERIAL_METALLIC_U8 = 0;
        } else if MATERIAL_METALLIC_U8 > 255 {
            MATERIAL_METALLIC_U8 = 255;
        }
        if MATERIAL_ROUGHNESS_U8 < 0 {
            MATERIAL_ROUGHNESS_U8 = 0;
        } else if MATERIAL_ROUGHNESS_U8 > 255 {
            MATERIAL_ROUGHNESS_U8 = 255;
        }

        // Update EPU configuration
        epu_set(0, PRESETS[PRESET_INDEX as usize].as_ptr() as *const u64);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Cycle presets with A/B buttons
        if button_pressed(0, button::A) != 0 {
            PRESET_INDEX = (PRESET_INDEX + 1) % PRESET_COUNT as i32;
            epu_set(0, PRESETS[PRESET_INDEX as usize].as_ptr() as *const u64);
        }
        if button_pressed(0, button::B) != 0 {
            PRESET_INDEX = (PRESET_INDEX + PRESET_COUNT as i32 - 1) % PRESET_COUNT as i32;
            epu_set(0, PRESETS[PRESET_INDEX as usize].as_ptr() as *const u64);
        }

        // Cycle shapes with X button
        if button_pressed(0, button::X) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT;
        }

        // Camera control via left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION -= stick_y * 2.0;
            CAM_ELEVATION = if CAM_ELEVATION < -60.0 {
                -60.0
            } else if CAM_ELEVATION > 60.0 {
                60.0
            } else {
                CAM_ELEVATION
            };
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Calculate camera position
        let angle_rad = CAM_ANGLE * 0.0174533;
        let elev_rad = CAM_ELEVATION * 0.0174533;
        let dist = 5.0;

        let cam_x = dist * libm::cosf(elev_rad) * libm::sinf(angle_rad);
        let cam_y = dist * libm::sinf(elev_rad) + 1.0;
        let cam_z = dist * libm::cosf(elev_rad) * libm::cosf(angle_rad);

        camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Draw the EPU environment background
        epu_draw(0);

        // Draw a shape to show lighting from the environment
        push_identity();
        set_color(0x888899FF);
        material_metallic((MATERIAL_METALLIC_U8 as f32) / 255.0);
        material_roughness((MATERIAL_ROUGHNESS_U8 as f32) / 255.0);

        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        // Draw UI overlay
        draw_ui();
    }
}

unsafe fn draw_ui() {
    // Title
    let title = b"EPU Inspector (v2)";
    set_color(0xFFFFFFFF);
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

    // Current preset name
    let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
    let mut label = [0u8; 48];
    let prefix = b"Preset: ";
    label[..prefix.len()].copy_from_slice(prefix);
    let name = preset_name.as_bytes();
    let name_len = if name.len() > 40 { 40 } else { name.len() };
    label[prefix.len()..prefix.len() + name_len].copy_from_slice(&name[..name_len]);
    set_color(0xCCCCCCFF);
    draw_text(
        label.as_ptr(),
        (prefix.len() + name_len) as u32,
        10.0,
        42.0,
        18.0,
    );

    // Current shape name
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut shape_label = [0u8; 32];
    let shape_prefix = b"Shape: ";
    shape_label[..shape_prefix.len()].copy_from_slice(shape_prefix);
    let sname = shape_name.as_bytes();
    shape_label[shape_prefix.len()..shape_prefix.len() + sname.len()].copy_from_slice(sname);
    set_color(0xAAAAAAFF);
    draw_text(
        shape_label.as_ptr(),
        (shape_prefix.len() + sname.len()) as u32,
        10.0,
        66.0,
        16.0,
    );

    // Instructions
    let hint1 = b"A/B: Cycle presets | X: Cycle shapes";
    set_color(0x888888FF);
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 94.0, 14.0);

    let hint2 = b"Left stick: Orbit camera | F4: Debug panel";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 112.0, 14.0);

    // Preset index indicator (supports up to 99 presets)
    let mut idx_label = [0u8; 16];
    let current = PRESET_INDEX as u8 + 1;
    let total = PRESET_COUNT as u8;
    let mut pos = 0usize;

    idx_label[pos] = b'[';
    pos += 1;

    // Write current index (1-based)
    if current >= 10 {
        idx_label[pos] = b'0' + (current / 10);
        pos += 1;
    }
    idx_label[pos] = b'0' + (current % 10);
    pos += 1;

    idx_label[pos] = b'/';
    pos += 1;

    // Write total count
    if total >= 10 {
        idx_label[pos] = b'0' + (total / 10);
        pos += 1;
    }
    idx_label[pos] = b'0' + (total % 10);
    pos += 1;

    idx_label[pos] = b']';
    pos += 1;

    set_color(0x666666FF);
    draw_text(idx_label.as_ptr(), pos as u32, 10.0, 130.0, 12.0);
}
