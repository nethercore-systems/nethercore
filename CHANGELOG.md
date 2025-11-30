# Emberware Development Changelog

Complete development history of Emberware Z fantasy console.

---

## 2025-12-1

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

### **[POLISH] Performance Optimizations - Inline and Code Cleanup**
**Status:** Completed ✅
**Changes Made:**
- Added `#[inline]` attribute to camera math methods in `core/src/wasm/camera.rs`:
  - `view_matrix()`, `projection_matrix()`, `view_projection_matrix()`
- Added `#[inline]` attribute to all input FFI hot path functions:
  - `right_stick_x`, `right_stick_y`, `left_stick`, `right_stick`
  - `trigger_left`, `trigger_right`
- Removed duplicate `vertex_stride()` function and FORMAT constants from `emberware-z/src/ffi/mod.rs`
  - Added import from `crate::graphics` to use canonical implementations
- Removed `Clone` derive from `PendingTexture` and `PendingMesh` structs
  - Prevents accidental expensive clones of large resource data
- Verified `DrawCommand::DrawText` already stores `Vec<u8>` instead of `String`
- All tests passing ✓

**Impact:**
- Reduced function call overhead in hot paths
- Eliminated duplicate code
- Prevented accidental performance issues from cloning large data

---

### **[STABILITY] Refactor rollback to use automatic WASM linear memory snapshotting**
**Status:** Completed ✅
**Changes Made:**
- Implemented automatic WASM linear memory snapshotting in `GameInstance::save_state()` and `GameInstance::load_state()`
- Games no longer need to implement manual serialization callbacks
- Host snapshots entire WASM linear memory transparently
- Comprehensive test coverage for memory snapshotting
- Documentation updated in rollback-architecture.md
- All tests passing ✓

---

### **[STABILITY] Remove duplicate TestConsole definitions**
**Status:** Completed ✅
**Changes Made:**
- Created shared `test_utils.rs` module with common test utilities
- Moved TestConsole, TestGraphics, TestAudio, and TestInput to shared module
- Updated integration.rs to use shared test utilities (removed 120+ lines)
- Updated runtime.rs to use shared test utilities (removed 90+ lines)
- All 194 tests passing ✓

---

### **[STABILITY] Remove reliance on MAX_STATE_SIZE and use console spec provided RAM to limit**
**Status:** Completed ✅
**Changes Made:**
- Updated `RollbackStateManager::new(max_state_size)` to accept console-specific RAM limit
- Added `RollbackStateManager::with_defaults()` for backward compatibility
- Updated all `RollbackSession` constructors to accept `max_state_size` parameter
- Added documentation to `MAX_STATE_SIZE` constant explaining it's a fallback
- Consoles now use `console.specs().ram_limit` when creating rollback sessions:
  - Emberware Z: 4MB
  - Emberware Classic: 1MB
- All tests passing ✓


## 2025-11-30

### Codebase Cleanup

- Removed unused `emberware-z/pbr-lite.wgsl` (duplicate of code in mode2_pbr.wgsl)
- Removed unused `shader_gen_example/` directory (reference code from different project)
- Verified stub files are intentional: `download.rs`, `runtime/mod.rs`
- Adjusted Z's max rom size to 12mb.
- All 573 tests passing

### [STABILITY] Fix clippy warnings in test code

- Fixed 13 clippy warnings across test code in app.rs and ffi/mod.rs
- app.rs: Replaced field_reassign_with_default with struct initialization
- app.rs: Replaced single_match patterns with if-let and equality checks
- app.rs: Removed redundant unwrap on Some value
- app.rs: Added PartialEq derive to AppMode enum for cleaner tests
- ffi/mod.rs: Replaced assign_op_pattern (a = a * b) with compound assignment (a *= b)
- ffi/mod.rs: Replaced field_reassign_with_default with struct initialization
- ffi/mod.rs: Replaced neg_cmp_op_on_partial_ord with partial_cmp for NaN handling
- All 573 tests passing (196 core + 377 emberware-z)

### [STABILITY] Add session cleanup on exit

- Added explicit `game_session = None` cleanup when exiting Playing mode via ESC key
- Logs "Exiting game via ESC" for debugging
- Ensures game_session is properly dropped, which:
  - Drops the `Runtime<EmberwareZ>` containing the game instance
  - Drops the `RollbackSession` which cleans up GGRS P2P connections via Drop
  - Releases all game resources (textures, meshes, audio)
- Already handled for quit_requested and runtime error paths
- All 573 tests passing (196 core + 377 emberware-z)

### [STABILITY] Integrate session events into app (disconnect handling)

- Added `handle_session_events()` method to App that polls `Runtime::handle_session_events()` each frame
- `SessionEvent::Disconnected` → transitions to Library with "Player X disconnected" error
- `SessionEvent::Desync` → transitions to Library with desync error showing frame number
- `SessionEvent::NetworkInterrupted` → sets `DebugStats.network_interrupted` for UI warning
- `SessionEvent::NetworkResumed` → clears network interrupted warning
- Added `update_session_stats()` method that populates `DebugStats.ping_ms`, `rollback_frames`, and `frame_advantage` from P2P session
- Added `network_interrupted: Option<u64>` field to `DebugStats` for connection timeout display
- Updated debug overlay Network section to show connection interrupted warning with yellow label
- All session events (Synchronized, FrameAdvantageWarning, TimeSync, WaitingForPlayers) are logged appropriately
- 2 new tests for network_interrupted field

### [STABILITY] Implement local network testing

- Created `LocalSocket` UDP wrapper implementing GGRS `NonBlockingSocket<String>` trait
- Allows P2P sessions without matchbox signaling server
- Bind to any port with `LocalSocket::bind("127.0.0.1:0")` or specific port
- Connect to peer with `socket.connect("127.0.0.1:port")`
- Usage: Run two instances, each bound to different ports, each connecting to the other
- 12 new tests for socket binding, connecting, and UDP communication
- Exports: `LocalSocket`, `LocalSocketError`, `DEFAULT_LOCAL_PORT` from `emberware_core`

### [STABILITY] Replace unsafe transmute with wgpu::RenderPass::forget_lifetime()

- wgpu 23 provides `forget_lifetime()` which is a safe alternative to unsafe transmute
- Removed unsafe block that was working around egui-wgpu 0.30 API limitation
- The method safely converts compile-time lifetime errors to runtime errors if encoder is misused

### Wire up game execution in Playing mode

- Implemented game loop integration in App::render()
- Added process_pending_resources() to load textures/meshes from game into graphics backend
- Added execute_draw_commands() to translate game DrawCommands to ZGraphics commands
- Added run_game_frame() to orchestrate input → frame() → render() → resource processing
- Input flow: InputManager → map_input() → game.set_input() → FFI
- Game rendering: render_frame() with camera matrices and draw command execution
- Error handling: Runtime errors return to Library with error message
- Quit handling: game quit_requested flag returns to Library
- egui overlay: LoadOp::Load preserves game rendering when in Playing mode

### [STABILITY] Replace test unwrap() calls with descriptive expect() messages

- Replaced `.unwrap()` with `.expect()` in `create_test_game()` test helper function
- Added descriptive messages: "failed to create test game directory", "failed to serialize test manifest", "failed to write test manifest.json", "failed to write test rom.wasm"
- All 376 tests passing

### Add render_frame method to ZGraphics

- Core GPU rendering pipeline for executing buffered draw commands
- Creates render pass with depth buffer attachment
- Uploads vertex/index data to GPU buffers per vertex format
- Creates pipelines on-demand with proper bind groups
- Executes draw calls with material uniforms and texture bindings
- Added write_at method to GrowableBuffer for direct offset writes

### Add input delay configuration

- `NetplayConfig` struct with `input_delay: u8` (0-10 frames, default 2)
- Settings UI with slider and explanatory text
- Auto-saves to config.toml on change
- Already integrated with `SessionConfig` in core (uses `with_input_delay()`)

### Rollback state memory optimization

- `StatePool` with pre-allocated buffers to avoid allocations in hot path
- Buffer acquire/release pattern with automatic recycling
- Oversized buffers discarded to prevent memory bloat
- Pool exhaustion handled gracefully with new allocation and warning

### [STABILITY] Add bounds checking for potentially truncating type casts

- Added `checked_mul()` overflow protection in FFI functions:
  - `load_texture`: width × height × 4 calculation
  - `load_mesh`: vertex_count × stride calculation
  - `load_mesh_indexed`: vertex_count × stride and index_count × 4 calculations
  - `draw_triangles`: vertex_count × stride calculation
  - `draw_triangles_indexed`: vertex_count × stride and index_count × 4 calculations
- Returns 0/early returns with warning on overflow instead of wrapping
- Added 5 new tests for arithmetic overflow protection
- All 559 tests passing

### [STABILITY] Document resource cleanup strategy

- Added "Resource Cleanup Strategy" section to `emberware-z/src/graphics/mod.rs` module docs
- Documented cleanup behavior for: Textures, Retained Meshes, Vertex/Index Buffers, Pipelines, Per-Frame Resources
- Added "Resource Lifecycle" section to `core/src/wasm/state.rs` GameState docs
- Documented: Pending Textures/Meshes, Draw Commands, Save Data lifecycle
- Key findings:
  - All wgpu types (Texture, Buffer, Pipeline) auto-cleanup via Drop trait
  - ZGraphics persists across game switches to avoid expensive reinitialization
  - GPU resources from previous games remain until app exit (acceptable for single-game sessions)
  - No custom Drop implementations needed - Rust's RAII handles cleanup
- All 554 tests passing

### [STABILITY] Review clone operations for optimization

- Optimized `core/src/ffi.rs:118`: Used `data_and_store_mut()` to eliminate O(n) Vec clone in save data load
- Analyzed `emberware-z/src/app.rs:303,306,308`: Clones are necessary due to borrow checker constraints with egui closures
- Analyzed `emberware-z/src/graphics/command_buffer.rs:297`: Test code only, not production
- All 554 tests passing

### [STABILITY] Reduce DRY violations in vertex attribute generation

- Replaced 340-line match statement with data-driven static array
- Created helper functions: `attr_pos()`, `attr_uv()`, `attr_color()`, `attr_normal()`, `attr_bone_indices()`, `attr_bone_weights()`
- Added size constants: `SIZE_POS`, `SIZE_UV`, `SIZE_COLOR`, `SIZE_NORMAL`, `SIZE_BONE_INDICES`
- Added shader location constants: `LOC_POS`, `LOC_UV`, `LOC_COLOR`, `LOC_NORMAL`, `LOC_BONE_INDICES`, `LOC_BONE_WEIGHTS`
- `VERTEX_ATTRIBUTES` static array holds pre-computed attribute slices for all 16 formats
- `build_attributes()` now just indexes into the array
- Reduced code from ~340 lines to ~170 lines (50% reduction)
- All 554 tests passing

### [STABILITY] Split wasm.rs into modules

- Split 1917 lines into 5 submodules:
  - `camera.rs`: CameraState, DEFAULT_CAMERA_FOV (~115 lines)
  - `draw.rs`: DrawCommand, PendingTexture, PendingMesh (~380 lines)
  - `input.rs`: InputState (~75 lines)
  - `render.rs`: LightState, RenderState, InitConfig, MAX_BONES (~240 lines)
  - `state.rs`: GameState, MAX_* constants (~160 lines)
  - `mod.rs`: WasmEngine, GameInstance, re-exports (~970 lines)
- All 183 core tests passing
- All public API preserved via re-exports

### [STABILITY] Split graphics.rs into modules

- Split 3436 lines into 5 submodules:
  - `vertex.rs`: Vertex format constants, VertexFormatInfo, stride calculations (~600 lines)
  - `buffer.rs`: GrowableBuffer, MeshHandle, RetainedMesh (~180 lines)
  - `render_state.rs`: CullMode, BlendMode, TextureFilter, SkyUniforms, RenderState, TextureHandle (~470 lines)
  - `command_buffer.rs`: DrawCommand, CommandBuffer (~270 lines)
  - `pipeline.rs`: PipelineKey, PipelineEntry, create_pipeline, bind group layouts (~290 lines)
  - `mod.rs`: ZGraphics struct, core methods, re-exports (~1050 lines)
- All 371 tests passing
- All public API preserved via re-exports

### [STABILITY] Add negative test cases for FFI error conditions

- Added 67 new tests (139 total FFI tests, up from 72)
- **Invalid texture handle tests**: Zero handle, unloaded handle, slot independence
- **Invalid mesh handle tests**: Zero handle rejection, unloaded handle handling
- **Out-of-range parameter tests**: Resolution index, tick rate index, render mode, cull mode, blend mode, texture filter, vertex format, billboard mode, matcap slot, light index
- **Edge case tests**: Camera FOV clamping, transform rotate zero axis, material property clamping, light color/intensity negative values, light direction zero vector, transform stack overflow/underflow, bone count limits, draw triangles vertex count, mesh index count, texture dimensions, init-only guards, draw command buffer growth, pending resource growth, handle allocation overflow, special float values (NaN, infinity)

### [STABILITY] Review dead_code allowances

- Verified all 4 dead_code allowances are properly documented and necessary:
- `core/src/wasm.rs:494`: `instance` field must be kept alive for WASM function lifetimes
- `core/src/runtime.rs:44`: `console` field kept for future console-specific features
- `emberware-z/src/app.rs:149`: `handle_runtime_error` is infrastructure for future use
- `emberware-z/src/console.rs:67,85,121`: Button enum/helpers are public API for tests and console-side code

### [STABILITY] Clarify runtime TODO comment

- Replaced outdated TODO with accurate module layout documentation
- Now lists actual file locations for all runtime components across core and emberware-z
- References TASKS.md for unimplemented audio feature

### [STABILITY] Add error path tests for WASM memory access

- Added 14 new tests to `core/src/ffi.rs` for FFI memory access error paths
- Added 12 new tests to `core/src/wasm.rs` for GameInstance error paths
- All 304 tests passing (183 core + 121 emberware-z)
- Tests verify: out-of-bounds memory access, invalid slot handling, buffer overflow protection, missing memory handling, WASM trap propagation

### [STABILITY] Add documentation to shared crate public APIs

- Added module-level `//!` doc comment explaining API type categories with example usage
- Documented all API response structs, auth types, local types, and request types
- Documented `error_codes` module constants
- Added working doctests for auth types
- All 463 tests passing

### [STABILITY] Split rollback.rs into modules

- Extracted ~1846 lines into 4 submodules:
  - `config.rs`: GGRS configuration, SessionConfig, constants (127 lines)
  - `player.rs`: PlayerSessionConfig for local/remote player management (267 lines)
  - `state.rs`: GameStateSnapshot, StatePool, RollbackStateManager, error types (240 lines)
  - `session.rs`: RollbackSession, SessionEvent, SessionError, network stats (563 lines)
  - `mod.rs`: Module re-exports and documentation (50 lines)
- All 159 core tests passing
- All public API preserved via re-exports in lib.rs

### [STABILITY] Split ffi.rs input functions into separate module

- Extracted input FFI functions (14 functions, ~350 lines) to `ffi/input.rs`
- Created `ffi/mod.rs` to organize FFI module with public submodule
- All 463 tests passing (159 core + 304 emberware-z)
- `ffi.rs` reduced from 3120 lines to 2250 lines (mod.rs) + 310 lines (input.rs)

### [STABILITY] Add tests for graphics pipeline

- Added 32 new tests (98 total graphics tests, up from 66)
- **Sky Uniforms tests**: Default values, custom values, struct size (64 bytes), alignment
- **Retained Mesh tests**: Default values, non-indexed meshes, indexed meshes
- **Draw Command tests**: Creation, clone
- **Text Rendering tests**: Empty string, single char, multiple chars, color extraction, position, valid indices
- **Vertex Attribute tests**: Buffer layout for POS only, full format (FORMAT_ALL), attribute offsets, shader locations
- **Command Buffer Edge Cases**: Different formats, transform capture, large batch (1000 triangles)

### [STABILITY] Add tests for ui.rs

- Added 17 new tests for library UI
- **LibraryUi tests**: new(), select_game, deselect_game, change_selection
- **UiAction tests**: All variants (PlayGame, DeleteGame, OpenBrowser, OpenSettings, DismissError)
- **Trait tests**: Debug formatting, Clone, PartialEq
- **Edge cases**: Empty string game IDs, unicode game IDs, long game IDs, variant inequality
- Added `#[derive(Debug, Clone, PartialEq)]` to `UiAction` enum to support tests

### [STABILITY] Add missing documentation for public APIs

- `graphics.rs`: Added docs for `vertex_buffer_layout()` and `build_attributes()`
- `ui.rs`: Added docs for `LibraryUi` struct, `show()` method, and `UiAction` enum with all variants
- `config.rs`: Added module-level docs, struct docs for `Config`/`VideoConfig`/`AudioConfig`, and docs for functions with platform-specific path examples

### [STABILITY] Add tests for library.rs

- Added 24 new tests for library management functions
- **LocalGame struct tests**: Clone, Debug trait implementations
- **get_games_from_dir tests**: Empty dir, nonexistent dir, single game, multiple games, skips files, skips missing/invalid/incomplete manifests, correct rom_path
- **is_cached_in_dir tests**: Not present, directory only, with rom, complete game
- **delete_game_in_dir tests**: Nonexistent game, existing game, removes all contents, leaves other games intact
- **Edge case tests**: Full workflow (add/list/delete), special characters in game ID, unicode in metadata, empty strings, very long game ID
- Added documentation for public APIs
- Extracted internal testable functions for filesystem testing with temp directories
- Added `tempfile` as dev-dependency

### [STABILITY] Add tests for config.rs

- Added 21 new tests for config persistence and validation
- **Default value tests**: Config, VideoConfig, AudioConfig, helper functions
- **TOML serialization tests**: Serialize roundtrip, deserialize empty, partial video/audio
- **Edge case tests**: Volume 0/1, resolution scale values
- **Directory function tests**: config_dir/data_dir consistency
- **Trait tests**: Clone, Debug formatting
- **Load function tests**: Returns valid config without panic

### [STABILITY] Add tests for FFI validation

- Added 57 new tests (72 total FFI tests) covering FFI validation and edge cases
- **Vertex format tests**: Format constants, stride calculations for all 16 formats, skinned format boundaries
- **Render state tests**: Defaults, material defaults, light defaults
- **Init config tests**: Resolution values, tick rate values, render mode validation
- **Input state tests**: Defaults, button bitmask layout, stick/trigger range conversions, prev/curr independence
- **Draw command tests**: All command variants with state capture
- **Pending resource tests**: Texture/mesh structures, handle increments
- **Light state tests**: Default values, all fields, four slot validation
- **Save data tests**: Slot count, Option<Vec<u8>> storage
- **Color conversion tests**: RGBA unpacking
- **Game state lifecycle tests**: in_init flag, quit_requested flag, timing defaults, RNG seed

### [STABILITY] Fix unclosed HTML tag documentation warning

- Verified: No HTML tag warnings exist. `cargo doc --no-deps` builds cleanly with `RUSTDOCFLAGS="-D warnings"`.
- All generic type references in doc comments are properly escaped with backticks.

### [STABILITY] Suppress dead_code warnings for public API helpers in console.rs

- Added `#[allow(dead_code)]` to `Button` enum, `Button::mask()`, and `ZInput` helper methods
- These are public API items for console-side code, used by tests but not by FFI (WASM games use FFI)
- Added documentation explaining why dead_code is allowed

### [STABILITY] Add tests for app.rs state machine

- Added 39 new tests covering the state machine and related functionality
- Test coverage for `AppMode` enum, `RuntimeError` struct, `AppError` enum, `DebugStats` struct
- Test coverage for state transitions, runtime error handling, UI actions
- Test coverage for fullscreen toggle, resize validation, debug overlay toggle, frame time tracking
- All 331 tests passing (159 core + 172 emberware-z)

### [STABILITY] Remove dead code in download.rs

- Removed unused `API_URL` constant, `DownloadError` enum, and `download_game()` function
- Replaced with minimal stub module with doc comment explaining download is not yet implemented
- Removed related "Add tests for download.rs" task as it's no longer applicable

### [STABILITY] Remove dead code variants in app.rs

- Removed `AppMode::Downloading` variant and its render handling (download feature not implemented)
- Removed unused `AppError` variants (`Window`, `Graphics`, `Runtime`) - only `EventLoop` is used
- Simplified `RuntimeError` from enum with unused variants to simple `String` wrapper struct
- Added `#[allow(dead_code)]` with explanation to `handle_runtime_error()` (infrastructure for future use)

### [STABILITY] Document all unsafe blocks with SAFETY comments

- Added SAFETY comments to all unsafe impl blocks in core and emberware-z
- core/src/integration.rs: TestInput Pod/Zeroable impls
- core/src/rollback.rs: NetworkInput and TestInput Pod/Zeroable impls
- core/src/runtime.rs: TestInput Pod/Zeroable impls
- emberware-z/src/graphics.rs: SkyUniforms Pod/Zeroable impls with GPU alignment explanation
- emberware-z/src/app.rs: Already had SAFETY comment for transmute (egui-wgpu 0.30 API bug)
- All unsafe blocks now explain why they are safe (#[repr(C)] POD types, transparent wrappers, scoped transmute)
- All tests passing (292 total: 159 core + 133 emberware-z)

### [STABILITY] Replace panic! calls in shader_gen.rs with Result returns

- Added `ShaderGenError` enum with `InvalidRenderMode` and `MissingNormalFlag` variants
- Changed `generate_shader()` to return `Result<String, ShaderGenError>`
- Changed `get_template()` to return `Result<&'static str, ShaderGenError>`
- Updated `graphics.rs` to handle errors gracefully with fallback to Mode 0 (unlit)
- Updated all tests to handle Result types properly
- Added new tests for error conditions
- All 17 shader_gen tests passing

### Implement multiplayer player model

- Added `PlayerSessionConfig` struct for configuring local vs remote players
- Max 4 players total with flexible local/remote assignment via bitmask
- Constructors: `all_local(n)`, `one_local(n)`, `with_local_players(n, &[])`, `new(n, mask)`
- Methods: `is_local_player()`, `local_player_count()`, `remote_player_count()`, `local_player_indices()`, `remote_player_indices()`
- Added `configure_session(player_count, local_player_mask)` to `GameInstance`
- Updated `RollbackSession` to store and expose `PlayerSessionConfig`
- 24 new tests for PlayerSessionConfig and integration with RollbackSession

### Create developer guide

- Getting started tutorial: Step-by-step first game walkthrough
- Best practices for rollback-safe code: Determinism checklist, RNG usage, state management
- Asset pipeline recommendations: Embedding assets, image conversion, texture guidelines
- Debugging tips: log() usage, F3 overlay, common issues table, WASM size optimization

### Create `platformer` example

- Full mini-game demonstrating multiple Z features:
  - 2D gameplay using 3D renderer (side-scrolling platformer)
  - Textured sprites for player (8x8 character silhouette)
  - Platform tiles (8x8 brick pattern)
  - Collectible coins (8x8 golden circle with highlight)
  - Billboarded sprites in 3D space (MODE_CYLINDRICAL_Y for upright sprites)
  - Simple physics (gravity, friction, variable jump height)
  - AABB collision detection (platforms, collectibles)
  - Multiple players (up to 4) with analog stick input
  - 2D UI overlay with `draw_text()`, `draw_rect()` (scores, controls, game over)
  - Sky background with `set_sky()` (sunny day preset)
  - Rollback-safe game state via `save_state()`/`load_state()`
- ~700 lines of Rust code

### Create `billboard` example

- Demonstrates billboard drawing:
  - `draw_billboard()` with different modes (1-4)
  - Side-by-side comparison of all 4 modes (spherical, cylindrical Y/X/Z)
  - Particle system with spherical billboards (always face camera)
  - Tree/foliage sprites with cylindrical Y (stay upright)
  - Ground markers to show depth
  - Interactive camera rotation via analog stick
  - UI overlay with controls and mode labels
- ~450 lines of Rust code

### Update documentation

- Fixed `delete` return value in ffi.md (was "1 on success", now "0 on success, 1 if invalid slot")
- Fixed WASM import module name in emberware-z.md example (was "emberware", now "env")
- Added troubleshooting section with 7 common issues and solutions
- Added performance tips section covering vertex formats, batching, render modes, textures, and CPU budget

### Create `skinned-mesh` example

- Demonstrates GPU skinning:
  - Load skinned mesh with FORMAT_NORMAL | FORMAT_SKINNED (format 12)
  - 3-bone arm hierarchy with smooth weight blending
  - CPU-side bone animation (sine wave for wave-like bending)
  - `set_bones()` to upload bone matrices each frame
  - Shows workflow: CPU animation → GPU skinning
  - Interactive controls: L-stick rotate view, A pause, D-pad speed
  - Generates cylindrical arm mesh procedurally with vertex weights
- ~520 lines of Rust code

### Create `lighting` example

- Demonstrates PBR lighting (render mode 2):
  - `render_mode()` to select PBR mode
  - `set_sky()` for procedural sky lighting
  - `light_set()`, `light_color()`, `light_intensity()`, `light_disable()` for 4 dynamic lights
  - `material_metallic()`, `material_roughness()` for PBR materials
  - Interactive light positioning via analog sticks
  - Material property adjustment via triggers
  - Light toggling via face buttons
  - Icosphere mesh for demonstrating lighting
  - UI overlay showing current settings
- Note: render_mode is init-only, so mode selection is a compile-time constant
- ~400 lines of Rust code

### Create `cube` example

- Demonstrates retained mode 3D:
  - `load_mesh_indexed()` in init()
  - `draw_mesh()` in render()
  - Vertex format: POS_UV_NORMAL (format 5)
  - Camera setup: `camera_set()`, `camera_fov()`
  - Interactive rotation via analog stick
  - Mode 0 with normals (simple Lambert)
  - Procedural sky via `set_sky()` for lighting
  - 8x8 checkerboard texture with nearest-neighbor filtering
- ~200 lines of Rust code

### Create `textured-quad` example

- Demonstrates texture loading and 2D drawing:
  - `load_texture()` to create a texture from RGBA pixel data
  - `texture_bind()` to bind the texture for drawing
  - `draw_sprite()` for 2D sprite rendering
  - `set_color()` for tinting with color cycling animation
  - `texture_filter()` for nearest-neighbor filtering
- 8x8 checkerboard pattern (cyan/magenta) generated at compile time
- Triangle wave color animation (no libm required for no_std)
- Added to workspace exclude list
- ~125 lines of Rust code

### [STABILITY] Add tests for GPU skinning

- Added 9 tests to `core/src/wasm.rs`: RenderState bone matrix storage, clearing, max capacity, weighted blend simulation, bone hierarchy simulation
- Added 13 tests to `emberware-z/src/graphics.rs`: All 8 skinned vertex format stride calculations, VertexFormatInfo skinned flags, skinned format isolation, command buffer skinned vertices
- Added 15 tests to `emberware-z/src/ffi.rs`: Skinned format flag values, vertex stride calculations, MAX_BONES constant, RenderState bone matrix operations, bone matrix transforms, column-major layout verification, bone weight sum convention
- Total: 37 new GPU skinning tests (270 tests total, up from 233)

### [STABILITY] Add tests for input system

- Added 22 new tests (32 total input tests, up from 10):
- **Player slot assignment tests**: Sequential assignment, all full, gaps, disconnect freeing slots
- **Deadzone edge cases**: Negative values, max values, zero deadzone, boundary conditions
- **Get player input tests**: Valid range, out of range bounds checking
- **Keyboard input tests**: D-pad, face buttons, start/select, analog is zero, custom mapping
- **InputConfig tests**: Default values, partial deserialization
- **KeyCode roundtrip**: Comprehensive test of all supported key types

### [STABILITY] Implement settings web link

- Added `open` crate dependency for cross-platform browser opening.
- `UiAction::OpenBrowser` now opens https://emberware.io in the default browser.
- Error handling with tracing for failed browser open.

### [STABILITY] Add keyboard mapping serialization

- Implemented string-based KeyCode serialization (`keycode_to_string`, `string_to_keycode`).
- `KeyboardMapping` now derives `Serialize`/`Deserialize` with serde attributes.
- Supports 80+ key names (letters, numbers, arrows, function keys, modifiers, punctuation, numpad).
- Human-readable TOML config: `dpad_up = "ArrowUp"` instead of internal enum values.
- 10 new tests for serialization roundtrip, custom key parsing, and deadzone application.

### [STABILITY] Fix input not passed to game during rollback

- Added `to_input_state()` method to `ConsoleInput` trait for converting console-specific inputs to common `InputState`.
- Implemented `to_input_state()` for `ZInput` in emberware-z.
- Updated runtime to pass GGRS confirmed inputs to game before calling `update()` during rollback.
- Enables deterministic rollback netcode: games now receive correct inputs during replay.

### [STABILITY] Replace expect() calls in graphics initialization

- Changed `create_fallback_textures()` to return `Result<()>` and propagate errors.
- Uses `.context()` instead of `.expect()` for proper error context.
- Errors now bubble up through `ZGraphics::new()` for graceful handling.

### [STABILITY] Fix WasmEngine::Default panic

- Removed `Default` impl which violated Rust conventions (panicked on failure).
- Added documentation explaining why `Default` is not implemented.
- Callers should use `WasmEngine::new()` which returns `Result<Self>`.

### [STABILITY] Add comments to dead_code suppressions

- Added doc comments explaining `console` field in `Runtime` is kept for future use.
- Added doc comments explaining `instance` field in `GameInstance` is required for lifetime.

### [STABILITY] Fix potential panic in transform_set()

- Changed `.try_into().expect()` to use `let Ok(matrix) = ... else { warn; return; }` pattern.
- Now returns early with warning instead of panicking.

### [STABILITY] Fix gamepad initialization double-panic

- Changed `gilrs` field from `Gilrs` to `Option<Gilrs>`.
- Gracefully disables gamepad support if Gilrs::new() fails.
- Input polling handles None case by skipping gamepad events.

### [STABILITY] Remove outdated TODO comment

- Removed misleading comment that listed FFI functions as TODO when they were already implemented.
- Updated comment to accurately describe what register_ffi does.

### Create `triangle` example

- Minimal no_std WASM game demonstrating:
  - `draw_triangles()` (immediate mode 3D)
  - Vertex format: POS_COLOR (format 2)
  - Transform stack: `transform_rotate()` for spinning
  - Game lifecycle: init/update/render
- ~50 lines of Rust code
- Also fixed hello-world example to use correct FFI module ("env" instead of "emberware")
- Updated workspace Cargo.toml to exclude examples from workspace

### Create graphics tests

- **Shader compilation tests (all 40 shaders)**: 15 tests in shader_gen.rs
  - Validates all mode/format combinations
  - Entry point verification
  - Template placeholder validation
- **Vertex format stride validation**: 11 tests for all 16 formats
- **Texture loading and binding tests**: 4 new tests
- **Render state switching tests**: 9 new tests
- Total graphics tests: 42 (was 29, added 13 new tests)
- All 206 tests passing (132 core + 74 emberware-z)

### Update documentation

- Fixed save function return value documentation in ffi.md
- Fixed light_set signature in emberware-z.md
- Fixed Mode 3 (Hybrid) docs to use actual FFI functions
- All tests passing (62 tests)

### Create integration tests

- Added integration.rs module with 24 comprehensive tests
- **Game lifecycle tests**: Full init/update/render flow, minimal games, state persistence
- **Rollback simulation tests**: Save/load state, RollbackStateManager, checksum verification, multiple save points
- **Multi-player input tests**: Input state rotation, 4-player handling, local player mask, console input mapping
- **Resource limit tests**: Console specs, texture/mesh allocation tracking, save slot limits, transform stack limits, rollback state size, player count limits, draw command buffer

### Create unit tests for core framework

- Added wat dev-dependency for test WASM module parsing
- **wasm.rs**: 48 tests for WASM loading, GameState, CameraState, InputState, DrawCommand, transforms
- **ffi.rs**: 17 tests for FFI registration and bindings
- **runtime.rs**: 21 tests for Runtime and Console trait
- Total: 108 tests (from 26 baseline) - all passing

### Create shader compilation tests (all 40 shaders)

- Added naga as dev-dependency for WGSL parsing and validation
- `compile_and_validate_shader()` helper validates each shader permutation
- Tests for all mode/format combinations
- Entry point and placeholder verification
- Fixed shader bug: Modes 2/3 used wrong variable name (`color` vs `albedo`)
- Added `FS_ALBEDO_COLOR` and `FS_ALBEDO_UV` snippets for PBR/Hybrid modes

### Implement application state machine

- States: Library → Downloading → Playing → back to Library
- Error handling transitions via RuntimeError enum
- Handle errors: WasmPanic, NetworkDisconnect, OutOfMemory, Other
- Error display in Library UI with dismiss button
- handle_runtime_error() transitions back to library with error message

### Implement debug overlay (console-wide)

- FPS counter (update rate)
- Frame time with millisecond precision
- Frame time graph (120 sample history, color-coded bars)
- VRAM usage (current/limit with progress bar)
- Network stats (ping, rollback frames, frame advantage) - ready for P2P sessions
- Toggle via F3 hotkey
- Resizable debug window

### Handle GGRS events

- Integrated RollbackSession into Runtime game loop
- Runtime now owns optional RollbackSession<C::Input> and Audio backend
- Added methods: `set_session()`, `set_audio()`, `add_local_input()`, `poll_remote_clients()`, `handle_session_events()`
- Modified `frame()` to call `advance_frame()`, `handle_requests()`, and process GGRS requests
- `GGRSRequest::SaveGameState` → calls `session.handle_requests()` which saves WASM state via `game.save_state()`
- `GGRSRequest::LoadGameState` → calls `session.handle_requests()` which restores WASM state via `game.load_state()`
- `GGRSRequest::AdvanceFrame` → executes `game.update()` with confirmed inputs
- Audio muting during rollback via `audio.set_rollback_mode(session.is_rolling_back())`
- Session events (desync, network interruption, frame advantage warnings) exposed via `handle_session_events()`
- All tests passing with no warnings

### Implement egui integration for library UI

- egui-wgpu renderer setup with wgpu 23 (for egui 0.30 compatibility)
- egui-winit state integration for event handling
- Library screen with game list and selection
- Game actions: play, view info, delete
- Settings screen placeholder (not yet implemented)
- Download progress UI placeholder
- Debug overlay with F3 toggle showing FPS, frame time, mode
- Application state machine (Library → Playing → Settings)
- Error handling integration

### Implement keyboard/gamepad input

- Keyboard mapping to virtual controller (configurable)
- Gamepad support via gilrs
- Multiple local players (keyboard + gamepads)
- Input config persistence in config.toml
- Deadzone and sensitivity settings
- Created input.rs module with InputManager
- Integrated with App event loop
- Automatic gamepad detection and player slot assignment
- Deadzone application for analog sticks and triggers
- All tests passing

### Implement winit window management

- Window creation with configurable resolution (1920x1080 default)
- Fullscreen toggle via F11 key
- Event loop integration using winit 0.30 ApplicationHandler trait
- Window resize handling with graphics backend resize calls
- DPI/scale factor change handling
- Window close handling with graceful shutdown
- Escape key returns to library from game mode
- Fullscreen state persistence in config.toml
- Integration with ZGraphics backend initialization

### Implement Built-in Font

- Created `font.rs` module with 8x8 monospace bitmap font
- ASCII 32-126 (95 printable characters) in 128x48 texture atlas
- `get_glyph_uv(char_code)` returns normalized UV coordinates
- `generate_font_atlas()` creates RGBA8 texture data at runtime
- Font texture loaded during ZGraphics initialization
- `font_texture()` and `get_font_texture_view()` accessors added
- `generate_text_quads(text, x, y, size, color)` generates POS_UV_COLOR vertices
- Screen-space text rendering with variable size via scaling
- Left-aligned, single-line (no word wrap in v1)

### Implement GPU Skinning

- Bone storage buffer: 256 bones × 4×4 matrices = 16KB
- `set_bones(matrices_ptr, count)` FFI function implemented and registered
- Bone matrices stored in RenderState (Vec<Mat4>)
- Graphics trait extended with `set_bones()` method for GPU upload
- ZGraphics implements bone matrix upload via wgpu queue.write_buffer()
- Skinned vertex shader code in shader templates (VIN_SKINNED, VS_SKINNED)
- FORMAT_SKINNED (8) adds 20 bytes per vertex (4 u8 indices + 4 f32 weights)
- Vertex attribute order: pos → uv → color → normal → bone_indices → bone_weights
- All 16 vertex format permutations supported (8 base + 8 skinned variants)

### Implement Mode 2 (PBR) and Mode 3 (Hybrid) lighting functions

- `light_set(index, x, y, z)` — set directional light direction for light 0-3
- `light_color(index, r, g, b)` — set light color (linear RGB, supports HDR values > 1.0)
- `light_intensity(index, intensity)` — set light intensity multiplier
- `light_disable(index)` — disable light
- Light state tracked in RenderState (4 light slots)
- LightState struct with enabled, direction, color, intensity
- All lights are directional (normalized direction vectors)
- Mode 2: Uses all 4 lights in shader
- Mode 3: Uses light 0 as single directional light (same FFI functions)
- FFI validation: index range (0-3), zero-length direction vectors, negative color/intensity values

### Implement Mode 1 (Matcap) functions

- `matcap_set(slot, texture)` — bind matcap to slot 1-3
- Validate slot is 1-3, warn otherwise
- FFI function registered and implemented

### Implement material functions

- `material_mre(texture)` — bind MRE texture (R=Metallic, G=Roughness, B=Emissive)
- `material_albedo(texture)` — bind albedo texture (alternative to slot 0)
- `material_metallic(value)` — set metallic (0.0-1.0, default 0.0)
- `material_roughness(value)` — set roughness (0.0-1.0, default 0.5)
- `material_emissive(value)` — set emissive intensity (default 0.0)
- Material properties added to RenderState struct
- All FFI functions registered and implemented with validation

### Implement Procedural Sky System

- Created `SkyUniforms` struct with horizon_color, zenith_color, sun_direction, sun_color, sun_sharpness
- Implemented sky uniform buffer with GPU upload
- Added `set_sky()` FFI function with 13 f32 parameters
- Sun direction automatically normalized
- `sample_sky()` shader function implemented in all shader templates
- Gradient interpolation: `mix(horizon, zenith, direction.y * 0.5 + 0.5)`
- Sun calculation: `sun_color * pow(max(0, dot(direction, sun_direction)), sharpness)`
- Integrated with all render modes (0-3) for ambient lighting and reflections
- Default: all zeros (black sky, no sun, no lighting until configured via set_sky())

### Implement Shader Generation System

- Created 4 shader mode templates: mode0_unlit.wgsl, mode1_matcap.wgsl, mode2_pbr.wgsl, mode3_hybrid.wgsl
- Implemented template placeholder replacement system (`//VIN_*`, `//VOUT_*`, `//VS_*`, `//FS_*`)
- Template replacement function: `generate_shader(mode, format)`
- Mode 0 (Unlit): 16 shader permutations for all vertex formats
- Modes 1-3 (Matcap, PBR, Hybrid): 8 permutations each (formats with NORMAL flag)
- Total: 40 shader variations (16 + 8 + 8 + 8)
- Shader compilation and caching system
- Pipeline cache by (mode, format, blend_mode, depth_test, cull_mode)
- Bind group layouts for per-frame uniforms (group 0) and textures (group 1)
- Procedural sky system (gradient + sun) integrated in shaders
- Simple Lambert shading for Mode 0 with normals
- Matcap lighting (Mode 1) with 3 matcap texture slots
- PBR-lite (Mode 2): GGX specular, Schlick fresnel, up to 4 dynamic lights
- Hybrid mode (Mode 3): PBR direct lighting + matcap ambient

### Implement 2D Drawing FFI

- `draw_sprite(x, y, w, h, color)` — draw bound texture in screen space
- `draw_sprite_region(x, y, w, h, src_x, src_y, src_w, src_h, color)` — sprite sheet support
- `draw_sprite_ex(x, y, w, h, src_x, src_y, src_w, src_h, origin_x, origin_y, angle_deg, color)` — full control with rotation
- `draw_rect(x, y, w, h, color)` — solid color rectangle
- `draw_text(ptr, len, x, y, size, color)` — UTF-8 text rendering
- DrawSprite, DrawRect, and DrawText commands added to DrawCommand enum
- Screen-space coordinates (0,0 = top-left)
- UTF-8 validation and proper string handling

### Implement billboard drawing

- `draw_billboard(w, h, mode, color)` — draw billboard at current transform
- `draw_billboard_region(w, h, src_x, src_y, src_w, src_h, mode, color)` — with UV region
- Billboard modes: 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
- DrawBillboard command added to DrawCommand enum
- FFI functions registered and implemented with validation

### Implement Mesh FFI functions

- `load_mesh(data_ptr, vertex_count, format) -> u32` — non-indexed mesh
- `load_mesh_indexed(data_ptr, vertex_count, index_ptr, index_count, format) -> u32`
- `draw_mesh(handle)` — draw retained mesh with current transform/state
- Vertex format validation (0-15)
- Stride calculation and memory bounds checking
- PendingMesh struct for graphics backend integration
- DrawCommand enum for deferred rendering

### Implement Immediate Mode 3D FFI

- `draw_triangles(data_ptr, vertex_count, format)` — non-indexed immediate draw
- `draw_triangles_indexed(data_ptr, vertex_count, index_ptr, index_count, format)`
- Vertex format validation and stride calculation
- Index bounds checking
- Draw commands buffered with current transform and render state

### Implement Input FFI functions

- Individual button queries: `button_held`, `button_pressed`, `button_released`
- Bulk button queries: `buttons_held`, `buttons_pressed`, `buttons_released`
- Analog stick queries: `left_stick_x`, `left_stick_y`, `right_stick_x`, `right_stick_y`
- Bulk stick queries: `left_stick`, `right_stick`
- Trigger queries: `trigger_left`, `trigger_right`
- Full player validation (0-3), button validation (0-13)

### Implement Texture FFI functions

- `load_texture(width, height, pixels_ptr) -> u32` — creates texture from RGBA data
- `texture_bind(handle)` — bind to slot 0
- `texture_bind_slot(handle, slot)` — bind to specific slot (0-3)
- PendingTexture struct for graphics backend integration
- WASM memory bounds validation

### Implement Render State FFI functions

- `set_color(color)` — uniform tint color (0xRRGGBBAA)
- `depth_test(enabled)` — enable/disable depth testing
- `cull_mode(mode)` — 0=none, 1=back, 2=front
- `blend_mode(mode)` — 0=none, 1=alpha, 2=additive, 3=multiply
- `texture_filter(filter)` — 0=nearest, 1=linear
- Input validation with warnings for invalid values

### Implement camera functions

- `camera_set(x, y, z, target_x, target_y, target_z)` — look-at camera
- `camera_fov(fov_degrees: f32)` — field of view (default 60°)
- View matrix calculation
- Projection matrix calculation
- CameraState struct with view_matrix(), projection_matrix(), view_projection_matrix()

### Implement transform stack functions

- `transform_identity()` — reset to identity matrix
- `transform_translate(x, y, z)` — translate
- `transform_rotate(angle_deg, x, y, z)` — rotate around axis
- `transform_scale(x, y, z)` — scale
- `transform_push()` — push current matrix to stack (returns 1/0)
- `transform_pop()` — pop matrix from stack (returns 1/0)
- `transform_set(matrix_ptr)` — set from 16 floats (column-major)
- Stack depth: 16 matrices
- Matrix math using glam

### Implement configuration functions

- `set_resolution(res: u32)` — 0=360p, 1=540p, 2=720p, 3=1080p
- `set_tick_rate(fps: u32)` — 24, 30, 60, 120
- `set_clear_color(color: u32)` — 0xRRGGBBAA background color
- `render_mode(mode: u32)` — 0-3 (Unlit, Matcap, PBR, Hybrid)
- Enforce init-only: error/warning if called outside `init()`

### Implement vertex buffer architecture

- One vertex buffer per stride (format determines buffer)
- `GrowableBuffer` struct for auto-growing GPU buffers
- 8 base vertex formats (POS, POS_UV, POS_COLOR, etc.)
- 8 skinned variants (each base format + skinning data)
- Total: 16 vertex format pipelines per render mode

### Implement command buffer pattern

- Immediate-mode draws buffered on CPU side
- Single flush to GPU per frame (minimize draw calls)
- Draw command batching by pipeline/texture state
- Retained mesh handles separate from immediate draws

### Implement wgpu device initialization

- `ZGraphics` struct implementing `Graphics` trait
- wgpu `Instance`, `Adapter`, `Device`, `Queue` setup
- Surface configuration for window
- Resize handling with surface reconfiguration

### Implement texture management

- `TextureHandle` allocation and tracking
- `load_texture(width, height, pixels)` — create RGBA8 texture
- VRAM budget tracking (8MB limit)
- Fallback textures: 8×8 magenta/black checkerboard, 1×1 white
- Sampler creation (nearest, linear filters)

### Implement render state management

- Current color (uniform tint)
- Depth test enable/disable
- Cull mode (none, back, front)
- Blend mode (none, alpha, additive, multiply)
- Texture filter (nearest, linear)
- Currently bound textures (slots 0-3)

### Create Emberware Z `Console` implementation

- Implement `Console` trait for PS1/N64 aesthetic
- Define Z-specific specs:
  - Resolution: 360p, 540p (default), 720p, 1080p
  - Tick rate: 24, 30, 60 (default), 120 fps
  - RAM: 16MB, VRAM: 8MB, ROM: 32MB max
  - Color depth: RGBA8
  - CPU budget: 4ms per tick at 60fps
- `ZInput` struct (buttons, dual sticks, triggers)

### Handle GGRS events

- `GGRSRequest::SaveGameState` → serialize WASM state
- `GGRSRequest::LoadGameState` → deserialize WASM state
- `GGRSRequest::AdvanceFrame` → run `update()` with confirmed inputs
- Connection quality events (desync detection, frame advantage warnings)
- Audio muting during rollback replay

### Integrate GGRS session into runtime

- Local session (single player or local multiplayer, no rollback)
- P2P session with matchbox_socket (WebRTC)
- `advance_frame()` with GGRS requests handling
- Synchronization test mode for local debugging
- TODO [needs clarification]: Spectator session support

### Define GGRS config and input types

- `GGRSConfig` implementing `ggrs::Config` trait
- Generic input type parameterized by console's `ConsoleInput`
- Input serialization for network (bytemuck Pod)
- Input delay and frame advantage settings

### Implement rollback state management

- `save_game_state()` — call WASM `save_state`, store snapshot with checksum
- `load_game_state()` — call WASM `load_state`, restore snapshot
- State buffer pool for efficient rollback (avoid allocations in hot path)
- State compression (optional, for network sync)

### Create `core` crate with workspace configuration

- Add `core/Cargo.toml` with wasmtime, ggrs, matchbox_socket, winit
- Update root `Cargo.toml` workspace members
- Define core module structure: `lib.rs`, `console.rs`, `runtime.rs`, `wasm.rs`, `ffi.rs`, `rollback.rs`

### Create repository structure

- Root Cargo.toml workspace
- README.md with project overview
- CLAUDE.md with development instructions
- .gitignore and LICENSE

### Create `shared` crate

- API types: Game, Author, User, Auth responses
- Request/response types for platform API
- LocalGameManifest for downloaded games
- Error types and codes

### Create `emberware-z` crate skeleton

- Cargo.toml with dependencies
- main.rs entry point
- app.rs application state
- config.rs configuration management
- deep_link.rs URL parsing
- download.rs ROM fetching
- library.rs local game management
- ui.rs egui library interface
- runtime/mod.rs module declaration (stubs)

### Create FFI documentation

- docs/ffi.md with complete API reference
- All function signatures and examples
- Console specs and lifecycle documentation

### Create hello-world example

- Minimal no_std WASM game
- Demonstrates init/update/render lifecycle
- Basic input and rendering

### Initialize git repository and push to GitHub

### Define `Console` trait and associated types

- `Console` trait with specs, FFI registration, graphics/audio factory methods
- `Graphics` trait for rendering backend abstraction
- `Audio` trait for audio backend abstraction
- `ConsoleInput` trait with bytemuck requirements for GGRS serialization
- `ConsoleSpecs` struct (resolutions, tick rates, RAM/VRAM limits, ROM size)

### Implement `GameState` for WASM instance

- Wasmtime `Store` data structure containing all per-game state
- FFI context: graphics command buffer, audio commands, RNG state
- Input state for all 4 players
- Transform stack (16 matrices deep)
- Current render state (color, blend mode, depth test, cull mode, filter)
- Save data slots (8 slots × 64KB max each)

### Implement WASM runtime wrapper

- `WasmEngine` — shared wasmtime `Engine` (one per app)
- `GameInstance` — loaded game with `Module`, `Instance`, `Store`
- Export function bindings: `init()`, `update()`, `render()`
- Export function bindings: `save_state(ptr, max_len) -> len`, `load_state(ptr, len)`
- Memory access helpers for FFI string/buffer passing
- WASM memory bounds checking and validation

### Implement common FFI host functions

- System functions: `delta_time`, `elapsed_time`, `tick_count`, `log`, `quit`
- Rollback functions: `random` (deterministic PCG)
- Save data functions: `save`, `load`, `delete`
- Session functions: `player_count`, `local_player_mask`

### Implement game loop orchestration

- Fixed timestep update loop (configurable tick rate: 24, 30, 60, 120)
- Variable render rate with interpolation support (uncapped frame rate)
- Frame timing and delta time calculation
- Update/render separation (render skipped during rollback replay)
- CPU budget enforcement (4ms per tick at 60fps, warn on exceed)
