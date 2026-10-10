//! Full-byte stencil union, intersection, subtraction and parity.
#![no_std]
#![no_main]
use core::panic::PanicInfo;
use examples_common::{ffi::*, hud};
mod state;
use state::State;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut STATE: State = State::RESET;
const BACKGROUND: u32 = 0x18202cff;
const COLORS: [u32; 3] = [0x40c8f0ff, 0xf0b040ff, 0xc060e0ff];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x101824ff);
        for (name, ptr) in [
            ("mode", core::ptr::addr_of!(STATE.mode)),
            ("layout", core::ptr::addr_of!(STATE.layout)),
            ("swap", core::ptr::addr_of!(STATE.swap)),
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

unsafe fn rect([x, y, w, h]: [f32; 4]) {
    draw_rect(x, y, w, h);
}

unsafe fn stencil(compare: u32, reference: u32, operation: u32) {
    begin_pass_full(
        compare::ALWAYS,
        0,
        0,
        compare,
        reference,
        operation,
        stencil_op::KEEP,
        stencil_op::KEEP,
    );
}

unsafe fn result(state: State) {
    let [a, b] = state.shapes();
    // Rasterized ZERO, not a nonexistent independent stencil-clear API.
    stencil(compare::ALWAYS, 0, stencil_op::ZERO);
    draw_rect(0.0, 0.0, 288.0, 320.0);
    match state.mode {
        0 => {
            begin_pass_stencil_write(1, 0);
            rect(a);
            rect(b);
        }
        1 => {
            begin_pass_stencil_write(1, 0);
            rect(a);
            // Compare and REPLACE share one reference, so EQUAL 1 cannot
            // replace with 2. Conditional INCREMENT produces the intersection.
            // This pass CAN write color: match the background before final ink.
            stencil(compare::EQUAL, 1, stencil_op::INCREMENT_CLAMP);
            set_color(BACKGROUND);
            rect(b);
        }
        2 => {
            begin_pass_stencil_write(1, 0);
            rect(a);
            stencil(compare::ALWAYS, 0, stencil_op::ZERO);
            rect(b);
        }
        _ => {
            // Each shape is one quad: tessellated overlapping circle strips
            // would invert some pixels repeatedly and change the intended set.
            stencil(compare::ALWAYS, 0, stencil_op::INVERT);
            rect(a);
            rect(b);
        }
    }
    let reference = match state.mode {
        1 => 2,
        3 => 255,
        _ => 1,
    };
    stencil(compare::EQUAL, reference, stencil_op::KEEP);
    set_color(COLORS[2]);
    draw_rect(0.0, 0.0, 288.0, 320.0);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let state = STATE;
        let pair = state.shapes();
        for panel in 0..3 {
            viewport(24 + panel * 312, 130, 288, 320);
            begin_pass(0);
            z_index(0);
            set_color(BACKGROUND);
            draw_rect(0.0, 0.0, 288.0, 320.0);
            if panel == 2 {
                result(state);
            } else {
                set_color(COLORS[panel as usize]);
                rect(pair[panel as usize]);
            }
            begin_pass(0); // Border is not part of the mask or its stencil test.
            z_index(2);
            set_color(COLORS[panel as usize]);
            draw_rect(0.0, 0.0, 288.0, 2.0);
            draw_rect(0.0, 318.0, 288.0, 2.0);
            draw_rect(0.0, 2.0, 2.0, 316.0);
            draw_rect(286.0, 2.0, 2.0, 316.0);
        }
        viewport_clear();
        begin_pass(0);
        z_index(3);
        hud::text(
            24.0,
            14.0,
            24.0,
            0xffffffff,
            format_args!("STENCIL BOOLEAN LAB"),
        );
        hud::text(
            24.0,
            52.0,
            13.0,
            0xffffffff,
            format_args!("A operation | B overlap/nested/disjoint | X swap operands | Y reset"),
        );
        hud::text(
            24.0,
            84.0,
            12.0,
            0xccccccff,
            format_args!(
                "{} | {} | full 8-bit stencil, no per-bit masks",
                [
                    "UNION",
                    "INTERSECTION",
                    "FIRST minus SECOND",
                    "PARITY / XOR"
                ][state.mode as usize],
                ["OVERLAP", "NESTED", "DISJOINT"][state.layout as usize]
            ),
        );
        hud::text(
            24.0,
            108.0,
            14.0,
            COLORS[0],
            format_args!("FIRST ({})", if state.swap == 0 { "A" } else { "B" }),
        );
        hud::text(
            336.0,
            108.0,
            14.0,
            COLORS[1],
            format_args!("SECOND ({})", if state.swap == 0 { "B" } else { "A" }),
        );
        hud::text(
            648.0,
            108.0,
            14.0,
            COLORS[2],
            format_args!("STENCIL RESULT"),
        );
        hud::text(
            24.0,
            468.0,
            12.0,
            0xffffffff,
            format_args!(
                "{}",
                [
                    "ZERO, REPLACE 1 twice; test 1",
                    "ZERO, REPLACE 1; EQUAL 1 + INCREMENT; test 2",
                    "ZERO, REPLACE 1; ZERO second; test 1",
                    "ZERO, INVERT twice; test 255 (overlap returns to 0)"
                ][state.mode as usize]
            ),
        );
        hud::text(
            24.0,
            495.0,
            11.0,
            0xccccccff,
            format_args!(
                "Build masks first. Conditional writes can paint color: match the background."
            ),
        );
        hud::text(
            24.0,
            519.0,
            11.0,
            0xccccccff,
            format_args!(
                "Single-quad shapes avoid accidental parity flips from overlapping tessellation."
            ),
        );
    }
}
