# Emberware Z Audio System Specification

**Status:** Proposal / Draft  
**Author:** Zerve  
**Last Updated:** December 2025

---

## Executive Summary

This document specifies the audio system for Emberware Z, using ADPCM compression for samples and a tracker-based system for music. The goal is era-authentic audio that fits within ROM constraints while providing good quality.

**Key decisions:**
- ADPCM compression (4:1 ratio vs PCM)
- 22,050 Hz default sample rate
- Mono required for SFX, optional stereo for music
- Tracker system for music (small ROM footprint, era-authentic)

---

## Design Goals

1. **ROM efficiency** — Audio should not dominate the ROM budget
2. **Era-authentic sound** — PS1/Saturn/N64 quality, not modern HD audio
3. **Simple for developers** — Load sample, play sample
4. **Rollback-friendly** — Audio state must be deterministic or fire-and-forget

---

## Audio Formats

### Sound Effects: ADPCM

All sound effects use IMA-ADPCM compression.

| Property | Specification |
|----------|---------------|
| Codec | IMA-ADPCM (DVI ADPCM) |
| Sample rate | 22,050 Hz (default), 11,025 Hz (lo-fi option) |
| Channels | Mono only |
| Bits per sample | 4 (compressed) |
| Block size | 1024 samples per block |

**Storage cost:**
```
22,050 Hz mono ADPCM ≈ 5.5 KB per second
11,025 Hz mono ADPCM ≈ 2.75 KB per second
```

**Comparison to PCM:**
```
PCM 16-bit 22,050 Hz mono:   44.1 KB/sec
ADPCM 22,050 Hz mono:         5.5 KB/sec
Compression ratio:            8:1
```

> **Note:** The 8:1 ratio comes from 16-bit → 4-bit (4:1) being the primary compression. ADPCM also provides inter-sample prediction which maintains quality despite the bit reduction.

### Music: Tracker Modules

Music uses a tracker/sequencer format with ADPCM-compressed samples.

| Property | Specification |
|----------|---------------|
| Format | Platform-specific module (see Module Format section) |
| Sample format | ADPCM (same as SFX) |
| Channels | 8-16 simultaneous voices |
| Pattern resolution | 64 rows per pattern typical |

**Storage cost:**
```
Typical module with samples: 50-200 KB per song
vs PCM stereo (3 min song):  15+ MB

Ratio: 75-300× smaller
```

This is how SNES, Genesis, PS1, and N64 games achieved full soundtracks in limited ROM.

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

Developers may mix sample rates. The host resamples to output rate at playback.

---

## Mono vs Stereo

### Sound Effects: Mono Required

All SFX must be mono. Rationale:

1. **Spatial audio** — SFX are positioned in 3D space; the engine calculates stereo panning from world position
2. **ROM savings** — Stereo doubles size with no benefit for positioned sounds
3. **Era authenticity** — PS1/N64 SFX were mono with hardware spatialization

### Music: Mono Default, Stereo Optional

| Option | Size | Quality | Recommendation |
|--------|------|---------|----------------|
| Mono | 1× | Good through any speakers | Default |
| Stereo | 2× | Richer stereo field | Premium option |

Tracker modules naturally support stereo panning per-channel, so even "mono" samples can create stereo spread through arrangement.

---

## IMA-ADPCM Specification

### Overview

IMA-ADPCM (Interactive Multimedia Association Adaptive Differential Pulse Code Modulation) encodes 16-bit PCM as 4-bit deltas with adaptive step sizing.

**Characteristics:**
- Fixed 4:1 compression (16-bit → 4-bit)
- Very fast decode (two table lookups + basic math per sample)
- Slight noise increase vs PCM (inaudible for game audio)
- Stateful decoder (must decode from block start)

### Decode Tables

```rust
/// Step size table (89 entries)
/// Index by step_index [0..88]
pub const STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17,
    19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118,
    130, 143, 157, 173, 190, 209, 230, 253, 279, 307,
    337, 371, 408, 449, 494, 544, 598, 658, 724, 796,
    876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
    2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358,
    5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899,
    15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767
];

/// Index adjustment table (16 entries)  
/// Index by 4-bit nibble value [0..15]
pub const INDEX_TABLE: [i32; 16] = [
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8
];
```

### Decoder State

```rust
pub struct AdpcmState {
    /// Current predicted sample value [-32768, 32767]
    predictor: i32,
    
    /// Current index into step table [0, 88]  
    step_index: i32,
}

impl AdpcmState {
    pub fn new() -> Self {
        Self {
            predictor: 0,
            step_index: 0,
        }
    }
    
    /// Reset state (call at start of new sound)
    pub fn reset(&mut self) {
        self.predictor = 0;
        self.step_index = 0;
    }
}
```

### Decode Implementation

```rust
impl AdpcmState {
    /// Decode a single 4-bit nibble to 16-bit sample
    #[inline]
    pub fn decode_nibble(&mut self, nibble: u8) -> i16 {
        let nibble = nibble & 0x0F;
        let step = STEP_TABLE[self.step_index as usize];
        
        // Compute difference
        // diff = (nibble + 0.5) * step / 4
        // Implemented without floating point:
        let mut diff = step >> 3;
        if nibble & 1 != 0 { diff += step >> 2; }
        if nibble & 2 != 0 { diff += step >> 1; }
        if nibble & 4 != 0 { diff += step; }
        
        // Apply sign
        if nibble & 8 != 0 {
            self.predictor -= diff;
        } else {
            self.predictor += diff;
        }
        
        // Clamp to 16-bit range
        self.predictor = self.predictor.clamp(-32768, 32767);
        
        // Update step index
        self.step_index = (self.step_index + INDEX_TABLE[nibble as usize]).clamp(0, 88);
        
        self.predictor as i16
    }
    
    /// Decode a byte (2 samples, low nibble first)
    #[inline]
    pub fn decode_byte(&mut self, byte: u8) -> [i16; 2] {
        let sample0 = self.decode_nibble(byte & 0x0F);
        let sample1 = self.decode_nibble(byte >> 4);
        [sample0, sample1]
    }
}

/// Decode entire ADPCM buffer to PCM
pub fn decode_adpcm(adpcm_data: &[u8], pcm_output: &mut [i16]) {
    let mut state = AdpcmState::new();
    let mut out_idx = 0;
    
    for &byte in adpcm_data {
        let [s0, s1] = state.decode_byte(byte);
        
        if out_idx < pcm_output.len() {
            pcm_output[out_idx] = s0;
            out_idx += 1;
        }
        if out_idx < pcm_output.len() {
            pcm_output[out_idx] = s1;
            out_idx += 1;
        }
    }
}
```

### Encode Implementation

```rust
impl AdpcmState {
    /// Encode a single 16-bit sample to 4-bit nibble
    pub fn encode_sample(&mut self, sample: i16) -> u8 {
        let step = STEP_TABLE[self.step_index as usize];
        let diff = sample as i32 - self.predictor;
        
        // Determine nibble value
        let mut nibble: u8 = 0;
        let mut diff_abs = diff.abs();
        let mut predicted_diff = 0i32;
        
        if diff_abs >= step {
            nibble |= 4;
            diff_abs -= step;
            predicted_diff += step;
        }
        if diff_abs >= step >> 1 {
            nibble |= 2;
            diff_abs -= step >> 1;
            predicted_diff += step >> 1;
        }
        if diff_abs >= step >> 2 {
            nibble |= 1;
            predicted_diff += step >> 2;
        }
        
        // Add minimum step
        predicted_diff += step >> 3;
        
        // Apply sign
        if diff < 0 {
            nibble |= 8;
            self.predictor -= predicted_diff;
        } else {
            self.predictor += predicted_diff;
        }
        
        // Clamp predictor
        self.predictor = self.predictor.clamp(-32768, 32767);
        
        // Update step index
        self.step_index = (self.step_index + INDEX_TABLE[nibble as usize]).clamp(0, 88);
        
        nibble
    }
}

/// Encode PCM buffer to ADPCM
pub fn encode_adpcm(pcm_data: &[i16], adpcm_output: &mut [u8]) {
    let mut state = AdpcmState::new();
    
    for (i, chunk) in pcm_data.chunks(2).enumerate() {
        let nibble0 = state.encode_sample(chunk[0]);
        let nibble1 = if chunk.len() > 1 {
            state.encode_sample(chunk[1])
        } else {
            state.encode_sample(0) // Pad with silence
        };
        
        if i < adpcm_output.len() {
            adpcm_output[i] = nibble0 | (nibble1 << 4);
        }
    }
}
```

---

## Sound File Format (.ewzsnd)

### Header (16 bytes)

```rust
#[repr(C, packed)]
struct SoundHeader {
    /// Magic bytes: "EWZS"
    magic: [u8; 4],
    
    /// Format version
    version: u8,
    
    /// Flags
    /// Bit 0: 0 = 22050 Hz, 1 = 11025 Hz  
    /// Bit 1: 0 = mono, 1 = stereo (music only)
    /// Bit 2: 0 = one-shot, 1 = looping
    /// Bits 3-7: reserved
    flags: u8,
    
    /// Loop start (in samples, 0 if not looping)
    loop_start: u16,
    
    /// Total sample count
    sample_count: u32,
    
    /// Reserved for future use
    reserved: [u8; 4],
}
```

### Data Section

Immediately following header:
```
ADPCM data: ceil(sample_count / 2) bytes
```

### Example File

```
Offset  Size    Content
------  ------  -------
0x0000  4       Magic "EWZS"
0x0004  1       Version (1)
0x0005  1       Flags (0x00 = 22050 Hz, mono, one-shot)
0x0006  2       Loop start (0)
0x0008  4       Sample count (22050 = 1 second)
0x000C  4       Reserved
0x0010  11025   ADPCM data (22050 samples ÷ 2)

Total: 11,041 bytes for 1 second of audio
```

---

## Proposed FFI Functions

> **Open Question:** Function naming convention
> - `sound_*` prefix: `sound_load`, `sound_play`
> - `audio_*` prefix: `audio_load`, `audio_play`
> - `sfx_*` / `music_*` split: `sfx_play`, `music_play`

### Sound Loading

```rust
/// Load a sound effect from ROM
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzsnd data (embedded via include_bytes!)
/// * `byte_size` — Size of data
///
/// # Returns
/// Sound handle (0 = error)
fn load_sound(data_ptr: *const u8, byte_size: u32) -> u32;

/// Unload a sound and free resources
fn unload_sound(handle: u32);
```

### Sound Playback

```rust
/// Play a sound effect
///
/// # Arguments  
/// * `handle` — Sound handle from load_sound()
/// * `volume` — Volume 0.0 to 1.0
/// * `pan` — Stereo pan -1.0 (left) to 1.0 (right), 0.0 = center
///
/// # Returns
/// Voice ID for this playback instance (0 = failed, no free voices)
fn play_sound(handle: u32, volume: f32, pan: f32) -> u32;

/// Play a sound with 3D positioning
///
/// # Arguments
/// * `handle` — Sound handle
/// * `volume` — Base volume 0.0 to 1.0
/// * `x`, `y`, `z` — World position
///
/// Pan and attenuation calculated from listener position.
fn play_sound_3d(handle: u32, volume: f32, x: f32, y: f32, z: f32) -> u32;

/// Stop a playing voice
fn stop_voice(voice_id: u32);

/// Stop all instances of a sound
fn stop_sound(handle: u32);

/// Set master volume
fn set_master_volume(volume: f32);
```

### Listener (for 3D audio)

```rust
/// Set listener position and orientation for 3D audio
///
/// # Arguments
/// * `x`, `y`, `z` — Listener position
/// * `forward_x/y/z` — Forward direction (normalized)
/// * `up_x/y/z` — Up direction (normalized)
fn set_listener(
    x: f32, y: f32, z: f32,
    forward_x: f32, forward_y: f32, forward_z: f32,
    up_x: f32, up_y: f32, up_z: f32
);
```

### Music (Tracker)

```rust
/// Load a music module
///
/// # Arguments
/// * `data_ptr` — Pointer to module data
/// * `byte_size` — Size of data
///
/// # Returns
/// Module handle (0 = error)
fn load_music(data_ptr: *const u8, byte_size: u32) -> u32;

/// Play music module
///
/// # Arguments
/// * `handle` — Module handle
/// * `loop` — Whether to loop (1) or play once (0)
fn play_music(handle: u32, loop_flag: u32);

/// Stop current music
fn stop_music();

/// Pause/unpause music
fn pause_music(paused: u32);

/// Set music volume
fn set_music_volume(volume: f32);

/// Unload music module
fn unload_music(handle: u32);
```

---

## Voice Management

The host manages a fixed pool of voices (simultaneous sounds).

| Property | Value |
|----------|-------|
| Max voices (SFX) | 16-32 |
| Max voices (music) | 8-16 (within module) |
| Voice stealing | Oldest voice with same sound, then oldest overall |

When all voices are in use:
1. If the same sound is already playing, steal its oldest instance
2. Otherwise, steal the globally oldest voice
3. Return voice ID (never fail silently)

---

## Rollback Considerations

Audio is **fire-and-forget** for rollback purposes:

1. **Sound effects triggered during rolled-back frames** — May play briefly, then cut off. This is acceptable; players don't notice brief audio glitches during rollback.

2. **Music** — Continues playing, not affected by rollback. Music is non-gameplay state.

3. **No audio state in WASM memory** — All audio state lives host-side. WASM just issues play commands.

**Important:** Don't trigger sounds in `update()` that depend on game state that might be rolled back. Either:
- Trigger sounds in `render()` (not rolled back)
- Accept brief audio artifacts during rollback

---

## Budget Examples

### Fighting Game (10 characters)

```
Per-character voice set:
  10 callouts × 0.5 sec × 5.5 KB/sec = 27.5 KB per character
  10 characters = 275 KB

Announcer:
  20 callouts × 0.75 sec × 5.5 KB/sec = 82.5 KB

Common SFX:
  Hits, blocks, specials: 50 sounds × 0.25 sec × 5.5 KB/sec = 69 KB

Music:
  4 stage themes × 100 KB = 400 KB
  Menu/select music = 75 KB

UI sounds:
  20 sounds × 0.1 sec × 2.75 KB/sec (11025 Hz) = 5.5 KB
──────────────────────────────────────────────────────────
Total: ~900 KB
```

Very comfortable within 16 MB budget.

### Comparison to PCM

Same content as PCM 16-bit:
```
Character voices: 275 KB × 8 = 2.2 MB
Announcer: 82.5 KB × 8 = 660 KB
SFX: 69 KB × 8 = 552 KB
Music: Would need streaming or ~15 MB per song
──────────────────────────────────────────────────────
Total: 3.4 MB (SFX only) + impossible music
```

ADPCM makes the audio budget tractable.

---

## Tracker Module Format

> **Open Question:** Use existing format or define custom?
>
> Options:
> - **MOD** — Classic Amiga format, simple, well-documented
> - **XM** — Extended MOD, more features, FastTracker II
> - **IT** — Impulse Tracker, most features
> - **Custom** — Tailored to Emberware needs
>
> Recommendation: Support MOD or XM for familiarity, or define a simple custom format optimized for ADPCM samples.

### Custom Module Format (Proposed)

If using a custom format:

```rust
#[repr(C, packed)]
struct ModuleHeader {
    magic: [u8; 4],          // "EWZM"
    version: u8,
    flags: u8,
    num_channels: u8,        // 4-16
    num_patterns: u8,
    num_samples: u8,
    song_length: u8,         // Number of pattern entries in order list
    restart_position: u8,    // For looping
    initial_tempo: u8,       // Ticks per row
    initial_bpm: u8,         // Rows per minute / 4
    reserved: [u8; 2],
}

// Followed by:
// - Order list: song_length × u8 (pattern indices)
// - Sample headers: num_samples × SampleHeader
// - Pattern data: num_patterns × pattern_size
// - Sample data: concatenated ADPCM data
```

---

## Open Questions

1. **Tracker format** — MOD/XM/IT compatibility or custom format?

2. **Sample rate options** — Only 22050/11025, or allow 44100 for music?

3. **Voice count** — 16 sufficient, or need 32 for complex scenes?

4. **Streaming** — Any support for streaming long audio from ROM? (Probably not needed with tracker music)

5. **Effects** — Should host provide reverb/chorus, or leave to tracker?

6. **Function naming** — `sound_*`, `audio_*`, or split `sfx_*`/`music_*`?

---

## References

- [IMA ADPCM specification](http://www.cs.columbia.edu/~hgs/audio/dvi/IMA_ADPCM.pdf)
- [Multimedia Wiki: IMA ADPCM](https://wiki.multimedia.cx/index.php/IMA_ADPCM)
- [PS1 SPU documentation](https://psx-spx.consoledev.net/soundprocessingunitspu/)
- [MOD format specification](https://www.fileformat.info/format/mod/corion.htm)