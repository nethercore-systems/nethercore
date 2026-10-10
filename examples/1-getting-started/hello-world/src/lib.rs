//! First cartridge: one square, edge-triggered movement, a bounded screen and reset.
//! The C and Zig versions deliberately implement exactly this same lesson.
#![no_std]
#![no_main]

#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut Y_POS: f32 = 240.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x172334ff);
        debug_watch_f32(b"square_y".as_ptr(), 8, core::ptr::addr_of!(Y_POS).cast());
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::UP) != 0 {
            Y_POS -= 10.0;
        }
        if button_pressed(0, button::DOWN) != 0 {
            Y_POS += 10.0;
        }
        if button_pressed(0, button::A) != 0 {
            Y_POS = 240.0;
        }
        Y_POS = Y_POS.clamp(160.0, 384.0);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        set_color(0xffffffff);
        let title = b"Hello Nethercore!";
        draw_text(title.as_ptr(), title.len() as u32, 48.0, 36.0, 32.0);
        set_color(0xabc1d6ff);
        let lesson = b"960 x 540 | top-left origin | x=448 | 64 x 64 | y=160..384";
        draw_text(lesson.as_ptr(), lesson.len() as u32, 48.0, 92.0, 14.0);
        set_color(0x344c66ff);
        draw_rect(96.0, 156.0, 768.0, 4.0);
        draw_rect(96.0, 448.0, 768.0, 4.0);
        set_color(0xff967aff);
        draw_rect(448.0, Y_POS, 64.0, 64.0);
        set_color(0xffffffff);
        let y = Y_POS as u32;
        let label = [
            b'y',
            b'=',
            b'0' + (y / 100) as u8,
            b'0' + ((y / 10) % 10) as u8,
            b'0' + (y % 10) as u8,
        ];
        draw_text(
            label.as_ptr(),
            label.len() as u32,
            532.0,
            Y_POS + 22.0,
            18.0,
        );
        let hint = b"D-pad Up/Down: step 10 pixels   A: reset";
        draw_text(hint.as_ptr(), hint.len() as u32, 48.0, 480.0, 18.0);
        set_color(0xabc1d6ff);
        let keys = b"Keyboard: arrows / Z   |   Hold does not repeat.   F3: inspect";
        draw_text(keys.as_ptr(), keys.len() as u32, 48.0, 512.0, 14.0);
    }
}
