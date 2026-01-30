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

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
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
