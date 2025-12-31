//! Tracker utility functions
//!
//! Helper functions for tracker playback including sampling, waveforms,
//! frequency calculations, and lookup tables.

use super::channels::TrackerChannel;
use super::{FADE_IN_SAMPLES, FADE_OUT_SAMPLES};

/// 64-point quarter-sine lookup table for vibrato/tremolo (IT-compatible resolution)
/// Values represent sin(i * π/128) * 127 for i = 0..63
/// This gives 256 effective positions when mirrored across 4 quadrants
pub const SINE_LUT_64: [i8; 64] = [
    0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48,
    50, 52, 54, 56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 89, 91, 93, 95,
    96, 98, 100, 101, 103, 104, 106, 107, 108, 110, 111, 112, 113, 114, 115,
];

/// Legacy 16-point quarter-sine for XM/FT2 compatibility (used for panning calculations)
pub const SINE_LUT: [i8; 16] = [
    0, 12, 24, 37, 48, 60, 71, 81, 90, 98, 106, 112, 118, 122, 125, 127,
];

/// Linear frequency table for period-to-frequency conversion
///
/// 768 = 12 * 16 * 4 (12 notes × 16 finetune levels × 4 for portamento precision)
/// Entry 768 is included for interpolation at the boundary.
pub const LINEAR_FREQ_TABLE: [f32; 769] = {
    let mut table = [0.0f32; 769];
    let mut i = 0;
    while i < 769 {
        // 2^(i/768) using const-compatible computation
        // We use the identity: 2^x = e^(x * ln(2))
        // For const eval, we compute this at compile time
        let x = i as f64 / 768.0;
        // 2^x where x is in [0, 1]
        // Using a high-precision polynomial approximation for const context
        // P(x) ≈ 2^x, accurate to ~10 decimal places for x in [0,1]
        let ln2 = 0.693147180559945309417232121458176568;
        let t = x * ln2;
        // e^t Taylor series (enough terms for f32 precision)
        let e_t = 1.0
            + t * (1.0
                + t * (0.5
                    + t * (0.16666666666666666
                        + t * (0.041666666666666664
                            + t * (0.008333333333333333
                                + t * (0.001388888888888889 + t * 0.0001984126984126984))))));
        table[i] = e_t as f32;
        i += 1;
    }
    table
};

/// Sample a channel with linear interpolation and anti-pop fade-in/out
pub fn sample_channel(channel: &mut TrackerChannel, data: &[i16], sample_rate: u32) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    // Handle fade-out phase (anti-pop when sample ends)
    if channel.fade_out_samples > 0 {
        let fade_ratio = channel.fade_out_samples as f32 / FADE_OUT_SAMPLES as f32;
        channel.fade_out_samples -= 1;

        // Fade from previous sample value to zero
        let sample = channel.prev_sample * fade_ratio;

        // When fade-out completes, stop the channel
        if channel.fade_out_samples == 0 {
            channel.note_on = false;
            channel.prev_sample = 0.0;
        }

        return sample;
    }

    let pos = channel.sample_pos as usize;
    let frac = (channel.sample_pos - pos as f64) as f32;

    // Get samples for interpolation
    let sample1 = if pos < data.len() {
        data[pos] as f32 / 32768.0
    } else {
        0.0
    };

    // For interpolation sample2, we need to handle the loop boundary correctly.
    // If we're at or past (loop_end - 1), we should wrap to loop_start.
    let sample2 =
        if channel.sample_loop_type != 0 && channel.sample_loop_end > channel.sample_loop_start {
            // Check if we're at the loop boundary (pos is the last sample before loop_end)
            let loop_end = channel.sample_loop_end as usize;
            if pos + 1 >= loop_end {
                // Wrap to loop start for smooth loop interpolation
                let loop_start = channel.sample_loop_start as usize;
                if loop_start < data.len() {
                    data[loop_start] as f32 / 32768.0
                } else {
                    sample1
                }
            } else if pos + 1 < data.len() {
                data[pos + 1] as f32 / 32768.0
            } else {
                sample1
            }
        } else if pos + 1 < data.len() {
            data[pos + 1] as f32 / 32768.0
        } else {
            sample1
        };

    let mut sample = sample1 + (sample2 - sample1) * frac;

    // Handle fade-in phase (crossfade from previous sample when new note triggers)
    if channel.fade_in_samples > 0 {
        let fade_ratio = 1.0 - (channel.fade_in_samples as f32 / FADE_IN_SAMPLES as f32);
        channel.fade_in_samples -= 1;

        // Crossfade: blend from previous sample value to new sample
        sample = channel.prev_sample * (1.0 - fade_ratio) + sample * fade_ratio;
    }

    // Store current sample for future crossfade (only update after fade-in complete)
    if channel.fade_in_samples == 0 {
        channel.prev_sample = sample;
    }

    // Calculate playback rate from period
    // XM frequency tells us the target playback frequency
    // Divide by output sample rate to get sample increment per output sample
    let freq = period_to_frequency(channel.period);
    let rate = freq / sample_rate as f32;

    // Advance sample position
    channel.sample_pos += rate as f64 * channel.sample_direction as f64;

    // Handle loop
    if channel.sample_loop_type != 0 && channel.sample_loop_end > channel.sample_loop_start {
        if channel.sample_direction > 0 && channel.sample_pos >= channel.sample_loop_end as f64 {
            if channel.sample_loop_type == 2 {
                // Ping-pong
                channel.sample_direction = -1;
                channel.sample_pos = channel.sample_loop_end as f64
                    - (channel.sample_pos - channel.sample_loop_end as f64);
            } else {
                // Forward loop
                channel.sample_pos = channel.sample_loop_start as f64
                    + (channel.sample_pos - channel.sample_loop_end as f64);
            }
        } else if channel.sample_direction < 0
            && channel.sample_pos < channel.sample_loop_start as f64
        {
            // Ping-pong reverse hit
            channel.sample_direction = 1;
            channel.sample_pos = channel.sample_loop_start as f64
                + (channel.sample_loop_start as f64 - channel.sample_pos);
        }
    } else if channel.sample_pos >= data.len() as f64 {
        // No loop - start fade-out instead of abrupt stop (anti-pop)
        channel.fade_out_samples = FADE_OUT_SAMPLES;
    }

    sample
}

/// Fast panning gains using the existing SINE_LUT with interpolation
///
/// Uses the 16-point sine LUT already defined for vibrato/tremolo.
/// cos(x) = sin(π/2 - x), so we read the LUT in reverse for left channel.
#[inline]
pub fn fast_pan_gains(pan: f32) -> (f32, f32) {
    // Map pan [-1, 1] to [0, 15] range for LUT indexing
    let pos = (pan + 1.0) * 7.5;
    let idx = (pos as usize).min(14);
    let frac = pos - idx as f32;

    // Linear interpolation between LUT points
    // Right channel uses sin (direct LUT), left uses cos (reversed LUT)
    let sin_val = SINE_LUT[idx] as f32 * (1.0 - frac) + SINE_LUT[idx + 1] as f32 * frac;
    let cos_val =
        SINE_LUT[15 - idx] as f32 * (1.0 - frac) + SINE_LUT[14 - idx.min(14)] as f32 * frac;

    // Scale from [0, 127] to [0, 1]
    (cos_val / 127.0, sin_val / 127.0)
}

/// Apply panning to a sample using fast LUT lookup
#[inline]
pub fn apply_channel_pan(sample: f32, pan: f32) -> (f32, f32) {
    let (left_gain, right_gain) = fast_pan_gains(pan);
    (sample * left_gain, sample * right_gain)
}

/// Calculate samples per tick from BPM
///
/// XM timing: samples_per_tick = sample_rate * 2.5 / bpm
pub fn samples_per_tick(bpm: u16, sample_rate: u32) -> u32 {
    if bpm == 0 {
        return sample_rate; // Fallback to 1 tick per second
    }
    (sample_rate * 5 / 2) / bpm as u32
}

/// Get waveform value for vibrato/tremolo
///
/// Uses IT-compatible 64-point quarter-sine table (256 effective positions)
///
/// Waveform types:
/// - 0: Sine (IT LUT with quadrant mirroring)
/// - 1: Ramp down (sawtooth)
/// - 2: Square
/// - 3: Random (deterministic pseudo-random)
pub fn get_waveform_value(waveform: u8, position: u8) -> f32 {
    let pos = position; // Full 256 positions for IT compatibility

    match waveform & 0x03 {
        0 => {
            // IT 256-point sine using 64-point quarter table with mirroring
            // Quarter 0 (0-63): ascending from 0 to peak
            // Quarter 1 (64-127): descending from peak to 0
            // Quarter 2 (128-191): ascending from 0 to -peak
            // Quarter 3 (192-255): descending from -peak to 0
            let quarter = pos >> 6; // 0-3
            let idx = (pos & 0x3F) as usize; // 0-63
            let val = match quarter {
                0 => SINE_LUT_64[idx],       // 0-63: ascending
                1 => SINE_LUT_64[63 - idx],  // 64-127: descending
                2 => -SINE_LUT_64[idx],      // 128-191: negative ascending
                _ => -SINE_LUT_64[63 - idx], // 192-255: negative descending
            };
            val as f32 / 115.0 // Normalize to roughly -1.0 to 1.0
        }
        1 => {
            // Ramp down (sawtooth)
            // Position 0 = +1.0, position 128 = -1.0, position 255 = ~+1.0
            let ramp = 128i16 - (pos as i16);
            (ramp as f32) / 128.0
        }
        2 => {
            // Square wave: 1.0 for first half, -1.0 for second
            if pos < 128 { 1.0 } else { -1.0 }
        }
        _ => {
            // "Random" - deterministic pseudo-random using position as seed
            let x = position.wrapping_mul(0x9E) ^ 0x5C;
            (x as f32 / 127.5) - 1.0
        }
    }
}

/// Convert note number to period (XM linear frequency)
///
/// XM period formula: period = 10*12*16*4 - note*16*4 - finetune/2
/// Note 1 = C-0, note 97 = B-7 (XM range)
pub fn note_to_period(note: u8, finetune: i8) -> f32 {
    if note == 0 || note > 119 {
        return 0.0;
    }
    // XM linear period: 10*12*16*4 - (note-1)*16*4 - finetune/2
    // We use note-1 because XM notes are 1-indexed (1 = C-0)
    let period =
        10.0 * 12.0 * 16.0 * 4.0 - ((note - 1) as f32 * 16.0 * 4.0) - (finetune as f32 / 2.0);
    period.max(1.0)
}

/// Apply IT linear slide to period
///
/// IT uses a different slide formula than XM. Each slide unit represents
/// 4 times the XM fine slide unit.
pub fn apply_it_linear_slide(period: f32, slide: i16) -> f32 {
    // IT slide: each unit is 4 times the XM fine slide
    // Slide up = decrease period (higher pitch)
    // Slide down = increase period (lower pitch)
    let new_period = period - (slide as f32 * 4.0);
    new_period.max(1.0)
}

/// Convert period to frequency (Hz) using lookup table
///
/// Modified XM frequency formula for 22050 Hz samples:
/// Frequency = 22050 * 2^((4608 - Period) / 768)
///
/// The original XM formula used 8363 Hz (Amiga C-4 rate), but since all our
/// samples are resampled to 22050 Hz, we use that as the base frequency.
/// This ensures samples play at their natural pitch at C-4.
///
/// This uses a 768-entry lookup table for the fractional part of the exponent,
/// making it O(1) and fast even in debug builds (no powf() calls).
#[inline]
pub fn period_to_frequency(period: f32) -> f32 {
    if period <= 0.0 {
        return 0.0;
    }

    // 4608 = 6 * 12 * 16 * 4 (middle C-4 reference point)
    let diff = 4608.0 - period;

    // Split into octave (integer) and fractional parts
    // diff / 768 = number of octaves from C-4
    let octaves = (diff / 768.0).floor();
    let frac = diff - (octaves * 768.0);

    // Table lookup with linear interpolation for fractional indices
    let idx = frac as usize;
    let t = frac - idx as f32;

    // Clamp index to valid range (handles edge cases)
    let idx = idx.min(767);
    let freq_frac = LINEAR_FREQ_TABLE[idx] * (1.0 - t) + LINEAR_FREQ_TABLE[idx + 1] * t;

    // Apply octave scaling: multiply by 2^octaves
    // For positive octaves: multiply by 2^n
    // For negative octaves: divide by 2^|n|
    let octave_scale = if octaves >= 0.0 {
        (1u32 << (octaves as u32).min(31)) as f32
    } else {
        1.0 / (1u32 << ((-octaves) as u32).min(31)) as f32
    };

    // Base frequency is 22050 Hz (our standardized sample rate)
    // This replaces the original 8363 Hz (Amiga C-4 rate)
    22050.0 * freq_frac * octave_scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_period() {
        // XM linear period: 7680 - (note-1)*64
        // C-4 (note 49) should give period 7680 - 48*64 = 4608
        let period = note_to_period(49, 0);
        assert!(
            (period - 4608.0).abs() < 1.0,
            "Expected ~4608, got {}",
            period
        );

        // Higher notes = lower period
        let higher = note_to_period(61, 0); // C-5
        assert!(higher < period);

        // Finetune shifts period
        let finetuned = note_to_period(49, 64);
        assert!(finetuned < period);
    }

    #[test]
    fn test_period_to_frequency() {
        // XM frequency formula: 22050 * 2^((4608 - period) / 768)
        // C-4 (period 4608) produces 22050 Hz (sample plays at natural speed)
        let period = note_to_period(49, 0);
        let freq = period_to_frequency(period);
        assert!(
            (freq - 22050.0).abs() < 1.0,
            "Expected ~22050 Hz, got {}",
            freq
        );

        // C-5 (one octave up) should be double the frequency
        let period_c5 = note_to_period(61, 0);
        let freq_c5 = period_to_frequency(period_c5);
        assert!(
            (freq_c5 / freq - 2.0).abs() < 0.01,
            "C-5 should be ~2x C-4 frequency"
        );
    }

    #[test]
    fn test_waveform_sine() {
        // Position 0 should be 0
        let val = get_waveform_value(0, 0);
        assert!(val.abs() < 0.1);

        // Position 64 should be peak (~1.0)
        let peak = get_waveform_value(0, 64);
        assert!(peak > 0.9);

        // Position 128 should be 0
        let zero = get_waveform_value(0, 128);
        assert!(zero.abs() < 0.1);

        // Position 192 should be negative peak
        let neg_peak = get_waveform_value(0, 192);
        assert!(neg_peak < -0.9);
    }

    #[test]
    fn test_samples_per_tick() {
        // At 125 BPM and 44100 Hz: 44100 * 2.5 / 125 = 882 samples
        let spt = samples_per_tick(125, 44100);
        assert_eq!(spt, 882);

        // At 125 BPM and 22050 Hz: 22050 * 2.5 / 125 = 441 samples
        let spt = samples_per_tick(125, 22050);
        assert_eq!(spt, 441);
    }
}
