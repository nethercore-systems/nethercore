//! Compare post-multiplied API calls with explicit guest model matrices.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod composition;
use composition::{crane, left, mul, point, right, scale, translate, Matrix};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut CUBE: u32 = 0;
static mut STEP: u32 = 1;
static mut FLIP: u32 = 0;
static mut EXPLICIT: u32 = 0;
static mut RIGHT_X: f32 = 0.;
static mut RIGHT_Y: f32 = 0.;
static mut ELBOW_X: f32 = 0.;
static mut ELBOW_Y: f32 = 0.;
static mut TIP_X: f32 = 0.;
static mut TIP_Y: f32 = 0.;

unsafe fn refresh() {
    let (_, child) = crane(STEP, FLIP);
    let r = point(right(STEP), [0.; 3]);
    let elbow = point(child, [0.; 3]);
    let tip = point(child, [1., 0., 0.]);
    RIGHT_X = r[0];
    RIGHT_Y = r[1];
    ELBOW_X = elbow[0];
    ELBOW_Y = elbow[1];
    TIP_X = tip[0];
    TIP_Y = tip[1];
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        // cube arguments are HALF extents: this produces a one-unit cube.
        CUBE = cube(0.5, 0.5, 0.5);
        refresh();
        for (name, ptr) in [
            ("step", core::ptr::addr_of!(STEP)),
            ("flip", core::ptr::addr_of!(FLIP)),
            ("explicit", core::ptr::addr_of!(EXPLICIT)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
        for (name, ptr) in [
            ("right_x", core::ptr::addr_of!(RIGHT_X)),
            ("right_y", core::ptr::addr_of!(RIGHT_Y)),
            ("elbow_x", core::ptr::addr_of!(ELBOW_X)),
            ("elbow_y", core::ptr::addr_of!(ELBOW_Y)),
            ("tip_x", core::ptr::addr_of!(TIP_X)),
            ("tip_y", core::ptr::addr_of!(TIP_Y)),
        ] {
            debug_watch_f32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            STEP = (STEP + 1) % 8;
        }
        if button_pressed(0, button::B) != 0 {
            FLIP ^= 1;
        }
        if button_pressed(0, button::X) != 0 {
            EXPLICIT ^= 1;
        }
        if button_pressed(0, button::Y) != 0 {
            STEP = 1;
            FLIP = 0;
            EXPLICIT = 0;
        }
        refresh();
    }
}

unsafe fn block(model: Matrix, color: u32) {
    transform_set(model.as_ptr());
    set_color(color);
    draw_mesh(CUBE);
}

// This camera's z=0 world plane is 540/(2*8) pixels per world unit.
unsafe fn marker(p: [f32; 3]) {
    let x = 480. + 270. * p[0] / 8.;
    let y = 270. - 270. * p[1] / 8.;
    set_color(0xffffffff);
    draw_line(x - 5., y, x + 5., y, 2.);
    draw_line(x, y - 5., x, y + 5., 2.);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let step = STEP;
        let flip = FLIP;
        let explicit = EXPLICIT;
        let angle = step as f32 * 45.;
        camera_set(0., 0., 8., 0., 0., 0.);
        camera_fov(90.);
        use_uniform_color(1);
        if explicit == 0 {
            push_identity();
            push_translate(-7., 0., 0.); // Only separates the comparison panels.
            push_translate(2., 0., 0.);
            push_rotate_z(angle); // M = panel * T * R * S.
            push_scale(1.5, 0.4, 0.4);
            set_color(0x32cde1ff);
            draw_mesh(CUBE);
            push_identity(); // NOT a push/pop hierarchy stack.
            push_rotate_z(angle);
            push_translate(2., 0., 0.); // M = R * T * S.
            push_scale(1.5, 0.4, 0.4);
            set_color(0xffc828ff);
            draw_mesh(CUBE);
        } else {
            block(mul(left(step), scale(1.5, 0.4, 0.4)), 0x32cde1ff);
            block(mul(right(step), scale(1.5, 0.4, 0.4)), 0xffc828ff);
        }
        let (parent, child) = crane(step, flip);
        block(
            mul(translate(7., -1.2, 0.), scale(0.18, 2.4, 0.18)),
            0x8899aaff,
        );
        block(
            mul(translate(7., -2.4, 0.), scale(2., 0.15, 1.)),
            0x8899aaff,
        );
        block(
            mul(mul(parent, translate(0.75, 0., 0.)), scale(1.5, 0.25, 0.35)),
            0x32dd88ff,
        );
        block(
            mul(mul(child, translate(0.5, 0., 0.)), scale(1., 0.22, 0.3)),
            0x32dd88ff,
        );
        // 2D overlays are separate from the current 3D model transform.
        z_index(1);
        set_color(0x536a83ff);
        draw_line(48., 270., 352., 270., 1.);
        draw_line(368., 270., 616., 270., 1.);
        draw_circle_outline(480., 270., 270. * 2. / 8., 1.);
        z_index(2);
        marker([-7., 0., 0.]);
        marker(point(left(step), [0.; 3]));
        marker([0.; 3]);
        marker(point(right(step), [0.; 3]));
        marker([7., 0., 0.]);
        marker(point(child, [0.; 3]));
        marker(point(child, [1., 0., 0.]));
        hud::text(
            32.,
            20.,
            24.,
            0xffffffff,
            format_args!("TRANSFORM COMPOSITION / order matters"),
        );
        hud::text(
            32.,
            58.,
            12.,
            0xabc1d6ff,
            format_args!("Pad/key: A/Z +45deg | B/X elbow | X/C API/matrix | Y/V reset"),
        );
        hud::text(40., 100., 14., 0xffffffff, format_args!("CALLS: T, R"));
        hud::text(360., 100., 14., 0xffffffff, format_args!("CALLS: R, T"));
        hud::text(680., 100., 14., 0xffffffff, format_args!("EXPLICIT CRANE"));
        hud::text(
            40.,
            124.,
            11.,
            0xabc1d6ff,
            format_args!("POINT: rotate THEN translate"),
        );
        hud::text(
            360.,
            124.,
            11.,
            0xabc1d6ff,
            format_args!("POINT: translate THEN rotate"),
        );
        hud::text(
            680.,
            124.,
            11.,
            0xabc1d6ff,
            format_args!("Child = parent * T * R"),
        );
        hud::text(
            40.,
            388.,
            12.,
            0xffffffff,
            format_args!("T*R: center stays at 2,0"),
        );
        hud::text(
            360.,
            388.,
            12.,
            0xffffffff,
            format_args!("R*T: center {:.2},{:.2}", RIGHT_X, RIGHT_Y),
        );
        hud::text(
            680.,
            388.,
            11.,
            0xffffffff,
            format_args!("Elbow {:.2},{:.2}", ELBOW_X, ELBOW_Y),
        );
        hud::text(
            680.,
            412.,
            11.,
            0xffffffff,
            format_args!("Tip {:.2},{:.2}", TIP_X, TIP_Y),
        );
        hud::text(
            32.,
            458.,
            12.,
            0xffffffff,
            format_args!(
                "Angle {} / child {}45 / {}",
                angle as u32,
                if flip == 0 { "+" } else { "-" },
                if explicit == 0 {
                    "API calls"
                } else {
                    "guest matrices"
                }
            ),
        );
        hud::text(
            32.,
            489.,
            11.,
            0xabc1d6ff,
            format_args!(
                "Column vectors: rightmost transform acts first. Crosses = z=0 reference points."
            ),
        );
        hud::text(32.,516.,10.,0xabc1d6ff,format_args!("F3 state | Left panel offset -7; crane base +7. Each model is set explicitly; no pop."));
    }
}
