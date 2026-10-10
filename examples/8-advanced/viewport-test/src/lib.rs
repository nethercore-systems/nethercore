//! Split-screen cameras, viewport-local HUDs and scissored stencil masks.
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

static mut CUBE: u32 = 0;
static mut SPHERE: u32 = 0;
static mut FLOOR: u32 = 0;
static mut STATE: State = State::RESET;
const COLORS: [u32; 4] = [0x40c8f0ff, 0xf0b040ff, 0xc060e0ff, 0x70d880ff];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eff);
        CUBE = cube(1.0, 1.0, 1.0);
        SPHERE = sphere(0.5, 16, 8);
        FLOOR = plane(10.0, 10.0, 4, 4);
        for (name, ptr) in [
            ("quad", core::ptr::addr_of!(STATE.quad)),
            ("mask", core::ptr::addr_of!(STATE.mask)),
            ("ink", core::ptr::addr_of!(STATE.ink)),
            ("paused", core::ptr::addr_of!(STATE.paused)),
            ("tick", core::ptr::addr_of!(STATE.tick)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        for (p, name) in ["cam1", "cam2", "cam3", "cam4"].iter().enumerate() {
            debug_watch_f32(
                name.as_ptr(),
                name.len() as u32,
                core::ptr::addr_of!(STATE.angles[p]).cast(),
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let mut pressed = 0;
        for key in [button::A, button::B, button::X, button::Y, button::L1] {
            if button_pressed(0, key) != 0 {
                pressed |= 1 << key;
            }
        }
        let sticks = [
            left_stick_x(0),
            left_stick_x(1),
            left_stick_x(2),
            left_stick_x(3),
        ];
        (&mut *core::ptr::addr_of_mut!(STATE)).step(pressed, sticks);
    }
}

unsafe fn scene(state: State, player: usize) {
    let radians = state.angles[player].to_radians();
    camera_set(
        libm::cosf(radians) * 8.0,
        5.0,
        libm::sinf(radians) * 8.0,
        0.0,
        0.0,
        0.0,
    );
    camera_fov(60.0); // Projection aspect is the current viewport's width/height.
    if state.ink != 0 {
        // Deliberately oversize: the actual viewport scissor must clip every edge.
        let [_, _, w, h] = state.rect(player);
        set_color(COLORS[player]);
        draw_rect(-64.0, -64.0, w as f32 + 128.0, h as f32 + 128.0);
        return;
    }
    push_identity();
    push_translate(0.0, -1.0, 0.0);
    set_color(0x404050ff);
    draw_mesh(FLOOR);
    push_identity();
    push_rotate_y(state.tick as f32 * 0.5);
    set_color(0xff6644ff);
    draw_mesh(CUBE);
    for (i, color) in [0x44ff88ff, 0x4488ffff, 0xff44aaff, 0xffaa44ff]
        .iter()
        .enumerate()
    {
        let angle = (state.tick as f32 + i as f32 * 90.0).to_radians();
        push_identity();
        push_translate(libm::cosf(angle) * 3.0, 0.0, libm::sinf(angle) * 3.0);
        set_color(*color);
        draw_mesh(SPHERE);
    }
    // One off-axis tower makes world orientation legible from all four cameras.
    push_identity();
    push_translate(-1.5, 0.5, 2.2);
    push_scale(0.35, 2.0, 0.35);
    set_color(0xffff66ff);
    draw_mesh(CUBE);
}

unsafe fn local_hud(state: State, player: usize) {
    let [_, _, w, h] = state.rect(player);
    let (w, h) = (w as f32, h as f32);
    z_index(2);
    set_color(COLORS[player]);
    draw_rect(0.0, 0.0, w, 2.0);
    draw_rect(0.0, h - 2.0, w, 2.0);
    draw_rect(0.0, 0.0, 2.0, h);
    draw_rect(w - 2.0, 0.0, 2.0, h);
    set_color(0x101824ff);
    draw_rect(2.0, 2.0, w - 4.0, 34.0);
    draw_rect(w - 44.0, h - 24.0, 42.0, 22.0);
    z_index(3);
    hud::text(
        10.0,
        8.0,
        18.0,
        0xffffffff,
        format_args!("P{} / CAM {:.1}", player + 1, state.angles[player]),
    );
    hud::text(
        w - 34.0,
        h - 18.0,
        10.0,
        0xffffffff,
        format_args!("BR{}", player + 1),
    );
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let state = STATE;
        z_index(0);
        // Build every mask before any scene. Distinct references expose an
        // unclipped later mask overwriting an earlier viewport's stencil.
        if state.mask != 0 {
            for player in 0..state.count() {
                let [x, y, w, h] = state.rect(player);
                viewport(x, y, w, h);
                begin_pass_stencil_write(player as u32 + 1, 0);
                set_color(0xffffffff);
                draw_circle(120.0, h as f32 * 0.5, (h as f32 * 0.75).min(270.0));
            }
        }
        for player in 0..state.count() {
            let [x, y, w, h] = state.rect(player);
            viewport(x, y, w, h);
            if state.mask != 0 {
                begin_pass_stencil_test(player as u32 + 1, 0);
            } else {
                begin_pass(0);
            }
            z_index(0);
            scene(state, player);
            begin_pass(0); // Local UI must not inherit the scene's stencil test.
            local_hud(state, player);
        }
        viewport_clear();
        begin_pass(0);
        z_index(4);
        set_color(0x101824ff);
        draw_rect(0.0, 0.0, 960.0, 64.0);
        draw_rect(0.0, 492.0, 960.0, 48.0);
        z_index(5);
        hud::text(
            12.0,
            8.0,
            22.0,
            0xffffffff,
            format_args!("VIEWPORT CAMERA LAB"),
        );
        hud::text(
            500.0,
            14.0,
            12.0,
            0xccccccff,
            format_args!(
                "{}P  MASK={} INK={} PAUSE={}",
                state.count(),
                state.mask,
                state.ink,
                state.paused
            ),
        );
        hud::text(
            12.0,
            42.0,
            12.0,
            0xffffffff,
            format_args!("A 2P/4P | B mask | L1 ink | X play/pause | Y reset"),
        );
        hud::text(
            12.0,
            506.0,
            12.0,
            0xffffffff,
            format_args!("P1-P4 left stick: own camera | depth clear affects WHOLE attachment"),
        );
        hud::text(
            12.0,
            525.0,
            10.0,
            0xccccccff,
            format_args!(
                "Local split-screen; not a networking/joining demonstration. Starts paused."
            ),
        );
    }
}
