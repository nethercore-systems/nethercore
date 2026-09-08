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
        let ln2 = std::f64::consts::LN_2;
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

// Original one-pole fit to fresh 44.1/48/96kHz reference captures.
// ponytail: bounded 2048-frame tail; exact output-stage smoothing remains unclaimed.
const XM_STOP_TAIL_SAMPLES: u16 = 2_048;
const XM_STOP_TAIL_DECAY: f32 = 255.0 / 256.0;

/// Sample one mono or stereo frame with one shared playback-state advance.
pub fn sample_channels(channel: &mut TrackerChannel, data: &[i16], sample_rate: u32) -> (f32, f32) {
    if data.is_empty() {
        return (0.0, 0.0);
    }
    if channel.xm_loop_stopped {
        if channel.xm_stop_tail_remaining == 0 {
            return (0.0, 0.0);
        }
        let samples = (
            channel.prev_sample * XM_STOP_TAIL_DECAY,
            channel.prev_sample_right * XM_STOP_TAIL_DECAY,
        );
        channel.prev_sample = samples.0;
        channel.prev_sample_right = samples.1;
        channel.xm_stop_tail_remaining -= 1;
        if channel.xm_stop_tail_remaining == 0 {
            channel.prev_sample = 0.0;
            channel.prev_sample_right = 0.0;
        }
        return samples;
    }

    let stride = if channel.sample_is_stereo { 2 } else { 1 };
    let frame_count = data.len() / stride;
    let sample_at = |frame: usize, lane: usize| {
        data.get(frame * stride + lane.min(stride - 1))
            .map_or(0.0, |&sample| sample as f32 / 32768.0)
    };

    // IT sustain loops are active until key-off, then normal loops resume.
    let (loop_start, loop_end, loop_type) = if !channel.key_off
        && !channel.sample_sustain_released
        && channel.sample_sustain_loop_type != 0
        && channel.sample_sustain_loop_end > channel.sample_sustain_loop_start
    {
        (
            channel.sample_sustain_loop_start,
            channel.sample_sustain_loop_end,
            channel.sample_sustain_loop_type,
        )
    } else {
        (
            channel.sample_loop_start,
            channel.sample_loop_end,
            channel.sample_loop_type,
        )
    };

    let pos = if loop_type == 2 && loop_end > loop_start {
        (channel.sample_pos as usize).min(loop_end as usize - 1)
    } else {
        channel.sample_pos as usize
    };
    if channel.fade_out_samples > 0 {
        let fade_ratio = channel.fade_out_samples as f32 / FADE_OUT_SAMPLES as f32;
        channel.fade_out_samples -= 1;

        // Fade from previous sample value to zero
        let samples = (
            channel.prev_sample * fade_ratio,
            channel.prev_sample_right * fade_ratio,
        );

        // When fade-out completes, stop the channel
        if channel.fade_out_samples == 0 {
            channel.note_on = false;
            channel.prev_sample = 0.0;
            channel.prev_sample_right = 0.0;
        }

        return samples;
    }

    let frac = (channel.sample_pos - pos as f64) as f32;

    // Get samples for interpolation
    let sample1 = (sample_at(pos, 0), sample_at(pos, 1));

    // For interpolation sample2, we need to handle the loop boundary correctly.
    // If we're at or past (loop_end - 1), we should wrap to loop_start.
    let next_pos = if loop_type != 0 && loop_end > loop_start {
        // Check if we're at the loop boundary (pos is the last sample before loop_end)
        let loop_end = loop_end as usize;
        if pos + 1 >= loop_end {
            // Wrap to loop start for smooth loop interpolation
            let loop_start = loop_start as usize;
            if loop_type == 2 {
                pos // Reflect at the endpoint; never interpolate to the opposite end.
            } else if loop_start < frame_count {
                loop_start
            } else {
                pos
            }
        } else if pos + 1 < frame_count {
            pos + 1
        } else {
            pos
        }
    } else if pos + 1 < frame_count {
        pos + 1
    } else {
        pos
    };
    let sample2 = (sample_at(next_pos, 0), sample_at(next_pos, 1));

    let mut samples = (
        sample1.0 + (sample2.0 - sample1.0) * frac,
        sample1.1 + (sample2.1 - sample1.1) * frac,
    );

    // Measured stopped-sample restart attack; ordinary notes keep their existing crossfade.
    let restart_attack = channel.xm_restart_attack > 0;
    if restart_attack {
        let duration = if channel.xm_short_restart_attack {
            ((u64::from(sample_rate) * 16 + 22050) / 44100) as u32
        } else {
            sample_rate.div_ceil(200)
        }
        .max(1);
        let gain = channel.xm_restart_attack.min(duration) as f32 / duration as f32;
        samples.0 *= gain;
        samples.1 *= gain;
        channel.xm_restart_attack = if channel.xm_restart_attack >= duration {
            0
        } else {
            channel.xm_restart_attack + 1
        };
        channel.fade_in_samples = 0;
    }
    // Handle fade-in phase (crossfade from previous sample when new note triggers)
    if channel.fade_in_samples > 0 {
        let fade_ratio = 1.0 - (channel.fade_in_samples as f32 / FADE_IN_SAMPLES as f32);
        channel.fade_in_samples -= 1;

        // Crossfade: blend from previous sample value to new sample
        samples.0 = channel.prev_sample * (1.0 - fade_ratio) + samples.0 * fade_ratio;
        samples.1 = channel.prev_sample_right * (1.0 - fade_ratio) + samples.1 * fade_ratio;
    }

    // Store current sample for future crossfade (only update after fade-in complete)
    if channel.fade_in_samples == 0 {
        channel.prev_sample = samples.0;
        channel.prev_sample_right = samples.1;
    }

    // Calculate playback rate from period
    // XM frequency tells us the target playback frequency
    // Divide by output sample rate to get sample increment per output sample
    // IT pitch envelope units are half-semitones. OpenMPT applies envelope*8
    // through its 1/192-octave slide table, capped at index 255. Convert to
    // our linear period units without modifying slide/portamento state.
    let pitch_offset = channel.pitch_envelope_value.clamp(-31.875, 31.875) * 32.0;
    let period = if channel.glissando && channel.tone_porta_active {
        let fine = (f32::from(channel.finetune) + f32::from(channel.xm_finetune_delta)) / 2.0;
        if channel.xm_amiga_slides {
            let lower = ((channel.period + fine) / 64.0).ceil() * 64.0 - fine;
            let mut boundary = xm_native_period(f64::from(lower - 32.0), channel.xm_source_tuning);
            // The independently observed positive-finetune octave transition
            // saturates at the last measured subdivision of the lower octave.
            if channel.xm_source_finetune >= 64 {
                let source_note = 4608.0
                    + f64::from(channel.xm_source_tuning - i16::from(channel.xm_source_finetune))
                        / 2.0
                    - f64::from(lower);
                let octave = (source_note / 768.0).floor();
                let edge = f64::from(XM_MEASURED_PERIODS[95]) * 2.0_f64.powf(-octave - 1.0);
                boundary = boundary.max(edge);
            }
            if xm_native_period(f64::from(channel.period), channel.xm_source_tuning).round()
                >= boundary
            {
                lower
            } else {
                lower - 64.0
            }
        } else {
            ((channel.period + fine) / 64.0).round() * 64.0 - fine
        }
    } else {
        channel.period
    };
    let freq = period_to_frequency(period - pitch_offset);
    let rate = freq / sample_rate as f32;

    // Advance sample position
    channel.sample_pos += rate as f64 * channel.sample_direction as f64;

    // Keep stop timing in unrounded source space. Native PCM still wraps using
    // its rounded loop points below; one source-loop subtraction is intentional.
    if loop_type == 1 && channel.xm_forward_loop_limit > 0.0 {
        channel.xm_source_position += f64::from(rate);
        let source_end = channel.xm_forward_loop_start + channel.xm_forward_loop_limit;
        if channel.xm_source_position >= source_end {
            channel.xm_source_position -= channel.xm_forward_loop_limit;
            if channel.xm_source_position > source_end {
                channel.xm_loop_stopped = true;
                // Fade-in freezes prev_sample as its crossfade anchor. Once
                // stopped, the residual must decay this frame's actual output.
                channel.prev_sample = samples.0;
                channel.prev_sample_right = samples.1;
                channel.fade_in_samples = 0;
                channel.xm_stop_tail_remaining = XM_STOP_TAIL_SAMPLES;
            }
        }
    }

    // Handle loop
    if loop_type != 0 && loop_end > loop_start {
        if channel.sample_direction > 0 && channel.sample_pos >= loop_end as f64 {
            if loop_type == 2 {
                // Ping-pong
                channel.sample_direction = -1;
                channel.sample_pos = loop_end as f64 - (channel.sample_pos - loop_end as f64);
            } else {
                // Forward loop: preserve overshoot across any number of complete cycles.
                channel.sample_pos = loop_start as f64
                    + (channel.sample_pos - loop_end as f64)
                        .rem_euclid((loop_end - loop_start) as f64);
            }
        } else if channel.sample_direction < 0 && channel.sample_pos < loop_start as f64 {
            // Ping-pong reverse hit
            channel.sample_direction = 1;
            channel.sample_pos = loop_start as f64 + (loop_start as f64 - channel.sample_pos);
        }
        if loop_type == 2
            && (channel.sample_pos < loop_start as f64 || channel.sample_pos > loop_end as f64)
        {
            // A large increment can cross both ends; fold the remaining complete cycles.
            let length = (loop_end - loop_start) as f64;
            let phase = (channel.sample_pos - loop_start as f64).rem_euclid(2.0 * length);
            if phase >= length {
                channel.sample_pos = loop_end as f64 - (phase - length);
                channel.sample_direction = -channel.sample_direction;
            } else {
                channel.sample_pos = loop_start as f64 + phase;
            }
        }
    } else if channel.sample_pos >= frame_count as f64 {
        // No loop - start fade-out instead of abrupt stop (anti-pop)
        channel.fade_out_samples = FADE_OUT_SAMPLES;
    }

    samples
}

/// Sample a mono channel. Existing callers retain identical advancement.
pub fn sample_channel(channel: &mut TrackerChannel, data: &[i16], sample_rate: u32) -> f32 {
    sample_channels(channel, data, sample_rate).0
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

/// Default FT2 finetune measured with original boundary controls in both modes.
/// Legacy playback retains its separately measured tuning semantics.
pub(super) fn xm_sample_finetune(value: i8, legacy: bool) -> i8 {
    if legacy { value } else { (value >> 3) << 3 }
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

// Independently measured from our original sine-tone XM, not reference source.
// tools/tracker-debug/xm_period_measure.py; xm-independent-period-repeat-v1/report.json.
// C-3 octave, eight observations per semitone. Inferred integer periods differ
// from the two-window frequency measurements by less than 0.15 period units.
const XM_MEASURED_PERIODS: [u16; 96] = [
    3424, 3400, 3376, 3352, 3328, 3304, 3280, 3256, 3232, 3208, 3184, 3164, 3140, 3116, 3096, 3072,
    3048, 3028, 3008, 2984, 2964, 2944, 2920, 2900, 2880, 2860, 2836, 2816, 2796, 2776, 2756, 2736,
    2712, 2700, 2680, 2660, 2640, 2620, 2604, 2584, 2560, 2544, 2528, 2512, 2492, 2476, 2456, 2440,
    2416, 2404, 2388, 2368, 2352, 2336, 2320, 2300, 2280, 2268, 2252, 2236, 2220, 2204, 2188, 2172,
    2152, 2140, 2128, 2112, 2096, 2080, 2064, 2052, 2032, 2020, 2008, 1992, 1976, 1964, 1948, 1936,
    1920, 1908, 1896, 1880, 1868, 1852, 1840, 1828, 1812, 1800, 1788, 1772, 1760, 1748, 1736, 1724,
];

// Interpolate in the source period coordinate. Runtime pitch stays logarithmic
// because tuning is baked into PCM; source slide increments are not logarithmic.
fn xm_native_period(period: f64, tuning: i16) -> f64 {
    let step = (4608.0 + f64::from(tuning) / 2.0 - period) / 8.0;
    let octave = (step / 96.0).floor();
    let within = step - octave * 96.0;
    let index = within.floor() as usize;
    let left = f64::from(XM_MEASURED_PERIODS[index]);
    let right = if index == 95 {
        f64::from(XM_MEASURED_PERIODS[0]) / 2.0
    } else {
        f64::from(XM_MEASURED_PERIODS[index + 1])
    };
    (left + (right - left) * (within - index as f64)) * 2.0_f64.powf(-octave - 1.0)
}

fn xm_runtime_period(native: f64, tuning: i16) -> f32 {
    let octave = (1712.0 / native).log2().floor();
    let normalized = native * 2.0_f64.powf(octave + 1.0);
    let mut index = 0;
    while index < 95 && normalized < f64::from(XM_MEASURED_PERIODS[index + 1]) {
        index += 1;
    }
    let left = f64::from(XM_MEASURED_PERIODS[index]);
    let right = if index == 95 {
        f64::from(XM_MEASURED_PERIODS[0]) / 2.0
    } else {
        f64::from(XM_MEASURED_PERIODS[index + 1])
    };
    let step = octave * 96.0 + index as f64 + (left - normalized) / (left - right);
    (4608.0 + f64::from(tuning) / 2.0 - step * 8.0) as f32
}

/// Apply a signed XM period increment without confusing Amiga and linear units.
/// C-4 is runtime period4608 and source period1712; measured source spacing is nonuniform.
pub(super) fn slide_xm_period(period: f32, delta: f32, amiga: bool, tuning: i16) -> f32 {
    if !amiga {
        return (period + delta).max(1.0);
    }
    // The source period counter advances in integer units. Reconstruct its
    // integer before adding an effect so f32 coordinate round trips cannot
    // accumulate across a semitone threshold.
    let native = xm_native_period(f64::from(period), tuning).round();
    // Preserve the playable runtime-domain floor, as on the linear path.
    xm_runtime_period((native + f64::from(delta)).max(1.0), tuning).max(1.0)
}

// Original mathematical sine construction, not a reference-player lookup table.
const IT_SINE: [i16; 256] = {
    let mut values = [0; 256];
    let mut i = 0;
    while i < 256 {
        let phase = i % 128;
        let x =
            (if phase <= 64 { phase } else { 128 - phase }) as f64 * std::f64::consts::PI / 128.0;
        let mut term = x;
        let mut sum = x;
        let mut n = 1;
        while n < 8 {
            term *= -x * x / ((2 * n) * (2 * n + 1)) as f64;
            sum += term;
            n += 1;
        }
        values[i] = (sum * 64.0 + 0.5) as i16 * if i < 128 { 1 } else { -1 };
        i += 1;
    }
    values
};

pub(super) fn it_waveform(waveform: u8, position: u8) -> i16 {
    if waveform & 3 == 0 {
        IT_SINE[position as usize]
    } else {
        (get_waveform_value(waveform, position) * 64.0) as i16
    }
}

// Original mathematical sine construction, not a reference-player lookup table.
const XM_SINE: [u8; 32] = {
    let mut values = [0; 32];
    let mut i = 0;
    while i < 32 {
        let x = (if i <= 16 { i } else { 32 - i }) as f64 * std::f64::consts::PI / 32.0;
        let mut term = x;
        let mut sum = x;
        let mut n = 1;
        while n < 8 {
            term *= -x * x / ((2 * n) * (2 * n + 1)) as f64;
            sum += term;
            n += 1;
        }
        values[i] = (sum * 255.0 + 0.5) as u8;
        i += 1;
    }
    values
};

// Independently measured from original depth15 XM playback, not reference source.
// Provenance: xm-random-independent-measure-v1/inferred-waveform.json; <=1 unit inference uncertainty.
const XM_MEASURED_RANDOM: [i16; 64] = [
    196, -254, -86, 176, 204, 82, -129, -188, 250, 40, -142, -172, -140, -65, -33, -193, 33, 144,
    214, -10, 232, -138, -124, -80, 20, -122, 129, 218, -36, -76, -26, -152, -46, 176, 42, -188,
    16, 212, 42, -225, 12, 218, 40, -176, -60, 18, -254, 236, 84, -68, 178, -8, -102, -144, 42,
    -58, 225, 246, 168, -202, -184, 196, -108, -190,
];

// Measured original 7xy sine probes: depth4 oscillates by 0.25 around base.
pub(super) fn xm_tremolo_delta(depth: u8, waveform: u8, position: u8, vibrato_position: u8) -> f32 {
    let wave = if waveform & 3 == 0 {
        f32::from(XM_SINE[((position >> 2) & 31) as usize]) / 255.0
            * if position < 128 { 1.0 } else { -1.0 }
    } else if waveform & 3 == 1 {
        // Independent ramp controls: retained negative-half vibrato reverses its slope.
        let ramp = f32::from(position % 128) / 128.0;
        (if vibrato_position < 128 {
            ramp
        } else {
            1.0 - ramp
        }) * if position < 128 { 1.0 } else { -1.0 }
    } else if waveform & 3 == 3 {
        f32::from(XM_MEASURED_RANDOM[(position >> 2) as usize]) / 255.0
    } else {
        get_waveform_value(waveform, position)
    };
    wave * f32::from(depth) / 16.0
}

pub(super) fn xm_vibrato_period(
    base: f32,
    depth: u8,
    waveform: u8,
    position: u8,
    amiga: bool,
    tuning: i16,
) -> f32 {
    let wave = if waveform & 3 == 0 {
        i16::from(XM_SINE[((position >> 2) & 31) as usize]) * if position < 128 { 1 } else { -1 }
    } else if waveform & 3 == 1 {
        // Authored E41 measurements start at zero and ramp within each half-cycle.
        if position < 128 {
            i16::from(position) * 2
        } else {
            i16::from(position) * 2 - 511
        }
    } else if waveform & 3 == 3 {
        XM_MEASURED_RANDOM[(position >> 2) as usize]
    } else {
        (get_waveform_value(waveform, position) * 255.0) as i16
    };
    let magnitude = (wave.unsigned_abs() * u16::from(depth)) >> 5;
    slide_xm_period(
        base,
        if wave < 0 {
            -(magnitude as f32)
        } else {
            magnitude as f32
        },
        amiga,
        tuning,
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)] // Incremental sample controls.
    use super::*;

    #[test]
    fn ping_pong_interpolation_holds_endpoint_without_tail_or_wrap() {
        for stereo in [false, true] {
            let data = if stereo {
                vec![1000, -1000, 2000, -2000, 3000, -3000, 31000, -31000]
            } else {
                vec![1000, 2000, 3000, 31000]
            };
            for position in [2.5, 3.0] {
                let mut channel = TrackerChannel::default();
                channel.sample_is_stereo = stereo;
                channel.sample_loop_start = 1;
                channel.sample_loop_end = 3;
                channel.sample_loop_type = 2;
                channel.sample_pos = position;
                channel.sample_direction = 1;
                channel.period = 4608.0;
                let (left, right) = sample_channels(&mut channel, &data, 44100);
                assert_eq!(left, 3000.0 / 32768.0);
                assert_eq!(right, if stereo { -left } else { left });
            }
        }
    }

    #[test]
    fn ping_pong_multiple_crossings_preserve_position_and_direction() {
        for stereo in [false, true] {
            for direction in [-1, 1] {
                let mut channel = TrackerChannel {
                    period: 2304.0, // Four frames per output sample: one full loop cycle.
                    sample_pos: 2.5,
                    sample_direction: direction,
                    sample_is_stereo: stereo,
                    sample_loop_start: 1,
                    sample_loop_end: 3,
                    sample_loop_type: 2,
                    ..Default::default()
                };
                let data = if stereo {
                    vec![0, 0, 2000, -2000, 3000, -3000, 0, 0]
                } else {
                    vec![0, 2000, 3000, 0]
                };
                for _ in 0..8 {
                    sample_channels(&mut channel, &data, 44100);
                    assert_eq!(channel.sample_pos, 2.5);
                    assert_eq!(channel.sample_direction, direction);
                }
            }
        }
    }

    #[test]
    fn forward_multiple_crossings_stay_inside_loop() {
        for stereo in [false, true] {
            let mut channel = TrackerChannel {
                period: 2304.0,
                sample_pos: 2.5,
                sample_direction: 1,
                sample_is_stereo: stereo,
                sample_loop_start: 1,
                sample_loop_end: 3,
                sample_loop_type: 1,
                ..Default::default()
            };
            let data = if stereo {
                vec![0, 0, 2000, -2000, 2000, -2000, 0, 0]
            } else {
                vec![0, 2000, 2000, 0]
            };
            for _ in 0..8 {
                let (left, right) = sample_channels(&mut channel, &data, 44100);
                assert_eq!(channel.sample_pos, 2.5);
                assert_eq!(left, 2000.0 / 32768.0);
                assert_eq!(right, if stereo { -left } else { left });
            }
        }
    }

    #[test]
    fn xm_forward_source_limit_stops_only_opted_in_forward_voices() {
        for (loop_type, limit, stops) in [
            (1, 2.5, true),
            (1, 4.0, false),
            (1, 0.0, false),
            (2, 2.5, false),
        ] {
            let mut channel = TrackerChannel {
                note_on: true,
                period: 4608.0,
                sample_loop_type: loop_type,
                sample_loop_start: 1,
                sample_loop_end: 8,
                xm_forward_loop_limit: limit,
                ..Default::default()
            };
            let mut restored = channel.clone();
            assert_eq!(
                sample_channels(&mut channel, &[2000; 16], 44100),
                sample_channels(&mut restored, &[2000; 16], 44100)
            );
            assert_eq!(channel.sample_pos, restored.sample_pos);
            assert_eq!(channel.note_on, restored.note_on);
            assert_eq!(
                channel.xm_forward_loop_limit,
                restored.xm_forward_loop_limit
            );
            assert_eq!(channel.xm_loop_stopped, restored.xm_loop_stopped);
            if stops {
                channel.period = 2304.0;
                assert!(sample_channels(&mut channel, &[2000; 16], 44100).0 > 0.0);
                assert!(!channel.xm_loop_stopped);
                assert!(sample_channels(&mut channel, &[2000; 16], 44100).0 > 0.0);
                assert!(channel.xm_stop_tail_remaining > 0);

                channel.retrigger_sample(true);
                assert!(!channel.xm_loop_stopped);
                assert!(sample_channels(&mut channel, &[2000; 16], 44100).0 > 0.0);
            }
            assert!(channel.note_on);
            assert!(!channel.xm_loop_stopped);
        }
    }

    #[test]
    fn xm_forward_stop_tail_decays_and_retrigger_cancels_it() {
        let mut channel = TrackerChannel {
            note_on: true,
            period: 4608.0,
            sample_loop_type: 1,
            sample_loop_start: 0,
            sample_loop_end: 8,
            xm_forward_loop_limit: 0.25,
            ..Default::default()
        };
        let _first = sample_channels(&mut channel, &[2000; 16], 44100).0;
        assert!(!channel.xm_loop_stopped);
        let stop_sample = sample_channels(&mut channel, &[2000; 16], 44100).0;
        assert!(channel.xm_loop_stopped);
        assert_eq!(channel.xm_stop_tail_remaining, XM_STOP_TAIL_SAMPLES);
        let second = sample_channels(&mut channel, &[2000; 16], 44100).0;
        assert!((second - stop_sample * XM_STOP_TAIL_DECAY).abs() < f32::EPSILON);
        for _ in 1..XM_STOP_TAIL_SAMPLES {
            sample_channels(&mut channel, &[2000; 16], 44100);
        }
        assert_eq!(channel.fade_out_samples, 0);
        assert_eq!(channel.xm_stop_tail_remaining, 0);
        assert_eq!(
            sample_channels(&mut channel, &[2000; 16], 44100),
            (0.0, 0.0)
        );
        assert!(channel.note_on);
        channel.retrigger_sample(true);
        assert!(!channel.xm_loop_stopped);
        assert_eq!(channel.fade_out_samples, 0);
        assert_eq!(channel.xm_stop_tail_remaining, 0);
        assert!(sample_channels(&mut channel, &[2000; 16], 44100).0 > 0.0);
    }

    #[test]
    fn xm_source_phase_escape_matches_original_tempo_controls() {
        for (tempo, expected) in [(120, 21), (125, 28), (127, 26), (137, 30)] {
            let tick = 110250 / tempo;
            let base = note_to_period(77, 0);
            let mut channel = TrackerChannel {
                note_on: true,
                period: base,
                sample_loop_type: 1,
                sample_loop_start: 0,
                sample_loop_end: 3,
                xm_forward_loop_start: 4.0 * 22050.0 / 8363.0,
                xm_forward_loop_limit: 22050.0 / 8363.0,
                ..Default::default()
            };
            for _ in 0..6 * tick {
                sample_channels(&mut channel, &[8192; 16], 44100);
            }
            for delta in [0, 16, 32, 48] {
                channel.period = base - delta as f32;
                for _ in 0..tick {
                    sample_channels(&mut channel, &[8192; 16], 44100);
                }
            }
            assert!(!channel.xm_loop_stopped);
            channel.period = base - 64.0;
            let mut elapsed = 0;
            while !channel.xm_loop_stopped && elapsed < 100 {
                sample_channels(&mut channel, &[8192; 16], 44100);
                elapsed += 1;
            }
            assert_eq!(elapsed, expected, "tempo={tempo}");
        }
    }

    #[test]
    fn xm_initial_stop_tail_retains_last_attack_output() {
        for rate in [44100, 48000, 96000] {
            for stereo in [false, true] {
                for anchor in [0.0, -0.125] {
                    let mut channel = TrackerChannel {
                        note_on: true,
                        period: 2048.0,
                        sample_is_stereo: stereo,
                        sample_loop_type: 1,
                        sample_loop_end: 16,
                        sample_direction: 1,
                        xm_forward_loop_start: 8.0,
                        xm_forward_loop_limit: 0.25,
                        fade_in_samples: FADE_IN_SAMPLES,
                        prev_sample: anchor,
                        prev_sample_right: anchor,
                        ..Default::default()
                    };
                    let data: Vec<_> = (0..32)
                        .flat_map(|_| {
                            if stereo {
                                vec![8192, -4096]
                            } else {
                                vec![8192]
                            }
                        })
                        .collect();
                    let mut last = (0.0, 0.0);
                    for _ in 0..20 {
                        last = sample_channels(&mut channel, &data, rate);
                        if channel.xm_loop_stopped {
                            break;
                        }
                    }
                    assert!(channel.xm_loop_stopped);
                    assert_ne!(last.0, anchor, "stop must occur during audible attack");
                    let tail = sample_channels(&mut channel, &data, rate);
                    assert_eq!(
                        tail,
                        (last.0 * XM_STOP_TAIL_DECAY, last.1 * XM_STOP_TAIL_DECAY),
                        "rate={rate} stereo={stereo} anchor={anchor}"
                    );
                    assert_eq!(channel.fade_in_samples, 0);
                }
            }
        }
    }

    #[test]
    fn xm_stopped_restart_attack_is_rate_scaled_and_cloned() {
        for (rate, short, duration) in [
            (44100u32, false, 221),
            (48000, false, 240),
            (96000, false, 480),
            (44100, true, 16),
            (48000, true, 17),
            (96000, true, 35),
        ] {
            let mut channel = TrackerChannel {
                note_on: true,
                period: 4608.0,
                xm_short_restart_attack: short,
                xm_loop_stopped: true,
                sample_loop_type: 1,
                sample_loop_end: 16,
                ..Default::default()
            };
            channel.retrigger_sample(true);
            let mut replay = channel.clone();
            for n in 1..=duration {
                let expected = 0.25 * n as f32 / duration as f32;
                let actual = sample_channels(&mut channel, &[8192; 16], rate);
                assert!((actual.0 - expected).abs() < 0.000001);
                assert_eq!(actual, sample_channels(&mut replay, &[8192; 16], rate));
                assert_eq!(channel.xm_restart_attack, replay.xm_restart_attack);
            }
            assert_eq!(channel.xm_restart_attack, 0);
            channel.xm_restart_attack = 5;
            channel.trigger_note(49, None);
            assert_eq!(channel.xm_restart_attack, 0);
        }
    }

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
    fn measured_xm_period_roundtrip_and_integer_slide() {
        for tuning in [-1600, -64, 0, 32, 64, 96, 1600] {
            for native in (1..65536).step_by(127) {
                let period = xm_runtime_period(native as f64, tuning);
                assert_eq!(
                    xm_native_period(f64::from(period), tuning).round(),
                    native as f64
                );
                let next = slide_xm_period(period, 4.0, true, tuning);
                let expected = xm_runtime_period((native + 4) as f64, tuning);
                assert!(next.is_finite() && next >= 1.0);
                if expected >= 1.0 {
                    assert_eq!(
                        xm_native_period(f64::from(next), tuning).round(),
                        (native + 4) as f64
                    );
                } else {
                    assert_eq!(next, 1.0);
                }
            }
        }
        assert_eq!(slide_xm_period(4608.0, -20.0, false, 64), 4588.0);
        // Original failing glissando boundary: seven -20 source-period steps.
        let mut period = 4608.0;
        for _ in 0..7 {
            period = slide_xm_period(period, -20.0, true, 64);
        }
        assert_eq!(period, 4512.0);
        assert_eq!((period / 64.0).round() * 64.0, 4544.0);
    }

    #[test]
    fn xm_slide_lower_bound_stays_playable() {
        for relative in -128_i16..=127 {
            for fine in [-128_i16, -64, 0, 64, 127] {
                let tuning = relative * 128 + fine;
                let period = slide_xm_period(4608.0, -1.0e12, true, tuning);
                assert!(period.is_finite() && period >= 1.0, "{tuning}: {period}");
                assert!(period_to_frequency(period) > 0.0);
                let repeated = slide_xm_period(period, -1020.0, true, tuning);
                assert_eq!(repeated, period, "lower-bound saturation: {tuning}");
                let vibrato = xm_vibrato_period(period, 15, 0, 192, true, tuning);
                assert!(vibrato.is_finite() && vibrato >= 1.0);
            }
        }
    }

    #[test]
    fn xm_default_finetune_boundaries_and_legacy_isolation() {
        for (source, expected) in [
            (-9, -16),
            (-8, -8),
            (-7, -8),
            (-1, -8),
            (0, 0),
            (1, 0),
            (7, 0),
            (8, 8),
            (9, 8),
            (119, 112),
            (120, 120),
            (121, 120),
            (127, 120),
        ] {
            assert_eq!(xm_sample_finetune(source, false), expected);
            assert_eq!(xm_sample_finetune(source, true), source);
        }
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
