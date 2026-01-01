//! Portal Demo Example
//!
//! Demonstrates portal rendering using stencil masking.
//! Two portals connect different environments - look through one
//! to see what's on the other side.
//!
//! Controls:
//! - Left stick: Walk around
//! - Right stick: Look around
//! - Walk into portal to teleport

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


// Vertex format
const VF_POS: u32 = 0;

// Mesh handles
static mut CUBE: u32 = 0;
static mut SPHERE: u32 = 0;
static mut FLOOR: u32 = 0;
static mut PORTAL_RING: u32 = 0;

// Player state
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Z: f32 = 0.0;
static mut PLAYER_YAW: f32 = 0.0;
static mut PLAYER_PITCH: f32 = 0.0;
static mut CURRENT_WORLD: u32 = 0; // 0 = blue world, 1 = orange world

// Portal positions
const PORTAL1_X: f32 = 0.0;
const PORTAL1_Z: f32 = -10.0;
const PORTAL2_X: f32 = 0.0;
const PORTAL2_Z: f32 = 10.0;

// Animation
static mut TIME: f32 = 0.0;

// Portal disc mesh (flat circle for stencil mask)
const PORTAL_SEGMENTS: usize = 32;
static mut PORTAL_DISC: [f32; PORTAL_SEGMENTS * 3 * 3] = [0.0; PORTAL_SEGMENTS * 3 * 3];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(0);
        depth_test(1);

        // Generate meshes
        CUBE = cube(1.0, 1.0, 1.0);
        SPHERE = sphere(0.5, 16, 8);
        FLOOR = plane(40.0, 40.0, 8, 8);
        PORTAL_RING = torus(2.5, 0.2, 32, 8);

        // Generate portal disc mesh (vertical circle at origin, facing +Z)
        generate_portal_disc();

        // Start in blue world
        CURRENT_WORLD = 0;
    }
}

/// Generate a vertical disc mesh for portal stencil mask
unsafe fn generate_portal_disc() {
    let radius = 2.3; // Slightly smaller than ring
    let angle_step = core::f32::consts::TAU / PORTAL_SEGMENTS as f32;

    for i in 0..PORTAL_SEGMENTS {
        let angle1 = i as f32 * angle_step;
        let angle2 = (i + 1) as f32 * angle_step;

        // Vertical disc (X-Y plane at Z=0)
        let x1 = libm::cosf(angle1) * radius;
        let y1 = libm::sinf(angle1) * radius;
        let x2 = libm::cosf(angle2) * radius;
        let y2 = libm::sinf(angle2) * radius;

        let base = i * 9;
        // Center
        PORTAL_DISC[base + 0] = 0.0;
        PORTAL_DISC[base + 1] = 0.0;
        PORTAL_DISC[base + 2] = 0.0;
        // Edge 1
        PORTAL_DISC[base + 3] = x1;
        PORTAL_DISC[base + 4] = y1;
        PORTAL_DISC[base + 5] = 0.0;
        // Edge 2
        PORTAL_DISC[base + 6] = x2;
        PORTAL_DISC[base + 7] = y2;
        PORTAL_DISC[base + 8] = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Camera control with right stick
        let rx = right_stick_x(0);
        let ry = right_stick_y(0);
        PLAYER_YAW += rx * 2.0;
        PLAYER_PITCH -= ry * 1.5;
        if PLAYER_PITCH > 45.0 { PLAYER_PITCH = 45.0; }
        if PLAYER_PITCH < -45.0 { PLAYER_PITCH = -45.0; }

        // Movement with left stick (relative to camera direction)
        let lx = left_stick_x(0);
        let ly = left_stick_y(0);

        let yaw_rad = PLAYER_YAW.to_radians();
        let forward_x = libm::sinf(yaw_rad);
        let forward_z = libm::cosf(yaw_rad);
        let right_x = forward_z;
        let right_z = -forward_x;

        let speed = 0.15;
        PLAYER_X += (forward_x * ly + right_x * lx) * speed;
        PLAYER_Z += (forward_z * ly + right_z * lx) * speed;

        // Check portal collision
        let portal_radius = 2.0;

        // Portal 1 (blue side)
        let dx1 = PLAYER_X - PORTAL1_X;
        let dz1 = PLAYER_Z - PORTAL1_Z;
        if dx1 * dx1 + dz1 * dz1 < portal_radius * portal_radius {
            // Teleport to other side
            PLAYER_Z = PORTAL2_Z + 3.0;
            CURRENT_WORLD = 1;
        }

        // Portal 2 (orange side)
        let dx2 = PLAYER_X - PORTAL2_X;
        let dz2 = PLAYER_Z - PORTAL2_Z;
        if dx2 * dx2 + dz2 * dz2 < portal_radius * portal_radius {
            // Teleport to other side
            PLAYER_Z = PORTAL1_Z - 3.0;
            CURRENT_WORLD = 0;
        }
    }
}

/// Set camera based on player position and look direction
unsafe fn setup_camera() {
    let yaw_rad = PLAYER_YAW.to_radians();
    let pitch_rad = PLAYER_PITCH.to_radians();

    let cos_pitch = libm::cosf(pitch_rad);
    let sin_pitch = libm::sinf(pitch_rad);

    let look_x = PLAYER_X + libm::sinf(yaw_rad) * cos_pitch * 10.0;
    let look_y = 1.8 + sin_pitch * 10.0;
    let look_z = PLAYER_Z + libm::cosf(yaw_rad) * cos_pitch * 10.0;

    camera_set(PLAYER_X, 1.8, PLAYER_Z, look_x, look_y, look_z);
    camera_fov(75.0);
}

/// Set camera for portal view - looking from destination portal
/// src_portal: the portal we're looking through
/// dst_portal: the portal we'd exit from (camera position)
unsafe fn setup_portal_camera(src_x: f32, src_z: f32, dst_x: f32, dst_z: f32) {
    // Calculate relative position from source portal
    let rel_x = PLAYER_X - src_x;
    let rel_z = PLAYER_Z - src_z;

    // Mirror the position to the destination portal
    // Portals face opposite directions, so we negate the Z offset
    let cam_x = dst_x + rel_x;
    let cam_z = dst_z - rel_z;

    // Mirror the yaw (looking in opposite direction through portal)
    let mirrored_yaw = PLAYER_YAW + 180.0;
    let yaw_rad = mirrored_yaw.to_radians();
    let pitch_rad = PLAYER_PITCH.to_radians();

    let cos_pitch = libm::cosf(pitch_rad);
    let sin_pitch = libm::sinf(pitch_rad);

    let look_x = cam_x + libm::sinf(yaw_rad) * cos_pitch * 10.0;
    let look_y = 1.8 + sin_pitch * 10.0;
    let look_z = cam_z + libm::cosf(yaw_rad) * cos_pitch * 10.0;

    camera_set(cam_x, 1.8, cam_z, look_x, look_y, look_z);
    camera_fov(75.0);
}

/// Draw blue world (world 0)
unsafe fn draw_blue_world() {
    // Blue sky gradient
    env_gradient(0, 0x1144AAFF, 0x4488FFFF, 0x4488FFFF, 0x223366FF, 0.0, 0.0);
    draw_env();

    // Blue floor
    push_identity();
    set_color(0x223366FF);
    draw_mesh(FLOOR);

    // Blue cubes
    for i in 0..5 {
        let angle = (i as f32) * 72.0 + TIME * 20.0;
        let rad = angle.to_radians();
        let x = libm::cosf(rad) * 8.0;
        let z = libm::sinf(rad) * 8.0 - 10.0;

        push_identity();
        push_translate(x, 1.0, z);
        push_rotate_y(TIME * 50.0);
        set_color(0x4488FFFF);
        draw_mesh(CUBE);
    }

    // Blue sphere pillars
    for x in [-15.0, 15.0].iter() {
        for z in [-15.0, 5.0].iter() {
            push_identity();
            push_translate(*x, 2.0, *z);
            set_color(0x66AAFFFF);
            push_scale(1.0, 4.0, 1.0);
            draw_mesh(CUBE);
        }
    }
}

/// Draw orange world (world 1)
unsafe fn draw_orange_world() {
    // Orange sky gradient
    env_gradient(0, 0xAA4411FF, 0xFF8844FF, 0xFF8844FF, 0x663322FF, 0.0, 0.0);
    draw_env();

    // Orange floor
    push_identity();
    set_color(0x663322FF);
    draw_mesh(FLOOR);

    // Orange spheres
    for i in 0..8 {
        let angle = (i as f32) * 45.0 - TIME * 15.0;
        let rad = angle.to_radians();
        let x = libm::cosf(rad) * 6.0;
        let z = libm::sinf(rad) * 6.0 + 10.0;

        push_identity();
        push_translate(x, 1.5, z);
        let bounce = libm::fabsf(libm::sinf(TIME * 3.0 + i as f32));
        push_translate(0.0, bounce * 1.5, 0.0);
        set_color(0xFF8844FF);
        draw_mesh(SPHERE);
    }

    // Tall pillars
    for x in [-12.0, 12.0].iter() {
        push_identity();
        push_translate(*x, 3.0, 20.0);
        push_scale(1.5, 6.0, 1.5);
        set_color(0xFFAA66FF);
        draw_mesh(CUBE);
    }
}

/// Draw the portal frame and effect
unsafe fn draw_portal(x: f32, z: f32, color: u32, is_portal1: bool) {
    // Swirling effect
    let rotation = TIME * 60.0 * if is_portal1 { 1.0 } else { -1.0 };

    // Portal ring
    push_identity();
    push_translate(x, 2.3, z);
    push_rotate_x(90.0); // Vertical portal
    push_rotate_y(rotation);
    set_color(color);
    draw_mesh(PORTAL_RING);

    // Inner glow
    push_identity();
    push_translate(x, 2.3, z);
    push_rotate_x(90.0);
    push_scale(0.9, 0.9, 0.9);
    push_rotate_y(-rotation * 1.5);
    set_color((color & 0xFFFFFF00) | 0x88); // Semi-transparent
    draw_mesh(PORTAL_RING);
}

/// Draw portal disc for stencil mask
unsafe fn draw_portal_mask(x: f32, z: f32) {
    push_identity();
    push_translate(x, 2.3, z);
    // Portal faces the player (approximation - should face camera)
    let to_player_x = PLAYER_X - x;
    let to_player_z = PLAYER_Z - z;
    let angle = libm::atan2f(to_player_x, to_player_z).to_degrees();
    push_rotate_y(angle);

    draw_triangles(
        PORTAL_DISC.as_ptr(),
        (PORTAL_SEGMENTS * 3) as u32,
        VF_POS,
    );
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        setup_camera();

        // Determine which world we're in and which portal to look through
        if CURRENT_WORLD == 0 {
            // In blue world, looking through portal1 shows orange world
            draw_blue_world();

            // Draw portal1 showing orange world
            stencil_begin();
            draw_portal_mask(PORTAL1_X, PORTAL1_Z);
            stencil_end();

            // Inside portal: render orange world from portal2's perspective
            // Set camera as if looking from the destination portal
            setup_portal_camera(PORTAL1_X, PORTAL1_Z, PORTAL2_X, PORTAL2_Z);
            depth_test(0);  // Disable depth test
            draw_orange_world();
            depth_test(1);  // Re-enable depth test

            stencil_clear();

            // Restore main camera for portal frame
            setup_camera();

            // Draw portal frames on top
            draw_portal(PORTAL1_X, PORTAL1_Z, 0xFF8844FF, true); // Orange portal (leads to orange)
        } else {
            // In orange world, looking through portal2 shows blue world
            draw_orange_world();

            // Draw portal2 showing blue world
            stencil_begin();
            draw_portal_mask(PORTAL2_X, PORTAL2_Z);
            stencil_end();

            // Inside portal: render blue world from portal1's perspective
            setup_portal_camera(PORTAL2_X, PORTAL2_Z, PORTAL1_X, PORTAL1_Z);
            depth_test(0);  // Disable depth test
            draw_blue_world();
            depth_test(1);  // Re-enable depth test

            stencil_clear();

            // Restore main camera for portal frame
            setup_camera();

            // Draw portal frame
            draw_portal(PORTAL2_X, PORTAL2_Z, 0x4488FFFF, false); // Blue portal (leads to blue)
        }

        // UI - Title
        let title = "PORTAL STENCIL DEMO";
        set_color(0xFFFFFFFF,
        );
        draw_text(
            title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

        // Current dimension
        let world_text = if CURRENT_WORLD == 0 { "Current: Blue Dimension" } else { "Current: Orange Dimension" };
        set_color(if CURRENT_WORLD == 0 { 0x4488FFFF } else { 0xFF8844FF },
        );
        draw_text(
            world_text.as_ptr(), world_text.len() as u32, 10.0, 40.0, 16.0);

        // Explanation
        let explain1 = "Portals use stencil masking to show other dimension";
        set_color(0x888888FF,
        );
        draw_text(
            explain1.as_ptr(), explain1.len() as u32, 10.0, 65.0, 12.0);
        let explain2 = "Camera transforms to destination portal perspective";
        set_color(0x888888FF,
        );
        draw_text(
            explain2.as_ptr(), explain2.len() as u32, 10.0, 80.0, 12.0);

        // Controls at bottom
        let controls = "Controls: Left Stick = Move | Right Stick = Look";
        set_color(0xAAAAAAFF,
        );
        draw_text(
            controls.as_ptr(), controls.len() as u32, 10.0, 500.0, 14.0);

        let instr = "Walk into the portal ring to teleport!";
        set_color(0xCCCCCCFF,
        );
        draw_text(
            instr.as_ptr(), instr.len() as u32, 10.0, 520.0, 14.0);
    }
}
