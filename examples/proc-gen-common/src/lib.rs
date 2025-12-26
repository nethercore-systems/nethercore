//! Procedural Generation Viewer - Shared Library
//!
//! This library contains all the viewer logic and provides a macro for
//! creating viewers with different render modes.
//!
//! Usage in wrapper crate (must have #![no_std] and #![no_main] at file top):
//! ```ignore
//! #![no_std]
//! #![no_main]
//! proc_gen_common::viewer!(2, "Metallic Roughness");
//! ```

#![no_std]

pub use libm::{sinf, cosf};

// Re-export core items wrapper crates need
pub use core::panic::PanicInfo;

/// Generate a complete proc-gen viewer with the specified render mode.
///
/// This macro generates all FFI declarations, state, and exported functions.
/// The render mode is configured at compile time in the init() function.
///
/// IMPORTANT: The calling crate MUST have `#![no_std]` and `#![no_main]`
/// as the first lines of the file, BEFORE invoking this macro.
#[macro_export]
macro_rules! viewer {
    ($render_mode:expr, $mode_name:expr) => {
        use proc_gen_common::{sinf, cosf, PanicInfo};

        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            core::arch::wasm32::unreachable()
        }

        // =============================================================================
        // FFI Declarations
        // =============================================================================

        #[link(wasm_import_module = "env")]
        extern "C" {
            // Configuration
            fn set_clear_color(color: u32);
            fn render_mode(mode: u32);

            // Camera
            fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
            fn camera_fov(fov_degrees: f32);

            // Input
            fn button_pressed(player: u32, button: u32) -> u32;
            fn left_stick_x(player: u32) -> f32;
            fn left_stick_y(player: u32) -> f32;
            fn right_stick_y(player: u32) -> f32;

            // Procedural mesh generation
            fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
            fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
            fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;
            fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;
            fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;
            fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

            // Mesh drawing
            fn draw_mesh(handle: u32);

            // Transform
            fn push_identity();
            fn push_translate(x: f32, y: f32, z: f32);
            fn push_rotate_x(angle_deg: f32);
            fn push_rotate_y(angle_deg: f32);
            fn push_scale(x: f32, y: f32, z: f32);

            // Render state
            fn set_color(color: u32);
            fn depth_test(enabled: u32);

            // Lighting
            fn light_set(index: u32, x: f32, y: f32, z: f32);
            fn light_color(index: u32, color: u32);
            fn light_intensity(index: u32, intensity: f32);
            fn light_enable(index: u32);

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
            fn env_lines(
                layer: u32,
                variant: u32,
                line_type: u32,
                thickness: u32,
                spacing: f32,
                fade_distance: f32,
                color_primary: u32,
                color_accent: u32,
                accent_every: u32,
                phase: u32,
            );
            fn draw_env();

            // 2D UI
            fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
            fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
        }

        // =============================================================================
        // Constants
        // =============================================================================

        // Button indices
        const BUTTON_A: u32 = 4;
        const BUTTON_X: u32 = 6;
        const BUTTON_Y: u32 = 7;
        const BUTTON_L1: u32 = 8;
        const BUTTON_R1: u32 = 9;
        const BUTTON_START: u32 = 12;

        // Shape names
        const SHAPE_NAMES: [&str; 6] = ["Cube", "Sphere", "Cylinder", "Plane", "Torus", "Capsule"];

        // Number of shapes
        const NUM_SHAPES: u32 = 6;

        // Number of subdivision levels
        const NUM_SUBDIV_LEVELS: u32 = 3;

        // Vertex/triangle counts per shape per subdivision level
        static MESH_STATS: [[(u32, u32); 3]; 6] = [
            // Cube: 24 verts, 12 tris at base
            [(24, 12), (96, 48), (384, 192)],
            // Sphere at 8x4, 16x8, 32x16
            [(36, 64), (136, 256), (528, 1024)],
            // Cylinder at 8, 16, 32 segments
            [(32, 28), (64, 60), (128, 124)],
            // Plane at 2x2, 4x4, 8x8
            [(9, 8), (25, 32), (81, 128)],
            // Torus at 8x4, 16x8, 32x16
            [(32, 64), (128, 256), (512, 1024)],
            // Capsule at 8x2, 16x4, 32x8
            [(48, 80), (160, 288), (576, 1088)],
        ];

        // =============================================================================
        // State
        // =============================================================================

        // Camera orbit state
        static mut CAMERA_YAW: f32 = 0.0;
        static mut CAMERA_PITCH: f32 = 20.0;
        static mut CAMERA_DISTANCE: f32 = 5.0;

        // View state
        static mut CURRENT_SHAPE: u32 = 0;
        static mut CURRENT_SUBDIV: u32 = 0;
        static mut SHOW_GRID: bool = true;

        // Mesh handles: [shape_index][subdiv_level]
        static mut MESH_HANDLES: [[u32; 3]; 6] = [[0; 3]; 6];

        // Grid mesh handle
        static mut GRID_MESH: u32 = 0;

        // =============================================================================
        // Init
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn init() {
            unsafe {
                // Configure rendering
                set_clear_color(0x1a1a2eFF);
                render_mode($render_mode);  // Configurable render mode
                depth_test(1);

                // Generate meshes at different subdivision levels
                // Level 0: Low poly
                MESH_HANDLES[0][0] = cube(1.0, 1.0, 1.0);
                MESH_HANDLES[1][0] = sphere(1.2, 8, 4);
                MESH_HANDLES[2][0] = cylinder(0.8, 0.8, 2.0, 8);
                MESH_HANDLES[3][0] = plane(2.5, 2.5, 2, 2);
                MESH_HANDLES[4][0] = torus(1.0, 0.4, 8, 4);
                MESH_HANDLES[5][0] = capsule(0.6, 1.2, 8, 2);

                // Level 1: Medium poly
                MESH_HANDLES[0][1] = cube(1.0, 1.0, 1.0);
                MESH_HANDLES[1][1] = sphere(1.2, 16, 8);
                MESH_HANDLES[2][1] = cylinder(0.8, 0.8, 2.0, 16);
                MESH_HANDLES[3][1] = plane(2.5, 2.5, 4, 4);
                MESH_HANDLES[4][1] = torus(1.0, 0.4, 16, 8);
                MESH_HANDLES[5][1] = capsule(0.6, 1.2, 16, 4);

                // Level 2: High poly
                MESH_HANDLES[0][2] = cube(1.0, 1.0, 1.0);
                MESH_HANDLES[1][2] = sphere(1.2, 32, 16);
                MESH_HANDLES[2][2] = cylinder(0.8, 0.8, 2.0, 32);
                MESH_HANDLES[3][2] = plane(2.5, 2.5, 8, 8);
                MESH_HANDLES[4][2] = torus(1.0, 0.4, 32, 16);
                MESH_HANDLES[5][2] = capsule(0.6, 1.2, 32, 8);

                // Grid for floor reference
                GRID_MESH = plane(10.0, 10.0, 20, 20);

                // Set up lighting
                light_set(0, 0.5, -0.7, 0.5);
                light_color(0, 0xFFF2E6FF);
                light_intensity(0, 1.5);
                light_enable(0);

                light_set(1, -0.5, -0.3, -0.5);
                light_color(1, 0x99B3FFFF);
                light_intensity(1, 0.8);
                light_enable(1);
            }
        }

        // =============================================================================
        // Update
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn update() {
            unsafe {
                // Camera orbit control (left stick)
                let stick_x = left_stick_x(0);
                let stick_y = left_stick_y(0);

                if stick_x.abs() > 0.1 {
                    CAMERA_YAW += stick_x * 2.5;
                }
                if stick_y.abs() > 0.1 {
                    CAMERA_PITCH += stick_y * 2.0;
                    if CAMERA_PITCH > 89.0 {
                        CAMERA_PITCH = 89.0;
                    }
                    if CAMERA_PITCH < -89.0 {
                        CAMERA_PITCH = -89.0;
                    }
                }

                // Camera distance control (right stick Y)
                let rstick_y = right_stick_y(0);
                if rstick_y.abs() > 0.1 {
                    CAMERA_DISTANCE += rstick_y * 0.1;
                    if CAMERA_DISTANCE < 2.0 {
                        CAMERA_DISTANCE = 2.0;
                    }
                    if CAMERA_DISTANCE > 15.0 {
                        CAMERA_DISTANCE = 15.0;
                    }
                }

                // A button: Toggle grid
                if button_pressed(0, BUTTON_A) != 0 {
                    SHOW_GRID = !SHOW_GRID;
                }

                // X button: Increase subdivision
                if button_pressed(0, BUTTON_X) != 0 {
                    if CURRENT_SUBDIV < NUM_SUBDIV_LEVELS - 1 {
                        CURRENT_SUBDIV += 1;
                    }
                }

                // Y button: Decrease subdivision
                if button_pressed(0, BUTTON_Y) != 0 {
                    if CURRENT_SUBDIV > 0 {
                        CURRENT_SUBDIV -= 1;
                    }
                }

                // L1/R1: Cycle shapes
                if button_pressed(0, BUTTON_R1) != 0 {
                    CURRENT_SHAPE = (CURRENT_SHAPE + 1) % NUM_SHAPES;
                }
                if button_pressed(0, BUTTON_L1) != 0 {
                    if CURRENT_SHAPE == 0 {
                        CURRENT_SHAPE = NUM_SHAPES - 1;
                    } else {
                        CURRENT_SHAPE -= 1;
                    }
                }

                // START: Reset camera
                if button_pressed(0, BUTTON_START) != 0 {
                    CAMERA_YAW = 0.0;
                    CAMERA_PITCH = 20.0;
                    CAMERA_DISTANCE = 5.0;
                }
            }
        }

        // =============================================================================
        // Render
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn render() {
            unsafe {
                // Calculate camera position from orbit parameters
                let yaw_rad = CAMERA_YAW * 0.01745329;
                let pitch_rad = CAMERA_PITCH * 0.01745329;

                let cam_x = CAMERA_DISTANCE * cosf(pitch_rad) * sinf(yaw_rad);
                let cam_y = CAMERA_DISTANCE * sinf(pitch_rad);
                let cam_z = CAMERA_DISTANCE * cosf(pitch_rad) * cosf(yaw_rad);

                // Set camera looking at origin
                camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
                camera_fov(60.0);

                // Draw environment
                env_gradient(
                    0,
                    0x1a1a2eFF,
                    0x2d2d4dFF,
                    0x2d2d4dFF,
                    0x0a0a14FF,
                    0.0,
                    0.0,
                );

                if SHOW_GRID {
                    env_lines(
                        1,
                        0,
                        2,
                        2,
                        1.0,
                        20.0,
                        0x404060FF,
                        0x606080FF,
                        5,
                        0,
                    );
                }

                draw_env();

                // Draw grid floor
                if SHOW_GRID {
                    push_identity();
                    push_translate(0.0, -2.0, 0.0);
                    set_color(0x30304080);
                    draw_mesh(GRID_MESH);
                }

                // Draw the current shape
                push_identity();
                set_color(0xFFFFFFFF);
                let mesh_handle = MESH_HANDLES[CURRENT_SHAPE as usize][CURRENT_SUBDIV as usize];
                draw_mesh(mesh_handle);

                // Draw UI
                draw_ui();
            }
        }

        // =============================================================================
        // UI Drawing
        // =============================================================================

        unsafe fn draw_ui() {
            // Semi-transparent background for UI panel
            draw_rect(5.0, 5.0, 260.0, 160.0, 0x00000088);

            // Title with mode name
            let title = concat!("Proc-Gen: ", $mode_name);
            let title_bytes = title.as_bytes();
            draw_text(title_bytes.as_ptr(), title_bytes.len() as u32, 15.0, 15.0, 20.0, 0xFFFFFFFF);

            // Current shape
            let shape_name = SHAPE_NAMES[CURRENT_SHAPE as usize];
            draw_label(15.0, 45.0, b"Shape:", shape_name);

            // Subdivision level
            let subdiv_names = ["Low", "Medium", "High"];
            let subdiv_name = subdiv_names[CURRENT_SUBDIV as usize];
            draw_label(15.0, 65.0, b"Detail:", subdiv_name);

            // Grid status
            let grid_status = if SHOW_GRID { "On" } else { "Off" };
            draw_label(15.0, 85.0, b"Grid:", grid_status);

            // Stats
            let (verts, tris) = MESH_STATS[CURRENT_SHAPE as usize][CURRENT_SUBDIV as usize];
            draw_stat_line(15.0, 110.0, b"Vertices:", verts);
            draw_stat_line(15.0, 130.0, b"Triangles:", tris);

            // Controls hint at bottom
            let controls = b"L1/R1:Shape  X/Y:Detail  A:Grid";
            draw_text(controls.as_ptr(), controls.len() as u32, 10.0, 520.0, 12.0, 0x888888FF);
        }

        unsafe fn draw_label(x: f32, y: f32, prefix: &[u8], value: &str) {
            draw_text(prefix.as_ptr(), prefix.len() as u32, x, y, 14.0, 0xAAAAAAFF);
            let value_bytes = value.as_bytes();
            draw_text(value_bytes.as_ptr(), value_bytes.len() as u32, x + 70.0, y, 14.0, 0xFFFFFFFF);
        }

        unsafe fn draw_stat_line(x: f32, y: f32, prefix: &[u8], value: u32) {
            draw_text(prefix.as_ptr(), prefix.len() as u32, x, y, 14.0, 0xAAAAAAFF);
            let mut buf = [0u8; 16];
            let len = format_u32(value, &mut buf);
            draw_text(buf.as_ptr(), len as u32, x + 80.0, y, 14.0, 0x88FF88FF);
        }

        fn format_u32(mut n: u32, buf: &mut [u8; 16]) -> usize {
            if n == 0 {
                buf[0] = b'0';
                return 1;
            }

            let mut i = 0;
            let mut temp = [0u8; 16];

            while n > 0 {
                temp[i] = b'0' + (n % 10) as u8;
                n /= 10;
                i += 1;
            }

            for j in 0..i {
                buf[j] = temp[i - 1 - j];
            }

            i
        }
    };
}

/// Generate an asset preview viewer that loads ROM meshes with proper material setup.
///
/// This macro creates mode-aware viewers that properly configure materials:
/// - Mode 2 (PBR): Uses metallic/roughness properties
/// - Mode 3 (Blinn-Phong): Uses specular/shininess properties
///
/// Usage:
/// ```ignore
/// #![no_std]
/// #![no_main]
/// proc_gen_common::asset_viewer!(2, "NEON DRIFT", [
///     ("speedster", "Speedster"),
///     ("muscle", "Muscle Car"),
/// ]);
/// ```
#[macro_export]
macro_rules! asset_viewer {
    ($render_mode:expr, $title:expr, [$(($id:expr, $name:expr)),* $(,)?]) => {
        use proc_gen_common::{sinf, cosf, PanicInfo};

        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            core::arch::wasm32::unreachable()
        }

        // =============================================================================
        // FFI Declarations
        // =============================================================================

        #[link(wasm_import_module = "env")]
        extern "C" {
            // Configuration
            fn set_clear_color(color: u32);
            fn render_mode(mode: u32);

            // Camera
            fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
            fn camera_fov(fov_degrees: f32);

            // Input
            fn button_pressed(player: u32, button: u32) -> u32;
            fn left_stick_x(player: u32) -> f32;
            fn left_stick_y(player: u32) -> f32;
            fn right_stick_y(player: u32) -> f32;

            // ROM asset loading
            fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
            fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;

            // Textures
            fn texture_bind(handle: u32);

            // Procedural mesh (for grid)
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

            // Materials - Mode 2 (PBR)
            fn material_metallic(value: f32);
            fn material_roughness(value: f32);
            fn material_emissive(value: f32);

            // Materials - Mode 3 (Blinn-Phong)
            fn material_shininess(value: f32);
            fn material_specular(color: u32);

            // Lighting
            fn light_set(index: u32, x: f32, y: f32, z: f32);
            fn light_color(index: u32, color: u32);
            fn light_intensity(index: u32, intensity: f32);
            fn light_enable(index: u32);

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
            fn env_lines(
                layer: u32,
                variant: u32,
                line_type: u32,
                thickness: u32,
                spacing: f32,
                fade_distance: f32,
                color_primary: u32,
                color_accent: u32,
                accent_every: u32,
                phase: u32,
            );
            fn draw_env();

            // 2D UI
            fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
            fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
        }

        // =============================================================================
        // Asset Data
        // =============================================================================

        const ASSET_IDS: &[&str] = &[$($id),*];
        const ASSET_NAMES: &[&str] = &[$($name),*];
        const NUM_ASSETS: u32 = { let mut n = 0u32; $(let _ = $id; n += 1;)* n };
        const RENDER_MODE: u32 = $render_mode;

        // Color palette for assets (distinct colors per asset)
        const ASSET_COLORS: [u32; 16] = [
            0xE57373FF, // Red
            0x64B5F6FF, // Blue
            0x81C784FF, // Green
            0xFFD54FFF, // Amber
            0xBA68C8FF, // Purple
            0x4DD0E1FF, // Cyan
            0xFF8A65FF, // Deep Orange
            0xA1887FFF, // Brown
            0x90A4AEFF, // Blue Grey
            0xF06292FF, // Pink
            0xAED581FF, // Light Green
            0xFFB74DFF, // Orange
            0x7986CBFF, // Indigo
            0x4FC3F7FF, // Light Blue
            0xDCE775FF, // Lime
            0x9575CDFF, // Deep Purple
        ];

        // =============================================================================
        // Constants
        // =============================================================================

        const BUTTON_A: u32 = 4;
        const BUTTON_B: u32 = 5;
        const BUTTON_L1: u32 = 8;
        const BUTTON_R1: u32 = 9;
        const BUTTON_START: u32 = 12;
        const MAX_ASSETS: usize = 32;

        // =============================================================================
        // State
        // =============================================================================

        static mut CAMERA_YAW: f32 = 0.0;
        static mut CAMERA_PITCH: f32 = 20.0;
        static mut CAMERA_DISTANCE: f32 = 5.0;
        static mut ROTATION: f32 = 0.0;
        static mut AUTO_ROTATE: bool = true;

        static mut CURRENT_ASSET: u32 = 0;
        static mut SHOW_GRID: bool = true;

        static mut MESH_HANDLES: [u32; MAX_ASSETS] = [0; MAX_ASSETS];
        static mut TEXTURE_HANDLES: [u32; MAX_ASSETS] = [0; MAX_ASSETS];
        static mut GRID_MESH: u32 = 0;

        // =============================================================================
        // Init
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn init() {
            unsafe {
                set_clear_color(0x1a1a2eFF);
                render_mode(RENDER_MODE);
                depth_test(1);

                // Load all assets from ROM
                let mut i = 0usize;
                $(
                    let id_bytes = $id.as_bytes();
                    MESH_HANDLES[i] = rom_mesh(id_bytes.as_ptr(), id_bytes.len() as u32);
                    // Try to load matching texture (may return 0 if not found)
                    TEXTURE_HANDLES[i] = rom_texture(id_bytes.as_ptr(), id_bytes.len() as u32);
                    i += 1;
                )*
                let _ = i; // Suppress unused warning

                // Grid for floor
                GRID_MESH = plane(10.0, 10.0, 20, 20);

                // Set up lighting - brighter for showcase
                light_set(0, 0.5, -0.7, 0.5);
                light_color(0, 0xFFF8F0FF);
                light_intensity(0, 2.0);
                light_enable(0);

                light_set(1, -0.5, -0.3, -0.5);
                light_color(1, 0xB0C0FFFF);
                light_intensity(1, 0.8);
                light_enable(1);

                // Third light for rim/backlight
                light_set(2, 0.0, 0.3, -0.8);
                light_color(2, 0xFFFFFFFF);
                light_intensity(2, 0.4);
                light_enable(2);
            }
        }

        // =============================================================================
        // Update
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn update() {
            unsafe {
                // Auto-rotation
                if AUTO_ROTATE {
                    ROTATION += 0.5;
                    if ROTATION >= 360.0 {
                        ROTATION -= 360.0;
                    }
                }

                // Camera orbit control
                let stick_x = left_stick_x(0);
                let stick_y = left_stick_y(0);

                if stick_x.abs() > 0.1 {
                    CAMERA_YAW += stick_x * 2.5;
                    AUTO_ROTATE = false;
                }
                if stick_y.abs() > 0.1 {
                    CAMERA_PITCH += stick_y * 2.0;
                    CAMERA_PITCH = CAMERA_PITCH.clamp(-89.0, 89.0);
                }

                // Camera distance
                let rstick_y = right_stick_y(0);
                if rstick_y.abs() > 0.1 {
                    CAMERA_DISTANCE += rstick_y * 0.1;
                    CAMERA_DISTANCE = CAMERA_DISTANCE.clamp(1.0, 20.0);
                }

                // A: Toggle grid
                if button_pressed(0, BUTTON_A) != 0 {
                    SHOW_GRID = !SHOW_GRID;
                }

                // L1/R1: Cycle assets
                if button_pressed(0, BUTTON_R1) != 0 {
                    CURRENT_ASSET = (CURRENT_ASSET + 1) % NUM_ASSETS;
                    CAMERA_DISTANCE = 5.0; // Reset distance for new asset
                }
                if button_pressed(0, BUTTON_L1) != 0 {
                    if CURRENT_ASSET == 0 {
                        CURRENT_ASSET = NUM_ASSETS - 1;
                    } else {
                        CURRENT_ASSET -= 1;
                    }
                    CAMERA_DISTANCE = 5.0;
                }

                // START: Reset camera + enable auto-rotate
                if button_pressed(0, BUTTON_START) != 0 {
                    CAMERA_YAW = 0.0;
                    CAMERA_PITCH = 20.0;
                    CAMERA_DISTANCE = 5.0;
                    AUTO_ROTATE = true;
                }
            }
        }

        // =============================================================================
        // Render
        // =============================================================================

        #[no_mangle]
        pub extern "C" fn render() {
            unsafe {
                let yaw_rad = CAMERA_YAW * 0.01745329;
                let pitch_rad = CAMERA_PITCH * 0.01745329;

                let cam_x = CAMERA_DISTANCE * cosf(pitch_rad) * sinf(yaw_rad);
                let cam_y = CAMERA_DISTANCE * sinf(pitch_rad);
                let cam_z = CAMERA_DISTANCE * cosf(pitch_rad) * cosf(yaw_rad);

                camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
                camera_fov(60.0);

                // Environment - mode-specific colors
                if RENDER_MODE == 2 {
                    // Cyberpunk/neon aesthetic for Mode 2
                    env_gradient(0, 0x0a0a1aFF, 0x1a1a3aFF, 0x1a1a3aFF, 0x050510FF, 0.0, 0.0);
                } else {
                    // Fantasy/warm aesthetic for Mode 3
                    env_gradient(0, 0x1a1a2eFF, 0x2d2d4dFF, 0x2d2d4dFF, 0x0a0a14FF, 0.0, 0.0);
                }

                if SHOW_GRID {
                    env_lines(1, 0, 2, 2, 1.0, 20.0, 0x404060FF, 0x606080FF, 5, 0);
                }

                draw_env();

                // Grid floor
                if SHOW_GRID {
                    push_identity();
                    push_translate(0.0, -1.5, 0.0);
                    set_color(0x30304080);
                    draw_mesh(GRID_MESH);
                }

                // Set up material based on render mode
                let asset_color = ASSET_COLORS[(CURRENT_ASSET as usize) % 16];

                if RENDER_MODE == 2 {
                    // Mode 2: PBR (Metallic-Roughness)
                    // Vary metallic/roughness based on asset index for variety
                    let metallic = 0.3 + (((CURRENT_ASSET * 17) % 7) as f32) * 0.1;
                    let roughness = 0.3 + (((CURRENT_ASSET * 13) % 5) as f32) * 0.1;
                    material_metallic(metallic);
                    material_roughness(roughness);
                    material_emissive(0.0);
                } else {
                    // Mode 3: Blinn-Phong (Specular-Shininess)
                    // Vary shininess based on asset index
                    let shininess = 0.3 + (((CURRENT_ASSET * 11) % 6) as f32) * 0.1;
                    material_shininess(shininess);
                    material_specular(0xFFFFFFFF); // White specular highlights
                }

                // Bind texture if available
                let tex_handle = TEXTURE_HANDLES[CURRENT_ASSET as usize];
                if tex_handle != 0 {
                    texture_bind(tex_handle);
                }

                // Current asset with rotation
                push_identity();
                push_rotate_y(ROTATION);

                // Use distinct color per asset
                set_color(asset_color);

                let handle = MESH_HANDLES[CURRENT_ASSET as usize];
                draw_mesh(handle);

                // UI
                draw_ui();
            }
        }

        // =============================================================================
        // UI
        // =============================================================================

        unsafe fn draw_ui() {
            draw_rect(5.0, 5.0, 320.0, 120.0, 0x00000088);

            // Title
            let title = $title;
            let title_bytes = title.as_bytes();
            draw_text(title_bytes.as_ptr(), title_bytes.len() as u32, 15.0, 15.0, 18.0, 0xFFFFFFFF);

            // Render mode indicator
            if RENDER_MODE == 2 {
                let mode_text = b"PBR (Metallic-Roughness)";
                draw_text(mode_text.as_ptr(), mode_text.len() as u32, 15.0, 40.0, 12.0, 0x88AAFFFF);
            } else {
                let mode_text = b"Blinn-Phong (Specular)";
                draw_text(mode_text.as_ptr(), mode_text.len() as u32, 15.0, 40.0, 12.0, 0x88AAFFFF);
            }

            // Current asset name
            let name = ASSET_NAMES[CURRENT_ASSET as usize];
            let name_bytes = name.as_bytes();
            draw_text(b"Asset:".as_ptr(), 6, 15.0, 65.0, 14.0, 0xAAAAAAFF);
            draw_text(name_bytes.as_ptr(), name_bytes.len() as u32, 80.0, 65.0, 14.0, 0x88FF88FF);

            // Asset counter
            let mut buf = [0u8; 16];
            let len = format_counter(CURRENT_ASSET + 1, NUM_ASSETS, &mut buf);
            draw_text(buf.as_ptr(), len as u32, 15.0, 90.0, 12.0, 0x888888FF);

            // Controls - more detailed
            let controls = b"L1/R1:Asset  A:Grid  START:Reset  L-Stick:Orbit  R-Stick:Zoom";
            draw_text(controls.as_ptr(), controls.len() as u32, 10.0, 520.0, 11.0, 0x888888FF);
        }

        fn format_counter(current: u32, total: u32, buf: &mut [u8; 16]) -> usize {
            let mut i = 0;
            i += format_u32_into(current, &mut buf[i..]);
            buf[i] = b'/';
            i += 1;
            i += format_u32_into(total, &mut buf[i..]);
            i
        }

        fn format_u32_into(mut n: u32, buf: &mut [u8]) -> usize {
            if n == 0 {
                buf[0] = b'0';
                return 1;
            }
            let mut temp = [0u8; 16];
            let mut len = 0;
            while n > 0 {
                temp[len] = b'0' + (n % 10) as u8;
                n /= 10;
                len += 1;
            }
            for j in 0..len {
                buf[j] = temp[len - 1 - j];
            }
            len
        }
    };
}
