//! Nested full-reference masks with explicit pop/restore and sibling reuse.
#![no_std]
#![no_main]
use core::panic::PanicInfo;
use examples_common::{ffi::*, hud};
mod state;
use state::{Rect, State, MAX_REFERENCE};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut STATE: State = State::RESET;
const COLORS: [u32; 4] = [0x40c8f0ff, 0xf0b040ff, 0xc060e0ff, 0x70d880ff];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x101824ff);
        for (name, ptr) in [
            ("deep", core::ptr::addr_of!(STATE.deep)),
            ("cleanup", core::ptr::addr_of!(STATE.cleanup)),
            ("paused", core::ptr::addr_of!(STATE.paused)),
            ("tick", core::ptr::addr_of!(STATE.tick)),
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

unsafe fn rect([x, y, w, h]: Rect) {
    draw_rect(x, y, w, h);
}

unsafe fn push(reference: u32, region: Rect) {
    begin_pass_stencil_write(reference, 0);
    rect(region);
}

unsafe fn clear(region: Rect) {
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
    rect(region);
}

unsafe fn paint(reference: u32, color: u32, region: Rect) {
    begin_pass_stencil_test(reference, 0);
    set_color(color);
    rect(region);
}

unsafe fn panel(state: State, index: u32) {
    const FULL: Rect = [0.0, 0.0, 456.0, 352.0];
    let r = state.regions(index);
    viewport(16 + index * 472, 120, 456, 352);
    begin_pass(0);
    z_index(0);
    set_color(0x18202cff);
    rect(FULL);
    clear(FULL);
    // These authored rectangles are strictly nested (unit + native checked).
    // No arbitrary concave geometry/intersection allocator is implied.
    for (level, region) in r.iter().take(state.levels()).enumerate() {
        push(level as u32 + 1, *region);
    }
    for (level, color) in COLORS.iter().take(state.levels()).enumerate() {
        paint(level as u32 + 1, *color, FULL);
    }
    // Unconditional ZERO/REPLACE have no color writes, preserving painted
    // children. EQUAL + DECREMENT after painting could pollute their color.
    if state.deep != 0 {
        clear(r[2]);
        push(2, r[1]);
        paint(2, 0xffffffff, [0.0, 168.0, 456.0, 6.0]);
    }
    clear(r[1]);
    push(1, r[0]);
    paint(
        1,
        0xffffffff,
        [r[0][0] + r[0][2] * 0.5 - 3.0, 0.0, 6.0, 352.0],
    );
    if state.cleanup != 0 {
        clear(r[0]);
    }
    // Reuse the released reference in a disjoint sibling. Without root cleanup,
    // this final test incorrectly paints the entire old parent green as well.
    push(1, r[3]);
    paint(1, COLORS[3], FULL);
    clear(r[3]);
    begin_pass(0);
    z_index(2);
    set_color(0xccccccff);
    rect([0.0, 0.0, 456.0, 2.0]);
    rect([0.0, 350.0, 456.0, 2.0]);
    rect([0.0, 2.0, 2.0, 348.0]);
    rect([454.0, 2.0, 2.0, 348.0]);
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
            format_args!("STENCIL REGION STACK"),
        );
        hud::text(
            16.0,
            48.0,
            12.0,
            0xffffffff,
            format_args!("A 2/3 levels | B break cleanup | X motion/pause | Y reset"),
        );
        hud::text(
            16.0,
            74.0,
            11.0,
            0xccccccff,
            format_args!(
                "CLEAN={} PAUSE={} TICK={} | peak {}/{} nonzero refs",
                state.cleanup,
                state.paused,
                state.tick,
                state.levels(),
                MAX_REFERENCE
            ),
        );
        for index in 0..2 {
            hud::text(
                16.0 + index as f32 * 472.0,
                98.0,
                14.0,
                0xffffffff,
                format_args!(
                    "P{} / refs 1..{} / phase {}",
                    index + 1,
                    state.levels(),
                    state.tick.wrapping_add(index) & 1
                ),
            );
        }
        hud::text(
            16.0,
            491.0,
            10.0,
            0xffffffff,
            format_args!(
                "Push 1..N; paint rings; pop and restore parents; reuse 1 for green sibling."
            ),
        );
        hud::text(
            16.0,
            517.0,
            10.0,
            0xccccccff,
            format_args!(
                "Fixed 3-level recipe; full byte masks. White bars prove restored parent tests."
            ),
        );
    }
}
