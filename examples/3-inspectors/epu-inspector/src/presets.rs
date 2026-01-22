//! EPU Preset Configurations (v2 128-bit format)
//!
//! Contains all 20 environment presets for the EPU inspector demo.
//!
//! Each 128-bit layer is stored as [hi, lo] u64 pair:
//!
//! u64 hi [bits 127..64]:
//!   bits 63..59: opcode     (5)   - NOP=0, bounds=1..7 (RAMP=1, LOBE=2, BAND=3, FOG=4; 5..7 reserved), features=8.. (DECAL=8, GRID=9, SCATTER=10, FLOW=11)
//!   bits 58..56: region     (3)   - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001, ALL=0b111
//!   bits 55..53: blend      (3)   - ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
//!   bits 52..49: reserved   (4)
//!   bit  48:     reserved   (1)
//!   bits 47..24: color_a    (24)  - RGB24 primary color
//!   bits 23..0:  color_b    (24)  - RGB24 secondary color
//!
//! u64 lo [bits 63..0]:
//!   bits 63..56: intensity  (8)
//!   bits 55..48: param_a    (8)
//!   bits 47..40: param_b    (8)
//!   bits 39..32: param_c    (8)
//!   bits 31..24: param_d    (8)
//!   bits 23..8:  direction  (16)  - Octahedral encoded
//!   bits 7..4:   alpha_a    (4)   - color_a alpha (0-15)
//!   bits 3..0:   alpha_b    (4)   - color_b alpha (0-15)
//!
//! Slots 0-3: Bounds layers (RAMP, LOBE, BAND, FOG)
//! Slots 4-7: Feature layers (DECAL, GRID, SCATTER, FLOW)

use crate::constants::{
    hi,
    hi_meta,
    lo,
    APERTURE_ROUNDED_RECT,
    ATMO_RAYLEIGH,
    // Blend modes
    BLEND_ADD,
    BLEND_LERP,
    BLEND_MULTIPLY,
    BLEND_SCREEN,
    CELESTIAL_MOON,
    // Variant IDs
    CELL_VORONOI,
    DIR_DOWN,
    DIR_SUN,
    DIR_SUNSET,
    // Directions
    DIR_UP,
    DOMAIN_AXIS_CYL,
    // Domain IDs
    DOMAIN_DIRECT3D,
    DOMAIN_TANGENT_LOCAL,
    NOP_LAYER,
    OP_APERTURE,
    OP_ATMOSPHERE,
    OP_BAND,
    OP_CELESTIAL,
    OP_CELL,
    OP_DECAL,
    OP_FLOW,
    OP_FOG,
    OP_GRID,
    OP_LOBE,
    OP_PATCHES,
    OP_PLANE,
    OP_PORTAL,
    // Opcodes
    OP_RAMP,
    OP_SCATTER,
    // vNext Enclosure opcodes
    OP_SILHOUETTE,
    OP_SPLIT,
    // vNext Radiance opcodes
    OP_TRACE,
    OP_VEIL,
    PATCHES_BLOBS,
    PLANE_TILES,
    PORTAL_VORTEX,
    // Regions
    REGION_ALL,
    REGION_FLOOR,
    REGION_SKY,
    REGION_WALLS,
    SILHOUETTE_CITY,
    SPLIT_HALF,
    TRACE_LIGHTNING,
    VEIL_CURTAINS,
};

// =============================================================================
// 20 Environment Presets (v2 128-bit format)
// =============================================================================

/// 1. Sunny Meadow - Bright blue sky, green grass, warm sun
const PRESET_SUNNY_MEADOW: [[u64; 2]; 8] = [
    // RAMP: sky=blue, floor=green
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x6496DC, 0x508C50),
        lo(180, 200, 180, 150, 0xA5, DIR_UP, 15, 15),
    ],
    // LOBE: warm sun glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 15, 0xFFF0C8, 0xFFE8B0),
        lo(180, 32, 0, 0, 0, DIR_SUN, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
    // DECAL: sun disk
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFDCB4),
        lo(255, 0x02, 12, 0, 0, DIR_SUN, 15, 15),
    ],
    // FLOW: clouds (LERP)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_LERP, 0, 0xFFFFFF, 0xE0E8F0),
        lo(60, 32, 96, 0x30, 64, DIR_UP, 15, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 2. Cyberpunk Alley - Neon-lit urban with fog and rain
const PRESET_CYBERPUNK_ALLEY: [[u64; 2]; 8] = [
    // RAMP: dark walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x101020, 0x080810),
        lo(100, 20, 20, 30, 0xA5, DIR_UP, 15, 15),
    ],
    // LOBE: magenta glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFF00FF, 0xFF44AA),
        lo(140, 24, 0, 2, 180, 0x30C0, 15, 10),
    ],
    // LOBE: cyan glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0x00FFFF, 0x00AAFF),
        lo(120, 28, 0, 2, 170, 0xD060, 15, 10),
    ],
    // FOG: purple haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x402040, 0x201030),
        lo(80, 140, 100, 0, 0, DIR_UP, 12, 8),
    ],
    // GRID: walls, cyan accent
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 8, 0x00AAAA, 0x006666),
        lo(80, 48, 12, 0x00, 0, 0, 15, 10),
    ],
    // DECAL: pink sign
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 10, 0xFF88AA, 0xFF4488),
        lo(200, 0x04, 40, 30, 50, 0x80C0, 15, 12),
    ],
    // FLOW: rain
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x808080, 0x404050),
        lo(40, 64, 180, 0x11, 0, DIR_DOWN, 10, 6),
    ],
    // SCATTER: warm windows
    [
        hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 6, 0xFFAA44, 0xFF8822),
        lo(180, 120, 35, 0x23, 0, 0, 15, 12),
    ],
];

/// 3. Void Stars - Black void with twinkling stars
const PRESET_VOID_STARS: [[u64; 2]; 8] = [
    // RAMP: black everywhere
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000005, 0x000008),
        lo(10, 0, 0, 0, 0xF0, DIR_UP, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    // SCATTER: white stars
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 15, 0xFFFFFF, 0xAABBFF),
        lo(255, 200, 20, 0x83, 0, 0, 15, 10),
    ],
    // SCATTER: blue distant stars
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x8888FF, 0x4444AA),
        lo(180, 255, 8, 0x44, 0, 0, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 4. Sunset Desert - Orange/red desert at dusk
const PRESET_SUNSET_DESERT: [[u64; 2]; 8] = [
    // RAMP: orange sky, tan sand
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xFF6030, 0xC09060),
        lo(200, 220, 180, 140, 0xA3, DIR_UP, 15, 15),
    ],
    // LOBE: setting sun glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 14, 0xFFAA40, 0xFF6600),
        lo(240, 28, 0, 0, 0, DIR_SUNSET, 15, 12),
    ],
    // FOG: dust haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x804020, 0x603010),
        lo(60, 100, 80, 0, 0, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // DECAL: sun disk near horizon
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFF4400, 0xFF2200),
        lo(255, 0x04, 22, 0, 0, DIR_SUNSET, 15, 15),
    ],
    // FLOW: blowing sand
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_LERP, 0, 0xC09060, 0x906040),
        lo(50, 32, 90, 0x11, 100, 0x4080, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 5. Underwater Cave - Deep blue caustics and bubbles
const PRESET_UNDERWATER_CAVE: [[u64; 2]; 8] = [
    // RAMP: dark teal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x004060, 0x002030),
        lo(100, 30, 50, 40, 0xBB, DIR_UP, 15, 15),
    ],
    // LOBE: blue-green from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0x40A0A0, 0x206060),
        lo(100, 16, 0, 0, 0, DIR_UP, 15, 10),
    ],
    // FOG: deep blue
    [
        hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x203050, 0x102030),
        lo(120, 80, 100, 0, 0, DIR_UP, 12, 8),
    ],
    NOP_LAYER,
    // FLOW: caustics
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 4, 0x40C0C0, 0x208080),
        lo(80, 40, 90, 0x22, 120, 0x9870, 15, 10),
    ],
    // SCATTER: bubbles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFFFFFF, 0xAADDFF),
        lo(140, 80, 12, 0x64, 0, 0, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 6. Storm Front - Dark skies, lightning, rain
const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // RAMP: dark gray storm clouds
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x202830, 0x303840),
        lo(80, 40, 50, 60, 0xC8, DIR_UP, 15, 15),
    ],
    // LOBE: dim ambient from sky
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 4, 0x405060, 0x304050),
        lo(60, 24, 0, 1, 40, DIR_UP, 12, 10),
    ],
    // BAND: dark cloud band
    [
        hi(OP_BAND, REGION_SKY, BLEND_MULTIPLY, 0, 0x101820, 0x081018),
        lo(120, 60, 140, 0, 80, DIR_UP, 15, 12),
    ],
    // FOG: mist
    [
        hi(OP_FOG, REGION_ALL, BLEND_LERP, 0, 0x404850, 0x303840),
        lo(100, 120, 80, 0, 0, DIR_UP, 10, 8),
    ],
    // SCATTER: lightning flashes
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xCCDDFF),
        lo(255, 10, 80, 0xF2, 0, 0, 15, 12),
    ],
    // FLOW: rain
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x606880, 0x404860),
        lo(60, 80, 200, 0x11, 0, DIR_DOWN, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 7. Neon City - Futuristic city at night
const PRESET_NEON_CITY: [[u64; 2]; 8] = [
    // RAMP: dark blue night
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x080818, 0x040410),
        lo(60, 10, 15, 20, 0xA8, DIR_UP, 15, 15),
    ],
    // LOBE: orange street glow from below
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0xFF8844, 0xCC6622),
        lo(100, 20, 0, 2, 120, 0x8000, 15, 12),
    ],
    // BAND: sky glow pollution
    [
        hi(OP_BAND, REGION_SKY, BLEND_SCREEN, 2, 0x442244, 0x221122),
        lo(40, 80, 180, 0, 40, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // GRID: building windows
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 6, 0xFFCC88, 0xFF8844),
        lo(120, 32, 20, 0x02, 0, 0, 15, 12),
    ],
    // SCATTER: neon signs
    [
        hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 10, 0xFF00AA, 0x00FFAA),
        lo(200, 40, 50, 0x34, 0, 0, 15, 15),
    ],
    // DECAL: billboard
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 12, 0x00FFFF, 0xFF00FF),
        lo(220, 0x04, 60, 40, 30, 0x90B0, 15, 15),
    ],
    // FLOW: rain haze
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x606870, 0x303040),
        lo(50, 64, 200, 0x11, 40, DIR_DOWN, 10, 6),
    ],
];

/// 8. Enchanted Forest - Magical woodland with glowing elements
const PRESET_ENCHANTED_FOREST: [[u64; 2]; 8] = [
    // RAMP: deep green-blue
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x102818, 0x081810),
        lo(80, 40, 60, 50, 0xD4, DIR_UP, 15, 15),
    ],
    // LOBE: moonlight filtering through
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 6, 0xA0C0D0, 0x608090),
        lo(70, 14, 0, 0, 0, 0x60E0, 15, 10),
    ],
    // FOG: mystical mist
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x204030, 0x102820),
        lo(80, 100, 80, 0, 0, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // SCATTER: fireflies/magical particles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x80FF60, 0x40FF80),
        lo(220, 60, 10, 0xC9, 0, 0, 15, 12),
    ],
    // SCATTER: glowing mushrooms
    [
        hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 8, 0x6060FF, 0x4040AA),
        lo(160, 30, 15, 0x21, 0, 0, 15, 10),
    ],
    // FLOW: floating pollen
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 2, 0xFFFFAA, 0xAAAA66),
        lo(40, 20, 80, 0x30, 120, 0x6080, 10, 6),
    ],
    NOP_LAYER,
];

/// 9. Arctic Tundra - Cold, snowy, white/blue palette
const PRESET_ARCTIC_TUNDRA: [[u64; 2]; 8] = [
    // RAMP: white snow, pale blue sky
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x6080A8, 0x90B8E8),
        lo(180, 210, 220, 235, 0xBE, DIR_UP, 15, 15),
    ],
    // LOBE: cold sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0xE8F4FF, 0xC0D8FF),
        lo(100, 16, 0, 0, 0, 0xC0A0, 15, 12),
    ],
    // FOG: cold haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x90B8E8, 0x607090),
        lo(40, 80, 100, 0, 0, DIR_UP, 12, 10),
    ],
    NOP_LAYER,
    // SCATTER: snowflakes
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 4, 0xFFFFFF, 0xDDEEFF),
        lo(200, 100, 72, 0x4C, 0, DIR_DOWN, 15, 12),
    ],
    // FLOW: blowing snow
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0xFFFFFF, 0xCCDDEE),
        lo(40, 50, 160, 0x11, 140, 0x40A0, 10, 8),
    ],
    // FLOW: aurora
    [
        hi(OP_FLOW, REGION_SKY, BLEND_SCREEN, 0, 0x40FFB0, 0x8040FF),
        lo(20, 24, 60, 0x30, 160, 0x5090, 12, 8),
    ],
    NOP_LAYER,
];

/// 10. Neon Arcade - Retro gaming, colorful neon on black
const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // RAMP: black everywhere
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000010, 0x000008),
        lo(50, 0, 0, 0, 0x00, DIR_UP, 15, 15),
    ],
    // BAND: magenta stripes
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 10, 0xFF00FF, 0xAA00AA),
        lo(200, 48, 96, 0, 80, DIR_UP, 15, 12),
    ],
    // BAND: cyan stripes offset
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 8, 0x00FFFF, 0x00AAAA),
        lo(160, 48, 144, 0, 120, DIR_UP, 15, 12),
    ],
    NOP_LAYER,
    // GRID: cyan floor tiles
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 8, 0x00FFFF, 0x008888),
        lo(160, 64, 16, 0x14, 0, 0, 15, 12),
    ],
    // SCATTER: magenta particles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 8, 0xFF00FF, 0xFF44FF),
        lo(220, 64, 60, 0x3C, 0, 0, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 11. Desert Dunes - Hot, arid, golden/orange midday
const PRESET_DESERT_DUNES: [[u64; 2]; 8] = [
    // RAMP: golden sand floor, bright sky
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xE0C080, 0xD0A050),
        lo(160, 220, 200, 180, 0xB2, DIR_UP, 15, 15),
    ],
    // LOBE: blazing sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 15, 0xFFFFE0, 0xFFF0C0),
        lo(255, 32, 0, 0, 0, 0x8090, 15, 12),
    ],
    // FOG: golden haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0xE0C080, 0xC0A060),
        lo(100, 80, 100, 0, 0, DIR_UP, 12, 10),
    ],
    NOP_LAYER,
    // DECAL: sun disk
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFF8E0),
        lo(255, 0x02, 14, 0, 0, 0x8090, 15, 15),
    ],
    // FLOW: sand drift
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_LERP, 0, 0xD0A050, 0xB08040),
        lo(80, 24, 120, 0x11, 90, DIR_UP, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 12. Alien Planet - Strange colors, foreign atmosphere
const PRESET_ALIEN_PLANET: [[u64; 2]; 8] = [
    // RAMP: purple sky, teal ground
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x602080, 0x208060),
        lo(140, 160, 140, 120, 0xB6, DIR_UP, 15, 15),
    ],
    // LOBE: twin suns (orange and blue)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFF8800, 0x0088FF),
        lo(180, 20, 0, 1, 60, 0xA0C0, 15, 12),
    ],
    // FOG: alien atmosphere
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x402060, 0x204040),
        lo(80, 100, 80, 0, 0, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // SCATTER: floating spores
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFF44FF, 0x44FFFF),
        lo(160, 80, 20, 0x55, 0, 0, 15, 12),
    ],
    // DECAL: alien sun
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 14, 0xFF6600, 0x0066FF),
        lo(240, 0x04, 20, 0, 40, 0xA0C0, 15, 15),
    ],
    // FLOW: gas currents
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0x804080, 0x408040),
        lo(50, 40, 100, 0x22, 160, 0x5090, 10, 8),
    ],
    NOP_LAYER,
];

/// 13. Stormy Ocean - Dark seas, crashing waves
const PRESET_STORMY_OCEAN: [[u64; 2]; 8] = [
    // RAMP: dark gray-blue
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x304050, 0x203040),
        lo(100, 50, 60, 80, 0xC6, DIR_UP, 15, 15),
    ],
    // LOBE: dim overcast light
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 4, 0x506070, 0x405060),
        lo(60, 20, 0, 0, 0, DIR_UP, 12, 10),
    ],
    // BAND: horizon line
    [
        hi(OP_BAND, REGION_ALL, BLEND_LERP, 0, 0x405060, 0x304050),
        lo(80, 40, 128, 0, 40, DIR_UP, 12, 10),
    ],
    // FOG: sea spray
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x405060, 0x304050),
        lo(100, 120, 80, 0, 0, DIR_UP, 10, 8),
    ],
    // FLOW: waves
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 4, 0x6080A0, 0x406080),
        lo(100, 60, 80, 0x21, 80, 0x80A0, 15, 10),
    ],
    // SCATTER: foam
    [
        hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 2, 0xFFFFFF, 0xCCDDEE),
        lo(180, 60, 30, 0x32, 0, 0, 12, 10),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 14. Crystal Cavern - Glowing crystals, refractions
const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // RAMP: dark purple rock
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x201030, 0x100820),
        lo(80, 30, 40, 50, 0xD5, DIR_UP, 15, 15),
    ],
    // LOBE: crystal glow from below
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0x8040FF, 0x4020AA),
        lo(140, 18, 0, 1, 60, 0x8010, 15, 12),
    ],
    // FOG: faint mist
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x302040, 0x201030),
        lo(60, 80, 100, 0, 0, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // SCATTER: glowing crystals (cyan)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 12, 0x00FFFF, 0x0088AA),
        lo(220, 40, 30, 0x24, 0, 0, 15, 12),
    ],
    // SCATTER: glowing crystals (magenta)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0xFF00FF, 0xAA0088),
        lo(200, 30, 25, 0x23, 0, 0, 15, 12),
    ],
    // GRID: crystal facets on walls
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 6, 0x6040A0, 0x402080),
        lo(80, 24, 12, 0x22, 48, 0, 12, 10),
    ],
    NOP_LAYER,
];

/// 15. Toxic Wasteland - Green/yellow toxic, industrial decay
const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // RAMP: sickly yellow-green
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x404020, 0x606030),
        lo(100, 60, 80, 70, 0xC4, DIR_UP, 15, 15),
    ],
    // LOBE: toxic glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0x80FF00, 0x40AA00),
        lo(120, 16, 0, 2, 140, 0x9060, 15, 12),
    ],
    // FOG: toxic haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x605020, 0x403010),
        lo(140, 100, 80, 0, 0, DIR_UP, 12, 10),
    ],
    NOP_LAYER,
    // SCATTER: toxic bubbles
    [
        hi(OP_SCATTER, REGION_FLOOR, BLEND_ADD, 10, 0xAAFF00, 0x66AA00),
        lo(200, 50, 20, 0x62, 0, 0, 15, 12),
    ],
    // FLOW: toxic sludge
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 6, 0x80C000, 0x408000),
        lo(120, 30, 90, 0x20, 150, DIR_UP, 12, 10),
    ],
    // GRID: industrial grating
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 2, 0x404040, 0x303030),
        lo(40, 32, 8, 0x11, 0, 0, 10, 8),
    ],
    NOP_LAYER,
];

/// 16. Cherry Blossom - Pink petals, soft spring day
const PRESET_CHERRY_BLOSSOM: [[u64; 2]; 8] = [
    // RAMP: soft blue sky, green grass
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xA0C0E0, 0x60A060),
        lo(180, 200, 180, 160, 0xA6, DIR_UP, 15, 15),
    ],
    // LOBE: warm spring sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFFF0E0, 0xFFE0C0),
        lo(160, 24, 0, 0, 0, DIR_SUN, 15, 12),
    ],
    // FOG: soft haze
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0xFFE0F0, 0xFFC0E0),
        lo(40, 100, 80, 0, 0, DIR_UP, 8, 6),
    ],
    NOP_LAYER,
    // SCATTER: pink petals
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 4, 0xFFAACC, 0xFF88AA),
        lo(200, 80, 40, 0x54, 0, 0, 15, 12),
    ],
    // FLOW: drifting petals
    [
        hi(OP_FLOW, REGION_ALL, BLEND_LERP, 0, 0xFFCCDD, 0xFFAABB),
        lo(60, 30, 80, 0x21, 120, 0x40B0, 12, 10),
    ],
    // DECAL: sun through branches
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 10, 0xFFFFE0, 0xFFF0C0),
        lo(180, 0x02, 16, 0, 30, DIR_SUN, 15, 12),
    ],
    NOP_LAYER,
];

/// 17. Void Dimension - Abstract, surreal void space
const PRESET_VOID_DIMENSION: [[u64; 2]; 8] = [
    // RAMP: deep purple-black
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x100818, 0x080410),
        lo(40, 0, 0, 0, 0x00, DIR_UP, 15, 15),
    ],
    // LOBE: eerie glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 6, 0x8000FF, 0x400088),
        lo(80, 12, 0, 2, 120, 0x70C0, 15, 10),
    ],
    // BAND: dimensional rift
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 12, 0xFF00FF, 0x0000FF),
        lo(160, 30, 128, 80, 160, 0x5090, 15, 15),
    ],
    // FOG: void mist
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x200830, 0x100418),
        lo(60, 120, 100, 0, 0, DIR_UP, 8, 6),
    ],
    // SCATTER: floating fragments
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0xAA00FF, 0x6600AA),
        lo(180, 60, 40, 0x45, 0, 0, 15, 12),
    ],
    // GRID: reality grid
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 4, 0x400880, 0x200440),
        lo(60, 48, 8, 0x14, 64, 0, 10, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 18. Retro Synthwave - 80s aesthetic, grid, sunset
const PRESET_RETRO_SYNTHWAVE: [[u64; 2]; 8] = [
    // RAMP: dark blue to magenta gradient
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000040, 0x400040),
        lo(120, 0, 20, 80, 0xA0, DIR_UP, 15, 15),
    ],
    // LOBE: sunset horizon glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFF4488, 0xFF0044),
        lo(200, 28, 0, 1, 40, 0xF020, 15, 12),
    ],
    // BAND: sun stripe
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 14, 0xFF8844, 0xFF4400),
        lo(220, 20, 140, 80, 40, 0xF020, 15, 15),
    ],
    NOP_LAYER,
    // GRID: perspective floor grid
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 8, 0x00FFFF, 0x0088AA),
        lo(180, 64, 16, 0x16, 0, 0, 15, 12),
    ],
    // DECAL: setting sun
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFF6600, 0xFF0066),
        lo(255, 0x04, 24, 0, 20, 0xF020, 15, 15),
    ],
    // SCATTER: stars
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 6, 0xFFFFFF, 0xAABBFF),
        lo(160, 100, 12, 0x43, 0, 0, 12, 10),
    ],
    NOP_LAYER,
];

/// 19. Ancient Temple - Stone, torchlight, mysterious
const PRESET_ANCIENT_TEMPLE: [[u64; 2]; 8] = [
    // RAMP: dark stone
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x302820, 0x201810),
        lo(100, 40, 50, 60, 0xC8, DIR_UP, 15, 15),
    ],
    // LOBE: torch glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFF8840, 0xCC6620),
        lo(140, 16, 0, 2, 200, 0x60A0, 15, 12),
    ],
    // FOG: dust and smoke
    [
        hi(OP_FOG, REGION_ALL, BLEND_MULTIPLY, 0, 0x403020, 0x302010),
        lo(80, 100, 80, 0, 0, DIR_UP, 12, 10),
    ],
    NOP_LAYER,
    // SCATTER: torch flames
    [
        hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 12, 0xFF6600, 0xFF4400),
        lo(220, 20, 25, 0x83, 0, 0, 15, 15),
    ],
    // GRID: stone blocks
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 2, 0x504030, 0x403020),
        lo(60, 40, 12, 0x00, 0, 0, 12, 10),
    ],
    // SCATTER: floating dust
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 2, 0xCCBBAA, 0xAA9988),
        lo(80, 40, 10, 0x32, 0, 0, 10, 8),
    ],
    // FLOW: torch smoke
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0x403020, 0x201810),
        lo(30, 40, 60, 0x20, 120, DIR_UP, 10, 8),
    ],
];

/// 20. Deep Space Nebula - Colorful cosmic clouds
const PRESET_DEEP_SPACE_NEBULA: [[u64; 2]; 8] = [
    // RAMP: deep space black
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000008, 0x000004),
        lo(20, 0, 0, 0, 0x00, DIR_UP, 15, 15),
    ],
    // LOBE: nebula glow (purple)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_SCREEN, 6, 0x8040C0, 0x402080),
        lo(100, 32, 0, 1, 40, 0x50A0, 15, 10),
    ],
    // LOBE: nebula glow (teal)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_SCREEN, 5, 0x208080, 0x104040),
        lo(80, 28, 0, 1, 35, 0xB060, 15, 10),
    ],
    // FOG: cosmic dust
    [
        hi(OP_FOG, REGION_ALL, BLEND_SCREEN, 0, 0x201030, 0x100818),
        lo(40, 150, 100, 0, 0, DIR_UP, 8, 6),
    ],
    // SCATTER: stars
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 14, 0xFFFFFF, 0xAABBFF),
        lo(255, 180, 16, 0x42, 0, 0, 15, 12),
    ],
    // SCATTER: distant galaxies
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 8, 0xFF88CC, 0x88CCFF),
        lo(140, 40, 30, 0x33, 0, 0, 12, 10),
    ],
    // FLOW: gas currents
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 2, 0x604080, 0x408060),
        lo(40, 50, 110, 0x20, 160, 0x6090, 8, 6),
    ],
    NOP_LAYER,
];

// =============================================================================
// vNext Presets - Demonstrating new opcodes
// =============================================================================

/// 21. Voronoi Cells - Shattered glass/crystal mosaic sky (CELL opcode)
const PRESET_VORONOI_CELLS: [[u64; 2]; 8] = [
    // RAMP: dark blue base
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x101830, 0x080810),
        lo(100, 40, 50, 60, 0xA5, DIR_UP, 15, 15),
    ],
    // CELL (0x05): Voronoi variant (2) - shattered enclosure
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x000000,
            0x3080C0,
        ),
        lo(180, 48, 180, 32, 0, DIR_UP, 15, 10),
    ],
    // LOBE: blue glow from cells
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x4080FF, 0x2040AA),
        lo(120, 20, 0, 0, 0, 0x60C0, 15, 10),
    ],
    NOP_LAYER,
    // SCATTER: bright points at cell centers
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 8, 0xFFFFFF, 0xAADDFF),
        lo(200, 40, 20, 0x53, 0, 0, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// 22. Noise Patches - Organic cloudy enclosure (PATCHES opcode)
const PRESET_NOISE_PATCHES: [[u64; 2]; 8] = [
    // RAMP: sky blue base
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x4080C0, 0x302820),
        lo(160, 180, 160, 140, 0xA5, DIR_UP, 15, 15),
    ],
    // PATCHES (0x06): Blobs variant (0) - organic cloud patches
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0x4080C0,
            0xFFFFFF,
        ),
        lo(0, 64, 160, 80, 0, DIR_UP, 15, 15),
    ],
    // LOBE: sun glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFFF0C8, 0xFFE8B0),
        lo(160, 28, 0, 0, 0, DIR_SUN, 15, 12),
    ],
    NOP_LAYER,
    // DECAL: sun disk
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFDCB4),
        lo(255, 0x02, 12, 0, 0, DIR_SUN, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// 23. Shaped Aperture - Interior view through rounded window (APERTURE opcode)
const PRESET_SHAPED_APERTURE: [[u64; 2]; 8] = [
    // RAMP: outdoor sky visible through aperture
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x6496DC, 0x508C50),
        lo(180, 200, 180, 150, 0xA5, DIR_UP, 15, 15),
    ],
    // APERTURE (0x07): Rounded rect variant (2) - window frame
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ROUNDED_RECT,
            0x000000,
            0x000000,
        ),
        lo(80, 128, 128, 64, 48, 0x7F00, 0, 0),
    ],
    // LOBE: sun glow through window
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFFF8E0, 0xFFE0B0),
        lo(140, 24, 0, 0, 0, DIR_SUN, 15, 12),
    ],
    NOP_LAYER,
    // Interior wall texture via GRID
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 4, 0x404040, 0x303030),
        lo(60, 32, 8, 0x00, 0, 0, 12, 10),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// 24. Lightning Storm - Electric traces in the sky (TRACE opcode)
const PRESET_LIGHTNING_TRACES: [[u64; 2]; 8] = [
    // RAMP: dark storm clouds
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x202830, 0x303840),
        lo(80, 40, 50, 60, 0xC8, DIR_UP, 15, 15),
    ],
    // BAND: ominous cloud layer
    [
        hi(OP_BAND, REGION_SKY, BLEND_MULTIPLY, 0, 0x181828, 0x101020),
        lo(120, 60, 140, 0, 80, DIR_UP, 15, 12),
    ],
    // FOG: mist
    [
        hi(OP_FOG, REGION_ALL, BLEND_LERP, 0, 0x404850, 0x303840),
        lo(80, 100, 80, 0, 0, DIR_UP, 10, 8),
    ],
    NOP_LAYER,
    // TRACE (0x0C): Lightning variant (0) - electric bolts
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            TRACE_LIGHTNING,
            0xFFFFFF,
            0x8080FF,
        ),
        lo(255, 48, 64, 160, 0x24, DIR_UP, 15, 10),
    ],
    // SCATTER: rain
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_LERP, 0, 0x808080, 0x606070),
        lo(50, 80, 40, 0x54, 0, DIR_DOWN, 10, 6),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 25. Curtain Veil - Theatrical lighting ribbons (VEIL opcode)
const PRESET_CURTAIN_VEIL: [[u64; 2]; 8] = [
    // RAMP: dark stage
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x080810, 0x040408),
        lo(60, 20, 20, 30, 0xA5, DIR_UP, 15, 15),
    ],
    // VEIL (0x0D): Curtains variant (0) - vertical ribbons
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0xFF4488,
            0x8822AA,
        ),
        lo(200, 80, 100, 80, 48, DIR_UP, 15, 8),
    ],
    // Second VEIL layer - cyan ribbons
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x44FFFF,
            0x2288AA,
        ),
        lo(180, 64, 80, 64, 32, DIR_UP, 15, 6),
    ],
    NOP_LAYER,
    // SCATTER: floating dust/particles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFFFFFF, 0xFFCCDD),
        lo(140, 60, 15, 0x43, 0, 0, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// 26. Rayleigh Sky - Physically-inspired atmosphere (ATMOSPHERE opcode)
const PRESET_RAYLEIGH_SKY: [[u64; 2]; 8] = [
    // RAMP: base sky gradient
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x2040A0, 0x404030),
        lo(140, 180, 160, 120, 0xA5, DIR_UP, 15, 15),
    ],
    // ATMOSPHERE (0x0E): Rayleigh variant (1) - blue zenith scatter
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x4080FF,
            0xFFB060,
        ),
        lo(200, 80, 128, 128, 64, 0xC0A0, 15, 0),
    ],
    // LOBE: sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 14, 0xFFFFE0, 0xFFF0C0),
        lo(220, 32, 0, 0, 0, 0xC0A0, 15, 12),
    ],
    NOP_LAYER,
    // DECAL: sun disk
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0xFFF8E0),
        lo(255, 0x02, 14, 0, 0, 0xC0A0, 15, 15),
    ],
    // FLOW: subtle clouds
    [
        hi(OP_FLOW, REGION_SKY, BLEND_LERP, 0, 0xFFFFFF, 0xE0E8F0),
        lo(40, 40, 80, 0x30, 48, DIR_UP, 15, 6),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 27. Tiled Floor - Ground plane textures (PLANE opcode)
const PRESET_TILED_FLOOR: [[u64; 2]; 8] = [
    // RAMP: interior colors
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x303040, 0x505060),
        lo(120, 60, 60, 70, 0xC8, DIR_UP, 15, 15),
    ],
    // LOBE: overhead light
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 10, 0xFFE8D0, 0xFFC8A0),
        lo(160, 20, 0, 0, 0, DIR_UP, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
    // PLANE (0x0F): Tiles variant (0) - checkerboard floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x808080,
            0x404040,
        ),
        lo(180, 64, 40, 128, 0, 0x8000, 12, 0),
    ],
    // GRID: wall panels
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 4, 0x606060, 0x404040),
        lo(80, 40, 10, 0x11, 0, 0, 12, 10),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 28. Moon Rise - Celestial body in sky (CELESTIAL opcode)
const PRESET_MOON_RISE: [[u64; 2]; 8] = [
    // RAMP: night sky
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x080818, 0x101020),
        lo(60, 20, 25, 35, 0xB5, DIR_UP, 15, 15),
    ],
    // CELESTIAL (0x10): Moon variant (0) - lunar body
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xE8E8D8,
            0x405060,
        ),
        lo(220, 24, 140, 80, 0, 0xB0D0, 15, 10),
    ],
    // LOBE: moonlight glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 6, 0x8090A0, 0x506070),
        lo(80, 16, 0, 0, 0, 0xB0D0, 15, 10),
    ],
    NOP_LAYER,
    // SCATTER: stars
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 12, 0xFFFFFF, 0xAABBFF),
        lo(200, 160, 16, 0x63, 0, 0, 15, 10),
    ],
    // SCATTER: dimmer distant stars
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 8, 0x8888AA, 0x444466),
        lo(120, 220, 8, 0x42, 0, 0, 10, 6),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 29. Portal Rift - Dimensional tear (PORTAL opcode)
const PRESET_PORTAL_RIFT: [[u64; 2]; 8] = [
    // RAMP: dark void
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x100818, 0x080410),
        lo(50, 10, 15, 20, 0xA5, DIR_UP, 15, 15),
    ],
    // LOBE: eerie ambient glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 4, 0x6020A0, 0x301050),
        lo(60, 12, 0, 0, 0, 0x7080, 12, 10),
    ],
    // PORTAL (0x11): Vortex variant (3) - swirling portal
    [
        hi_meta(
            OP_PORTAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x000000,
            0xAA00FF,
        ),
        lo(240, 80, 100, 160, 80, 0x7F00, 15, 14),
    ],
    NOP_LAYER,
    // SCATTER: energy particles around portal
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0xCC44FF, 0x8800AA),
        lo(180, 50, 25, 0xC5, 0, 0, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// 30. Split World - Geometric division (SPLIT opcode)
const PRESET_SPLIT_WORLD: [[u64; 2]; 8] = [
    // SPLIT (0x04): Half variant (0) - bisected world
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SPLIT_HALF,
            0x6080C0,
            0xC06040,
        ),
        lo(0, 40, 64, 8, 0, 0x4080, 0, 0),
    ],
    // LOBE: blue side glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0x4080FF, 0x2040AA),
        lo(100, 20, 0, 0, 0, 0x00C0, 15, 10),
    ],
    // LOBE: orange side glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 8, 0xFF8040, 0xAA4020),
        lo(100, 20, 0, 0, 0, 0xFF40, 15, 10),
    ],
    NOP_LAYER,
    // SCATTER: particles on both sides
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 6, 0xFFFFFF, 0xDDDDDD),
        lo(160, 80, 20, 0x53, 0, 0, 12, 8),
    ],
    // GRID: geometric lines
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 4, 0x404040, 0x303030),
        lo(60, 64, 8, 0x14, 0, 0, 10, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// 31. Silhouette City - Urban skyline cutout (SILHOUETTE opcode)
const PRESET_SILHOUETTE_CITY: [[u64; 2]; 8] = [
    // RAMP: sunset sky
    [
        hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xFF6030, 0x202030),
        lo(180, 200, 160, 100, 0xA5, DIR_UP, 15, 15),
    ],
    // SILHOUETTE (0x03): City variant (1) - urban skyline
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x000000,
            0x000000,
        ),
        lo(80, 128, 160, 0x42, 32, DIR_UP, 12, 0),
    ],
    // LOBE: setting sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 12, 0xFFAA40, 0xFF6600),
        lo(200, 28, 0, 0, 0, DIR_SUNSET, 15, 12),
    ],
    // BAND: warm horizon glow
    [
        hi(OP_BAND, REGION_SKY, BLEND_SCREEN, 6, 0xFF8040, 0x802010),
        lo(100, 100, 140, 64, 20, DIR_UP, 12, 8),
    ],
    // DECAL: sun disk
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFF6600, 0xFF4400),
        lo(255, 0x04, 20, 0, 0, DIR_SUNSET, 15, 15),
    ],
    // SCATTER: city lights starting to appear
    [
        hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 8, 0xFFAA44, 0xFF8822),
        lo(160, 80, 25, 0x23, 0, 0, 15, 12),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

// =============================================================================
// Preset Arrays
// =============================================================================

/// All presets array (31 presets - 20 v2 + 11 vNext)
pub static PRESETS: [[[u64; 2]; 8]; 31] = [
    // v2 presets (1-20)
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
    // vNext presets (21-31)
    PRESET_VORONOI_CELLS,
    PRESET_NOISE_PATCHES,
    PRESET_SHAPED_APERTURE,
    PRESET_LIGHTNING_TRACES,
    PRESET_CURTAIN_VEIL,
    PRESET_RAYLEIGH_SKY,
    PRESET_TILED_FLOOR,
    PRESET_MOON_RISE,
    PRESET_PORTAL_RIFT,
    PRESET_SPLIT_WORLD,
    PRESET_SILHOUETTE_CITY,
];

/// Preset names for display
pub const PRESET_NAMES: [&str; 31] = [
    // v2 presets (1-20)
    "Sunny Meadow",
    "Cyberpunk Alley (Rain)",
    "Void Stars",
    "Sunset Desert",
    "Underwater Cave",
    "Storm Front (Rain)",
    "Neon City",
    "Enchanted Forest",
    "Arctic Tundra (Snowfall)",
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
    // vNext presets (21-31)
    "[vNext] Voronoi Cells",
    "[vNext] Noise Patches",
    "[vNext] Shaped Aperture",
    "[vNext] Lightning Traces",
    "[vNext] Curtain Veil",
    "[vNext] Rayleigh Sky",
    "[vNext] Tiled Floor",
    "[vNext] Moon Rise",
    "[vNext] Portal Rift",
    "[vNext] Split World",
    "[vNext] Silhouette City",
];

pub const PRESET_COUNT: usize = 31;
