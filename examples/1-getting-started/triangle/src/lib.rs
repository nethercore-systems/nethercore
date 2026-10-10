//! One triangle: vertex order, camera side and model rotation are independent.
#![no_std]
#![no_main]
#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static TRIANGLE: [f32; 18] = [
    0.0, 1.0, 0.0, 1.0, 0.0, 0.0, -0.866, -0.5, 0.0, 0.0, 1.0, 0.0, 0.866, -0.5, 0.0, 0.0, 0.0, 1.0,
];
static mut REVERSE: u32 = 0;
static mut CAMERA_BACK: u32 = 0;
static mut ROTATION: u32 = 0;
static mut TWO_SIDED: u32 = 0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        for (name, ptr) in [
            ("reverse", core::ptr::addr_of!(REVERSE)),
            ("camera_back", core::ptr::addr_of!(CAMERA_BACK)),
            ("rotation", core::ptr::addr_of!(ROTATION)),
            ("two_sided", core::ptr::addr_of!(TWO_SIDED)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            REVERSE ^= 1;
        }
        if button_pressed(0, button::B) != 0 {
            CAMERA_BACK ^= 1;
        }
        if button_pressed(0, button::X) != 0 {
            ROTATION = (ROTATION + 1) % 3;
        }
        if button_pressed(0, button::L1) != 0 {
            TWO_SIDED ^= 1;
        }
        if button_pressed(0, button::Y) != 0 {
            REVERSE = 0;
            CAMERA_BACK = 0;
            ROTATION = 0;
            TWO_SIDED = 0;
        }
    }
}
unsafe fn text(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    set_color(color);
    draw_text(s.as_ptr(), s.len() as u32, x, y, size);
}
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        camera_set(
            0.0,
            0.0,
            if CAMERA_BACK == 0 { 3.0 } else { -3.0 },
            0.0,
            0.0,
            0.0,
        );
        camera_fov(60.0);
        cull_mode(if TWO_SIDED == 0 {
            cull::BACK
        } else {
            cull::NONE
        });
        push_identity();
        push_rotate_y(ROTATION as f32 * 45.0);
        let mut vertices = TRIANGLE;
        if REVERSE != 0 {
            for i in 0..6 {
                vertices.swap(6 + i, 12 + i);
            }
        }
        set_color(0xffffffff);
        draw_triangles(vertices.as_ptr(), 3, format::POS_COLOR as u32);

        text(
            b"ONE TRIANGLE / three independent choices",
            24.0,
            24.0,
            22.0,
            0xffffffff,
        );
        text(
            b"A: winding | B: camera side | X: rotate | L1: culling | Y: reset",
            24.0,
            66.0,
            14.0,
            0xabc1d6ff,
        );
        text(b"VERTICES (XYZ)", 24.0, 148.0, 15.0, 0xffffffff);
        text(b"R (0, 1, 0)", 24.0, 186.0, 16.0, 0xff9191ff);
        text(b"G (-.866, -.5, 0)", 24.0, 220.0, 15.0, 0x91ffadff);
        text(b"B (.866, -.5, 0)", 24.0, 254.0, 15.0, 0x91bdffff);
        text(
            if REVERSE == 0 {
                b"Order: R,G,B"
            } else {
                b"Order: R,B,G"
            },
            24.0,
            320.0,
            17.0,
            0xffffffff,
        );
        text(b"CAMERA / MODEL", 700.0, 148.0, 15.0, 0xffffffff);
        text(
            if CAMERA_BACK == 0 {
                b"Eye: (0,0,+3)"
            } else {
                b"Eye: (0,0,-3)"
            },
            700.0,
            186.0,
            16.0,
            0xffcd66ff,
        );
        text(b"Target: origin", 700.0, 220.0, 15.0, 0xabc1d6ff);
        text(b"FOV: 60 degrees", 700.0, 252.0, 15.0, 0xabc1d6ff);
        let rotations: [&[u8]; 3] = [b"Model: Y 0 deg", b"Model: Y 45 deg", b"Model: Y 90 deg"];
        text(rotations[ROTATION as usize], 700.0, 288.0, 15.0, 0xffffffff);
        text(
            if TWO_SIDED == 0 {
                b"Cull: back"
            } else {
                b"Cull: none"
            },
            700.0,
            324.0,
            17.0,
            0xffffffff,
        );
        text(
            b"Swap G/B to reverse winding. Back culling shows only the front face.",
            32.0,
            446.0,
            12.0,
            0xabc1d6ff,
        );
        text(
            b"Cull none shows either face. At Y=90 degrees the triangle is edge-on.",
            32.0,
            474.0,
            12.0,
            0xabc1d6ff,
        );
        text(
            b"Keys: Z/X/C/Q/V | F3: inspect | One triangle, no lighting.",
            32.0,
            510.0,
            12.0,
            0xffffffff,
        );
    }
}
