# Emberware Development Tasks

**Task Status Tags:**
- `[STABILITY]` — Robustness, error handling, testing, safety improvements
- `[FEATURE]` — New functionality for game developers
- `[NETWORKING]` — P2P, matchmaking, rollback netcode
- `[POLISH]` — UX improvements, optimization, documentation

---

## Architecture Overview

```
emberware/
├── shared/           # API types for platform communication
├── core/             # Console framework, WASM runtime, GGRS rollback
├── emberware-z/      # PS1/N64 fantasy console implementation
├── docs/
└── examples/
```

### Core Framework Design

The `core` crate provides a generic `Console` trait that each fantasy console implements:

```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;  // Console-specific input layout

    fn name(&self) -> &'static str;
    fn specs(&self) -> &ConsoleSpecs;

    // FFI registration for console-specific host functions
    fn register_ffi(&self, linker: &mut Linker<GameState>) -> Result<()>;

    // Create graphics/audio backends
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;
    fn create_audio(&self) -> Result<Self::Audio>;

    // Map physical input to console-specific input
    fn map_input(&self, raw: &RawInput) -> Self::Input;
}

// Must be POD for GGRS network serialization
pub trait ConsoleInput: Clone + Copy + Default + bytemuck::Pod + bytemuck::Zeroable {}

pub trait Graphics: Send {
    fn resize(&mut self, width: u32, height: u32);
    // Console calls into this during render via FFI
}

pub trait Audio: Send {
    fn play(&mut self, handle: SoundHandle, volume: f32, looping: bool);
    fn stop(&mut self, handle: SoundHandle);
    fn set_rollback_mode(&mut self, rolling_back: bool); // Mute during rollback
}
```

### Input Abstraction

Each console defines its own input type for GGRS serialization:

```rust
// Emberware Z (PS2/Xbox style with analog triggers)
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct ZInput {
    pub buttons: u16,        // D-pad + A/B/X/Y + L/R bumpers + Start/Select + L3/R3
    pub left_stick_x: i8,
    pub left_stick_y: i8,
    pub right_stick_x: i8,
    pub right_stick_y: i8,
    pub left_trigger: u8,    // 0..255 analog
    pub right_trigger: u8,
}

// Emberware Classic (6-button, no analog)
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct ClassicInput {
    pub buttons: u16,  // D-pad + A/B/C/X/Y/Z + L/R + Start/Select
}
```

Input FFI functions are console-specific (e.g., `trigger_value` only exists on Z).

The `Runtime<C: Console>` handles:
- WASM loading and execution via wasmtime
- GGRS rollback session management
- Game loop timing (fixed tick rate, variable render rate)
- State serialization for rollback (save_state/load_state calls into WASM)
- Input synchronization across network

---

## In Progress

---

## TODO

### **[NEEDS CLARIFICATION] Define and enforce console runtime limits**

**Current State:** Partial limit enforcement - VRAM tracking (8MB), vertex format validation, memory bounds checking. No enforcement for draw calls, vertex counts, mesh counts, or CPU budget per frame.

**Why This Matters:**
- Enforces fantasy console authenticity (PS1/N64 had strict hardware limits)
- Prevents performance issues from runaway resource usage
- Helps developers understand and work within platform constraints
- Maintains consistent performance across games

**Potential Limits to Enforce:**

| Limit | Suggested Value | Rationale |
|-------|----------------|-----------|
| Max draw calls/frame | 512 | PS1/N64 could handle ~500-1000 triangles/sec at 30fps |
| Max vertices/frame (immediate) | 100,000 | Reasonable for fantasy console aesthetic |
| Max retained meshes | 256 | Encourages efficient resource management |
| Max vertices per mesh | 65,536 | u16 index limit, PS1-era constraint |
| CPU budget per tick | 4ms @ 60fps | Console spec already defines this, needs enforcement |
| RAM limit (WASM heap) | 16MB | Console spec defines this, not currently enforced |

**Questions to Resolve:**
1. Should limits be enforced at runtime with warnings/errors, or just tracked for debugging?
2. Should limits be per-console (Z has different limits than Classic)?
3. How to handle limit violations - reject draw calls, log warnings, or hard error?
4. Should some limits be configurable in debug mode for development?
5. Do we need separate limits for 2D vs 3D draws (e.g., UI overlay doesn't count toward 3D limits)?

**Implementation Approach (Once Clarified):**
1. Add limit constants to `ConsoleSpecs` struct
2. Add runtime tracking counters (reset per frame):
   - `draw_calls_this_frame: usize`
   - `immediate_vertices_this_frame: usize`
3. Validate against limits in FFI functions (`draw_triangles`, `draw_mesh`, etc.)
4. Add warnings/errors when limits exceeded
5. Expose stats via debug overlay (show current/max for each limit)
6. Document limits in console documentation

**Files to Modify:**
- `core/src/console.rs` - Add limit fields to `ConsoleSpecs`
- `emberware-z/src/console.rs` - Define Z-specific limits
- `emberware-z/src/graphics/mod.rs` - Track draw calls, vertices per frame
- `emberware-z/src/ffi/mod.rs` - Validate limits in draw functions
- `emberware-z/src/app.rs` - Display stats in debug overlay
- `docs/emberware-z.md` - Document console limits

---

### **[NETWORKING] Implement synchronized save slots (VMU-style memory cards)**

Similar to Dreamcast VMUs, each player "brings" their own save data to a networked session.
This enables fighting games with unlocked characters, RPGs with player stats, etc.

**Design:**
1. Each player has their own "memory card" (save slot) that travels with their controller
2. During P2P session setup, save data is exchanged before `init()` runs
3. All clients receive identical slot layout: slot 0 = P1's data, slot 1 = P2's data, etc.
4. Games use `player_count()` and `local_player_mask()` to know which slot is "theirs"
5. Save data is raw bytes - games handle serialization/deserialization

**Implementation steps:**
1. Add `save_data_limit: usize` to `ConsoleSpecs` (e.g., 64KB per player for Emberware Z)
2. Add `SessionSaveData` struct to hold per-player save buffers
3. Modify session setup to exchange save data via signaling/WebRTC data channel:
   - Before GGRS session starts, exchange save buffers
   - Use a simple protocol: `[player_index: u8][length: u32][data: [u8; length]]`
   - All players must receive all save data before proceeding
4. Populate `GameState.save_data[player_index]` slots identically on all clients
5. Call `init()` only after save data is synchronized
6. For local sessions: load save data from disk into slot 0 before init
7. Existing `save()`/`load()`/`delete()` FFI works unchanged - just reads from synchronized slots

**Platform integration:**
- Platform layer loads player's save from `~/.emberware/games/{game_id}/saves/player.sav`
- On session end, local player's slot is written back to disk
- Save data versioning/migration is game's responsibility

**Edge cases:**
- Player without save data: slot contains empty buffer (len=0)
- Save data too large: reject during session setup, show error
- Player disconnect during exchange: abort session, show error
- Spectators: receive all save data but don't contribute a slot

---

### **[NETWORKING] Implement matchbox signaling connection**

**Status:** Needs clarification - matchmaking handled by platform service, integration details TBD

- Connect to matchbox signaling server
- WebRTC peer connection establishment
- ICE candidate exchange
- Connection timeout handling
- Matchmaking handled by platform service - integration details TBD

---

### **[NETWORKING] Implement host/join game flow**

**Status:** Needs clarification - requires matchbox signaling connection to be implemented first

- Requires matchbox signaling connection to be implemented first
- Host game via deep link: `emberware://host/{game_id}`
- Join game via deep link: `emberware://join/{game_id}?token=...`
- Integration with platform matchmaking TBD

---

## In Progress

---

## Done

### **[STABILITY] Fix clippy warnings for code quality**

**Completed:** Resolved all clippy warnings in audio FFI code

Fixed two clippy warnings:
1. **Use .is_multiple_of()**: Replaced manual modulo check `byte_len % 2 != 0` with idiomatic `!byte_len.is_multiple_of(2)` in load_sound function
2. **Move items before test module**: Relocated audio FFI functions (load_sound, play_sound, etc.) before #[cfg(test)] module to follow Rust conventions

**Files Modified:**
- `emberware-z/src/ffi/mod.rs` - Improved code quality, proper module organization

**Test Results:** 520 tests passing, zero clippy warnings

---

### **[FEATURE] Complete audio backend playback**

**Completed:** Full PS1/N64-style audio system with thread-safe rodio playback

Implemented complete audio backend with:
- **Audio server thread**: Background thread owns rodio OutputStream/Sinks, satisfies Send requirement
- **16 sound effect channels**: Independent volume and pan control per channel
- **Dedicated music channel**: Always loops, separate from SFX channels
- **Rollback-aware**: Commands discarded during rollback replay (industry standard)
- **Channel state tracking**: Same sound playing doesn't restart (rollback-friendly)
- **8 FFI functions**: load_sound, play_sound, channel_play/set/stop, music_play/stop/set_volume

**Architecture:**
- Main thread buffers AudioCommands during update/render
- Commands sent to audio server via mpsc channel after rendering
- Audio server processes commands on background thread
- Sounds are Arc<Vec<i16>> for efficient cloning across thread boundary

**Implementation Details:**
- Custom rodio Source implementation for raw PCM playback
- 22,050 Hz sample rate (PS1/N64 authentic)
- Mono 16-bit signed PCM format
- Looping via rodio's repeat_infinite()
- Volume clamping to 0.0-1.0 range
- Pan parameter accepted but not yet implemented (rodio limitation)

**Files Modified:**
- `emberware-z/src/audio.rs` - Full rewrite: AudioServer, ZAudio, SoundSource implementation
- `emberware-z/src/ffi/mod.rs` - Added 8 audio FFI functions (load_sound, play_sound, etc.)
- `emberware-z/src/app.rs` - Process audio commands after rendering each frame

**Test Results:** 520 tests passing

---

### **[STABILITY] Suppress audio stub warnings**

**Completed:** Suppressed dead code warnings for audio infrastructure stub

Audio infrastructure items (constants, fields, methods) are unused in the stub implementation
but will be used once full rodio playback is implemented. Added `#[allow(dead_code)]` attributes
with explanatory comments to maintain clean build while documenting future use.

**Files Modified:**
- `emberware-z/src/audio.rs` - Suppressed MAX_CHANNELS, SAMPLE_RATE, Sound.data, AudioCommand variants, process_commands
- `emberware-z/src/state.rs` - Suppressed sounds and next_sound_handle fields

**Test Results:** 520 tests passing, zero compiler warnings

**NOTE:** This task was superseded by "Complete audio backend playback" which removed the stub and implemented full audio.

---

### **[FEATURE] Implement retained mesh drawing**

**Completed:** Implemented in `emberware-z/src/graphics/mod.rs`

**Implementation:**
- Retrieves mesh data from retained mesh storage using mesh handle
- Converts byte offsets to vertex/index counts for draw commands
- Submits draw calls with proper vertex format and transform
- Supports both indexed and non-indexed meshes

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Implemented ZDrawCommand::DrawMesh case in process_draw_commands()

---

### **[FEATURE] Implement billboard rendering**

**Completed:** Implemented in `emberware-z/src/graphics/mod.rs`

**Implementation:**
- Generates camera-facing quad geometry based on billboard mode:
  - Mode 1: Spherical (faces camera completely using view matrix)
  - Mode 2: Cylindrical Y-axis (rotates around Y to face camera)
  - Mode 3: Cylindrical X-axis (rotates around X to face camera)
  - Mode 4: Cylindrical Z-axis (rotates around Z to face camera)
- Extracts position from transform matrix
- Calculates right and up vectors based on camera orientation
- Generates quad vertices in POS_UV_COLOR format with proper UV mapping
- Submits as indexed triangles (2 triangles, 6 indices)

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Implemented ZDrawCommand::DrawBillboard case in process_draw_commands()

---

### **[FEATURE] Implement 2D sprite rendering**

**Completed:** Implemented in `emberware-z/src/graphics/mod.rs`

**Implementation:**
- Generates quad geometry in screen space (pixel coordinates)
- Supports optional UV rectangles for texture atlas usage
- Supports optional origin offset for rotation pivot
- Supports rotation around origin point
- Applies color tint from RGBA value
- Uses POS_UV_COLOR format (format 3)
- Renders with identity transform and no depth test (2D overlay)
- Submits as indexed triangles (4 vertices, 6 indices)

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Implemented ZDrawCommand::DrawSprite case in process_draw_commands()

---

### **[FEATURE] Implement 2D rectangle rendering**

**Completed:** Implemented in `emberware-z/src/graphics/mod.rs`

**Implementation:**
- Generates solid color quad in screen space (pixel coordinates)
- No texture - uses vertex color only
- Uses POS_COLOR format (format 2, no UV)
- Applies color from RGBA value
- Renders with identity transform and no depth test (2D overlay)
- Submits as indexed triangles (4 vertices, 6 indices)

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Implemented ZDrawCommand::DrawRect case in process_draw_commands()

---

### **[STABILITY] Fix compiler warnings for unused code**

**Completed:** Fixed warnings in `emberware-z/src/state.rs`

**Implementation:**
- Marked `CameraState::view_projection_matrix()` with `#[allow(dead_code)]` - public API method for games
- Removed redundant `render_mode` field from `ZFFIState` (render mode stored in `init_config.render_mode`)
- Marked `ZFFIState::new()` with `#[cfg(test)]` - only used in test code

**Files Modified:**
- `emberware-z/src/state.rs` - Removed dead code and marked public API appropriately

---

### **[STABILITY] Fix clippy warnings for code quality**

**Completed:** Addressed clippy warnings across the codebase

**Implementation:**
- Added `Default` derive and implementation for `GameStateWithConsole` (suggested by clippy)
- Removed needless borrow in mesh loading code
- Replaced manual range checks with `.contains()` for cleaner code (2 instances)
- Used `#[derive(Default)]` instead of manual impl for `LightsUniforms`
- Replaced manual div_ceil with `.div_ceil()` method
- Allowed complex type for local HashMap cache (local optimization, no benefit to type alias)

**Files Modified:**
- `core/src/wasm/state.rs` - Added Default implementation
- `emberware-z/src/app.rs` - Removed needless borrow
- `emberware-z/src/ffi/mod.rs` - Used range contains
- `emberware-z/src/graphics/mod.rs` - Used range contains, div_ceil, allowed type complexity
- `emberware-z/src/graphics/render_state.rs` - Derived Default

---

### **[FEATURE] Implement audio backend infrastructure**

**Completed:** Audio infrastructure in place (playback implementation pending)

**Implementation:**
- Created `audio.rs` module with Sound and AudioCommand types
- Implemented ZAudio backend with rollback-aware command buffering
- Added audio state to ZFFIState (sounds, audio_commands, next_sound_handle)
- Integrated ZAudio with Console trait via create_audio()
- Commands are buffered per frame and cleared after processing
- Rollback mode support (commands discarded during replay)

**What Was Completed:**
1. ✅ Sound struct and sounds Vec<Option<Sound>>
2. ✅ AudioCommand enum with all command types
3. ✅ audio_commands Vec buffering in ZFFIState
4. ✅ ZAudio with process_commands() and set_rollback_mode()
5. ✅ Integration with console initialization

**What Remains (see TODO):**
- Thread-safe rodio integration (audio server thread + channels)
- Actual audio playback implementation
- FFI functions (load_sound, play_sound, channel_*, music_*)

**Files Modified:**
- `emberware-z/src/audio.rs` - New module with audio infrastructure
- `emberware-z/src/main.rs` - Added audio module
- `emberware-z/src/console.rs` - Updated ZAudio impl and create_audio()
- `emberware-z/src/state.rs` - Added audio fields to ZFFIState

---

