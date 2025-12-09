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

## Future Enhancements

1. **Video export**: Render replay to video file
2. **Replay browser**: Online sharing of replays
3. **Ghost mode**: Overlay replay on live gameplay (racing games)
4. **Replay diff**: Compare two replays to find divergence
5. **Input display**: Show controller overlay during playback
6. **Commentary track**: Record voice-over with replay
