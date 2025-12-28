# XM Tracker Playback Spec Compliance Analysis

## Summary

**Scope**: Core playback mechanics only (effects excluded per user request)

**Files to Modify**:
- [tracker.rs](nethercore-zx/src/tracker.rs) - Playback engine

**Reference**:
- [module.rs](nether-xm/src/module.rs) - XM data structures with envelope support

---

## Fixes Required

### 1. Volume Envelopes (CRITICAL)
**Location**: `render_sample()` / `render_sample_and_advance()` around line 1014

Add after getting channel volume:
```rust
// Apply volume envelope
let mut vol = channel.volume;
if let Some(ref env) = instrument.volume_envelope {
    if env.enabled {
        let env_val = env.value_at(channel.volume_envelope_pos) as f32 / 64.0;
        vol *= env_val;

        // Advance envelope (unless sustained and not key-off)
        if !channel.key_off {
            if let Some(sus_tick) = env.sustain_tick() {
                if channel.volume_envelope_pos < sus_tick {
                    channel.volume_envelope_pos += 1;
                }
            } else {
                channel.volume_envelope_pos += 1;
            }
        } else {
            channel.volume_envelope_pos += 1;
        }

        // Handle envelope loop
        if let Some((loop_start, loop_end)) = env.loop_range() {
            if channel.volume_envelope_pos >= loop_end {
                channel.volume_envelope_pos = loop_start;
            }
        }
    }
}
```

### 2. Volume Fadeout (CRITICAL)
**Location**: After envelope processing

```rust
// Apply fadeout after key-off
if channel.key_off && channel.volume_fadeout > 0 {
    vol *= channel.volume_fadeout as f32 / 65535.0;
    channel.volume_fadeout = channel.volume_fadeout.saturating_sub(instrument.volume_fadeout);
}
```

### 3. Sample Relative Note (CRITICAL)
**Location**: `note_to_period()` calls in `trigger_note()` and `process_note_internal()`

Change from:
```rust
channel.base_period = note_to_period(note.note, finetune);
```

To:
```rust
let effective_note = (note.note as i16 + instrument.sample_relative_note as i16)
    .clamp(1, 96) as u8;
channel.base_period = note_to_period(effective_note, finetune);
```

### 4. Panning Envelopes (MODERATE)
Similar to volume envelopes, add panning envelope processing after panning is read.

---

## CORRECT Implementations

### 1. Note Numbering
- Notes 1-96 = C-0 to B-7
- Note 97 = Key-Off
- Note 0 = empty

### 2. Linear Period Calculation
```rust
let period = 10 * 12 * 16 * 4 - n * 16 * 4 - ft / 2;
// = 7680 - (note-1)*64 - finetune/2
```
Matches XM spec: `Period = 7680 - (Note * 64) - (FineTune / 2)`

### 3. Period-to-Frequency Conversion
```rust
// Frequency = 8363 * 2^((4608 - Period) / 768)
```
Uses 768-entry LUT for 2^(i/768) - correct optimization

### 4. Timing Calculation
```rust
(sample_rate * 5 / 2) / bpm
// = sample_rate * 2.5 / bpm
```
Matches XM spec: 1 tick = 2.5/BPM seconds

### 5. Default Speed/BPM
- DEFAULT_SPEED = 6
- DEFAULT_BPM = 125

### 6. Sample Loop Types
- Loop type 0 = none
- Loop type 1 = forward
- Loop type 2 = ping-pong

### 7. Linear Interpolation
Sample playback uses lerp between adjacent samples

### 8. Ping-Pong Loop Direction Reversal
Correctly reverses `sample_direction` at loop boundaries

### 9. Volume Column (Set Volume Only)
0x10-0x50 correctly decoded as volume 0-64

### 10. Panning Formula
Uses equal-power panning (cos/sin formula)

---

## Issues Found

### CRITICAL: Volume/Panning Envelopes Not Applied

**Location**: [tracker.rs:1014-1015](nethercore-zx/src/tracker.rs#L1014-L1015)

The code has envelope structures but **never applies them**:
```rust
// Apply volume (with envelope if present)
let vol = channel.volume * self.global_volume;
```

**XM Spec**: Envelope values must modulate volume/panning every frame. The `XmEnvelope::value_at()` method exists but is never called.

**Impact**: All envelope-based instruments sound wrong (swells, fades, auto-vibrato).

---

### CRITICAL: Volume Fadeout Not Applied

**Location**: `volume_fadeout` field is set but never decremented/applied

**XM Spec**: After key-off:
- Decrement fadeout by `instrument.volume_fadeout` each tick
- Multiply final volume by `fadeout / 65536`

**Impact**: Notes with fadeout play at full volume forever after key-off.

---

### CRITICAL: Sample Relative Note Ignored

**Location**: [tracker.rs:624](nethercore-zx/src/tracker.rs#L624)

```rust
channel.base_period = note_to_period(note.note, finetune);
```

**Missing**: `sample_relative_note` offset not applied.

**XM Spec**: `effective_note = note + sample_relative_note`

**Impact**: Instruments tuned to different base notes play at wrong pitch.

---

### DEFERRED: Volume Column Effects (0x60-0xFF)
Being handled with other effects separately.

---

### MINOR: Note-to-Sample Keymap Ignored

**Location**: Instrument loading assumes 1 sample per instrument

**XM Spec**: Each instrument has a 96-byte table mapping notes to samples. A single instrument can have 16 different samples for different note ranges.

**Impact**: Multi-sample instruments only play one sample.

---

### MINOR: Auto-Vibrato Not Applied

**Location**: `XmInstrument` has `vibrato_type/sweep/depth/rate` but not used

**XM Spec**: Instruments can have automatic vibrato applied to all notes.

---

### MINOR: Instrument Panning Not Applied

XM instruments have per-instrument default panning. Currently using channel default.

---

## Implementation Order

1. **Sample Relative Note** - Simple one-liner, fix pitch first
2. **Volume Envelopes** - Most impactful for sound quality
3. **Volume Fadeout** - Completes the key-off behavior
4. **Panning Envelopes** - Same pattern as volume envelopes

## Deferred (Future Work)

- Volume Column Effects (0x60+) - handled with other effects
- Auto-Vibrato - low priority
- Multi-Sample Keymap - depends on content needs

---

# PERFORMANCE ISSUES (Causing Choppy Audio)

## CRITICAL: Trig Functions Called Per-Sample (TWO LOCATIONS)

**Location 1**: [audio.rs:372-374](nethercore-zx/src/audio.rs#L372-L374) - SFX channels
**Location 2**: [tracker.rs:1275-1280](nethercore-zx/src/tracker.rs#L1275-L1280) - Tracker channels

```rust
fn apply_pan/apply_channel_pan(sample: f32, pan: f32, ...) -> (f32, f32) {
    let angle = (pan + 1.0) * 0.25 * std::f32::consts::PI;
    let left_gain = angle.cos();  // EXPENSIVE
    let right_gain = angle.sin(); // EXPENSIVE
```

**Impact**: Both SFX and tracker call cos()/sin() per-sample. With 16 SFX channels + 32 tracker channels at 44.1kHz, this could be millions of trig calls/second.

**Fix**: Use existing `SINE_LUT` for fast panning lookup:

```rust
/// Fast panning using existing SINE_LUT (reuse from vibrato/tremolo)
/// Maps pan [-1, 1] to left/right gains using quarter-sine lookup
#[inline]
fn fast_pan_gains(pan: f32) -> (f32, f32) {
    // Map pan [-1, 1] to index [0, 15]
    let idx = ((pan + 1.0) * 7.5) as usize;
    let idx = idx.min(15);

    // sin for right, cos (reverse lookup) for left
    let right = SINE_LUT[idx] as f32 / 127.0;
    let left = SINE_LUT[15 - idx] as f32 / 127.0;

    (left, right)
}
```

This replaces ~100 CPU cycles (cos+sin) with ~5 cycles (2 array lookups + math).

---

## CRITICAL: TrackerEngine Created Every Frame

**Location**: [audio.rs:196-204](nethercore-zx/src/audio.rs#L196-L204)

```rust
pub fn generate_audio_frame(...) {
    let mut tracker_state = TrackerState::default();
    generate_audio_frame_with_tracker(
        ...
        &mut TrackerEngine::new(),  // ALLOCATES HashMap EVERY FRAME
        ...
    );
}
```

**Impact**: Creates a new TrackerEngine with HashMap allocation 60 times per second.

**Fix**: This function should use a cached/static engine or require one passed in.

---

## CRITICAL: Allocation in push_samples()

**Location**: [audio.rs:449-452](nethercore-zx/src/audio.rs#L449-L452)

```rust
if (self.master_volume - 1.0).abs() < f32::EPSILON {
    output.push_samples(samples);
} else {
    let scaled: Vec<f32> = samples.iter().map(|s| s * self.master_volume).collect();
    output.push_samples(&scaled);
}
```

**Impact**: Allocates ~6KB Vec (1470 floats) every frame when master_volume != 1.0.

**Fix**: Scale in-place or use a pre-allocated buffer.

---

## HIGH: soft_clip() Uses tanh() Per-Sample

**Location**: [audio.rs:385-392](nethercore-zx/src/audio.rs#L385-L392)

```rust
fn soft_clip(x: f32) -> f32 {
    if x.abs() <= 1.0 {
        x
    } else {
        x.signum() * (1.0 + (x.abs() - 1.0).tanh())  // tanh is expensive
    }
}
```

**Impact**: Called 88,200 times/second. `tanh()` is slow even when usually taking the fast path.

**Fix**: Use polynomial approximation or lookup table for the clipping region.

---

## HIGH: Module Lookup Inside Per-Sample Loop

**Location**: [tracker.rs:1056-1063](nethercore-zx/src/tracker.rs#L1056-L1063)

```rust
pub fn render_sample_and_advance(...) -> (f32, f32) {
    ...
    let module = match self.modules
        .get(state.handle as usize)
        .and_then(|m| m.as_ref())  // Called 44,100 times/sec
```

**Impact**: Repeated Option unwrapping and bounds checking per sample.

**Fix**: Cache the module reference at frame start, not per-sample.

---

## MODERATE: HashMap Linear Search in RowStateCache

**Location**: [tracker.rs:289-300](nethercore-zx/src/tracker.rs#L289-L300)

```rust
fn find_nearest(&self, target_order: u16, target_row: u16) -> Option<...> {
    self.cache.iter()
        .filter(|((order, row), _)| ...)
        .max_by_key(...)
}
```

**Impact**: O(n) iteration through up to 256 cache entries during rollback seeks.

**Fix**: Use BTreeMap with range queries or sorted Vec with binary search.

---

## Recommended Performance Fixes (Priority Order)

1. **Use SINE_LUT for panning** - Replace cos()/sin() with `fast_pan_gains()` using existing LUT
   - Apply to both `apply_pan()` in audio.rs and `apply_channel_pan()` in tracker.rs
   - ~20x speedup for panning calculations
2. **Remove per-frame TrackerEngine allocation** - Fix `generate_audio_frame()`
3. **Pre-allocate master volume buffer** - Avoid allocation in `push_samples()`
4. **Cache module reference per-frame** - Move lookup outside sample loop
5. **Optimize soft_clip** - Use fast polynomial or skip when not clipping
6. **Use BTreeMap for RowStateCache** - O(log n) instead of O(n) seeks
