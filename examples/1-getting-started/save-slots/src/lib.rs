//!
//! Save slots demo.
//!
//! Demonstrates the `save/load/delete` API:
//! - Slots 0-3 are persistent and controller-backed (for local session slots)
//! - Slots 4-7 are ephemeral
//!
//! Controls (per player):
//! - A: increment counter
//! - START: save counter to that player's session slot
//! - SELECT: load counter from that player's session slot
//! - B: delete counter in that player's session slot

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

const COLOR_BG: u32 = 0x121826FF;
const COLOR_TEXT: u32 = 0xE6EEF8FF;
const COLOR_MUTED: u32 = 0x8CA0B8FF;
const COLOR_ACCENT: u32 = 0x6EE7B7FF;

const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;

static mut COUNTERS: [u32; 4] = [0; 4];

fn u32_to_4digits(mut n: u32) -> [u8; 4] {
    n %= 10_000;
    let d3 = (n % 10) as u8;
    n /= 10;
    let d2 = (n % 10) as u8;
    n /= 10;
    let d1 = (n % 10) as u8;
    n /= 10;
    let d0 = (n % 10) as u8;
    [b'0' + d0, b'0' + d1, b'0' + d2, b'0' + d3]
}

fn load_counter_for_slot(slot: u32) -> Option<u32> {
    let mut buf = [0u8; 4];
    let read = unsafe { load(slot, buf.as_mut_ptr(), buf.len() as u32) };
    if read != 4 {
        return None;
    }
    Some(u32::from_le_bytes(buf))
}

fn save_counter_for_slot(slot: u32, value: u32) {
    let bytes = value.to_le_bytes();
    unsafe {
        let _ = save(slot, bytes.as_ptr(), bytes.len() as u32);
    }
}

fn delete_counter_for_slot(slot: u32) {
    unsafe {
        let _ = delete(slot);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_BG);

        // Load any existing persistent counters.
        let pc = player_count().min(4);
        for p in 0..pc {
            if let Some(v) = load_counter_for_slot(p) {
                COUNTERS[p as usize] = v;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let pc = player_count().min(4);
        for p in 0..pc {
            let pi = p as usize;

            if button_pressed(p, BUTTON_A) != 0 {
                COUNTERS[pi] = COUNTERS[pi].wrapping_add(1);
            }

            if button_pressed(p, BUTTON_START) != 0 {
                save_counter_for_slot(p, COUNTERS[pi]);
            }

            if button_pressed(p, BUTTON_SELECT) != 0 {
                if let Some(v) = load_counter_for_slot(p) {
                    COUNTERS[pi] = v;
                }
            }

            if button_pressed(p, BUTTON_B) != 0 {
                delete_counter_for_slot(p);
                COUNTERS[pi] = 0;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        set_color(COLOR_TEXT);
        let title = b"Save Slots";
        draw_text(title.as_ptr(), title.len() as u32, 32.0, 32.0, 40.0);

        set_color(COLOR_MUTED);
        let hint = b"A:+1  START:save  SELECT:load  B:delete (per player)";
        draw_text(hint.as_ptr(), hint.len() as u32, 32.0, 84.0, 18.0);

        let pc = player_count().min(4);
        let mask = local_player_mask();

        // Header line
        set_color(COLOR_MUTED);
        let header = b"slot  local  counter";
        draw_text(header.as_ptr(), header.len() as u32, 32.0, 130.0, 18.0);

        for p in 0..pc {
            let y = 160.0 + (p as f32) * 32.0;
            let local = ((mask & (1 << p)) != 0) as u8;
            let digits = u32_to_4digits(COUNTERS[p as usize]);

            // "P<d>" label
            set_color(COLOR_TEXT);
            let label = [b'P', b'0' + (p as u8)];
            draw_text(label.as_ptr(), label.len() as u32, 32.0, y, 20.0);

            // local marker
            set_color(if local != 0 {
                COLOR_ACCENT
            } else {
                COLOR_MUTED
            });
            let local_text: &[u8] = if local != 0 { b"yes" } else { b"no" };
            draw_text(local_text.as_ptr(), local_text.len() as u32, 90.0, y, 20.0);

            // counter digits
            set_color(COLOR_TEXT);
            draw_text(digits.as_ptr(), digits.len() as u32, 170.0, y, 20.0);
        }

        set_color(COLOR_MUTED);
        let note = b"Note: only local slots persist to disk (netplay-safe).";
        draw_text(note.as_ptr(), note.len() as u32, 32.0, 320.0, 18.0);
    }
}
