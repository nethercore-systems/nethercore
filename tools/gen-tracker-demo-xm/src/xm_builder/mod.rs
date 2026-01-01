//! XM file generation framework
//!
//! This module provides the XM file format generation for tracker-demo.
//! Each genre has its own pattern module.

pub mod eurobeat;
pub mod funk;
pub mod synthwave;

use crate::synthesizers::SAMPLE_RATE;

// Re-export pattern generators
pub use eurobeat::{generate_eurobeat_xm, generate_eurobeat_xm_embedded};
pub use funk::{generate_funk_xm, generate_funk_xm_embedded};
pub use synthwave::{generate_synthwave_xm, generate_synthwave_xm_embedded};

// ============================================================================
// XM Pattern Note Helpers
// ============================================================================

/// Helper to write a note with volume
pub fn write_note_vol(data: &mut Vec<u8>, note: u8, instrument: u8, volume: u8) {
    data.push(0x80 | 0x01 | 0x02 | 0x04); // packed byte: has note + instrument + volume
    data.push(note);
    data.push(instrument);
    data.push(volume);
}

/// Helper to write a note
pub fn write_note(data: &mut Vec<u8>, note: u8, instrument: u8) {
    data.push(0x80 | 0x01 | 0x02); // packed byte: has note + instrument
    data.push(note);
    data.push(instrument);
}

/// Helper to write an empty channel
pub fn write_empty(data: &mut Vec<u8>) {
    data.push(0x80); // packed byte: nothing
}

/// Helper to write a note with volume and effect (e.g., note-cut)
/// effect_type: 0x0C = note cut (ECx), 0x0F = set speed, etc.
/// effect_param: parameter for the effect (e.g., tick to cut at)
pub fn _write_note_vol_fx(
    data: &mut Vec<u8>,
    note: u8,
    instrument: u8,
    volume: u8,
    effect_type: u8,
    effect_param: u8,
) {
    // packed byte: has note + instrument + volume + effect type + effect param
    data.push(0x80 | 0x01 | 0x02 | 0x04 | 0x08 | 0x10);
    data.push(note);
    data.push(instrument);
    data.push(volume);
    data.push(effect_type);
    data.push(effect_param);
}

/// Helper to write a note with effect (no explicit volume)
pub fn write_note_fx(data: &mut Vec<u8>, note: u8, instrument: u8, effect_type: u8, effect_param: u8) {
    // packed byte: has note + instrument + effect type + effect param
    data.push(0x80 | 0x01 | 0x02 | 0x08 | 0x10);
    data.push(note);
    data.push(instrument);
    data.push(effect_type);
    data.push(effect_param);
}

// ============================================================================
// Pitch Correction
// ============================================================================

/// Calculate finetune and relative_note for a sample at given sample rate.
///
/// XM expects samples tuned for 8363 Hz at C-4. This function calculates
/// the pitch correction needed to play a sample at the correct pitch.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio (e.g., 22050)
///
/// # Returns
/// (finetune, relative_note) where:
/// - finetune: -128 to 127 (1/128th semitone precision)
/// - relative_note: semitone offset from C-4
///
/// # Formula
/// ```text
/// semitones = 12 × log₂(sample_rate / 8363)
/// relative_note = floor(semitones)
/// finetune = round((semitones - relative_note) × 128)
/// ```
///
/// # Examples
/// - 22050 Hz → (101, 16) — plays 22050Hz sample at correct pitch
/// - 44100 Hz → (101, 28) — one octave higher than 22050
/// - 8363 Hz → (0, 0) — native XM rate, no correction needed
pub fn calculate_pitch_correction(sample_rate: u32) -> (i8, i8) {
    const BASE_FREQ: f64 = 8363.0; // C-4 reference frequency

    // Calculate semitone offset: 12 × log₂(sample_rate / 8363)
    let semitones = 12.0 * (sample_rate as f64 / BASE_FREQ).log2();

    // Split into integer and fractional parts
    let relative_note = semitones.floor() as i32;
    let finetune = ((semitones - relative_note as f64) * 128.0).round() as i32;

    // Handle edge case where finetune rounds to 128
    let (finetune, relative_note) = if finetune >= 128 {
        (finetune - 128, relative_note + 1)
    } else {
        (finetune, relative_note)
    };

    (finetune as i8, relative_note as i8)
}

// ============================================================================
// Instrument Writing
// ============================================================================

/// Write an instrument header with pitch correction for ROM samples at 22050 Hz.
///
/// This writes a full instrument header with num_samples=1 but sample_length=0.
/// The pitch correction (finetune/relative_note) tells the tracker how to play
/// ROM samples that are stored at 22050 Hz (ZX standard) when XM expects 8363 Hz.
///
/// For 22050 Hz samples: relative_note=16, finetune=101
/// Formula verification: 8363 × 2^((16 + 101/128) / 12) ≈ 22050 Hz
pub fn write_instrument(xm: &mut Vec<u8>, name: &str) {
    write_instrument_with_pitch(xm, name, SAMPLE_RATE as u32);
}

/// Write an instrument header with pitch correction for a specific sample rate.
///
/// Use this when samples are generated at different rates (e.g., bass at 11025 Hz).
pub fn write_instrument_with_pitch(xm: &mut Vec<u8>, name: &str, sample_rate: u32) {
    // Extended instrument header size INCLUDES the 4-byte field itself
    // Same structure as write_instrument_with_sample, but sample_length = 0
    // Content: 22 name + 1 type + 2 num_samples + 4 sample_header_size + 96 mapping +
    // 48 vol_env + 48 pan_env + 2 num_points + 6 sustain/loop + 2 env_flags +
    // 4 vibrato + 2 fadeout + 2 reserved = 239
    // Total with field: 4 + 239 = 243
    let header_size: u32 = 243;
    xm.extend_from_slice(&header_size.to_le_bytes());

    // Instrument name (22 bytes)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat_n(0u8, 22 - name_bytes.len().min(22)));

    // Instrument type (0)
    xm.push(0);

    // Number of samples (1) - we need a sample header for pitch info
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Sample header size (40 bytes)
    xm.extend_from_slice(&40u32.to_le_bytes());

    // Sample mapping (96 bytes - all notes map to sample 0)
    xm.extend(std::iter::repeat_n(0u8, 96));

    // Volume envelope points (48 bytes) - simple sustain envelope
    xm.extend_from_slice(&0u16.to_le_bytes()); // Point 0: x=0
    xm.extend_from_slice(&64u16.to_le_bytes()); // Point 0: y=64
    xm.extend(std::iter::repeat_n(0u8, 44)); // Remaining 11 points

    // Panning envelope points (48 bytes) - disabled
    xm.extend(std::iter::repeat_n(0u8, 48));

    // Number of volume/panning envelope points
    xm.push(1); // num_vol_points
    xm.push(1); // num_pan_points

    // Volume envelope sustain/loop points (3 bytes)
    xm.push(0); // vol_sustain
    xm.push(0); // vol_loop_start
    xm.push(0); // vol_loop_end

    // Panning envelope sustain/loop points (3 bytes)
    xm.push(0); // pan_sustain
    xm.push(0); // pan_loop_start
    xm.push(0); // pan_loop_end

    // Envelope type flags (2 bytes)
    xm.push(0x01); // vol_type (volume envelope enabled)
    xm.push(0x00); // pan_type (panning envelope disabled)

    // Vibrato (4 bytes)
    xm.push(0); // vibrato_type
    xm.push(0); // vibrato_sweep
    xm.push(0); // vibrato_depth
    xm.push(0); // vibrato_rate

    // Volume fadeout (2 bytes)
    xm.extend_from_slice(&328u16.to_le_bytes());

    // Reserved (2 bytes to reach header_size - 4)
    xm.extend_from_slice(&[0u8; 2]);

    // ========== Sample Header (40 bytes) ==========

    // Calculate pitch correction for the sample rate
    let (finetune, relative_note) = calculate_pitch_correction(sample_rate);

    // Sample length (4 bytes) - 0 because sample comes from ROM
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Loop start/length (no loop - ROM samples handle their own loops)
    xm.extend_from_slice(&0u32.to_le_bytes());
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Volume (64 = max)
    xm.push(64);

    // Finetune (signed byte)
    xm.push(finetune as u8);

    // Type (0x10 = 16-bit, no loop)
    xm.push(0x10);

    // Panning (128 = center)
    xm.push(128);

    // Relative note (signed byte)
    xm.push(relative_note as u8);

    // Reserved
    xm.push(0);

    // Sample name (22 bytes) - same as instrument name
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat_n(0u8, 22 - name_bytes.len().min(22)));

    // NO SAMPLE DATA - sample comes from ROM at runtime
}

/// Write an instrument header with embedded sample data
pub fn write_instrument_with_sample(xm: &mut Vec<u8>, name: &str, sample_data: &[i16]) {
    // Extended instrument header size INCLUDES the 4-byte field itself
    // Parser does: cursor.seek(header_start + header_size - 4)
    // Content: 22 name + 1 type + 2 num_samples + 4 sample_header_size + 96 mapping +
    // 48 vol_env + 48 pan_env + 2 num_points + 6 sustain/loop + 2 env_flags +
    // 4 vibrato + 2 fadeout + 2 reserved = 239
    // Total with field: 4 + 239 = 243
    let header_size: u32 = 243;
    xm.extend_from_slice(&header_size.to_le_bytes());

    // Instrument name (22 bytes)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat_n(0u8, 22 - name_bytes.len().min(22)));

    // Instrument type (0)
    xm.push(0);

    // Number of samples (1)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Sample header size (40 bytes)
    xm.extend_from_slice(&40u32.to_le_bytes());

    // Sample mapping (96 bytes - all notes map to sample 0)
    xm.extend(std::iter::repeat_n(0u8, 96));

    // Volume envelope points (48 bytes) - simple sustain envelope
    xm.extend_from_slice(&0u16.to_le_bytes()); // Point 0: x=0
    xm.extend_from_slice(&64u16.to_le_bytes()); // Point 0: y=64
    xm.extend(std::iter::repeat_n(0u8, 44)); // Remaining 11 points

    // Panning envelope points (48 bytes) - disabled
    xm.extend(std::iter::repeat_n(0u8, 48));

    // Number of volume/panning envelope points
    xm.push(1); // num_vol_points
    xm.push(1); // num_pan_points

    // Volume envelope sustain/loop points (3 bytes)
    xm.push(0); // vol_sustain
    xm.push(0); // vol_loop_start
    xm.push(0); // vol_loop_end

    // Panning envelope sustain/loop points (3 bytes)
    xm.push(0); // pan_sustain
    xm.push(0); // pan_loop_start
    xm.push(0); // pan_loop_end

    // Envelope type flags (2 bytes)
    xm.push(0x01); // vol_type (volume envelope enabled)
    xm.push(0x00); // pan_type (panning envelope disabled)

    // Vibrato (4 bytes)
    xm.push(0); // vibrato_type
    xm.push(0); // vibrato_sweep
    xm.push(0); // vibrato_depth
    xm.push(0); // vibrato_rate

    // Volume fadeout (2 bytes)
    xm.extend_from_slice(&328u16.to_le_bytes());

    // Reserved (2 bytes to reach header_size - 4)
    xm.extend_from_slice(&[0u8; 2]);

    // Sample header
    let sample_len = (sample_data.len() * 2) as u32; // 16-bit samples
    xm.extend_from_slice(&sample_len.to_le_bytes());

    // Loop start/length (no loop)
    xm.extend_from_slice(&0u32.to_le_bytes());
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Volume (64 = max)
    xm.push(64);

    // Calculate pitch correction for 22050 Hz samples
    // Formula: semitones = 12 × log₂(22050/8363) = 16.784
    // finetune = 100, relative_note = 16
    let (finetune, relative_note) = calculate_pitch_correction(SAMPLE_RATE as u32);
    xm.push(finetune as u8);

    // Type (0x10 = 16-bit)
    xm.push(0x10);

    // Panning (128 = center)
    xm.push(128);

    // Relative note (calculated from sample rate)
    xm.push(relative_note as u8);

    // Reserved
    xm.push(0);

    // Sample name (22 bytes) - same as instrument name
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat_n(0u8, 22 - name_bytes.len().min(22)));

    // Sample data (delta-encoded 16-bit)
    let mut old = 0i16;
    for &sample in sample_data {
        let delta = sample.wrapping_sub(old);
        xm.extend_from_slice(&delta.to_le_bytes());
        old = sample;
    }
}
