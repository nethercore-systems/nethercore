#![no_std]
#![no_main]

// Import showcase definitions (single source of truth!)
use proc_gen_showcase_defs::SHOWCASE_SOUNDS;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Init config
    fn set_clear_color(color: u32);

    // Camera (required even for 2D)
    fn camera_set(eye_x: f32, eye_y: f32, eye_z: f32, center_x: f32, center_y: f32, center_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn button_pressed(player: u32, button: u32) -> u32;

    // Drawing
    fn draw_text(text_ptr: *const u8, text_len: u32, x: f32, y: f32, scale: f32, color: u32);
    fn draw_rect(x: f32, y: f32, width: f32, height: f32, color: u32);

    // Audio
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;
    fn play_sound(sound: u32, volume: f32, pan: f32);
}

// =============================================================================
// Button Constants
// =============================================================================

pub mod button {
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const A: u32 = 4;
    pub const START: u32 = 12;
}

// =============================================================================
// Constants
// =============================================================================

const NUM_SOUNDS: usize = SHOWCASE_SOUNDS.len();
const MAX_SOUNDS: usize = 32; // Maximum capacity for future expansion

// =============================================================================
// Global State
// =============================================================================

static mut SOUND_HANDLES: [u32; MAX_SOUNDS] = [0; MAX_SOUNDS];
static mut SELECTED_INDEX: usize = 0;
static mut AUTO_DEMO: bool = false;
static mut AUTO_TIMER: f32 = 0.0;
static mut LAST_PLAYED: usize = MAX_SOUNDS; // MAX_SOUNDS = none
static mut LAST_PLAYED_TIMER: f32 = 0.0; // Visual feedback duration

// =============================================================================
// Initialization
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Set clear color (dark blue)
        set_clear_color(0x1a1a2eFF);

        // Load all sounds from ROM automatically
        for (i, sound) in SHOWCASE_SOUNDS.iter().enumerate() {
            let id_bytes = sound.id.as_bytes();
            SOUND_HANDLES[i] = rom_sound(id_bytes.as_ptr(), id_bytes.len() as u32);
        }
    }
}

// =============================================================================
// Update
// =============================================================================

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Navigation: UP/DOWN
        if button_pressed(0, button::UP) != 0 {
            if SELECTED_INDEX == 0 {
                SELECTED_INDEX = NUM_SOUNDS - 1;
            } else {
                SELECTED_INDEX -= 1;
            }
        }
        if button_pressed(0, button::DOWN) != 0 {
            SELECTED_INDEX = (SELECTED_INDEX + 1) % NUM_SOUNDS;
        }

        // Play selected sound: A button
        if button_pressed(0, button::A) != 0 {
            play_sound(SOUND_HANDLES[SELECTED_INDEX], 0.7, 0.0);
            LAST_PLAYED = SELECTED_INDEX;
            LAST_PLAYED_TIMER = 0.5; // Show "Playing" for 0.5 seconds
        }

        // Toggle auto-demo mode: START button
        if button_pressed(0, button::START) != 0 {
            AUTO_DEMO = !AUTO_DEMO;
            if AUTO_DEMO {
                AUTO_TIMER = 0.0;
                LAST_PLAYED = MAX_SOUNDS; // Reset
            }
        }

        // Auto-demo mode: cycle through sounds every 2 seconds
        if AUTO_DEMO {
            AUTO_TIMER += 0.016; // Assuming ~60 FPS
            if AUTO_TIMER >= 2.0 {
                AUTO_TIMER = 0.0;
                play_sound(SOUND_HANDLES[SELECTED_INDEX], 0.7, 0.0);
                LAST_PLAYED = SELECTED_INDEX;
                LAST_PLAYED_TIMER = 1.8; // Show longer in auto mode
                SELECTED_INDEX = (SELECTED_INDEX + 1) % NUM_SOUNDS;
            }
        }

        // Decay "playing" visual feedback
        if LAST_PLAYED_TIMER > 0.0 {
            LAST_PLAYED_TIMER -= 0.016;
            if LAST_PLAYED_TIMER <= 0.0 {
                LAST_PLAYED = MAX_SOUNDS;
            }
        }
    }
}

// =============================================================================
// Render
// =============================================================================

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (required even for 2D)
        camera_set(0.0, 0.0, 3.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Semi-transparent background panel
        draw_rect(20.0, 20.0, 600.0, 500.0, 0x00000088);

        // Title
        let title = b"Procedural Sound Effects Viewer";
        draw_text(
            title.as_ptr(),
            title.len() as u32,
            35.0,
            35.0,
            24.0,
            0xFFFFFFFF,
        );

        // Instruction
        let instruction = b"Select a sound to play:";
        draw_text(
            instruction.as_ptr(),
            instruction.len() as u32,
            35.0,
            75.0,
            16.0,
            0xAAAAAAFF,
        );

        // Draw sound list (automatically includes all showcase sounds!)
        let mut y = 120.0;
        for (i, sound) in SHOWCASE_SOUNDS.iter().enumerate() {
            let is_selected = i == SELECTED_INDEX;
            let is_playing = i == LAST_PLAYED;

            // Selection indicator
            if is_selected {
                let arrow = b"> ";
                draw_text(arrow.as_ptr(), arrow.len() as u32, 35.0, y, 18.0, 0x88FF88FF);
            }

            // Sound name
            let name_color = if is_selected { 0x88FF88FF } else { 0xFFFFFFFF };
            let name_bytes = sound.name.as_bytes();
            draw_text(
                name_bytes.as_ptr(),
                name_bytes.len() as u32,
                60.0,
                y,
                18.0,
                name_color,
            );

            // Description
            let desc_bytes = sound.description.as_bytes();
            draw_text(
                desc_bytes.as_ptr(),
                desc_bytes.len() as u32,
                200.0,
                y,
                14.0,
                0x888888FF,
            );

            // "Playing" indicator
            if is_playing {
                let playing = b"[Playing]";
                draw_text(
                    playing.as_ptr(),
                    playing.len() as u32,
                    480.0,
                    y,
                    14.0,
                    0x00FF00FF,
                );
            }

            y += 35.0;
        }

        // Auto-demo status
        if AUTO_DEMO {
            let auto_text = b"[AUTO-DEMO MODE]";
            draw_text(
                auto_text.as_ptr(),
                auto_text.len() as u32,
                35.0,
                y + 20.0,
                16.0,
                0xFFAA00FF,
            );
        }

        // Controls hint
        let controls = b"Controls: UP/DOWN - Select  |  A - Play  |  START - Auto-Demo";
        draw_text(
            controls.as_ptr(),
            controls.len() as u32,
            35.0,
            490.0,
            12.0,
            0x666666FF,
        );
    }
}

// =============================================================================
// Panic Handler
// =============================================================================

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
