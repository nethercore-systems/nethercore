//! XM Tracker Demo
//!
//! Demonstrates XM tracker music playback with Nethercore ZX:
//! - Procedurally generated drum and synth sounds
//! - 4-channel XM beat pattern
//! - Interactive playback controls
//! - Visual beat indicator
//!
//! Controls:
//! - A button: Pause/Resume playback
//! - B button: Restart from beginning
//! - Up/Down: Adjust tempo (+/- 10 BPM)
//! - Left/Right: Adjust volume

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// === FFI Imports ===

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);

    // Input
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // 2D Drawing
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // ROM Assets
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_tracker(id_ptr: *const u8, id_len: u32) -> u32;

    // Tracker Audio
    fn tracker_play(handle: u32, volume: f32, looping: u32);
    fn tracker_stop();
    fn tracker_pause(paused: u32);
    fn tracker_set_volume(volume: f32);
    fn tracker_is_playing() -> u32;
    fn tracker_jump(order: u32, row: u32);
    fn tracker_position() -> u32;
    fn tracker_set_tempo(bpm: u32);
}

// === Constants ===

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;

// Colors
const COLOR_BG: u32 = 0x1a1a2eFF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x888888FF;
const COLOR_DARK_GRAY: u32 = 0x444444FF;
const COLOR_ACCENT: u32 = 0xFF6B6BFF;
const COLOR_ACCENT2: u32 = 0x4ECDC4FF;
const COLOR_PLAYING: u32 = 0x00FF00FF;
const COLOR_PAUSED: u32 = 0xFFAA00FF;

// === Global State ===

static mut TRACKER_HANDLE: u32 = 0;
static mut CURRENT_TEMPO: u32 = 125;
static mut VOLUME: f32 = 0.8;
static mut IS_PAUSED: bool = false;

// === Helper Functions ===

fn draw_text_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size, color);
    }
}

fn load_rom_sound(id: &[u8]) -> u32 {
    unsafe { rom_sound(id.as_ptr(), id.len() as u32) }
}

fn load_rom_tracker(id: &[u8]) -> u32 {
    unsafe { rom_tracker(id.as_ptr(), id.len() as u32) }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_BG);

        // Load sound samples first (required for tracker instrument mapping)
        // The tracker's instruments reference these by name
        load_rom_sound(b"kick");
        load_rom_sound(b"snare");
        load_rom_sound(b"hihat");
        load_rom_sound(b"bass");

        // Load and start tracker
        TRACKER_HANDLE = load_rom_tracker(b"demo");
        tracker_play(TRACKER_HANDLE, VOLUME, 1); // Looping
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // A button: Toggle pause/resume
        if button_pressed(0, BUTTON_A) != 0 {
            IS_PAUSED = !IS_PAUSED;
            tracker_pause(if IS_PAUSED { 1 } else { 0 });
        }

        // B button: Restart from beginning
        if button_pressed(0, BUTTON_B) != 0 {
            tracker_jump(0, 0);
            if IS_PAUSED {
                IS_PAUSED = false;
                tracker_pause(0);
            }
        }

        // Up/Down: Adjust tempo
        if button_pressed(0, BUTTON_UP) != 0 {
            CURRENT_TEMPO = if CURRENT_TEMPO < 250 { CURRENT_TEMPO + 10 } else { 250 };
            tracker_set_tempo(CURRENT_TEMPO);
        }
        if button_pressed(0, BUTTON_DOWN) != 0 {
            CURRENT_TEMPO = if CURRENT_TEMPO > 60 { CURRENT_TEMPO - 10 } else { 60 };
            tracker_set_tempo(CURRENT_TEMPO);
        }

        // Left/Right: Adjust volume
        if button_held(0, BUTTON_RIGHT) != 0 {
            VOLUME += 0.02;
            if VOLUME > 1.0 {
                VOLUME = 1.0;
            }
            tracker_set_volume(VOLUME);
        }
        if button_held(0, BUTTON_LEFT) != 0 {
            VOLUME -= 0.02;
            if VOLUME < 0.0 {
                VOLUME = 0.0;
            }
            tracker_set_volume(VOLUME);
        }
    }
}

// === Render ===

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let pos = tracker_position();
        let order = (pos >> 16) as u32;
        let row = (pos & 0xFFFF) as u32;

        // Title
        draw_text_str(b"XM Tracker Demo", 340.0, 40.0, 36.0, COLOR_WHITE);

        // Playback status
        let is_playing = tracker_is_playing() != 0 && !IS_PAUSED;
        let status_color = if is_playing { COLOR_PLAYING } else { COLOR_PAUSED };
        let status_text: &[u8] = if IS_PAUSED {
            b"PAUSED"
        } else if is_playing {
            b"PLAYING"
        } else {
            b"STOPPED"
        };
        draw_text_str(status_text, 430.0, 90.0, 24.0, status_color);

        // Position info
        draw_text_str(b"Position:", 100.0, 150.0, 20.0, COLOR_GRAY);

        // Order display
        draw_text_str(b"Order:", 100.0, 180.0, 18.0, COLOR_DARK_GRAY);
        let order_digit = b'0' + (order % 10) as u8;
        let order_text = [order_digit];
        draw_text(order_text.as_ptr(), 1, 180.0, 180.0, 18.0, COLOR_WHITE);

        // Row display
        draw_text_str(b"Row:", 100.0, 210.0, 18.0, COLOR_DARK_GRAY);
        let row_tens = b'0' + ((row / 10) % 10) as u8;
        let row_ones = b'0' + (row % 10) as u8;
        let row_text = [row_tens, row_ones];
        draw_text(row_text.as_ptr(), 2, 160.0, 210.0, 18.0, COLOR_WHITE);

        // Tempo display
        draw_text_str(b"Tempo:", 100.0, 260.0, 18.0, COLOR_DARK_GRAY);
        let tempo_hundreds = b'0' + ((CURRENT_TEMPO / 100) % 10) as u8;
        let tempo_tens = b'0' + ((CURRENT_TEMPO / 10) % 10) as u8;
        let tempo_ones = b'0' + (CURRENT_TEMPO % 10) as u8;
        let tempo_text = [tempo_hundreds, tempo_tens, tempo_ones];
        draw_text(tempo_text.as_ptr(), 3, 180.0, 260.0, 18.0, COLOR_WHITE);
        draw_text_str(b"BPM", 220.0, 260.0, 18.0, COLOR_DARK_GRAY);

        // Volume bar
        draw_text_str(b"Volume:", 100.0, 300.0, 18.0, COLOR_DARK_GRAY);
        draw_rect(180.0, 300.0, 200.0, 20.0, COLOR_DARK_GRAY);
        draw_rect(180.0, 300.0, 200.0 * VOLUME, 20.0, COLOR_ACCENT2);

        // Beat visualizer - large pulsing rectangle
        let center_x = SCREEN_WIDTH / 2.0;
        let center_y = SCREEN_HEIGHT / 2.0 + 50.0;

        // Different colors for different beats
        let beat_color = match row % 4 {
            0 => COLOR_ACCENT,   // Kick beat (red)
            2 => COLOR_ACCENT2,  // Snare beat (teal)
            _ => COLOR_GRAY,     // Hi-hat beats
        };

        // Pulse effect: larger on beat, smaller between
        let row_frac = (row % 4) as f32 / 4.0;
        let pulse = 1.0 - row_frac * 0.3;
        let base_size = 150.0;
        let size = base_size * pulse;

        let rect_x = center_x - size / 2.0;
        let rect_y = center_y - size / 2.0;
        draw_rect(rect_x, rect_y, size, size, beat_color);

        // Row indicator dots (16 rows in pattern)
        let dot_start_x = center_x - 120.0;
        let dot_y = center_y + 130.0;
        for i in 0..16u32 {
            let dot_x = dot_start_x + (i as f32) * 16.0;
            let dot_color = if i == row {
                COLOR_WHITE
            } else if i % 4 == 0 {
                COLOR_ACCENT
            } else {
                COLOR_DARK_GRAY
            };
            draw_rect(dot_x, dot_y, 10.0, 10.0, dot_color);
        }

        // Controls help
        draw_text_str(b"Controls:", 100.0, 450.0, 18.0, COLOR_GRAY);
        draw_text_str(b"[A] Pause/Resume   [B] Restart", 100.0, 475.0, 16.0, COLOR_DARK_GRAY);
        draw_text_str(b"[Up/Down] Tempo   [Left/Right] Volume", 100.0, 495.0, 16.0, COLOR_DARK_GRAY);

        // Channel activity (simple visualization)
        draw_text_str(b"Channels:", 600.0, 150.0, 18.0, COLOR_GRAY);

        // Show which instruments are active based on row
        let kick_active = row == 0;
        let snare_active = row == 8;
        let hihat_active = row % 4 == 0;
        let bass_active = row == 0;

        let active_color = COLOR_PLAYING;
        let inactive_color = COLOR_DARK_GRAY;

        draw_text_str(b"Kick", 600.0, 180.0, 16.0, if kick_active { active_color } else { inactive_color });
        draw_text_str(b"Snare", 600.0, 205.0, 16.0, if snare_active { active_color } else { inactive_color });
        draw_text_str(b"HiHat", 600.0, 230.0, 16.0, if hihat_active { active_color } else { inactive_color });
        draw_text_str(b"Bass", 600.0, 255.0, 16.0, if bass_active { active_color } else { inactive_color });
    }
}
