#![no_std]
#![no_main]

#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

// Global state
static mut BEEP_SOUND: u32 = 0;
static mut PAN_POSITION: f32 = 0.0;  // -1.0 to 1.0
static mut IS_PLAYING: bool = false;
static mut AUTO_MODE: bool = true;
static mut AUTO_TIMER: f32 = 0.0;

/// Generate a smooth sine wave tone (440Hz, 1 second, seamless loop)
fn generate_tone() -> [i16; 22050] {  // 22050 Hz × 1s = 22050 samples
    let mut samples = [0i16; 22050];
    let frequency = 440.0;  // A4 note
    let sample_rate = 22050.0;

    // Calculate samples per complete wave cycle for seamless looping
    let samples_per_cycle = sample_rate / frequency;
    // Find number of complete cycles that fit in our buffer
    let complete_cycles = libm::floorf(22050.0 / samples_per_cycle);
    let actual_samples = (complete_cycles * samples_per_cycle) as usize;

    for i in 0..actual_samples {
        // Sine wave for smooth sound
        let t = i as f32 / sample_rate;
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t);

        samples[i] = (value * 32767.0 * 0.25) as i16;  // 25% volume
    }

    // Fill remaining samples to complete the buffer (maintains loop point)
    for i in actual_samples..22050 {
        let t = i as f32 / sample_rate;
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t);
        samples[i] = (value * 32767.0 * 0.25) as i16;
    }

    samples
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Set clear color
        set_clear_color(0x19172AFF); // Dark blue-purple

        // Generate and load smooth tone
        let tone_samples = generate_tone();
        BEEP_SOUND = load_sound(
            tone_samples.as_ptr(),
            (tone_samples.len() * 2) as u32  // byte length
        );

        // Start in auto mode with a sound playing
        channel_play(0, BEEP_SOUND, 0.5, 0.0, 1);  // Looping tone at 50% volume
        IS_PLAYING = true;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Toggle auto mode with B button
        if button_pressed(0, button::B) != 0 {
            AUTO_MODE = !AUTO_MODE;
            if !AUTO_MODE {
                AUTO_TIMER = 0.0;
            }
        }

        if AUTO_MODE {
            // Auto mode: pan oscillates left-right smoothly (triangle wave)
            AUTO_TIMER += 0.016;  // Assuming ~60 FPS

            // Triangle wave: -1 to 1 and back, period ~3 seconds
            let cycle_time = AUTO_TIMER % 3.0;
            PAN_POSITION = if cycle_time < 1.5 {
                (cycle_time / 1.5) * 2.0 - 1.0  // -1 to 1
            } else {
                1.0 - ((cycle_time - 1.5) / 1.5) * 2.0  // 1 to -1
            };
        } else {
            // Manual mode: use D-pad to control pan
            if button_held(0, button::LEFT) != 0 {
                PAN_POSITION -= 0.02;
                if PAN_POSITION < -1.0 {
                    PAN_POSITION = -1.0;
                }
            }
            if button_held(0, button::RIGHT) != 0 {
                PAN_POSITION += 0.02;
                if PAN_POSITION > 1.0 {
                    PAN_POSITION = 1.0;
                }
            }
        }

        // Toggle playback with A button
        if button_pressed(0, button::A) != 0 {
            if IS_PLAYING {
                channel_stop(0);
                IS_PLAYING = false;
            } else {
                channel_play(0, BEEP_SOUND, 0.5, PAN_POSITION, 1);
                IS_PLAYING = true;
            }
        }

        // Update pan in real-time
        if IS_PLAYING {
            channel_set(0, 0.5, PAN_POSITION);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 0.0, 3.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Title
        let title = b"Audio Panning Demo";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 20.0, 20.0, 32.0);

        // Instructions
        let line1: &[u8] = if AUTO_MODE {
            b"Mode: AUTO (sound pans left-right)"
        } else {
            b"Mode: MANUAL (use D-pad to pan)"
        };
        set_color(0xFFAAAAFF);
        draw_text(line1.as_ptr(), line1.len() as u32, 20.0, 70.0, 18.0);

        let line2 = b"Controls:";
        set_color(0xFFFFFFFF);
        draw_text(line2.as_ptr(), line2.len() as u32, 20.0, 110.0, 20.0);

        let line3 = b"  [A] Play/Stop";
        set_color(0xCCCCCCFF);
        draw_text(line3.as_ptr(), line3.len() as u32, 20.0, 145.0, 18.0);

        let line4 = b"  [B] Toggle Auto/Manual";
        set_color(0xCCCCCCFF);
        draw_text(line4.as_ptr(), line4.len() as u32, 20.0, 175.0, 18.0);

        let line5: &[u8] = if AUTO_MODE {
            b"  (Auto mode active)"
        } else {
            b"  [D-pad] Adjust pan"
        };
        set_color(0xCCCCCCFF);
        draw_text(line5.as_ptr(), line5.len() as u32, 20.0, 205.0, 18.0);

        // Visual pan indicator
        let line6 = b"Pan Position:";
        set_color(0xFFFFFFFF);
        draw_text(line6.as_ptr(), line6.len() as u32, 20.0, 260.0, 20.0);

        // Draw pan slider background
        set_color(0x333333FF);
        draw_rect(80.0, 300.0, 480.0, 30.0);

        // Draw center line
        set_color(0x666666FF);
        draw_rect(318.0, 290.0, 4.0, 50.0);

        // Draw pan indicator
        let indicator_x = 320.0 + (PAN_POSITION * 240.0);
        let indicator_color = if IS_PLAYING {
            0x00FF00FF  // Green when playing
        } else {
            0x666666FF  // Gray when stopped
        };
        set_color(indicator_color);
        draw_rect(indicator_x - 10.0, 290.0, 20.0, 50.0);

        // Draw speaker labels
        let left_label = b"L";
        set_color(0xFFFFFFFF);
        draw_text(left_label.as_ptr(), left_label.len() as u32, 50.0, 300.0, 24.0);

        let right_label = b"R";
        set_color(0xFFFFFFFF);
        draw_text(right_label.as_ptr(), right_label.len() as u32, 575.0, 300.0, 24.0);

        // Show pan value
        let mut pan_text = [0u8; 32];
        let pan_percent = (PAN_POSITION * 100.0) as i32;
        let pan_str = format_pan(pan_percent, &mut pan_text);
        set_color(0xFFFFFFFF);
        draw_text(pan_str.as_ptr(), pan_str.len() as u32, 260.0, 370.0, 20.0);

        // Status
        let status: &[u8] = if IS_PLAYING {
            b"Status: PLAYING (440Hz tone)"
        } else {
            b"Status: STOPPED"
        };
        let status_color = if IS_PLAYING { 0x00FF00FF } else { 0xFF0000FF };  // Green/Red
        set_color(status_color);
        draw_text(status.as_ptr(), status.len() as u32, 20.0, 420.0, 20.0);
    }
}

// Simple formatting helpers (no_std compatible)
fn format_pan(value: i32, buffer: &mut [u8]) -> &[u8] {
    let mut idx = 0;

    // "Pan: "
    buffer[idx..idx + 5].copy_from_slice(b"Pan: ");
    idx += 5;

    // Sign
    if value < 0 {
        buffer[idx] = b'-';
        idx += 1;
    } else if value > 0 {
        buffer[idx] = b'+';
        idx += 1;
    } else {
        buffer[idx] = b' ';
        idx += 1;
    }

    // Value
    let abs_val = value.abs();
    if abs_val >= 100 {
        buffer[idx] = b'1';
        idx += 1;
        buffer[idx] = b'0';
        idx += 1;
        buffer[idx] = b'0';
        idx += 1;
    } else if abs_val >= 10 {
        buffer[idx] = b'0' + ((abs_val / 10) as u8);
        idx += 1;
        buffer[idx] = b'0' + ((abs_val % 10) as u8);
        idx += 1;
    } else {
        buffer[idx] = b'0' + (abs_val as u8);
        idx += 1;
    }

    buffer[idx] = b'%';
    idx += 1;

    &buffer[..idx]
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
