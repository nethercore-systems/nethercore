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

### **[POLISH] Performance Optimizations**
**Status:** Completed ✅

**What Was Implemented:**
All high-priority and medium-priority performance optimizations have been completed:

1. ✅ **Vec4 padding (#1)** - Replaced manual padding with vec4 types in SkyUniforms across all shaders
   - Updated all 4 shader files (mode0, mode1, mode2, mode3)
   - Updated Rust SkyUniforms struct to match
   - Eliminates manual padding errors, improves maintainability

2. ✅ **draw_text String allocation (#3)** - Already stores Vec<u8> instead of String
   - No String allocation on every draw_text() call
   - UTF-8 validation deferred to render time

3. ✅ **#[inline] input functions (#5)** - Added to all input FFI hot path functions
   - button_held, button_pressed, button_released
   - stick_axis, left_stick, right_stick
   - trigger_left, trigger_right
   - Reduces call overhead for frequently-called input functions

4. ✅ **Remove Clone from PendingTexture/PendingMesh (#6)** - Prevents accidental clones
   - Removed Clone derive from resource structs
   - Documents intent (resources are moved, not copied)
   - Prevents expensive clones of MB-sized texture data

5. ✅ **#[inline] camera math (#9)** - Added to view/projection matrix methods
   - view_matrix(), projection_matrix(), view_projection_matrix()
   - Helps with register allocation for matrix math

6. ✅ **Dedupe vertex_stride (#10)** - Removed duplicate implementations
   - Consolidated to single canonical implementation
   - Removed duplicate FORMAT constants
   - Ensures consistency across codebase

**Deferred Optimizations:**
- #2 (Array copy) - Negligible impact (16 bytes)
- #4 (Vec clone) - Not cloned in hot path
- #7 (RenderState copy) - Complex refactor, unclear gain
- #11 (Keycode matching) - Unlikely bottleneck

**Impact:**
- Eliminated allocations in hot paths
- Reduced function call overhead
- Improved code maintainability and readability
- Prevented accidental expensive operations

**Compilation:** ✅ All tests passing

---

### **[FEATURE] Implement custom font loading**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ FFI functions already implemented: `load_font()`, `load_font_ex()`, `font_bind()`
- ✅ Font struct already defined in ZFFIState with texture handle, glyph dimensions, codepoint range
- ✅ Support for both fixed-width and variable-width bitmap fonts
- ✅ `generate_text_quads()` function already supports custom fonts via `font_opt` parameter
- ✅ Implemented actual text rendering in `process_draw_commands()`:
  - Looks up custom font by handle (0 = built-in font)
  - Maps custom font texture handle to graphics texture handle
  - Generates text quads with proper UV coordinates for glyph atlas
  - Submits quads as indexed triangles (POS_UV_COLOR format)
  - Uses built-in font texture as fallback if custom texture not found
- ✅ Text rendering uses 2D screen space (identity transform, no depth test)
- ✅ All 518 tests passing (155 in core + 363 in emberware-z)

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Implemented DrawText rendering in process_draw_commands

**Impact:**
- Game developers can now load custom bitmap fonts from texture atlases
- Fixed-width fonts for retro aesthetics (e.g., 8×8, 16×16 pixel fonts)
- Variable-width fonts for better readability (each character can have custom width)
- UTF-8 compatible - games provide glyphs for any codepoints they need
- Built-in 8×8 font remains available for quick debugging (font handle 0)
- Fonts arranged in 16-column grids in texture atlas (PS1/N64 style)

**Compilation:** ✅ All tests passing

---

### **[REFACTOR] Eliminate redundant state mutation in draw command processing**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ Added `CommandBuffer::append_vertex_data()` method to append vertex data and return base_vertex index
- ✅ Added `CommandBuffer::append_index_data()` method to append index data and return first_index
- ✅ Added `CommandBuffer::add_command()` method to directly push DrawCommand to the commands vec
- ✅ Refactored `process_draw_commands()` to directly convert ZDrawCommand → DrawCommand without state mutation
- ✅ Added helper functions `convert_matcap_blend_mode()` and `map_texture_handles()` for clean conversion
- ✅ Deleted obsolete methods: `execute_draw_command()`, `draw_triangles()`, `draw_triangles_indexed()`, `bind_textures_from_game()`
- ✅ All 518 tests passing (155 in core + 363 in emberware-z)

**Files Modified:**
- `emberware-z/src/graphics/command_buffer.rs` - Added append_vertex_data, append_index_data, add_command methods
- `emberware-z/src/graphics/mod.rs` - Refactored process_draw_commands to eliminate unpack-set-read-repack cycle, deleted obsolete methods

**Impact:**
- 🚀 Eliminates wasteful unpack-set-read-repack cycle in draw command processing
- 🚀 Fewer function calls per draw command (direct data flow)
- 🚀 No state mutations on ZGraphics (better for future multi-threading)
- 🚀 More cache-friendly (direct data flow, no intermediate state)
- 🧹 Cleaner architecture (one-to-one ZDrawCommand → DrawCommand mapping)
- 🧹 ~150 lines of obsolete code removed

**Compilation:** ✅ All tests passing

---

### **[FEATURE] Implement matcap blend modes**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ Updated MaterialUniforms struct in Mode 1 shader to include `matcap_blend_modes: vec4<u32>` field
- ✅ Added `rgb_to_hsv()` and `hsv_to_rgb()` helper functions for HSV color space conversion
- ✅ Added `blend_colors()` function supporting three blend modes:
  - Mode 0: Multiply (default matcap behavior)
  - Mode 1: Add (for glow/emission effects)
  - Mode 2: HSV Modulate (for hue shifting and iridescence)
- ✅ Updated fragment shader to use `blend_colors()` for each matcap slot (slots 1-3)
- ✅ GPU integration already completed in previous session (material buffer cache includes blend modes)
- ✅ All 518 tests passing ✓ (155 in core + 363 in emberware-z)

**Files Modified:**
- `emberware-z/shaders/mode1_matcap.wgsl` - Updated MaterialUniforms, added color blending functions, updated fragment shader

**Impact:**
- Game developers can now use different blend modes for matcaps to achieve various artistic effects
- Multiply mode maintains traditional matcap behavior
- Add mode enables glow and emission effects
- HSV Modulate mode enables dynamic hue shifting and iridescence effects
- Each of the 3 matcap slots (1-3) can use independent blend modes

**Compilation:** ✅ All tests passing

---

### **[POLISH] Performance Optimizations - Bone Matrix Investigation**
**Status:** Completed - No optimization needed
**Investigation Results:**
- Task #8: Investigated Vec<Mat4> cloning in RenderState for bone matrices
- Finding: `bone_matrices: Vec<Mat4>` exists in `ZFFIState` (emberware-z/src/state.rs:279)
- Finding: Bone matrices are NOT cloned - they're stored once in ZFFIState and not copied into DrawCommand variants
- Finding: GPU skinning FFI is implemented but not yet wired up to rendering backend
- Conclusion: No performance issue exists - bone matrices would be consumed directly from ZFFIState during rendering
- No changes needed ✓

**Impact:**
- Confirmed that bone matrix handling is already optimal
- No unnecessary cloning occurs in the hot path
- Documents architecture: bone matrices stored in ZFFIState, not in per-draw-command state

---

### **[FEATURE] Implement matcap blend modes (Partial - GPU Integration)**
**Status:** GPU integration complete, shader implementation pending
**Changes Made:**
- ✅ Added `matcap_blend_modes: [MatcapBlendMode; 4]` field to DrawCommand struct
- ✅ Updated command buffer to capture matcap blend modes from RenderState
- ✅ Updated material buffer structure to include `matcap_blend_modes: [u32; 4]`
- ✅ Updated material buffer cache key to include blend modes (5-tuple)
- ✅ All 518 tests passing ✓ (155 in core + 363 in emberware-z)

**Remaining work:**
- Update Mode 1 shader (mode1_matcap.wgsl) to use blend modes:
  - Add `matcap_blend_modes: vec4<u32>` field to MaterialUniforms struct
  - Add `rgb_to_hsv()` and `hsv_to_rgb()` helper functions
  - Add `blend_colors()` function supporting modes 0-2 (Multiply/Add/HSV Modulate)
  - Update fragment shader to use `blend_colors()` instead of direct multiplication

**Files Modified:**
- `emberware-z/src/graphics/command_buffer.rs` - Added matcap_blend_modes field to DrawCommand
- `emberware-z/src/graphics/mod.rs` - Updated material buffer creation and cache key

---

### **[POLISH] Performance Optimizations - Additional Improvements**
**Status:** Completed
**Changes Made:**
- Task #3: Verified `DrawCommand::DrawText` already stores `Vec<u8>` instead of `String` - no changes needed
- Task #5: Verified all input FFI functions already have `#[inline]` attribute - no changes needed
- Task #6: Removed `Clone` derive from `PendingTexture` and `PendingMesh` structs in `emberware-z/src/state.rs`
  - Also removed `Clone` from `ZFFIState` and `ZDrawCommand` to maintain consistency
  - Verified none of these structs are cloned anywhere in the codebase
  - Prevents accidental expensive clones of large resource data (textures can be MB-sized)
- All 518 tests passing ✓ (155 in core + 363 in emberware-z)

**Impact:**
- Documents intent that these resource structures are moved to GPU, not copied
- Prevents accidental performance issues from cloning large vertex/texture data
- Compiler will now error if anyone tries to clone these structures accidentally

---

### **[REFACTOR] Simplify execute_draw_commands architecture**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ Added `process_draw_commands()` method to ZGraphics that directly consumes ZFFIState
- ✅ Added private `execute_draw_command()` helper method to ZGraphics for processing individual draw commands
- ✅ Added private helper methods `convert_cull_mode()`, `convert_blend_mode()`, and `bind_textures_from_game()` to ZGraphics
- ✅ Updated `app.rs` to call `graphics.process_draw_commands()` instead of `execute_draw_commands()`
- ✅ Removed old `execute_draw_commands()` function from app.rs (~220 lines removed)
- ✅ Removed old helper functions `convert_cull_mode()`, `convert_blend_mode()`, and `bind_textures()` from app.rs
- ✅ Removed unused imports (`ZDrawCommand`, `BlendMode`, `CullMode`) from app.rs
- ✅ All 363 tests passing

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Added new methods for direct draw command processing
- `emberware-z/src/app.rs` - Removed ~250 lines of redundant translation code

**Impact:**
- Cleaner architecture: ZGraphics directly consumes ZFFIState without intermediate unpacking/repacking
- Better performance: No intermediate data copies or translations
- Easier maintenance: Draw command processing logic is now centralized in the graphics module
- Reduced code duplication: Helper functions moved from app.rs to ZGraphics where they belong

**Compilation:** ✅ All tests passing

---

### **[FEATURE] Update PBR shaders to use camera/lights/material uniforms**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ Updated bind group creation in `mod.rs` to conditionally bind lights and camera buffers for modes 2-3
- ✅ Fixed material buffer to include both color AND properties (metallic, roughness, emissive) in single uniform
- ✅ Updated material buffer cache key to include material properties (metallic/roughness/emissive) via float bit representation
- ✅ Updated Mode 2 shader (`mode2_pbr.wgsl`):
  - Changed Light struct to match Rust layout: `direction_and_enabled: vec4<f32>`, `color_and_intensity: vec4<f32>`
  - Updated fragment shader to extract direction, color, intensity from packed vec4s
  - Properly checks `direction_and_enabled.w` for enabled state
- ✅ Updated Mode 3 shader (`mode3_hybrid.wgsl`):
  - Changed from single DirectionalLight to LightUniforms array (same as Mode 2)
  - Uses first light from array (lights_uniforms.lights[0])
  - Extracts direction and color from packed vec4 format
- ✅ All 363 tests passing including shader compilation tests

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Updated bind group creation to conditionally bind lights/camera for modes 2-3, fixed material buffer structure and caching
- `emberware-z/shaders/mode2_pbr.wgsl` - Updated Light struct and fragment shader to match Rust data layout
- `emberware-z/shaders/mode3_hybrid.wgsl` - Updated to use LightUniforms array instead of single DirectionalLight

**Impact:**
- PBR and Hybrid render modes (Mode 2 & 3) now have access to camera position, lights, and material properties
- Dynamic lighting with up to 4 directional lights fully functional in Mode 2
- Mode 3 uses first light for direct lighting as intended
- Material properties (metallic, roughness, emissive) properly transmitted to shaders
- Camera position available for specular calculations and view direction

**Compilation:** ✅ All tests passing

---

### **[FEATURE] Wire up camera and light uniforms to GPU**
**Status:** Completed ✅

**What Was Implemented:**
- ✅ Added `CameraUniforms`, `LightUniform`, `LightsUniforms`, and `MaterialUniforms` structs to `render_state.rs`
- ✅ Added `camera_buffer`, `lights_buffer`, and `material_buffer` to ZGraphics
- ✅ Initialized all three uniform buffers in `ZGraphics::new()`
- ✅ Implemented `update_scene_uniforms()` method that:
  - Computes view and projection matrices from camera state
  - Uploads camera position (for specular calculations in PBR)
  - Uploads 4 directional lights (direction, color, intensity, enabled flag)
  - Uploads material properties (metallic, roughness, emissive)
- ✅ Updated `app.rs` to call `update_scene_uniforms()` before processing draw commands
- ✅ All fields properly initialized with bytemuck Pod/Zeroable traits for GPU upload

**Files Modified:**
- `emberware-z/src/graphics/render_state.rs` - Added uniform structs (CameraUniforms, LightUniform, LightsUniforms, MaterialUniforms)
- `emberware-z/src/graphics/mod.rs` - Added buffers, initialization, update_scene_uniforms() method, and buffer getters
- `emberware-z/src/app.rs` - Added call to update_scene_uniforms() before execute_draw_commands()

**Impact:**
- Camera position, view, and projection are now uploaded to GPU every frame
- Lights (4 directional) are now uploaded to GPU every frame
- Material properties (metallic, roughness, emissive) are now uploaded to GPU every frame
- PBR and Hybrid render modes (Mode 2 & 3) can now access lighting data
- **Note:** Shaders still need to be updated to bind and use these buffers (future task)

**Compilation:** ✅ Successful with only harmless dead code warnings

---

### **[FEATURE] Implement matcap blend modes** (Partial - FFI only)

**Status:** Partially completed - FFI function working, shader integration pending

**Completed:**
1. ✓ Added `MatcapBlendMode` enum to `emberware-z/src/graphics/render_state.rs`
2. ✓ Added `matcap_blend_modes: [MatcapBlendMode; 4]` field to emberware-z's RenderState
3. ✓ Added `matcap_blend_modes: [u8; 4]` field to core's RenderState
4. ✓ Implemented `matcap_blend_mode(slot: u32, mode: u32)` FFI function with validation
5. ✓ Registered FFI function in linker
6. ✓ All 571 tests passing

**Remaining work:**
- Update DrawCommand structs to include matcap_blend_modes field
- Update all DrawCommand construction sites to pass matcap_blend_modes
- Update MaterialUniforms in shader to include blend modes
- Update Mode 1 shader with blend_colors function and rgb_to_hsv/hsv_to_rgb helpers
- Update material buffer creation to pack blend modes into uniforms
- Update cache key to include blend modes

**Notes:**
- FFI function is fully functional and can be called from games
- State is tracked in RenderState but not yet passed to GPU
- Next implementer should update shader uniforms and material buffer logic

---

### **[POLISH] Performance Optimizations - Replace manual padding with Vec4 types in uniforms**
**Status:** Completed
**Changes Made:**
- Updated SkyUniforms struct in all 4 shader files (mode0, mode1, mode2, mode3):
  - Replaced `vec3<f32>` + manual `_pad` fields with `vec4<f32>`
  - Renamed `sun_color` and `sun_sharpness` to `sun_color_and_sharpness: vec4<f32>` (.xyz = color, .w = sharpness)
- Updated shader code to access new fields:
  - `sky.horizon_color` → `sky.horizon_color.xyz`
  - `sky.sun_direction` → `sky.sun_direction.xyz`
  - `sky.sun_color` → `sky.sun_color_and_sharpness.xyz`
  - `sky.sun_sharpness` → `sky.sun_color_and_sharpness.w`
- Updated Rust SkyUniforms struct in `emberware-z/src/graphics/render_state.rs`:
  - Changed all fields from `[f32; 3] + _pad` to `[f32; 4]`
  - Updated Default impl to use vec4 layout
  - Updated safety comments to reflect new structure
- Updated `set_sky()` in `emberware-z/src/graphics/mod.rs` to pack data into vec4 fields
- Updated tests to use new field structure
- All 571 tests passing ✓ (194 in core + 377 in emberware-z)

**Impact:**
- Improved code readability - no more manual padding fields
- Eliminates manual padding errors
- Makes future uniform additions easier
- Same memory layout (64 bytes) and performance

---

### **[POLISH] Performance Optimizations - Additional Quick Wins**
**Status:** Completed
**Changes Made:**
- Task #9: Added `#[inline]` attribute to camera math methods in `core/src/wasm/camera.rs`:
  - `view_matrix()` - Called once per frame for view transform
  - `projection_matrix()` - Called once per frame for projection
  - `view_projection_matrix()` - Computes combined VP matrix
- Task #10: Removed duplicate `vertex_stride()` function and FORMAT constants from `emberware-z/src/ffi/mod.rs`:
  - Removed duplicate `vertex_stride()` implementation (lines 648-665)
  - Removed duplicate FORMAT_UV, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED constants
  - Added import from `crate::graphics` to use canonical implementations from `vertex.rs`
  - Kept MAX_VERTEX_FORMAT constant as it's used for validation
- All 571 tests passing ✓ (194 in core + 377 in emberware-z)

### **[POLISH] Performance Optimizations - Quick Wins**
**Status:** Completed
**Changes Made:**
- Task #3: `DrawCommand::DrawText` already stores `Vec<u8>` instead of `String`, eliminating String allocation
- Task #5: Added `#[inline]` attribute to all input FFI hot path functions:
  - `right_stick_x`, `right_stick_y` (were missing inline)
  - `left_stick`, `right_stick` (were missing inline)
  - `trigger_left`, `trigger_right` (were missing inline)
  - Other input functions already had `#[inline]` applied
- Task #6: Removed `Clone` derive from `PendingTexture` and `PendingMesh` structs
  - These are moved via `.drain()`, not cloned
  - Prevents accidental expensive clones of large resource data
- All 571 tests passing ✓ (194 in core + 377 in emberware-z)

### **[STABILITY] Refactor rollback to use automatic WASM linear memory snapshotting**
**Status:** Completed
**Changes Made:**
- Implemented automatic WASM linear memory snapshotting in `GameInstance::save_state()` and `GameInstance::load_state()`
- Games no longer need to implement manual serialization callbacks
- Host snapshots entire WASM linear memory transparently
- Comprehensive test coverage for memory snapshotting
- Documentation updated in rollback-architecture.md
- All tests passing ✓

### Remove duplicate TestConsole definitions
**Status:** Completed
**Changes Made:**
- Created shared `test_utils.rs` module with common test utilities
- Moved TestConsole, TestGraphics, TestAudio, and TestInput to shared module
- Updated integration.rs to use shared test utilities (removed 120+ lines)
- Updated runtime.rs to use shared test utilities (removed 90+ lines)
- All 194 tests passing ✓

### Remove reliance on MAX_STATE_SIZE and use console spec provided RAM to limit
**Status:** Completed
**Changes Made:**
- Updated `RollbackStateManager::new(max_state_size)` to accept console-specific RAM limit
- Added `RollbackStateManager::with_defaults()` for backward compatibility
- Updated all `RollbackSession` constructors to accept `max_state_size` parameter
- Added documentation to `MAX_STATE_SIZE` constant explaining it's a fallback
- Consoles now use `console.specs().ram_limit` when creating rollback sessions:
  - Emberware Z: 4MB
  - Emberware Classic: 1MB
- All tests passing ✓
