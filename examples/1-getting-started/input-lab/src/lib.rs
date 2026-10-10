//! Observe the console input contract, not a guest keyboard API.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[derive(Clone, Copy)]
struct Pad {
    held: u32,
    pressed: u32,
    released: u32,
    presses: u32,
    lx: f32,
    ly: f32,
    rx: f32,
    ry: f32,
    lt: f32,
    rt: f32,
    fx: f32,
    fy: f32,
}
const ZERO: Pad = Pad {
    held: 0,
    pressed: 0,
    released: 0,
    presses: 0,
    lx: 0.0,
    ly: 0.0,
    rx: 0.0,
    ry: 0.0,
    lt: 0.0,
    rt: 0.0,
    fx: 0.0,
    fy: 0.0,
};
static mut PADS: [Pad; 4] = [ZERO; 4];
static mut PLAYERS: u32 = 1;
static mut LOCAL_MASK: u32 = 1;
const NAMES: [&str; 14] = [
    "UP", "DOWN", "LEFT", "RIGHT", "A", "B", "X", "Y", "L1", "R1", "L3", "R3", "START", "SEL",
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        debug_watch_u32(b"players".as_ptr(), 7, core::ptr::addr_of!(PLAYERS).cast());
        debug_watch_u32(
            b"local_mask".as_ptr(),
            10,
            core::ptr::addr_of!(LOCAL_MASK).cast(),
        );
        let pads = &*core::ptr::addr_of!(PADS);
        for (i, p) in pads.iter().enumerate() {
            let group = [b'P', b'1' + i as u8];
            debug_group_begin(group.as_ptr(), 2);
            for (name, ptr) in [
                ("held", &p.held),
                ("pressed", &p.pressed),
                ("released", &p.released),
                ("presses", &p.presses),
            ] {
                debug_watch_u32(name.as_ptr(), name.len() as u32, (ptr as *const u32).cast());
            }
            for (name, ptr) in [
                ("lx", &p.lx),
                ("ly", &p.ly),
                ("rx", &p.rx),
                ("ry", &p.ry),
                ("lt", &p.lt),
                ("rt", &p.rt),
                ("filtered_x", &p.fx),
                ("filtered_y", &p.fy),
            ] {
                debug_watch_f32(name.as_ptr(), name.len() as u32, (ptr as *const f32).cast());
            }
            debug_group_end();
        }
    }
}

fn deadzone(x: f32, y: f32) -> (f32, f32) {
    // Guest hard radial deadzone; preserves magnitudes outside radius 0.20.
    if x * x + y * y < 0.20 * 0.20 {
        (0.0, 0.0)
    } else {
        (x, y)
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        PLAYERS = player_count();
        LOCAL_MASK = local_player_mask();
        // WASM callbacks are sequential. No reference survives this callback.
        let pads = &mut *core::ptr::addr_of_mut!(PADS);
        for (i, p) in pads.iter_mut().enumerate() {
            let id = i as u32;
            p.held = buttons_held(id);
            p.pressed = buttons_pressed(id);
            p.released = buttons_released(id);
            p.presses = if p.pressed & (1 << button::SELECT) != 0 {
                0
            } else {
                p.presses.saturating_add(p.pressed.count_ones())
            };
            p.lx = left_stick_x(id);
            p.ly = left_stick_y(id);
            p.rx = right_stick_x(id);
            p.ry = right_stick_y(id);
            p.lt = trigger_left(id);
            p.rt = trigger_right(id);
            (p.fx, p.fy) = deadzone(p.lx, p.ly);
        }
    }
}

unsafe fn stick(x: f32, y: f32, sx: f32, sy: f32) {
    set_color(0x20344dff);
    draw_circle(x, y, 30.0);
    set_color(0x405970ff);
    draw_circle_outline(x, y, 30.0, 1.0);
    draw_line(x - 30.0, y, x + 30.0, y, 1.0);
    draw_line(x, y - 30.0, x, y + 30.0, 1.0);
    set_color(0x3a4152ff);
    draw_circle(x, y, 6.0);
    let (fx, fy) = deadzone(sx, sy);
    set_color(0x67d3eeff);
    draw_circle(x + sx * 28.0, y + sy * 28.0, 4.0);
    set_color(0xffcd66ff);
    draw_circle_outline(x + fx * 28.0, y + fy * 28.0, 6.0, 2.0);
}

// Same-pass batches may reorder. Explicit layers keep labels over controls,
// and controls over card backgrounds, independent of texture-sort order.
unsafe fn label(x: f32, y: f32, size: f32, color: u32, args: core::fmt::Arguments<'_>) {
    z_index(2);
    hud::text(x, y, size, color, args);
    z_index(1);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        label(
            24.0,
            18.0,
            28.0,
            0xffffffff,
            format_args!("INPUT LAB / four session slots"),
        );
        label(
            24.0,
            54.0,
            13.0,
            0xabc1d6ff,
            format_args!("Blue held | Green press | Pink release | Sticks: API blue / filter gold"),
        );
        for (i, p) in (&*core::ptr::addr_of!(PADS)).iter().enumerate() {
            let x = 24.0 + (i % 2) as f32 * 468.0;
            let y = 82.0 + (i / 2) as f32 * 208.0;
            z_index(0);
            set_color(0x1d2c41ff);
            draw_rect(x, y, 444.0, 196.0);
            z_index(1);
            let status = if i as u32 >= PLAYERS {
                "inactive"
            } else if LOCAL_MASK & (1 << i) != 0 {
                "local"
            } else {
                "remote"
            };
            label(
                x + 10.0,
                y + 8.0,
                15.0,
                0xffffffff,
                format_args!("P{} {} | presses {}", i + 1, status, p.presses),
            );
            for (b, name) in NAMES.iter().enumerate() {
                let cx = x + 10.0 + (b % 7) as f32 * 60.0;
                let cy = y + 36.0 + (b / 7) as f32 * 27.0;
                let bit = 1 << b;
                let color = if p.pressed & bit != 0 {
                    0x3cba7dff
                } else if p.released & bit != 0 {
                    0xbd618aff
                } else if p.held & bit != 0 {
                    0x367ca0ff
                } else {
                    0x2b3b50ff
                };
                set_color(color);
                draw_rect(cx, cy, 54.0, 22.0);
                label(
                    cx + 4.0,
                    cy + 5.0,
                    9.0,
                    0xffffffff,
                    format_args!("{}", name),
                );
            }
            label(x + 24.0, y + 94.0, 12.0, 0xabc1d6ff, format_args!("LEFT"));
            label(x + 144.0, y + 94.0, 12.0, 0xabc1d6ff, format_args!("RIGHT"));
            stick(x + 54.0, y + 142.0, p.lx, p.ly);
            stick(x + 174.0, y + 142.0, p.rx, p.ry);
            for (j, (name, value)) in [("LT", p.lt), ("RT", p.rt)].iter().enumerate() {
                let ty = y + 115.0 + j as f32 * 34.0;
                label(
                    x + 224.0,
                    ty,
                    12.0,
                    0xffffffff,
                    format_args!("{} {:.2}", name, value),
                );
                set_color(0x34475eff);
                draw_rect(x + 312.0, ty, 118.0, 14.0);
                set_color(0x67d3eeff);
                draw_rect(x + 312.0, ty, 118.0 * value, 14.0);
            }
            label(
                x + 218.0,
                y + 176.0,
                11.0,
                0xabc1d6ff,
                format_args!("F3: values"),
            );
        }
        label(
            24.0,
            502.0,
            12.0,
            0xffffffff,
            format_args!("Arrows ZXCV QE | WASD / IJKL: sticks | U/O: triggers | R/Y: L3/R3"),
        );
        label(24.0, 522.0, 11.0, 0xabc1d6ff, format_args!("Select/Shift: clear counts | Blue: host input, gold: 0.20 deadzone | Y points down"));
    }
}
