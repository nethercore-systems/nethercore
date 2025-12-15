# Per-Frame Audio Generation Specification

**Status:** REJECTED - in favor of combined-refactor-plan.md
**Author:** Claude
**Last Updated:** December 2025

---

## Executive Summary

This document proposes replacing Emberware's rodio-based audio system with a per-frame audio generation approach using [cpal](https://github.com/RustAudio/cpal). This architecture makes audio state deterministic and compatible with GGRS rollback netcode.

**Key benefits:**
- Audio state (playhead positions, volumes, pans) becomes part of rollback state - saved/restored via POD bytemuck cast
- No cross-thread atomic synchronization needed for dynamic parameter updates
- Predictable latency (exactly one frame of audio delay)
- Simpler mental model: "generate audio samples just like you generate graphics"

**Trade-offs:**
- More implementation work upfront
- Need to manage audio device directly instead of relying on rodio's abstractions
- Ring buffer coordination between game thread and audio callback thread
- Breaking change to `Audio` trait in core

---

## Problem Statement

### Current Architecture Issues

The current rodio-based system has fundamental architectural issues for a rollback netcode game:

1. **Audio state is external to game state**
   - Playhead positions, looping state, and channel states live in the audio server thread
   - Cannot be rolled back when GGRS rewinds game state
   - Current workaround: `set_rollback_mode()` mutes audio during replay, but this can cause audio artifacts

2. **Complex thread synchronization for dynamic updates**
   - Recently fixed a bug where `repeat_infinite()` cloning broke dynamic pan updates
   - Required `SharedPan(Arc<AtomicU32>)` pattern for real-time pan updates
   - Still fragile - depends on rodio internals we don't control

3. **Unpredictable latency**
   - Rodio's internal buffering creates variable latency
   - Makes tight audio-visual synchronization difficult

### The Root Cause

The fundamental issue is that rodio's streaming model doesn't match Emberware's deterministic game loop. Audio backends like rodio assume continuous streaming, but rollback netcode requires discrete, deterministic frames.

---

## Proposed Solution: Per-Frame Audio Generation

### Core Concept

Instead of streaming audio to a backend, the game generates exactly one frame's worth of audio samples during each frame, then pushes them to a ring buffer consumed by the cpal audio callback.

```
┌─────────────────────────────────────────────────────────────┐
│                      Game Thread                            │
│  ┌──────────┐    ┌──────────┐    ┌────────────────────┐    │
│  │ update() │ →  │ render() │ →  │ generate_audio()   │    │
│  │ game     │    │ graphics │    │ samples per frame  │    │
│  │ logic    │    │          │    │ (tick-rate aware)  │    │
│  └──────────┘    └──────────┘    └─────────┬──────────┘    │
│                                            │               │
│                                            ↓               │
│                              ┌─────────────────────────┐   │
│                              │     Ring Buffer         │   │
│                              │  (lock-free, ~50ms)     │   │
│                              └───────────┬─────────────┘   │
│                                          │                 │
└──────────────────────────────────────────│─────────────────┘
                                           │
                                           ↓
┌──────────────────────────────────────────────────────────────┐
│                     Audio Thread (cpal)                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  data_callback(&mut [f32])                              │ │
│  │  - Read from ring buffer (lock-free)                    │ │
│  │  - Fill output buffer                                   │ │
│  │  - If underrun: output silence                          │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### Why This Works Better

1. **Audio state is explicit rollback state**
   - Playhead positions, volumes, pans stored in `ZRollbackState`
   - Saved/restored via zero-copy POD cast (bytemuck) during GGRS snapshots
   - No special handling needed - same pattern as other deterministic state

2. **No cross-thread synchronization for parameters**
   - Volume/pan calculated during `generate_audio()` on game thread
   - No atomics, no shared state with audio thread
   - Ring buffer is the only synchronization point (lock-free)

3. **Deterministic behavior**
   - Same inputs → same audio samples generated
   - Rollback replays produce identical audio
   - Only the final "real" audio makes it to the ring buffer

---

## Architecture: WasmGameContext

### Renaming GameStateWithConsole

The current `GameStateWithConsole` struct is renamed to `WasmGameContext` for clarity, and gains a `rollback` field:

```rust
/// Context for WASM game execution
///
/// Contains all state accessible to WASM FFI functions.
pub struct WasmGameContext<I: ConsoleInput, S, R: ConsoleRollbackState> {
    /// Core WASM game state (memory snapshots from this)
    pub game: GameState,

    /// Console-specific per-frame FFI state (NOT rolled back)
    /// Rebuilt each frame from FFI calls, cleared after render.
    pub ffi: S,  // e.g., ZFFIState

    /// Console-specific rollback state (IS rolled back)
    /// Saved/restored alongside WASM memory during rollback.
    pub rollback: R,  // e.g., ZRollbackState
}
```

This allows FFI functions to access both per-frame state and rollback state:

```rust
fn channel_play(
    mut caller: Caller<'_, WasmGameContext<ZInput, ZFFIState, ZRollbackState>>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel as usize];
    ch.sound = sound;
    ch.position = 0;
    ch.volume = volume;
    ch.pan = pan;
    ch.looping = looping;  // 0 or 1
}
```

### Key Insight: Host-Side vs WASM Memory

**CRITICAL:** Audio playback state lives on the HOST side (in `ZRollbackState`), NOT in WASM linear memory. This is the same pattern as other console state - `ZFFIState` (now `ffi`) is host-side and rebuilt each frame from FFI calls.

For rollback to work correctly, audio playback state must be:
1. Stored in a dedicated rollback state struct (`ZRollbackState`)
2. POD (Plain Old Data) for zero-copy serialization via bytemuck
3. Included in `GameStateSnapshot` alongside WASM memory snapshots

### ConsoleRollbackState Trait (POD-based)

Add to `core/src/console.rs`:

```rust
use bytemuck::{Pod, Zeroable};

/// Console-specific rollback state that needs explicit serialization
///
/// This is for HOST-SIDE state that affects deterministic behavior but
/// lives outside WASM linear memory. Must be POD for zero-copy save/load.
///
/// Examples:
/// - Audio playback state (playhead positions, channel volumes)
/// - Any other console-specific deterministic state
pub trait ConsoleRollbackState: Pod + Zeroable + Default + Send + 'static {}

/// Dummy implementation for consoles with no extra rollback state
unsafe impl Pod for () {}
unsafe impl Zeroable for () {}
impl ConsoleRollbackState for () {}
```

### Console Trait Update

```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;
    type State: Send + Default + 'static;  // Per-frame FFI state (ZFFIState)
    type RollbackState: ConsoleRollbackState;  // NEW: Rollback state (ZRollbackState)
    type ResourceManager: ConsoleResourceManager;

    // ... existing methods ...
}
```

### GameStateSnapshot Update

```rust
pub struct GameStateSnapshot {
    /// Serialized WASM game state (linear memory)
    pub data: Vec<u8>,

    /// Serialized console rollback state (host-side, POD bytes)
    pub console_data: Vec<u8>,

    /// Combined checksum for desync detection
    pub checksum: u64,

    /// Frame number
    pub frame: i32,
}

impl GameStateSnapshot {
    /// Save console rollback state (zero-copy via bytemuck)
    pub fn save_console<R: ConsoleRollbackState>(state: &R) -> Vec<u8> {
        bytemuck::bytes_of(state).to_vec()
    }

    /// Load console rollback state (zero-copy via bytemuck)
    pub fn load_console<R: ConsoleRollbackState>(data: &[u8]) -> R {
        *bytemuck::from_bytes(data)
    }
}
```

---

## Design Details

### Audio Playback State (POD)

Audio playback state lives in `ZRollbackState`. All types are POD for zero-copy serialization:

```rust
use bytemuck::{Pod, Zeroable};

/// State for a single audio channel (20 bytes, POD)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ChannelState {
    /// Sound handle being played (0 = silent)
    pub sound: u32,

    /// Current playhead position in samples
    pub position: u32,

    /// Is this channel looping? (0 = no, 1 = yes)
    /// Using u32 instead of bool for POD compatibility
    pub looping: u32,

    /// Current volume (0.0 to 1.0)
    pub volume: f32,

    /// Current pan (-1.0 left to 1.0 right)
    pub pan: f32,
}

/// Audio playback state - included in rollback snapshots (340 bytes)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct AudioPlaybackState {
    /// 16 effect channels (320 bytes)
    pub channels: [ChannelState; 16],

    /// Dedicated music channel (20 bytes)
    pub music: ChannelState,
}

/// Emberware Z rollback state (host-side deterministic state)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ZRollbackState {
    /// Audio playback state
    pub audio: AudioPlaybackState,
}

// POD trait impl - enables zero-copy serialization
unsafe impl Pod for ZRollbackState {}
unsafe impl Zeroable for ZRollbackState {}
impl ConsoleRollbackState for ZRollbackState {}
```

**Size:** 340 bytes total - trivial to snapshot/restore via `bytemuck::bytes_of()`.

### Variable Tick Rate Support

Emberware Z supports multiple tick rates: **24, 30, 60, 120 fps**

Samples per frame varies based on tick rate:
- 24 fps: 22050 / 24 = 918.75 → 919 samples mono, 1838 stereo
- 30 fps: 22050 / 30 = 735 samples mono, 1470 stereo
- 60 fps: 22050 / 60 = 367.5 → 368 samples mono, 736 stereo
- 120 fps: 22050 / 120 = 183.75 → 184 samples mono, 368 stereo

```rust
/// Calculate samples per frame based on tick rate
pub fn samples_per_frame(tick_rate: u32, sample_rate: u32) -> usize {
    // Round up - any slight overgeneration is absorbed by ring buffer
    ((sample_rate + tick_rate - 1) / tick_rate) as usize
}
```

### Sample Rate Handling

**Strategy:** Request 22050Hz, fall back to device native rate if unsupported.

```rust
const PREFERRED_SAMPLE_RATE: u32 = 22050;

impl AudioOutput {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No output device")?;

        // Try preferred rate first
        let sample_rate = if device_supports_rate(&device, PREFERRED_SAMPLE_RATE) {
            PREFERRED_SAMPLE_RATE
        } else {
            // Fall back to device's default/preferred rate
            let default_config = device.default_output_config()
                .map_err(|e| e.to_string())?;
            default_config.sample_rate().0
        };

        log::info!("Audio sample rate: {}Hz", sample_rate);

        // Store sample_rate for generate_audio calculations
        // ...
    }
}

fn device_supports_rate(device: &cpal::Device, rate: u32) -> bool {
    device.supported_output_configs()
        .map(|configs| {
            configs.any(|c| {
                c.min_sample_rate().0 <= rate && c.max_sample_rate().0 >= rate
            })
        })
        .unwrap_or(false)
}
```

If the device rate differs from 22050Hz, audio will play at slightly different pitch. This is acceptable for a retro-aesthetic console.

### Per-Frame Sample Generation

Each frame, after `update()` and `render()`, generate audio samples:

```rust
/// Generate one frame's worth of audio samples
/// Called from game loop after render()
pub fn generate_audio_frame(
    playback_state: &mut AudioPlaybackState,
    sounds: &[Option<Sound>],
    tick_rate: u32,
    sample_rate: u32,
    output: &mut Vec<f32>,  // Stereo interleaved
) {
    let samples_mono = samples_per_frame(tick_rate, sample_rate);
    let samples_stereo = samples_mono * 2;

    output.clear();
    output.resize(samples_stereo, 0.0);

    // Mix all active effect channels
    for channel in &mut playback_state.channels {
        mix_channel(channel, sounds, samples_mono, output);
    }

    // Mix music channel (same system as SFX)
    mix_channel(&mut playback_state.music, sounds, samples_mono, output);
}

fn mix_channel(
    channel: &mut ChannelState,
    sounds: &[Option<Sound>],
    samples_mono: usize,
    output: &mut [f32],
) {
    if channel.sound == 0 {
        return;  // Channel not playing
    }

    let Some(sound) = sounds.get(channel.sound as usize).and_then(|s| s.as_ref()) else {
        return;
    };

    // Calculate equal-power pan gains
    let (left_gain, right_gain) = equal_power_pan(channel.pan);
    let looping = channel.looping != 0;

    for i in 0..samples_mono {
        let sample = if (channel.position as usize) < sound.data.len() {
            let s = sound.data[channel.position as usize] as f32 / 32768.0;
            channel.position += 1;
            s * channel.volume
        } else if looping {
            channel.position = 0;
            let s = sound.data[0] as f32 / 32768.0;
            channel.position = 1;
            s * channel.volume
        } else {
            channel.sound = 0;  // Stop channel
            0.0
        };

        // Mix into stereo output (interleaved L/R)
        output[i * 2] += sample * left_gain;
        output[i * 2 + 1] += sample * right_gain;
    }
}

/// Equal-power panning (constant perceived loudness)
fn equal_power_pan(pan: f32) -> (f32, f32) {
    // pan: -1.0 (full left) to 1.0 (full right)
    let angle = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    (angle.cos(), angle.sin())
}
```

### Ring Buffer (Lock-Free)

A lock-free ring buffer bridges the game thread and audio callback.

**IMPORTANT:** The audio callback runs in a real-time thread. Never use `Mutex` or any blocking synchronization in the callback - this causes priority inversion and audio glitches.

**Dependency:** `ringbuf = "0.4"` (latest stable version)

```rust
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

pub struct AudioOutput {
    /// Producer for game thread to push samples
    producer: ringbuf::HeapProd<f32>,

    /// cpal stream handle (keeps stream alive)
    _stream: cpal::Stream,

    /// Current sample rate (may differ from preferred)
    sample_rate: u32,
}

impl AudioOutput {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No output device")?;

        // Determine sample rate (prefer 22050Hz)
        let sample_rate = determine_sample_rate(&device)?;

        // Create ring buffer: ~50ms of audio
        let buffer_samples = (sample_rate as usize * 2) / 20;  // stereo, 50ms
        let rb = HeapRb::<f32>::new(buffer_samples);
        let (producer, mut consumer) = rb.split();

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(512),
        };

        // Build stream with lock-free consumer access
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                // LOCK-FREE: consumer.pop() is wait-free
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
            },
            |err| log::error!("Audio stream error: {}", err),
            None,
        ).map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        Ok(Self {
            producer,
            _stream: stream,
            sample_rate,
        })
    }

    /// Push one frame of audio samples to the ring buffer
    pub fn push_frame(&mut self, samples: &[f32]) {
        for &sample in samples {
            // If buffer is full, drop sample (better than blocking)
            let _ = self.producer.try_push(sample);
        }
    }

    /// Get current sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check buffer health (0.0 = empty, 1.0 = full)
    pub fn buffer_health(&self) -> f32 {
        let occupied = self.producer.capacity() - self.producer.vacant_len();
        occupied as f32 / self.producer.capacity() as f32
    }
}
```

### Integration with Game Loop

In `game_session.rs`:

```rust
// After render(), generate audio (only on confirmed frames)
if !session.is_rolling_back() {
    let tick_rate = session.specs().tick_rates[session.tick_rate_index()];
    let sample_rate = session.audio_output.sample_rate();

    // Get rollback state and sounds from resource manager
    let ctx = session.runtime.context_mut();
    let sounds = session.resource_manager.sounds();

    // Generate audio from playback state
    let mut audio_buffer = Vec::new();
    generate_audio_frame(
        &mut ctx.rollback.audio,
        sounds,
        tick_rate,
        sample_rate,
        &mut audio_buffer,
    );

    // Push to audio output
    session.audio_output.push_frame(&audio_buffer);
}
```

### Rollback Handling

With this architecture, rollback works correctly:

1. **During rollback replay:**
   - `GameStateSnapshot` restored (includes both WASM memory AND `ZRollbackState` via bytemuck)
   - `update()` runs with old inputs
   - `is_rolling_back()` returns true → skip audio generation
   - Audio callback continues playing from ring buffer (existing samples)

2. **After rollback completes:**
   - Game state is now at the "real" current frame
   - `is_rolling_back()` returns false
   - `generate_audio_frame()` produces the correct audio
   - Push to ring buffer → audio callback plays it
   - No gaps, no duplicate sounds

The key insight: audio playback state rolls back with game state via zero-copy POD cast, but we only send audio to the device on confirmed frames.

---

## Sound Loading

### Init-Only Constraint

Sound loading happens **only during `init()`**, similar to other resources (textures, meshes). This ensures:
- All sounds loaded before first frame
- No allocation during gameplay
- Deterministic initialization

```rust
// Sound loading FFI functions check init guard
fn load_sound_impl(ctx: &mut WasmGameContext<...>, data_ptr: u32, len: u32) -> u32 {
    if !ctx.ffi.in_init_phase {
        log::error!("load_sound() called outside init() - ignored");
        return 0;
    }
    // ... load sound via resource manager ...
}
```

### Sound Storage (ZResourceManager)

Loaded sounds are stored in `ZResourceManager`, consistent with other resources (textures, meshes, animations):

```rust
/// Loaded sound data (PCM samples)
pub struct Sound {
    /// Mono 16-bit PCM samples at 22050Hz
    pub data: Vec<i16>,
}

impl ZResourceManager {
    /// Loaded sounds (handle → Sound)
    sounds: Vec<Option<Sound>>,

    /// Get sounds slice for audio generation
    pub fn sounds(&self) -> &[Option<Sound>] {
        &self.sounds
    }

    /// Load sound from ROM or procedurally generated data
    pub fn load_sound(&mut self, data: Vec<i16>) -> u32 {
        let handle = self.sounds.len() as u32;
        self.sounds.push(Some(Sound { data }));
        handle
    }
}
```

Sounds are NOT part of rollback state - only playback positions are rolled back.

---

## FFI Changes

### Minimal API Changes

The existing FFI API remains unchanged:

```rust
// These still work exactly as before
fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
fn play_sound(sound: u32, volume: f32, pan: f32);
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);
fn channel_set(channel: u32, volume: f32, pan: f32);
fn channel_stop(channel: u32);
fn music_play(sound: u32, volume: f32);
fn music_stop();
fn music_set_volume(volume: f32);
```

The difference is internal:
- **Before:** These buffer `AudioCommand` enums, processed by audio server thread
- **After:** These directly modify `AudioPlaybackState` in `ctx.rollback`

### Implementation Change

```rust
// Before (command buffering)
fn channel_play_impl(state: &mut ZFFIState, channel: u32, sound: u32, ...) {
    state.audio_commands.push(AudioCommand::ChannelPlay { channel, sound, ... });
}

// After (direct state modification on rollback state)
fn channel_play(
    mut caller: Caller<'_, WasmGameContext<ZInput, ZFFIState, ZRollbackState>>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel as usize];
    ch.sound = sound;
    ch.position = 0;
    ch.volume = volume;
    ch.pan = pan;
    ch.looping = looping;
}
```

---

## Audio Trait Changes (Breaking)

The `Audio` trait in `core/src/console.rs` must be redesigned:

### Before (rodio model)

```rust
pub trait Audio: Send {
    fn play(&mut self, handle: SoundHandle, volume: f32, looping: bool);
    fn stop(&mut self, handle: SoundHandle);
    fn set_rollback_mode(&mut self, rolling_back: bool);
}
```

### After (per-frame model)

```rust
pub trait Audio: Send {
    /// Push one frame's worth of generated audio samples
    fn push_frame(&mut self, samples: &[f32]);

    /// Get current sample rate (may differ from preferred)
    fn sample_rate(&self) -> u32;

    /// Check buffer health (0.0 = empty, 1.0 = full)
    fn buffer_health(&self) -> f32;
}
```

**Breaking change notes:**
- `play()`, `stop()`, `set_rollback_mode()` removed
- Audio state now managed via `ConsoleRollbackState`, not `Audio` trait
- `TestAudio` in `test_utils.rs` needs updating

---

## Latency Analysis

### Current System (rodio)

- Rodio internal buffer: variable (platform-dependent)
- Estimated latency: 20-100ms depending on platform
- Unpredictable

### Proposed System (per-frame + cpal)

- Ring buffer: ~50ms (configurable)
- cpal buffer: ~23ms (512 samples @ 22050Hz)
- Frame generation: varies by tick rate (8.3ms @ 120fps to 41.7ms @ 24fps)
- **Total worst-case: ~115ms** (at 24fps)
- **Typical @ 60fps: ~90ms**
- **Predictable**

The latency is slightly higher but *consistent*, which is more important for game feel.

### Reducing Latency

If needed, latency can be reduced by:
1. Smaller cpal buffer size (256 samples → ~12ms)
2. Smaller ring buffer (~25ms)
3. Higher tick rate (120fps → 8.3ms frame time)

---

## Comparison to Current System

| Aspect | Current (rodio) | Proposed (cpal per-frame) |
|--------|-----------------|---------------------------|
| **Audio state location** | Audio server thread | `ZRollbackState` (host-side POD) |
| **Rollback behavior** | Muted during replay | Rolls back with game state |
| **Serialization** | N/A | Zero-copy via bytemuck (340 bytes) |
| **Dynamic pan/volume** | Atomic updates (complex) | Direct state modification |
| **Thread sync** | MPSC + SharedPan atomics | Lock-free ring buffer only |
| **Latency** | Variable (20-100ms) | Predictable (~50-90ms) |
| **Code complexity** | High (multiple wrappers) | Lower (straightforward) |
| **Dependencies** | rodio (high-level) | cpal + ringbuf (low-level) |
| **Tick rate support** | Implicit | Explicit (24/30/60/120) |

---

## Implementation Plan

### Phase 1: Core Architecture

1. Rename `GameStateWithConsole` → `WasmGameContext` with `ffi` and `rollback` fields
2. Add `ConsoleRollbackState` trait (POD-based) to `core/src/console.rs`
3. Update `GameStateSnapshot` to include `console_data: Vec<u8>`
4. Update `RollbackStateManager` to save/load console state via bytemuck
5. Update `Console` trait with `type RollbackState`

**Files:** `core/src/console.rs`, `core/src/wasm.rs`, `core/src/rollback/state.rs`

### Phase 2: Audio Backend

1. Add `cpal = "0.15"` and `ringbuf = "0.4"` dependencies, remove `rodio`
2. Create `AudioOutput` struct with lock-free ring buffer
3. Implement sample rate detection with fallback
4. Test basic audio output

**Files:** `emberware-z/Cargo.toml`, `emberware-z/src/audio.rs`

### Phase 3: Audio Playback State

1. Create `ZRollbackState` with `AudioPlaybackState` (POD)
2. Implement `ConsoleRollbackState` for `ZRollbackState`
3. Move sound storage to `ZResourceManager`
4. Implement `generate_audio_frame()` with equal-power panning
5. Test mixing multiple channels

**Files:** `emberware-z/src/state/rollback_state.rs`, `emberware-z/src/audio.rs`, `emberware-z/src/resource_manager.rs`

### Phase 4: FFI Integration

1. Update FFI type signature to use `WasmGameContext<ZInput, ZFFIState, ZRollbackState>`
2. Modify FFI functions to update `ctx.rollback.audio` directly
3. Remove command buffering (`AudioCommand` enum)
4. Update game loop to call `generate_audio_frame()`

**Files:** `emberware-z/src/ffi/audio.rs`, `emberware-z/src/app/game_session.rs`

### Phase 5: Cleanup and Testing

1. Remove old rodio code and `set_rollback_mode()`
2. Update `Audio` trait in core (breaking change)
3. Update `TestAudio` in test_utils
4. Test with local 2-player (rollback active)
5. Verify no audio duplication during rollback
6. Run all examples

---

## Risks and Mitigations

### Risk 1: Buffer Underruns

**Symptom:** Audio crackling/stuttering when game can't keep up
**Mitigation:**
- Ring buffer sized for ~50ms (3 frames of slack at 60fps)
- Underrun produces silence (not random noise)
- Debug overlay shows buffer health

### Risk 2: Sample Rate Mismatch

**Symptom:** Audio pitch slightly off on some systems
**Mitigation:**
- Prefer 22050Hz, fall back to device native rate
- Log actual sample rate for debugging
- Acceptable for retro aesthetic (slight pitch variation)

### Risk 3: Platform-Specific Issues

**Symptom:** Audio doesn't work on some platforms
**Mitigation:**
- cpal has good cross-platform support
- Test on all target platforms before release
- Fallback to silence if audio fails (game still playable)
- Log detailed error messages

### Risk 4: Breaking Change to Audio Trait

**Symptom:** Other consoles need updating
**Mitigation:**
- Clear migration path documented
- TestConsole updated as reference
- Other consoles (Classic, etc.) can use dummy implementation initially

---

## Platform Considerations

### Windows
- WASAPI backend (default)
- 22050Hz widely supported
- Low latency achievable

### macOS
- CoreAudio backend
- 22050Hz may require resampling
- Fall back to 44100Hz if needed

### Linux
- ALSA or PulseAudio backend
- 22050Hz support varies
- Fall back to device default

### Web (Future)
- WebAudio backend
- Sample rate typically 44100Hz or 48000Hz
- Will require resampling strategy

---

## Future Considerations

### ADPCM Integration

The ADPCM compression spec remains compatible:
- Sounds still stored as PCM after decompression at load time
- `generate_audio_frame()` reads from PCM samples
- No changes needed to compression/decompression

### Tracker Music

Future tracker/sequencer system would fit naturally:
- Tracker generates samples per-frame
- Mixed into the same audio buffer
- State is deterministic (pattern position, etc.)
- Would be part of `AudioPlaybackState`

---

## References

- [cpal documentation](https://docs.rs/cpal/0.15)
- [ringbuf crate](https://docs.rs/ringbuf/0.4) - Lock-free ring buffer
- [bytemuck crate](https://docs.rs/bytemuck) - Safe POD casting
- [Libretro Dynamic Rate Control](https://docs.libretro.com/development/cores/dynamic-rate-control/) - Emulator audio sync
- [nesdev: Audio emulation approaches](https://forums.nesdev.org/viewtopic.php?t=10048)
- [GGPO](https://www.ggpo.net) - Rollback netcode reference

---

## Summary of Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Game context struct | `WasmGameContext` with `ffi` + `rollback` | Clear separation, FFI access to both |
| Rollback state | POD via bytemuck | Zero-copy serialization, 340 bytes |
| Music channel | Same per-frame system as SFX | Simpler, uniform implementation |
| Sample rate | 22050Hz preferred, device fallback | Practical compatibility |
| Tick rates | 24, 30, 60, 120 fps | Match existing console specs |
| Sound storage | `ZResourceManager` | Consistent with textures/meshes |
| Ring buffer | Lock-free (ringbuf 0.4) | Real-time audio thread safety |
| Panning | Equal-power | Constant perceived loudness |
