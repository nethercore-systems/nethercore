//! Stencil Demo Example
//!
//! Demonstrates all 4 stencil masking modes:
//! - Circle mask (render inside)
//! - Inverted mask (render outside / vignette)
//! - Diagonal split
//! - Multiple masks
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
    fn button_held(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Immediate mode triangles
    fn draw_triangles(data_ptr: *const f32, vertex_count: u32, format: u32);

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
}

// Vertex format: position only (for stencil mask shapes)
const VF_POS: u32 = 0;

// Input
const BUTTON_A: u32 = 1;

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;
static mut FLOOR_MESH: u32 = 0;

// State
static mut DEMO_MODE: u32 = 0;
static mut PREV_A_BUTTON: u32 = 0;
static mut TIME: f32 = 0.0;
static mut ROTATION: f32 = 0.0;

// Circle mesh data (generated at init)
const CIRCLE_SEGMENTS: usize = 32;
static mut CIRCLE_VERTICES: [f32; CIRCLE_SEGMENTS * 3 * 3] = [0.0; CIRCLE_SEGMENTS * 3 * 3];

// Demo mode names
static DEMO_NAMES: [&str; 4] = [
    "Circle Mask (Render Inside)",
    "Inverted Mask (Vignette)",
    "Diagonal Split",
    "Animated Portal",
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(0);
        depth_test(1);

        // Generate scene objects
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        SPHERE_MESH = sphere(0.5, 16, 8);
        FLOOR_MESH = plane(10.0, 10.0, 4, 4);

        // Generate circle mesh vertices (triangle fan centered at screen center)
        generate_circle_mesh(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 200.0);
    }
}

/// Generate a circle mesh as a triangle fan (position only format)
unsafe fn generate_circle_mesh(cx: f32, cy: f32, radius: f32) {
    let angle_step = core::f32::consts::TAU / CIRCLE_SEGMENTS as f32;

    for i in 0..CIRCLE_SEGMENTS {
        let angle1 = i as f32 * angle_step;
        let angle2 = (i + 1) as f32 * angle_step;

        let x1 = cx + libm::cosf(angle1) * radius;
        let y1 = cy + libm::sinf(angle1) * radius;
        let x2 = cx + libm::cosf(angle2) * radius;
        let y2 = cy + libm::sinf(angle2) * radius;

        // Triangle: center, edge1, edge2 (CCW winding)
        let base = i * 9;
        CIRCLE_VERTICES[base + 0] = cx;
        CIRCLE_VERTICES[base + 1] = cy;
        CIRCLE_VERTICES[base + 2] = 0.0;
        CIRCLE_VERTICES[base + 3] = x1;
        CIRCLE_VERTICES[base + 4] = y1;
        CIRCLE_VERTICES[base + 5] = 0.0;
        CIRCLE_VERTICES[base + 6] = x2;
        CIRCLE_VERTICES[base + 7] = y2;
        CIRCLE_VERTICES[base + 8] = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Cycle demo modes with A button
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            DEMO_MODE = (DEMO_MODE + 1) % 4;
        }
        PREV_A_BUTTON = a_button;

        // Rotate scene with stick
        let stick_x = left_stick_x(0);
        if stick_x.abs() > 0.1 {
            ROTATION += stick_x * 2.0;
        } else {
            ROTATION += 0.5; // Auto-rotate
        }
    }
}

/// Draw the 3D scene
unsafe fn draw_scene() {
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);
    camera_fov(60.0);

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

/// Draw the circle mask shape to stencil buffer
unsafe fn draw_circle_mask() {
    // Use 2D camera for screen-space mask
    camera_set(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    camera_fov(90.0);

    push_identity();

    // Draw circle as immediate triangles
    draw_triangles(
        CIRCLE_VERTICES.as_ptr(),
        (CIRCLE_SEGMENTS * 3) as u32,
        VF_POS,
    );
}

/// Draw a diagonal triangle for split mask
unsafe fn draw_diagonal_mask(top_left: bool) {
    camera_set(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    camera_fov(90.0);

    push_identity();

    // Triangle covering half the screen diagonally
    let vertices: [f32; 9] = if top_left {
        [
            0.0, 0.0, 0.0,           // Top-left
            SCREEN_WIDTH, 0.0, 0.0,  // Top-right
            0.0, SCREEN_HEIGHT, 0.0, // Bottom-left
        ]
    } else {
        [
            SCREEN_WIDTH, 0.0, 0.0,           // Top-right
            SCREEN_WIDTH, SCREEN_HEIGHT, 0.0, // Bottom-right
            0.0, SCREEN_HEIGHT, 0.0,          // Bottom-left
        ]
    };

    draw_triangles(vertices.as_ptr(), 3, VF_POS);
}

/// Demo 0: Circle mask - render scene only inside circle
unsafe fn demo_circle_mask() {
    // Draw circle shape to stencil buffer
    stencil_begin();
    draw_circle_mask();
    stencil_end();

    // Draw scene - only visible inside circle
    draw_scene();

    // Return to normal rendering
    stencil_clear();

    // Draw outside area with solid color
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000FF);
}

/// Demo 1: Inverted mask - vignette effect
unsafe fn demo_inverted_mask() {
    // Draw circle shape to stencil buffer
    stencil_begin();
    draw_circle_mask();
    stencil_invert(); // Render OUTSIDE the mask

    // Draw dark vignette overlay only outside circle
    set_color(0x000000AA);
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000AA);

    // Return to normal rendering
    stencil_clear();

    // Draw scene (visible everywhere, vignette was already applied)
    draw_scene();
}

/// Demo 2: Diagonal split - different rendering on each side
unsafe fn demo_diagonal_split() {
    // First half: top-left triangle
    stencil_begin();
    draw_diagonal_mask(true);
    stencil_end();

    // Draw scene with warm tint
    set_color(0xFFAA88FF);
    draw_scene();

    stencil_clear();

    // Second half: bottom-right triangle
    stencil_begin();
    draw_diagonal_mask(false);
    stencil_end();

    // Draw scene with cool tint
    set_color(0x88AAFFFF);
    draw_scene();

    stencil_clear();
}

/// Demo 3: Animated portal effect
unsafe fn demo_animated_portal() {
    // Animate the circle radius
    let base_radius = 150.0;
    let pulse = libm::sinf(TIME * 3.0) * 30.0;
    let radius = base_radius + pulse;

    // Regenerate circle with animated radius
    generate_circle_mesh(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, radius);

    // Draw portal mask
    stencil_begin();
    draw_circle_mask();
    stencil_end();

    // Inside portal: zoomed-in scene with different camera
    camera_set(0.0, 2.0, 5.0, 0.0, 0.0, 0.0);
    camera_fov(90.0);

    // Purple tinted "other dimension"
    set_color(0xAA44FFFF);
    draw_scene();

    stencil_clear();

    // Draw portal ring border
    let ring_color = 0x8800FFFF;
    let ring_width = 4.0;

    // Draw outer edge (larger circle)
    generate_circle_mesh(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, radius + ring_width);
    stencil_begin();
    draw_circle_mask();
    stencil_invert();

    // Inner part should be cut out
    generate_circle_mesh(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, radius);
    stencil_begin();
    draw_circle_mask();
    stencil_end();

    set_color(ring_color);
    draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, ring_color);

    stencil_clear();

    // Restore circle for next frame
    generate_circle_mesh(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, radius);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Clear background first for vignette demo
        if DEMO_MODE == 1 {
            // Draw scene first for vignette (scene is behind the vignette)
            draw_scene();
        }

        // Run current demo
        match DEMO_MODE {
            0 => demo_circle_mask(),
            1 => demo_inverted_mask(),
            2 => demo_diagonal_split(),
            3 => demo_animated_portal(),
            _ => {}
        }

        // UI overlay
        let demo_name = DEMO_NAMES[DEMO_MODE as usize];
        draw_text(
            demo_name.as_ptr(),
            demo_name.len() as u32,
            10.0,
            10.0,
            24.0,
            0xFFFFFFFF,
        );

        let instruction = "Press A to cycle demos";
        draw_text(
            instruction.as_ptr(),
            instruction.len() as u32,
            10.0,
            40.0,
            16.0,
            0xAAAAAAFF,
        );
    }
}
