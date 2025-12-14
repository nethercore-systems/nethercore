# ADPCM Audio Compression Specification

**Status:** Rejected in favor of QOA
**Author:** Zerve
**Last Updated:** December 2025

---

## Executive Summary

This document specifies ADPCM compression integration for Emberware Z's existing audio system. The goal is to enable era-authentic audio that fits within the 16 MB ROM budget while maintaining the current channel-based playback architecture.

**Key decisions:**
- IMA-ADPCM compression (4:1 ratio vs PCM) for ROM efficiency
- 22,050 Hz output rate (11,025 Hz option in file format)
- Decompression at load time (init only) — zero runtime cost
- Mono required for SFX, optional stereo for music
- Three loading paths: procedural PCM, ROM assets (ADPCM), power-user ADPCM
- Existing playback API unchanged (16 SFX channels + 1 music channel)

---

## Scope & Integration

This specification focuses on **ADPCM compression integration** for ROM efficiency. It is not a redesign of the audio system, but an upgrade to support compressed audio assets.

**What this spec covers:**
- IMA-ADPCM codec implementation (encode/decode)
- `.ewzsnd` file format for compressed audio assets
- Integration with existing 16-channel audio system
- Three loading paths (procedural PCM, ROM ADPCM, power-user ADPCM)
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
All ADPCM decompression happens **at load time (init only)**. Once loaded, sounds are stored as PCM samples and played through the existing system. This means zero CPU cost during gameplay and full compatibility with the current architecture.

---

## Design Goals

1. **ROM efficiency** — Audio should not dominate the ROM budget (ADPCM provides 4:1 compression)
2. **Era-authentic sound** — PS1/Saturn/N64 quality, not modern HD audio
3. **Transparent integration** — Developers use the same playback API, compression is automatic
4. **Backwards compatible** — Procedural/synthesized PCM sounds still supported
5. **Rollback-friendly** — Audio state must be deterministic or fire-and-forget

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

### Music: ADPCM Samples

Music currently uses the same ADPCM-compressed sample format as SFX. Future tracker/sequencer support is planned.

**Current approach:**
- Music tracks are ADPCM-compressed audio files loaded via `rom_sound()`
- Played on dedicated music channel (channel 0)
- Same compression benefits as SFX (4:1 ratio)

**Future work:**
- Tracker/sequencer system with patterns and multi-channel arrangement
- See XM format research for potential direction
- Would use ADPCM samples within module format

**Storage comparison (for reference):**
```
ADPCM music file (3 min):   ~1-2 MB compressed
vs PCM stereo (3 min):      ~15 MB uncompressed

Ratio: ~8-15× smaller
```

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

Developers may mix sample rates. The host resamples from stored sample rate (11,025 or 22,050 Hz) to output rate (22,050 Hz) during decompression at load time.

---

## Mono vs Stereo

### Sound Effects: Mono Required

All SFX must be mono. Rationale:

1. **Stereo panning** — SFX use simple stereo panning (-1.0 left to 1.0 right) applied at playback
2. **ROM savings** — Stereo doubles size with no benefit when panning is applied in code
3. **Era authenticity** — PS1/N64 SFX were mono with runtime panning

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

> **Note:** .ewzsnd is an **ADPCM-only format** used for bundled ROM assets. It is not a dual-format container. Raw PCM sounds use the `load_sound()` function directly without a file format.

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

## Integration Architecture

### Decompression Strategy

**All decompression happens at load time (init only):**
1. Assets authored as WAV/FLAC → `ember-export` → .ewzsnd (ADPCM)
2. Bundled in ROM data pack via `ember.toml`
3. `rom_sound()` loads .ewzsnd → decodes ADPCM → stores as PCM i16
4. Playback system unchanged (works with PCM samples)

**Benefits:**
- Zero CPU cost during gameplay (no runtime decoding)
- Simple architecture (decoder only runs during init)
- ROM compression (4:1 ratio) without RAM cost
- Compatible with existing audio system

**Trade-off:**
- RAM usage: Decompressed PCM (but negligible on modern systems)
- ROM savings: 4:1 compression makes 16 MB budget feasible

### Asset Workflow

```
Developer workflow:
1. Author: sound.wav (PCM audio file)
2. Add to ember.toml:
   [[assets.sounds]]
   id = "jump"
   path = "assets/jump.wav"
3. Build: ember-export converts to jump.ewzsnd (ADPCM)
4. Pack: ember pack bundles into ROM data pack
5. Load: rom_sound(b"jump".as_ptr(), 4) in init()
6. Play: play_sound(handle, 1.0, 0.0) in update()/render()
```

### Current System Compatibility

**No changes to:**
- Audio::play/stop/set_rollback_mode trait
- 16 channel architecture
- Command buffering system
- Rodio playback backend
- 22,050 Hz output rate
- Equal-power stereo panning

**New additions:**
- ADPCM decoder (used only during asset loading)
- .ewzsnd file format parser
- `load_sound_adpcm()` FFI function
- ember-export ADPCM encoding support

---

## FFI Functions

Emberware Z provides three loading paths to support different use cases:

### 1. Procedural PCM Loading (Unchanged)

For runtime-generated or synthesized sounds:

```rust
/// Load PCM sound from WASM memory (init only)
///
/// Use for procedural/synthesized audio generated at runtime.
/// Data is raw 16-bit signed PCM samples.
///
/// # Arguments
/// * `data_ptr` — Pointer to i16 PCM samples in WASM memory
/// * `byte_len` — Size in bytes (must be even)
///
/// # Returns
/// Sound handle (0 = error)
fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
```

### 2. ROM Asset Loading (Primary Path - ADPCM)

For bundled game assets (recommended):

```rust
/// Load sound from ROM data pack (init only)
///
/// Primary method for loading bundled audio assets.
/// Automatically decompresses ADPCM to PCM at load time.
/// Assets are .ewzsnd format (ADPCM compressed).
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string (e.g., "jump", "music_stage1")
/// * `id_len` — Length of ID string
///
/// # Returns
/// Sound handle (0 = error)
fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;
```

### 3. Power User ADPCM Loading (Advanced)

For custom asset pipelines:

```rust
/// Load ADPCM sound from WASM memory (init only)
///
/// For advanced users handling their own compression.
/// Accepts raw .ewzsnd format data.
/// Decompresses ADPCM to PCM at load time.
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzsnd file data in WASM memory
/// * `byte_len` — Size in bytes
///
/// # Returns
/// Sound handle (0 = error)
fn load_sound_adpcm(data_ptr: *const u8, byte_len: u32) -> u32;
```

### Playback Functions (Unchanged)

All playback functions work identically regardless of loading method:

```rust
/// Play a sound effect (fire-and-forget)
fn play_sound(sound: u32, volume: f32, pan: f32);

/// Play on managed channel with looping control
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);

/// Update channel parameters
fn channel_set(channel: u32, volume: f32, pan: f32);

/// Stop a channel
fn channel_stop(channel: u32);

/// Play music (uses dedicated channel 0)
fn music_play(sound: u32, volume: f32);

/// Stop music
fn music_stop();

/// Set music volume
fn music_set_volume(volume: f32);
```

**Key points:**
- 16 SFX channels (fire-and-forget or managed)
- 1 dedicated music channel (always looping)
- Pan range: -1.0 (left) to 1.0 (right), 0.0 (center)
- Volume range: 0.0 to 1.0
- All sounds loaded during `init()`, playback in `update()`/`render()`

---

## Rollback Considerations

Audio is **fire-and-forget** for rollback purposes. The current implementation handles this via the `set_rollback_mode()` flag on the Audio trait.

**Technical implementation:**
- During rollback replay, `set_rollback_mode(true)` is called
- Audio commands are discarded (not sent to audio thread) during rollback
- This prevents audio desync without requiring state serialization
- When rollback completes, `set_rollback_mode(false)` resumes normal playback

**Behavior:**
1. **Sound effects triggered during rolled-back frames** — May play briefly, then cut off when rollback occurs. This is acceptable; players don't notice brief audio glitches during rollback.

2. **Music** — Continues playing, not affected by rollback. Music is non-gameplay state.

3. **No audio state in WASM memory** — All audio state lives host-side. WASM just issues play commands.

**Best practices:**
- Trigger sounds in `render()` for guaranteed playback (render is not rolled back)
- Or trigger in `update()` and accept brief artifacts during rollback
- Don't rely on audio state for game logic (audio is non-deterministic)

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

## Future Work: Tracker/Sequencer Music

A tracker-based music system is planned for future implementation. This would provide:
- Pattern-based sequencing (like MOD/XM formats)
- Multi-channel arrangement (8-16 voices)
- ADPCM-compressed samples within modules
- Small ROM footprint compared to streaming audio

**Potential direction:**
- Support XM (Extended Module) format for tooling compatibility
- Or use libopenmpt library to support MOD/XM/IT formats
- Custom format tailored to Emberware's needs remains an option

**Next steps:**
- Complete ADPCM compression implementation first
- **After ADPCM is done:** Create `docs/pending/tracker-music-spec.md` with detailed tracker system design
- This creates a continuous spec → implement → new spec workflow

This is deferred to allow ADPCM compression integration to be implemented first.

---

## References

- [IMA ADPCM specification](http://www.cs.columbia.edu/~hgs/audio/dvi/IMA_ADPCM.pdf)
- [Multimedia Wiki: IMA ADPCM](https://wiki.multimedia.cx/index.php/IMA_ADPCM)
- [PS1 SPU documentation](https://psx-spx.consoledev.net/soundprocessingunitspu/)
- [MOD format specification](https://www.fileformat.info/format/mod/corion.htm)