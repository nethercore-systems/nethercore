# Emberware Development Tasks

## Needs Clarification

These items are marked TODO throughout the document and need decisions before implementation:

- **[STABILITY] Audio backend** — Architecture, formats, sample rates, channel count (shelved for now)
  - `ZAudio::play()`, `ZAudio::stop()`, and `create_audio()` are stubs in `emberware-z/src/console.rs`
- **Custom fonts** — Allow games to load custom fonts for draw_text?
- **Spectator support** — GGRS spectator sessions for watching games
- **Matchmaking** — Handled by platform service, but integration details TBD
- **Matcap blend modes** — Currently multiply only; future: add, screen, overlay, HSV shift, etc.

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

### Stability (Shelved)

- **[STABILITY] Implement audio backend** — See "Needs Clarification" section above

### Phase 5: Networking & Polish

- **Implement matchbox signaling connection** [NEEDS CLARIFICATION]
  - Connect to matchbox signaling server
  - WebRTC peer connection establishment
  - ICE candidate exchange
  - Connection timeout handling
  - Matchmaking handled by platform service - integration details TBD

- **Implement netplay session management**
  - Host/join game flow via platform deep links
  - Connection quality display (ping bars)
  - Disconnect handling (return to library)
  - Session cleanup on exit

- **Implement local network testing**
  - Multiple instances on same machine via localhost
  - Connect via `127.0.0.1:port` for local testing
  - Debug mode: disable matchbox, use direct connections

- **Performance optimization**
  - Render batching already implemented in CommandBuffer
  - Profile and optimize hot paths - requires game execution to measure

### Phase 8: Game Execution Integration

- **Wire up game execution in Playing mode**
  - Load WASM from LocalGame.rom_path when entering Playing mode
  - Create Runtime<ZConsole> with game instance
  - Run game loop: poll input → update() → render()
  - Pass ZInput from InputManager to game via FFI
  - Execute ZGraphics draw commands to render frame
  - Handle game errors (WASM trap, OOM) → return to Library with error

---
## In Progress

(empty)

---

## Done

- **Add input delay configuration** (Phase 5)
  - `NetplayConfig` struct with `input_delay: u8` (0-10 frames, default 2)
  - Settings UI with slider and explanatory text
  - Auto-saves to config.toml on change
  - Already integrated with `SessionConfig` in core (uses `with_input_delay()`)

- **Rollback state memory optimization** (Phase 5)
  - `StatePool` with pre-allocated buffers to avoid allocations in hot path
  - Buffer acquire/release pattern with automatic recycling
  - Oversized buffers discarded to prevent memory bloat
  - Pool exhaustion handled gracefully with new allocation and warning

- **[STABILITY] Add bounds checking for potentially truncating type casts** (codebase-wide)
  - Added `checked_mul()` overflow protection in FFI functions:
    - `load_texture`: width × height × 4 calculation
    - `load_mesh`: vertex_count × stride calculation
    - `load_mesh_indexed`: vertex_count × stride and index_count × 4 calculations
    - `draw_triangles`: vertex_count × stride calculation
    - `draw_triangles_indexed`: vertex_count × stride and index_count × 4 calculations
  - Returns 0/early returns with warning on overflow instead of wrapping
  - Added 5 new tests for arithmetic overflow protection:
    - `test_texture_size_overflow_protection`
    - `test_vertex_data_size_overflow_protection`
    - `test_index_data_size_overflow_protection`
    - `test_realistic_mesh_sizes_no_overflow`
    - `test_realistic_texture_sizes_no_overflow`
  - All 559 tests passing

- **[STABILITY] Document resource cleanup strategy** (graphics resources)
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

- **[STABILITY] Review clone operations for optimization** (multiple files)
  - Optimized `core/src/ffi.rs:118`: Used `data_and_store_mut()` to eliminate O(n) Vec clone in save data load
  - Analyzed `emberware-z/src/app.rs:303,306,308`: Clones are necessary due to borrow checker constraints with egui closures
    - `mode.clone()` - enum clone, O(1) or O(n) for game_id string, unavoidable
    - `last_error.clone()` - only allocates when there's an error, rare path
    - `window.clone()` - Arc<Window> increment, O(1), already optimal
  - Analyzed `emberware-z/src/graphics/command_buffer.rs:297`: Test code only, not production
  - All 554 tests passing

- **[STABILITY] Reduce DRY violations in vertex attribute generation** (`emberware-z/src/graphics/vertex.rs`)
  - Replaced 340-line match statement with data-driven static array
  - Created helper functions: `attr_pos()`, `attr_uv()`, `attr_color()`, `attr_normal()`, `attr_bone_indices()`, `attr_bone_weights()`
  - Added size constants: `SIZE_POS`, `SIZE_UV`, `SIZE_COLOR`, `SIZE_NORMAL`, `SIZE_BONE_INDICES`
  - Added shader location constants: `LOC_POS`, `LOC_UV`, `LOC_COLOR`, `LOC_NORMAL`, `LOC_BONE_INDICES`, `LOC_BONE_WEIGHTS`
  - `VERTEX_ATTRIBUTES` static array holds pre-computed attribute slices for all 16 formats
  - `build_attributes()` now just indexes into the array
  - Reduced code from ~340 lines to ~170 lines (50% reduction)
  - All 554 tests passing

- **[STABILITY] Split wasm.rs into modules** (`core/src/wasm.rs`)
  - Split 1917 lines into 5 submodules:
    - `camera.rs`: CameraState, DEFAULT_CAMERA_FOV (~115 lines)
    - `draw.rs`: DrawCommand, PendingTexture, PendingMesh (~380 lines)
    - `input.rs`: InputState (~75 lines)
    - `render.rs`: LightState, RenderState, InitConfig, MAX_BONES (~240 lines)
    - `state.rs`: GameState, MAX_* constants (~160 lines)
    - `mod.rs`: WasmEngine, GameInstance, re-exports (~970 lines)
  - All 183 core tests passing
  - All public API preserved via re-exports


- **[STABILITY] Split graphics.rs into modules** (`emberware-z/src/graphics.rs`)
  - Split 3436 lines into 5 submodules:
    - `vertex.rs`: Vertex format constants, VertexFormatInfo, stride calculations (~600 lines)
    - `buffer.rs`: GrowableBuffer, MeshHandle, RetainedMesh (~180 lines)
    - `render_state.rs`: CullMode, BlendMode, TextureFilter, SkyUniforms, RenderState, TextureHandle (~470 lines)
    - `command_buffer.rs`: DrawCommand, CommandBuffer (~270 lines)
    - `pipeline.rs`: PipelineKey, PipelineEntry, create_pipeline, bind group layouts (~290 lines)
    - `mod.rs`: ZGraphics struct, core methods, re-exports (~1050 lines)
  - All 371 tests passing
  - All public API preserved via re-exports

- **[STABILITY] Add negative test cases for FFI error conditions** (`emberware-z/src/ffi/mod.rs`)
  - Added 67 new tests (139 total FFI tests, up from 72)
  - **Invalid texture handle tests**: Zero handle, unloaded handle, slot independence
  - **Invalid mesh handle tests**: Zero handle rejection, unloaded handle handling
  - **Out-of-range parameter tests**: Resolution index, tick rate index, render mode, cull mode, blend mode, texture filter, vertex format, billboard mode, matcap slot, light index
  - **Edge case tests**: Camera FOV clamping, transform rotate zero axis, material property clamping, light color/intensity negative values, light direction zero vector, transform stack overflow/underflow, bone count limits, draw triangles vertex count, mesh index count, texture dimensions, init-only guards, draw command buffer growth, pending resource growth, handle allocation overflow, special float values (NaN, infinity)

- **[STABILITY] Review dead_code allowances** (multiple files)
  - Verified all 4 dead_code allowances are properly documented and necessary:
  - `core/src/wasm.rs:494`: `instance` field must be kept alive for WASM function lifetimes
  - `core/src/runtime.rs:44`: `console` field kept for future console-specific features
  - `emberware-z/src/app.rs:149`: `handle_runtime_error` is infrastructure for future use
  - `emberware-z/src/console.rs:67,85,121`: Button enum/helpers are public API for tests and console-side code

- **[STABILITY] Clarify runtime TODO comment** (`emberware-z/src/runtime/mod.rs:15`)
  - Replaced outdated TODO with accurate module layout documentation
  - Now lists actual file locations for all runtime components across core and emberware-z
  - References TASKS.md for unimplemented audio feature

- **[STABILITY] Add error path tests for WASM memory access** (`core/src/wasm.rs`, `core/src/ffi.rs`)
  - Added 14 new tests to `core/src/ffi.rs` for FFI memory access error paths:
    - `test_log_message_out_of_bounds`, `test_log_message_wrapping_overflow`, `test_log_no_memory`
    - `test_save_invalid_slot`, `test_save_data_too_large`, `test_save_out_of_bounds_pointer`, `test_save_no_memory`
    - `test_load_invalid_slot`, `test_load_empty_slot`, `test_load_out_of_bounds_pointer`, `test_load_no_memory`
    - `test_delete_invalid_slot`, `test_save_load_roundtrip`, `test_save_boundary_slot_values`
  - Added 12 new tests to `core/src/wasm.rs` for GameInstance error paths:
    - `test_game_instance_save_state_returns_invalid_length`, `test_game_instance_save_state_oob_ptr`
    - `test_game_instance_load_state_too_large`, `test_game_instance_load_state_no_memory`
    - `test_game_instance_save_state_no_memory`, `test_game_instance_save_state_valid`
    - `test_game_instance_load_state_valid`
    - `test_game_instance_init_trap_propagates`, `test_game_instance_update_trap_propagates`, `test_game_instance_render_trap_propagates`
  - All 304 tests passing (183 core + 121 emberware-z)
  - Tests verify: out-of-bounds memory access, invalid slot handling, buffer overflow protection, missing memory handling, WASM trap propagation

- **[STABILITY] Add documentation to shared crate public APIs** (`shared/src/lib.rs`)
  - Added module-level `//!` doc comment explaining API type categories with example usage
  - Documented all API response structs: `Author`, `Game`, `GamesResponse`, `RomUrlResponse`, `VersionResponse`
  - Documented auth types: `User`, `AuthResponse`, `ApiError` with working doctests
  - Documented `error_codes` module constants
  - Documented local types: `LocalGameManifest` with storage location info
  - Documented request types: `RegisterRequest`, `LoginRequest`, `CreateGameRequest`, `UpdateGameRequest`, `CreateGameResponse`, `UploadUrls`, `SuccessResponse`
  - All 463 tests passing

- **[STABILITY] Split rollback.rs into modules** (`core/src/rollback.rs`)
  - Extracted ~1846 lines into 4 submodules:
    - `config.rs`: GGRS configuration, SessionConfig, constants (127 lines)
    - `player.rs`: PlayerSessionConfig for local/remote player management (267 lines)
    - `state.rs`: GameStateSnapshot, StatePool, RollbackStateManager, error types (240 lines)
    - `session.rs`: RollbackSession, SessionEvent, SessionError, network stats (563 lines)
    - `mod.rs`: Module re-exports and documentation (50 lines)
  - All 159 core tests passing
  - All public API preserved via re-exports in lib.rs

- **[STABILITY] Split ffi.rs input functions into separate module** (`emberware-z/src/ffi.rs`)
  - Extracted input FFI functions (14 functions, ~350 lines) to `ffi/input.rs`
  - Created `ffi/mod.rs` to organize FFI module with public submodule
  - Functions extracted: `button_held`, `button_pressed`, `button_released`, `buttons_held`, `buttons_pressed`, `buttons_released`, `left_stick_x`, `left_stick_y`, `right_stick_x`, `right_stick_y`, `left_stick`, `right_stick`, `trigger_left`, `trigger_right`
  - All 463 tests passing (159 core + 304 emberware-z)
  - `ffi.rs` reduced from 3120 lines to 2250 lines (mod.rs) + 310 lines (input.rs)

- **[STABILITY] Add tests for graphics pipeline** (`emberware-z/src/graphics.rs`)
  - Added 32 new tests (98 total graphics tests, up from 66)
  - **Sky Uniforms tests**: Default values, custom values, struct size (64 bytes), alignment
  - **Retained Mesh tests**: Default values, non-indexed meshes, indexed meshes
  - **Draw Command tests**: Creation, clone
  - **Text Rendering tests**: Empty string, single char, multiple chars, color extraction, position, valid indices
  - **Vertex Attribute tests**: Buffer layout for POS only, full format (FORMAT_ALL), attribute offsets, shader locations
  - **Command Buffer Edge Cases**: Different formats, transform capture, large batch (1000 triangles)

- **[STABILITY] Add tests for ui.rs** (`emberware-z/src/ui.rs`)
  - Added 17 new tests for library UI
  - **LibraryUi tests**: new(), select_game, deselect_game, change_selection
  - **UiAction tests**: All variants (PlayGame, DeleteGame, OpenBrowser, OpenSettings, DismissError)
  - **Trait tests**: Debug formatting, Clone, PartialEq
  - **Edge cases**: Empty string game IDs, unicode game IDs, long game IDs, variant inequality
  - Added `#[derive(Debug, Clone, PartialEq)]` to `UiAction` enum to support tests

- **[STABILITY] Add missing documentation for public APIs**
  - `graphics.rs`: Added docs for `vertex_buffer_layout()` (wgpu layout creation) and `build_attributes()` (shader location assignment)
  - `ui.rs`: Added docs for `LibraryUi` struct, `show()` method, and `UiAction` enum with all variants
  - `config.rs`: Added module-level docs, struct docs for `Config`/`VideoConfig`/`AudioConfig`, and docs for `config_dir()`, `data_dir()`, `load()`, `save()` functions with platform-specific path examples

- **[STABILITY] Add tests for library.rs** (`emberware-z/src/library.rs`)
  - Added 24 new tests for library management functions
  - **LocalGame struct tests**: Clone, Debug trait implementations
  - **get_games_from_dir tests**: Empty dir, nonexistent dir, single game, multiple games, skips files, skips missing/invalid/incomplete manifests, correct rom_path
  - **is_cached_in_dir tests**: Not present, directory only, with rom, complete game
  - **delete_game_in_dir tests**: Nonexistent game, existing game, removes all contents, leaves other games intact
  - **Edge case tests**: Full workflow (add/list/delete), special characters in game ID, unicode in metadata, empty strings, very long game ID
  - Added documentation for `LocalGame`, `get_local_games()`, `is_cached()`, `delete_game()` public APIs
  - Extracted internal testable functions (`get_games_from_dir`, `is_cached_in_dir`, `delete_game_in_dir`) for filesystem testing with temp directories
  - Added `tempfile` as dev-dependency

- **[STABILITY] Add tests for config.rs** (`emberware-z/src/config.rs`)
  - Added 21 new tests for config persistence and validation
  - **Default value tests**: Config, VideoConfig, AudioConfig, helper functions
  - **TOML serialization tests**: Serialize roundtrip, deserialize empty, partial video/audio
  - **Edge case tests**: Volume 0/1, resolution scale values
  - **Directory function tests**: config_dir/data_dir consistency
  - **Trait tests**: Clone, Debug formatting
  - **Load function tests**: Returns valid config without panic

- **[STABILITY] Add tests for FFI validation** (`emberware-z/src/ffi.rs`)
  - Added 57 new tests (72 total FFI tests) covering FFI validation and edge cases
  - **Vertex format tests**: Format constants, stride calculations for all 16 formats, skinned format boundaries
  - **Render state tests**: Defaults (color, depth_test, cull_mode, blend_mode, texture_filter), material defaults, light defaults
  - **Init config tests**: Resolution values, tick rate values, render mode validation
  - **Input state tests**: Defaults, button bitmask layout, stick/trigger range conversions, prev/curr independence
  - **Draw command tests**: All command variants (Mesh, Triangles, Billboard, Sprite, Text, Rect, SetSky) with state capture
  - **Pending resource tests**: Texture/mesh structures, handle increments
  - **Light state tests**: Default values, all fields, four slot validation
  - **Save data tests**: Slot count, Option<Vec<u8>> storage
  - **Color conversion tests**: RGBA unpacking for white, red, transparent, semi-transparent colors
  - **Game state lifecycle tests**: in_init flag, quit_requested flag, timing defaults, RNG seed

- **[STABILITY] Fix unclosed HTML tag documentation warning** (`emberware-z/`)
  - Verified: No HTML tag warnings exist. `cargo doc --no-deps` builds cleanly with `RUSTDOCFLAGS="-D warnings"`.
  - All generic type references in doc comments are properly escaped with backticks.

- **[STABILITY] Suppress dead_code warnings for public API helpers in console.rs** (`emberware-z/src/console.rs`)
  - Added `#[allow(dead_code)]` to `Button` enum, `Button::mask()`, and `ZInput` helper methods
  - These are public API items for console-side code, used by tests but not by FFI (WASM games use FFI)
  - Added documentation explaining why dead_code is allowed

- **[STABILITY] Add tests for app.rs state machine** (`emberware-z/src/app.rs`)
  - Added 39 new tests covering the state machine and related functionality
  - Test coverage for `AppMode` enum (Library, Playing, Settings variants)
  - Test coverage for `RuntimeError` struct (Display, Debug, Clone)
  - Test coverage for `AppError` enum (EventLoop variant)
  - Test coverage for `DebugStats` struct (default values, frame times, network stats)
  - Test coverage for state transitions (Library→Playing, Playing→Library via ESC, Settings→Library)
  - Test coverage for runtime error handling (transitions to Library with error stored)
  - Test coverage for UI actions (PlayGame, DeleteGame, OpenBrowser, OpenSettings, DismissError)
  - Test coverage for fullscreen toggle, resize validation, debug overlay toggle
  - Test coverage for frame time tracking and ring buffer logic
  - All 331 tests passing (159 core + 172 emberware-z)

- **[STABILITY] Remove dead code in download.rs** (`emberware-z/src/download.rs`)
  - Removed unused `API_URL` constant, `DownloadError` enum, and `download_game()` function
  - Replaced with minimal stub module with doc comment explaining download is not yet implemented
  - Removed related "Add tests for download.rs" task as it's no longer applicable

- **[STABILITY] Remove dead code variants in app.rs** (`emberware-z/src/app.rs`)
  - Removed `AppMode::Downloading` variant and its render handling (download feature not implemented)
  - Removed unused `AppError` variants (`Window`, `Graphics`, `Runtime`) - only `EventLoop` is used
  - Simplified `RuntimeError` from enum with unused variants to simple `String` wrapper struct
  - Added `#[allow(dead_code)]` with explanation to `handle_runtime_error()` (infrastructure for future use)

- **[STABILITY] Document all unsafe blocks with SAFETY comments** (46 blocks across codebase)
  - Added SAFETY comments to all unsafe impl blocks in core and emberware-z
  - core/src/integration.rs: TestInput Pod/Zeroable impls
  - core/src/rollback.rs: NetworkInput and TestInput Pod/Zeroable impls
  - core/src/runtime.rs: TestInput Pod/Zeroable impls
  - emberware-z/src/graphics.rs: SkyUniforms Pod/Zeroable impls with GPU alignment explanation
  - emberware-z/src/app.rs: Already had SAFETY comment for transmute (egui-wgpu 0.30 API bug)
  - All unsafe blocks now explain why they are safe (#[repr(C)] POD types, transparent wrappers, scoped transmute)
  - All tests passing (292 total: 159 core + 133 emberware-z)

- **[STABILITY] Replace panic! calls in shader_gen.rs with Result returns** (`emberware-z/src/shader_gen.rs`)
  - Added `ShaderGenError` enum with `InvalidRenderMode` and `MissingNormalFlag` variants
  - Changed `generate_shader()` to return `Result<String, ShaderGenError>`
  - Changed `get_template()` to return `Result<&'static str, ShaderGenError>`
  - Updated `graphics.rs` to handle errors gracefully with fallback to Mode 0 (unlit)
  - Updated all tests to handle Result types properly
  - Added new tests: `test_mode1_without_normals_returns_error`, `test_invalid_render_mode_returns_error`, `test_get_template_returns_error_for_invalid_mode`
  - All 17 shader_gen tests passing

- **Implement multiplayer player model (Phase 5)**
  - Added `PlayerSessionConfig` struct for configuring local vs remote players
  - Max 4 players total with flexible local/remote assignment via bitmask
  - Constructors: `all_local(n)`, `one_local(n)`, `with_local_players(n, &[])`, `new(n, mask)`
  - Methods: `is_local_player()`, `local_player_count()`, `remote_player_count()`, `local_player_indices()`, `remote_player_indices()`
  - Added `configure_session(player_count, local_player_mask)` to `GameInstance`
  - Updated `RollbackSession` to store and expose `PlayerSessionConfig`
  - Added `new_local_with_config()`, `new_sync_test_with_config()`, `new_p2p_with_config()` constructors
  - Added `player_config()` accessor on `RollbackSession`
  - 24 new tests for PlayerSessionConfig and integration with RollbackSession

- **Create developer guide (Phase 7)**
  - Getting started tutorial: Step-by-step first game walkthrough
  - Best practices for rollback-safe code: Determinism checklist, RNG usage, state management
  - Asset pipeline recommendations: Embedding assets, image conversion, texture guidelines
  - Debugging tips: log() usage, F3 overlay, common issues table, WASM size optimization

- **Create `platformer` example (Phase 6)**
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

- **Create `billboard` example (Phase 6)**
  - Demonstrates billboard drawing:
    - `draw_billboard()` with different modes (1-4)
    - Side-by-side comparison of all 4 modes (spherical, cylindrical Y/X/Z)
    - Particle system with spherical billboards (always face camera)
    - Tree/foliage sprites with cylindrical Y (stay upright)
    - Ground markers to show depth
    - Interactive camera rotation via analog stick
    - UI overlay with controls and mode labels
  - ~450 lines of Rust code

- **Update documentation (Phase 7)**
  - Fixed `delete` return value in ffi.md (was "1 on success", now "0 on success, 1 if invalid slot")
  - Fixed WASM import module name in emberware-z.md example (was "emberware", now "env")
  - Added troubleshooting section with 7 common issues and solutions
  - Added performance tips section covering vertex formats, batching, render modes, textures, and CPU budget

- **Create `skinned-mesh` example (Phase 6)**
  - Demonstrates GPU skinning:
    - Load skinned mesh with FORMAT_NORMAL | FORMAT_SKINNED (format 12)
    - 3-bone arm hierarchy with smooth weight blending
    - CPU-side bone animation (sine wave for wave-like bending)
    - `set_bones()` to upload bone matrices each frame
    - Shows workflow: CPU animation → GPU skinning
    - Interactive controls: L-stick rotate view, A pause, D-pad speed
    - Generates cylindrical arm mesh procedurally with vertex weights
  - ~520 lines of Rust code

- **Create `lighting` example (Phase 6)**
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

- **Create `cube` example (Phase 6)**
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

- **Create `textured-quad` example (Phase 6)**
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

- **[STABILITY] Add tests for GPU skinning** (`emberware-z/src/graphics.rs`, `emberware-z/src/ffi.rs`, `core/src/wasm.rs`)
  - Added 9 tests to `core/src/wasm.rs`: RenderState bone matrix storage, clearing, max capacity, weighted blend simulation, bone hierarchy simulation
  - Added 13 tests to `emberware-z/src/graphics.rs`: All 8 skinned vertex format stride calculations, VertexFormatInfo skinned flags, skinned format isolation, command buffer skinned vertices
  - Added 15 tests to `emberware-z/src/ffi.rs`: Skinned format flag values, vertex stride calculations, MAX_BONES constant, RenderState bone matrix operations, bone matrix transforms, column-major layout verification, bone weight sum convention
  - Total: 37 new GPU skinning tests (270 tests total, up from 233)

- **[STABILITY] Add tests for input system** (`emberware-z/src/input.rs`)
  - Added 22 new tests (32 total input tests, up from 10):
  - **Player slot assignment tests**: Sequential assignment, all full, gaps, disconnect freeing slots
  - **Deadzone edge cases**: Negative values, max values, zero deadzone, boundary conditions
  - **Get player input tests**: Valid range, out of range bounds checking
  - **Keyboard input tests**: D-pad, face buttons, start/select, analog is zero, custom mapping
  - **InputConfig tests**: Default values, partial deserialization
  - **KeyCode roundtrip**: Comprehensive test of all supported key types

- **[STABILITY] Implement settings web link** (`emberware-z/src/app.rs:269`)
  - Added `open` crate dependency for cross-platform browser opening.
  - `UiAction::OpenBrowser` now opens https://emberware.io in the default browser.
  - Error handling with tracing for failed browser open.

- **[STABILITY] Add keyboard mapping serialization** (`emberware-z/src/input.rs:40`)
  - Implemented string-based KeyCode serialization (`keycode_to_string`, `string_to_keycode`).
  - `KeyboardMapping` now derives `Serialize`/`Deserialize` with serde attributes.
  - Supports 80+ key names (letters, numbers, arrows, function keys, modifiers, punctuation, numpad).
  - Human-readable TOML config: `dpad_up = "ArrowUp"` instead of internal enum values.
  - 10 new tests for serialization roundtrip, custom key parsing, and deadzone application.

- **[STABILITY] Fix input not passed to game during rollback** (`core/src/runtime.rs:178`)
  - Added `to_input_state()` method to `ConsoleInput` trait for converting console-specific inputs to common `InputState`.
  - Implemented `to_input_state()` for `ZInput` in emberware-z.
  - Updated runtime to pass GGRS confirmed inputs to game before calling `update()` during rollback.
  - Enables deterministic rollback netcode: games now receive correct inputs during replay.

- **[STABILITY] Replace expect() calls in graphics initialization** (`emberware-z/src/graphics.rs`)
  - Changed `create_fallback_textures()` to return `Result<()>` and propagate errors.
  - Uses `.context()` instead of `.expect()` for proper error context.
  - Errors now bubble up through `ZGraphics::new()` for graceful handling.

- **[STABILITY] Fix WasmEngine::Default panic** (`core/src/wasm.rs`)
  - Removed `Default` impl which violated Rust conventions (panicked on failure).
  - Added documentation explaining why `Default` is not implemented.
  - Callers should use `WasmEngine::new()` which returns `Result<Self>`.

- **[STABILITY] Add comments to dead_code suppressions** (`core/src/runtime.rs`, `core/src/wasm.rs`)
  - Added doc comments explaining `console` field in `Runtime` is kept for future use.
  - Added doc comments explaining `instance` field in `GameInstance` is required for lifetime.

- **[STABILITY] Fix potential panic in transform_set()** (`emberware-z/src/ffi.rs:422`)
  - Changed `.try_into().expect()` to use `let Ok(matrix) = ... else { warn; return; }` pattern.
  - Now returns early with warning instead of panicking.

- **[STABILITY] Fix gamepad initialization double-panic** (`emberware-z/src/input.rs:132-137`)
  - Changed `gilrs` field from `Gilrs` to `Option<Gilrs>`.
  - Gracefully disables gamepad support if Gilrs::new() fails.
  - Input polling handles None case by skipping gamepad events.

- **[STABILITY] Remove outdated TODO comment** (`emberware-z/src/console.rs:225-230`)
  - Removed misleading comment that listed FFI functions as TODO when they were already implemented.
  - Updated comment to accurately describe what register_ffi does.

- **Create `triangle` example (Phase 6)**
  - Minimal no_std WASM game demonstrating:
    - `draw_triangles()` (immediate mode 3D)
    - Vertex format: POS_COLOR (format 2)
    - Transform stack: `transform_rotate()` for spinning
    - Game lifecycle: init/update/render
  - ~50 lines of Rust code
  - Also fixed hello-world example to use correct FFI module ("env" instead of "emberware")
  - Updated workspace Cargo.toml to exclude examples from workspace

- **Create graphics tests (Phase 7)**
  - **Shader compilation tests (all 40 shaders)**: 15 tests in shader_gen.rs
    - `test_compile_all_40_shaders` - validates all mode/format combinations
    - `test_compile_mode0_all_formats` - all 16 formats for Mode 0 (Unlit)
    - `test_compile_mode1_matcap` - 8 formats with NORMAL for Mode 1
    - `test_compile_mode2_pbr` - 8 formats with NORMAL for Mode 2
    - `test_compile_mode3_hybrid` - 8 formats with NORMAL for Mode 3
    - `test_compile_skinned_variants` - all skinned permutations
    - `test_shader_has_vertex_entry` / `test_shader_has_fragment_entry` - entry points
    - `test_no_unreplaced_placeholders` - template validation
  - **Vertex format stride validation**: 11 tests for all 16 formats
    - `test_vertex_stride_*` - validates stride calculations for each format
    - `test_all_16_vertex_formats` - comprehensive format coverage
    - `test_vertex_format_info_*` - name and flag validation
  - **Texture loading and binding tests**: 4 new tests
    - `test_texture_slot_binding` - slot assignment
    - `test_texture_slot_rebinding` - rebinding and unbinding
    - `test_texture_slots_all_bound` - all 4 slots bound
    - `test_draw_command_captures_texture_slots` - state capture
  - **Render state switching tests**: 9 new tests
    - `test_render_state_depth_test_toggle` - depth test on/off
    - `test_render_state_cull_mode_switching` - cull mode changes
    - `test_render_state_blend_mode_switching` - blend mode changes
    - `test_render_state_color_changes` - color and vec4 conversion
    - `test_render_state_texture_filter_switching` - filter mode changes
    - `test_draw_commands_capture_render_state` - full state capture
    - `test_render_state_equality` / `test_render_state_clone` - Copy/PartialEq
  - Total graphics tests: 42 (was 29, added 13 new tests)
  - All 206 tests passing (132 core + 74 emberware-z)

- **Update documentation (Phase 7)**
  - Fixed save function return value documentation in ffi.md (was "1 on success", now "0 on success, 1 invalid slot, 2 data too large")
  - Fixed light_set signature in emberware-z.md (removed non-existent light_type parameter)
  - Fixed Mode 3 (Hybrid) docs to use actual light_set/light_color/light_intensity functions instead of non-existent light_direction/ambient_color
  - All tests passing (62 tests)

- **Create integration tests (Phase 7)**
  - Added integration.rs module with 24 comprehensive tests
  - **Game lifecycle tests**: Full init/update/render flow, minimal games, state persistence
  - **Rollback simulation tests**: Save/load state, RollbackStateManager, checksum verification, multiple save points
  - **Multi-player input tests**: Input state rotation, 4-player handling, local player mask, console input mapping
  - **Resource limit tests**: Console specs, texture/mesh allocation tracking, save slot limits, transform stack limits, rollback state size, player count limits, draw command buffer

- **Create unit tests for core framework (Phase 7)**
  - Added wat dev-dependency for test WASM module parsing
  - **wasm.rs**: 48 tests for WASM loading, GameState, CameraState, InputState, DrawCommand, transforms
    - `test_wasm_engine_*` - Engine creation and module loading
    - `test_game_state_*` - GameState initialization and defaults
    - `test_camera_state_*` - View/projection matrix calculations
    - `test_input_state_*` - Input serialization roundtrips
    - `test_render_state_*` - Render state defaults
    - `test_draw_command_*` - All DrawCommand variants
    - `test_game_instance_*` - WASM game lifecycle (init, update, render)
    - `test_transform_*` - Matrix math verification
  - **ffi.rs**: 17 tests for FFI registration and bindings
    - `test_register_common_ffi` - FFI function registration
    - `test_ffi_with_wasm_module` - FFI imports work from WASM
    - `test_ffi_random_from_wasm` - RNG callable from WASM
    - `test_ffi_quit_from_wasm` - Quit callable from WASM
    - `test_rng_*` - RNG determinism and edge cases
    - `test_save_data_*` - Save slot management
  - **runtime.rs**: 21 tests for Runtime and Console trait
    - `test_runtime_*` - Runtime creation, game loading, session management
    - `test_console_*` - Console trait implementation tests
  - Total: 108 tests (from 26 baseline) - all passing

- **Create shader compilation tests (all 40 shaders) (Phase 7)**
  - Added naga as dev-dependency for WGSL parsing and validation
  - `compile_and_validate_shader()` helper validates each shader permutation
  - `test_compile_all_40_shaders` - comprehensive test of all mode/format combinations
  - `test_compile_mode0_all_formats` - all 16 formats for Mode 0 (Unlit)
  - `test_compile_mode1_matcap` - 8 formats with NORMAL for Mode 1 (Matcap)
  - `test_compile_mode2_pbr` - 8 formats with NORMAL for Mode 2 (PBR)
  - `test_compile_mode3_hybrid` - 8 formats with NORMAL for Mode 3 (Hybrid)
  - `test_compile_skinned_variants` - all skinned format permutations
  - `test_shader_has_vertex_entry` - verify `fn vs()` entry point exists
  - `test_shader_has_fragment_entry` - verify `fn fs()` entry point exists
  - `test_no_unreplaced_placeholders` - verify all template placeholders replaced
  - Fixed shader bug: Modes 2/3 used wrong variable name (`color` vs `albedo`)
  - Added `FS_ALBEDO_COLOR` and `FS_ALBEDO_UV` snippets for PBR/Hybrid modes

- **Implement application state machine (Phase 4)**
  - States: Library → Downloading → Playing → back to Library
  - Error handling transitions via RuntimeError enum
  - Handle errors: WasmPanic, NetworkDisconnect, OutOfMemory, Other
  - Error display in Library UI with dismiss button
  - handle_runtime_error() transitions back to library with error message

- **Implement debug overlay (console-wide) (Phase 4)**
  - FPS counter (update rate)
  - Frame time with millisecond precision
  - Frame time graph (120 sample history, color-coded bars)
  - VRAM usage (current/limit with progress bar)
  - Network stats (ping, rollback frames, frame advantage) - ready for P2P sessions
  - Toggle via F3 hotkey
  - Resizable debug window

- **Handle GGRS events (Phase 2)**
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

- **Implement egui integration for library UI (Phase 4)**
  - egui-wgpu renderer setup with wgpu 23 (for egui 0.30 compatibility)
  - egui-winit state integration for event handling
  - Library screen with game list and selection
  - Game actions: play, view info, delete
  - Settings screen placeholder (not yet implemented)
  - Download progress UI placeholder
  - Debug overlay with F3 toggle showing FPS, frame time, mode
  - Application state machine (Library → Playing → Settings)
  - Error handling integration

- **Implement keyboard/gamepad input**
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

- **Implement winit window management (Phase 4)**
  - Window creation with configurable resolution (1920x1080 default)
  - Fullscreen toggle via F11 key
  - Event loop integration using winit 0.30 ApplicationHandler trait
  - Window resize handling with graphics backend resize calls
  - DPI/scale factor change handling
  - Window close handling with graceful shutdown
  - Escape key returns to library from game mode
  - Fullscreen state persistence in config.toml
  - Integration with ZGraphics backend initialization

- **Implement Built-in Font (Phase 3.17)**
  - Created `font.rs` module with 8x8 monospace bitmap font
  - ASCII 32-126 (95 printable characters) in 128x48 texture atlas
  - `get_glyph_uv(char_code)` returns normalized UV coordinates
  - `generate_font_atlas()` creates RGBA8 texture data at runtime
  - Font texture loaded during ZGraphics initialization
  - `font_texture()` and `get_font_texture_view()` accessors added
  - `generate_text_quads(text, x, y, size, color)` generates POS_UV_COLOR vertices
  - Screen-space text rendering with variable size via scaling
  - Left-aligned, single-line (no word wrap in v1)

- **Implement GPU Skinning (Phase 3.16)**
  - Bone storage buffer: 256 bones × 4×4 matrices = 16KB
  - `set_bones(matrices_ptr, count)` FFI function implemented and registered
  - Bone matrices stored in RenderState (Vec<Mat4>)
  - Graphics trait extended with `set_bones()` method for GPU upload
  - ZGraphics implements bone matrix upload via wgpu queue.write_buffer()
  - Skinned vertex shader code in shader templates (VIN_SKINNED, VS_SKINNED)
  - FORMAT_SKINNED (8) adds 20 bytes per vertex (4 u8 indices + 4 f32 weights)
  - Vertex attribute order: pos → uv → color → normal → bone_indices → bone_weights
  - All 16 vertex format permutations supported (8 base + 8 skinned variants)

- **Implement Mode 2 (PBR) and Mode 3 (Hybrid) lighting functions (Phase 3.15)**
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
  - No light uniform buffer needed yet (will be added when graphics backend processes lights)

- **Implement Mode 1 (Matcap) functions (Phase 3.15)**
  - `matcap_set(slot, texture)` — bind matcap to slot 1-3
  - Validate slot is 1-3, warn otherwise
  - FFI function registered and implemented

- **Implement material functions (Phase 3.15)**
  - `material_mre(texture)` — bind MRE texture (R=Metallic, G=Roughness, B=Emissive)
  - `material_albedo(texture)` — bind albedo texture (alternative to slot 0)
  - `material_metallic(value)` — set metallic (0.0-1.0, default 0.0)
  - `material_roughness(value)` — set roughness (0.0-1.0, default 0.5)
  - `material_emissive(value)` — set emissive intensity (default 0.0)
  - Material properties added to RenderState struct
  - All FFI functions registered and implemented with validation

- **Implement Procedural Sky System (Phase 3.14)**
  - Created `SkyUniforms` struct with horizon_color, zenith_color, sun_direction, sun_color, sun_sharpness
  - Implemented sky uniform buffer with GPU upload
  - Added `set_sky()` FFI function with 13 f32 parameters
  - Sun direction automatically normalized
  - `sample_sky()` shader function implemented in all shader templates
  - Gradient interpolation: `mix(horizon, zenith, direction.y * 0.5 + 0.5)`
  - Sun calculation: `sun_color * pow(max(0, dot(direction, sun_direction)), sharpness)`
  - Integrated with all render modes (0-3) for ambient lighting and reflections
  - Default: all zeros (black sky, no sun, no lighting until configured via set_sky())

- **Implement Shader Generation System (Phase 3.13)**
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

- **Implement 2D Drawing FFI (Phase 3.11)**
  - `draw_sprite(x, y, w, h, color)` — draw bound texture in screen space
  - `draw_sprite_region(x, y, w, h, src_x, src_y, src_w, src_h, color)` — sprite sheet support
  - `draw_sprite_ex(x, y, w, h, src_x, src_y, src_w, src_h, origin_x, origin_y, angle_deg, color)` — full control with rotation
  - `draw_rect(x, y, w, h, color)` — solid color rectangle
  - `draw_text(ptr, len, x, y, size, color)` — UTF-8 text rendering
  - DrawSprite, DrawRect, and DrawText commands added to DrawCommand enum
  - Screen-space coordinates (0,0 = top-left)
  - UTF-8 validation and proper string handling

- **Implement billboard drawing (Phase 3.10)**
  - `draw_billboard(w, h, mode, color)` — draw billboard at current transform
  - `draw_billboard_region(w, h, src_x, src_y, src_w, src_h, mode, color)` — with UV region
  - Billboard modes: 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
  - DrawBillboard command added to DrawCommand enum
  - FFI functions registered and implemented with validation

- **Implement Mesh FFI functions (Phase 3.7)**
  - `load_mesh(data_ptr, vertex_count, format) -> u32` — non-indexed mesh
  - `load_mesh_indexed(data_ptr, vertex_count, index_ptr, index_count, format) -> u32`
  - `draw_mesh(handle)` — draw retained mesh with current transform/state
  - Vertex format validation (0-15)
  - Stride calculation and memory bounds checking
  - PendingMesh struct for graphics backend integration
  - DrawCommand enum for deferred rendering

- **Implement Immediate Mode 3D FFI (Phase 3.8)**
  - `draw_triangles(data_ptr, vertex_count, format)` — non-indexed immediate draw
  - `draw_triangles_indexed(data_ptr, vertex_count, index_ptr, index_count, format)`
  - Vertex format validation and stride calculation
  - Index bounds checking
  - Draw commands buffered with current transform and render state

- **Implement Input FFI functions (Phase 3.5)**
  - Individual button queries: `button_held`, `button_pressed`, `button_released`
  - Bulk button queries: `buttons_held`, `buttons_pressed`, `buttons_released`
  - Analog stick queries: `left_stick_x`, `left_stick_y`, `right_stick_x`, `right_stick_y`
  - Bulk stick queries: `left_stick`, `right_stick`
  - Trigger queries: `trigger_left`, `trigger_right`
  - Full player validation (0-3), button validation (0-13)

- **Implement Texture FFI functions (Phase 3.6)**
  - `load_texture(width, height, pixels_ptr) -> u32` — creates texture from RGBA data
  - `texture_bind(handle)` — bind to slot 0
  - `texture_bind_slot(handle, slot)` — bind to specific slot (0-3)
  - PendingTexture struct for graphics backend integration
  - WASM memory bounds validation

- **Implement Render State FFI functions (Phase 3.12)**
  - `set_color(color)` — uniform tint color (0xRRGGBBAA)
  - `depth_test(enabled)` — enable/disable depth testing
  - `cull_mode(mode)` — 0=none, 1=back, 2=front
  - `blend_mode(mode)` — 0=none, 1=alpha, 2=additive, 3=multiply
  - `texture_filter(filter)` — 0=nearest, 1=linear
  - Input validation with warnings for invalid values

- **Implement camera functions (Phase 3.4)**
  - `camera_set(x, y, z, target_x, target_y, target_z)` — look-at camera
  - `camera_fov(fov_degrees: f32)` — field of view (default 60°)
  - View matrix calculation
  - Projection matrix calculation
  - CameraState struct with view_matrix(), projection_matrix(), view_projection_matrix()

- **Implement transform stack functions (Phase 3.9)**
  - `transform_identity()` — reset to identity matrix
  - `transform_translate(x, y, z)` — translate
  - `transform_rotate(angle_deg, x, y, z)` — rotate around axis
  - `transform_scale(x, y, z)` — scale
  - `transform_push()` — push current matrix to stack (returns 1/0)
  - `transform_pop()` — pop matrix from stack (returns 1/0)
  - `transform_set(matrix_ptr)` — set from 16 floats (column-major)
  - Stack depth: 16 matrices
  - Matrix math using glam


- **Implement configuration functions (Phase 3.3)**
  - `set_resolution(res: u32)` — 0=360p, 1=540p, 2=720p, 3=1080p
  - `set_tick_rate(fps: u32)` — 24, 30, 60, 120
  - `set_clear_color(color: u32)` — 0xRRGGBBAA background color
  - `render_mode(mode: u32)` — 0-3 (Unlit, Matcap, PBR, Hybrid)
  - Enforce init-only: error/warning if called outside `init()`

- **Implement vertex buffer architecture**
  - One vertex buffer per stride (format determines buffer)
  - `GrowableBuffer` struct for auto-growing GPU buffers
  - 8 base vertex formats (POS, POS_UV, POS_COLOR, etc.)
  - 8 skinned variants (each base format + skinning data)
  - Total: 16 vertex format pipelines per render mode

- **Implement command buffer pattern**
  - Immediate-mode draws buffered on CPU side
  - Single flush to GPU per frame (minimize draw calls)
  - Draw command batching by pipeline/texture state
  - Retained mesh handles separate from immediate draws

- **Implement wgpu device initialization**
  - `ZGraphics` struct implementing `Graphics` trait
  - wgpu `Instance`, `Adapter`, `Device`, `Queue` setup
  - Surface configuration for window
  - Resize handling with surface reconfiguration

- **Implement texture management**
  - `TextureHandle` allocation and tracking
  - `load_texture(width, height, pixels)` — create RGBA8 texture
  - VRAM budget tracking (8MB limit)
  - Fallback textures: 8×8 magenta/black checkerboard, 1×1 white
  - Sampler creation (nearest, linear filters)

- **Implement render state management**
  - Current color (uniform tint)
  - Depth test enable/disable
  - Cull mode (none, back, front)
  - Blend mode (none, alpha, additive, multiply)
  - Texture filter (nearest, linear)
  - Currently bound textures (slots 0-3)

- **Create Emberware Z `Console` implementation**
  - Implement `Console` trait for PS1/N64 aesthetic
  - Define Z-specific specs:
    - Resolution: 360p, 540p (default), 720p, 1080p
    - Tick rate: 24, 30, 60 (default), 120 fps
    - RAM: 16MB, VRAM: 8MB, ROM: 32MB max
    - Color depth: RGBA8
    - CPU budget: 4ms per tick at 60fps
  - `ZInput` struct (buttons, dual sticks, triggers)

- **Handle GGRS events**
  - `GGRSRequest::SaveGameState` → serialize WASM state
  - `GGRSRequest::LoadGameState` → deserialize WASM state
  - `GGRSRequest::AdvanceFrame` → run `update()` with confirmed inputs
  - Connection quality events (desync detection, frame advantage warnings)
  - Audio muting during rollback replay

- **Integrate GGRS session into runtime**
  - Local session (single player or local multiplayer, no rollback)
  - P2P session with matchbox_socket (WebRTC)
  - `advance_frame()` with GGRS requests handling
  - Synchronization test mode for local debugging
  - TODO [needs clarification]: Spectator session support

- **Define GGRS config and input types**
  - `GGRSConfig` implementing `ggrs::Config` trait
  - Generic input type parameterized by console's `ConsoleInput`
  - Input serialization for network (bytemuck Pod)
  - Input delay and frame advantage settings

- **Implement rollback state management**
  - `save_game_state()` — call WASM `save_state`, store snapshot with checksum
  - `load_game_state()` — call WASM `load_state`, restore snapshot
  - State buffer pool for efficient rollback (avoid allocations in hot path)
  - State compression (optional, for network sync)

- **Create `core` crate with workspace configuration**
  - Add `core/Cargo.toml` with wasmtime, ggrs, matchbox_socket, winit
  - Update root `Cargo.toml` workspace members
  - Define core module structure: `lib.rs`, `console.rs`, `runtime.rs`, `wasm.rs`, `ffi.rs`, `rollback.rs`

- **Create repository structure**
  - Root Cargo.toml workspace
  - README.md with project overview
  - CLAUDE.md with development instructions
  - .gitignore and LICENSE

- **Create `shared` crate**
  - API types: Game, Author, User, Auth responses
  - Request/response types for platform API
  - LocalGameManifest for downloaded games
  - Error types and codes

- **Create `emberware-z` crate skeleton**
  - Cargo.toml with dependencies
  - main.rs entry point
  - app.rs application state
  - config.rs configuration management
  - deep_link.rs URL parsing
  - download.rs ROM fetching
  - library.rs local game management
  - ui.rs egui library interface
  - runtime/mod.rs module declaration (stubs)

- **Create FFI documentation**
  - docs/ffi.md with complete API reference
  - All function signatures and examples
  - Console specs and lifecycle documentation

- **Create hello-world example**
  - Minimal no_std WASM game
  - Demonstrates init/update/render lifecycle
  - Basic input and rendering

- **Initialize git repository and push to GitHub**

- **Define `Console` trait and associated types**
  - `Console` trait with specs, FFI registration, graphics/audio factory methods
  - `Graphics` trait for rendering backend abstraction
  - `Audio` trait for audio backend abstraction
  - `ConsoleInput` trait with bytemuck requirements for GGRS serialization
  - `ConsoleSpecs` struct (resolutions, tick rates, RAM/VRAM limits, ROM size)

- **Implement `GameState` for WASM instance**
  - Wasmtime `Store` data structure containing all per-game state
  - FFI context: graphics command buffer, audio commands, RNG state
  - Input state for all 4 players
  - Transform stack (16 matrices deep)
  - Current render state (color, blend mode, depth test, cull mode, filter)
  - Save data slots (8 slots × 64KB max each)

- **Implement WASM runtime wrapper**
  - `WasmEngine` — shared wasmtime `Engine` (one per app)
  - `GameInstance` — loaded game with `Module`, `Instance`, `Store`
  - Export function bindings: `init()`, `update()`, `render()`
  - Export function bindings: `save_state(ptr, max_len) -> len`, `load_state(ptr, len)`
  - Memory access helpers for FFI string/buffer passing
  - WASM memory bounds checking and validation

- **Implement common FFI host functions**
  - System functions: `delta_time`, `elapsed_time`, `tick_count`, `log`, `quit`
  - Rollback functions: `random` (deterministic PCG)
  - Save data functions: `save`, `load`, `delete`
  - Session functions: `player_count`, `local_player_mask`

- **Implement game loop orchestration**
  - Fixed timestep update loop (configurable tick rate: 24, 30, 60, 120)
  - Variable render rate with interpolation support (uncapped frame rate)
  - Frame timing and delta time calculation
  - Update/render separation (render skipped during rollback replay)
  - CPU budget enforcement (4ms per tick at 60fps, warn on exceed)

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
  - Demonstrates Classic-specific 2D features
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