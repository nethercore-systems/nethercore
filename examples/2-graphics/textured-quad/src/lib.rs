//! An asymmetric labeled atlas separates UV orientation from per-draw sampling.
#![no_std]
#![no_main]
use examples_common::{checkerboard_8x8, ffi::*, hud};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut TEXTURE: u32 = 0;
static mut FILTER: u32 = 0;
static mut REGION: u32 = 0;
static mut INSET: u32 = 0;

const fn atlas() -> [u8; 1024] {
    let tiles = [
        checkerboard_8x8(0xe63636ff, 0x5a1010ff),
        checkerboard_8x8(0x36dd66ff, 0x105020ff),
        checkerboard_8x8(0x3677ffff, 0x102050ff),
        checkerboard_8x8(0xffd336ff, 0x605010ff),
    ];
    let digits = [
        [2, 6, 2, 2, 7],
        [6, 1, 2, 4, 7],
        [6, 1, 2, 1, 6],
        [5, 5, 7, 1, 1],
    ];
    let mut pixels = [0u8; 1024];
    let mut y = 0;
    while y < 16 {
        let mut x = 0;
        while x < 16 {
            let tile = (y / 8) * 2 + x / 8;
            let lx = x % 8;
            let ly = y % 8;
            let ink = lx >= 2
                && lx <= 4
                && ly >= 1
                && ly <= 5
                && (digits[tile][ly - 1] & (1 << (4 - lx))) != 0;
            let mut c = 0;
            while c < 4 {
                pixels[(y * 16 + x) * 4 + c] = if ink {
                    255
                } else {
                    tiles[tile][(ly * 8 + lx) * 4 + c]
                };
                c += 1;
            }
            x += 1;
        }
        y += 1;
    }
    pixels
}
const ATLAS: [u8; 1024] = atlas();

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        TEXTURE = load_texture(16, 16, ATLAS.as_ptr());
        for (name, ptr) in [
            ("filter", core::ptr::addr_of!(FILTER)),
            ("region", core::ptr::addr_of!(REGION)),
            ("inset", core::ptr::addr_of!(INSET)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            FILTER ^= 1;
        }
        if button_pressed(0, button::B) != 0 {
            REGION = (REGION + 1) % 5;
        }
        if button_pressed(0, button::L1) != 0 {
            INSET ^= 1;
        }
        if button_pressed(0, button::Y) != 0 {
            FILTER = 0;
            REGION = 0;
            INSET = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let filter = FILTER;
        let region = REGION;
        let inset = INSET;
        // Rebind explicitly: text/shape helpers may select a different texture.
        texture_bind(TEXTURE);
        texture_filter(0);
        set_color(0xffffffff);
        draw_sprite(64.0, 154.0, 320.0, 320.0);
        texture_filter(filter);
        let (mut u, mut v, mut width) = if region == 0 {
            (0.0, 0.0, 1.0)
        } else {
            (
                ((region - 1) % 2) as f32 * 0.5,
                ((region - 1) / 2) as f32 * 0.5,
                0.5,
            )
        };
        let mut height = width;
        if inset != 0 {
            // Half a texel at each edge; deliberately visible, not hidden in a helper.
            u += 0.5 / 16.0;
            v += 0.5 / 16.0;
            width -= 1.0 / 16.0;
            height -= 1.0 / 16.0;
        }
        draw_sprite_region(520.0, 154.0, 320.0, 320.0, u, v, width, height);
        texture_filter(0); // Restore nearest for labels; the preceding draws keep their own state.
        hud::text(
            32.0,
            24.0,
            24.0,
            0xffffffff,
            format_args!("TEXTURED QUAD / UV and filtering"),
        );
        hud::text(
            32.0,
            66.0,
            14.0,
            0xabc1d6ff,
            format_args!("A: filter | B: UV region | L1: half-texel inset | Y: reset"),
        );
        hud::text(
            64.0,
            116.0,
            17.0,
            0xffcd66ff,
            format_args!("REFERENCE / nearest / full"),
        );
        hud::text(
            520.0,
            102.0,
            16.0,
            0xffcd66ff,
            format_args!(
                "SUBJECT {} / tile {}",
                if filter == 0 { "nearest" } else { "linear" },
                region
            ),
        );
        hud::text(
            520.0,
            128.0,
            12.0,
            0xabc1d6ff,
            format_args!("UV {:.3},{:.3} + {:.3},{:.3}", u, v, width, height),
        );
        hud::text(
            32.0,
            488.0,
            12.0,
            0xffffffff,
            format_args!(
                "1 red TL | 2 green TR | 3 blue BL | 4 gold BR | Inset {}",
                inset
            ),
        );
        hud::text(
            32.0,
            516.0,
            11.0,
            0xabc1d6ff,
            format_args!(
                "Keys Z/X/Q/V | F3 state | Linear tile edges can bleed; inset excludes neighbors."
            ),
        );
    }
}
