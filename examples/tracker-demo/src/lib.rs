//! XM Tracker Demo
//!
//! Demonstrates XM tracker music playback with Nethercore ZX:
//! - Procedurally generated drum and synth sounds
//! - 4-channel XM beat pattern
//! - Interactive playback controls
//! - Visual beat indicator
//! - Real-time pattern/position display
//!
//! Controls:
//! - A button: Pause/Resume playback
//! - B button: Restart from beginning
//! - Up/Down: Adjust tempo (+/- 10 BPM)
//! - Left/Right: Adjust volume
//! - L/R Shoulder: Adjust speed (+/- 1 tick)

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

    // Unified Music API (works with both PCM and tracker handles)
    fn music_play(handle: u32, volume: f32, looping: u32);
    fn music_stop();
    fn music_pause(paused: u32);
    fn music_set_volume(volume: f32);
    fn music_is_playing() -> u32;
    fn music_jump(order: u32, row: u32);
    fn music_position() -> u32;
    fn music_length(handle: u32) -> u32;
    fn music_set_speed(speed: u32);
    fn music_set_tempo(bpm: u32);
    fn music_info(handle: u32) -> u32;
    fn music_name(handle: u32, out_ptr: *mut u8, max_len: u32) -> u32;
}

// === Constants ===

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Button constants
pub mod button {
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const LEFT: u32 = 2;
    pub const RIGHT: u32 = 3;
    pub const A: u32 = 4;
    pub const B: u32 = 5;
    pub const X: u32 = 6;
    pub const Y: u32 = 7;
    pub const L1: u32 = 8;
    pub const R1: u32 = 9;
    pub const L3: u32 = 10;
    pub const R3: u32 = 11;
    pub const START: u32 = 12;
    pub const SELECT: u32 = 13;
}

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
static mut CURRENT_SPEED: u32 = 6;
static mut VOLUME: f32 = 0.8;
static mut IS_PAUSED: bool = false;

// Tracker info (cached from init)
static mut SONG_LENGTH: u32 = 0;
static mut NUM_CHANNELS: u32 = 0;
static mut NUM_PATTERNS: u32 = 0;
static mut NUM_INSTRUMENTS: u32 = 0;
static mut SONG_NAME: [u8; 32] = [0u8; 32];
static mut SONG_NAME_LEN: u32 = 0;

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
        load_rom_sound(b"lead");

        // Load and start tracker
        TRACKER_HANDLE = load_rom_tracker(b"demo");

        // Cache tracker info
        let info = music_info(TRACKER_HANDLE);
        NUM_CHANNELS = (info >> 24) & 0xFF;
        NUM_PATTERNS = (info >> 16) & 0xFF;
        NUM_INSTRUMENTS = (info >> 8) & 0xFF;
        SONG_LENGTH = info & 0xFF;

        // Get song name
        SONG_NAME_LEN = music_name(TRACKER_HANDLE, SONG_NAME.as_mut_ptr(), 32);

        // Start playback
        music_play(TRACKER_HANDLE, VOLUME, 1); // Looping
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // A button: Toggle pause/resume
        if button_pressed(0, button::A) != 0 {
            IS_PAUSED = !IS_PAUSED;
            music_pause(if IS_PAUSED { 1 } else { 0 });
        }

        // B button: Restart from beginning
        if button_pressed(0, button::B) != 0 {
            music_jump(0, 0);
            if IS_PAUSED {
                IS_PAUSED = false;
                music_pause(0);
            }
        }

        // Up/Down: Adjust tempo
        if button_pressed(0, button::UP) != 0 {
            CURRENT_TEMPO = if CURRENT_TEMPO < 250 { CURRENT_TEMPO + 10 } else { 250 };
            music_set_tempo(CURRENT_TEMPO);
        }
        if button_pressed(0, button::DOWN) != 0 {
            CURRENT_TEMPO = if CURRENT_TEMPO > 60 { CURRENT_TEMPO - 10 } else { 60 };
            music_set_tempo(CURRENT_TEMPO);
        }

        // Left/Right: Adjust volume
        if button_held(0, button::RIGHT) != 0 {
            VOLUME += 0.02;
            if VOLUME > 1.0 {
                VOLUME = 1.0;
            }
            music_set_volume(VOLUME);
        }
        if button_held(0, button::LEFT) != 0 {
            VOLUME -= 0.02;
            if VOLUME < 0.0 {
                VOLUME = 0.0;
            }
            music_set_volume(VOLUME);
        }

        // L/R: Adjust speed (ticks per row)
        if button_pressed(0, button::R1) != 0 {
            CURRENT_SPEED = if CURRENT_SPEED < 31 { CURRENT_SPEED + 1 } else { 31 };
            music_set_speed(CURRENT_SPEED);
        }
        if button_pressed(0, button::L1) != 0 {
            CURRENT_SPEED = if CURRENT_SPEED > 1 { CURRENT_SPEED - 1 } else { 1 };
            music_set_speed(CURRENT_SPEED);
        }
    }
}

// === Render ===

/// Helper to format a number as 2 digits
fn format_2digit(n: u32) -> [u8; 2] {
    [b'0' + ((n / 10) % 10) as u8, b'0' + (n % 10) as u8]
}

/// Helper to format a number as 3 digits
fn format_3digit(n: u32) -> [u8; 3] {
    [
        b'0' + ((n / 100) % 10) as u8,
        b'0' + ((n / 10) % 10) as u8,
        b'0' + (n % 10) as u8,
    ]
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let pos = music_position();
        let order = (pos >> 16) as u32;
        let row = (pos & 0xFFFF) as u32;

        // === Header Section ===

        // Title (song name if available, otherwise default)
        if SONG_NAME_LEN > 0 {
            draw_text(SONG_NAME.as_ptr(), SONG_NAME_LEN, 340.0, 30.0, 32.0, COLOR_WHITE);
        } else {
            draw_text_str(b"XM Tracker Demo", 340.0, 30.0, 32.0, COLOR_WHITE);
        }

        // Playback status indicator
        let is_playing = music_is_playing() != 0 && !IS_PAUSED;
        let status_color = if is_playing { COLOR_PLAYING } else { COLOR_PAUSED };
        let status_text: &[u8] = if IS_PAUSED {
            b"PAUSED"
        } else if is_playing {
            b"PLAYING"
        } else {
            b"STOPPED"
        };
        draw_text_str(status_text, 440.0, 70.0, 20.0, status_color);

        // === Left Panel: Position & Timing ===

        let left_x = 60.0;
        let mut y = 120.0;

        draw_text_str(b"POSITION", left_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Order / Song Length progress bar
        draw_text_str(b"Order:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let order_text = format_2digit(order);
        draw_text(order_text.as_ptr(), 2, left_x + 70.0, y, 16.0, COLOR_WHITE);
        draw_text_str(b"/", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);
        let len_text = format_2digit(SONG_LENGTH);
        draw_text(len_text.as_ptr(), 2, left_x + 105.0, y, 16.0, COLOR_WHITE);

        // Order progress bar
        let bar_x = left_x + 140.0;
        draw_rect(bar_x, y, 150.0, 14.0, COLOR_DARK_GRAY);
        if SONG_LENGTH > 0 {
            let progress = (order as f32 / SONG_LENGTH as f32) * 150.0;
            draw_rect(bar_x, y, progress, 14.0, COLOR_ACCENT2);
        }
        y += 25.0;

        // Row display with progress
        draw_text_str(b"Row:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let row_text = format_2digit(row);
        draw_text(row_text.as_ptr(), 2, left_x + 70.0, y, 16.0, COLOR_WHITE);
        draw_text_str(b"/32", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);

        // Row progress bar
        draw_rect(bar_x, y, 150.0, 14.0, COLOR_DARK_GRAY);
        let row_progress = (row as f32 / 32.0) * 150.0;
        draw_rect(bar_x, y, row_progress, 14.0, COLOR_ACCENT);
        y += 35.0;

        // === Timing Section ===
        draw_text_str(b"TIMING", left_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Tempo
        draw_text_str(b"Tempo:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let tempo_text = format_3digit(CURRENT_TEMPO);
        draw_text(tempo_text.as_ptr(), 3, left_x + 70.0, y, 16.0, COLOR_WHITE);
        draw_text_str(b"BPM", left_x + 110.0, y, 14.0, COLOR_DARK_GRAY);
        y += 22.0;

        // Speed
        draw_text_str(b"Speed:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let speed_text = format_2digit(CURRENT_SPEED);
        draw_text(speed_text.as_ptr(), 2, left_x + 70.0, y, 16.0, COLOR_WHITE);
        draw_text_str(b"ticks/row", left_x + 100.0, y, 14.0, COLOR_DARK_GRAY);
        y += 35.0;

        // Volume bar
        draw_text_str(b"Volume:", left_x, y, 16.0, COLOR_DARK_GRAY);
        draw_rect(left_x + 70.0, y, 180.0, 16.0, COLOR_DARK_GRAY);
        draw_rect(left_x + 70.0, y, 180.0 * VOLUME, 16.0, COLOR_ACCENT2);
        let vol_pct = (VOLUME * 100.0) as u32;
        let vol_text = format_3digit(vol_pct);
        draw_text(vol_text.as_ptr(), 3, left_x + 260.0, y, 14.0, COLOR_WHITE);
        draw_text_str(b"%", left_x + 295.0, y, 14.0, COLOR_DARK_GRAY);

        // === Center: Beat Visualizer ===

        let center_x = SCREEN_WIDTH / 2.0;
        let center_y = SCREEN_HEIGHT / 2.0 + 30.0;

        // Different colors for different beats
        let beat_color = match row % 4 {
            0 => COLOR_ACCENT,   // Kick beat (red)
            2 => COLOR_ACCENT2,  // Snare beat (teal)
            _ => COLOR_GRAY,     // Hi-hat beats
        };

        // Pulse effect: larger on beat, smaller between
        let row_frac = (row % 4) as f32 / 4.0;
        let pulse = 1.0 - row_frac * 0.3;
        let base_size = 120.0;
        let size = base_size * pulse;

        let rect_x = center_x - size / 2.0;
        let rect_y = center_y - size / 2.0;
        draw_rect(rect_x, rect_y, size, size, beat_color);

        // Row indicator dots (32 rows in pattern, shown in 2 rows of 16)
        let dot_start_x = center_x - 120.0;
        let dot_y = center_y + 90.0;

        // First row of dots (rows 0-15)
        for i in 0..16u32 {
            let dot_x = dot_start_x + (i as f32) * 16.0;
            let dot_color = if i == row && row < 16 {
                COLOR_WHITE
            } else if i % 4 == 0 {
                COLOR_ACCENT
            } else {
                COLOR_DARK_GRAY
            };
            draw_rect(dot_x, dot_y, 10.0, 10.0, dot_color);
        }

        // Second row of dots (rows 16-31)
        let dot_y2 = dot_y + 16.0;
        for i in 0..16u32 {
            let dot_x = dot_start_x + (i as f32) * 16.0;
            let actual_row = i + 16;
            let dot_color = if actual_row == row {
                COLOR_WHITE
            } else if i % 4 == 0 {
                COLOR_ACCENT
            } else {
                COLOR_DARK_GRAY
            };
            draw_rect(dot_x, dot_y2, 10.0, 10.0, dot_color);
        }

        // === Right Panel: Track Info ===

        let right_x = 700.0;
        let mut y = 120.0;

        draw_text_str(b"TRACK INFO", right_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Number of channels
        draw_text_str(b"Channels:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let ch_text = format_2digit(NUM_CHANNELS);
        draw_text(ch_text.as_ptr(), 2, right_x + 100.0, y, 16.0, COLOR_WHITE);
        y += 22.0;

        // Number of patterns
        draw_text_str(b"Patterns:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let pat_text = format_2digit(NUM_PATTERNS);
        draw_text(pat_text.as_ptr(), 2, right_x + 100.0, y, 16.0, COLOR_WHITE);
        y += 22.0;

        // Number of instruments
        draw_text_str(b"Instruments:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let inst_text = format_2digit(NUM_INSTRUMENTS);
        draw_text(inst_text.as_ptr(), 2, right_x + 120.0, y, 16.0, COLOR_WHITE);
        y += 35.0;

        // Channel activity
        draw_text_str(b"CHANNELS", right_x, y, 18.0, COLOR_GRAY);
        y += 25.0;

        // Pattern order: 0=Intro, 1=Main, 2=Melody, 3=Breakdown
        // Order table: [0, 1, 1, 2, 1, 2, 3, 1] - we can infer pattern from order
        let pattern = match order % 8 {
            0 => 0, // Intro
            1 | 2 | 4 | 7 => 1, // Main
            3 | 5 => 2, // Melody
            6 => 3, // Breakdown
            _ => 1,
        };

        // Activity based on actual pattern data
        let kick_active = match pattern {
            0 => if row < 16 { row == 0 } else { row % 8 == 0 || row % 8 == 4 },
            1 | 2 => row % 8 == 0 || row % 8 == 4,
            3 => if row < 16 { row % 8 == 0 } else if row < 24 { row % 4 == 0 } else { row % 2 == 0 },
            _ => false,
        };

        let snare_active = match pattern {
            0 => row >= 16 && row % 8 == 4,
            1 | 2 => row % 8 == 4,
            3 => row == 12 || row == 28 || row == 30,
            _ => false,
        };

        let hihat_active = match pattern {
            0 => if row < 8 { row % 8 == 0 } else if row < 16 { row % 4 == 0 } else { row % 2 == 0 },
            1 => row % 2 == 0,
            2 => row % 2 == 0,
            3 => if row < 24 { row % 8 == 0 } else { row % 2 == 0 },
            _ => false,
        };

        // Bass: has notes on specific rows in all patterns
        let bass_active = match pattern {
            0 => row >= 16 && (row == 16 || row == 17 || row == 20 || row == 24 || row == 25 || row == 28 || row == 30),
            1 | 2 => row % 8 < 2 || row % 8 == 4, // First 2 rows + beat 2 of each 8-row section
            3 => row % 4 == 0 || row >= 28, // Quarter notes + rapid at end
            _ => false,
        };

        // Lead: only in melody and breakdown patterns
        let lead_active = match pattern {
            2 => row % 2 == 0, // Melody has notes on even rows
            3 => row >= 24 || row == 7 || row == 15 || row == 22, // Sparse + build at end
            _ => false,
        };

        // Lead harmony: only in melody pattern
        let lead2_active = pattern == 2 && row % 2 == 0;

        let active_color = COLOR_PLAYING;
        let inactive_color = COLOR_DARK_GRAY;

        // Channel indicators with activity bars (6 channels)
        let ch_names: [&[u8]; 6] = [b"CH1 Kick", b"CH2 Snare", b"CH3 HiHat", b"CH4 Bass", b"CH5 Lead", b"CH6 Harm"];
        let ch_active = [kick_active, snare_active, hihat_active, bass_active, lead_active, lead2_active];

        for i in 0..6 {
            let color = if ch_active[i] { active_color } else { inactive_color };
            draw_text(ch_names[i].as_ptr(), ch_names[i].len() as u32, right_x, y, 14.0, color);
            // Activity indicator
            if ch_active[i] {
                draw_rect(right_x + 100.0, y + 2.0, 40.0, 10.0, active_color);
            }
            y += 18.0;
        }

        // === Bottom: Controls Help ===

        let help_y = SCREEN_HEIGHT - 60.0;
        draw_text_str(b"Controls:", 60.0, help_y, 16.0, COLOR_GRAY);
        draw_text_str(b"[A] Pause   [B] Restart   [Up/Down] Tempo   [Left/Right] Volume   [L/R] Speed",
            60.0, help_y + 22.0, 14.0, COLOR_DARK_GRAY);
    }
}
