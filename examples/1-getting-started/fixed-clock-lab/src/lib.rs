//! Fixed simulation ticks; zero or repeated draws cannot advance the clock.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut HOST_TICK: u32 = 0;
static mut SIM_TICKS: u32 = 0;
static mut PAUSED: u32 = 0;
static mut COPIES: u32 = 1;
static mut POSITION: u32 = 0;
static mut DT: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        for (name, ptr) in [
            ("host_tick", core::ptr::addr_of!(HOST_TICK)),
            ("sim_ticks", core::ptr::addr_of!(SIM_TICKS)),
            ("paused", core::ptr::addr_of!(PAUSED)),
            ("copies", core::ptr::addr_of!(COPIES)),
            ("position", core::ptr::addr_of!(POSITION)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        debug_watch_f32(b"delta_time".as_ptr(), 10, core::ptr::addr_of!(DT).cast());
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        HOST_TICK = tick_count() as u32;
        DT = delta_time();
        if button_pressed(0, button::A) != 0 {
            PAUSED ^= 1;
        }
        if button_pressed(0, button::B) != 0 {
            COPIES = if COPIES == 1 { 4 } else { 1 };
        }
        if button_pressed(0, button::Y) != 0 {
            SIM_TICKS = 0;
            PAUSED = 0;
        } else if PAUSED == 0 {
            SIM_TICKS = SIM_TICKS.wrapping_add(1);
        }
        POSITION = (SIM_TICKS % 200) * 4;
        // One diagnostic per simulated second, including replay endpoint tick 180.
        // Logging may repeat under rollback; it is not an audio/persistence event.
        if HOST_TICK % 60 == 0 {
            let tick = HOST_TICK;
            let sim = SIM_TICKS;
            let pos = POSITION;
            hud::log(format_args!(
                "CLOCK tick={} sim={} position={}",
                tick, sim, pos
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let tick = HOST_TICK;
        let sim = SIM_TICKS;
        let dt = DT;
        let copies = COPIES;
        hud::text(
            32.0,
            24.0,
            28.0,
            0xffffffff,
            format_args!("FIXED CLOCK / update owns time"),
        );
        hud::text(
            32.0,
            70.0,
            16.0,
            0xabc1d6ff,
            format_args!("A: pause | B: 1 or 4 draws | Y: reset simulation"),
        );
        hud::text(
            48.0,
            128.0,
            22.0,
            0x67d3eeff,
            format_args!("Host tick {} | delta_time {:.6} s", tick, dt),
        );
        hud::text(
            48.0,
            172.0,
            22.0,
            0xffcd66ff,
            format_args!("Simulation ticks {} | {:.3} seconds", sim, sim as f32 * dt),
        );
        hud::text(
            48.0,
            216.0,
            18.0,
            0xffffffff,
            format_args!(
                "{} | {} draws per render callback",
                if PAUSED != 0 { "PAUSED" } else { "RUNNING" },
                copies
            ),
        );
        set_color(0x34475eff);
        draw_rect(64.0, 316.0, 832.0, 8.0);
        // Deliberately repeat the actual draws, reading exactly the same state.
        // No counter, time accumulator or RNG call belongs in this callback.
        for _ in 0..copies {
            set_color(0xffcd66ff);
            draw_rect(64.0 + POSITION as f32, 292.0, 32.0, 56.0);
        }
        hud::text(
            48.0,
            392.0,
            16.0,
            0xabc1d6ff,
            format_args!("4 pixels per simulation tick; wraps every 200 ticks."),
        );
        hud::text(
            48.0,
            424.0,
            16.0,
            0xabc1d6ff,
            format_args!("Host ticks continue while simulation is paused."),
        );
        hud::text(
            48.0,
            472.0,
            14.0,
            0xffffffff,
            format_args!("Headless (zero draws) and native replay reach the same state."),
        );
        hud::text(
            48.0,
            508.0,
            12.0,
            0xabc1d6ff,
            format_args!(
                "F3: inspect | No wall clock, render-time mutation or interpolation required."
            ),
        );
    }
}
