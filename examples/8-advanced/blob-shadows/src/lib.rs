//! A reusable radial-alpha ground shadow; real depth, no framebuffer blending.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod state;
use state::{clip_floor, State};
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
static mut STATE: State = State::RESET;
static mut TEXTURE: u32 = 0;
const PIXELS: [u8; 4096] = state::pixels();
const UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x101824ff);
        TEXTURE = load_texture(32, 32, PIXELS.as_ptr());
        assert!(TEXTURE != 0);
        for (name, ptr) in [
            ("lane", core::ptr::addr_of!(STATE.lane)),
            ("bad_depth", core::ptr::addr_of!(STATE.bad_depth)),
            ("write_depth", core::ptr::addr_of!(STATE.write_depth)),
            ("shadow", core::ptr::addr_of!(STATE.shadow)),
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
        for key in [
            button::A,
            button::B,
            button::X,
            button::Y,
            button::L1,
            button::R1,
        ] {
            if button_pressed(0, key) != 0 {
                pressed |= 1 << key;
            }
        }
        (&mut *core::ptr::addr_of_mut!(STATE)).step(pressed);
    }
}

unsafe fn quad(points: [[f32; 3]; 4], uv: [[f32; 2]; 4]) {
    let mut vertices = [0.0; 30];
    for (i, n) in [0, 1, 2, 0, 2, 3].iter().enumerate() {
        vertices[i * 5..i * 5 + 3].copy_from_slice(&points[*n]);
        vertices[i * 5 + 3..i * 5 + 5].copy_from_slice(&uv[*n]);
    }
    draw_triangles(vertices.as_ptr(), 6, format::POS_UV as u32);
}
unsafe fn ground([x0, x1, z0, z1]: [f32; 4], y: f32, uv: [[f32; 2]; 4]) {
    quad([[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]], uv);
}
unsafe fn box_mesh([x0, x1, y0, y1, z0, z1]: [f32; 6], colors: [u32; 2]) {
    let p = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x0, y1, z0],
        [x1, y1, z0],
        [x0, y0, z1],
        [x1, y0, z1],
        [x0, y1, z1],
        [x1, y1, z1],
    ];
    for (face, shade) in [
        ([0, 1, 3, 2], 0),
        ([4, 5, 7, 6], 0),
        ([0, 4, 6, 2], 0),
        ([1, 5, 7, 3], 0),
        ([2, 3, 7, 6], 1),
        ([0, 1, 5, 4], 0),
    ] {
        set_color(colors[shade]);
        quad(face.map(|i| p[i]), UV);
    }
}
unsafe fn scene(s: State) {
    let (x, h, r, alpha) = s.pose();
    viewport(32, 120, 896, 352);
    // Fixed orthographic teaching view. Dyadic coefficients keep projected
    // mask geometry and moving texel edges away from near-boundary ambiguity.
    push_view_matrix(
        1.0, 0.0, 0.0, 0.0, 0.0, 0.75, 0.5, 0.0, 0.0, -0.5, 0.75, 0.0, 0.0, 0.0, -10.0, 1.0,
    );
    push_projection_matrix(
        1.0 / 7.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / 2.75,
        0.0,
        0.0,
        0.0,
        0.0,
        -0.05,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );
    push_identity();
    cull_mode(cull::NONE);
    texture_filter(0);
    begin_pass(1);
    uniform_alpha(15);
    dither_offset(0, 0);
    texture_bind(0);
    set_color(0x5c697cff);
    ground([-5.0, 5.0, -3.0, 3.0], 0.0, UV);
    box_mesh(
        [x - 0.35, x + 0.35, h, h + 0.75, -0.75, 0.0],
        [0x30a0f0ff, 0x60c0ffff],
    );
    box_mesh([-0.2, 0.8, 0.0, 1.15, 0.3, 1.1], [0x40b070ff, 0x60e080ff]);
    begin_pass_full(
        if s.bad_depth == 0 {
            compare::LESS_EQUAL
        } else {
            compare::ALWAYS
        },
        s.write_depth,
        0,
        compare::ALWAYS,
        0,
        stencil_op::KEEP,
        stencil_op::KEEP,
        stencil_op::KEEP,
    );
    if s.shadow != 0 {
        // CPU clip a known flat receiver, retaining UVs from the full disk.
        if let Some([x0, x1, z0, z1]) = clip_floor([x - r, x + r, -0.375 - r, -0.375 + r]) {
            let u0 = (x0 - x + r) / (2.0 * r);
            let u1 = (x1 - x + r) / (2.0 * r);
            let v0 = (z0 + 0.375 + r) / (2.0 * r);
            let v1 = (z1 + 0.375 + r) / (2.0 * r);
            texture_bind(TEXTURE);
            set_color(0x040608ff);
            uniform_alpha(alpha);
            dither_offset(0, 0);
            // Geometry offset along receiver normal; not native depth bias.
            ground(
                [x0, x1, z0, z1],
                1.0 / 1024.0,
                [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
            );
        }
    }
    // This later inlay is above the receiver but behind the blob. It remains
    // visible only if the preceding shadow has NOT polluted the depth buffer.
    begin_pass(0);
    texture_bind(0);
    uniform_alpha(15);
    dither_offset(0, 0);
    if let Some(rect) = clip_floor([x - 0.5, x - 0.4375, -1.4, 1.2]) {
        set_color(0xebecf0ff);
        ground(rect, 1.0 / 4096.0, UV);
    }
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
    z_index(2);
    set_color(0xccccccff);
    for [x, y, w, h] in [
        [0.0, 0.0, 896.0, 2.0],
        [0.0, 350.0, 896.0, 2.0],
        [0.0, 2.0, 2.0, 348.0],
        [894.0, 2.0, 2.0, 348.0],
    ] {
        draw_rect(x, y, w, h);
    }
}
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let s = STATE;
        scene(s);
        viewport_clear();
        begin_pass(0);
        z_index(3);
        hud::text(16.0, 12.0, 24.0, 0xffffffff, format_args!("BLOB SHADOWS"));
        hud::text(
            16.0,
            48.0,
            11.0,
            0xffffffff,
            format_args!("A lane | B depth bug | X run/pause | L1 shadow | R1 write bug | Y reset"),
        );
        let (_, height, radius, alpha) = s.pose();
        hud::text(
            16.0,
            74.0,
            10.0,
            0xccccccff,
            format_args!(
                "Lane {} | H {:.2} R {:.2} alpha {}/15 | shadow {} depth {} write {} pause {}",
                s.lane,
                height,
                radius,
                alpha,
                s.shadow,
                if s.bad_depth == 0 { "LE" } else { "ALWAYS" },
                s.write_depth,
                s.paused
            ),
        );
        hud::text(
            32.0,
            98.0,
            12.0,
            0xffffffff,
            format_args!("Known floor [-5,5] x [-3,3] | tick {}", s.tick),
        );
        hud::text(
            16.0,
            491.0,
            10.0,
            0xffffffff,
            format_args!(
                "Blue actor / green foreground / white inlay tests depth-write OFF (R1 breaks it)."
            ),
        );
        hud::text(16.0,517.0,10.0,0xccccccff,
        format_args!("Known flat receiver; texture alpha + Bayer coverage. No multiply blend or depth bias."));
    }
}
