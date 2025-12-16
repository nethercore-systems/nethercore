# QOA Audio Compression Specification

**Status:** Integration Spec / Ready for Implementation
**Author:** Claude (based on ADPCM spec by Zerve)
**Last Updated:** December 2025

---

## Executive Summary

This document specifies QOA (Quite OK Audio) compression integration for Emberware Z's existing audio system. QOA replaces IMA-ADPCM as the audio codec, providing 20% better compression with improved quality.

**Key decisions:**
- QOA compression (5:1 ratio vs PCM, 3.2 bits/sample)
- 22,050 Hz output rate (11,025 Hz option in file format)
- Decompression at load time (init only) — zero runtime cost
- Mono required for SFX, optional stereo for music
- Three loading paths: procedural PCM, ROM assets (QOA), power-user QOA
- Existing playback API unchanged (16 SFX channels + 1 music channel)

**Why QOA over IMA-ADPCM:**
- 20% smaller files (3.2 bits vs 4.0 bits per sample)
- Better audio quality (less "crunch", cleaner transients)
- Modern codec designed for games (2023)
- Simple implementation (~150 lines decode, ~200 lines encode)
- Fits Emberware Z philosophy: modern techniques, retro constraints

---

## Scope & Integration

This specification focuses on **QOA compression integration** for ROM efficiency. It is not a redesign of the audio system, but an upgrade to support compressed audio assets.

**What this spec covers:**
- QOA codec implementation (encode/decode)
- `.ewzsnd` file format for compressed audio assets
- Integration with existing 16-channel audio system
- Three loading paths (procedural PCM, ROM QOA, power-user QOA)
- Decompression strategy (load time vs runtime)

**What remains unchanged:**
- Channel-based playback architecture (16 SFX + 1 music)
- Existing FFI playback functions (`play_sound`, `channel_*`, `music_*`)
- Audio command buffering system
- Rollback integration (`set_rollback_mode()`)
- 22,050 Hz playback rate

**Out of scope (future work):**
- 3D spatial audio positioning
- Tracker/sequencer music system
- Runtime audio effects (reverb, chorus, etc.)

**Integration approach:**
All QOA decompression happens **at load time (init only)**. Once loaded, sounds are stored as PCM samples and played through the existing system. This means zero CPU cost during gameplay and full compatibility with the current architecture.

---

## Design Goals

1. **ROM efficiency** — Audio should not dominate the ROM budget (QOA provides 5:1 compression)
2. **Quality without cruft** — Clean audio without era-specific artifacts
3. **Transparent integration** — Developers use the same playback API, compression is automatic
4. **Backwards compatible** — Procedural/synthesized PCM sounds still supported
5. **Rollback-friendly** — Audio state must be deterministic or fire-and-forget

---

## QOA vs IMA-ADPCM Comparison

| Metric | IMA-ADPCM | QOA | Improvement |
|--------|-----------|-----|-------------|
| Bits per sample | 4.0 | 3.2 | 20% smaller |
| Compression ratio | 4:1 | 5:1 | Better |
| Quality | Noticeable artifacts | Cleaner | Better |
| Decode complexity | ~25 lines | ~80 lines | Acceptable |
| Encode complexity | ~40 lines | ~120 lines | Acceptable |
| Era authentic | Yes (1992) | No (2023) | N/A |

**Storage cost comparison (22,050 Hz mono):**
```
PCM 16-bit:     44.1 KB/sec
IMA-ADPCM:      11.0 KB/sec (4:1)
QOA:             8.8 KB/sec (5:1)
```

---

## Audio Formats

### Sound Effects: QOA

All sound effects use QOA compression.

| Property | Specification |
|----------|---------------|
| Codec | QOA (Quite OK Audio) |
| Sample rate | 22,050 Hz (default), 11,025 Hz (lo-fi option) |
| Channels | Mono only |
| Bits per sample | 3.2 (compressed) |
| Frame size | 5,120 samples max |

**Storage cost:**
```
22,050 Hz mono QOA ≈ 4.4 KB per second
11,025 Hz mono QOA ≈ 2.2 KB per second
```

### Music: QOA Samples

Music uses the same QOA-compressed sample format as SFX.

**Current approach:**
- Music tracks are QOA-compressed audio files loaded via `rom_sound()`
- Played on dedicated music channel (channel 0)
- Same compression benefits as SFX (5:1 ratio)

**Future work:**
- Tracker/sequencer system with patterns and multi-channel arrangement
- Would use QOA samples within module format

---

## Sample Rate Guidance

| Use Case | Recommended Rate | Notes |
|----------|------------------|-------|
| Character voices | 22,050 Hz | Speech needs clarity |
| Announcer callouts | 22,050 Hz | "Round 1, FIGHT!" |
| Impact/hit sounds | 22,050 Hz | Punchy, clear transients |
| Ambient loops | 11,025 Hz | Background, less critical |
| UI sounds | 11,025 Hz | Bleeps and bloops |
| Music samples | 22,050 Hz | Instrument clarity |
| Lo-fi aesthetic | 11,025 Hz | Intentional crunch |

---

## QOA Specification

### Overview

QOA (Quite OK Audio) uses an LMS (Least Mean Squares) predictor with adaptive quantization. It encodes 20 samples into 64-bit "slices" using 3-bit residuals.

**Characteristics:**
- Fixed 5:1 compression (16-bit → 3.2-bit effective)
- LMS prediction with 4-tap filter
- 16 scalefactor levels for adaptive quantization
- Frame-based structure (max 5,120 samples per frame)
- Simple integer math (no floating point)

### Constants

```rust
/// QOA file magic number ("qoaf" in big-endian)
pub const QOA_MAGIC: u32 = 0x716f6166;

/// Samples per slice
pub const QOA_SLICE_LEN: usize = 20;

/// Maximum slices per frame per channel
pub const QOA_MAX_SLICES: usize = 256;

/// Maximum samples per frame (256 slices × 20 samples)
pub const QOA_FRAME_SAMPLES: usize = 5120;

/// LMS filter history/weight length
pub const QOA_LMS_LEN: usize = 4;

/// File header size
pub const QOA_FILE_HEADER_SIZE: usize = 8;

/// Frame header size
pub const QOA_FRAME_HEADER_SIZE: usize = 8;

/// LMS state size per channel (4 history + 4 weights as i16)
pub const QOA_LMS_STATE_SIZE: usize = 16;
```

### Lookup Tables

```rust
/// Scalefactor table (16 entries)
/// Used to scale residuals during quantization
pub const QOA_SCALEFACTOR_TAB: [i32; 16] = [
    1, 7, 21, 45, 84, 138, 211, 304,
    421, 562, 731, 928, 1157, 1419, 1715, 2048
];

/// Quantization table (17 entries)
/// Maps residual / scalefactor result (-8..8) to 3-bit index
pub const QOA_QUANT_TAB: [u8; 17] = [
    7, 7, 7, 5, 5, 3, 3, 1,  // -8..-1
    0,                        //  0
    0, 2, 2, 4, 4, 6, 6, 6   //  1..8
];

/// Dequantization table (16 scalefactors × 8 quantized values)
/// Pre-computed: dequant_tab[sf][qval] = round(scalefactor * dequant_mul[qval])
/// where dequant_mul = [0.75, -0.75, 2.5, -2.5, 4.5, -4.5, 7.0, -7.0]
pub const QOA_DEQUANT_TAB: [[i32; 8]; 16] = [
    [   1,    -1,    3,    -3,    5,    -5,    7,    -7],
    [   5,    -5,   18,   -18,   32,   -32,   49,   -49],
    [  16,   -16,   53,   -53,   95,   -95,  147,  -147],
    [  34,   -34,  113,  -113,  203,  -203,  315,  -315],
    [  63,   -63,  210,  -210,  378,  -378,  588,  -588],
    [ 104,  -104,  345,  -345,  621,  -621,  966,  -966],
    [ 158,  -158,  528,  -528,  950,  -950, 1477, -1477],
    [ 228,  -228,  760,  -760, 1368, -1368, 2128, -2128],
    [ 316,  -316, 1053, -1053, 1895, -1895, 2947, -2947],
    [ 422,  -422, 1405, -1405, 2529, -2529, 3934, -3934],
    [ 548,  -548, 1828, -1828, 3290, -3290, 5117, -5117],
    [ 696,  -696, 2320, -2320, 4176, -4176, 6496, -6496],
    [ 868,  -868, 2893, -2893, 5207, -5207, 8099, -8099],
    [1064, -1064, 3548, -3548, 6386, -6386, 9933, -9933],
    [1286, -1286, 4288, -4288, 7718, -7718, 12005, -12005],
    [1536, -1536, 5120, -5120, 9216, -9216, 14336, -14336],
];
```

### LMS State

```rust
/// LMS (Least Mean Squares) predictor state
#[derive(Clone, Copy)]
pub struct QoaLms {
    /// History of last 4 reconstructed samples
    pub history: [i32; 4],

    /// Adaptive filter weights
    pub weights: [i32; 4],
}

impl QoaLms {
    /// Create new LMS state with default weights
    pub fn new() -> Self {
        Self {
            history: [0; 4],
            // Initial weights tuned for typical audio
            weights: [0, 0, -(1 << 13), 1 << 14],
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.history = [0; 4];
        self.weights = [0, 0, -(1 << 13), 1 << 14];
    }

    /// Predict next sample based on history
    #[inline]
    pub fn predict(&self) -> i32 {
        let mut prediction = 0i32;
        for i in 0..4 {
            prediction = prediction.wrapping_add(
                self.weights[i].wrapping_mul(self.history[i])
            );
        }
        prediction >> 13
    }

    /// Update weights and history after decoding a sample
    #[inline]
    pub fn update(&mut self, sample: i32, residual: i32) {
        let delta = residual >> 4;
        for i in 0..4 {
            self.weights[i] += if self.history[i] < 0 { -delta } else { delta };
        }

        // Shift history, add new sample
        self.history[0] = self.history[1];
        self.history[1] = self.history[2];
        self.history[2] = self.history[3];
        self.history[3] = sample;
    }
}
```

### Decode Implementation

```rust
/// Clamp value to 16-bit signed range
#[inline]
fn clamp_i16(v: i32) -> i32 {
    if v < -32768 { -32768 }
    else if v > 32767 { 32767 }
    else { v }
}

/// Decode a single slice (8 bytes = 20 samples)
///
/// Slice format (64 bits, big-endian):
/// - Bits 60-63: Scalefactor index (4 bits)
/// - Bits 0-59:  20 quantized residuals (3 bits each)
pub fn decode_slice(
    slice: u64,
    lms: &mut QoaLms,
    output: &mut [i16],
) -> usize {
    let scalefactor = ((slice >> 60) & 0xF) as usize;
    let mut sample_count = 0;

    for i in 0..QOA_SLICE_LEN {
        if sample_count >= output.len() {
            break;
        }

        // Extract 3-bit quantized value (from high bits down)
        let quantized = ((slice >> (57 - i * 3)) & 0x7) as usize;

        // Predict and dequantize
        let predicted = lms.predict();
        let dequantized = QOA_DEQUANT_TAB[scalefactor][quantized];
        let sample = clamp_i16(predicted + dequantized);

        // Update LMS state
        lms.update(sample, dequantized);

        output[sample_count] = sample as i16;
        sample_count += 1;
    }

    sample_count
}

/// Decode entire QOA buffer to PCM
pub fn decode_qoa(qoa_data: &[u8]) -> Result<(Vec<i16>, u32), &'static str> {
    if qoa_data.len() < QOA_FILE_HEADER_SIZE {
        return Err("File too small");
    }

    // Read file header
    let magic = u32::from_be_bytes([qoa_data[0], qoa_data[1], qoa_data[2], qoa_data[3]]);
    if magic != QOA_MAGIC {
        return Err("Invalid magic number");
    }

    let total_samples = u32::from_be_bytes([qoa_data[4], qoa_data[5], qoa_data[6], qoa_data[7]]);

    let mut output = Vec::with_capacity(total_samples as usize);
    let mut data_idx = QOA_FILE_HEADER_SIZE;
    let mut lms_states = [QoaLms::new(); 8]; // Max 8 channels
    let mut sample_rate = 0u32;

    while data_idx + QOA_FRAME_HEADER_SIZE <= qoa_data.len()
          && output.len() < total_samples as usize
    {
        // Read frame header
        let channels = qoa_data[data_idx] as usize;
        sample_rate = u32::from_be_bytes([
            0,
            qoa_data[data_idx + 1],
            qoa_data[data_idx + 2],
            qoa_data[data_idx + 3]
        ]);
        let samples_in_frame = u16::from_be_bytes([
            qoa_data[data_idx + 4],
            qoa_data[data_idx + 5]
        ]) as usize;
        let frame_size = u16::from_be_bytes([
            qoa_data[data_idx + 6],
            qoa_data[data_idx + 7]
        ]) as usize;

        if channels == 0 || channels > 8 {
            return Err("Invalid channel count");
        }

        data_idx += QOA_FRAME_HEADER_SIZE;

        // Read LMS state for each channel
        for ch in 0..channels {
            // History (4 × i16, big-endian)
            for i in 0..4 {
                lms_states[ch].history[i] = i16::from_be_bytes([
                    qoa_data[data_idx + i * 2],
                    qoa_data[data_idx + i * 2 + 1],
                ]) as i32;
            }
            data_idx += 8;

            // Weights (4 × i16, big-endian)
            for i in 0..4 {
                lms_states[ch].weights[i] = i16::from_be_bytes([
                    qoa_data[data_idx + i * 2],
                    qoa_data[data_idx + i * 2 + 1],
                ]) as i32;
            }
            data_idx += 8;
        }

        // Decode slices
        let slices_per_channel = (samples_in_frame + QOA_SLICE_LEN - 1) / QOA_SLICE_LEN;

        for slice_idx in 0..slices_per_channel {
            for ch in 0..channels {
                if data_idx + 8 > qoa_data.len() {
                    return Err("Truncated slice data");
                }

                let slice = u64::from_be_bytes([
                    qoa_data[data_idx], qoa_data[data_idx + 1],
                    qoa_data[data_idx + 2], qoa_data[data_idx + 3],
                    qoa_data[data_idx + 4], qoa_data[data_idx + 5],
                    qoa_data[data_idx + 6], qoa_data[data_idx + 7],
                ]);
                data_idx += 8;

                let samples_remaining = samples_in_frame
                    .saturating_sub(slice_idx * QOA_SLICE_LEN);
                let samples_to_decode = samples_remaining.min(QOA_SLICE_LEN);

                let mut temp = [0i16; QOA_SLICE_LEN];
                decode_slice(slice, &mut lms_states[ch], &mut temp[..samples_to_decode]);

                // For mono, copy directly; for stereo, mix to mono
                if channels == 1 {
                    output.extend_from_slice(&temp[..samples_to_decode]);
                }
                // Multi-channel mixing handled separately
            }
        }
    }

    Ok((output, sample_rate))
}
```

### Encode Implementation

```rust
/// Encode a slice of up to 20 samples
/// Returns the 64-bit encoded slice
pub fn encode_slice(samples: &[i16], lms: &mut QoaLms) -> u64 {
    // Try all 16 scalefactors, pick best
    let mut best_slice = 0u64;
    let mut best_error = i64::MAX;
    let mut best_lms = *lms;

    for sf in 0..16 {
        let mut test_lms = *lms;
        let mut slice = (sf as u64) << 60;
        let mut total_error = 0i64;

        for (i, &sample) in samples.iter().enumerate().take(QOA_SLICE_LEN) {
            let predicted = test_lms.predict();
            let residual = sample as i32 - predicted;

            // Quantize: divide by scalefactor, clamp to -8..8, lookup index
            let scaled = residual / QOA_SCALEFACTOR_TAB[sf].max(1);
            let clamped = scaled.clamp(-8, 8);
            let quantized = QOA_QUANT_TAB[(clamped + 8) as usize];

            // Dequantize to get reconstruction
            let dequantized = QOA_DEQUANT_TAB[sf][quantized as usize];
            let reconstructed = clamp_i16(predicted + dequantized);

            // Update LMS
            test_lms.update(reconstructed, dequantized);

            // Accumulate error
            let error = (sample as i32 - reconstructed).abs() as i64;
            total_error += error * error;

            // Pack quantized value into slice
            slice |= (quantized as u64) << (57 - i * 3);
        }

        if total_error < best_error {
            best_error = total_error;
            best_slice = slice;
            best_lms = test_lms;
        }
    }

    *lms = best_lms;
    best_slice
}

/// Encode PCM samples to QOA format
pub fn encode_qoa(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let total_samples = samples.len();
    let mut output = Vec::new();

    // File header
    output.extend_from_slice(&QOA_MAGIC.to_be_bytes());
    output.extend_from_slice(&(total_samples as u32).to_be_bytes());

    let mut lms = QoaLms::new();
    let mut sample_idx = 0;

    while sample_idx < total_samples {
        let samples_in_frame = (total_samples - sample_idx).min(QOA_FRAME_SAMPLES);
        let slices_in_frame = (samples_in_frame + QOA_SLICE_LEN - 1) / QOA_SLICE_LEN;

        // Calculate frame size
        let frame_size = QOA_FRAME_HEADER_SIZE + QOA_LMS_STATE_SIZE + slices_in_frame * 8;

        // Frame header (mono)
        output.push(1); // channels
        output.extend_from_slice(&sample_rate.to_be_bytes()[1..4]); // 24-bit
        output.extend_from_slice(&(samples_in_frame as u16).to_be_bytes());
        output.extend_from_slice(&(frame_size as u16).to_be_bytes());

        // LMS state
        for i in 0..4 {
            output.extend_from_slice(&(lms.history[i] as i16).to_be_bytes());
        }
        for i in 0..4 {
            output.extend_from_slice(&(lms.weights[i] as i16).to_be_bytes());
        }

        // Encode slices
        for slice_idx in 0..slices_in_frame {
            let start = sample_idx + slice_idx * QOA_SLICE_LEN;
            let end = (start + QOA_SLICE_LEN).min(total_samples);
            let slice = encode_slice(&samples[start..end], &mut lms);
            output.extend_from_slice(&slice.to_be_bytes());
        }

        sample_idx += samples_in_frame;
    }

    output
}
```

---

## Sound File Format (.ewzsnd)

### Header (8 bytes) + QOA Data

```rust
#[repr(C, packed)]
struct EwzSoundHeader {
    /// Magic bytes: "EWZS"
    magic: [u8; 4],

    /// Format version (2 = QOA)
    version: u8,

    /// Flags
    /// Bit 0: 0 = 22050 Hz, 1 = 11025 Hz
    /// Bit 1: 0 = mono, 1 = stereo
    /// Bit 2: 0 = one-shot, 1 = looping
    flags: u8,

    /// Loop start (in samples, 0 if not looping)
    loop_start: u16,
}
```

### Example File

```
Offset  Size    Content
------  ------  -------
0x0000  4       Magic "EWZS"
0x0004  1       Version (2 = QOA)
0x0005  1       Flags (0x00 = 22050 Hz, mono, one-shot)
0x0006  2       Loop start (0)
--- QOA data follows ---
0x0008  4       QOA Magic "qoaf"
0x000C  4       Total samples (22050 = 1 second)
0x0010  ...     Frame data

Total: ~4,400 bytes for 1 second (vs 11,000 ADPCM, vs 44,000 PCM)
```

---

## FFI Functions

### 1. Procedural PCM Loading (Unchanged)

```rust
/// Load PCM sound from WASM memory (init only)
fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
```

### 2. ROM Asset Loading (Primary Path - QOA)

```rust
/// Load sound from ROM data pack (init only)
/// Automatically decompresses QOA to PCM at load time.
fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;
```

### 3. Power User QOA Loading

```rust
/// Load QOA sound from WASM memory (init only)
fn load_sound_qoa(data_ptr: *const u8, byte_len: u32) -> u32;
```

### Playback Functions (Unchanged)

```rust
fn play_sound(sound: u32, volume: f32, pan: f32);
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);
fn channel_set(channel: u32, volume: f32, pan: f32);
fn channel_stop(channel: u32);
fn music_play(sound: u32, volume: f32);
fn music_stop();
fn music_set_volume(volume: f32);
```

---

## Budget Examples

### Fighting Game (10 characters)

```
Per-character voice set:
  10 callouts × 0.5 sec × 4.4 KB/sec = 22 KB per character
  10 characters = 220 KB

Announcer:
  20 callouts × 0.75 sec × 4.4 KB/sec = 66 KB

Common SFX:
  50 sounds × 0.25 sec × 4.4 KB/sec = 55 KB

Music:
  4 stage themes × 80 KB = 320 KB

UI sounds:
  20 sounds × 0.1 sec × 2.2 KB/sec = 4.4 KB
──────────────────────────────────────────────────────────
Total: ~725 KB (vs ~900 KB ADPCM = 20% savings)
```

---

## Testing & Validation

### Validation Strategy

To guarantee correctness, we use a **three-layer validation approach**:

1. **Unit tests** — Test individual functions (LMS predict, slice decode, etc.)
2. **Cross-validation** — Compare against `qoaudio` crate (decode) and reference encoder
3. **Roundtrip tests** — Encode → decode → verify SNR across multiple signal types

```
┌─────────────────────────────────────────────────────────────────┐
│                    VALIDATION PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐   │
│  │ Unit Tests   │ ──► │ Cross-Valid  │ ──► │ Roundtrip    │   │
│  │              │     │              │     │              │   │
│  │ • LMS predict│     │ • qoaudio    │     │ • Sine waves │   │
│  │ • Slice dec  │     │   crate      │     │ • Noise      │   │
│  │ • Clamp      │     │ • Reference  │     │ • Silence    │   │
│  │ • Tables     │     │   C encoder  │     │ • Impulse    │   │
│  └──────────────┘     └──────────────┘     └──────────────┘   │
│                                                                 │
│  If ALL THREE pass ──► Implementation is correct               │
└─────────────────────────────────────────────────────────────────┘
```

### Dev Dependencies

```toml
[dev-dependencies]
qoaudio = "0.7"      # Reference decoder for cross-validation
rand = "0.8"         # For noise generation in tests
```

---

### Unit Tests

#### Test 1: LMS Prediction

```rust
#[test]
fn test_lms_predict_known_values() {
    let mut lms = QoaLms {
        history: [1000, 2000, 3000, 4000],
        weights: [4096, 4096, 4096, 4096],  // All equal weights
    };

    // prediction = (1000*4096 + 2000*4096 + 3000*4096 + 4000*4096) >> 13
    //            = (4096000 + 8192000 + 12288000 + 16384000) >> 13
    //            = 40960000 >> 13
    //            = 5000
    let predicted = lms.predict();
    assert_eq!(predicted, 5000);
}

#[test]
fn test_lms_predict_default_weights() {
    // Default weights: [0, 0, -(1<<13), 1<<14]
    //                = [0, 0, -8192, 16384]
    let mut lms = QoaLms::new();
    lms.history = [100, 200, 300, 400];

    // prediction = (100*0 + 200*0 + 300*(-8192) + 400*16384) >> 13
    //            = (0 + 0 - 2457600 + 6553600) >> 13
    //            = 4096000 >> 13
    //            = 500
    let predicted = lms.predict();
    assert_eq!(predicted, 500);
}

#[test]
fn test_lms_update() {
    let mut lms = QoaLms::new();
    lms.history = [100, 200, 300, 400];
    lms.weights = [1000, 2000, 3000, 4000];

    // Update with sample=500, residual=160
    // delta = 160 >> 4 = 10
    // weights[i] += delta if history[i] >= 0, else -= delta
    // All history values are positive, so all weights increase by 10
    lms.update(500, 160);

    assert_eq!(lms.weights, [1010, 2010, 3010, 4010]);
    assert_eq!(lms.history, [200, 300, 400, 500]); // Shifted, new sample added
}

#[test]
fn test_lms_update_negative_history() {
    let mut lms = QoaLms::new();
    lms.history = [-100, 200, -300, 400];
    lms.weights = [1000, 2000, 3000, 4000];

    // delta = 160 >> 4 = 10
    // history[0] < 0: weight[0] -= 10 → 990
    // history[1] >= 0: weight[1] += 10 → 2010
    // history[2] < 0: weight[2] -= 10 → 2990
    // history[3] >= 0: weight[3] += 10 → 4010
    lms.update(500, 160);

    assert_eq!(lms.weights, [990, 2010, 2990, 4010]);
}
```

#### Test 2: Dequantization Table Verification

```rust
#[test]
fn test_dequant_table_symmetry() {
    // Verify table has correct symmetry: [+x, -x, +y, -y, ...]
    for sf in 0..16 {
        assert_eq!(QOA_DEQUANT_TAB[sf][0], -QOA_DEQUANT_TAB[sf][1]);
        assert_eq!(QOA_DEQUANT_TAB[sf][2], -QOA_DEQUANT_TAB[sf][3]);
        assert_eq!(QOA_DEQUANT_TAB[sf][4], -QOA_DEQUANT_TAB[sf][5]);
        assert_eq!(QOA_DEQUANT_TAB[sf][6], -QOA_DEQUANT_TAB[sf][7]);
    }
}

#[test]
fn test_dequant_table_scaling() {
    // Verify scalefactors increase monotonically
    for sf in 1..16 {
        assert!(QOA_DEQUANT_TAB[sf][0] > QOA_DEQUANT_TAB[sf - 1][0]);
    }
}

#[test]
fn test_dequant_table_known_values() {
    // Spot-check against reference implementation
    assert_eq!(QOA_DEQUANT_TAB[0], [1, -1, 3, -3, 5, -5, 7, -7]);
    assert_eq!(QOA_DEQUANT_TAB[15], [1536, -1536, 5120, -5120, 9216, -9216, 14336, -14336]);
}
```

#### Test 3: Slice Decode

```rust
#[test]
fn test_decode_slice_zeros() {
    // Slice with sf=0 and all quantized values = 0
    // Slice format: [sf:4][q0:3][q1:3]...[q19:3] = 64 bits
    // sf=0, all q=0 → slice = 0x0000_0000_0000_0000
    let slice: u64 = 0x0000_0000_0000_0000;
    let mut lms = QoaLms::new();
    let mut output = [0i16; 20];

    let decoded = decode_slice(slice, &mut lms, &mut output);

    assert_eq!(decoded, 20);
    // With default weights and zero history, prediction starts at 0
    // dequant[0][0] = 1, so first sample = clamp(0 + 1) = 1
    assert_eq!(output[0], 1);
}

#[test]
fn test_decode_slice_max_scalefactor() {
    // sf=15 (max), all q=0
    let slice: u64 = 0xF000_0000_0000_0000;
    let mut lms = QoaLms::new();
    let mut output = [0i16; 20];

    decode_slice(slice, &mut lms, &mut output);

    // dequant[15][0] = 1536
    assert_eq!(output[0], 1536);
}
```

---

### Cross-Validation Tests

#### Test 4: Decoder vs qoaudio Crate (Bit-Exact)

```rust
#[test]
fn test_decoder_matches_qoaudio() {
    // Generate test signal
    let original: Vec<i16> = (0..22050)
        .map(|i| {
            let t = i as f32 / 22050.0;
            (f32::sin(t * 440.0 * std::f32::consts::TAU) * 16000.0) as i16
        })
        .collect();

    // Encode with our encoder
    let encoded = encode_qoa(&original, 22050);

    // Decode with qoaudio crate (reference)
    let reference = qoaudio::decode_all(&encoded[..]).unwrap();

    // Decode with our decoder
    let (ours, _) = decode_qoa(&encoded).unwrap();

    // Must be BIT-EXACT (not just close)
    assert_eq!(ours.len(), reference.samples.len());
    for (i, (a, b)) in ours.iter().zip(reference.samples.iter()).enumerate() {
        assert_eq!(*a, *b, "Sample {} differs: ours={}, reference={}", i, a, b);
    }
}

#[test]
fn test_decode_reference_encoded_file() {
    // This test requires a .qoa file encoded with the reference C encoder
    // Generate with: qoaconv test.wav test.qoa

    // For CI, we embed a small known-good QOA file
    const REFERENCE_QOA: &[u8] = include_bytes!("test_data/reference_440hz.qoa");
    const EXPECTED_SAMPLES: &[i16] = include_bytes!("test_data/reference_440hz_pcm.raw")
        .chunks(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();

    let (decoded, sample_rate) = decode_qoa(REFERENCE_QOA).unwrap();

    assert_eq!(sample_rate, 22050);
    assert_eq!(decoded.len(), EXPECTED_SAMPLES.len());
    assert_eq!(decoded, EXPECTED_SAMPLES);
}
```

#### Test 5: Encoder Output Decodable by qoaudio

```rust
#[test]
fn test_encoder_output_valid_qoa() {
    let samples: Vec<i16> = (0..5120)  // One full frame
        .map(|i| ((i as f32 * 0.1).sin() * 10000.0) as i16)
        .collect();

    let encoded = encode_qoa(&samples, 22050);

    // Verify qoaudio can decode it without error
    let result = qoaudio::decode_all(&encoded[..]);
    assert!(result.is_ok(), "qoaudio failed to decode our output: {:?}", result.err());

    let decoded = result.unwrap();
    assert_eq!(decoded.samples.len(), samples.len());
}
```

---

### Roundtrip Tests

#### Test 6: Multiple Signal Types

```rust
fn compute_snr(original: &[i16], decoded: &[i16]) -> f64 {
    let signal: f64 = original.iter().map(|&s| (s as f64).powi(2)).sum();
    let noise: f64 = original.iter().zip(decoded.iter())
        .map(|(&a, &b)| (a as f64 - b as f64).powi(2))
        .sum();

    if noise == 0.0 { return f64::INFINITY; }
    10.0 * (signal / noise).log10()
}

fn generate_sine(freq: f32, sample_rate: u32, duration_sec: f32) -> Vec<i16> {
    let num_samples = (sample_rate as f32 * duration_sec) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (f32::sin(t * freq * std::f32::consts::TAU) * 16000.0) as i16
        })
        .collect()
}

fn generate_noise(sample_rate: u32, duration_sec: f32) -> Vec<i16> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let num_samples = (sample_rate as f32 * duration_sec) as usize;
    (0..num_samples)
        .map(|_| rng.gen_range(-16000i16..16000i16))
        .collect()
}

fn generate_silence(sample_rate: u32, duration_sec: f32) -> Vec<i16> {
    vec![0i16; (sample_rate as f32 * duration_sec) as usize]
}

fn generate_impulse(sample_rate: u32) -> Vec<i16> {
    let mut samples = vec![0i16; sample_rate as usize];
    samples[0] = 32767;
    samples[sample_rate as usize / 2] = -32768;
    samples
}

fn generate_sweep(sample_rate: u32, duration_sec: f32) -> Vec<i16> {
    let num_samples = (sample_rate as f32 * duration_sec) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let freq = 20.0 + (t / duration_sec) * 10000.0; // 20 Hz to 10 kHz
            (f32::sin(t * freq * std::f32::consts::TAU) * 16000.0) as i16
        })
        .collect()
}

#[test]
fn test_roundtrip_sine_440hz() {
    let original = generate_sine(440.0, 22050, 1.0);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    assert_eq!(decoded.len(), original.len());
    let snr = compute_snr(&original, &decoded);
    assert!(snr > 25.0, "440 Hz sine SNR too low: {:.1} dB", snr);
}

#[test]
fn test_roundtrip_sine_8000hz() {
    let original = generate_sine(8000.0, 22050, 1.0);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    let snr = compute_snr(&original, &decoded);
    assert!(snr > 20.0, "8 kHz sine SNR too low: {:.1} dB", snr);
}

#[test]
fn test_roundtrip_white_noise() {
    let original = generate_noise(22050, 1.0);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    let snr = compute_snr(&original, &decoded);
    // Noise is harder to compress, accept lower SNR
    assert!(snr > 15.0, "White noise SNR too low: {:.1} dB", snr);
}

#[test]
fn test_roundtrip_silence() {
    let original = generate_silence(22050, 1.0);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    // Silence should be near-perfect
    let max_error: i16 = original.iter().zip(&decoded)
        .map(|(a, b)| (a - b).abs())
        .max()
        .unwrap_or(0);
    assert!(max_error < 10, "Silence max error too high: {}", max_error);
}

#[test]
fn test_roundtrip_impulse() {
    let original = generate_impulse(22050);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    let snr = compute_snr(&original, &decoded);
    assert!(snr > 20.0, "Impulse SNR too low: {:.1} dB", snr);
}

#[test]
fn test_roundtrip_frequency_sweep() {
    let original = generate_sweep(22050, 2.0);
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    let snr = compute_snr(&original, &decoded);
    assert!(snr > 20.0, "Frequency sweep SNR too low: {:.1} dB", snr);
}
```

#### Test 7: Edge Cases

```rust
#[test]
fn test_roundtrip_single_sample() {
    let original = vec![12345i16];
    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    assert_eq!(decoded.len(), 1);
    // Single sample won't be exact, but should be close
    assert!((decoded[0] - original[0]).abs() < 1000);
}

#[test]
fn test_roundtrip_exactly_one_frame() {
    let original: Vec<i16> = (0..5120)  // Exactly QOA_FRAME_SAMPLES
        .map(|i| (i as i16).wrapping_mul(7))
        .collect();

    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    assert_eq!(decoded.len(), 5120);
}

#[test]
fn test_roundtrip_frame_boundary() {
    // 5121 samples = 1 full frame + 1 sample in next frame
    let original: Vec<i16> = (0..5121)
        .map(|i| (i as i16).wrapping_mul(7))
        .collect();

    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    assert_eq!(decoded.len(), 5121);
}

#[test]
fn test_roundtrip_max_amplitude() {
    // Samples at maximum amplitude
    let original: Vec<i16> = (0..22050)
        .map(|i| if i % 2 == 0 { 32767 } else { -32768 })
        .collect();

    let encoded = encode_qoa(&original, 22050);
    let (decoded, _) = decode_qoa(&encoded).unwrap();

    // Extreme signals will have more error, but should still decode
    assert_eq!(decoded.len(), original.len());
}

#[test]
fn test_different_sample_rates() {
    for &rate in &[8000u32, 11025, 22050, 44100, 48000] {
        let original = generate_sine(440.0, rate, 0.5);
        let encoded = encode_qoa(&original, rate);
        let (decoded, decoded_rate) = decode_qoa(&encoded).unwrap();

        assert_eq!(decoded_rate, rate);
        assert_eq!(decoded.len(), original.len());
    }
}
```

---

### Integration Tests

#### Test 8: Full Asset Pipeline

```rust
#[test]
fn test_ewzsnd_roundtrip() {
    let original = generate_sine(440.0, 22050, 1.0);

    // Create .ewzsnd file (Emberware header + QOA data)
    let qoa_data = encode_qoa(&original, 22050);
    let mut ewzsnd = Vec::new();
    ewzsnd.extend_from_slice(b"EWZS");  // Magic
    ewzsnd.push(2);                      // Version (QOA)
    ewzsnd.push(0x00);                   // Flags: 22050 Hz, mono, one-shot
    ewzsnd.extend_from_slice(&0u16.to_le_bytes()); // Loop start
    ewzsnd.extend_from_slice(&qoa_data);

    // Parse .ewzsnd
    assert_eq!(&ewzsnd[0..4], b"EWZS");
    assert_eq!(ewzsnd[4], 2); // Version

    // Decode QOA portion
    let (decoded, _) = decode_qoa(&ewzsnd[8..]).unwrap();
    assert_eq!(decoded.len(), original.len());
}
```

---

### CI Test Script

```bash
#!/bin/bash
# test_qoa.sh - Run all QOA validation tests

set -e

echo "=== QOA Validation Suite ==="

echo "1. Running unit tests..."
cargo test qoa::tests --release

echo "2. Running cross-validation against qoaudio..."
cargo test test_decoder_matches_qoaudio --release

echo "3. Running roundtrip tests..."
cargo test test_roundtrip_ --release

echo "4. Running edge case tests..."
cargo test test_roundtrip_single_sample --release
cargo test test_roundtrip_frame_boundary --release
cargo test test_roundtrip_max_amplitude --release

echo "5. Checking compression ratio..."
cargo test test_compression_ratio --release

echo ""
echo "=== All QOA tests passed! ==="
```

---

### Compression Ratio Verification

```rust
#[test]
fn test_compression_ratio() {
    let original = generate_sine(440.0, 22050, 10.0); // 10 seconds
    let encoded = encode_qoa(&original, 22050);

    let pcm_size = original.len() * 2; // 16-bit samples
    let qoa_size = encoded.len();
    let ratio = pcm_size as f64 / qoa_size as f64;

    // QOA should achieve approximately 5:1 compression
    assert!(ratio > 4.5, "Compression ratio too low: {:.2}:1", ratio);
    assert!(ratio < 5.5, "Compression ratio too high: {:.2}:1", ratio);

    println!("Compression ratio: {:.2}:1", ratio);
    println!("PCM size: {} bytes", pcm_size);
    println!("QOA size: {} bytes", qoa_size);
}
```

---

### Test Data Generation Script

For generating reference test files with the official QOA encoder:

```bash
#!/bin/bash
# generate_test_data.sh - Generate reference QOA files

# Requires: qoaconv from https://github.com/phoboslab/qoa

# Generate 440 Hz sine wave WAV (using sox or similar)
sox -n -r 22050 -b 16 -c 1 test_440hz.wav synth 1 sine 440

# Encode with reference encoder
./qoaconv test_440hz.wav test_data/reference_440hz.qoa

# Keep raw PCM for comparison
sox test_440hz.wav -t raw -e signed -b 16 test_data/reference_440hz_pcm.raw

echo "Test data generated in test_data/"
```

---

### Summary: Correctness Guarantees

| Test Layer | What It Verifies | Pass Criteria |
|------------|------------------|---------------|
| **Unit tests** | Individual functions work correctly | Exact expected values |
| **Cross-validation** | Our decoder matches qoaudio crate | Bit-exact output |
| **Encoder validation** | qoaudio can decode our output | No decode errors |
| **Roundtrip tests** | Encode→decode preserves signal | SNR > 20 dB |
| **Edge cases** | Boundary conditions handled | No panics, correct lengths |
| **Compression ratio** | Achieving expected 5:1 | Ratio between 4.5:1 and 5.5:1 |

**If all tests pass, the implementation is correct.**

---

## References

- [QOA Format Specification](https://qoaformat.org/)
- [QOA Reference Implementation](https://github.com/phoboslab/qoa)
- [QOA Algorithm Deep Dive](https://phoboslab.org/log/2023/02/qoa-time-domain-audio-compression)
- [qoaudio Rust Crate](https://docs.rs/qoaudio/latest/qoaudio/)
