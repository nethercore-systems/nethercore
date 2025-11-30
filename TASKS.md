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

### Core Structs are specific to EmberwareZ
- EmberwareZ Specific rendering data exists in the wasm/render.rs file
- In fact, the wasm/input.rs is also tied.
- This needs to be removed and made generic, other consoles (like the future Emberware Classic) will have their own invocation.
- Please check the project architecture, and likely refactor many of the wasm/ folder so that it is console agnostic.

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

### **[POLISH] Performance Optimizations**

Quick wins for reducing allocations, copies, and overhead in hot paths.

---

#### 1. **[HIGH] Replace manual padding with Vec4 types in uniforms**

**Location:** Shader files and Rust uniform structs
- `emberware-z/shaders/mode1_matcap.wgsl:14-23`
- `emberware-z/shaders/mode2_pbr.wgsl` (similar)
- `emberware-z/shaders/mode3_hybrid.wgsl` (similar)
- `emberware-z/src/graphics/mod.rs` (SkyUniforms Rust struct)

**Current Code (UGLY):**
```wgsl
struct SkyUniforms {
    horizon_color: vec3<f32>,
    _pad0: f32,              // Manual padding
    zenith_color: vec3<f32>,
    _pad1: f32,              // Manual padding
    sun_direction: vec3<f32>,
    _pad2: f32,              // Manual padding
    sun_color: vec3<f32>,
    sun_sharpness: f32,
}
```

**Proposed Fix (CLEAN):**
```wgsl
struct SkyUniforms {
    horizon_color: vec4<f32>,      // .w unused but clean
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}
```

**Impact:** HIGH - Improves code readability, eliminates manual padding errors, makes future uniform additions easier.

**Implementation:**
1. Update SkyUniforms struct in all 3 shader templates (mode1, mode2, mode3)
2. Update Rust-side SkyUniforms struct in `emberware-z/src/graphics/mod.rs` to match
3. Update `set_sky()` FFI to pack sun_sharpness into sun_color.w
4. Update shader code that reads `sky.sun_sharpness` to `sky.sun_color_and_sharpness.w`
5. Search for other uniform structs with `_pad` fields and apply same pattern

---

#### 2. **[HIGH] Eliminate array copy in sprite/billboard functions**

**Location:** `emberware-z/src/ffi/mod.rs:1357` and similar lines in draw_billboard, draw_sprite_region, etc.

**Current Code:**
```rust
state.draw_commands.push(DrawCommand::DrawSprite {
    // ... other fields
    bound_textures: state.render_state.bound_textures,  // Copies [u32; 4]
});
```

**Issue:** Every draw call copies the 16-byte texture slot array. Called 100+ times per frame.

**Proposed Fix:** Store a reference/index to render state instead of copying fields
```rust
// Option A: Store render state index/hash (if render states are deduplicated)
bound_textures_key: u32,  // Index into deduped render states

// Option B: Accept the copy (it's only 16 bytes and likely in cache)
// Keep as-is, this is a micro-optimization
```

**Impact:** MEDIUM - 1.6KB saved per 100 draw calls. Likely not worth refactoring unless profiling shows it's hot.

**Recommendation:** Defer until profiling shows this matters. The copy is cheap (16 bytes, stack-to-stack).

---

#### 3. **[HIGH] Eliminate String allocation in draw_text()**

**Location:** `emberware-z/src/ffi/mod.rs:1430`

**Current Code:**
```rust
let text_string = match std::str::from_utf8(bytes) {
    Ok(s) => s.to_string(),  // ❌ Allocates String on every call
    Err(e) => {
        warn!("draw_text: invalid UTF-8 string: {}", e);
        return;
    }
};
```

**Proposed Fix:** Change DrawCommand::DrawText to store bytes + validate later
```rust
// In FFI:
let text_bytes = bytes.to_vec();  // Already a copy, unavoidable
state.draw_commands.push(DrawCommand::DrawText {
    text: text_bytes,  // Vec<u8> instead of String
    // ...
});

// In graphics backend when rendering:
let text_str = std::str::from_utf8(&cmd.text).unwrap_or("");
```

**Impact:** HIGH - Eliminates allocation for every draw_text() call. Common in UI-heavy games.

**Implementation:**
1. Change `DrawCommand::DrawText::text` from `String` to `Vec<u8>`
2. Update FFI to store bytes directly (remove `to_string()`)
3. Update graphics backend to decode UTF-8 during rendering (one-time per text draw)

---

#### 4. **[HIGH] Reduce Vec clones in DrawCommand variants**

**Location:** `core/src/wasm/draw.rs:29-50`

**Current Code:**
```rust
#[derive(Debug, Clone)]  // ❌ Clones Vec<f32> and Vec<u32>
pub enum DrawCommand {
    DrawTriangles {
        vertex_data: Vec<f32>,   // Cloned if DrawCommand is cloned
        // ...
    },
    DrawTrianglesIndexed {
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,    // Cloned if DrawCommand is cloned
        // ...
    },
    // ...
}
```

**Issue:** If DrawCommand is ever cloned (e.g., during state sorting), large vertex buffers get deep-copied.

**Analysis:** After optimization #3 in previous session, we sort commands in-place, so DrawCommand is NEVER cloned in hot path. The `Clone` derive is only used for debugging/tests.

**Proposed Fix:** None needed - sorting is already in-place. Keep `Clone` for flexibility.

**Impact:** LOW - Not an issue in current implementation. Monitor if DrawCommand cloning appears in profiles.

---

#### 5. **[MEDIUM-HIGH] Add #[inline] to input FFI hot path functions**

**Location:** `emberware-z/src/ffi/mod.rs` - input functions

**Current Code:**
```rust
fn button_held(mut caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    // Called 10-20 times per frame per player
}

fn stick_axis(mut caller: Caller<'_, GameState>, player: u32, axis: u32) -> f32 {
    // Called 2-4 times per frame per player
}
```

**Proposed Fix:**
```rust
#[inline]
fn button_held(mut caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    // ...
}

#[inline]
fn stick_axis(mut caller: Caller<'_, GameState>, player: u32, axis: u32) -> f32 {
    // ...
}
```

**Impact:** MEDIUM-HIGH - Input functions are called many times per frame. Inlining reduces call overhead.

**Implementation:** Add `#[inline]` to all input FFI functions (button_held, button_pressed, button_released, stick_axis, trigger_value, etc.)

---

#### 6. **[MEDIUM] Remove Clone derive from PendingTexture and PendingMesh**

**Location:** `core/src/wasm/draw.rs:8, 17`

**Current Code:**
```rust
#[derive(Debug, Clone)]  // ❌ Unnecessary - these are never cloned
pub struct PendingTexture {
    pub data: Vec<u8>,  // Can be MB-sized texture data
}

#[derive(Debug, Clone)]  // ❌ Unnecessary
pub struct PendingMesh {
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u32>>,
}
```

**Proposed Fix:**
```rust
#[derive(Debug)]  // Remove Clone - these are moved, not cloned
pub struct PendingTexture { /* ... */ }

#[derive(Debug)]
pub struct PendingMesh { /* ... */ }
```

**Impact:** MEDIUM - Prevents accidental clones of large resource data. Documents intent (these are moved to GPU, not copied).

**Verification:** Search codebase for `.clone()` calls on PendingTexture/PendingMesh - should be none.

---

#### 7. **[MEDIUM] Reduce render state field copying**

**Location:** `core/src/wasm/render.rs:48-77`

**Current Code:** Every DrawCommand copies multiple RenderState fields:
```rust
state.draw_commands.push(DrawCommand::DrawMesh {
    color: state.render_state.color,
    depth_test: state.render_state.depth_test,
    cull_mode: state.render_state.cull_mode,
    blend_mode: state.render_state.blend_mode,
    bound_textures: state.render_state.bound_textures,
    // ...
});
```

**Proposed Fix:** Store a snapshot of RenderState or index into deduped states
```rust
// Option A: Store RenderState snapshot
render_state: RenderState,  // 32 bytes total

// Option B: Dedupe and store index
render_state_key: u16,  // Index into Vec<RenderState>
```

**Impact:** MEDIUM - Simplifies DrawCommand variants, reduces field duplication. Trade-off: adds indirection during rendering.

**Recommendation:** Defer - current approach is simple and performant. Only optimize if profiling shows issue.

---

#### 8. **[MEDIUM] Eliminate Vec<Mat4> clone in RenderState**

**Location:** `core/src/wasm/render.rs` (if bone matrices stored in RenderState)

**Analysis Needed:** Check if bone matrices (for skinned meshes) are stored in RenderState or passed separately.

**If they're in RenderState:**
```rust
pub struct RenderState {
    bone_matrices: Vec<Mat4>,  // ❌ Cloned on every skinned draw?
}
```

**Proposed Fix:** Store bones in a separate shared structure, reference by ID
```rust
pub struct GameState {
    bone_sets: Vec<Vec<Mat4>>,  // Shared pool
}

pub struct DrawCommand {
    bone_set_id: u32,  // Reference instead of clone
}
```

**Impact:** MEDIUM - Eliminates 4KB+ clones for skinned meshes (256 bones × 64 bytes).

**Implementation:** Audit RenderState and DrawCommand for Vec<Mat4> fields, refactor to use shared bone set pool.

---

#### 9. **[MEDIUM] Add #[inline] to camera math methods**

**Location:** `emberware-z/src/graphics/camera.rs` (if it exists) or wherever view/projection matrices are computed

**Current Code:**
```rust
impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        // Matrix math
    }

    pub fn projection_matrix(&self) -> Mat4 {
        // Matrix math
    }
}
```

**Proposed Fix:**
```rust
impl Camera {
    #[inline]
    pub fn view_matrix(&self) -> Mat4 { /* ... */ }

    #[inline]
    pub fn projection_matrix(&self) -> Mat4 { /* ... */ }
}
```

**Impact:** MEDIUM - Called once per frame, but inlining helps with register allocation for matrix math.

**Implementation:** Add `#[inline]` to camera methods and other hot math helpers (transform composition, etc.)

---

#### 10. **[LOW] Remove duplicate vertex_stride() function**

**Location:** Search for `fn vertex_stride` across codebase

**Issue:** If vertex_stride is defined in multiple places (e.g., graphics/vertex.rs and graphics/buffer.rs), consolidate to single source.

**Proposed Fix:**
```rust
// In graphics/vertex.rs (canonical location):
pub const fn vertex_stride(format: u8) -> u32 {
    // Canonical implementation
}

// Remove from other files, import this one
```

**Impact:** LOW - Reduces code duplication, ensures consistency.

**Verification:** `rg "fn vertex_stride"` should show only ONE definition.

---

#### 11. **[LOW-MEDIUM] Optimize keycode matching**

**Location:** `emberware-z/src/input.rs` (keyboard to button mapping)

**Current Code (hypothetical):**
```rust
match keycode {
    KeyCode::KeyW => Some(BUTTON_DPAD_UP),
    KeyCode::KeyS => Some(BUTTON_DPAD_DOWN),
    KeyCode::KeyA => Some(BUTTON_DPAD_LEFT),
    KeyCode::KeyD => Some(BUTTON_DPAD_RIGHT),
    // ... 20+ more cases
    _ => None,
}
```

**Proposed Fix:** Use a lookup table (array or phf) instead of match
```rust
// At compile time:
static KEYCODE_TO_BUTTON: phf::Map<u32, u16> = phf_map! {
    KeyCode::KeyW as u32 => BUTTON_DPAD_UP,
    // ...
};

// At runtime:
KEYCODE_TO_BUTTON.get(&(keycode as u32)).copied()
```

**Impact:** LOW-MEDIUM - Reduces match overhead. Only matters if keyboard input is polled frequently (not typical for controller-primary games).

**Recommendation:** Defer - match is already fast for ~20 cases. Only optimize if profiling shows this in hot path.

---

**Summary:**

| Priority | Optimization | Estimated Savings | Effort |
|----------|-------------|-------------------|--------|
| HIGH | Vec4 padding (#1) | Readability + future-proofing | Medium |
| HIGH | draw_text String (#3) | 1 alloc per text draw | Low |
| MEDIUM-HIGH | #[inline] input (#5) | 5-10% input overhead | Low |
| MEDIUM | Remove Clone on Pending (#6) | Safety + clarity | Low |
| MEDIUM | #[inline] camera (#9) | Minor perf gain | Low |
| LOW | Dedupe vertex_stride (#10) | Code quality | Low |
| DEFER | Array copy (#2) | Negligible (16 bytes) | N/A |
| DEFER | Vec clone (#4) | Not cloned in hot path | N/A |
| DEFER | RenderState copy (#7) | Complex refactor, unclear gain | N/A |
| DEFER | Bone matrix clone (#8) | Needs investigation first | N/A |
| DEFER | Keycode matching (#11) | Unlikely bottleneck | N/A |

**Implementation Order:**
1. #3 (draw_text String) - Quick win, high impact
2. #5 (#[inline] input) - Quick win, medium impact
3. #6 (Remove Clone) - Quick win, safety improvement
4. #9 (#[inline] camera) - Quick win
5. #10 (Dedupe vertex_stride) - Code quality
6. #1 (Vec4 padding) - Larger refactor, but improves maintainability

---

#### 1. **[HIGH] Replace manual padding with Vec4 types in uniforms**

**Location:** Shader files and Rust uniform structs
- `emberware-z/shaders/mode1_matcap.wgsl:14-23`
- `emberware-z/shaders/mode2_pbr.wgsl` (similar)
- `emberware-z/shaders/mode3_hybrid.wgsl` (similar)
- `emberware-z/src/graphics/mod.rs` (SkyUniforms Rust struct)

**Current Code (UGLY):**
```wgsl
struct SkyUniforms {
    horizon_color: vec3<f32>,
    _pad0: f32,              // Manual padding
    zenith_color: vec3<f32>,
    _pad1: f32,              // Manual padding
    sun_direction: vec3<f32>,
    _pad2: f32,              // Manual padding
    sun_color: vec3<f32>,
    sun_sharpness: f32,
}
```

**Proposed Fix (CLEAN):**
```wgsl
struct SkyUniforms {
    horizon_color: vec4<f32>,      // .w unused but clean
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_and_sharpness: vec4<f32>,  // .xyz = color, .w = sharpness
}
```

**Impact:** HIGH - Improves code readability, eliminates manual padding errors, makes future uniform additions easier.

**Implementation:**
1. Update SkyUniforms struct in all 3 shader templates (mode1, mode2, mode3)
2. Update Rust-side SkyUniforms struct in `emberware-z/src/graphics/mod.rs` to match
3. Update `set_sky()` FFI to pack sun_sharpness into sun_color.w
4. Update shader code that reads `sky.sun_sharpness` to `sky.sun_color_and_sharpness.w`
5. Search for other uniform structs with `_pad` fields and apply same pattern

---

#### 2. **[HIGH] Eliminate array copy in sprite/billboard functions**

**Location:** `emberware-z/src/ffi/mod.rs:1357` and similar lines in draw_billboard, draw_sprite_region, etc.

**Current Code:**
```rust
state.draw_commands.push(DrawCommand::DrawSprite {
    // ... other fields
    bound_textures: state.render_state.bound_textures,  // Copies [u32; 4]
});
```

**Issue:** Every draw call copies the 16-byte texture slot array. Called 100+ times per frame.

**Proposed Fix:** Store a reference/index to render state instead of copying fields
```rust
// Option A: Store render state index/hash (if render states are deduplicated)
bound_textures_key: u32,  // Index into deduped render states

// Option B: Accept the copy (it's only 16 bytes and likely in cache)
// Keep as-is, this is a micro-optimization
```

**Impact:** MEDIUM - 1.6KB saved per 100 draw calls. Likely not worth refactoring unless profiling shows it's hot.

**Recommendation:** Defer until profiling shows this matters. The copy is cheap (16 bytes, stack-to-stack).

---

#### 3. **[HIGH] Eliminate String allocation in draw_text()**

**Location:** `emberware-z/src/ffi/mod.rs:1430`

**Current Code:**
```rust
let text_string = match std::str::from_utf8(bytes) {
    Ok(s) => s.to_string(),  // ❌ Allocates String on every call
    Err(e) => {
        warn!("draw_text: invalid UTF-8 string: {}", e);
        return;
    }
};
```

**Proposed Fix:** Change DrawCommand::DrawText to store bytes + validate later
```rust
// In FFI:
let text_bytes = bytes.to_vec();  // Already a copy, unavoidable
state.draw_commands.push(DrawCommand::DrawText {
    text: text_bytes,  // Vec<u8> instead of String
    // ...
});

// In graphics backend when rendering:
let text_str = std::str::from_utf8(&cmd.text).unwrap_or("");
```

**Impact:** HIGH - Eliminates allocation for every draw_text() call. Common in UI-heavy games.

**Implementation:**
1. Change `DrawCommand::DrawText::text` from `String` to `Vec<u8>`
2. Update FFI to store bytes directly (remove `to_string()`)
3. Update graphics backend to decode UTF-8 during rendering (one-time per text draw)

---

#### 4. **[HIGH] Reduce Vec clones in DrawCommand variants**

**Location:** `core/src/wasm/draw.rs:29-50`

**Current Code:**
```rust
#[derive(Debug, Clone)]  // ❌ Clones Vec<f32> and Vec<u32>
pub enum DrawCommand {
    DrawTriangles {
        vertex_data: Vec<f32>,   // Cloned if DrawCommand is cloned
        // ...
    },
    DrawTrianglesIndexed {
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,    // Cloned if DrawCommand is cloned
        // ...
    },
    // ...
}
```

**Issue:** If DrawCommand is ever cloned (e.g., during state sorting), large vertex buffers get deep-copied.

**Analysis:** After optimization #3 in previous session, we sort commands in-place, so DrawCommand is NEVER cloned in hot path. The `Clone` derive is only used for debugging/tests.

**Proposed Fix:** None needed - sorting is already in-place. Keep `Clone` for flexibility.

**Impact:** LOW - Not an issue in current implementation. Monitor if DrawCommand cloning appears in profiles.

---

#### 5. **[MEDIUM-HIGH] Add #[inline] to input FFI hot path functions**

**Location:** `emberware-z/src/ffi/mod.rs` - input functions

**Current Code:**
```rust
fn button_held(mut caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    // Called 10-20 times per frame per player
}

fn stick_axis(mut caller: Caller<'_, GameState>, player: u32, axis: u32) -> f32 {
    // Called 2-4 times per frame per player
}
```

**Proposed Fix:**
```rust
#[inline]
fn button_held(mut caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    // ...
}

#[inline]
fn stick_axis(mut caller: Caller<'_, GameState>, player: u32, axis: u32) -> f32 {
    // ...
}
```

**Impact:** MEDIUM-HIGH - Input functions are called many times per frame. Inlining reduces call overhead.

**Implementation:** Add `#[inline]` to all input FFI functions (button_held, button_pressed, button_released, stick_axis, trigger_value, etc.)

---

#### 6. **[MEDIUM] Remove Clone derive from PendingTexture and PendingMesh**

**Location:** `core/src/wasm/draw.rs:8, 17`

**Current Code:**
```rust
#[derive(Debug, Clone)]  // ❌ Unnecessary - these are never cloned
pub struct PendingTexture {
    pub data: Vec<u8>,  // Can be MB-sized texture data
}

#[derive(Debug, Clone)]  // ❌ Unnecessary
pub struct PendingMesh {
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u32>>,
}
```

**Proposed Fix:**
```rust
#[derive(Debug)]  // Remove Clone - these are moved, not cloned
pub struct PendingTexture { /* ... */ }

#[derive(Debug)]
pub struct PendingMesh { /* ... */ }
```

**Impact:** MEDIUM - Prevents accidental clones of large resource data. Documents intent (these are moved to GPU, not copied).

**Verification:** Search codebase for `.clone()` calls on PendingTexture/PendingMesh - should be none.

---

#### 7. **[MEDIUM] Reduce render state field copying**

**Location:** `core/src/wasm/render.rs:48-77`

**Current Code:** Every DrawCommand copies multiple RenderState fields:
```rust
state.draw_commands.push(DrawCommand::DrawMesh {
    color: state.render_state.color,
    depth_test: state.render_state.depth_test,
    cull_mode: state.render_state.cull_mode,
    blend_mode: state.render_state.blend_mode,
    bound_textures: state.render_state.bound_textures,
    // ...
});
```

**Proposed Fix:** Store a snapshot of RenderState or index into deduped states
```rust
// Option A: Store RenderState snapshot
render_state: RenderState,  // 32 bytes total

// Option B: Dedupe and store index
render_state_key: u16,  // Index into Vec<RenderState>
```

**Impact:** MEDIUM - Simplifies DrawCommand variants, reduces field duplication. Trade-off: adds indirection during rendering.

**Recommendation:** Defer - current approach is simple and performant. Only optimize if profiling shows issue.

---

#### 8. **[MEDIUM] Eliminate Vec<Mat4> clone in RenderState**

**Location:** `core/src/wasm/render.rs` (if bone matrices stored in RenderState)

**Analysis Needed:** Check if bone matrices (for skinned meshes) are stored in RenderState or passed separately.

**If they're in RenderState:**
```rust
pub struct RenderState {
    bone_matrices: Vec<Mat4>,  // ❌ Cloned on every skinned draw?
}
```

**Proposed Fix:** Store bones in a separate shared structure, reference by ID
```rust
pub struct GameState {
    bone_sets: Vec<Vec<Mat4>>,  // Shared pool
}

pub struct DrawCommand {
    bone_set_id: u32,  // Reference instead of clone
}
```

**Impact:** MEDIUM - Eliminates 4KB+ clones for skinned meshes (256 bones × 64 bytes).

**Implementation:** Audit RenderState and DrawCommand for Vec<Mat4> fields, refactor to use shared bone set pool.

---

#### 9. **[MEDIUM] Add #[inline] to camera math methods**

**Location:** `emberware-z/src/graphics/camera.rs` (if it exists) or wherever view/projection matrices are computed

**Current Code:**
```rust
impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        // Matrix math
    }

    pub fn projection_matrix(&self) -> Mat4 {
        // Matrix math
    }
}
```

**Proposed Fix:**
```rust
impl Camera {
    #[inline]
    pub fn view_matrix(&self) -> Mat4 { /* ... */ }

    #[inline]
    pub fn projection_matrix(&self) -> Mat4 { /* ... */ }
}
```

**Impact:** MEDIUM - Called once per frame, but inlining helps with register allocation for matrix math.

**Implementation:** Add `#[inline]` to camera methods and other hot math helpers (transform composition, etc.)

---

#### 10. **[LOW] Remove duplicate vertex_stride() function**

**Location:** Search for `fn vertex_stride` across codebase

**Issue:** If vertex_stride is defined in multiple places (e.g., graphics/vertex.rs and graphics/buffer.rs), consolidate to single source.

**Proposed Fix:**
```rust
// In graphics/vertex.rs (canonical location):
pub const fn vertex_stride(format: u8) -> u32 {
    // Canonical implementation
}

// Remove from other files, import this one
```

**Impact:** LOW - Reduces code duplication, ensures consistency.

**Verification:** `rg "fn vertex_stride"` should show only ONE definition.

---

#### 11. **[LOW-MEDIUM] Optimize keycode matching**

**Location:** `emberware-z/src/input.rs` (keyboard to button mapping)

**Current Code (hypothetical):**
```rust
match keycode {
    KeyCode::KeyW => Some(BUTTON_DPAD_UP),
    KeyCode::KeyS => Some(BUTTON_DPAD_DOWN),
    KeyCode::KeyA => Some(BUTTON_DPAD_LEFT),
    KeyCode::KeyD => Some(BUTTON_DPAD_RIGHT),
    // ... 20+ more cases
    _ => None,
}
```

**Proposed Fix:** Use a lookup table (array or phf) instead of match
```rust
// At compile time:
static KEYCODE_TO_BUTTON: phf::Map<u32, u16> = phf_map! {
    KeyCode::KeyW as u32 => BUTTON_DPAD_UP,
    // ...
};

// At runtime:
KEYCODE_TO_BUTTON.get(&(keycode as u32)).copied()
```

**Impact:** LOW-MEDIUM - Reduces match overhead. Only matters if keyboard input is polled frequently (not typical for controller-primary games).

**Recommendation:** Defer - match is already fast for ~20 cases. Only optimize if profiling shows this in hot path.

---

**Summary:**

| Priority | Optimization | Estimated Savings | Effort |
|----------|-------------|-------------------|--------|
| HIGH | Vec4 padding (#1) | Readability + future-proofing | Medium |
| HIGH | draw_text String (#3) | 1 alloc per text draw | Low |
| MEDIUM-HIGH | #[inline] input (#5) | 5-10% input overhead | Low |
| MEDIUM | Remove Clone on Pending (#6) | Safety + clarity | Low |
| MEDIUM | #[inline] camera (#9) | Minor perf gain | Low |
| LOW | Dedupe vertex_stride (#10) | Code quality | Low |
| DEFER | Array copy (#2) | Negligible (16 bytes) | N/A |
| DEFER | Vec clone (#4) | Not cloned in hot path | N/A |
| DEFER | RenderState copy (#7) | Complex refactor, unclear gain | N/A |
| DEFER | Bone matrix clone (#8) | Needs investigation first | N/A |
| DEFER | Keycode matching (#11) | Unlikely bottleneck | N/A |

**Implementation Order:**
1. #3 (draw_text String) - Quick win, high impact
2. #5 (#[inline] input) - Quick win, medium impact
3. #6 (Remove Clone) - Quick win, safety improvement
4. #9 (#[inline] camera) - Quick win
5. #10 (Dedupe vertex_stride) - Code quality
6. #1 (Vec4 padding) - Larger refactor, but improves maintainability

---
## In Progress

### Remove reliance on MAX_STATE_SIZE and instead use console spec provided RAM to limit
- Rollback config.rs has defined MAX_STATE_SIZE, this may change per console Z, Class, or others
- We have a ConsoleSpecs trait which defines the maximum RAM via ram_limit
- Ram limit should be used to determine the MAX_STATE_SIZE, not some hardcoded magic number

### **[STABILITY] Refactor rollback to use automatic WASM linear memory snapshotting**

**Current State:** Games manually serialize state via FFI callbacks (`save_state(ptr, max_len) -> len` and `load_state(ptr, len)`). This requires boilerplate in every game and is error-prone.

**Target State:** Automatic memory snapshotting as described in [docs/rollback-architecture.md](docs/rollback-architecture.md). The host snapshots entire WASM linear memory transparently. Games require zero serialization code.

**Implementation in progress...**

## Done

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
