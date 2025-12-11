//! Level Loader Demo
//!
//! Demonstrates loading custom binary level data from a ROM data pack
//! using rom_data_len() and rom_data() FFI functions.
//!
//! Level format (simple tilemap):
//! - Bytes 0-3: Magic "ELVL"
//! - Byte 4: Version (u8)
//! - Byte 5: Level number (u8)
//! - Bytes 6-7: Width (u16 little-endian)
//! - Bytes 8-9: Height (u16 little-endian)
//! - Remaining: Tile indices (1 byte per tile)

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// FFI Declarations
// =============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // ROM Data Pack Loading
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_data_len(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_data(id_ptr: *const u8, id_len: u32, dst_ptr: *mut u8, max_len: u32) -> u32;

    // Configuration (init-only)
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn button_pressed(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Drawing
    fn texture_bind(handle: u32);
    fn draw_sprite_region(x: f32, y: f32, w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32, color: u32);
    fn draw_rect(x: f32, y: f32, width: f32, height: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, scale: f32, color: u32);

    // Render state
    fn set_color(color: u32);
}

// Button constants
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;

// =============================================================================
// Level Data Structure
// =============================================================================

const HEADER_SIZE: usize = 10; // Magic(4) + Version(1) + LevelNum(1) + Width(2) + Height(2)
const MAX_LEVEL_SIZE: usize = 4096; // Max level data including header
const TILE_SIZE: f32 = 32.0; // Larger tiles for better visibility

// Level data buffer (stored in WASM memory)
static mut LEVEL_DATA: [u8; MAX_LEVEL_SIZE] = [0u8; MAX_LEVEL_SIZE];
static mut LEVEL_WIDTH: u16 = 0;
static mut LEVEL_HEIGHT: u16 = 0;
static mut LEVEL_NUM: u8 = 0;

// Tileset texture
static mut TILESET: u32 = 0;

// Current level and camera
static mut CURRENT_LEVEL: u32 = 0;
static mut CAMERA_X: f32 = 0.0;
static mut CAMERA_Y: f32 = 0.0;

// Level IDs
const LEVEL_IDS: [(&[u8], usize); 3] = [
    (b"level1", 6),
    (b"level2", 6),
    (b"level3", 6),
];

// Tile colors for visualization
const TILE_COLORS: [u32; 4] = [
    0x2a2a4aFF, // Floor (dark blue)
    0x8888AAFF, // Wall (light gray-blue)
    0xFFAA44FF, // Decoration (orange)
    0x44FF44FF, // Other (green)
];

// =============================================================================
// Level Loading
// =============================================================================

/// Load level data from ROM data pack into WASM memory
unsafe fn load_level(level_index: u32) {
    if level_index as usize >= LEVEL_IDS.len() {
        return;
    }

    let (id, id_len) = LEVEL_IDS[level_index as usize];

    // First, get the size of the level data
    let data_size = rom_data_len(id.as_ptr(), id_len as u32);
    if data_size == 0 || data_size as usize > MAX_LEVEL_SIZE {
        return;
    }

    // Load the level data into our buffer
    let bytes_read = rom_data(
        id.as_ptr(),
        id_len as u32,
        LEVEL_DATA.as_mut_ptr(),
        data_size,
    );

    if bytes_read < HEADER_SIZE as u32 {
        return; // Need full header
    }

    // Verify magic bytes "ELVL"
    if LEVEL_DATA[0] != b'E' || LEVEL_DATA[1] != b'L'
       || LEVEL_DATA[2] != b'V' || LEVEL_DATA[3] != b'L' {
        return; // Invalid format
    }

    // Parse header
    // Byte 4: Version (ignored for now)
    LEVEL_NUM = LEVEL_DATA[5]; // Byte 5: Level number
    LEVEL_WIDTH = u16::from_le_bytes([LEVEL_DATA[6], LEVEL_DATA[7]]);
    LEVEL_HEIGHT = u16::from_le_bytes([LEVEL_DATA[8], LEVEL_DATA[9]]);

    // Reset camera to center of level
    CAMERA_X = (LEVEL_WIDTH as f32 * TILE_SIZE) / 2.0 - 200.0;
    CAMERA_Y = (LEVEL_HEIGHT as f32 * TILE_SIZE) / 2.0 - 150.0;

    CURRENT_LEVEL = level_index;
}

/// Get tile at position (tiles start after header)
unsafe fn get_tile(x: u32, y: u32) -> u8 {
    if x >= LEVEL_WIDTH as u32 || y >= LEVEL_HEIGHT as u32 {
        return 0;
    }
    let index = HEADER_SIZE + (y * LEVEL_WIDTH as u32 + x) as usize;
    if index < MAX_LEVEL_SIZE {
        LEVEL_DATA[index]
    } else {
        0
    }
}

/// Clamp a value between min and max
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

// =============================================================================
// Game Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a2a3aFF);

        // Set up 2D camera
        camera_set(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Load tileset texture from data pack
        TILESET = rom_texture(b"tileset".as_ptr(), 7);

        // Load first level
        load_level(0);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Switch levels with D-pad left/right
        if button_pressed(0, BUTTON_LEFT) != 0 && CURRENT_LEVEL > 0 {
            load_level(CURRENT_LEVEL - 1);
        }
        if button_pressed(0, BUTTON_RIGHT) != 0 && CURRENT_LEVEL < 2 {
            load_level(CURRENT_LEVEL + 1);
        }

        // Pan camera with analog stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        CAMERA_X += stick_x * 4.0;
        CAMERA_Y += stick_y * 4.0;

        // Clamp camera to level bounds
        let max_x = (LEVEL_WIDTH as f32 * TILE_SIZE) - 400.0;
        let max_y = (LEVEL_HEIGHT as f32 * TILE_SIZE) - 300.0;
        let max_x = if max_x < 0.0 { 0.0 } else { max_x };
        let max_y = if max_y < 0.0 { 0.0 } else { max_y };
        CAMERA_X = clamp(CAMERA_X, 0.0, max_x);
        CAMERA_Y = clamp(CAMERA_Y, 0.0, max_y);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // UI Header area (top 120 pixels)
        let map_offset_y = 130.0;

        // Render tilemap using colored rectangles
        // (In a real game, you'd use textured sprites from the tileset)

        // Calculate visible tile range (offset for header)
        let start_x = (CAMERA_X / TILE_SIZE) as i32;
        let start_y = (CAMERA_Y / TILE_SIZE) as i32;
        let end_x = start_x + 15; // Fewer tiles horizontally for larger size
        let end_y = start_y + 12; // Fewer tiles vertically

        for y in start_y.max(0)..end_y.min(LEVEL_HEIGHT as i32) {
            for x in start_x.max(0)..end_x.min(LEVEL_WIDTH as i32) {
                let tile = get_tile(x as u32, y as u32);

                // Calculate screen position (offset below header)
                let screen_x = (x as f32 * TILE_SIZE) - CAMERA_X;
                let screen_y = (y as f32 * TILE_SIZE) - CAMERA_Y + map_offset_y;

                // Get color for this tile type
                let color = TILE_COLORS[(tile as usize) % TILE_COLORS.len()];

                // Draw tile as colored rectangle
                draw_rect(screen_x, screen_y, TILE_SIZE - 2.0, TILE_SIZE - 2.0, color);
            }
        }

        // UI overlay (proper text sizes for 540p)
        let title = b"Level Loader Demo";
        draw_text(title.as_ptr(), title.len() as u32, 20.0, 20.0, 28.0, 0xFFFFFFFF);

        // Level indicator with visual arrows
        let left_arrow = if CURRENT_LEVEL > 0 { b"<< " } else { b"   " };
        let right_arrow = if CURRENT_LEVEL < 2 { b" >>" } else { b"   " };

        draw_text(left_arrow.as_ptr(), 3, 20.0, 60.0, 24.0, 0x88FF88FF);

        let level_text = match CURRENT_LEVEL {
            0 => b"Level 1 of 3" as &[u8],
            1 => b"Level 2 of 3" as &[u8],
            _ => b"Level 3 of 3" as &[u8],
        };
        draw_text(level_text.as_ptr(), level_text.len() as u32, 80.0, 60.0, 24.0, 0xFFFF00FF);
        draw_text(right_arrow.as_ptr(), 3, 300.0, 60.0, 24.0, 0x88FF88FF);

        // Controls hint
        let hint = b"[D-Pad] Switch  [Stick] Pan";
        draw_text(hint.as_ptr(), hint.len() as u32, 20.0, 100.0, 18.0, 0xAAAAAAFF);

        // Tile legend on the right side
        let legend_x = 700.0;
        let legend = b"Legend:";
        draw_text(legend.as_ptr(), legend.len() as u32, legend_x, 150.0, 20.0, 0xFFFFFFFF);

        // Draw color swatches with labels (larger)
        draw_rect(legend_x, 185.0, 24.0, 24.0, TILE_COLORS[0]);
        let floor = b"Floor";
        draw_text(floor.as_ptr(), floor.len() as u32, legend_x + 35.0, 185.0, 18.0, 0xCCCCCCFF);

        draw_rect(legend_x, 220.0, 24.0, 24.0, TILE_COLORS[1]);
        let wall = b"Wall";
        draw_text(wall.as_ptr(), wall.len() as u32, legend_x + 35.0, 220.0, 18.0, 0xCCCCCCFF);

        draw_rect(legend_x, 255.0, 24.0, 24.0, TILE_COLORS[2]);
        let deco = b"Decor";
        draw_text(deco.as_ptr(), deco.len() as u32, legend_x + 35.0, 255.0, 18.0, 0xCCCCCCFF);

        draw_rect(legend_x, 290.0, 24.0, 24.0, TILE_COLORS[3]);
        let other = b"Other";
        draw_text(other.as_ptr(), other.len() as u32, legend_x + 35.0, 290.0, 18.0, 0xCCCCCCFF);

        // Info at bottom
        let info = b"Data loaded via rom_data()";
        draw_text(info.as_ptr(), info.len() as u32, 20.0, 500.0, 16.0, 0x888888FF);
    }
}
