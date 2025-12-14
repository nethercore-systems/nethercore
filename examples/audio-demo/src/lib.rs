#![no_std]
#![no_main]

extern "C" {
    // Camera functions
    fn camera_set(eye_x: f32, eye_y: f32, eye_z: f32, center_x: f32, center_y: f32, center_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Init config
    fn set_clear_color(color: u32);

    // Input
    fn button_held(player: u32, button: u32) -> u32;
    fn button_pressed(player: u32, button: u32) -> u32;

    // Drawing
    fn draw_text(text_ptr: *const u8, text_len: u32, x: f32, y: f32, scale: f32, color: u32);
    fn draw_rect(x: f32, y: f32, width: f32, height: f32, color: u32);

    // Audio
    fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
    fn play_sound(sound: u32, volume: f32, pan: f32);
    fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);
    fn channel_set(channel: u32, volume: f32, pan: f32);
    fn channel_stop(channel: u32);
}

// Button constants (Z input layout)
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;

// Global state
static mut BEEP_SOUND: u32 = 0;
static mut PAN_POSITION: f32 = 0.0;  // -1.0 to 1.0
static mut IS_PLAYING: bool = false;
static mut AUTO_MODE: bool = true;
static mut AUTO_TIMER: f32 = 0.0;

/// Generate a simple beep sound (440Hz square wave approximation, 0.1 seconds)
fn generate_beep() -> [i16; 2205] {  // 22050 Hz × 0.1s = 2205 samples
    let mut samples = [0i16; 2205];
    let frequency = 440.0;  // A4 note
    let sample_rate = 22050.0;
    let period_samples = (sample_rate / frequency) as usize;

    for (i, sample) in samples.iter_mut().enumerate() {
        // Square wave: alternate between high and low
        let value = if (i % period_samples) < (period_samples / 2) {
            1.0
        } else {
            -1.0
        };

        // Apply envelope (fade in/out to avoid clicks)
        let envelope = if i < 100 {
            i as f32 / 100.0  // Fade in
        } else if i > 2105 {
            (2205 - i) as f32 / 100.0  // Fade out
        } else {
            1.0
        };

        *sample = (value * envelope * 32767.0 * 0.3) as i16;  // 30% volume
    }

    samples
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Set clear color
        set_clear_color(0x19172AFF); // Dark blue-purple

        // Generate and load beep sound
        let beep_samples = generate_beep();
        BEEP_SOUND = load_sound(
            beep_samples.as_ptr(),
            (beep_samples.len() * 2) as u32  // byte length
        );

        // Start in auto mode with a sound playing
        channel_play(0, BEEP_SOUND, 0.8, 0.0, 1);  // Looping beep
        IS_PLAYING = true;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Toggle auto mode with B button
        if button_pressed(0, BUTTON_B) != 0 {
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
            if button_held(0, BUTTON_LEFT) != 0 {
                PAN_POSITION -= 0.02;
                if PAN_POSITION < -1.0 {
                    PAN_POSITION = -1.0;
                }
            }
            if button_held(0, BUTTON_RIGHT) != 0 {
                PAN_POSITION += 0.02;
                if PAN_POSITION > 1.0 {
                    PAN_POSITION = 1.0;
                }
            }
        }

        // Toggle playback with A button
        if button_pressed(0, BUTTON_A) != 0 {
            if IS_PLAYING {
                channel_stop(0);
                IS_PLAYING = false;
            } else {
                channel_play(0, BEEP_SOUND, 0.8, PAN_POSITION, 1);
                IS_PLAYING = true;
            }
        }

        // Update pan in real-time
        if IS_PLAYING {
            channel_set(0, 0.8, PAN_POSITION);
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
        draw_text(title.as_ptr(), title.len() as u32, 180.0, 30.0, 2.0, 0xFFFFFFFF);

        // Instructions
        let line1: &[u8] = if AUTO_MODE {
            b"Mode: AUTO (sound pans left-right)"
        } else {
            b"Mode: MANUAL (use D-pad to pan)"
        };
        draw_text(line1.as_ptr(), line1.len() as u32, 50.0, 100.0, 1.0, 0xFFAAAAFF);

        let line2 = b"Controls:";
        draw_text(line2.as_ptr(), line2.len() as u32, 50.0, 130.0, 1.0, 0xFFFFFFFF);

        let line3 = b"  A - Play/Stop sound";
        draw_text(line3.as_ptr(), line3.len() as u32, 50.0, 150.0, 1.0, 0xCCCCCCFF);

        let line4 = b"  B - Toggle Auto/Manual mode";
        draw_text(line4.as_ptr(), line4.len() as u32, 50.0, 170.0, 1.0, 0xCCCCCCFF);

        let line5: &[u8] = if AUTO_MODE {
            b"  (Auto mode active)"
        } else {
            b"  LEFT/RIGHT - Adjust pan"
        };
        draw_text(line5.as_ptr(), line5.len() as u32, 50.0, 190.0, 1.0, 0xCCCCCCFF);

        // Visual pan indicator
        let line6 = b"Pan Position:";
        draw_text(line6.as_ptr(), line6.len() as u32, 50.0, 240.0, 1.0, 0xFFFFFFFF);

        // Draw pan slider background
        draw_rect(100.0, 270.0, 440.0, 20.0, 0x333333FF);

        // Draw center line
        draw_rect(318.0, 265.0, 4.0, 30.0, 0x666666FF);

        // Draw pan indicator
        let indicator_x = 320.0 + (PAN_POSITION * 220.0);
        let indicator_color = if IS_PLAYING {
            0x00FF00FF  // Green when playing
        } else {
            0x666666FF  // Gray when stopped
        };
        draw_rect(indicator_x - 8.0, 265.0, 16.0, 30.0, indicator_color);

        // Draw speaker labels
        let left_label = b"L";
        draw_text(left_label.as_ptr(), left_label.len() as u32, 80.0, 275.0, 1.5, 0xFFFFFFFF);

        let right_label = b"R";
        draw_text(right_label.as_ptr(), right_label.len() as u32, 550.0, 275.0, 1.5, 0xFFFFFFFF);

        // Show pan value
        let mut pan_text = [0u8; 32];
        let pan_percent = (PAN_POSITION * 100.0) as i32;
        let pan_str = format_pan(pan_percent, &mut pan_text);
        draw_text(pan_str.as_ptr(), pan_str.len() as u32, 280.0, 320.0, 1.0, 0xFFFFFFFF);

        // Status
        // Color format: 0xRRGGBBAA (R in highest byte, A in lowest)
        let status: &[u8] = if IS_PLAYING {
            b"Status: PLAYING (440Hz beep)"
        } else {
            b"Status: STOPPED"
        };
        let status_color = if IS_PLAYING { 0x00FF00FF } else { 0xFF0000FF };  // Green/Red
        draw_text(status.as_ptr(), status.len() as u32, 50.0, 360.0, 1.0, status_color);
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
