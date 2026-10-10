//! Four stencil demonstrations. Simulation changes only in update; render is read-only.
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::{ffi::*, hud};
mod state;
use state::{tint_color, State};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const WIDTH: f32 = 960.0;
const HEIGHT: f32 = 540.0;
const BASE_CAMERA: [f32; 4] = [0.0, 5.0, 10.0, 60.0];
const PORTAL_CAMERA: [f32; 4] = [6.0, 3.0, 14.0, 45.0];
static mut CUBE: u32 = 0;
static mut SPHERE: u32 = 0;
static mut FLOOR: u32 = 0;
static mut STATE: State = State::RESET;

const NAMES: [&str; 4] = [
    "1. Inside circle",
    "2. Outside circle",
    "3. Diagonal split",
    "4. Alternate-camera portal",
];
const PASSES: [&str; 4] = [
    "1 WRITE circle -> 2 EQUAL scene -> 3 NORMAL HUD",
    "1 BASE scene -> 2 WRITE circle -> 3 NOT_EQUAL dark -> 4 NORMAL HUD",
    "1 WRITE triangle -> 2 EQUAL warm -> 3 NOT_EQUAL cool -> 4 NORMAL HUD",
    "1 BASE scene -> 2 RING no depth write -> 3 WRITE -> 4 EQUAL other camera -> 5 HUD",
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eff);
        CUBE = cube(1.0, 1.0, 1.0);
        SPHERE = sphere(0.5, 16, 8);
        FLOOR = plane(10.0, 10.0, 4, 4);
        for (name, ptr) in [
            ("mode", core::ptr::addr_of!(STATE.mode)),
            ("clear_depth", core::ptr::addr_of!(STATE.clear_depth)),
            ("paused", core::ptr::addr_of!(STATE.paused)),
            ("mask", core::ptr::addr_of!(STATE.mask)),
            ("tick", core::ptr::addr_of!(STATE.tick)),
            ("rotation", core::ptr::addr_of!(STATE.rotation)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let mut pressed = 0;
        for button in [
            button::LEFT,
            button::RIGHT,
            button::A,
            button::B,
            button::X,
            button::Y,
            button::L1,
        ] {
            if button_pressed(0, button) != 0 {
                pressed |= 1 << button;
            }
        }
        (&mut *core::ptr::addr_of_mut!(STATE)).step(pressed);
    }
}

/// Reuse identical geometry/camera setup in every stencil-test pass.
unsafe fn scene(camera: [f32; 4], tint: u32) {
    if STATE.mask != 0 {
        // Solid ink exercises stencil coverage; screen quads use Always depth
        // comparison. The depth-clear counterexample is the real 3D scene.
        set_color(if tint == 0xffffffff { 0x44dd99ff } else { tint });
        draw_rect(0.0, 0.0, WIDTH, HEIGHT);
        return;
    }
    camera_set(camera[0], camera[1], camera[2], 0.0, 0.0, 0.0);
    camera_fov(camera[3]);
    push_identity();
    push_translate(0.0, -1.0, 0.0);
    set_color(tint_color(0x404050ff, tint));
    draw_mesh(FLOOR);
    push_identity();
    push_rotate_y(STATE.rotation as f32 * 0.5);
    set_color(tint_color(0xff6644ff, tint));
    draw_mesh(CUBE);
    for (i, color) in [0x44ff88ff, 0x4488ffff, 0xff44aaff, 0xffaa44ff]
        .iter()
        .enumerate()
    {
        let radians = (STATE.tick as f32 + i as f32 * 90.0).to_radians();
        push_identity();
        push_translate(libm::cosf(radians) * 3.0, 0.0, libm::sinf(radians) * 3.0);
        set_color(tint_color(*color, tint));
        draw_mesh(SPHERE);
    }
}

unsafe fn outside_mask() {
    begin_pass_full(
        compare::LESS,
        1,
        0,
        compare::NOT_EQUAL,
        1,
        stencil_op::KEEP,
        stencil_op::KEEP,
        stencil_op::KEEP,
    );
}

unsafe fn circle_mask(radius: f32) {
    begin_pass_stencil_write(1, 0);
    set_color(0xffffffff);
    draw_circle(WIDTH * 0.5, HEIGHT * 0.5, radius);
}

unsafe fn inside_circle() {
    circle_mask(200.0);
    begin_pass_stencil_test(1, 0);
    scene(BASE_CAMERA, 0xffffffff);
}

unsafe fn outside_circle() {
    scene(BASE_CAMERA, 0xffffffff);
    circle_mask(250.0);
    outside_mask();
    // Opaque vignette, not a claim of framebuffer alpha blending.
    z_index(1);
    set_color(0x101018ff);
    draw_rect(0.0, 0.0, WIDTH, HEIGHT);
    z_index(0);
}

unsafe fn diagonal_split() {
    begin_pass_stencil_write(1, 0);
    set_color(0xffffffff);
    // Column-major, screen pixels -> WebGPU clip space, identity view.
    push_projection_matrix(
        2.0 / WIDTH,
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / HEIGHT,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0,
        0.0,
        -1.0,
        1.0,
        0.0,
        1.0,
    );
    push_view_matrix(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    push_identity();
    let triangle = [0.0, 0.0, 0.0, WIDTH, 0.0, 0.0, 0.0, HEIGHT, 0.0];
    draw_triangles(triangle.as_ptr(), 3, 0);
    begin_pass_stencil_test(1, 0);
    scene(BASE_CAMERA, 0xffcc99ff);
    outside_mask();
    scene(BASE_CAMERA, 0x99ccffff);
}

unsafe fn portal() {
    let radius = 150.0 + libm::sinf(STATE.tick as f32 / 60.0 * 3.0) * 30.0;
    // Keep the near camera's depth in the aperture: B deliberately exposes the
    // wrong-camera occlusion if the following interior pass does not clear it.
    scene(BASE_CAMERA, 0xffffffff);
    begin_pass_full(
        compare::ALWAYS,
        0,
        0,
        compare::ALWAYS,
        0,
        stencil_op::KEEP,
        stencil_op::KEEP,
        stencil_op::KEEP,
    );
    set_color(0x8800ffff);
    draw_circle(WIDTH * 0.5, HEIGHT * 0.5, radius + 3.0);
    set_color(0x000000ff);
    draw_circle(WIDTH * 0.5, HEIGHT * 0.5, radius - 3.0);
    // Write only the interior mask once; a second write does not erase an old mask.
    circle_mask(radius - 3.0);
    begin_pass_stencil_test(1, STATE.clear_depth);
    scene(PORTAL_CAMERA, 0xccaaffff);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let state = STATE;
        z_index(0);
        match state.mode {
            0 => inside_circle(),
            1 => outside_circle(),
            2 => diagonal_split(),
            _ => portal(),
        }
        // Explicitly remove the stencil test; layered HUD must survive every mask.
        begin_pass(0);
        z_index(2);
        set_color(0x101824ff);
        draw_rect(0.0, 0.0, WIDTH, 104.0);
        draw_rect(0.0, 476.0, WIDTH, 64.0);
        z_index(3);
        hud::text(16.0, 8.0, 24.0, 0xffffffff, format_args!("STENCIL DEMO"));
        hud::text(
            16.0,
            40.0,
            18.0,
            0x88ff88ff,
            format_args!("{}", NAMES[state.mode as usize]),
        );
        hud::text(
            16.0,
            69.0,
            10.0,
            0xccccccff,
            format_args!("{}", PASSES[state.mode as usize]),
        );
        hud::text(
            16.0,
            87.0,
            10.0,
            0xccccccff,
            format_args!(
                "View={} | Pause={} | Tick={} | Portal depth clear={} (whole attachment)",
                if state.mask == 0 {
                    "3D scene"
                } else {
                    "mask ink"
                },
                state.paused,
                state.tick,
                state.clear_depth
            ),
        );
        hud::text(
            16.0,
            486.0,
            12.0,
            0xffffffff,
            format_args!("A modes | B portal depth clear | X play/pause | Y reset"),
        );
        hud::text(
            16.0,
            512.0,
            12.0,
            0xffffffff,
            format_args!("L1 mask ink | D-pad left/right rotate 15 degrees | Starts paused"),
        );
    }
}
