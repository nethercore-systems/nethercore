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

/// Unpack a layer's [hi, lo] into the EDITOR state
unsafe fn unpack_layer(_hi: u64, _lo: u64) {
    // TODO: implement in Task 3
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
