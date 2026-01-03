//! Stencil Demo Example
//!
//! Demonstrates the render pass system for stencil masking:
//! - Circle mask (render inside)
//! - Inverted mask (render outside / vignette)
//! - Diagonal split
//! - Animated portal
//!
//! Press A to cycle through demo modes.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// Input (button indices from zx.rs)
const BUTTON_A: u32 = 4;

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const SCREEN_CX: f32 = SCREEN_WIDTH / 2.0;
const SCREEN_CY: f32 = SCREEN_HEIGHT / 2.0;

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;
static mut FLOOR_MESH: u32 = 0;

// State
static mut DEMO_MODE: u32 = 0;
static mut TIME: f32 = 0.0;
static mut ROTATION: f32 = 0.0;

// Demo mode names and descriptions
static DEMO_NAMES: [&str; 4] = [
    "1. Circle Mask",
    "2. Inverted Mask (Vignette)",
    "3. Diagonal Split",
    "4. Animated Portal",
];

static DEMO_DESCRIPTIONS: [&str; 4] = [
    "begin_pass_stencil_write/test: Only render inside circle",
    "begin_pass_full: Render outside mask (dark edges)",
    "Two masks: Different tints per half",
    "Pulsing portal with different camera inside",
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(0); // Lambert mode

        // Generate scene objects
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        SPHERE_MESH = sphere(0.5, 16, 8);
        FLOOR_MESH = plane(10.0, 10.0, 4, 4);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Cycle demo modes with A button
        if button_pressed(0, BUTTON_A) != 0 {
            DEMO_MODE = (DEMO_MODE + 1) % 4;
        }

        // Rotate scene with stick
        let stick_x = left_stick_x(0);
        if stick_x.abs() > 0.1 {
            ROTATION += stick_x * 2.0;
        } else {
            ROTATION += 0.5; // Auto-rotate
        }
    }
}

/// Draw the 3D scene with default camera
unsafe fn draw_scene() {
    draw_scene_with_camera(0.0, 5.0, 10.0, 60.0);
}

/// Draw the 3D scene with custom camera position
unsafe fn draw_scene_with_camera(cam_x: f32, cam_y: f32, cam_z: f32, fov: f32) {
    camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
    camera_fov(fov);

    // Floor
    push_identity();
    push_translate(0.0, -1.0, 0.0);
    set_color(0x404050FF);
    draw_mesh(FLOOR_MESH);

    // Central cube (rotating)
    push_identity();
    push_rotate_y(ROTATION);
    set_color(0xFF6644FF);
    draw_mesh(CUBE_MESH);

    // Orbiting spheres
    for i in 0..4 {
        let angle = TIME * 60.0 + (i as f32) * 90.0;
        let rad = angle.to_radians();
        let x = libm::cosf(rad) * 3.0;
        let z = libm::sinf(rad) * 3.0;

        push_identity();
        push_translate(x, 0.0, z);

        let colors = [0x44FF88FF, 0x4488FFFF, 0xFF44AAFF, 0xFFAA44FF];
        set_color(colors[i as usize]);
        draw_mesh(SPHERE_MESH);
    }
}

/// Draw the 3D scene with a color tint applied to all objects
unsafe fn draw_scene_tinted(tint: u32) {
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);
    camera_fov(60.0);

    // Extract tint RGB components (0-255)
    let tint_r = ((tint >> 24) & 0xFF) as f32 / 255.0;
    let tint_g = ((tint >> 16) & 0xFF) as f32 / 255.0;
    let tint_b = ((tint >> 8) & 0xFF) as f32 / 255.0;

    // Floor - tinted
    push_identity();
    push_translate(0.0, -1.0, 0.0);
    let floor_base = 0x404050FF_u32;
    set_color(tint_color(floor_base, tint_r, tint_g, tint_b));
    draw_mesh(FLOOR_MESH);

    // Central cube - tinted
    push_identity();
    push_rotate_y(ROTATION);
    set_color(tint_color(0xFF6644FF, tint_r, tint_g, tint_b));
    draw_mesh(CUBE_MESH);

    // Orbiting spheres - tinted
    let colors = [0x44FF88FF_u32, 0x4488FFFF, 0xFF44AAFF, 0xFFAA44FF];
    for i in 0..4 {
        let angle = TIME * 60.0 + (i as f32) * 90.0;
        let rad = angle.to_radians();
        let x = libm::cosf(rad) * 3.0;
        let z = libm::sinf(rad) * 3.0;

        push_identity();
        push_translate(x, 0.0, z);
        set_color(tint_color(colors[i], tint_r, tint_g, tint_b));
        draw_mesh(SPHERE_MESH);
    }
}

/// Apply a tint to a color
fn tint_color(color: u32, tint_r: f32, tint_g: f32, tint_b: f32) -> u32 {
    let r = ((color >> 24) & 0xFF) as f32 * tint_r;
    let g = ((color >> 16) & 0xFF) as f32 * tint_g;
    let b = ((color >> 8) & 0xFF) as f32 * tint_b;
    let a = color & 0xFF;
    ((r.min(255.0) as u32) << 24)
        | ((g.min(255.0) as u32) << 16)
        | ((b.min(255.0) as u32) << 8)
        | a
}

/// Demo 0: Circle mask - render scene only inside circle
unsafe fn demo_circle_mask() {
    // Draw circle shape to stencil buffer (mask creation)
    // Screen-space quads now use PassConfig depth settings, so depth_write=false in stencil_write
    begin_pass_stencil_write(1, 0);
    set_color(0xFFFFFFFF);
    draw_circle(SCREEN_CX, SCREEN_CY, 200.0);

    // Draw scene - only visible inside circle
    begin_pass_stencil_test(1, 0);
    draw_scene();

    // Return to normal rendering
    begin_pass(0);
}

/// Demo 1: Inverted mask - vignette effect
unsafe fn demo_inverted_mask() {
    // First draw the scene (visible everywhere)
    draw_scene();

    // Create circle mask
    begin_pass_stencil_write(1, 0);
    set_color(0xFFFFFFFF);
    draw_circle(SCREEN_CX, SCREEN_CY, 250.0);

    // Enable inverted stencil test (render where stencil != 1)
    begin_pass_full(
        compare::LESS,      // depth_compare
        1,                  // depth_write
        0,                  // clear_depth
        compare::NOT_EQUAL, // stencil_compare (inverted)
        1,                  // stencil_ref
        stencil_op::KEEP,   // stencil_pass_op
        stencil_op::KEEP,   // stencil_fail_op
        stencil_op::KEEP,   // stencil_depth_fail_op
    );

    // Draw dark vignette overlay only outside circle
    set_color(0x000000AA);
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);

    // Return to normal rendering
    begin_pass(0);
}

/// Demo 2: Diagonal split - different tints on each side
unsafe fn demo_diagonal_split() {
    // Create diagonal mask using draw_triangles with ortho projection
    begin_pass_stencil_write(1, 0);
    set_color(0xFFFFFFFF);

    // Set up orthographic projection for screen-space triangle
    // Ortho matrix: maps (0,0)-(width,height) to clip space (-1,1)
    let w = SCREEN_WIDTH;
    let h = SCREEN_HEIGHT;
    push_projection_matrix(
        2.0 / w, 0.0,      0.0,  0.0,  // column 0
        0.0,    -2.0 / h,  0.0,  0.0,  // column 1
        0.0,     0.0,     -1.0,  0.0,  // column 2
       -1.0,     1.0,      0.0,  1.0,  // column 3
    );

    // Identity view matrix (camera at origin looking at +Z)
    push_view_matrix(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    // Draw triangle covering diagonal (top-left to bottom-right split)
    // Vertices: (0,0) -> (width,0) -> (0,height)
    let vertices: [f32; 9] = [
        0.0, 0.0, 0.0,    // top-left
        w,   0.0, 0.0,    // top-right
        0.0, h,   0.0,    // bottom-left
    ];
    push_identity();
    draw_triangles(vertices.as_ptr(), 3, 0);

    // Draw top-left half with warm tint
    begin_pass_stencil_test(1, 0);
    draw_scene_tinted(0xFFCC99FF); // Warm orange tint

    // Bottom-right half - invert stencil to draw other side
    begin_pass_full(
        compare::LESS,      // depth_compare
        1,                  // depth_write
        0,                  // clear_depth
        compare::NOT_EQUAL, // stencil_compare (inverted)
        1,                  // stencil_ref
        stencil_op::KEEP,   // stencil_pass_op
        stencil_op::KEEP,   // stencil_fail_op
        stencil_op::KEEP,   // stencil_depth_fail_op
    );
    draw_scene_tinted(0x99CCFFFF); // Cool blue tint

    begin_pass(0);
}

/// Demo 3: Animated portal effect
unsafe fn demo_animated_portal() {
    // Animate the circle radius
    let base_radius = 150.0;
    let pulse = libm::sinf(TIME * 3.0) * 30.0;
    let radius = base_radius + pulse;

    // 1. Draw scene outside portal (inverted stencil)
    begin_pass_stencil_write(1, 0);
    set_color(0xFFFFFFFF);
    draw_circle(SCREEN_CX, SCREEN_CY, radius);

    // Enable inverted stencil test (render outside portal)
    begin_pass_full(
        compare::LESS,      // depth_compare
        1,                  // depth_write
        0,                  // clear_depth
        compare::NOT_EQUAL, // stencil_compare (inverted)
        1,                  // stencil_ref
        stencil_op::KEEP,   // stencil_pass_op
        stencil_op::KEEP,   // stencil_fail_op
        stencil_op::KEEP,   // stencil_depth_fail_op
    );
    draw_scene();

    // 2. Draw portal ring (NO depth write - decorative only)
    // Must not write depth or it will block the inner portal scene
    begin_pass_full(
        compare::ALWAYS,    // depth_compare - always pass
        0,                  // depth_write = FALSE (critical!)
        0,                  // clear_depth
        compare::ALWAYS,    // stencil_compare
        0,                  // stencil_ref
        stencil_op::KEEP,   // stencil_pass_op
        stencil_op::KEEP,   // stencil_fail_op
        stencil_op::KEEP,   // stencil_depth_fail_op
    );
    let ring_width = 6.0;
    set_color(0x8800FFFF);
    draw_circle(SCREEN_CX, SCREEN_CY, radius + ring_width / 2.0);
    set_color(0x000000FF);
    draw_circle(SCREEN_CX, SCREEN_CY, radius - ring_width / 2.0);

    // 3. Draw portal interior (zoomed view, with depth clear)
    begin_pass_stencil_write(1, 0);
    set_color(0xFFFFFFFF);
    draw_circle(SCREEN_CX, SCREEN_CY, radius - ring_width / 2.0);

    // Render inside portal with depth clear for proper 3D view
    begin_pass_stencil_test(1, 1); // clear_depth = 1 for portal interior
    draw_scene_with_camera(0.0, 2.0, 5.0, 90.0);

    // Return to normal rendering
    begin_pass(0);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Run current demo (each demo handles its own scene drawing)
        match DEMO_MODE {
            0 => demo_circle_mask(),
            1 => demo_inverted_mask(),
            2 => demo_diagonal_split(),
            3 => demo_animated_portal(),
            _ => {}
        }

        // UI overlay - Title
        let title = "STENCIL DEMO";
        set_color(0xFFFFFFFF,
        );
        draw_text(
            title.as_ptr(), title.len() as u32, 10.0, 10.0, 28.0);

        // Current demo name
        let demo_name = DEMO_NAMES[DEMO_MODE as usize];
        set_color(0x88FF88FF,
        );
        draw_text(
            demo_name.as_ptr(), demo_name.len() as u32, 10.0, 45.0, 20.0);

        // Demo description
        let demo_desc = DEMO_DESCRIPTIONS[DEMO_MODE as usize];
        set_color(0xCCCCCCFF,
        );
        draw_text(
            demo_desc.as_ptr(), demo_desc.len() as u32, 10.0, 70.0, 14.0);

        // Controls
        let controls = "Controls: A = Next Demo | Left Stick = Rotate Scene";
        set_color(0xAAAAAAFF,
        );
        draw_text(
            controls.as_ptr(), controls.len() as u32, 10.0, SCREEN_HEIGHT - 30.0, 14.0);

        // Explanation
        let explanation = "Stencil buffer masks which pixels can be drawn";
        set_color(0x888888FF,
        );
        draw_text(
            explanation.as_ptr(), explanation.len() as u32, 10.0, SCREEN_HEIGHT - 50.0, 12.0);
    }
}
