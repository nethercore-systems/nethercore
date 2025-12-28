//! Stencil Demo Example
//!
//! Demonstrates all 4 stencil masking modes:
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

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Stencil functions
    fn stencil_begin();
    fn stencil_end();
    fn stencil_clear();
    fn stencil_invert();

    // Input
    fn button_pressed(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_circle(x: f32, y: f32, radius: f32, color: u32);
}

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
    "stencil_begin/end: Only render inside circle",
    "stencil_invert: Render outside mask (dark edges)",
    "Two masks: Different tints per half",
    "Pulsing portal with different camera inside",
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(0); // Lambert mode
        depth_test(1);

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
    // First, fill the background with black (this will be visible outside the mask)
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000FF);

    // Draw circle shape to stencil buffer
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, 200.0, 0xFFFFFFFF);
    stencil_end();

    // Draw scene - only visible inside circle
    draw_scene();

    // Return to normal rendering
    stencil_clear();
}

/// Demo 1: Inverted mask - vignette effect
unsafe fn demo_inverted_mask() {
    // First draw the scene (visible everywhere)
    draw_scene();

    // Create circle mask
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, 250.0, 0xFFFFFFFF);
    stencil_end();

    // Invert the mask to render OUTSIDE the circle
    stencil_invert();

    // Draw dark vignette overlay only outside circle
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000AA);

    // Return to normal rendering
    stencil_clear();
}

/// Demo 2: Diagonal split - different tints on each side
unsafe fn demo_diagonal_split() {
    // First half: top-left triangle (using a rect for simplicity, we'll mask diagonally)
    // For a true diagonal, we'd use draw_triangles, but let's do left/right split instead
    // which is simpler and still demonstrates the concept

    // Left half mask
    stencil_begin();
    draw_rect(0.0, 0.0, SCREEN_WIDTH / 2.0, SCREEN_HEIGHT, 0xFFFFFFFF);
    stencil_end();

    // Draw scene with warm tint (left side)
    draw_scene_tinted(0xFFCC99FF); // Warm orange tint

    stencil_clear();

    // Right half mask
    stencil_begin();
    draw_rect(SCREEN_WIDTH / 2.0, 0.0, SCREEN_WIDTH / 2.0, SCREEN_HEIGHT, 0xFFFFFFFF);
    stencil_end();

    // Draw scene with cool tint (right side)
    draw_scene_tinted(0x99CCFFFF); // Cool blue tint

    stencil_clear();
}

/// Demo 3: Animated portal effect
unsafe fn demo_animated_portal() {
    // Animate the circle radius
    let base_radius = 150.0;
    let pulse = libm::sinf(TIME * 3.0) * 30.0;
    let radius = base_radius + pulse;
    let ring_width = 6.0;

    // Step 1: Draw the portal ring border first (underneath everything)
    // Use outer circle mask, then invert to draw OUTSIDE it
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, radius, 0xFFFFFFFF);
    stencil_end();

    // Invert - now we can only draw OUTSIDE the inner circle
    stencil_invert();

    // Mask additionally by inner edge of ring (we want ring area only)
    // Draw the ring color as a full-screen rect, but it only appears outside inner circle
    // We need to also limit to inside outer circle - use nested approach

    // Actually simpler: just draw the ring as a circle outline
    stencil_clear();

    // Draw outer scene first (normal view)
    // Mask: everything EXCEPT the portal area
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, radius, 0xFFFFFFFF);
    stencil_end();
    stencil_invert(); // Draw OUTSIDE the portal

    draw_scene(); // Normal scene outside portal

    stencil_clear();

    // Draw portal ring (between radius and radius+ring_width)
    // Outer circle mask
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, radius + ring_width, 0xFFFFFFFF);
    stencil_end();

    // Draw purple, but only inside outer circle
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x8800FFFF);

    stencil_clear();

    // Now cut out the inner part by drawing portal scene over it
    stencil_begin();
    draw_circle(SCREEN_CX, SCREEN_CY, radius, 0xFFFFFFFF);
    stencil_end();

    // Inside portal: close-up view with wider FOV
    draw_scene_with_camera(0.0, 2.0, 5.0, 90.0);

    stencil_clear();
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
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            10.0,
            10.0,
            28.0,
            0xFFFFFFFF,
        );

        // Current demo name
        let demo_name = DEMO_NAMES[DEMO_MODE as usize];
        draw_text(
            demo_name.as_ptr(),
            demo_name.len() as u32,
            10.0,
            45.0,
            20.0,
            0x88FF88FF,
        );

        // Demo description
        let demo_desc = DEMO_DESCRIPTIONS[DEMO_MODE as usize];
        draw_text(
            demo_desc.as_ptr(),
            demo_desc.len() as u32,
            10.0,
            70.0,
            14.0,
            0xCCCCCCFF,
        );

        // Controls
        let controls = "Controls: A = Next Demo | Left Stick = Rotate Scene";
        draw_text(
            controls.as_ptr(),
            controls.len() as u32,
            10.0,
            SCREEN_HEIGHT - 30.0,
            14.0,
            0xAAAAAAFF,
        );

        // Explanation
        let explanation = "Stencil buffer masks which pixels can be drawn";
        draw_text(
            explanation.as_ptr(),
            explanation.len() as u32,
            10.0,
            SCREEN_HEIGHT - 50.0,
            12.0,
            0x888888FF,
        );
    }
}
