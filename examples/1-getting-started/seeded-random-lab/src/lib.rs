//! A fixed replay seed controls a bounded route. No guest host-reseed API exists.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const POINTS: usize = 32;
static POINT_COUNT: u32 = POINTS as u32;
static mut ROUTE: [i32; POINTS] = [0; POINTS];
static mut ROUTE_HASH: u32 = 0;
static mut MAX_LANE: u32 = 0;
static mut GENERATION: u32 = 0;
static mut CURSOR: u32 = 0;
static mut AUTO: u32 = 0;

unsafe fn generate() {
    let route = &mut *core::ptr::addr_of_mut!(ROUTE);
    let mut lane = 0i32;
    let mut hash = 2166136261u32;
    let mut max_lane = 0;
    for point in route.iter_mut() {
        lane = (lane + random_range(-1, 2)).clamp(-3, 3);
        *point = lane;
        max_lane = max_lane.max(lane.unsigned_abs());
        hash = (hash ^ (lane + 3) as u32).wrapping_mul(16777619);
    }
    ROUTE_HASH = hash;
    MAX_LANE = max_lane;
    GENERATION = GENERATION.wrapping_add(1);
    CURSOR = 0;
    let generation = GENERATION;
    hud::log(format_args!(
        "ROUTE generation={} points=32 hash={}",
        generation, hash
    ));
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        for (name, ptr) in [
            ("route_hash", core::ptr::addr_of!(ROUTE_HASH)),
            ("max_lane", core::ptr::addr_of!(MAX_LANE)),
            ("generation", core::ptr::addr_of!(GENERATION)),
            ("cursor", core::ptr::addr_of!(CURSOR)),
            ("auto", core::ptr::addr_of!(AUTO)),
            ("points", core::ptr::addr_of!(POINT_COUNT)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        generate();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::B) != 0 {
            generate();
        }
        if button_pressed(0, button::X) != 0 {
            AUTO ^= 1;
        }
        if button_held(0, button::LEFT) != 0 {
            CURSOR = CURSOR.saturating_sub(1);
        }
        if button_held(0, button::RIGHT) != 0 || (AUTO != 0 && tick_count() % 6 == 0) {
            CURSOR = (CURSOR + 1).min(POINTS as u32 - 1);
        }
        if button_pressed(0, button::A) != 0 {
            CURSOR = 0;
            AUTO = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let generation = GENERATION;
        let hash = ROUTE_HASH;
        let cursor = CURSOR;
        hud::text(
            32.0,
            24.0,
            28.0,
            0xffffffff,
            format_args!("SEEDED RANDOM / a bounded route"),
        );
        hud::text(
            32.0,
            70.0,
            14.0,
            0xabc1d6ff,
            format_args!("Left/Right: walk | A: reset runner | B: next route | X: autorun"),
        );
        hud::text(
            48.0,
            116.0,
            18.0,
            0xffcd66ff,
            format_args!("Generation {} | 32 points | hash {:08x}", generation, hash),
        );
        set_color(0x263b50ff);
        for lane in -3..=3 {
            let y = 268.0 + lane as f32 * 28.0;
            draw_line(64.0, y, 901.0, y, 1.0);
        }
        let route = &*core::ptr::addr_of!(ROUTE);
        for i in 0..POINTS {
            let x = 64.0 + i as f32 * 27.0;
            let y = 268.0 + route[i] as f32 * 28.0;
            if i > 0 {
                set_color(0x67d3eeff);
                draw_line(x - 27.0, 268.0 + route[i - 1] as f32 * 28.0, x, y, 3.0);
            }
            set_color(0x67d3eeff);
            draw_circle(x, y, 4.0);
        }
        set_color(0xffcd66ff);
        draw_circle(
            64.0 + cursor as f32 * 27.0,
            268.0 + route[cursor as usize] as f32 * 28.0,
            10.0,
        );
        hud::text(
            48.0,
            392.0,
            18.0,
            0xffffffff,
            format_args!(
                "Runner {} / 31 | {}",
                cursor,
                if AUTO != 0 { "AUTO" } else { "MANUAL" }
            ),
        );
        hud::text(
            48.0,
            434.0,
            14.0,
            0xabc1d6ff,
            format_args!("32 RNG draws per route. Lanes -3..3; no retry loop."),
        );
        hud::text(
            48.0,
            468.0,
            14.0,
            0xffffffff,
            format_args!("A resets the runner; B advances the existing RNG stream."),
        );
        hud::text(
            48.0,
            510.0,
            12.0,
            0xabc1d6ff,
            format_args!("Same replay seed = same route. F3: state. Render never consumes RNG."),
        );
    }
}
