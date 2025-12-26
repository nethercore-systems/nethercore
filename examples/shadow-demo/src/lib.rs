//! Shadow Demo Example
//!
//! Demonstrates stencil-based shadow projection.
//! Objects cast flat shadows onto the ground plane using stencil masking.
//!
//! Controls:
//! - Left stick: Rotate light source
//! - A button: Toggle between objects

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

    // Stencil
    fn stencil_begin();
    fn stencil_end();
    fn stencil_clear();

    // Input
    fn button_held(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;
    fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_rotate_x(angle_deg: f32);
    fn push_scale(x: f32, y: f32, z: f32);
    fn transform_set(matrix_ptr: *const f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // Environment
    fn env_gradient(
        layer: u32,
        zenith: u32,
        sky_horizon: u32,
        ground_horizon: u32,
        nadir: u32,
        rotation: f32,
        shift: f32,
    );
    fn draw_env();

    // 2D drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

// Input (button indices from zx.rs)
const BUTTON_A: u32 = 4;

// Mesh handles
static mut CUBE: u32 = 0;
static mut SPHERE: u32 = 0;
static mut FLOOR: u32 = 0;
static mut TORUS: u32 = 0;
static mut LIGHT_MARKER: u32 = 0;

// State
static mut LIGHT_ANGLE: f32 = 45.0;
static mut LIGHT_HEIGHT: f32 = 8.0;
static mut CURRENT_OBJECT: u32 = 0;
static mut PREV_A_BUTTON: u32 = 0;
static mut TIME: f32 = 0.0;

// Light distance from center
const LIGHT_DISTANCE: f32 = 8.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x87CEEBFF);
        render_mode(0);
        depth_test(1);

        // Generate meshes
        CUBE = cube(2.0, 2.0, 2.0);
        SPHERE = sphere(1.5, 24, 12);
        FLOOR = plane(20.0, 20.0, 4, 4);
        TORUS = torus(1.0, 0.4, 24, 12);
        LIGHT_MARKER = sphere(0.3, 8, 4);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Control light with stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            LIGHT_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            LIGHT_HEIGHT += stick_y * 0.1;
            if LIGHT_HEIGHT < 2.0 { LIGHT_HEIGHT = 2.0; }
            if LIGHT_HEIGHT > 15.0 { LIGHT_HEIGHT = 15.0; }
        }

        // Auto-rotate when stick is centered
        if stick_x.abs() < 0.1 && stick_y.abs() < 0.1 {
            LIGHT_ANGLE += 0.3;
        }

        // Toggle object with A
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            CURRENT_OBJECT = (CURRENT_OBJECT + 1) % 3;
        }
        PREV_A_BUTTON = a_button;
    }
}

/// Get light position
unsafe fn get_light_pos() -> (f32, f32, f32) {
    let rad = LIGHT_ANGLE.to_radians();
    let x = libm::cosf(rad) * LIGHT_DISTANCE;
    let z = libm::sinf(rad) * LIGHT_DISTANCE;
    (x, LIGHT_HEIGHT, z)
}

/// Create a shadow projection matrix for a plane at y=0
/// This projects vertices onto the ground plane based on light position.
///
/// Uses the standard planar shadow matrix: M = (n·L)I - L⊗n
/// For ground plane y=0 with normal n=(0,1,0) and light at L=(lx,ly,lz).
unsafe fn shadow_matrix(light_x: f32, light_y: f32, light_z: f32) -> [f32; 16] {
    let ly = light_y;

    // Shadow projection matrix (column-major for wgpu)
    // Transforms point P to its shadow position on y=0 plane
    // Result needs perspective divide: (x', 0, z') = (x/w, 0, z/w)
    [
        ly,        0.0, 0.0, 0.0,       // Column 0: x' = ly * x
        -light_x,  0.0, -light_z, -1.0, // Column 1: contribution from y (note: -1 for w)
        0.0,       0.0, ly,  0.0,       // Column 2: z' = ly * z
        0.0,       0.0, 0.0, ly,        // Column 3: w offset
    ]
}

/// Draw the current object at origin
unsafe fn draw_object() {
    push_identity();
    push_translate(0.0, 2.0, 0.0);
    push_rotate_y(TIME * 30.0);

    match CURRENT_OBJECT {
        0 => {
            set_color(0xFF4444FF);
            draw_mesh(CUBE);
        }
        1 => {
            set_color(0x44FF44FF);
            draw_mesh(SPHERE);
        }
        2 => {
            push_rotate_x(45.0);
            set_color(0x4444FFFF);
            draw_mesh(TORUS);
        }
        _ => {}
    }
}

/// Draw the object's shadow using stencil
unsafe fn draw_shadow() {
    let (lx, ly, lz) = get_light_pos();
    let shadow = shadow_matrix(lx, ly, lz);

    // First pass: write shadow shape to stencil
    stencil_begin();

    push_identity();
    // Apply shadow projection matrix
    transform_set(shadow.as_ptr());
    push_translate(0.0, 2.0, 0.0);
    push_rotate_y(TIME * 30.0);

    // Draw projected shadow geometry
    match CURRENT_OBJECT {
        0 => draw_mesh(CUBE),
        1 => draw_mesh(SPHERE),
        2 => {
            push_rotate_x(45.0);
            draw_mesh(TORUS);
        }
        _ => {}
    }

    stencil_end();

    // Second pass: draw dark quad only where stencil was written
    push_identity();
    push_translate(0.0, 0.01, 0.0); // Slightly above ground to prevent z-fighting
    set_color(0x00000080); // Semi-transparent black
    draw_mesh(FLOOR);

    stencil_clear();
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Camera
        camera_set(12.0, 10.0, 12.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Draw sky
        env_gradient(0, 0x4169E1FF, 0xADD8E6FF, 0xADD8E6FF, 0x87CEEBFF, 0.0, 0.0);
        draw_env();

        // Draw ground first
        push_identity();
        set_color(0x88AA88FF); // Light green
        draw_mesh(FLOOR);

        // Draw shadow (uses stencil)
        draw_shadow();

        // Draw the actual object on top
        draw_object();

        // Draw light marker
        let (lx, ly, lz) = get_light_pos();
        push_identity();
        push_translate(lx, ly, lz);
        set_color(0xFFFF00FF); // Yellow
        draw_mesh(LIGHT_MARKER);

        // Draw light ray (simple line to ground)
        push_identity();
        push_translate(lx, ly / 2.0, lz);
        push_scale(0.05, ly / 2.0, 0.05);
        set_color(0xFFFF0088);
        draw_mesh(CUBE);

        // UI - Title
        let title = "STENCIL SHADOW DEMO";
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            10.0,
            10.0,
            24.0,
            0xFFFFFFFF,
        );

        // Current object
        let objects = ["Cube", "Sphere", "Torus"];
        let obj_label = "Object: ";
        draw_text(
            obj_label.as_ptr(),
            obj_label.len() as u32,
            10.0,
            40.0,
            16.0,
            0xAAAAAAFF,
        );
        let obj_name = objects[CURRENT_OBJECT as usize];
        draw_text(
            obj_name.as_ptr(),
            obj_name.len() as u32,
            90.0,
            40.0,
            16.0,
            0x88FF88FF,
        );

        // Explanation
        let explain1 = "Shadows use stencil buffer to mask ground plane";
        draw_text(
            explain1.as_ptr(),
            explain1.len() as u32,
            10.0,
            70.0,
            12.0,
            0x888888FF,
        );
        let explain2 = "1. Draw projected shape to stencil";
        draw_text(
            explain2.as_ptr(),
            explain2.len() as u32,
            10.0,
            85.0,
            12.0,
            0x888888FF,
        );
        let explain3 = "2. Draw dark overlay where stencil was written";
        draw_text(
            explain3.as_ptr(),
            explain3.len() as u32,
            10.0,
            100.0,
            12.0,
            0x888888FF,
        );

        // Controls at bottom
        let controls = "Controls: Left Stick = Move Light | A = Change Object";
        draw_text(
            controls.as_ptr(),
            controls.len() as u32,
            10.0,
            500.0,
            14.0,
            0xAAAAAAFF,
        );

        // Light indicator
        let light_info = "Yellow sphere = Light source (orbiting)";
        draw_text(
            light_info.as_ptr(),
            light_info.len() as u32,
            10.0,
            520.0,
            12.0,
            0xFFFF88FF,
        );
    }
}
