# Emberware Development Tasks

**Task Status Tags:**
- `[STABILITY]` — Robustness, error handling, testing, safety improvements
- `[FEATURE]` — New functionality for game developers
- `[NETWORKING]` — P2P, matchmaking, rollback netcode
- `[POLISH]` — UX improvements, optimization, documentation

---

**Architecture Overview:** See [CLAUDE.md](./CLAUDE.md) for framework design and Console trait details.

---

## In Progress

---

## TODO

### **[POLISH] BUG: Window Size scaling issues**
- When loading a game, black bars appear on the sides. The inner window should "snap" to the nearest perfect integer scaling of the window (in integer scaling mode), or just stay at that size for stretch
- We are not able to resize the window to a size equal to the fantasy console (and game ROMs) initialized resolution. We should be able to scale down to a 1x scaling, but not any smaller to prevent a crash
- These problems may be due to some kind of egui scaling based on the OS scaling rules.

### **[POLISH] Document ALL FFI Functions **
- We need to know these at a quick glance so developers can copy paste a "cheat sheet" into their games before working

### **[POLISH] Add axis-to-keyboard binding support**

**Status:** Not yet implemented

**Current State:**
- Keyboard bindings only support button presses (digital input)
- Analog sticks and triggers cannot be controlled via keyboard
- Settings UI has deadzone sliders for analog inputs

**What's Needed:**
Allow users to bind keyboard keys to simulate analog stick axes and triggers.

**Implementation Plan:**

1. **Extend KeyboardMapping struct** in `input.rs`:
   - Add fields for axis bindings (e.g., `left_stick_up_key`, `left_stick_down_key`, etc.)
   - Each axis direction gets its own key binding
   - When pressed, outputs 0 or 1 (binary analog values)

2. **Update InputManager**:
   - Check axis key bindings in addition to button bindings
   - Combine axis keys to generate stick/trigger values
   - Examples:
     - Left stick: W/S for Y axis (-1/+1), A/D for X axis (-1/+1)
     - Triggers: Q/E for left/right trigger (0 or 255)

3. **Settings UI additions** in `settings_ui.rs`:
   - Add "Analog Sticks" section to Controls tab
   - Add "Triggers" section to Controls tab
   - Each axis gets 4 key bindings (left stick: up/down/left/right)
   - Triggers get 2 key bindings (left trigger, right trigger)
   - Use same click-to-rebind UX as existing button bindings

4. **Config serialization**:
   - Update serde derives to include new axis binding fields
   - Provide sensible defaults (e.g., WASD for left stick, arrows for right stick)

**Files to Modify:**
- `emberware-z/src/input.rs` - Add axis binding fields to KeyboardMapping
- `emberware-z/src/settings_ui.rs` - Add axis remapping UI sections
- `emberware-z/src/config.rs` - Ensure new fields serialize correctly

**User Benefit:**
Keyboard players can use analog stick inputs in games, enabling full control without a gamepad.

---

### **[OPTIMIZATION] Share quad index buffer for sprites and billboards**

**Status:** Minor optimization opportunity

**Current State:**
- Every sprite/billboard allocates a new `Vec<u32>` for indices (line 1347, 1460)
- Indices are always the same: `[0, 1, 2, 0, 2, 3]`
- Hundreds of redundant allocations per frame

**Impact:**
- Modest: ~24 bytes allocated per sprite/billboard
- Adds up with particle systems (100+ sprites = 2.4KB of redundant allocations)

**Solution:**
Pre-allocate shared quad index buffer at init time:
```rust
// In ZGraphics::new()
let quad_indices: &[u32] = &[0, 1, 2, 0, 2, 3];
self.shared_quad_index_offset = self.command_buffer.append_index_data(SPRITE_FORMAT, quad_indices);
```

Then use it in sprite/billboard generation (no allocation):
```rust
let first_index = self.shared_quad_index_offset;  // Reuse
```

**Trade-off:**
- Simple implementation (10 lines of code)
- Small memory win
- Not a huge perf gain, but good hygiene

**Files to Modify:**
- `emberware-z/src/graphics/mod.rs` - Add shared index buffer in new(), use in process_draw_commands()

---

### **[EXAMPLES] Create comprehensive example suite**

**Status:** Examples exist but coverage gaps

**Current Examples (8):**
1. ✅ hello-world - 2D text + rectangles, basic input
2. ✅ triangle - Immediate mode 3D (POS_COLOR)
3. ✅ textured-quad - Textured geometry
4. ✅ cube - 3D cube mesh
5. ✅ lighting - PBR mode (mode 2), dynamic lights
6. ✅ billboard - All 4 billboard modes
7. ✅ skinned-mesh - GPU skinning demo
8. ✅ platformer - 2D platformer game

**Missing Examples:**
- ❌ **Mode 0 (Unlit)** example - No example demonstrates unlit rendering
- ❌ **Mode 1 (Matcap)** example - No matcap rendering demo
- ❌ **Mode 3 (Hybrid)** example - No hybrid PBR+matcap demo
- ❌ **Matcap blend modes** - No demo of multiply/add/HSV modulate
- ❌ **Audio system** - NO AUDIO EXAMPLES AT ALL despite fully working audio!
- ❌ **Custom fonts** - No font loading demo
- ❌ **Sprite batching** - No sprite-heavy 2D demo
- ❌ **Blend modes** - No demo of alpha/additive/multiply blending
- ❌ **Depth test** - No demo of depth buffer usage
- ❌ **Cull mode** - No demo of face culling
- ❌ **Transform stack** - No demo of push/pop transforms
- ❌ **Multiplayer input** - No demo of 2-4 player local input

**Recommended New Examples:**
1. **matcap-showcase** - All 3 matcap slots with blend modes (mode 1)
2. **unlit-lowpoly** - PS1-style low-poly with vertex colors (mode 0)
3. **hybrid-character** - Character with matcap ambient + PBR lighting (mode 3)
4. **audio-test** - Sound effects, music, channels (CRITICAL - audio undocumented!)
5. **custom-font** - Load bitmap font, render styled text
6. **particle-system** - Hundreds of sprites, blend modes
7. **multiplayer-pong** - 2-player local, demonstrates player_count/local_player_mask

**Example Location:**
- Move from `/examples` to `/emberware-z/examples` (they're Z-specific, not core)
- Update Cargo workspace to reflect new location
- Update README to point to new location

**Files to Modify:**
- Create 7 new example projects
- Move existing examples to emberware-z/examples/
- Update root Cargo.toml workspace members

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

## Done

### **[PERFORMANCE] Cache frame bind groups to avoid recreation per draw call**

**Completed:** Implemented bind group caching to eliminate redundant GPU resource creation

**Implementation:**
- Added `HashMap<u64, wgpu::BindGroup>` to cache frame bind groups by material buffer pointer address
- Frame bind groups now created once per unique material per frame instead of per draw call
- Used `.entry().or_insert_with()` pattern for efficient lazy initialization
- Cache key is material buffer pointer address (stable within a frame)
- Eliminates hundreds/thousands of bind group allocations per frame for typical scenes

**Performance Impact:**
- Before: New bind group created for every draw call (worst case: 1000 draw calls = 1000 allocations)
- After: Bind groups cached by material (typical case: 1000 draw calls with 10 materials = 10 allocations)
- ~100× reduction in bind group creation for material-heavy scenes

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Added frame bind group cache (lines 2136-2336)

---

### **[POLISH] Document audio system in docs/emberware-z.md**

**Completed:** Replaced "shelved" notice with comprehensive audio documentation

**Documentation Added:**
- Removed misleading "Audio system is shelved" notice
- Documented all 8 audio FFI functions with signatures, parameters, examples:
  - `load_sound` - Load 16-bit PCM sound data (init-only)
  - `play_sound` - Fire-and-forget playback for one-shot SFX
  - `channel_play` - Managed channel playback with looping support
  - `channel_set` - Real-time volume/pan updates for positional audio
  - `channel_stop` - Stop channel playback
  - `music_play` - Looping background music
  - `music_stop` - Stop music
  - `music_set_volume` - Adjust music volume
- Documented audio specs (22,050 Hz, 16-bit signed PCM, mono)
- Added best practices for channel allocation strategy
- Included ffmpeg conversion command for asset preparation
- Provided positional audio example with distance attenuation

**Files Modified:**
- `docs/emberware-z.md` - Added complete Audio FFI section (lines 898-1003)

---

### **[POLISH] Document custom font loading in docs/emberware-z.md**

**Completed:** Added comprehensive custom font system documentation

**Documentation Added:**
- `load_font` - Fixed-width bitmap fonts from texture atlas
- `load_font_ex` - Variable-width bitmap fonts with per-glyph widths
- `font_bind` - Switch active font for draw_text calls
- Texture atlas layout explanation (grid-based arrangement)
- Examples for both fixed and variable-width fonts
- Best practices for atlas preparation and character coverage
- Performance notes (font textures loaded once in init)
- Styling tips (outline/shadow pre-baked, size scaling via draw_text parameter)

**Files Modified:**
- `docs/emberware-z.md` - Added Custom Fonts section (lines 806-933)

---

### **[POLISH] Document matcap blend modes in docs/emberware-z.md**

**Completed:** Enhanced Mode 1 (Matcap) documentation with blend mode details

**Documentation Added:**
- `matcap_blend_mode(slot, mode)` function signature
- Three blend modes explained:
  - Mode 0 (Multiply) - Standard matcap behavior, darkens
  - Mode 1 (Add) - Additive blending for glow/emission effects
  - Mode 2 (HSV Modulate) - Hue shift for iridescence effects
- Use cases for each mode (lighting, rim lights, beetle shell iridescence)
- Example combining all three slots with different blend modes
- Performance note (all modes identical cost)

**Files Modified:**
- `docs/emberware-z.md` - Expanded Mode 1 section (lines 209-242)

---

### **[POLISH] Document texture filtering in docs/emberware-z.md**

**Completed:** Expanded texture filtering documentation with practical guidance

**Documentation Added:**
- Detailed explanation of nearest (0) vs linear (1) filtering
- Visual differences (pixelated vs smooth)
- When to use each mode (pixel art vs 3D textures, UI vs models)
- Performance notes (negligible difference, choose based on visual needs)
- PS1/N64 authenticity tip (use nearest for true 5th-gen look)
- Default mode (nearest)
- Example showing how to mix filter modes per-texture within a frame
- Note about filter mode persistence

**Files Modified:**
- `docs/emberware-z.md` - Expanded Render State section (lines 770-802)

---

### **[STABILITY] Add safety documentation to unsafe blocks**

**Completed:** Added comprehensive SAFETY documentation to unsafe block in load_sound FFI function

Added detailed SAFETY comment explaining why the unsafe memory access is sound:
1. Pointer validity - comes from WASM memory export, guaranteed valid by wasmtime
2. Alignment correctness - byte_len validated as even, ensuring proper i16 alignment
3. Length calculation - sample_count = byte_len / 2, reading exact number of i16 samples
4. Lifetime guarantees - data immediately copied to owned Vec, no aliasing issues
5. WASM memory validity - linear memory guaranteed valid for call duration

**Why This Matters:**
- Documents safety invariants for reviewers
- Prevents future modifications from violating safety assumptions
- Follows Rust best practices for unsafe code
- Improves code maintainability and auditability

**Files Modified:**
- `emberware-z/src/ffi/mod.rs` - Added SAFETY comment to load_sound unsafe block

**Test Results:** 520 tests passing

---

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

## DONE

### **[FEATURE] Implement offscreen render target for fixed internal resolution**

**Status:** ✅ Completed - Games now render at fixed resolution with automatic scaling

**What Was Implemented:**

1. ✅ **RenderTarget struct** - Offscreen color + depth textures at game resolution
2. ✅ **Blit shader** (`shaders/blit.wgsl`) - Fullscreen triangle for texture scaling
3. ✅ **Blit pipeline** - Nearest-neighbor sampling for pixel-perfect look
4. ✅ **render_frame() updated** - Game content renders to offscreen target, then blits to window
5. ✅ **Resolution change detection** - `graphics.update_resolution()` called each frame
6. ✅ **Dynamic render target recreation** - Automatically recreates when resolution changes
7. ✅ **ScaleMode config setting** - Added to `VideoConfig` (Stretch, PixelPerfect)
8. ✅ **PixelPerfect scaling mode** - Integer scaling with letterboxing and centered viewport

**Current State:**
- ✅ 2D elements (text, sprites, rects) convert pixel coordinates to NDC using game's configured resolution
- ✅ Dynamic resolution support from `init_config.resolution_index` (640×360, 960×540, 1280×720, 1920×1080)
- ✅ Games render to offscreen target at fixed resolution
- ✅ Render target automatically recreated when game changes resolution
- ✅ Window resizing no longer affects game viewport
- ✅ Stretch mode works (fills window, may distort aspect ratio)
- ✅ PixelPerfect mode works (integer scaling with black bars for pixel-perfect display)

**Benefits Achieved:**
- ✅ True fantasy console behavior - fixed internal resolution scales to display
- ✅ Window resize doesn't change game viewport anymore
- ✅ Games can switch resolution at runtime (e.g., 360p for menus, 1080p for gameplay)
- ✅ Easy to add CRT filters, scanlines, or other post-processing effects in blit shader
- ✅ Consistent with PS1/N64/Dreamcast behavior

**Files Modified:**
- `emberware-z/src/graphics/mod.rs` - Added RenderTarget struct, `create_render_target()`, `create_blit_pipeline()`, `update_resolution()`, `recreate_render_target()`, `set_scale_mode()`, viewport calculation in `render_frame()` for both Stretch and PixelPerfect modes
- `emberware-z/src/app.rs` - Added `graphics.update_resolution()` and `graphics.set_scale_mode()` calls
- `emberware-z/src/config.rs` - Added ScaleMode enum and `scale_mode` field to VideoConfig, updated tests
- `emberware-z/shaders/blit.wgsl` - New fullscreen triangle shader for texture scaling

---

### **[POLISH] Implement Settings UI with input remapping**

**Status:** ✅ Completed

**What Was Implemented:**

1. ✅ **SettingsUi struct** - Created `settings_ui.rs` with tab-based interface (Video, Audio, Controls)
2. ✅ **Key remapping system** - Click-to-rebind interface with waiting state and ESC to cancel
3. ✅ **Video settings** - Fullscreen, V-Sync, and Scale Mode (Stretch/PixelPerfect) with live preview
4. ✅ **Audio settings** - Master volume slider with percentage display
5. ✅ **Controls settings** - Keyboard remapping for all buttons (D-Pad, Face Buttons, Shoulder Buttons, System Buttons)
6. ✅ **Deadzone settings** - Stick and trigger deadzone sliders
7. ✅ **Config persistence** - Apply & Save button writes to disk, Reset to Defaults button
8. ✅ **Temporary config editing** - Changes aren't applied until user clicks Apply & Save
9. ✅ **Integration with app.rs** - Settings mode fully functional, key press handling for remapping

**Files Created:**
- `emberware-z/src/settings_ui.rs` - Complete settings UI implementation

**Files Modified:**
- `emberware-z/src/ui.rs` - Added SaveSettings and SetScaleMode actions
- `emberware-z/src/app.rs` - Integrated SettingsUi, added key press handler, save/apply logic
- `emberware-z/src/config.rs` - Added PartialEq derives for comparison
- `emberware-z/src/input.rs` - Added PartialEq derives
- `emberware-z/src/main.rs` - Registered settings_ui module

**User Benefit:**
Players can customize controls, adjust audio levels, and configure video settings through an intuitive UI without editing TOML files manually.

---

### **[POLISH] Scale bitmap font for better readability**

**Status:** ✅ Completed

**Problem:**
Text in examples was very small and difficult to read at higher resolutions.

**Solution:**
Implemented 2x scaling of the bitmap font from 8x8 to 16x16 using nearest-neighbor upscaling.

**What Was Implemented:**
1. ✅ Added `FONT_SCALE` constant (set to 2)
2. ✅ Separated source glyph size (8x8) from output glyph size (16x16)
3. ✅ Modified atlas generation to perform nearest-neighbor scaling
4. ✅ Each source pixel becomes a FONT_SCALE × FONT_SCALE block in output
5. ✅ Maintains crisp bitmap font aesthetic while being more readable

**Files Modified:**
- `emberware-z/src/font.rs` - Added scaling constants and modified `generate_font_atlas()`

**User Benefit:**
Text is now 2x larger (16x16 instead of 8x8), making it much more readable in UI and debug displays.

---

### **[STABILITY] Fix window resize panic in pixel-perfect mode**

**Status:** ✅ Completed

**Problem:**
When using integer scaling mode, resizing the window smaller than the game's render resolution caused a panic due to invalid viewport calculations (negative or zero viewport dimensions).

**Solution:**
Dynamically set window minimum size based on the game's current render resolution, preventing the window from becoming too small.

**What Was Implemented:**
1. ✅ Added dynamic `set_min_inner_size()` call in render loop
2. ✅ Minimum size updates whenever game resolution changes via `update_resolution()`
3. ✅ Uses `PhysicalSize::new(graphics.width(), graphics.height())` for minimum
4. ✅ Prevents viewport panic by ensuring window is always >= render resolution

**Files Modified:**
- `emberware-z/src/app.rs` - Added dynamic window minimum size constraint

**User Benefit:**
Application no longer crashes when resizing window in pixel-perfect scaling mode. Window size is constrained to prevent invalid viewport calculations.

---

