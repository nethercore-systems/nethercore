//! Anchor pixels, UV subregions and a deliberately explicit two-link hierarchy.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod geometry;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut TEXTURE: u32 = 0;
static mut STEP: u32 = 0;
static mut FLIP: u32 = 0;
static mut REGION: u32 = 0;
static mut ELBOW_X: f32 = 870.0;
static mut ELBOW_Y: f32 = 270.0;
static mut TIP_X: f32 = 870.0;
static mut TIP_Y: f32 = 330.0;

const fn atlas() -> [u8; 32 * 8 * 4] {
    let mut pixels = [0; 32 * 8 * 4];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 32 {
            let lx = x % 16;
            let shaft = lx < 11 && (y == 3 || y == 4);
            let half = if y < 4 { 3 - y } else { y - 4 };
            let head = lx >= 10 && lx <= 15 - half;
            let notch = lx == 2 && (y == 1 || y == 2);
            if shaft || head || notch {
                let color = if notch {
                    [240, 65, 110, 255]
                } else if x < 16 {
                    [50, 205, 225, 255]
                } else {
                    [255, 200, 40, 255]
                };
                let mut c = 0;
                while c < 4 {
                    pixels[(y * 32 + x) * 4 + c] = color[c];
                    c += 1;
                }
            }
            x += 1;
        }
        y += 1;
    }
    pixels
}
const PIXELS: [u8; 32 * 8 * 4] = atlas();

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        TEXTURE = load_texture(32, 8, PIXELS.as_ptr());
        for (name, ptr) in [
            ("step", core::ptr::addr_of!(STEP)),
            ("flip", core::ptr::addr_of!(FLIP)),
            ("region", core::ptr::addr_of!(REGION)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        for (name, ptr) in [
            ("elbow_x", core::ptr::addr_of!(ELBOW_X)),
            ("elbow_y", core::ptr::addr_of!(ELBOW_Y)),
            ("tip_x", core::ptr::addr_of!(TIP_X)),
            ("tip_y", core::ptr::addr_of!(TIP_Y)),
        ] {
            debug_watch_f32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            STEP = (STEP + 1) % 8;
        }
        if button_pressed(0, button::B) != 0 {
            FLIP ^= 1;
        }
        if button_pressed(0, button::X) != 0 {
            REGION ^= 1;
        }
        if button_pressed(0, button::Y) != 0 {
            STEP = 0;
            FLIP = 0;
            REGION = 0;
        }
        let (elbow, tip) = geometry::joints(STEP, FLIP);
        ELBOW_X = elbow.0;
        ELBOW_Y = elbow.1;
        TIP_X = tip.0;
        TIP_Y = tip.1;
    }
}

unsafe fn marker(x: f32, y: f32) {
    set_color(0xffffffff);
    draw_line(x - 6.0, y, x + 6.0, y, 2.0);
    draw_line(x, y - 6.0, x, y + 6.0, 2.0);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let step = STEP;
        let flip = FLIP;
        let region = REGION;
        let (elbow, tip) = geometry::joints(step, flip);
        let angle = step as f32 * 45.0;
        let child = angle + if flip == 0 { 90.0 } else { -90.0 };
        z_index(0);
        set_color(0x202f44ff);
        draw_rect(24.0, 108.0, 288.0, 340.0);
        draw_rect(320.0, 108.0, 328.0, 340.0);
        draw_rect(656.0, 108.0, 280.0, 340.0);
        set_color(0x536a83ff);
        draw_circle_outline(176.0, 270.0, 80.0, 1.0);
        draw_circle_outline(480.0, 270.0, 160.0, 1.0);
        draw_line(790.0, 270.0, elbow.0, elbow.1, 1.0);
        draw_line(elbow.0, elbow.1, tip.0, tip.1, 1.0);
        z_index(1);
        texture_bind(TEXTURE);
        texture_filter(0);
        set_color(0xffffffff);
        let u = region as f32 * 0.5;
        // x/y is the chosen SCREEN PIVOT, not a pre-rotated top-left corner.
        draw_sprite_ex(
            176.0, 270.0, 160.0, 80.0, u, 0.0, 0.5, 1.0, 80.0, 40.0, angle,
        );
        draw_sprite_ex(
            480.0, 270.0, 160.0, 80.0, u, 0.0, 0.5, 1.0, 0.0, 40.0, angle,
        );
        draw_sprite_ex(790.0, 270.0, 80.0, 40.0, u, 0.0, 0.5, 1.0, 0.0, 20.0, angle);
        draw_sprite_ex(
            elbow.0, elbow.1, 60.0, 30.0, u, 0.0, 0.5, 1.0, 0.0, 15.0, child,
        );
        z_index(2);
        marker(176.0, 270.0);
        marker(480.0, 270.0);
        marker(790.0, 270.0);
        marker(elbow.0, elbow.1);
        marker(tip.0, tip.1);
        hud::text(
            32.0,
            20.0,
            24.0,
            0xffffffff,
            format_args!("SPRITE PIVOTS / origin is an anchor"),
        );
        hud::text(
            32.0,
            58.0,
            12.0,
            0xabc1d6ff,
            format_args!("Pad/key: A/Z +45deg | B/X child turn | X/C UV tile | Y/V reset"),
        );
        hud::text(52.0, 84.0, 14.0, 0xffcd66ff, format_args!("CENTER (80,40)"));
        hud::text(360.0, 84.0, 14.0, 0xffcd66ff, format_args!("EDGE (0,40)"));
        hud::text(
            676.0,
            84.0,
            14.0,
            0xffcd66ff,
            format_args!("PARENT + CHILD"),
        );
        hud::text(
            32.0,
            458.0,
            12.0,
            0xffffffff,
            format_args!("Angle {} / UV {}", angle as u32, region),
        );
        hud::text(
            336.0,
            458.0,
            12.0,
            0xffffffff,
            format_args!("Anchor 480,270"),
        );
        hud::text(
            660.0,
            458.0,
            11.0,
            0xffffffff,
            format_args!("Elbow {:.1},{:.1}", elbow.0, elbow.1),
        );
        hud::text(
            660.0,
            480.0,
            11.0,
            0xffffffff,
            format_args!("Tip {:.1},{:.1}", tip.0, tip.1),
        );
        hud::text(
            32.0,
            493.0,
            11.0,
            0xabc1d6ff,
            format_args!(
                "Screen pivot x,y. Origin = local sprite pixels. White crosses mark anchors."
            ),
        );
        hud::text(
            32.0,
            518.0,
            10.0,
            0xabc1d6ff,
            format_args!(
                "F3 state | Two arm links use explicit guest coordinates, not a scene graph."
            ),
        );
    }
}
