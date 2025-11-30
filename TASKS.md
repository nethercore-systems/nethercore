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

## TODO

### **[STABILITY] Refactor rollback to use automatic WASM linear memory snapshotting**

**Current State:** Games manually serialize state via FFI callbacks (`save_state(ptr, max_len) -> len` and `load_state(ptr, len)`). This requires boilerplate in every game and is error-prone.

**Target State:** Automatic memory snapshotting as described in [docs/rollback-architecture.md](docs/rollback-architecture.md). The host snapshots entire WASM linear memory transparently. Games require zero serialization code.

**Why This Matters:**
- ✅ Zero boilerplate for game developers
- ✅ Can't forget to serialize fields (entire memory is saved)
- ✅ Works naturally with any data structures (hash maps, dynamic arrays, custom allocators)
- ✅ Smaller effective snapshots (only game state, resources stay in GPU memory)

**Implementation Steps:**

1. **Modify `GameInstance` to snapshot WASM linear memory directly**
   - Location: [core/src/wasm/mod.rs:158-183](core/src/wasm/mod.rs#L158-L183)
   - Change `GameInstance::save_state(&mut self, buffer: &mut [u8]) -> Result<usize>` to:
     ```rust
     pub fn save_state(&mut self) -> Result<Vec<u8>> {
         let memory = self.store.data().memory.context("No memory export")?;
         let mem_data = memory.data(&self.store);
         Ok(mem_data.to_vec())  // Copy entire linear memory
     }
     ```
   - Change `GameInstance::load_state(&mut self, buffer: &[u8]) -> Result<()>` to:
     ```rust
     pub fn load_state(&mut self, snapshot: &[u8]) -> Result<()> {
         let memory = self.store.data().memory.context("No memory export")?;
         let mem_data = memory.data_mut(&mut self.store);
         anyhow::ensure!(snapshot.len() == mem_data.len(),
             "Snapshot size mismatch: {} vs {}", snapshot.len(), mem_data.len());
         mem_data.copy_from_slice(snapshot);
         Ok(())
     }
     ```
   - Remove `save_state_fn` and `load_state_fn` fields from `GameInstance` struct
   - Remove lookup of save_state/load_state exports in `GameInstance::new()`

2. **Update `RollbackStateManager` to use new memory snapshot API**
   - Location: [core/src/rollback/state.rs:188-235](core/src/rollback/state.rs#L188-L235)
   - Change `save_state()` to call `game.save_state()` directly (no buffer passing)
   - Update `MAX_STATE_SIZE` constant to be WASM memory size (typically 1-16MB, configurable)
   - Consider adaptive pool buffer sizing based on actual memory size
   - Ensure `StatePool` buffers are sized appropriately for full memory snapshots

3. **Update `EmberwareConfig` GGRS state type**
   - Location: [core/src/rollback/config.rs:40-44](core/src/rollback/config.rs#L40-L44)
   - `GameStateSnapshot` is already the state type - no changes needed
   - Verify `MAX_STATE_SIZE` is appropriate for WASM memory (update from 1MB default)

4. **Remove FFI save_state/load_state from ALL game examples**
   - Primary location: [examples/platformer/src/lib.rs:746-835](examples/platformer/src/lib.rs#L746-L835)
   - Search all example games for `save_state` and `load_state` exports:
     - `examples/hello-world/src/lib.rs`
     - `examples/triangle/src/lib.rs`
     - `examples/textured-quad/src/lib.rs`
     - `examples/cube/src/lib.rs`
     - `examples/lighting/src/lib.rs`
     - `examples/skinned-mesh/src/lib.rs`
     - `examples/billboard/src/lib.rs`
     - `examples/platformer/src/lib.rs`
   - Delete entire `save_state()` and `load_state()` export functions from any that have them
   - Add comment to each example's lib.rs: `// Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.`
   - Verify platformer still works with rollback (netcode integration tests)
   - Test that examples compile and run correctly after removal

5. **Update FFI documentation**
   - Location: [docs/ffi.md](docs/ffi.md)
   - Remove `save_state(ptr: *mut u8, max_len: u32) -> u32` from save data section
   - Remove `load_state(ptr: *const u8, len: u32)` from save data section
   - Add note: "Rollback state is automatically saved/restored by snapshotting WASM linear memory. Games do not need to implement serialization."
   - Clarify that `save(slot, ptr, len)` and `load(slot, ptr, len)` are for persistent save files, NOT rollback

6. **Update developer guide**
   - Location: [docs/developer-guide.md](docs/developer-guide.md)
   - Update "Best practices for rollback-safe code" section
   - Remove any mention of manual state serialization
   - Explain that all game state in WASM linear memory is automatically rolled back
   - Add guidance: "Resources (textures, meshes, sounds) are loaded during init() and stay in GPU/host memory. Only their handles (IDs) are in WASM memory, which get snapshotted correctly."
   - Add note about determinism: "Rollback works transparently as long as your update() is deterministic (same inputs → same outputs). Use the provided RNG, don't use external time sources."

7. **Update rollback architecture documentation**
   - Location: [docs/rollback-architecture.md](docs/rollback-architecture.md)
   - Add "Implementation Status" section at top noting this is now the actual implementation
   - Add note about WASM memory size configuration and snapshot size implications
   - Document that this approach is fully transparent to game developers

8. **Add integration test for automatic rollback**
   - Location: [core/src/integration.rs](core/src/integration.rs) or new test file
   - Create test game that allocates dynamic memory (Vec, HashMap, etc.)
   - Verify state is correctly saved and restored without any game-side serialization
   - Test with different memory sizes and allocation patterns
   - Verify resource handles (texture IDs, mesh IDs) survive rollback correctly

9. **Update WASM memory allocation strategy documentation**
   - Add to CLAUDE.md or developer docs
   - Explain WASM linear memory starts small (64KB default) and grows via `memory.grow`
   - Document that snapshot size = current allocated linear memory size (not max)
   - Note: Rust's allocator will call `memory.grow` automatically as needed
   - Recommend games pre-allocate stable working set in init() for predictable snapshot sizes

10. **Performance testing and optimization**
    - Profile snapshot/restore performance with realistic game state sizes
    - Measure memory copy overhead (expect ~1-5ms for 1-4MB on modern CPUs)
    - Consider incremental/delta snapshots if full copy becomes bottleneck (deferred optimization)
    - Document expected rollback performance characteristics

**Files to Modify:**
- `core/src/wasm/mod.rs` (GameInstance save/load methods)
- `core/src/rollback/state.rs` (RollbackStateManager integration)
- `core/src/rollback/config.rs` (MAX_STATE_SIZE constant)
- `examples/platformer/src/lib.rs` (remove save_state/load_state exports)
- `examples/*/src/lib.rs` (remove save_state/load_state from all examples if present)
- `docs/ffi.md` (remove save_state/load_state from API reference)
- `docs/developer-guide.md` (update rollback best practices)
- `docs/rollback-architecture.md` (mark as implemented)

**Testing Strategy:**
- Verify all existing integration tests still pass
- Run platformer example in local P2P session and trigger rollbacks
- Monitor snapshot sizes and performance
- Test edge cases: empty memory, large allocations, fragmented heap

**Breaking Changes:**
- Games that export `save_state`/`load_state` will have those exports ignored
- This is a **breaking change** for any games relying on partial state serialization
- Migration path: Remove save_state/load_state exports entirely - rollback is now automatic
- Persistent save files (save()/load() FFI) are unaffected

---

### **[FEATURE] Implement audio backend**

PS1/N64-style audio system with fire-and-forget sounds and managed channels for positional audio.

**Console Audio Specs:**
| Spec | Value | Rationale |
|------|-------|-----------|
| Sample rate | 22,050 Hz | Authentic PS1/N64, half the ROM space |
| Bit depth | 16-bit signed | Standard, good dynamic range |
| Format | Mono, raw PCM (s16le) | Zero parsing, stereo via pan param |
| Managed channels | 16 | PS1/N64 typical (8 SFX + 8 music/ambient) |

**Rollback Behavior & Caveats:**

Audio is NOT part of rollback state. This is industry standard (GGPO, Rollback Netcode) because:
- Sound already left the speakers - can't "un-play" it
- Rewinding audio sounds terrible
- Users tolerate audio glitches better than visual desyncs

**How it works:**
1. Game calls audio FFI functions during `update()`
2. Commands are buffered in `audio_commands: Vec<AudioCommand>`
3. After GGRS confirms the frame, commands are sent to `ZAudio` for playback
4. During rollback replay (`set_rollback_mode(true)`), commands are DISCARDED
5. After rollback, game re-executes with corrected inputs, re-issuing audio commands

**Edge cases implementers must handle:**

| Scenario | What happens | Mitigation |
|----------|--------------|------------|
| Sound triggers, then rollback | Sound already playing, might re-trigger | Discard commands during replay |
| Looping sound starts, then rollback | Loop continues, game might call channel_play again | channel_play on occupied channel should update params, not restart |
| Positional sound panning | Pan was wrong during misprediction | Game must call channel_set() EVERY frame, not just on start |
| Music playing during rollback | Music continues uninterrupted | Music is never affected by rollback - it's "UI layer" |
| Sound should have played but didn't (prediction missed trigger) | Silence where sound should be | Unavoidable - accept it |
| load_sound() called in update() | Would re-load during replay! | Enforce init-only for load_sound() |

**Critical implementation details:**

1. **Discard vs mute**: When `set_rollback_mode(true)`, DISCARD all audio commands entirely.
   Don't just mute output - if you mute and still process commands, channel state diverges.

2. **channel_play on occupied channel**: If channel 5 is playing sound A and game calls
   `channel_play(5, A, vol, pan, loop)` again, DON'T restart the sound. Just update vol/pan.
   This handles the "looping sound survives rollback" case gracefully.

3. **channel_set during rollback**: These SHOULD still be processed (not discarded) so that
   when rollback ends, positional sounds have correct pan/volume immediately.
   Only play_sound/channel_play/music_play are discarded.

4. **In-flight sounds**: Sounds that started before rollback continue playing to completion.
   Don't stop them on rollback start - that sounds jarring. Let them finish naturally.

5. **Audio buffer latency**: Hardware audio has ~20-50ms latency. Audio is always slightly
   "behind" visuals. This is fine - humans don't notice small audio/visual desync.

**Game developer guidance (for docs):**
- Use `play_sound()` for one-shots (hits, jumps, pickups) - fire and forget
- Use `channel_play()` + `channel_set()` every frame for positional sounds (engines, footsteps)
- Don't rely on frame-perfect audio sync in networked games
- Keep sound effects short (<1 sec) so mispredictions are less noticeable
- Music should be ambient/looping, not synced to gameplay events

**FFI Functions:**
```rust
// Load raw PCM sound data (22.05kHz, 16-bit signed, mono)
load_sound(data_ptr: *const u8, byte_len: u32) -> u32

// Fire-and-forget (one-shot sounds: gunshots, jumps, coins)
play_sound(sound: u32, volume: f32, pan: f32)  // uses next free channel

// Managed channels (positional/looping: engines, ambient, footsteps)
channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: bool)
channel_set(channel: u32, volume: f32, pan: f32)  // update each frame for positional
channel_stop(channel: u32)

// Music (dedicated, always loops)
music_play(sound: u32, volume: f32)
music_stop()
music_set_volume(volume: f32)
```

**Implementation steps:**
1. Add `Sound` struct and `sounds: Vec<Sound>` to GameState
2. Add `AudioCommand` enum (Play, ChannelPlay, ChannelSet, ChannelStop, MusicPlay, etc.)
3. Add `audio_commands: Vec<AudioCommand>` to GameState (buffered per frame)
4. Implement `ZAudio` with rodio backend:
   - 16 channel mixer
   - Dedicated music channel
   - `process_commands()` called after confirmed frames
5. Wire up `Audio::set_rollback_mode()` to discard commands during replay
6. Register FFI functions

**Stubs to replace:** `emberware-z/src/console.rs` - `ZAudio::play()`, `ZAudio::stop()`, `create_audio()`

---

### **[FEATURE] Implement custom font loading**

Allow games to load bitmap fonts for `draw_text()` beyond the built-in 8x8 ASCII font.

**Design (PS1/N64 style - bitmap font atlases):**
- Games embed font textures with glyph grids
- Fixed-width and variable-width support
- UTF-8 compatible (game provides glyphs for any codepoints they need)
- Built-in font (already implemented) remains default for quick debugging

**FFI Functions:**
```rust
// Fixed-width bitmap font
load_font(texture: u32, char_width: u8, char_height: u8, first_codepoint: u32, char_count: u32) -> u32

// Variable-width bitmap font (widths array has char_count entries)
load_font_ex(texture: u32, widths_ptr: *const u8, char_height: u8, first_codepoint: u32, char_count: u32) -> u32

// Bind font for subsequent draw_text calls (0 = built-in)
font_bind(font_handle: u32)
```

**Implementation steps:**
1. Add `Font` struct with texture handle, glyph dimensions, codepoint range, optional width array
2. Add `fonts: Vec<Font>` to GameState
3. Add `current_font: u32` to RenderState (0 = built-in)
4. Modify `draw_text` to look up glyphs from current font
5. Update `generate_text_quads()` to handle variable-width fonts

---

### **[FEATURE] Implement matcap blend modes**

Extend matcap system (Mode 1) with multiple blend modes for artistic flexibility.

**Supported modes:**
| Value | Mode | Effect |
|-------|------|--------|
| 0 | Multiply | Standard matcap (current behavior) |
| 1 | Add | Glow/emission effects |
| 2 | HSV Modulate | Hue shifting, iridescence |

**FFI Function:**
```rust
matcap_blend_mode(slot: u32, mode: u32)  // slot 1-3, mode 0-2
```

**Implementation steps:**
1. Add `blend_mode: u8` field to matcap slot tracking in RenderState
2. Add `matcap_blend_mode` FFI function with validation
3. Update Mode 1 shader template with blend_colors switch statement
4. Add rgb_to_hsv/hsv_to_rgb helper functions to shader

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

### **[POLISH] Performance optimization**

- Render batching already implemented in CommandBuffer
- Profile and optimize hot paths - requires game execution to measure

---

## IN PROGRESS

(empty)

---

## DONE (Recent)

See [CHANGELOG.md](CHANGELOG.md) for full development history. Recent completions:

- **Codebase cleanup** (2025-11-30)
  - Removed unused `emberware-z/pbr-lite.wgsl` (duplicate of code in mode2_pbr.wgsl)
  - Removed unused `shader_gen_example/` directory (reference code from different project)
  - Verified stub files are intentional: `download.rs`, `runtime/mod.rs`
  - All 573 tests passing

- **[STABILITY] Fix clippy warnings in test code**
  - Fixed 13 clippy warnings across test code in app.rs and ffi/mod.rs
  - All 573 tests passing (196 core + 377 emberware-z)

- **[STABILITY] Add session cleanup on exit**
  - Added explicit `game_session = None` cleanup when exiting Playing mode via ESC key
  - Ensures proper drop of Runtime, RollbackSession, and game resources

- **[STABILITY] Integrate session events into app** (disconnect handling)
  - Added `handle_session_events()` method to App that polls `Runtime::handle_session_events()` each frame
  - `SessionEvent::Disconnected` → transitions to Library with error message
  - `SessionEvent::Desync` → transitions to Library with desync error
  - Network interruption warnings in debug overlay

- **[STABILITY] Implement local network testing**
  - Created `LocalSocket` UDP wrapper implementing GGRS `NonBlockingSocket<String>` trait
  - Allows P2P sessions without matchbox signaling server
  - 12 new tests for socket binding, connecting, and UDP communication

- **[STABILITY] Replace unsafe transmute with wgpu::RenderPass::forget_lifetime()**
  - wgpu 23 provides safe alternative to unsafe transmute
  - Removed unsafe block from `emberware-z/src/app.rs`

- **Wire up game execution in Playing mode**
  - Implemented game loop integration in App::render()
  - Added process_pending_resources() and execute_draw_commands()
  - Input flow: InputManager → map_input() → game.set_input() → FFI
  - Error handling: Runtime errors return to Library with error message

- **[STABILITY] Add comprehensive test coverage**
  - 573 total tests (196 core + 377 emberware-z)
  - Integration tests for game lifecycle, rollback, multi-player input, resource limits
  - FFI validation tests covering all error conditions and edge cases
  - Graphics pipeline tests (shader compilation, vertex formats, texture binding, render state)
  - Input system tests (deadzone, player slots, keyboard mapping)

- **Implement multiplayer player model**
  - Added `PlayerSessionConfig` struct for configuring local vs remote players
  - Max 4 players total with flexible local/remote assignment via bitmask
  - Integration with `RollbackSession`

- **Create comprehensive examples**
  - `hello-world` — Minimal no_std WASM game
  - `triangle` — Immediate mode 3D with transforms
  - `textured-quad` — Texture loading and 2D sprites
  - `cube` — Retained mode 3D with indexed meshes
  - `lighting` — PBR lighting with dynamic lights
  - `skinned-mesh` — GPU skeletal animation
  - `billboard` — Billboard sprites in 3D space
  - `platformer` — Full mini-game with rollback netcode

---

## DEFERRED (Emberware Classic)

These tasks are deferred until Emberware Z is complete. Classic shares the core framework but has its own console implementation.

### Classic Console Implementation

- **Create Emberware Classic `Console` implementation**
  - Implement `Console` trait for SNES/Genesis aesthetic
  - Define Classic-specific specs (384×216 default, 60fps, 4MB RAM, 2MB VRAM)
  - 8 resolution options (4× 16:9 + 4× 4:3, pixel-perfect to 1080p)

- **Implement Classic graphics backend**
  - `ClassicGraphics` implementing `Graphics` trait
  - 2D-only rendering pipeline (no 3D transforms)
  - Sprite layers (4 layers, back-to-front)
  - Tilemap system (4 layers with parallax scrolling)
  - Palette swapping (256-color indexed textures)

- **Implement Classic-specific FFI functions**
  - Textures: `load_texture`, `texture_bind`
  - Sprites: `draw_sprite`, `draw_sprite_region`, `draw_sprite_ex` (with flip)
  - Sprite control: `sprite_layer`, `draw_sprite_flipped`
  - Tilemaps: `tilemap_create`, `tilemap_set_texture`, `tilemap_set_tile`, `tilemap_set_tiles`, `tilemap_scroll`
  - Palettes: `palette_create`, `palette_bind`
  - Input: `button_held`, `button_pressed`, `button_released`, `dpad_x`, `dpad_y`
  - Render state: `blend_mode`, `texture_filter`

### Classic Examples

- **Create `sprites` example (Classic)**
  - Sprite sheets with `draw_sprite_region`
  - Sprite flipping with `draw_sprite_ex`
  - Sprite layers and priority
  - D-pad input for movement

- **Create `tilemap` example (Classic)**
  - Demonstrates `tilemap_create` and `tilemap_scroll`
  - Multiple parallax layers
  - Tile animation via `tilemap_set_tile`
  - Sprite/tilemap layer interleaving

- **Create `palette-swap` example (Classic)**
  - Demonstrates `palette_create` and `palette_bind`
  - Enemy color variants from single sprite
  - Damage flash effect
  - Dynamic palette cycling

- **Create `platformer` example (Classic)**
  - Full mini-game demonstrating Classic features
  - Tilemap-based levels with scrolling
  - Animated sprite character
  - Parallax background layers
  - 6-button input scheme
