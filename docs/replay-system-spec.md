# Replay System Specification

## Overview

The Replay System records gameplay sessions as input streams that can be played back deterministically. Because GGRS already requires deterministic execution, replay is "free" - we just need to record and replay inputs.

This enables:
- Bug reproduction: "Here's a replay file that crashes at frame 4521"
- Testing: Automated regression testing by replaying known-good sessions
- Content creation: Record gameplay for trailers, tutorials
- Debug analysis: Step through problematic frames with debug tools

## Architecture

### Replay Data Structure

```rust
/// Complete replay file
pub struct Replay {
    /// Header with metadata
    pub header: ReplayHeader,

    /// Snapshot at start of recording (enables mid-game replays)
    pub initial_state: Vec<u8>,

    /// Input stream - one entry per frame
    pub inputs: Vec<FrameInputs>,

    /// Optional: periodic state snapshots for seeking
    pub checkpoints: Vec<Checkpoint>,
}

/// Replay metadata
pub struct ReplayHeader {
    /// Magic bytes: "EMBR"
    pub magic: [u8; 4],

    /// Replay format version
    pub version: u32,

    /// Game identifier
    pub game_id: String,

    /// Game ROM hash (for compatibility check)
    pub rom_hash: [u8; 32],

    /// Console type (e.g., "emberware-z")
    pub console: String,

    /// Recording timestamp (Unix epoch)
    pub timestamp: u64,

    /// Total frame count
    pub frame_count: u64,

    /// Number of players
    pub player_count: u8,

    /// Random seed used
    pub random_seed: u64,

    /// Optional user metadata
    pub metadata: HashMap<String, String>,
}

/// Inputs for a single frame
pub struct FrameInputs {
    /// Frame number
    pub frame: u64,

    /// Input for each player (serialized console input type)
    pub player_inputs: Vec<Vec<u8>>,
}

/// State checkpoint for fast seeking
pub struct Checkpoint {
    /// Frame number of this checkpoint
    pub frame: u64,

    /// Serialized game state at this frame
    pub state: Vec<u8>,
}
```

### Recording Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    Recording Mode                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  User presses F9 to start recording                         │
│                    │                                         │
│                    ▼                                         │
│  1. Capture initial state via save_state                    │
│                    │                                         │
│                    ▼                                         │
│  2. Store header metadata (game_id, rom_hash, seed, etc.)   │
│                    │                                         │
│                    ▼                                         │
│  ┌─────────────────────────────────────┐                    │
│  │         Game Loop (recording)        │                    │
│  │                                       │                    │
│  │  For each frame:                     │                    │
│  │    • Capture player inputs           │                    │
│  │    • Append to inputs[]              │                    │
│  │    • Every N frames: save checkpoint │                    │
│  │    • Normal update() / render()      │                    │
│  │                                       │                    │
│  └─────────────────────────────────────┘                    │
│                    │                                         │
│                    ▼                                         │
│  User presses F9 to stop recording                          │
│                    │                                         │
│                    ▼                                         │
│  3. Finalize replay file                                    │
│                    │                                         │
│                    ▼                                         │
│  4. Save to ~/.emberware/games/{game_id}/replays/           │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Playback Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    Playback Mode                             │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  User loads replay file                                      │
│                    │                                         │
│                    ▼                                         │
│  1. Validate header (magic, version, rom_hash)              │
│                    │                                         │
│                    ├──── ROM mismatch ──► Warn user         │
│                    │                                         │
│                    ▼                                         │
│  2. Load initial_state via load_state                       │
│                    │                                         │
│                    ▼                                         │
│  3. Set random seed from header                             │
│                    │                                         │
│                    ▼                                         │
│  ┌─────────────────────────────────────┐                    │
│  │         Game Loop (playback)         │                    │
│  │                                       │                    │
│  │  For each frame:                     │                    │
│  │    • Read inputs[frame]              │                    │
│  │    • Feed to update() (ignore live)  │                    │
│  │    • Normal render()                 │                    │
│  │    • Handle playback controls        │                    │
│  │                                       │                    │
│  └─────────────────────────────────────┘                    │
│                    │                                         │
│                    ▼                                         │
│  End of inputs[] reached                                    │
│                    │                                         │
│                    ▼                                         │
│  Options: Loop, Pause, Return to menu                       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## FFI API

### Recording Control (Host → Game)

```rust
// Notify game that recording has started
// Game can display "REC" indicator, etc.
extern "C" fn __replay_recording_started();

// Notify game that recording has stopped
extern "C" fn __replay_recording_stopped();
```

### Playback Control (Host → Game)

```rust
// Notify game that playback has started
extern "C" fn __replay_playback_started();

// Notify game that playback has stopped
extern "C" fn __replay_playback_stopped();

// Called when seeking to a different frame
extern "C" fn __replay_seeked(new_frame: u64);
```

### Game Queries (Game → Host)

```rust
// Check current mode
extern "C" fn replay_is_recording() -> i32;
extern "C" fn replay_is_playing() -> i32;

// Get playback info
extern "C" fn replay_current_frame() -> u64;
extern "C" fn replay_total_frames() -> u64;

// Get playback speed (1.0 = normal, 0.5 = half, 2.0 = double)
extern "C" fn replay_playback_speed() -> f32;
```

## Playback Controls

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| F9 | Toggle recording |
| F10 | Load replay file (opens file picker) |
| Space | Play/Pause (during playback) |
| Left Arrow | Step back 1 frame |
| Right Arrow | Step forward 1 frame |
| Shift+Left | Jump back 60 frames (1 second) |
| Shift+Right | Jump forward 60 frames |
| Home | Jump to start |
| End | Jump to end |
| [ | Decrease playback speed (0.25x, 0.5x, 1x, 2x, 4x) |
| ] | Increase playback speed |
| L | Toggle loop mode |
| Escape | Exit playback |

### Timeline UI

```
┌──────────────────────────────────────────────────────────────┐
│  ◄◄  ◄  ▶  ►  ►►  │ ▬▬▬▬▬▬▬▬▬▬▬▬●▬▬▬▬▬▬▬▬▬▬▬ │ 2:34 / 5:00 │
│                   │ ▲ checkpoint markers      │   1.0x  🔁  │
└──────────────────────────────────────────────────────────────┘
```

- Scrubber bar shows position in replay
- Checkpoint markers show where fast-seeking is available
- Current time / total time display
- Speed indicator and loop toggle

## Seeking Implementation

### Challenge
Deterministic replay means to reach frame N, you must execute frames 0 through N-1. For a 5-minute replay (18,000 frames), seeking to the end would require simulating all frames.

### Solution: Periodic Checkpoints

During recording, save full state snapshots every N frames (e.g., every 300 frames = 5 seconds at 60fps).

**Seeking algorithm:**
1. Find nearest checkpoint before target frame
2. Load checkpoint state
3. Simulate forward from checkpoint to target frame

**Example:** To seek to frame 1500:
1. Load checkpoint at frame 1200
2. Simulate frames 1201-1500 (300 frames)
3. This takes ~50ms instead of ~2500ms

### Checkpoint Frequency Trade-off

| Checkpoint Interval | File Size Overhead | Max Seek Time (60fps) |
|---------------------|-------------------|----------------------|
| 60 frames (1s) | ~10% larger | ~17ms |
| 300 frames (5s) | ~2% larger | ~83ms |
| 600 frames (10s) | ~1% larger | ~167ms |
| None | No overhead | Full replay time |

**Recommendation:** 300 frames (5 seconds) - good balance of size and responsiveness.

## File Format

### Binary Format (`.embreplay`)

```
┌────────────────────────────────────────┐
│ Header (fixed size)                    │
│ ├─ magic: "EMBR" (4 bytes)            │
│ ├─ version: u32 (4 bytes)             │
│ ├─ flags: u32 (4 bytes)               │
│ ├─ game_id_len: u16 (2 bytes)         │
│ ├─ game_id: [u8; game_id_len]         │
│ ├─ rom_hash: [u8; 32]                 │
│ ├─ console_len: u16                   │
│ ├─ console: [u8; console_len]         │
│ ├─ timestamp: u64                     │
│ ├─ frame_count: u64                   │
│ ├─ player_count: u8                   │
│ ├─ random_seed: u64                   │
│ ├─ input_size: u16 (per player)       │
│ ├─ initial_state_len: u32             │
│ ├─ checkpoint_count: u32              │
│ └─ metadata_len: u32                  │
├────────────────────────────────────────┤
│ Initial State                          │
│ └─ [u8; initial_state_len]            │
├────────────────────────────────────────┤
│ Checkpoints Index                      │
│ └─ [(frame: u64, offset: u64); N]     │
├────────────────────────────────────────┤
│ Input Stream                           │
│ └─ [player_inputs; frame_count]       │
│    └─ [[u8; input_size]; player_count]│
├────────────────────────────────────────┤
│ Checkpoint Data                        │
│ └─ [state_data; checkpoint_count]     │
├────────────────────────────────────────┤
│ Metadata (JSON)                        │
│ └─ [u8; metadata_len]                 │
└────────────────────────────────────────┘
```

### Compression

- Input stream: Delta compression (most frames have similar inputs)
- State data: LZ4 compression (fast decompression for seeking)
- Estimated compression ratio: 3-5x for inputs, 2-3x for state

## Storage Location

```
~/.emberware/games/{game_id}/replays/
├── 2024-01-15_14-30-00.embreplay
├── 2024-01-15_16-45-22.embreplay
└── bug_report_crash.embreplay
```

## Integration with Debug Tools

### Debug Inspection Sync

When playing back a replay with the debug panel open:
- All registered variables update in real-time
- Graphs show historical data
- Pausing the replay allows full inspection

### Frame Stepping with Debug

Combine replay with frame stepping for powerful debugging:
1. Play replay to problem area
2. Pause
3. Enable debug panel
4. Step frame-by-frame
5. Watch values change
6. Identify exact frame where bug occurs

### Annotations

Allow adding annotations to replays:

```rust
/// Annotation at a specific frame
pub struct Annotation {
    pub frame: u64,
    pub text: String,
    pub category: AnnotationCategory,
}

pub enum AnnotationCategory {
    Bug,        // Mark where a bug occurs
    Note,       // General note
    Milestone,  // Important gameplay moment
    Marker,     // Named position for seeking
}
```

Timeline UI shows annotation markers that can be clicked to seek.

## Pending Questions

### Q1: Multi-player replay format?
For netplay recordings:
- A) Record all player inputs in single file (current proposal)
- B) Record each player's perspective separately
- C) Record from one player's perspective with predicted inputs

**Recommendation:** Option A - single authoritative replay with all inputs.

### Q2: Replay sharing format?
Should replays be shareable across different machines?
- ROM hash ensures same game version
- But what about different emulator versions?

**Options:**
- A) Strict: Require exact emulator version match
- B) Lenient: Try to play, warn on mismatch
- C) Versioned: Include emulator version, migration support

**Recommendation:** Option B with clear warnings.

### Q3: "Take over" from replay?
Should users be able to:
- A) Only watch replays passively
- B) Press a key to "take over" and play from current point
- C) Both modes available

**Recommendation:** Option C - "take over" is powerful for bug reproduction.

### Q4: Recording in netplay?
Should recording be available during P2P netplay?
- A) No - too complex with rollback
- B) One designated player records
- C) All players record their own perspective

**Recommendation:** Option B - host records authoritative replay.

### Q5: Automatic recording?
Should the emulator automatically record all sessions?
- A) No - explicit recording only
- B) Yes - configurable, auto-delete old replays
- C) "Instant replay" buffer - last N minutes always available

**Recommendation:** Option C - instant replay buffer is most useful.

### Q6: External replay loading?
Should replays from untrusted sources be playable?
- Security concern: Crafted inputs might trigger bugs
- Options: Sandboxed playback, warning dialog, signature verification

**Recommendation:** Warning dialog + sandboxed playback.

## Pros

1. **Leverages GGRS determinism**: No new infrastructure needed for replay itself
2. **Bug reproduction**: Share exact failure conditions
3. **Regression testing**: Automated replay verification
4. **Content creation**: Easy gameplay recording
5. **Debug integration**: Combine with inspection/stepping for powerful analysis
6. **Relatively simple**: Just record/replay inputs

## Cons

1. **State size**: Checkpoints can be large for complex games
2. **ROM coupling**: Replays only work with exact same ROM
3. **Seeking performance**: Fast-seeking requires checkpoints
4. **No video export**: Raw replay, not video file
5. **Multiplayer complexity**: Netplay recording has edge cases

## Implementation Complexity

**Estimated effort:** Medium

**Key components:**
1. Replay data structures - 0.5 days
2. Recording system - 1 day
3. Playback system - 1 day
4. Checkpoint management - 1 day
5. Seeking implementation - 1 day
6. Timeline UI - 2 days
7. File format & compression - 1 day
8. File management UI - 1 day
9. Testing - 1.5 days

**Total:** ~10 days

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Replay Types to Core

**File: `core/src/replay/mod.rs` (new file)**

```rust
//! Replay recording and playback system

use crate::console::ConsoleInput;
use bytemuck::Pod;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod recording;
pub mod playback;

/// Complete replay file
#[derive(Serialize, Deserialize)]
pub struct Replay {
    pub header: ReplayHeader,
    pub initial_state: Vec<u8>,
    pub inputs: Vec<FrameInputs>,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplayHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub game_id: String,
    pub rom_hash: [u8; 32],
    pub console: String,
    pub timestamp: u64,
    pub frame_count: u64,
    pub player_count: u8,
    pub random_seed: u64,
    pub input_size: usize,  // Size of serialized input type
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FrameInputs {
    pub frame: u64,
    pub player_inputs: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub frame: u64,
    pub state: Vec<u8>,
}

impl Replay {
    pub const MAGIC: [u8; 4] = *b"EMBR";
    pub const VERSION: u32 = 1;

    /// Find nearest checkpoint at or before target frame
    pub fn find_checkpoint(&self, target_frame: u64) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .filter(|c| c.frame <= target_frame)
            .max_by_key(|c| c.frame)
    }
}
```

### Step 2: Implement Recording System

**File: `core/src/replay/recording.rs`**

```rust
use super::*;
use crate::console::ConsoleInput;
use crate::wasm::GameInstance;
use bytemuck::Pod;

/// Active recording session
pub struct ReplayRecorder<I: ConsoleInput + Pod> {
    header: ReplayHeader,
    initial_state: Vec<u8>,
    inputs: Vec<FrameInputs>,
    checkpoints: Vec<Checkpoint>,
    checkpoint_interval: u64,
    last_checkpoint_frame: u64,
    _phantom: std::marker::PhantomData<I>,
}

impl<I: ConsoleInput + Pod> ReplayRecorder<I> {
    pub fn new(
        game_id: &str,
        rom_hash: [u8; 32],
        console: &str,
        player_count: u8,
        random_seed: u64,
        initial_state: Vec<u8>,
    ) -> Self {
        Self {
            header: ReplayHeader {
                magic: Replay::MAGIC,
                version: Replay::VERSION,
                game_id: game_id.to_string(),
                rom_hash,
                console: console.to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                frame_count: 0,
                player_count,
                random_seed,
                input_size: std::mem::size_of::<I>(),
                metadata: HashMap::new(),
            },
            initial_state,
            inputs: Vec::new(),
            checkpoints: Vec::new(),
            checkpoint_interval: 300, // Every 5 seconds at 60fps
            last_checkpoint_frame: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Record inputs for a frame
    pub fn record_frame(&mut self, frame: u64, player_inputs: &[I]) {
        let serialized: Vec<Vec<u8>> = player_inputs
            .iter()
            .map(|i| bytemuck::bytes_of(i).to_vec())
            .collect();

        self.inputs.push(FrameInputs {
            frame,
            player_inputs: serialized,
        });
        self.header.frame_count = frame;
    }

    /// Add checkpoint (call periodically from game loop)
    pub fn maybe_add_checkpoint(&mut self, frame: u64, state: &[u8]) {
        if frame - self.last_checkpoint_frame >= self.checkpoint_interval {
            self.checkpoints.push(Checkpoint {
                frame,
                state: state.to_vec(),
            });
            self.last_checkpoint_frame = frame;
        }
    }

    /// Finalize recording into a Replay
    pub fn finish(self) -> Replay {
        Replay {
            header: self.header,
            initial_state: self.initial_state,
            inputs: self.inputs,
            checkpoints: self.checkpoints,
        }
    }
}
```

### Step 3: Implement Playback System

**File: `core/src/replay/playback.rs`**

```rust
use super::*;
use crate::console::ConsoleInput;
use bytemuck::Pod;

/// Replay playback state
pub struct ReplayPlayer<I: ConsoleInput + Pod> {
    replay: Replay,
    current_frame: u64,
    playback_speed: f32,
    paused: bool,
    _phantom: std::marker::PhantomData<I>,
}

impl<I: ConsoleInput + Pod> ReplayPlayer<I> {
    pub fn new(replay: Replay) -> Self {
        Self {
            replay,
            current_frame: 0,
            playback_speed: 1.0,
            paused: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get inputs for current frame (returns None if past end)
    pub fn get_frame_inputs(&self) -> Option<Vec<I>> {
        self.replay.inputs
            .iter()
            .find(|fi| fi.frame == self.current_frame)
            .map(|fi| {
                fi.player_inputs
                    .iter()
                    .map(|bytes| *bytemuck::from_bytes(bytes))
                    .collect()
            })
    }

    /// Advance to next frame
    pub fn advance(&mut self) {
        if !self.paused && self.current_frame < self.replay.header.frame_count {
            self.current_frame += 1;
        }
    }

    /// Seek to a specific frame
    pub fn seek(&mut self, target_frame: u64) -> Option<&Checkpoint> {
        self.current_frame = target_frame.min(self.replay.header.frame_count);
        self.replay.find_checkpoint(target_frame)
    }

    pub fn toggle_pause(&mut self) { self.paused = !self.paused; }
    pub fn is_paused(&self) -> bool { self.paused }
    pub fn current_frame(&self) -> u64 { self.current_frame }
    pub fn total_frames(&self) -> u64 { self.replay.header.frame_count }
    pub fn progress(&self) -> f32 {
        self.current_frame as f32 / self.replay.header.frame_count.max(1) as f32
    }
}
```

### Step 4: Integrate into Runtime

**File: `core/src/runtime.rs`**

Add replay state to Runtime:

```rust
use crate::replay::{Replay, ReplayRecorder, ReplayPlayer};

pub struct Runtime<C: Console> {
    // ... existing fields ...

    /// Active replay recording (if any)
    replay_recorder: Option<ReplayRecorder<C::Input>>,
    /// Active replay playback (if any)
    replay_player: Option<ReplayPlayer<C::Input>>,
}

impl<C: Console> Runtime<C> {
    /// Start recording a replay
    pub fn start_recording(&mut self, game_id: &str, rom_hash: [u8; 32]) -> anyhow::Result<()> {
        let game = self.game.as_ref().ok_or_else(|| anyhow::anyhow!("No game loaded"))?;
        let initial_state = game.save_state()?;
        let seed = game.state().rng_state;

        self.replay_recorder = Some(ReplayRecorder::new(
            game_id,
            rom_hash,
            self.console.specs().name,
            game.state().player_count as u8,
            seed,
            initial_state,
        ));
        Ok(())
    }

    /// Stop recording and return the replay
    pub fn stop_recording(&mut self) -> Option<Replay> {
        self.replay_recorder.take().map(|r| r.finish())
    }

    /// Start playing a replay
    pub fn start_playback(&mut self, replay: Replay) -> anyhow::Result<()> {
        let game = self.game.as_mut().ok_or_else(|| anyhow::anyhow!("No game loaded"))?;

        // Restore initial state
        game.load_state(&replay.initial_state)?;

        // Reseed RNG
        game.state_mut().seed_rng(replay.header.random_seed);

        self.replay_player = Some(ReplayPlayer::new(replay));
        Ok(())
    }

    /// During frame(), feed inputs from replay instead of live input
    pub fn frame(&mut self) -> anyhow::Result<(u32, f32)> {
        // If playing replay, override inputs
        if let Some(player) = &mut self.replay_player {
            if let Some(inputs) = player.get_frame_inputs() {
                // Feed replay inputs to game
                if let Some(game) = &mut self.game {
                    for (i, input) in inputs.iter().enumerate() {
                        game.set_input(i, *input);
                    }
                }
            }
            player.advance();
        }

        // If recording, capture inputs
        if let Some(recorder) = &mut self.replay_recorder {
            if let Some(game) = &self.game {
                let frame = game.state().tick_count;
                let inputs = game.get_all_inputs();
                recorder.record_frame(frame, &inputs);

                // Maybe add checkpoint
                if let Ok(state) = game.save_state() {
                    recorder.maybe_add_checkpoint(frame, &state);
                }
            }
        }

        // ... existing frame logic ...
    }
}
```

### Step 5: Add Replay FFI Functions

**File: `core/src/ffi.rs`**

```rust
// Add to register_common_ffi
linker.func_wrap("env", "replay_is_recording", replay_is_recording)?;
linker.func_wrap("env", "replay_is_playing", replay_is_playing)?;
linker.func_wrap("env", "replay_current_frame", replay_current_frame)?;
linker.func_wrap("env", "replay_total_frames", replay_total_frames)?;

fn replay_is_recording<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> i32 {
    if caller.data().game.replay_recording { 1 } else { 0 }
}

fn replay_is_playing<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> i32 {
    if caller.data().game.replay_playing { 1 } else { 0 }
}

fn replay_current_frame<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> u64 {
    caller.data().game.replay_frame
}

fn replay_total_frames<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> u64 {
    caller.data().game.replay_total_frames
}
```

### Step 6: Add Timeline UI

**File: `core/src/app/replay_ui.rs` (new file)**

```rust
use egui::{Ui, Response};

pub struct ReplayTimelineUI {
    seeking: bool,
    seek_frame: u64,
}

impl ReplayTimelineUI {
    pub fn new() -> Self {
        Self { seeking: false, seek_frame: 0 }
    }

    pub fn show(&mut self, ui: &mut Ui, current: u64, total: u64, paused: bool) -> ReplayAction {
        let mut action = ReplayAction::None;

        ui.horizontal(|ui| {
            // Play/pause button
            if ui.button(if paused { "▶" } else { "⏸" }).clicked() {
                action = ReplayAction::TogglePause;
            }

            // Step buttons
            if ui.button("⏮").clicked() { action = ReplayAction::Seek(0); }
            if ui.button("◀").clicked() { action = ReplayAction::StepBack; }
            if ui.button("▶").clicked() { action = ReplayAction::StepForward; }
            if ui.button("⏭").clicked() { action = ReplayAction::Seek(total); }

            // Timeline slider
            let mut progress = current as f32 / total.max(1) as f32;
            if ui.add(egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false)).changed() {
                action = ReplayAction::Seek((progress * total as f32) as u64);
            }

            // Frame counter
            ui.label(format!("{} / {}", current, total));
        });

        action
    }
}

pub enum ReplayAction {
    None,
    TogglePause,
    StepForward,
    StepBack,
    Seek(u64),
}
```

### Step 7: Wire into GameSession

**File: `core/src/app/session.rs`**

```rust
use crate::replay::{Replay, ReplayPlayer};
use super::replay_ui::{ReplayTimelineUI, ReplayAction};

impl<C: Console> GameSession<C> {
    /// Handle replay hotkeys
    pub fn handle_replay_hotkey(&mut self, key: &winit::keyboard::Key) -> bool {
        use winit::keyboard::{Key, NamedKey};
        match key {
            Key::Named(NamedKey::F9) => {
                self.toggle_recording();
                true
            }
            Key::Named(NamedKey::Space) if self.runtime.is_replaying() => {
                self.runtime.toggle_replay_pause();
                true
            }
            Key::Named(NamedKey::ArrowLeft) if self.runtime.is_replaying() => {
                self.runtime.step_replay_back();
                true
            }
            Key::Named(NamedKey::ArrowRight) if self.runtime.is_replaying() => {
                self.runtime.step_replay_forward();
                true
            }
            _ => false,
        }
    }

    /// Render replay timeline UI (call from egui context)
    pub fn render_replay_ui(&mut self, ctx: &egui::Context) {
        if let Some(player) = self.runtime.replay_player() {
            egui::TopBottomPanel::bottom("replay_timeline").show(ctx, |ui| {
                let action = self.replay_ui.show(
                    ui,
                    player.current_frame(),
                    player.total_frames(),
                    player.is_paused(),
                );
                // Handle action...
            });
        }
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/replay/mod.rs` | New file: Replay, ReplayHeader, FrameInputs, Checkpoint types |
| `core/src/replay/recording.rs` | New file: ReplayRecorder implementation |
| `core/src/replay/playback.rs` | New file: ReplayPlayer implementation |
| `core/src/runtime.rs` | Add replay_recorder, replay_player fields and methods |
| `core/src/wasm/state.rs` | Add replay_recording, replay_playing, replay_frame flags |
| `core/src/ffi.rs` | Add replay FFI functions |
| `core/src/app/replay_ui.rs` | New file: Timeline UI widget |
| `core/src/app/session.rs` | Add replay hotkey handling and UI rendering |
| `core/src/lib.rs` | Export replay module |
| `emberware-z/src/app/mod.rs` | Wire replay UI into egui rendering |
| `core/Cargo.toml` | Add `lz4_flex` for compression (optional) |

### Test Cases

1. **Basic recording**: Start game, press F9, play, press F9, verify replay saved
2. **Playback**: Load replay, verify identical game state at each frame
3. **Seeking**: Jump to frame 1000, verify state matches checkpoint
4. **Determinism**: Play same replay twice, verify byte-identical states
5. **Checkpoint seeking**: Seek to frame 1500, verify correct checkpoint loaded
6. **Take over**: During playback, press key to take control, verify game continues
7. **File format**: Save/load replay file, verify roundtrip

## Future Enhancements

1. **Video export**: Render replay to video file
2. **Replay browser**: Online sharing of replays
3. **Ghost mode**: Overlay replay on live gameplay (racing games)
4. **Replay diff**: Compare two replays to find divergence
5. **Input display**: Show controller overlay during playback
6. **Commentary track**: Record voice-over with replay
