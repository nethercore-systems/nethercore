//! Hello World Example
//!
//! Demonstrates basic 2D drawing with text and rectangles.
//! Use D-pad to move the square, A button to reset position.
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn button_pressed(player: u32, button: u32) -> u32;
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

static mut Y_POS: f32 = 120.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue-gray background
        set_clear_color(0x1a1a2eFF);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, 0) != 0 {
            Y_POS -= 10.0;
        } // UP
        if button_pressed(0, 1) != 0 {
            Y_POS += 10.0;
        } // DOWN
        if button_pressed(0, 4) != 0 {
            Y_POS = 120.0;
        } // A
        Y_POS = Y_POS.clamp(20.0, 200.0);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let title = b"Hello Nethercore!";
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            80.0,
            30.0,
            24.0,
            0xFFFFFFFF,
        );
        draw_rect(140.0, Y_POS, 40.0, 40.0, 0xFF6B6BFF);
    }
}
