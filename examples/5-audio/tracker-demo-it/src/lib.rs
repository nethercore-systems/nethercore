//! IT Tracker Demo
//!
//! Simple IT (Impulse Tracker) music playback demo.
//!
//! Controls:
//! - A: Next song
//! - B: Previous song
//! - X: Pause/Resume
//! - Y: Restart
//! - Up/Down: Adjust tempo
//! - Left/Right: Adjust volume

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// Colors
const COLOR_BG: u32 = 0x1a1a2eFF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x888888FF;
const COLOR_PLAYING: u32 = 0x00FF00FF;
const COLOR_PAUSED: u32 = 0xFFAA00FF;
const COLOR_DAWN: u32 = 0xFFD700FF;
const COLOR_MIST: u32 = 0x6A5ACDFF;
const COLOR_STORM: u32 = 0xFF4500FF;

const NUM_SONGS: u32 = 3;
const TEMPOS: [u32; 3] = [90, 70, 174];

static mut TRACKER_HANDLES: [u32; 3] = [0; 3];
static mut CURRENT_SONG: u32 = 0;
static mut CURRENT_TEMPO: u32 = 90;
static mut VOLUME: f32 = 0.8;
static mut IS_PAUSED: bool = false;

fn draw_text_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        set_color(color);
        draw_text(s.as_ptr(), s.len() as u32, x, y, size);
    }
}

fn format_2digit(n: u32) -> [u8; 2] {
    [b'0' + ((n / 10) % 10) as u8, b'0' + (n % 10) as u8]
}

fn format_3digit(n: u32) -> [u8; 3] {
    [b'0' + ((n / 100) % 10) as u8, b'0' + ((n / 10) % 10) as u8, b'0' + (n % 10) as u8]
}

fn switch_song(index: u32) {
    unsafe {
        music_stop();
        CURRENT_SONG = index;
        CURRENT_TEMPO = TEMPOS[index as usize];
        music_play(TRACKER_HANDLES[index as usize], VOLUME, 1);
        music_set_tempo(CURRENT_TEMPO);
        IS_PAUSED = false;
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_BG);
        TRACKER_HANDLES[0] = rom_tracker(b"nether_dawn".as_ptr(), 11);
        TRACKER_HANDLES[1] = rom_tracker(b"nether_mist".as_ptr(), 11);
        TRACKER_HANDLES[2] = rom_tracker(b"nether_storm".as_ptr(), 12);
        music_play(TRACKER_HANDLES[0], VOLUME, 1);
        music_set_tempo(CURRENT_TEMPO);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            switch_song((CURRENT_SONG + 1) % NUM_SONGS);
        }
        if button_pressed(0, button::B) != 0 {
            switch_song(if CURRENT_SONG == 0 { NUM_SONGS - 1 } else { CURRENT_SONG - 1 });
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
            CURRENT_TEMPO = (CURRENT_TEMPO + 10).min(250);
            music_set_tempo(CURRENT_TEMPO);
        }
        if button_pressed(0, button::DOWN) != 0 {
            CURRENT_TEMPO = CURRENT_TEMPO.saturating_sub(10).max(30);
            music_set_tempo(CURRENT_TEMPO);
        }
        if button_held(0, button::RIGHT) != 0 {
            VOLUME = (VOLUME + 0.02).min(1.0);
            music_set_volume(VOLUME);
        }
        if button_held(0, button::LEFT) != 0 {
            VOLUME = (VOLUME - 0.02).max(0.0);
            music_set_volume(VOLUME);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let pos = music_position();
        let order = (pos >> 16) as u32;
        let row = (pos & 0xFFFF) as u32;

        // Song info
        let (title, color): (&[u8], u32) = match CURRENT_SONG {
            0 => (b"Nether Dawn (Orchestral)", COLOR_DAWN),
            1 => (b"Nether Mist (Ambient)", COLOR_MIST),
            _ => (b"Nether Storm (DnB)", COLOR_STORM),
        };
        draw_text_str(title, 60.0, 60.0, 32.0, color);

        // Status
        let status = if IS_PAUSED { b"PAUSED" as &[u8] } else { b"PLAYING" };
        let status_color = if IS_PAUSED { COLOR_PAUSED } else { COLOR_PLAYING };
        draw_text_str(status, 60.0, 110.0, 20.0, status_color);

        // Position
        draw_text_str(b"Order:", 60.0, 160.0, 16.0, COLOR_GRAY);
        let order_text = format_2digit(order);
        set_color(COLOR_WHITE);
        draw_text(order_text.as_ptr(), 2, 130.0, 160.0, 16.0);

        draw_text_str(b"Row:", 180.0, 160.0, 16.0, COLOR_GRAY);
        let row_text = format_2digit(row);
        set_color(COLOR_WHITE);
        draw_text(row_text.as_ptr(), 2, 230.0, 160.0, 16.0);

        // Tempo and Volume
        draw_text_str(b"Tempo:", 60.0, 200.0, 16.0, COLOR_GRAY);
        let tempo_text = format_3digit(CURRENT_TEMPO);
        set_color(COLOR_WHITE);
        draw_text(tempo_text.as_ptr(), 3, 130.0, 200.0, 16.0);

        draw_text_str(b"Volume:", 200.0, 200.0, 16.0, COLOR_GRAY);
        let vol_pct = (VOLUME * 100.0) as u32;
        let vol_text = format_3digit(vol_pct);
        set_color(COLOR_WHITE);
        draw_text(vol_text.as_ptr(), 3, 280.0, 200.0, 16.0);
        draw_text_str(b"%", 310.0, 200.0, 16.0, COLOR_GRAY);

        // Controls
        draw_text_str(b"[A/B] Song  [X] Pause  [Y] Restart  [Up/Dn] Tempo  [L/R] Vol",
            60.0, 280.0, 14.0, COLOR_GRAY);
    }
}
