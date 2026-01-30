//! EPU Inspector - Live EPU Editor Playground
//!
//! Debug-panel-driven editor for tweaking EPU layer values in real-time.
//! Press F4 to open the debug panel. Edit values and see immediate results.
//!
//! Features:
//! - Layer-by-layer editing (8 layers)
//! - Isolation mode to view single layers
//! - Export to hex for preset authoring

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

// ============================================================================
// Editor State - Unpacked layer values for debug panel editing
// ============================================================================

/// Unpacked representation of a single EPU layer for editing
#[derive(Clone, Copy)]
struct EditorState {
    // Hi word fields
    opcode: u8,
    region_sky: u8,      // bool as u8 for FFI
    region_walls: u8,
    region_floor: u8,
    blend: u8,
    domain_id: u8,
    variant_id: u8,
    color_a: u32,        // RGBA for color picker
    color_b: u32,

    // Lo word fields
    intensity: u8,
    param_a: u8,
    param_b: u8,
    param_c: u8,
    param_d: u8,
    azimuth: f32,        // 0-360 degrees
    elevation: f32,      // -90 to +90 degrees
    alpha_a: u8,
    alpha_b: u8,
}

impl EditorState {
    const fn default() -> Self {
        Self {
            opcode: 1,           // RAMP
            region_sky: 1,
            region_walls: 1,
            region_floor: 1,
            blend: 0,            // ADD
            domain_id: 0,
            variant_id: 0,
            color_a: 0x6496DCFF, // Sky blue
            color_b: 0x285028FF, // Ground green
            intensity: 180,
            param_a: 180,
            param_b: 165,
            param_c: 0,
            param_d: 128,
            azimuth: 0.0,
            elevation: 0.0,
            alpha_a: 15,
            alpha_b: 15,
        }
    }
}

// ============================================================================
// Global State
// ============================================================================

/// The 8-layer EPU configuration (16 u64 values = 128 bytes)
static mut LAYERS: [[u64; 2]; 8] = [[0; 2]; 8];

/// Current editor state (unpacked from selected layer)
static mut EDITOR: EditorState = EditorState::default();

/// Control state
static mut LAYER_INDEX: u8 = 1;      // 1-8 (user-facing)
static mut ISOLATE_LAYER: u8 = 0;    // bool
static mut SHOW_HINTS: u8 = 1;       // bool

/// Track previous layer index for change detection
static mut PREV_LAYER_INDEX: u8 = 1;

// ============================================================================
// Pack/Unpack Helpers
// ============================================================================

const PI: f32 = 3.14159265358979323846;

/// Convert octahedral-encoded direction to (azimuth, elevation) in degrees
fn octahedral_to_angles(dir16: u16) -> (f32, f32) {
    // Unpack: lo byte = u, hi byte = v (after byte swap in pack)
    let u_byte = (dir16 & 0xFF) as f32;
    let v_byte = ((dir16 >> 8) & 0xFF) as f32;

    // Convert to [-1, 1] range
    let u = u_byte / 127.5 - 1.0;
    let v = v_byte / 127.5 - 1.0;

    // Octahedral decode
    let z = 1.0 - libm::fabsf(u) - libm::fabsf(v);
    let (x, y) = if z >= 0.0 {
        (u, v)
    } else {
        // Reflect for lower hemisphere
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        ((1.0 - libm::fabsf(v)) * sign_u, (1.0 - libm::fabsf(u)) * sign_v)
    };

    // Normalize
    let len = libm::sqrtf(x * x + y * y + z * z);
    let (nx, ny, nz) = if len > 0.0001 {
        (x / len, y / len, z / len)
    } else {
        (0.0, 0.0, 1.0)
    };

    // Convert to spherical
    let elevation = libm::asinf(if nz > 1.0 { 1.0 } else if nz < -1.0 { -1.0 } else { nz }) * 180.0 / PI;
    let azimuth = libm::atan2f(ny, nx) * 180.0 / PI;
    let azimuth = if azimuth < 0.0 { azimuth + 360.0 } else { azimuth };

    (azimuth, elevation)
}

/// Convert (azimuth, elevation) in degrees to octahedral encoding
fn angles_to_octahedral(azimuth: f32, elevation: f32) -> u16 {
    let az_rad = azimuth * PI / 180.0;
    let el_rad = elevation * PI / 180.0;

    // Spherical to Cartesian
    let cos_el = libm::cosf(el_rad);
    let x = libm::cosf(az_rad) * cos_el;
    let y = libm::sinf(az_rad) * cos_el;
    let z = libm::sinf(el_rad);

    // Octahedral encode
    let sum = libm::fabsf(x) + libm::fabsf(y) + libm::fabsf(z);
    let (mut u, mut v) = if sum > 0.0001 {
        (x / sum, y / sum)
    } else {
        (0.0, 0.0)
    };

    if z < 0.0 {
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        let new_u = (1.0 - libm::fabsf(v)) * sign_u;
        let new_v = (1.0 - libm::fabsf(u)) * sign_v;
        u = new_u;
        v = new_v;
    }

    // Convert to [0, 255] range
    let u_byte = ((u + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
    let v_byte = ((v + 1.0) * 127.5).clamp(0.0, 255.0) as u8;

    // Pack: lo byte = u, hi byte = v
    (u_byte as u16) | ((v_byte as u16) << 8)
}

/// Unpack a layer's [hi, lo] into the EDITOR state
unsafe fn unpack_layer(hi: u64, lo: u64) {
    // Hi word extraction
    EDITOR.opcode = ((hi >> 59) & 0x1F) as u8;
    let region = ((hi >> 56) & 0x7) as u8;
    EDITOR.region_sky = if region & 0b100 != 0 { 1 } else { 0 };
    EDITOR.region_walls = if region & 0b010 != 0 { 1 } else { 0 };
    EDITOR.region_floor = if region & 0b001 != 0 { 1 } else { 0 };
    EDITOR.blend = ((hi >> 53) & 0x7) as u8;

    let meta_hi = ((hi >> 49) & 0xF) as u8;
    let meta_lo = ((hi >> 48) & 0x1) as u8;
    let meta5 = (meta_hi << 1) | meta_lo;
    EDITOR.domain_id = (meta5 >> 3) & 0x3;
    EDITOR.variant_id = meta5 & 0x7;

    // Colors: stored as RGB24, convert to RGBA for color picker
    let rgb_a = ((hi >> 24) & 0xFFFFFF) as u32;
    let rgb_b = (hi & 0xFFFFFF) as u32;
    EDITOR.color_a = (rgb_a << 8) | 0xFF;
    EDITOR.color_b = (rgb_b << 8) | 0xFF;

    // Lo word extraction
    EDITOR.intensity = ((lo >> 56) & 0xFF) as u8;
    EDITOR.param_a = ((lo >> 48) & 0xFF) as u8;
    EDITOR.param_b = ((lo >> 40) & 0xFF) as u8;
    EDITOR.param_c = ((lo >> 32) & 0xFF) as u8;
    EDITOR.param_d = ((lo >> 24) & 0xFF) as u8;

    // Direction: extract and convert to angles
    let dir_packed = ((lo >> 8) & 0xFFFF) as u16;
    let (az, el) = octahedral_to_angles(dir_packed);
    EDITOR.azimuth = az;
    EDITOR.elevation = el;

    EDITOR.alpha_a = ((lo >> 4) & 0xF) as u8;
    EDITOR.alpha_b = (lo & 0xF) as u8;
}

/// Pack the EDITOR state back into [hi, lo]
unsafe fn pack_layer() -> (u64, u64) {
    let opcode = (EDITOR.opcode as u64 & 0x1F) << 59;
    let region = ((if EDITOR.region_sky != 0 { 0b100u64 } else { 0 })
        | (if EDITOR.region_walls != 0 { 0b010 } else { 0 })
        | (if EDITOR.region_floor != 0 { 0b001 } else { 0 }))
        << 56;
    let blend = (EDITOR.blend as u64 & 0x7) << 53;

    let meta5 = ((EDITOR.domain_id & 0x3) << 3) | (EDITOR.variant_id & 0x7);
    let meta_hi = ((meta5 >> 1) & 0xF) as u64;
    let meta_lo = (meta5 & 0x1) as u64;
    let meta = (meta_hi << 49) | (meta_lo << 48);

    // Colors: RGBA -> RGB24
    let rgb_a = ((EDITOR.color_a >> 8) & 0xFFFFFF) as u64;
    let rgb_b = ((EDITOR.color_b >> 8) & 0xFFFFFF) as u64;
    let colors = (rgb_a << 24) | rgb_b;

    let hi = opcode | region | blend | meta | colors;

    // Lo word
    let intensity = (EDITOR.intensity as u64) << 56;
    let param_a = (EDITOR.param_a as u64) << 48;
    let param_b = (EDITOR.param_b as u64) << 40;
    let param_c = (EDITOR.param_c as u64) << 32;
    let param_d = (EDITOR.param_d as u64) << 24;

    let dir16 = angles_to_octahedral(EDITOR.azimuth, EDITOR.elevation);
    let direction = (dir16 as u64) << 8;

    let alpha_a = (EDITOR.alpha_a as u64 & 0xF) << 4;
    let alpha_b = EDITOR.alpha_b as u64 & 0xF;

    let lo = intensity | param_a | param_b | param_c | param_d | direction | alpha_a | alpha_b;

    (hi, lo)
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Initialize layer 0 with a simple RAMP (sky gradient)
        // hi: opcode=1(RAMP), region=7(ALL), blend=0(ADD), meta5=0, colors
        // lo: intensity, params, direction, alphas
        LAYERS[0] = [
            0x0F00_6496_DC28_5028, // RAMP, ALL, ADD, sky-blue / ground-green
            0xB4B4_A500_8000_FF,   // intensity=180, param_a=180, param_b=165, dir=center, alpha=15/15
        ];

        // Unpack layer 0 into editor state
        unpack_layer(LAYERS[0][0], LAYERS[0][1]);
    }
}

#[no_mangle]
pub extern "C" fn update() {}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        camera_set(0.0, 0.0, 5.0, 0.0, 0.0, 0.0);
        draw_epu();
    }
}
