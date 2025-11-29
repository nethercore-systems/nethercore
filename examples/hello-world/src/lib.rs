#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

#[link(wasm_import_module = "emberware")]
extern "C" {
    fn clear(color: u32);
    fn frame_begin();
    fn frame_end();
    fn delta_time() -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

static mut Y_POS: f32 = 120.0;

#[no_mangle]
pub extern "C" fn init() {}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, 0) != 0 { Y_POS -= 10.0; } // UP
        if button_pressed(0, 1) != 0 { Y_POS += 10.0; } // DOWN
        if button_pressed(0, 4) != 0 { Y_POS = 120.0; } // A
        Y_POS = Y_POS.clamp(20.0, 200.0);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        frame_begin();
        clear(0x1a1a2eFF);
        let title = b"Hello Emberware!";
        draw_text(title.as_ptr(), title.len() as u32, 80.0, 30.0, 16.0, 0xFFFFFFFF);
        draw_rect(140.0, Y_POS, 40.0, 40.0, 0xFF6B6BFF);
        frame_end();
    }
}
