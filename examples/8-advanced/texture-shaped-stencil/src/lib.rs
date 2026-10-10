//! The real sprite fragment's alpha/dither discard defines stencil coverage.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod state;
use state::State;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
static mut STATE: State = State::RESET;
static mut TEXTURE: u32 = 0;
const PIXELS: [u8; 4096] = state::pixels();

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x101824ff);
        TEXTURE = load_texture(32, 32, PIXELS.as_ptr());
        assert!(TEXTURE != 0);
        for (name, ptr) in [
            ("alpha", core::ptr::addr_of!(STATE.alpha)),
            ("offset", core::ptr::addr_of!(STATE.offset)),
            ("box_mask", core::ptr::addr_of!(STATE.box_mask)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let mut pressed = 0;
        for key in [button::A, button::B, button::X, button::Y] {
            if button_pressed(0, key) != 0 {
                pressed |= 1 << key;
            }
        }
        (&mut *core::ptr::addr_of_mut!(STATE)).step(pressed);
    }
}

unsafe fn sprite(state: State, color: u32) {
    texture_bind(TEXTURE);
    texture_filter(0);
    uniform_alpha(state.level());
    dither_offset(state.offset, (state.offset * 2) & 3);
    set_color(color);
    draw_sprite(68.0, 16.0, 320.0, 320.0);
}

unsafe fn panel(state: State, index: u32) {
    viewport(16 + index * 472, 120, 456, 352);
    begin_pass(0);
    z_index(0);
    uniform_alpha(15);
    dither_offset(0, 0);
    set_color(0x18202cff);
    draw_rect(0.0, 0.0, 456.0, 352.0);
    if index == 0 {
        sprite(state, 0x40c8f0ff);
    } else {
        // Rasterized reset: no separate stencil-clear API is assumed.
        begin_pass_full(
            compare::ALWAYS,
            0,
            0,
            compare::ALWAYS,
            0,
            stencil_op::ZERO,
            stencil_op::KEEP,
            stencil_op::KEEP,
        );
        draw_rect(0.0, 0.0, 456.0, 352.0);
        // ALWAYS + REPLACE writes stencil, not color. Fragment discard still
        // runs: transparent texels and failed Bayer samples write no reference.
        begin_pass_stencil_write(1, 0);
        if state.box_mask == 0 {
            sprite(state, 0xffffffff);
        } else {
            uniform_alpha(state.level());
            dither_offset(state.offset, (state.offset * 2) & 3);
            set_color(0xffffffff);
            draw_rect(68.0, 16.0, 320.0, 320.0);
        }
        // Opaque, unrelated content. Do not dither again after writing the mask.
        uniform_alpha(15);
        dither_offset(0, 0);
        begin_pass_stencil_test(1, 0);
        set_color(0xf0b040ff);
        draw_rect(-80.0, -80.0, 620.0, 520.0);
        // Separate pass preserves cover-before-stripe order despite batching.
        begin_pass_stencil_test(1, 0);
        set_color(0xc060e0ff);
        for stripe in 0..8 {
            draw_rect(stripe as f32 * 64.0, 0.0, 32.0, 352.0);
        }
    }
    uniform_alpha(15);
    dither_offset(0, 0);
    begin_pass(0);
    z_index(2);
    set_color(0xccccccff);
    for [x, y, w, h] in [
        [0.0, 0.0, 456.0, 2.0],
        [0.0, 350.0, 456.0, 2.0],
        [0.0, 2.0, 2.0, 348.0],
        [454.0, 2.0, 2.0, 348.0],
    ] {
        draw_rect(x, y, w, h);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let state = STATE;
        panel(state, 0);
        panel(state, 1);
        viewport_clear();
        begin_pass(0);
        z_index(3);
        hud::text(
            16.0,
            12.0,
            23.0,
            0xffffffff,
            format_args!("TEXTURE-SHAPED STENCIL"),
        );
        hud::text(
            16.0,
            48.0,
            12.0,
            0xffffffff,
            format_args!("A opacity | B dither phase | X box counterexample | Y reset"),
        );
        hud::text(
            16.0,
            74.0,
            11.0,
            0xccccccff,
            format_args!(
                "Alpha {}/15 | offset ({},{}) | {} mask",
                state.level(),
                state.offset,
                (state.offset * 2) & 3,
                if state.box_mask == 0 {
                    "SPRITE"
                } else {
                    "BAD BOX"
                }
            ),
        );
        hud::text(
            16.0,
            98.0,
            14.0,
            0xffffffff,
            format_args!("SOURCE / alpha + dither"),
        );
        hud::text(
            488.0,
            98.0,
            14.0,
            0xffffffff,
            format_args!("MASK / different content"),
        );
        hud::text(
            16.0,
            491.0,
            10.0,
            0xffffffff,
            format_args!(
                "Cyan sprite writes only surviving fragments; gold/purple cover tests stencil 1."
            ),
        );
        hud::text(
            16.0,
            517.0,
            10.0,
            0xccccccff,
            format_args!(
                "Holes stay empty. Fractional edges are screen-door coverage, not smooth blending."
            ),
        );
    }
}
