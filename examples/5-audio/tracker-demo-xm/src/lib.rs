//! XM Tracker Demo
//!
//! Demonstrates XM tracker music playback with Nethercore ZX:
//! - Multiple songs with different genres
//! - 8-channel XM playback with genre-specific instruments
//! - Interactive playback controls
//! - Visual beat indicator
//! - Real-time pattern/position display
//!
//! Controls:
//! - A button: Next song
//! - B button: Previous song
//! - X button: Pause/Resume playback
//! - Y button: Restart from beginning
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

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

// === Constants ===

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Note: Button constants available from ffi::button::*

// Colors
const COLOR_BG: u32 = 0x1a1a2eFF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x888888FF;
const COLOR_DARK_GRAY: u32 = 0x444444FF;
const COLOR_ACCENT: u32 = 0xFF6B6BFF;
const COLOR_ACCENT2: u32 = 0x4ECDC4FF;
const COLOR_PLAYING: u32 = 0x00FF00FF;
const COLOR_PAUSED: u32 = 0xFFAA00FF;
const COLOR_FUNK: u32 = 0xE040FBFF;    // Purple for funk
const COLOR_EURO: u32 = 0xFF5722FF;    // Orange for eurobeat
const COLOR_SYNTH: u32 = 0x4CAF50FF;   // Green for synthwave

// Song indices and count
const SONG_FUNK: u32 = 0;
const SONG_EURO: u32 = 1;
const SONG_SYNTH: u32 = 2;
const NUM_SONGS: u32 = 3;

// Default tempos per song
const TEMPOS: [u32; 3] = [110, 155, 105]; // Funk, Euro, Synth

// === Global State ===

static mut TRACKER_HANDLES: [u32; 3] = [0; 3];
static mut CURRENT_SONG: u32 = SONG_FUNK;
static mut CURRENT_TEMPO: u32 = 110; // Start with funk tempo
static mut CURRENT_SPEED: u32 = 6;
static mut VOLUME: f32 = 0.8;
static mut IS_PAUSED: bool = false;

// Tracker info (cached from current song)
static mut SONG_LENGTH: u32 = 0;
static mut NUM_CHANNELS: u32 = 0;
static mut NUM_PATTERNS: u32 = 0;
static mut NUM_INSTRUMENTS: u32 = 0;
static mut SONG_NAME: [u8; 32] = [0u8; 32];
static mut SONG_NAME_LEN: u32 = 0;

// === Helper Functions ===

fn draw_text_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        set_color(color);
        draw_text(s.as_ptr(), s.len() as u32, x, y, size);
    }
}

fn load_rom_sound(id: &[u8]) -> u32 {
    unsafe { rom_sound(id.as_ptr(), id.len() as u32) }
}

fn load_rom_tracker(id: &[u8]) -> u32 {
    unsafe { rom_tracker(id.as_ptr(), id.len() as u32) }
}

fn cache_song_info(handle: u32) {
    unsafe {
        let info = music_info(handle);
        NUM_CHANNELS = (info >> 24) & 0xFF;
        NUM_PATTERNS = (info >> 16) & 0xFF;
        NUM_INSTRUMENTS = (info >> 8) & 0xFF;
        SONG_LENGTH = info & 0xFF;
        SONG_NAME_LEN = music_name(handle, SONG_NAME.as_mut_ptr(), 32);
    }
}

fn switch_to_song(song_index: u32) {
    unsafe {
        if CURRENT_SONG == song_index {
            return;
        }

        music_stop();
        CURRENT_SONG = song_index;

        // Reset tempo and speed to defaults for the new song
        CURRENT_TEMPO = TEMPOS[song_index as usize];
        CURRENT_SPEED = 6; // Reset to default XM speed (ticks per row)

        let handle = TRACKER_HANDLES[song_index as usize];
        cache_song_info(handle);

        music_play(handle, VOLUME, 1);
        music_set_tempo(CURRENT_TEMPO);
        music_set_speed(CURRENT_SPEED);
        IS_PAUSED = false;
    }
}

fn next_song() {
    unsafe {
        let next = (CURRENT_SONG + 1) % NUM_SONGS;
        switch_to_song(next);
    }
}

fn prev_song() {
    unsafe {
        let prev = if CURRENT_SONG == 0 { NUM_SONGS - 1 } else { CURRENT_SONG - 1 };
        switch_to_song(prev);
    }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_BG);

        // Load Funky Jazz instruments
        load_rom_sound(b"kick_funk");
        load_rom_sound(b"snare_funk");
        load_rom_sound(b"hihat_funk");
        load_rom_sound(b"bass_funk");
        load_rom_sound(b"epiano");
        load_rom_sound(b"lead_jazz");

        // Load Eurobeat instruments
        load_rom_sound(b"kick_euro");
        load_rom_sound(b"snare_euro");
        load_rom_sound(b"hihat_euro");
        load_rom_sound(b"bass_euro");
        load_rom_sound(b"supersaw");
        load_rom_sound(b"brass_euro");
        load_rom_sound(b"pad_euro");

        // Load Synthwave instruments
        load_rom_sound(b"kick_synth");
        load_rom_sound(b"snare_synth");
        load_rom_sound(b"hihat_synth");
        load_rom_sound(b"bass_synth");
        load_rom_sound(b"lead_synth");
        load_rom_sound(b"arp_synth");
        load_rom_sound(b"pad_synth");

        // Load tracker modules
        TRACKER_HANDLES[SONG_FUNK as usize] = load_rom_tracker(b"nether_groove");
        TRACKER_HANDLES[SONG_EURO as usize] = load_rom_tracker(b"nether_fire");
        TRACKER_HANDLES[SONG_SYNTH as usize] = load_rom_tracker(b"nether_drive");

        // Cache info for default song (Funky Jazz)
        cache_song_info(TRACKER_HANDLES[SONG_FUNK as usize]);

        // Start playback with Funky Jazz
        music_play(TRACKER_HANDLES[SONG_FUNK as usize], VOLUME, 1);
        music_set_tempo(CURRENT_TEMPO);
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // A button: Next song
        if button_pressed(0, button::A) != 0 {
            next_song();
        }

        // B button: Previous song
        if button_pressed(0, button::B) != 0 {
            prev_song();
        }

        // X button: Toggle pause/resume
        if button_pressed(0, button::X) != 0 {
            IS_PAUSED = !IS_PAUSED;
            music_pause(if IS_PAUSED { 1 } else { 0 });
        }

        // Y button: Restart from beginning
        if button_pressed(0, button::Y) != 0 {
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

        // Song navigation indicator
        let song_color = match CURRENT_SONG {
            SONG_FUNK => COLOR_FUNK,
            SONG_EURO => COLOR_EURO,
            _ => COLOR_SYNTH,
        };
        draw_text_str(b"[B] Prev", 60.0, 30.0, 14.0, COLOR_GRAY);
        draw_text_str(b"[A] Next", 140.0, 30.0, 14.0, COLOR_GRAY);

        // Song number indicator
        let song_num: &[u8] = match CURRENT_SONG {
            SONG_FUNK => b"1/3",
            SONG_EURO => b"2/3",
            _ => b"3/3",
        };
        draw_text_str(song_num, 220.0, 30.0, 14.0, song_color);

        // Title (song name if available, otherwise default)
        if SONG_NAME_LEN > 0 {
            set_color(song_color);
        draw_text(SONG_NAME.as_ptr(), SONG_NAME_LEN, 380.0, 30.0, 32.0);
        } else {
            let title: &[u8] = match CURRENT_SONG {
                SONG_FUNK => b"Nether Groove",
                SONG_EURO => b"Nether Fire",
                _ => b"Nether Drive",
            };
            draw_text_str(title, 380.0, 30.0, 32.0, song_color);
        }

        // Genre indicator
        let genre: &[u8] = match CURRENT_SONG {
            SONG_FUNK => b"Funky Jazz",
            SONG_EURO => b"Eurobeat",
            _ => b"Synthwave",
        };
        draw_text_str(genre, 420.0, 65.0, 16.0, COLOR_GRAY);

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
        draw_text_str(status_text, 780.0, 30.0, 20.0, status_color);

        // === Left Panel: Position & Timing ===

        let left_x = 60.0;
        let mut y = 110.0;

        draw_text_str(b"POSITION", left_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Order / Song Length progress bar
        draw_text_str(b"Order:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let order_text = format_2digit(order);
        set_color(COLOR_WHITE);
        draw_text(order_text.as_ptr(), 2, left_x + 70.0, y, 16.0);
        draw_text_str(b"/", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);
        let len_text = format_2digit(SONG_LENGTH);
        set_color(COLOR_WHITE);
        draw_text(len_text.as_ptr(), 2, left_x + 105.0, y, 16.0);

        // Order progress bar
        let bar_x = left_x + 140.0;
        set_color(COLOR_DARK_GRAY);
        draw_rect(bar_x, y, 150.0, 14.0);
        if SONG_LENGTH > 0 {
            let progress = (order as f32 / SONG_LENGTH as f32) * 150.0;
            set_color(COLOR_ACCENT2);
        draw_rect(bar_x, y, progress, 14.0);
        }
        y += 25.0;

        // Row display with progress
        draw_text_str(b"Row:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let row_text = format_2digit(row);
        set_color(COLOR_WHITE);
        draw_text(row_text.as_ptr(), 2, left_x + 70.0, y, 16.0);
        draw_text_str(b"/32", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);

        // Row progress bar
        set_color(COLOR_DARK_GRAY);
        draw_rect(bar_x, y, 150.0, 14.0);
        let row_progress = (row as f32 / 32.0) * 150.0;
        set_color(COLOR_ACCENT);
        draw_rect(bar_x, y, row_progress, 14.0);
        y += 35.0;

        // === Timing Section ===
        draw_text_str(b"TIMING", left_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Tempo
        draw_text_str(b"Tempo:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let tempo_text = format_3digit(CURRENT_TEMPO);
        set_color(COLOR_WHITE);
        draw_text(tempo_text.as_ptr(), 3, left_x + 70.0, y, 16.0);
        draw_text_str(b"BPM", left_x + 110.0, y, 14.0, COLOR_DARK_GRAY);
        y += 22.0;

        // Speed
        draw_text_str(b"Speed:", left_x, y, 16.0, COLOR_DARK_GRAY);
        let speed_text = format_2digit(CURRENT_SPEED);
        set_color(COLOR_WHITE);
        draw_text(speed_text.as_ptr(), 2, left_x + 70.0, y, 16.0);
        draw_text_str(b"ticks/row", left_x + 100.0, y, 14.0, COLOR_DARK_GRAY);
        y += 35.0;

        // Volume bar
        draw_text_str(b"Volume:", left_x, y, 16.0, COLOR_DARK_GRAY);
        set_color(COLOR_DARK_GRAY);
        draw_rect(left_x + 70.0, y, 180.0, 16.0);
        set_color(COLOR_ACCENT2);
        draw_rect(left_x + 70.0, y, 180.0 * VOLUME, 16.0);
        let vol_pct = (VOLUME * 100.0) as u32;
        let vol_text = format_3digit(vol_pct);
        set_color(COLOR_WHITE);
        draw_text(vol_text.as_ptr(), 3, left_x + 260.0, y, 14.0);
        draw_text_str(b"%", left_x + 295.0, y, 14.0, COLOR_DARK_GRAY);

        // === Center: Beat Visualizer ===

        let center_x = SCREEN_WIDTH / 2.0;
        let center_y = SCREEN_HEIGHT / 2.0 + 30.0;

        // Different colors for different beats (genre-themed)
        let beat_color = match CURRENT_SONG {
            SONG_FUNK => match row % 4 {
                0 => COLOR_FUNK,      // Kick beat (purple)
                2 => COLOR_ACCENT2,   // Snare beat (teal)
                _ => COLOR_GRAY,      // Hi-hat beats
            },
            SONG_EURO => match row % 4 {
                0 => COLOR_EURO,      // Kick beat (orange)
                2 => COLOR_ACCENT,    // Snare beat (red)
                _ => COLOR_GRAY,      // Hi-hat beats
            },
            _ => match row % 4 {
                0 => COLOR_SYNTH,     // Kick beat (green)
                2 => COLOR_ACCENT2,   // Snare beat (teal)
                _ => COLOR_GRAY,      // Hi-hat beats
            },
        };

        // Pulse effect: larger on beat, smaller between
        let row_frac = (row % 4) as f32 / 4.0;
        let pulse = 1.0 - row_frac * 0.3;
        let base_size = 120.0;
        let size = base_size * pulse;

        let rect_x = center_x - size / 2.0;
        let rect_y = center_y - size / 2.0;
        set_color(beat_color);
        draw_rect(rect_x, rect_y, size, size);

        // Row indicator dots (32 rows in pattern, shown in 2 rows of 16)
        let dot_start_x = center_x - 120.0;
        let dot_y = center_y + 90.0;

        // First row of dots (rows 0-15)
        for i in 0..16u32 {
            let dot_x = dot_start_x + (i as f32) * 16.0;
            let dot_color = if i == row && row < 16 {
                COLOR_WHITE
            } else if i % 4 == 0 {
                song_color
            } else {
                COLOR_DARK_GRAY
            };
            set_color(dot_color);
        draw_rect(dot_x, dot_y, 10.0, 10.0);
        }

        // Second row of dots (rows 16-31)
        let dot_y2 = dot_y + 16.0;
        for i in 0..16u32 {
            let dot_x = dot_start_x + (i as f32) * 16.0;
            let actual_row = i + 16;
            let dot_color = if actual_row == row {
                COLOR_WHITE
            } else if i % 4 == 0 {
                song_color
            } else {
                COLOR_DARK_GRAY
            };
            set_color(dot_color);
        draw_rect(dot_x, dot_y2, 10.0, 10.0);
        }

        // === Right Panel: Track Info ===

        let right_x = 700.0;
        let mut y = 110.0;

        draw_text_str(b"TRACK INFO", right_x, y, 18.0, COLOR_GRAY);
        y += 30.0;

        // Number of channels
        draw_text_str(b"Channels:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let ch_text = format_2digit(NUM_CHANNELS);
        set_color(COLOR_WHITE);
        draw_text(ch_text.as_ptr(), 2, right_x + 100.0, y, 16.0);
        y += 22.0;

        // Number of patterns
        draw_text_str(b"Patterns:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let pat_text = format_2digit(NUM_PATTERNS);
        set_color(COLOR_WHITE);
        draw_text(pat_text.as_ptr(), 2, right_x + 100.0, y, 16.0);
        y += 22.0;

        // Number of instruments
        draw_text_str(b"Instruments:", right_x, y, 16.0, COLOR_DARK_GRAY);
        let inst_text = format_2digit(NUM_INSTRUMENTS);
        set_color(COLOR_WHITE);
        draw_text(inst_text.as_ptr(), 2, right_x + 120.0, y, 16.0);
        y += 35.0;

        // Channel activity
        draw_text_str(b"CHANNELS", right_x, y, 18.0, COLOR_GRAY);
        y += 25.0;

        let active_color = COLOR_PLAYING;
        let inactive_color = COLOR_DARK_GRAY;

        // 8 channel indicators based on current song
        match CURRENT_SONG {
            SONG_FUNK => {
                // Funky Jazz channels
                let ch_names: [&[u8]; 8] = [
                    b"CH1 Kick",
                    b"CH2 Snare",
                    b"CH3 HiHat",
                    b"CH4 Bass",
                    b"CH5 Lead",
                    b"CH6 E.Piano",
                    b"CH7 EPComp",
                    b"CH8 Resp",
                ];

                // Simplified activity based on typical funk patterns
                let kick_active = row % 8 == 0 || row % 8 == 6;
                let snare_active = row % 8 == 4 || (row % 4 == 2 && order > 0);
                let hihat_active = row % 2 == 0;
                let bass_active = row % 4 < 2 || row % 8 == 6;
                let lead_active = order > 0 && row % 4 == 0;
                let ep_active = row % 4 == 0 || row % 4 == 2;
                let comp_active = order > 1 && row % 2 == 0;
                let resp_active = order > 0 && row % 8 == 4;

                let ch_active = [kick_active, snare_active, hihat_active, bass_active,
                               lead_active, ep_active, comp_active, resp_active];

                for i in 0..8 {
                    let color = if ch_active[i] { active_color } else { inactive_color };
                    set_color(color);
        draw_text(ch_names[i].as_ptr(), ch_names[i].len() as u32, right_x, y, 14.0);
                    if ch_active[i] {
                        set_color(active_color);
        draw_rect(right_x + 95.0, y + 2.0, 40.0, 10.0);
                    }
                    y += 18.0;
                }
            },
            SONG_EURO => {
                // Eurobeat channels
                let ch_names: [&[u8]; 8] = [
                    b"CH1 Kick",
                    b"CH2 Snare",
                    b"CH3 HiHat",
                    b"CH4 Bass",
                    b"CH5 Supersaw",
                    b"CH6 Brass",
                    b"CH7 Pad",
                    b"CH8 Harmony",
                ];

                // Eurobeat: 4-on-floor kick, constant hihat, octave bass
                let kick_active = row % 4 == 0;
                let snare_active = row % 8 == 4;
                let hihat_active = row % 2 == 0;
                let bass_active = row % 2 == 0; // Octave bouncing
                let supersaw_active = order >= 4 && row % 4 == 0;
                let brass_active = order >= 1 && (row % 8 == 0 || row % 8 == 6);
                let pad_active = order > 0; // Always on after intro
                let harmony_active = order >= 5 && row % 4 == 0;

                let ch_active = [kick_active, snare_active, hihat_active, bass_active,
                               supersaw_active, brass_active, pad_active, harmony_active];

                for i in 0..8 {
                    let color = if ch_active[i] { active_color } else { inactive_color };
                    set_color(color);
        draw_text(ch_names[i].as_ptr(), ch_names[i].len() as u32, right_x, y, 14.0);
                    if ch_active[i] {
                        set_color(active_color);
        draw_rect(right_x + 95.0, y + 2.0, 40.0, 10.0);
                    }
                    y += 18.0;
                }
            },
            _ => {
                // Synthwave channels
                let ch_names: [&[u8]; 8] = [
                    b"CH1 Kick",
                    b"CH2 Snare",
                    b"CH3 HiHat",
                    b"CH4 Bass",
                    b"CH5 Lead",
                    b"CH6 Arp",
                    b"CH7 Pad",
                    b"CH8 Harmony",
                ];

                // Synthwave: steady beat, arpeggios, pads
                let kick_active = row % 8 == 0 || row % 8 == 4;
                let snare_active = row % 8 == 2 || row % 8 == 6;
                let hihat_active = row % 2 == 0;
                let bass_active = row % 4 == 0; // Quarter notes
                let lead_active = order >= 1 && (row % 4 == 0 || row % 4 == 2);
                let arp_active = order > 0; // Constant arpeggios
                let pad_active = order > 0; // Always after intro
                let harmony_active = order >= 3 && row % 4 == 0;

                let ch_active = [kick_active, snare_active, hihat_active, bass_active,
                               lead_active, arp_active, pad_active, harmony_active];

                for i in 0..8 {
                    let color = if ch_active[i] { active_color } else { inactive_color };
                    set_color(color);
        draw_text(ch_names[i].as_ptr(), ch_names[i].len() as u32, right_x, y, 14.0);
                    if ch_active[i] {
                        set_color(active_color);
        draw_rect(right_x + 95.0, y + 2.0, 40.0, 10.0);
                    }
                    y += 18.0;
                }
            }
        }

        // === Bottom: Controls Help ===

        let help_y = SCREEN_HEIGHT - 60.0;
        draw_text_str(b"Controls:", 60.0, help_y, 16.0, COLOR_GRAY);
        draw_text_str(b"[A/B] Songs  [X] Pause  [Y] Restart  [Up/Dn] Tempo  [L/R] Vol  [LB/RB] Speed",
            60.0, help_y + 22.0, 14.0, COLOR_DARK_GRAY);
    }
}
