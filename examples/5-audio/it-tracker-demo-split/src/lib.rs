//! IT Tracker Demo (Split Assets)
//!
//! Demonstrates IT tracker playback with explicitly loaded sample assets.
//! This version shows how samples can be used both for tracker playback
//! AND as standalone sound effects.
//!
//! Features:
//! - IT tracker music playback (same as embedded version)
//! - Explicit sample loading from WAV files
//! - Sample preview mode (press Start to toggle)
//! - Individual sample playback (D-pad in preview mode)
//!
//! Controls:
//! - A button: Next song
//! - B button: Previous song
//! - X button: Pause/Resume playback
//! - Y button: Restart from beginning
//! - Start: Toggle sample preview mode
//! - Up/Down (preview mode): Navigate samples
//! - A (preview mode): Play selected sample
//! - Up/Down (normal): Adjust tempo
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

    // Sound playback
    fn sound_play(handle: u32, volume: f32);

    // Unified Music API
    fn music_play(handle: u32, volume: f32, looping: u32);
    fn music_stop();
    fn music_pause(paused: u32);
    fn music_set_volume(volume: f32);
    fn music_is_playing() -> u32;
    fn music_jump(order: u32, row: u32);
    fn music_position() -> u32;
    fn music_set_speed(speed: u32);
    fn music_set_tempo(bpm: u32);
    fn music_info(handle: u32) -> u32;
    fn music_name(handle: u32, out_ptr: *mut u8, max_len: u32) -> u32;
}

// === Constants ===

const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

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
    pub const START: u32 = 10;
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
const COLOR_DAWN: u32 = 0xFFD700FF;
const COLOR_MIST: u32 = 0x6A5ACDFF;
const COLOR_STORM: u32 = 0xFF4500FF;
const COLOR_PREVIEW: u32 = 0xFF00FFFF; // Magenta for preview mode

// Song constants
const SONG_DAWN: u32 = 0;
const SONG_MIST: u32 = 1;
const SONG_STORM: u32 = 2;
const NUM_SONGS: u32 = 3;

const TEMPOS: [u32; 3] = [90, 70, 174];
const SPEEDS: [u32; 3] = [6, 6, 3];

// Sample counts per song
const DAWN_SAMPLES: u32 = 16;
const MIST_SAMPLES: u32 = 10;
const STORM_SAMPLES: u32 = 15;

// === Global State ===

static mut TRACKER_HANDLES: [u32; 3] = [0; 3];
static mut CURRENT_SONG: u32 = SONG_DAWN;
static mut CURRENT_TEMPO: u32 = 90;
static mut CURRENT_SPEED: u32 = 6;
static mut VOLUME: f32 = 0.8;
static mut IS_PAUSED: bool = false;

static mut SONG_LENGTH: u32 = 0;
static mut NUM_CHANNELS: u32 = 0;
static mut NUM_PATTERNS: u32 = 0;
static mut NUM_INSTRUMENTS: u32 = 0;
static mut SONG_NAME: [u8; 32] = [0u8; 32];
static mut SONG_NAME_LEN: u32 = 0;

// Sample preview mode
static mut PREVIEW_MODE: bool = false;
static mut SELECTED_SAMPLE: u32 = 0;

// Sound handles (loaded explicitly from WAV files)
static mut DAWN_SOUND_HANDLES: [u32; 16] = [0; 16];
static mut MIST_SOUND_HANDLES: [u32; 10] = [0; 10];
static mut STORM_SOUND_HANDLES: [u32; 15] = [0; 15];

// Sample names for display
const DAWN_SAMPLE_NAMES: [&[u8]; 16] = [
    b"Cello", b"Viola", b"Violin", b"Horn",
    b"Trumpet", b"Flute", b"Timpani", b"Snare Orch",
    b"Cymbal", b"Harp", b"Choir Ah", b"Choir Oh",
    b"Piano", b"Bass Epic", b"Pad Orch", b"FX Epic"
];

const MIST_SAMPLE_NAMES: [&[u8]; 10] = [
    b"Sub Pad", b"Air Pad", b"Warm Pad", b"Cold Pad",
    b"Breath", b"Bell Glass", b"Sub Bass", b"Ghost Lead",
    b"Reverb", b"Wind"
];

const STORM_SAMPLE_NAMES: [&[u8]; 15] = [
    b"Kick", b"Snare", b"HH Closed", b"HH Open",
    b"Break", b"Cymbal", b"Sub Bass", b"Reese",
    b"Wobble", b"Dark Pad", b"Lead Stab", b"Lead Main",
    b"Riser", b"Impact", b"Atmos"
];

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
        CURRENT_TEMPO = TEMPOS[song_index as usize];
        CURRENT_SPEED = SPEEDS[song_index as usize];
        SELECTED_SAMPLE = 0; // Reset sample selection

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

fn get_sample_count() -> u32 {
    unsafe {
        match CURRENT_SONG {
            SONG_DAWN => DAWN_SAMPLES,
            SONG_MIST => MIST_SAMPLES,
            _ => STORM_SAMPLES,
        }
    }
}

fn play_selected_sample() {
    unsafe {
        let handle = match CURRENT_SONG {
            SONG_DAWN => DAWN_SOUND_HANDLES[SELECTED_SAMPLE as usize],
            SONG_MIST => MIST_SOUND_HANDLES[SELECTED_SAMPLE as usize],
            _ => STORM_SOUND_HANDLES[SELECTED_SAMPLE as usize],
        };
        sound_play(handle, 1.0);
    }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_BG);

        // Load Nether Dawn samples (explicitly from WAV files)
        DAWN_SOUND_HANDLES[0] = load_rom_sound(b"strings_cello");
        DAWN_SOUND_HANDLES[1] = load_rom_sound(b"strings_viola");
        DAWN_SOUND_HANDLES[2] = load_rom_sound(b"strings_violin");
        DAWN_SOUND_HANDLES[3] = load_rom_sound(b"brass_horn");
        DAWN_SOUND_HANDLES[4] = load_rom_sound(b"brass_trumpet");
        DAWN_SOUND_HANDLES[5] = load_rom_sound(b"flute");
        DAWN_SOUND_HANDLES[6] = load_rom_sound(b"timpani");
        DAWN_SOUND_HANDLES[7] = load_rom_sound(b"snare_orch");
        DAWN_SOUND_HANDLES[8] = load_rom_sound(b"cymbal_crash");
        DAWN_SOUND_HANDLES[9] = load_rom_sound(b"harp_gliss");
        DAWN_SOUND_HANDLES[10] = load_rom_sound(b"choir_ah");
        DAWN_SOUND_HANDLES[11] = load_rom_sound(b"choir_oh");
        DAWN_SOUND_HANDLES[12] = load_rom_sound(b"piano");
        DAWN_SOUND_HANDLES[13] = load_rom_sound(b"bass_epic");
        DAWN_SOUND_HANDLES[14] = load_rom_sound(b"pad_orchestra");
        DAWN_SOUND_HANDLES[15] = load_rom_sound(b"fx_epic");

        // Load Nether Mist samples
        MIST_SOUND_HANDLES[0] = load_rom_sound(b"pad_sub");
        MIST_SOUND_HANDLES[1] = load_rom_sound(b"pad_air");
        MIST_SOUND_HANDLES[2] = load_rom_sound(b"pad_warm");
        MIST_SOUND_HANDLES[3] = load_rom_sound(b"pad_cold");
        MIST_SOUND_HANDLES[4] = load_rom_sound(b"noise_breath");
        MIST_SOUND_HANDLES[5] = load_rom_sound(b"bell_glass");
        MIST_SOUND_HANDLES[6] = load_rom_sound(b"bass_sub");
        MIST_SOUND_HANDLES[7] = load_rom_sound(b"lead_ghost");
        MIST_SOUND_HANDLES[8] = load_rom_sound(b"reverb_sim");
        MIST_SOUND_HANDLES[9] = load_rom_sound(b"atmos_wind");

        // Load Nether Storm samples
        STORM_SOUND_HANDLES[0] = load_rom_sound(b"kick_dnb");
        STORM_SOUND_HANDLES[1] = load_rom_sound(b"snare_dnb");
        STORM_SOUND_HANDLES[2] = load_rom_sound(b"hihat_closed");
        STORM_SOUND_HANDLES[3] = load_rom_sound(b"hihat_open");
        STORM_SOUND_HANDLES[4] = load_rom_sound(b"break_slice");
        STORM_SOUND_HANDLES[5] = load_rom_sound(b"cymbal");
        STORM_SOUND_HANDLES[6] = load_rom_sound(b"bass_sub_dnb");
        STORM_SOUND_HANDLES[7] = load_rom_sound(b"bass_reese");
        STORM_SOUND_HANDLES[8] = load_rom_sound(b"bass_wobble");
        STORM_SOUND_HANDLES[9] = load_rom_sound(b"pad_dark");
        STORM_SOUND_HANDLES[10] = load_rom_sound(b"lead_stab");
        STORM_SOUND_HANDLES[11] = load_rom_sound(b"lead_main");
        STORM_SOUND_HANDLES[12] = load_rom_sound(b"fx_riser");
        STORM_SOUND_HANDLES[13] = load_rom_sound(b"fx_impact");
        STORM_SOUND_HANDLES[14] = load_rom_sound(b"atmos_storm");

        // Load tracker modules
        TRACKER_HANDLES[SONG_DAWN as usize] = load_rom_tracker(b"nether_dawn");
        TRACKER_HANDLES[SONG_MIST as usize] = load_rom_tracker(b"nether_mist");
        TRACKER_HANDLES[SONG_STORM as usize] = load_rom_tracker(b"nether_storm");

        // Cache info and start playback
        cache_song_info(TRACKER_HANDLES[SONG_DAWN as usize]);
        music_play(TRACKER_HANDLES[SONG_DAWN as usize], VOLUME, 1);
        music_set_tempo(CURRENT_TEMPO);
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Start: Toggle preview mode
        if button_pressed(0, button::START) != 0 {
            PREVIEW_MODE = !PREVIEW_MODE;
            if PREVIEW_MODE {
                music_pause(1); // Pause music in preview mode
            } else {
                music_pause(if IS_PAUSED { 1 } else { 0 });
            }
        }

        if PREVIEW_MODE {
            // Preview mode controls
            let sample_count = get_sample_count();

            if button_pressed(0, button::UP) != 0 {
                SELECTED_SAMPLE = if SELECTED_SAMPLE == 0 {
                    sample_count - 1
                } else {
                    SELECTED_SAMPLE - 1
                };
            }
            if button_pressed(0, button::DOWN) != 0 {
                SELECTED_SAMPLE = (SELECTED_SAMPLE + 1) % sample_count;
            }
            if button_pressed(0, button::A) != 0 {
                play_selected_sample();
            }

            // Song switching still works
            if button_pressed(0, button::L1) != 0 {
                prev_song();
            }
            if button_pressed(0, button::R1) != 0 {
                next_song();
            }
        } else {
            // Normal mode controls
            if button_pressed(0, button::A) != 0 {
                next_song();
            }
            if button_pressed(0, button::B) != 0 {
                prev_song();
            }
            if button_pressed(0, button::X) != 0 {
                IS_PAUSED = !IS_PAUSED;
                music_pause(if IS_PAUSED { 1 } else { 0 });
            }
            if button_pressed(0, button::Y) != 0 {
                music_jump(0, 0);
                if IS_PAUSED {
                    IS_PAUSED = false;
                    music_pause(0);
                }
            }
            if button_pressed(0, button::UP) != 0 {
                CURRENT_TEMPO = if CURRENT_TEMPO < 250 { CURRENT_TEMPO + 10 } else { 250 };
                music_set_tempo(CURRENT_TEMPO);
            }
            if button_pressed(0, button::DOWN) != 0 {
                CURRENT_TEMPO = if CURRENT_TEMPO > 30 { CURRENT_TEMPO - 10 } else { 30 };
                music_set_tempo(CURRENT_TEMPO);
            }
            if button_held(0, button::RIGHT) != 0 {
                VOLUME += 0.02;
                if VOLUME > 1.0 { VOLUME = 1.0; }
                music_set_volume(VOLUME);
            }
            if button_held(0, button::LEFT) != 0 {
                VOLUME -= 0.02;
                if VOLUME < 0.0 { VOLUME = 0.0; }
                music_set_volume(VOLUME);
            }
        }
    }
}

// === Render ===

fn format_2digit(n: u32) -> [u8; 2] {
    [b'0' + ((n / 10) % 10) as u8, b'0' + (n % 10) as u8]
}

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
        let song_color = match CURRENT_SONG {
            SONG_DAWN => COLOR_DAWN,
            SONG_MIST => COLOR_MIST,
            _ => COLOR_STORM,
        };

        if PREVIEW_MODE {
            // === Sample Preview Mode ===
            draw_text_str(b"SAMPLE PREVIEW MODE", 60.0, 30.0, 24.0, COLOR_PREVIEW);
            draw_text_str(b"[Start] Exit  [Up/Down] Select  [A] Play  [LB/RB] Switch Song",
                60.0, 60.0, 14.0, COLOR_GRAY);

            // Current song
            let song_name: &[u8] = match CURRENT_SONG {
                SONG_DAWN => b"Nether Dawn (Orchestral)",
                SONG_MIST => b"Nether Mist (Ambient)",
                _ => b"Nether Storm (DnB)",
            };
            draw_text_str(song_name, 60.0, 100.0, 20.0, song_color);

            // Sample list
            let sample_count = get_sample_count();
            let (sample_names, _): (&[&[u8]], u32) = match CURRENT_SONG {
                SONG_DAWN => (&DAWN_SAMPLE_NAMES[..], DAWN_SAMPLES),
                SONG_MIST => (&MIST_SAMPLE_NAMES[..], MIST_SAMPLES),
                _ => (&STORM_SAMPLE_NAMES[..], STORM_SAMPLES),
            };

            let start_y = 140.0;
            let items_per_col = 10;

            for i in 0..sample_count {
                let col = i / items_per_col;
                let row = i % items_per_col;
                let x = 80.0 + (col as f32) * 280.0;
                let y = start_y + (row as f32) * 28.0;

                let is_selected = i == SELECTED_SAMPLE;
                let color = if is_selected { COLOR_WHITE } else { COLOR_GRAY };

                // Selection indicator
                if is_selected {
                    draw_rect(x - 20.0, y, 12.0, 18.0, COLOR_PREVIEW);
                }

                // Sample number
                let num = format_2digit(i + 1);
                draw_text(num.as_ptr(), 2, x, y, 16.0, COLOR_DARK_GRAY);

                // Sample name
                let name = sample_names[i as usize];
                draw_text(name.as_ptr(), name.len() as u32, x + 35.0, y, 16.0, color);
            }

            // Instructions
            draw_text_str(b"Press [A] to play the selected sample", 60.0, SCREEN_HEIGHT - 40.0, 16.0, COLOR_PREVIEW);

        } else {
            // === Normal Playback Mode ===
            let pos = music_position();
            let order = (pos >> 16) as u32;
            let row = (pos & 0xFFFF) as u32;

            // Header
            draw_text_str(b"[B] Prev", 60.0, 30.0, 14.0, COLOR_GRAY);
            draw_text_str(b"[A] Next", 140.0, 30.0, 14.0, COLOR_GRAY);
            draw_text_str(b"[Start] Samples", 220.0, 30.0, 14.0, COLOR_PREVIEW);

            // Title
            if SONG_NAME_LEN > 0 {
                draw_text(SONG_NAME.as_ptr(), SONG_NAME_LEN, 420.0, 30.0, 32.0, song_color);
            } else {
                let title: &[u8] = match CURRENT_SONG {
                    SONG_DAWN => b"Nether Dawn",
                    SONG_MIST => b"Nether Mist",
                    _ => b"Nether Storm",
                };
                draw_text_str(title, 420.0, 30.0, 32.0, song_color);
            }

            // Genre
            let genre: &[u8] = match CURRENT_SONG {
                SONG_DAWN => b"Orchestral",
                SONG_MIST => b"Ambient",
                _ => b"DnB / Action",
            };
            draw_text_str(genre, 460.0, 65.0, 16.0, COLOR_GRAY);

            // Status
            let is_playing = music_is_playing() != 0 && !IS_PAUSED;
            let status_color = if is_playing { COLOR_PLAYING } else { COLOR_PAUSED };
            let status_text: &[u8] = if IS_PAUSED { b"PAUSED" } else if is_playing { b"PLAYING" } else { b"STOPPED" };
            draw_text_str(status_text, 780.0, 30.0, 20.0, status_color);

            // Position info
            let left_x = 60.0;
            let mut y = 110.0;

            draw_text_str(b"POSITION", left_x, y, 18.0, COLOR_GRAY);
            y += 30.0;

            draw_text_str(b"Order:", left_x, y, 16.0, COLOR_DARK_GRAY);
            let order_text = format_2digit(order);
            draw_text(order_text.as_ptr(), 2, left_x + 70.0, y, 16.0, COLOR_WHITE);
            draw_text_str(b"/", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);
            let len_text = format_2digit(SONG_LENGTH);
            draw_text(len_text.as_ptr(), 2, left_x + 105.0, y, 16.0, COLOR_WHITE);

            let bar_x = left_x + 140.0;
            draw_rect(bar_x, y, 150.0, 14.0, COLOR_DARK_GRAY);
            if SONG_LENGTH > 0 {
                let progress = (order as f32 / SONG_LENGTH as f32) * 150.0;
                draw_rect(bar_x, y, progress, 14.0, COLOR_ACCENT2);
            }
            y += 25.0;

            draw_text_str(b"Row:", left_x, y, 16.0, COLOR_DARK_GRAY);
            let row_text = format_2digit(row);
            draw_text(row_text.as_ptr(), 2, left_x + 70.0, y, 16.0, COLOR_WHITE);
            draw_text_str(b"/64", left_x + 95.0, y, 16.0, COLOR_DARK_GRAY);

            draw_rect(bar_x, y, 150.0, 14.0, COLOR_DARK_GRAY);
            let row_progress = (row as f32 / 64.0) * 150.0;
            draw_rect(bar_x, y, row_progress, 14.0, COLOR_ACCENT);
            y += 35.0;

            // Timing
            draw_text_str(b"TIMING", left_x, y, 18.0, COLOR_GRAY);
            y += 30.0;

            draw_text_str(b"Tempo:", left_x, y, 16.0, COLOR_DARK_GRAY);
            let tempo_text = format_3digit(CURRENT_TEMPO);
            draw_text(tempo_text.as_ptr(), 3, left_x + 70.0, y, 16.0, COLOR_WHITE);
            draw_text_str(b"BPM", left_x + 110.0, y, 14.0, COLOR_DARK_GRAY);
            y += 22.0;

            draw_text_str(b"Volume:", left_x, y, 16.0, COLOR_DARK_GRAY);
            draw_rect(left_x + 70.0, y, 180.0, 16.0, COLOR_DARK_GRAY);
            draw_rect(left_x + 70.0, y, 180.0 * VOLUME, 16.0, COLOR_ACCENT2);

            // Center visualizer
            let center_x = SCREEN_WIDTH / 2.0;
            let center_y = SCREEN_HEIGHT / 2.0 + 30.0;

            let beat_color = match CURRENT_SONG {
                SONG_DAWN => if row % 16 == 0 { COLOR_DAWN } else { COLOR_DARK_GRAY },
                SONG_MIST => if row % 16 == 0 { COLOR_MIST } else { COLOR_DARK_GRAY },
                _ => match row % 4 {
                    0 => COLOR_STORM,
                    2 => COLOR_ACCENT,
                    _ => COLOR_GRAY,
                },
            };

            let pulse_period = if CURRENT_SONG == SONG_STORM { 4 } else { 16 };
            let row_frac = (row % pulse_period) as f32 / pulse_period as f32;
            let pulse = 1.0 - row_frac * 0.3;
            let size = 120.0 * pulse;

            draw_rect(center_x - size / 2.0, center_y - size / 2.0, size, size, beat_color);

            // Track info (right panel)
            let right_x = 700.0;
            let mut y = 110.0;

            draw_text_str(b"TRACK INFO", right_x, y, 18.0, COLOR_GRAY);
            y += 30.0;

            draw_text_str(b"Channels:", right_x, y, 16.0, COLOR_DARK_GRAY);
            let ch_text = format_2digit(NUM_CHANNELS);
            draw_text(ch_text.as_ptr(), 2, right_x + 100.0, y, 16.0, COLOR_WHITE);
            y += 22.0;

            draw_text_str(b"Patterns:", right_x, y, 16.0, COLOR_DARK_GRAY);
            let pat_text = format_2digit(NUM_PATTERNS);
            draw_text(pat_text.as_ptr(), 2, right_x + 100.0, y, 16.0, COLOR_WHITE);
            y += 22.0;

            draw_text_str(b"Samples:", right_x, y, 16.0, COLOR_DARK_GRAY);
            let sample_count = get_sample_count();
            let samp_text = format_2digit(sample_count);
            draw_text(samp_text.as_ptr(), 2, right_x + 100.0, y, 16.0, COLOR_WHITE);
            y += 35.0;

            draw_text_str(b"SPLIT ASSETS MODE", right_x, y, 14.0, COLOR_PREVIEW);
            y += 20.0;
            draw_text_str(b"Samples loaded from", right_x, y, 12.0, COLOR_DARK_GRAY);
            y += 16.0;
            draw_text_str(b"separate WAV files", right_x, y, 12.0, COLOR_DARK_GRAY);

            // Controls help
            let help_y = SCREEN_HEIGHT - 60.0;
            draw_text_str(b"Controls:", 60.0, help_y, 16.0, COLOR_GRAY);
            draw_text_str(b"[A/B] Songs  [X] Pause  [Y] Restart  [Up/Dn] Tempo  [L/R] Vol  [Start] Samples",
                60.0, help_y + 22.0, 14.0, COLOR_DARK_GRAY);
        }
    }
}
