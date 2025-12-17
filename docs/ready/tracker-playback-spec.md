# Tracker Playback System Specification

**Status:** Ready for Implementation
**Author:** Claude
**Version:** 2.0
**Last Updated:** December 2024

> **Key Change in v2.0:** Samples are loaded from ROM data pack (not embedded in XM). This enables sample reuse between trackers and SFX, significantly reducing ROM size.

---

## Summary

Implement a tracker module playback system for Emberware Z that allows games to play .XM (Extended Module) pattern data through the dedicated music channel. Tracker modules are compact, procedural music files that were standard on PS1/N64-era platforms—perfectly matching Emberware Z's aesthetic.

**Key concept:** Tracker files contain **pattern data only**—samples are loaded separately from the ROM data pack via naming convention. This allows sample reuse between trackers and SFX, keeping ROM sizes minimal.

**How it works:**
1. Samples declared in `[[assets.sounds]]` as usual
2. Tracker XM files reference samples by instrument name
3. At pack time, samples are stripped from XM; instrument names mapped to ROM sample IDs
4. At runtime, `rom_tracker()` resolves instruments to loaded samples

---

## Motivation

**Current music limitations:**
- Music channel only plays raw PCM loops (via `music_play()`)
- Large file sizes for music (22kHz × 16-bit × duration = ~2.6MB/minute)
- No dynamic music features (no pattern changes, no instrument variation)
- Wastes ROM space for long tracks

**Tracker module advantages:**
- **Tiny file sizes:** 50KB-500KB for full songs (vs. 2-10MB for PCM)
- **Era-authentic:** XM format from 1994, used extensively on PS1/N64/Amiga
- **Dynamic features:** Pattern-based, can jump to sections, change tempo
- **Instrument reuse:** Samples shared across patterns = space efficient
- **Open format:** XM is well-documented, many tools available
- **Rollback-friendly:** Deterministic playback from pattern position

**Use cases:**
- Full game soundtracks in <1MB total
- Dynamic music (combat → exploration transitions)
- Chiptune/retro aesthetic games
- Interactive music that responds to gameplay

---

## Format Selection: XM (Extended Module)

### Why XM?

| Format | Channels | Features | Era | Ecosystem |
|--------|----------|----------|-----|-----------|
| MOD | 4 | Basic | 1987 | Limited |
| S3M | 16 | Good | 1992 | Moderate |
| **XM** | **32** | **Excellent** | **1994** | **Excellent** |
| IT | 64 | Advanced | 1995 | Good |

**XM chosen because:**
1. **PS1/N64 era:** FastTracker 2 (1994) matches target aesthetic
2. **Rich features:** Volume/panning envelopes, vibrato, portamento
3. **32 channels:** Plenty for complex music
4. **Excellent tooling:** OpenMPT, MilkyTracker, Renoise export
5. **Well-documented:** Public specification, multiple implementations
6. **Reasonable complexity:** Simpler than IT, more capable than MOD/S3M
7. **Rust crates available:** `xmplayer`, `libxm-rs`, or custom implementation

### XM Format Overview

```
XM File Structure:
├── Header (60 bytes)
│   ├── ID: "Extended Module: "
│   ├── Module name (20 chars)
│   ├── Song length, restart position
│   ├── Channels, patterns, instruments
│   ├── Tempo, BPM
│   └── Pattern order table
├── Patterns (variable)
│   └── Per-channel note data (note, instrument, volume, effect)
└── Instruments (variable)
    ├── Envelope data (volume, panning)
    └── Samples (8/16-bit, variable rate)
```

---

## Current Audio Architecture

### Existing Music Channel (`emberware-z/src/ffi/audio.rs`)

```rust
// Current FFI
fn music_play(sound: u32, volume: f32);   // Play PCM loop
fn music_stop();                           // Stop playback
fn music_set_volume(volume: f32);          // Adjust volume

// Current rollback state (20 bytes)
struct ChannelState {
    sound: u32,      // Sound handle
    position: u32,   // Sample position
    looping: u32,    // Always 1 for music
    volume: f32,
    pan: f32,        // Always 0.0 for music
}
```

### Audio Generation (`emberware-z/src/audio.rs`)

```rust
fn generate_audio_frame(
    rollback: &ZRollbackState,
    sounds: &[Vec<i16>],           // PCM sample data
    tick_rate: u32,
    sample_rate: u32,
) -> Vec<f32> {
    // Mix 16 SFX channels + 1 music channel
    // Resample from 22.05kHz to output rate
    // Apply panning, soft clipping
}
```

---

## Proposed Architecture

### 1. TrackerModule Asset Type

New asset type in ZDataPack:

```rust
// z-common/src/formats/z_data_pack.rs
pub struct ZDataPack {
    // ... existing ...
    pub trackers: Vec<PackedTracker>,
    tracker_index: OnceLock<HashMap<String, usize>>,
}

pub struct PackedTracker {
    pub id: String,
    pub pattern_data: Vec<u8>,   // XM file with samples stripped
    pub sample_ids: Vec<String>, // Instrument index → ROM sample ID
}
```

**Note:** Samples are NOT embedded in the tracker. The `sample_ids` vec maps XM instrument indices to ROM sample IDs that must be loaded via `[[assets.sounds]]`.

### 2. TrackerState (Rollback-Safe)

Tracker playback state that can be rolled back:

```rust
// emberware-z/src/state/rollback_state.rs

/// Tracker playback state (POD for rollback)
/// Size: 64 bytes
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct TrackerState {
    /// Tracker handle (0 = none playing)
    pub handle: u32,

    /// Current position in pattern order table
    pub order_position: u16,

    /// Current row within pattern (0-255)
    pub row: u16,

    /// Current tick within row
    pub tick: u16,

    /// Ticks per row (from speed command)
    pub speed: u16,

    /// BPM (beats per minute)
    pub bpm: u16,

    /// Volume multiplier (0-256, 256 = 1.0)
    /// Note: Final volume = (this volume / 256) × XM global volume (Gxx effect)
    pub volume: u16,

    /// Flags: bit 0 = playing, bit 1 = looping, bit 2 = paused
    pub flags: u32,

    /// Sample-accurate position within tick
    pub tick_sample_pos: u32,

    /// Reserved for future use
    pub _reserved: [u8; 40],
}

// Updated ZRollbackState
pub struct ZRollbackState {
    pub audio: AudioPlaybackState,  // 340 bytes (unchanged)
    pub tracker: TrackerState,       // 64 bytes (NEW)
}
// Total: 404 bytes
```

### 3. TrackerEngine (Non-Rollback, Host-Side)

Heavy state that doesn't need rollback (reconstructable from TrackerState):

```rust
// emberware-z/src/audio/tracker.rs

pub struct TrackerEngine {
    /// Loaded tracker modules (by handle)
    modules: Vec<Option<XmModule>>,

    /// Per-channel state (32 channels max)
    channels: [TrackerChannel; 32],

    /// Mixing buffer
    mix_buffer: Vec<f32>,
}

struct TrackerChannel {
    sample_data: Option<Arc<[i16]>>,
    sample_pos: f64,           // Fractional for interpolation
    sample_loop_start: u32,
    sample_loop_end: u32,
    sample_loop_type: u8,      // 0=none, 1=forward, 2=pingpong

    volume: f32,               // 0.0-1.0
    panning: f32,              // -1.0 to 1.0

    period: f32,               // Frequency period
    target_period: f32,        // For portamento

    vibrato_pos: u8,
    vibrato_speed: u8,
    vibrato_depth: u8,

    tremolo_pos: u8,
    tremolo_speed: u8,
    tremolo_depth: u8,

    envelope_volume_pos: u16,
    envelope_panning_pos: u16,

    // ... effect state
}
```

### 4. Rollback Strategy

**Problem:** Full tracker channel state is large (~2KB for 32 channels)

**Solution:** Minimal rollback state + reconstruction

```
On Rollback:
1. Restore TrackerState (64 bytes) from GGRS snapshot
2. If order_position or row changed significantly:
   a. Seek tracker to (order_position, row)
   b. Process ticks from 0 to tick to restore channel states
3. Continue playback from exact position

This is deterministic because:
- XM playback is fully deterministic (no random)
- Same (order, row, tick) always produces same audio
- Channel states can be reconstructed by replaying from pattern start
```

**Optimization:** Cache channel states at row boundaries for fast rollback.

---

## Sample Resolution

### Naming Convention

XM instruments are mapped to ROM samples by **instrument name**:
- XM instrument named `"kick"` → resolves to `rom_sound("kick")`
- XM instrument named `"snare_01"` → resolves to `rom_sound("snare_01")`

**Single sample per instrument (MVP):** Each instrument maps to exactly one ROM sample. Pitch shifting handles all notes. Multi-sample instruments (where different notes trigger different samples) are not supported in MVP.

**Drum kit workaround:** Instead of one "drums" instrument with multiple samples, create separate instruments: "kick", "snare", "hihat", etc.

### Load-Time Validation

When `rom_tracker()` is called:
1. Parse XM pattern data from `PackedTracker`
2. Read `sample_ids` mapping
3. Resolve each sample ID to a loaded ROM sound handle
4. **Fail (return 0)** if any sample ID has no matching ROM sound

```rust
// Pseudocode
fn rom_tracker(id: &str) -> u32 {
    let tracker = data_pack.get_tracker(id)?;

    for (instr_idx, sample_id) in tracker.sample_ids.iter().enumerate() {
        let sound_handle = resolve_sound(sample_id);
        if sound_handle == 0 {
            log::warn!("Tracker '{}': instrument {} references unknown sample '{}'",
                       id, instr_idx, sample_id);
            return 0;  // Fail load
        }
        engine.bind_sample(instr_idx, sound_handle);
    }

    // Success
    engine.load_patterns(&tracker.pattern_data)
}
```

### Workflow for Composers

1. **Create samples** as individual `.wav` files (22.05kHz, 16-bit, mono)
2. **Add samples to ROM** via `[[assets.sounds]]` in `ember.toml`
3. **Create XM** in tracker software (MilkyTracker, OpenMPT, etc.)
4. **Name instruments** to exactly match ROM sample IDs
5. **Export XM** — samples will be stripped at pack time, only patterns kept

**Example:**
```toml
# ember.toml
[[assets.sounds]]
id = "kick"
path = "samples/kick.wav"

[[assets.sounds]]
id = "snare"
path = "samples/snare.wav"

[[assets.sounds]]
id = "bass"
path = "samples/bass.wav"

[[assets.trackers]]
id = "main_theme"
path = "music/main_theme.xm"
# XM instruments must be named: "kick", "snare", "bass"
```

---

## FFI API

### Loading (Init-only)

```rust
/// Load tracker module from ROM data pack
/// Returns: handle (>0) or 0 on error
fn rom_tracker(id_ptr: u32, id_len: u32) -> u32;

/// Load tracker from WASM memory (for embedded assets)
/// Returns: handle (>0) or 0 on error
fn load_tracker(data_ptr: u32, data_len: u32) -> u32;
```

### Playback Control

```rust
/// Start playing a tracker module
/// handle: Tracker handle from rom_tracker/load_tracker
/// volume: 0.0 to 1.0
/// looping: 0 = play once, 1 = loop
/// Note: Automatically stops PCM music (music_stop)
fn tracker_play(handle: u32, volume: f32, looping: u32);

/// Stop tracker playback
fn tracker_stop();

/// Pause/resume tracker
/// paused: 0 = resume, 1 = pause
fn tracker_pause(paused: u32);

/// Set tracker volume (multiplied with XM global volume)
fn tracker_set_volume(volume: f32);

/// Check if tracker is currently playing
/// Returns: 1 if playing, 0 if stopped/paused
fn tracker_is_playing() -> u32;
```

### Music Channel Interaction

**Tracker and PCM music are mutually exclusive:**
- `tracker_play()` automatically calls `music_stop()` internally
- `music_play()` automatically calls `tracker_stop()` internally
- Only one music source can be active at a time

This simplifies the mental model: there's one "music channel" that can either play PCM or tracker, but not both. Games don't need to manually manage stopping one before starting the other.

### Position Control (for dynamic music)

```rust
/// Jump to a specific position in the pattern order
/// order: Position in order table (0-based)
/// row: Row within pattern (0-based, usually 0)
fn tracker_jump(order: u32, row: u32);

/// Get current playback position
/// Returns: (order << 16) | row
fn tracker_position() -> u32;

/// Get song length in orders
fn tracker_length() -> u32;

/// Set playback speed (ticks per row)
/// speed: 1-31 (lower = faster)
fn tracker_set_speed(speed: u32);

/// Set playback tempo (BPM)
/// bpm: 32-255
fn tracker_set_tempo(bpm: u32);
```

### Query Functions

```rust
/// Get tracker info
/// Returns: (channels << 24) | (patterns << 16) | (instruments << 8) | orders
fn tracker_info(handle: u32) -> u32;

/// Get tracker name (copies to WASM memory)
/// Returns: bytes written
fn tracker_name(handle: u32, out_ptr: u32, max_len: u32) -> u32;
```

---

## Asset Pipeline

### ember.toml Manifest

```toml
[game]
id = "my-game"
title = "My Game"

# Samples declared here are available to all trackers
[[assets.sounds]]
id = "kick"
path = "samples/kick.wav"

[[assets.sounds]]
id = "snare"
path = "samples/snare.wav"

[[assets.sounds]]
id = "bass_synth"
path = "samples/bass.wav"

# Tracker references samples by instrument name
[[assets.trackers]]
id = "main_theme"
path = "music/main_theme.xm"

[[assets.trackers]]
id = "boss_battle"
path = "music/boss.xm"
```

### ember-export Support

```bash
# Validate XM file and check instrument names
ember-export tracker input.xm -o output.xm

# List instrument names (for debugging sample mapping)
ember-export tracker input.xm --list-instruments
```

**Validation checks:**
- Valid XM header and structure
- Channel count ≤ 32
- Instrument names are valid identifiers (no spaces, etc.)
- Warn if file contains large embedded samples (they'll be stripped)

### ember-cli pack

```rust
// tools/ember-cli/src/pack.rs

for entry in &manifest.assets.trackers {
    let xm_data = fs::read(&entry.path)?;

    // 1. Parse XM to extract instrument names
    let xm = parse_xm(&xm_data)?;

    // 2. Build sample_ids mapping from instrument names
    let mut sample_ids = Vec::new();
    for instr in &xm.instruments {
        let sample_id = instr.name.trim().to_string();
        // Validate sample exists in manifest
        if !manifest.assets.sounds.iter().any(|s| s.id == sample_id) {
            return Err(format!(
                "Tracker '{}' instrument '{}' references unknown sample '{}'",
                entry.id, instr.name, sample_id
            ));
        }
        sample_ids.push(sample_id);
    }

    // 3. Strip samples from XM, keep only pattern data
    let pattern_data = strip_xm_samples(&xm_data)?;

    trackers.push(PackedTracker {
        id: entry.id.clone(),
        pattern_data,
        sample_ids,
    });
}
```

---

## Audio Generation Integration

### Modified generate_audio_frame

```rust
// emberware-z/src/audio.rs

pub fn generate_audio_frame(
    rollback: &ZRollbackState,
    sounds: &[Vec<i16>],
    tracker_engine: &mut TrackerEngine,  // NEW
    tick_rate: u32,
    sample_rate: u32,
) -> Vec<f32> {
    let samples_per_tick = sample_rate / tick_rate;
    let mut output = vec![0.0f32; samples_per_tick as usize * 2];

    // Mix SFX channels (existing)
    for channel in &rollback.audio.channels {
        if channel.sound != 0 {
            mix_channel(&mut output, channel, sounds, sample_rate);
        }
    }

    // Mix music channel (existing PCM)
    if rollback.audio.music.sound != 0 {
        mix_channel(&mut output, &rollback.audio.music, sounds, sample_rate);
    }

    // Mix tracker (NEW) - takes priority over PCM music if playing
    if rollback.tracker.flags & 1 != 0 {  // Playing flag
        tracker_engine.render(
            &rollback.tracker,
            &mut output,
            sample_rate,
        );
    }

    // Soft clip
    soft_clip(&mut output);

    output
}
```

### TrackerEngine::render

```rust
impl TrackerEngine {
    pub fn render(
        &mut self,
        state: &TrackerState,
        output: &mut [f32],
        sample_rate: u32,
    ) {
        let module = match self.modules.get(state.handle as usize) {
            Some(Some(m)) => m,
            _ => return,
        };

        // Sync engine state to rollback state
        self.sync_to_state(state, module);

        // Calculate samples to render
        let samples = output.len() / 2;  // Stereo

        // Render tracker audio
        for i in 0..samples {
            let (left, right) = self.generate_sample(module, sample_rate);
            output[i * 2] += left * (state.volume as f32 / 256.0);
            output[i * 2 + 1] += right * (state.volume as f32 / 256.0);

            self.advance_tick(state, sample_rate);
        }
    }
}
```

---

## Rollback Integration

### State Synchronization

```rust
impl TrackerEngine {
    /// Sync engine to rollback state (called each frame)
    fn sync_to_state(&mut self, state: &TrackerState, module: &XmModule) {
        // Check if we need to seek
        if self.current_order != state.order_position
           || self.current_row != state.row
        {
            // Seek to correct position
            self.seek_to(module, state.order_position, state.row);

            // Replay ticks to restore channel state
            for _ in 0..state.tick {
                self.process_tick(module, state);
            }
        }

        // Restore sample position within tick
        self.tick_samples_rendered = state.tick_sample_pos;
    }

    /// Seek to a specific position (reconstructs channel state)
    fn seek_to(&mut self, module: &XmModule, order: u16, row: u16) {
        // Reset all channels
        self.reset_channels();

        // Fast-forward from song start to target position
        // (Or use cached row states if available)
        self.current_order = 0;
        self.current_row = 0;

        while self.current_order < order
              || (self.current_order == order && self.current_row < row)
        {
            self.process_row(module);
            self.advance_row(module);
        }
    }
}
```

### Optimization: Row State Caching

To minimize rollback seek time, channel states are cached at regular intervals:

**Caching Strategy:**
- Cache channel state **every 4 rows**
- Always cache at **pattern boundaries** (row 0 of each pattern)
- LRU eviction when cache exceeds **256KB**
- On rollback: seek to nearest cached state, then replay remaining rows/ticks

**Worst case seek:** 4 rows × 6 ticks × 32 channels = ~768 tick operations (fast)

```rust
/// Cache channel states for fast rollback
pub struct RowStateCache {
    /// Cached states: key = (order, row), value = channel states
    cache: HashMap<(u16, u16), [TrackerChannel; 32]>,
    max_size_bytes: usize,  // 256KB
}

impl RowStateCache {
    /// Store state at row boundary
    fn cache_row(&mut self, order: u16, row: u16, channels: &[TrackerChannel; 32]) {
        // Cache every 4 rows OR at pattern start
        if row % 4 != 0 { return; }

        // LRU eviction if cache full
        while self.current_size() >= self.max_size_bytes {
            self.evict_oldest();
        }

        self.cache.insert((order, row), channels.clone());
    }

    /// Find nearest cached state before target
    fn find_nearest(&self, order: u16, row: u16) -> Option<((u16, u16), &[TrackerChannel; 32])> {
        // Find closest (order, row) <= target
        self.cache.iter()
            .filter(|&((o, r), _)| *o < order || (*o == order && *r <= row))
            .max_by_key(|&((o, r), _)| (*o, *r))
            .map(|(k, v)| (*k, v))
    }
}
```

---

## XM Playback Implementation

### Core Playback Loop

```rust
impl TrackerEngine {
    fn process_tick(&mut self, module: &XmModule, state: &TrackerState) {
        if self.current_tick == 0 {
            // First tick of row: trigger notes and effects
            self.process_row(module);
        } else {
            // Subsequent ticks: process tick-based effects
            self.process_tick_effects(module);
        }

        // Advance tick counter
        self.current_tick += 1;
        if self.current_tick >= state.speed {
            self.current_tick = 0;
            self.advance_row(module, state);
        }
    }

    fn process_row(&mut self, module: &XmModule) {
        let pattern_idx = module.order_table[self.current_order as usize];
        let pattern = &module.patterns[pattern_idx as usize];

        for ch in 0..module.num_channels {
            let note = pattern.get_note(self.current_row, ch);
            self.process_note(ch, note, module);
        }
    }

    fn process_note(&mut self, ch: usize, note: &XmNote, module: &XmModule) {
        let channel = &mut self.channels[ch];

        // Trigger sample if note present
        if note.note != 0 && note.note != 97 {  // 97 = note off
            if let Some(instr) = module.instruments.get(note.instrument as usize) {
                channel.trigger_note(note.note, instr);
            }
        }

        // Note off
        if note.note == 97 {
            channel.note_off();
        }

        // Volume column
        if note.volume != 0 {
            self.apply_volume_column(ch, note.volume);
        }

        // Effect column
        if note.effect != 0 || note.effect_param != 0 {
            self.apply_effect(ch, note.effect, note.effect_param);
        }
    }
}
```

### Key XM Effects to Support

| Effect | Hex | Name | Priority | MVP |
|--------|-----|------|----------|-----|
| 0 | 0xy | Arpeggio | High | ✅ |
| 1 | 1xx | Portamento up | High | ✅ |
| 2 | 2xx | Portamento down | High | ✅ |
| 3 | 3xx | Tone portamento | High | ✅ |
| 4 | 4xy | Vibrato | High | ✅ |
| 5 | 5xy | Tone porta + vol slide | Medium | ✅ |
| 6 | 6xy | Vibrato + vol slide | Medium | ✅ |
| 7 | 7xy | Tremolo | Medium | ✅ |
| 8 | 8xx | Set panning | High | ✅ |
| 9 | 9xx | Sample offset | High | ✅ |
| A | Axy | Volume slide | High | ✅ |
| B | Bxx | Position jump | High | ✅ |
| C | Cxx | Set volume | High | ✅ |
| D | Dxx | Pattern break | High | ✅ |
| E | Exy | Extended effects | Mixed | ✅ | 14/15 (all except EF) |
| F | Fxx | Set speed/tempo | High | ✅ |
| G | Gxx | Set global volume | Medium | ✅ |
| H | Hxy | Global volume slide | Low | ✅ | Same as Axy for global vol |
| K | Kxx | Key off | High | ✅ |
| L | Lxx | Set envelope pos | Low | ✅ | Just set position value |
| P | Pxy | Panning slide | Low | ✅ | Same as Axy for pan |
| R | Rxy | Retrigger | Medium | ✅ |
| T | Txy | Tremor | Low | ❌ | Needs per-channel counter |
| X | Xxy | Extra fine effects | Low | ❌ | Many sub-effects |

### Extended Effects (Exy) - MVP Scope

| Effect | Hex | Name | MVP | Notes |
|--------|-----|------|-----|-------|
| E1 | E1x | Fine porta up | ✅ | Same as 1xx, 4x finer |
| E2 | E2x | Fine porta down | ✅ | Same as 2xx, 4x finer |
| E3 | E3x | Glissando control | ✅ | Quantize porta to semitones |
| E4 | E4x | Vibrato waveform | ✅ | Lookup table selector |
| E5 | E5x | Set finetune | ✅ | Per-channel frequency offset |
| E6 | E6x | Pattern loop | ✅ | Loop section of pattern |
| E7 | E7x | Tremolo waveform | ✅ | Lookup table selector |
| E8 | E8x | Set panning (coarse) | ✅ | Direct pan assignment |
| E9 | E9x | Retrigger note | ✅ | Re-trigger every N ticks |
| EA | EAx | Fine volume slide up | ✅ | Same as Axy, 4x finer |
| EB | EBx | Fine volume slide down | ✅ | Same as Axy, 4x finer |
| EC | ECx | Note cut | ✅ | Cut volume at tick N |
| ED | EDx | Note delay | ✅ | Delay note trigger N ticks |
| EE | EEx | Pattern delay | ✅ | Repeat row N times |
| EF | EFx | Invert loop | ❌ | Obsolete Amiga quirk, skip |

**MVP covers:** Full Exy support (14/15 effects). Only EF (invert loop) is omitted as it's an obsolete Amiga hardware quirk with no practical use.

---

## Memory & Performance

### Memory Budget

| Component | Size | Notes |
|-----------|------|-------|
| TrackerState (rollback) | 64 bytes | Per-frame snapshot |
| TrackerEngine | ~4 KB | Channels + buffers |
| XmModule (patterns only) | 5KB-50KB | Samples loaded separately from ROM |
| Row state cache | ~256 KB | LRU cache, every 4 rows |
| **Total per tracker** | **~270KB-310KB** | Much smaller than embedded-sample approach |

**Note:** Sample memory is shared with SFX via ROM data pack. A 3-song soundtrack reusing samples might only need 500KB total instead of 1.5MB with embedded samples.

### Performance

- **Target:** <0.5ms per frame for tracker rendering
- **32 channels × 735 samples/frame** @ 60fps = 23,520 sample-channel ops
- Linear interpolation for sample playback
- Envelope processing per-tick (not per-sample)

### ROM Space

With the hybrid approach (patterns + ROM samples):

**Pattern data only:**
- Simple chiptune: 2-10 KB
- Full song: 10-30 KB
- Complex arrangement: 30-50 KB

**Samples (shared across trackers):**
- Typical sample set: 200-500 KB
- Reused for SFX, no duplication

**Example ROM budget:**
```
3 songs × 20KB patterns = 60KB
Shared sample set       = 300KB
Total music             = 360KB
```

**Comparison to PCM:**
- 3-minute song @ 22kHz mono: ~7.9 MB (PCM) vs ~320 KB (tracker + samples)
- **~25x smaller!** (with sample reuse, even better)

---

## Edge Cases

### Rollback During Pattern Jump

**Scenario:** Game rolls back across a Bxx (position jump) effect

**Solution:**
- TrackerState stores target position, not effect
- On rollback, seek directly to stored position
- Deterministic regardless of how we got there

### Speed/Tempo Changes

**Scenario:** Fxx effect changes speed mid-song

**Solution:**
- speed and bpm stored in TrackerState
- On rollback, restored from snapshot
- No reconstruction needed

### Sample Loops

**Scenario:** Sample has pingpong loop, rollback mid-loop

**Solution:**
- Sample position is per-channel, reconstructed from row
- Pingpong direction determined by position within loop
- Deterministic because we replay ticks from row start

### Empty Patterns

**Scenario:** Pattern contains all empty notes

**Solution:**
- Channels continue playing until note-off
- Empty pattern = no new triggers, effects still process
- Standard XM behavior

### Invalid XM Files

**Validation at load time:**
```rust
fn validate_xm(data: &[u8]) -> Result<(), XmError> {
    // Check magic: "Extended Module: "
    if &data[0..17] != b"Extended Module: " {
        return Err(XmError::InvalidMagic);
    }

    // Check version (should be 0x0104)
    let version = u16::from_le_bytes([data[58], data[59]]);
    if version != 0x0104 {
        return Err(XmError::UnsupportedVersion);
    }

    // Validate channel count
    let channels = u16::from_le_bytes([data[68], data[69]]);
    if channels > 32 {
        return Err(XmError::TooManyChannels);
    }

    // ... more validation
    Ok(())
}
```

---

## Debug Panel Integration

When a tracker is playing, the F3 debug panel displays tracker state:

**Position Info:**
```
Tracker: main_theme
Position: Order 5/32 | Row 48/64 | Tick 3/6
Tempo: Speed 6 | BPM 125
```

**Channel Activity:**
Visual bars showing which channels are active and their current volume levels. Useful for debugging music sync and identifying silent channels.

```
Channels:
 0 ████████░░  1 ██████░░░░  2 ░░░░░░░░░░  3 ████░░░░░░
 4 ██░░░░░░░░  5 ░░░░░░░░░░  6 ██████████  7 ████████░░
...
```

**Implementation:**
```rust
// emberware-z/src/debug.rs
impl TrackerEngine {
    pub fn debug_stats(&self, state: &TrackerState) -> Vec<DebugStat> {
        if state.handle == 0 {
            return vec![];
        }

        vec![
            DebugStat::new("Tracker", &self.current_tracker_name()),
            DebugStat::new("Position", &format!(
                "Order {}/{} | Row {}/64 | Tick {}/{}",
                state.order_position, self.song_length(),
                state.row, state.tick, state.speed
            )),
            DebugStat::new("Tempo", &format!(
                "Speed {} | BPM {}",
                state.speed, state.bpm
            )),
            // Channel activity as visual bars
            DebugStat::new("Channels", &self.channel_activity_string()),
        ]
    }
}
```

---

## Implementation Plan

### Phase 1: XM Parser (2 days)

**Files:**
- `z-common/src/formats/tracker.rs` (new) — XM file parser

```rust
pub struct XmModule {
    pub name: String,
    pub num_channels: u8,
    pub num_patterns: u16,
    pub num_instruments: u16,
    pub song_length: u16,
    pub restart_position: u16,
    pub default_speed: u16,
    pub default_bpm: u16,
    pub order_table: Vec<u8>,
    pub patterns: Vec<XmPattern>,
    pub instruments: Vec<XmInstrument>,
}

pub fn parse_xm(data: &[u8]) -> Result<XmModule, XmError>;
```

### Phase 2: Asset Pipeline (1 day)

**Files:**
- `z-common/src/formats/z_data_pack.rs` — Add PackedTracker
- `tools/ember-cli/src/pack.rs` — Load trackers into data pack
- `tools/ember-cli/src/manifest.rs` — Parse [[assets.trackers]]

### Phase 3: Tracker Engine (3 days)

**Files:**
- `emberware-z/src/audio/tracker.rs` (new) — TrackerEngine
- `emberware-z/src/audio/mod.rs` — Re-export

Core implementation:
- Sample playback with interpolation
- Effect processing
- Envelope processing
- Channel mixing

### Phase 4: Rollback State (1 day)

**Files:**
- `emberware-z/src/state/rollback_state.rs` — Add TrackerState
- `emberware-z/src/audio/tracker.rs` — sync_to_state()

### Phase 5: FFI Functions (1 day)

**Files:**
- `emberware-z/src/ffi/audio.rs` — Add tracker FFI functions

```rust
fn rom_tracker(id_ptr: u32, id_len: u32) -> u32;
fn tracker_play(handle: u32, volume: f32, looping: u32);
fn tracker_stop();
fn tracker_set_volume(volume: f32);
fn tracker_jump(order: u32, row: u32);
fn tracker_position() -> u32;
// ...
```

### Phase 6: Integration (1 day)

**Files:**
- `emberware-z/src/audio.rs` — Integrate into generate_audio_frame
- `emberware-z/src/player.rs` — Pass tracker engine to audio gen
- `emberware-z/src/lib.rs` — Initialize tracker engine

### Phase 7: Testing & Examples (2 days)

**Files:**
- `examples/tracker-demo/` (new) — Example game with tracker music
- Unit tests for XM parser
- Integration tests for playback

---

## Files to Modify

| File | Changes |
|------|---------|
| `z-common/src/formats/mod.rs` | Add tracker module |
| `z-common/src/formats/tracker.rs` | New: XM parser |
| `z-common/src/formats/z_data_pack.rs` | Add PackedTracker |
| `emberware-z/src/audio/mod.rs` | Add tracker submodule |
| `emberware-z/src/audio/tracker.rs` | New: TrackerEngine |
| `emberware-z/src/audio.rs` | Integrate tracker mixing |
| `emberware-z/src/state/rollback_state.rs` | Add TrackerState |
| `emberware-z/src/ffi/audio.rs` | Add tracker FFI |
| `emberware-z/src/player.rs` | Initialize and pass engine |
| `tools/ember-cli/src/pack.rs` | Load tracker assets |
| `tools/ember-cli/src/manifest.rs` | Parse assets.trackers |

---

## Estimated Effort

| Component | Effort |
|-----------|--------|
| XM parser | 2 days |
| Asset pipeline | 1 day |
| Tracker engine | 3 days |
| Rollback integration | 1 day |
| FFI functions | 1 day |
| Audio integration | 1 day |
| Testing & examples | 2 days |
| **Total** | **~11 days** |

---

## Future Enhancements

**Post-MVP (prioritized):**
1. **Multi-sample instruments:** Note→sample mapping for realistic instruments and drum kits
2. **Remaining effects:** Tremor (Txy), extra fine effects (Xxy)
3. **Sample interpolation options:** None (authentic crunch) / cubic (smoother)

**Longer-term:**
4. **S3M/IT support:** Additional format parsers
5. **Subsong support:** Multiple songs in one module
6. **Channel muting:** Mute specific channels for layered music
7. **Real-time tempo sync:** Sync to game events
8. **Waveform visualization:** FFT for audio visualizers
9. **Sample streaming:** For very large samples (>1MB)

---

## References

- [XM File Format Specification](https://github.com/milkytracker/MilkyTracker/wiki/XM-file-format)
- [FastTracker 2 Documentation](https://www.un4seen.com/forum/?topic=3422.0)
- [libxm](https://github.com/Artefact2/libxm) — C reference implementation
- [OpenMPT](https://openmpt.org/) — Modern tracker with XM export
- [MilkyTracker](https://milkytracker.org/) — Cross-platform XM tracker
