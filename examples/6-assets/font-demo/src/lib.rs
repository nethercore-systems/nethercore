//! Real ROM-backed fixed/variable text, with bounded guest layout rather than a UI framework.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod layout;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const WIDTHS: &[u8; 95] = include_bytes!("../assets/widths.bin");
const SAMPLE: &str = "WWW ill";
const PARAGRAPH: &str = "Wide WWW and tiny ill wrap into a menu.";
const MISSING: &str = "AΩB";
#[derive(Clone, Copy)]
struct State {
    variable: u32,
    alignment: u32,
    narrow: u32,
    focus: u32,
}
impl State {
    const fn new() -> Self {
        Self {
            variable: 0,
            alignment: 0,
            narrow: 0,
            focus: 0,
        }
    }
}
static mut STATE: State = State::new();
static mut FONTS: [u32; 2] = [0; 2];
static mut FONTS_OK: u32 = 0;
static mut SAMPLE_WIDTH: f32 = 0.0;
static mut SAMPLE_X: f32 = 0.0;
static mut RAW_WIDTH: f32 = 0.0;
static mut FALLBACK_WIDTH: f32 = 0.0;
static mut LINE_COUNT: u32 = 0;

unsafe fn width(text: &str, size: f32) -> f32 {
    text_width(text.as_ptr(), text.len() as u32, size)
}
unsafe fn text(text: &str, x: f32, y: f32, size: f32, color: u32) {
    set_color(color);
    draw_text(text.as_ptr(), text.len() as u32, x, y, size);
}
fn wrap_width(state: State) -> f32 {
    if state.narrow == 0 {
        280.0
    } else {
        160.0
    }
}
unsafe fn measure() {
    let state = STATE;
    font_bind(FONTS[state.variable as usize]);
    SAMPLE_WIDTH = width(SAMPLE, 16.0);
    SAMPLE_X = layout::aligned_x(48.0, 528.0, SAMPLE_WIDTH, state.alignment);
    RAW_WIDTH = width(MISSING, 24.0);
    FALLBACK_WIDTH = width("A?B", 24.0);
    let mut remaining = PARAGRAPH;
    LINE_COUNT = 0;
    while !remaining.is_empty() {
        layout::next_line(&mut remaining, wrap_width(state), |s| width(s, 16.0));
        LINE_COUNT += 1;
    }
    font_bind(0);
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        let atlas = rom_texture(b"font_atlas".as_ptr(), 10);
        FONTS = [
            load_font(atlas, 8, 8, 32, 95),
            load_font_ex(atlas, WIDTHS.as_ptr(), 8, 32, 95),
        ];
        FONTS_OK = u32::from(FONTS[0] != 0 && FONTS[1] != 0 && FONTS[0] != FONTS[1]);
        for (name, ptr) in [
            ("variable", core::ptr::addr_of!(STATE.variable)),
            ("alignment", core::ptr::addr_of!(STATE.alignment)),
            ("narrow", core::ptr::addr_of!(STATE.narrow)),
            ("focus", core::ptr::addr_of!(STATE.focus)),
            ("fonts_ok", core::ptr::addr_of!(FONTS_OK)),
            ("lines", core::ptr::addr_of!(LINE_COUNT)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        for (name, ptr) in [
            ("sample_width", core::ptr::addr_of!(SAMPLE_WIDTH)),
            ("sample_x", core::ptr::addr_of!(SAMPLE_X)),
            ("raw_width", core::ptr::addr_of!(RAW_WIDTH)),
            ("fallback_width", core::ptr::addr_of!(FALLBACK_WIDTH)),
        ] {
            debug_watch_f32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        measure();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::Y) != 0 {
            STATE = State::new();
        } else {
            if button_pressed(0, button::A) != 0 {
                STATE.variable ^= 1;
            }
            if button_pressed(0, button::B) != 0 {
                STATE.alignment = (STATE.alignment + 1) % 3;
            }
            if button_pressed(0, button::X) != 0 {
                STATE.narrow ^= 1;
            }
            if button_pressed(0, button::DOWN) != 0 {
                STATE.focus = (STATE.focus + 1) % 3;
            }
            if button_pressed(0, button::UP) != 0 {
                STATE.focus = (STATE.focus + 2) % 3;
            }
        }
        measure();
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let state = STATE;
        let (line_count, raw_width, fallback_width) = (LINE_COUNT, RAW_WIDTH, FALLBACK_WIDTH);
        z_index(0);
        set_color(0x202f44ff);
        for (x, y, w, h) in [
            (32.0, 100.0, 560.0, 168.0),
            (620.0, 100.0, 308.0, 168.0),
            (32.0, 280.0, 400.0, 176.0),
            (452.0, 280.0, 476.0, 176.0),
        ] {
            draw_rect(x, y, w, h);
        }
        set_color(0x345f50ff);
        draw_rect(632.0, 130.0 + state.focus as f32 * 36.0, 284.0, 28.0);
        set_color(0x64d9b2ff);
        draw_rect(SAMPLE_X, 250.0, SAMPLE_WIDTH, 2.0);
        draw_rect(48.0 + wrap_width(state), 320.0, 1.0, 126.0);
        for x in [48.0, 312.0, 576.0] {
            draw_rect(x, 254.0, 1.0, 8.0);
        }
        z_index(1);
        font_bind(FONTS[state.variable as usize]);
        text("ABCDEFGHIJKLMNOPQRSTUVWXYZ", 48.0, 132.0, 16.0, 0xffffffff);
        text("abcdefghijklmnopqrstuvwxyz", 48.0, 162.0, 16.0, 0xffffffff);
        text("0123456789 !?.,:[]", 48.0, 192.0, 16.0, 0xffffffff);
        text(SAMPLE, SAMPLE_X, 230.0, 16.0, 0x65deffff);
        for (i, label) in ["New game", "Options", "Credits"].iter().enumerate() {
            text(label, 658.0, 136.0 + i as f32 * 36.0, 16.0, 0xffffffff);
        }
        text(
            ">",
            636.0,
            136.0 + state.focus as f32 * 36.0,
            16.0,
            0x65deffff,
        );
        let mut remaining = PARAGRAPH;
        let mut line = 0;
        while !remaining.is_empty() {
            let value = layout::next_line(&mut remaining, wrap_width(state), |s| width(s, 16.0));
            text(value, 48.0, 322.0 + line as f32 * 22.0, 16.0, 0x65deffff);
            line += 1;
        }
        text(MISSING, 700.0, 326.0, 24.0, 0x65deffff);
        let mut replaced = [0; 8];
        let n = layout::fallback(MISSING, &mut replaced);
        set_color(0x65deffff);
        draw_text(replaced.as_ptr(), n as u32, 700.0, 366.0, 24.0);
        z_index(2);
        font_bind(0);
        hud::text(
            32.0,
            22.0,
            24.0,
            0xffffffff,
            format_args!("FONT DEMO / CUSTOM TEXT"),
        );
        hud::text(
            32.0,
            62.0,
            14.0,
            0xffcd66ff,
            format_args!(
                "{} | {} | WIDTH {:.0} | {} LINES | FOCUS {}",
                ["FIXED", "VARIABLE"][state.variable as usize],
                ["LEFT", "CENTER", "RIGHT"][state.alignment as usize],
                wrap_width(state),
                line_count,
                state.focus + 1
            ),
        );
        text(
            "ASCII / ROM TEXTURE + METRICS",
            48.0,
            108.0,
            12.0,
            0xffcd66ff,
        );
        text("MENU / FOCUS ONLY", 636.0, 108.0, 12.0, 0xffcd66ff);
        text(
            "GUEST WRAP / MEASURE EACH WORD",
            48.0,
            294.0,
            12.0,
            0xffcd66ff,
        );
        text("MISSING GLYPH / U+03A9", 470.0, 294.0, 12.0, 0xffcd66ff);
        text("NATIVE: SKIP", 470.0, 330.0, 14.0, 0xffffffff);
        text("GUEST: USE ?", 470.0, 370.0, 14.0, 0xffffffff);
        hud::text(
            470.0,
            414.0,
            12.0,
            0xffcd66ff,
            format_args!("WIDTHS: {:.0} / {:.0} PX", raw_width, fallback_width),
        );
        text(
            "[A] Font  [B] Align  [X] Wrap width  [Up/Down] Focus  [Y] Reset",
            32.0,
            480.0,
            14.0,
            0xffffffff,
        );
        text(
            "CUSTOM FONT: UNSUPPORTED CODEPOINTS HAVE ZERO ADVANCE. NO NATIVE WRAP.",
            32.0,
            514.0,
            12.0,
            0xb5bfd0ff,
        );
    }
}
