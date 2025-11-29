# Emberware Development Tasks

## Needs Clarification

These items are marked TODO throughout the document and need decisions before implementation:

- **Audio system** — Architecture, formats, sample rates, channel count (shelved for now)
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

### Phase 2: GGRS Rollback Integration

- **Handle GGRS events**
  - `GGRSRequest::SaveGameState` → serialize WASM state
  - `GGRSRequest::LoadGameState` → deserialize WASM state
  - `GGRSRequest::AdvanceFrame` → run `update()` with confirmed inputs
  - Connection quality events (desync detection, frame advantage warnings)
  - Audio muting during rollback replay

### Phase 3: Emberware Z Implementation

#### 3.1 Console Setup

(Moved to In Progress)

#### 3.2 Graphics Backend (wgpu)

(Moved to Done)

#### 3.3 Configuration FFI (init-only)

(Moved to In Progress)

#### 3.4 Camera FFI

(Moved to In Progress)

#### 3.5 Input FFI

(Moved to In Progress)

#### 3.6 Texture FFI

(Moved to In Progress)

#### 3.7 Mesh FFI (Retained Mode)

(Moved to Done)

#### 3.8 Immediate Mode 3D FFI

(Moved to Done)

#### 3.9 Transform Stack FFI

(Moved to In Progress)

#### 3.11 2D Drawing FFI (Screen Space)

(Moved to In Progress)

#### 3.12 Render State FFI

(Moved to In Progress)


#### 3.15 Mode-Specific Lighting FFI

(Moved to In Progress)

#### 3.16 GPU Skinning

- **Implement bone uniform buffer**
  - Support up to 256 bones (256 × 4×4 matrices = 16KB)
  - `set_bones(matrices_ptr, count)` — upload bone transforms
  - Bone matrices in column-major order (16 floats each)

- **Implement skinned vertex shader**
  - Read bone indices (4 × u8) and weights (4 × f32) from vertex
  - Compute skinned position: `Σ(weight[i] * bone_matrix[bone_index[i]] * pos)`
  - Compute skinned normal: `Σ(weight[i] * inverse_transpose(bone_matrix[i]) * normal)`
  - Apply before model-view-projection transform

- **Update vertex formats for skinning**
  - `FORMAT_SKINNED` (8) adds 20 bytes per vertex
  - Bone indices: 4 bytes (4 × u8)
  - Bone weights: 16 bytes (4 × f32)
  - Attribute order: pos → uv → color → normal → bone_indices → bone_weights

#### 3.17 Built-in Font

- **Create embedded bitmap font**
  - ASCII + extended Latin characters minimum
  - Full UTF-8 support for CJK and other scripts (or subset)
  - Single texture atlas with glyph metrics
  - Embed via `include_bytes!()` in binary

- **Implement text rendering**
  - Parse UTF-8 string, look up glyph metrics
  - Generate quads for each character
  - Support variable size via scaling
  - Left-aligned, single line (no word wrap in v1)
  - TODO [needs clarification]: Custom font loading

### Phase 4: Application Shell

- **Implement winit window management**
  - Window creation with configurable resolution
  - Fullscreen toggle (F11 or Alt+Enter)
  - Event loop integration
  - Window resize handling
  - DPI/scale factor handling

- **Implement egui integration for library UI**
  - egui-wgpu renderer setup
  - Library screen (game list with thumbnails)
  - Game actions: play, view info, delete
  - Settings screen (video, audio, controls)
  - Download progress UI with cancel option

- **Implement application state machine**
  - States: Library → Downloading → Playing → back to Library
  - Error handling:
    - CPU exceeded → log warning, skip frame
    - OOM → crash with error message
    - WASM panic → return to library with error
    - Network disconnect → return to library

- **Implement keyboard/gamepad input**
  - Keyboard mapping to virtual controller (configurable)
  - Gamepad support via gilrs
  - Multiple local players (keyboard + gamepads)
  - Input config persistence in config.toml
  - Deadzone and sensitivity settings

- **Implement debug overlay (console-wide)**
  - FPS counter (update and render rates)
  - Memory usage (RAM/VRAM current and limit)
  - Network stats (ping, rollback frames, frame advantage)
  - Toggle via hotkey (F3 or similar)
  - Optional: frame time graph

### Phase 5: Networking & Polish

- **Implement multiplayer player model**
  - Max 4 players total (any mix of local + remote)
  - Examples: 4 local, 1 local + 3 remote, 2 local + 2 remote
  - Each local player maps to a physical input device
  - GGRS handles all players uniformly
  - Player slot assignment

- **Implement matchbox signaling connection**
  - Connect to matchbox signaling server
  - WebRTC peer connection establishment
  - ICE candidate exchange
  - Connection timeout handling
  - TODO [needs clarification]: Matchmaking handled by platform service

- **Implement netplay session management**
  - Host/join game flow via platform deep links
  - Connection quality display (ping bars)
  - Disconnect handling (return to library)
  - Session cleanup on exit

- **Implement local network testing**
  - Multiple instances on same machine via localhost
  - Connect via `127.0.0.1:port` for local testing
  - Debug mode: disable matchbox, use direct connections

- **Add input delay configuration**
  - Local input delay setting (0-10 frames)
  - Frame delay vs rollback tradeoff UI
  - Persist per-game or globally

- **Performance optimization**
  - State serialization optimization (avoid allocations)
  - Render batching (minimize state changes)
  - Memory pooling for rollback buffers
  - Profile and optimize hot paths

### Phase 6: Emberware Z Examples

- **Create `triangle` example**
  - Minimal no_std WASM game demonstrating:
    - `draw_triangles()` (immediate mode 3D)
    - Vertex format: POS_COLOR (format 2)
    - Transform stack: `transform_rotate()` for spinning
    - Game lifecycle: init/update/render
  - ~50 lines of Rust code

- **Create `textured-quad` example**
  - Demonstrates texture loading and 2D drawing:
    - `include_bytes!()` for embedded PNG
    - PNG decoding (minipng or similar)
    - `load_texture()` and `texture_bind()`
    - `draw_sprite()` for 2D rendering
    - `set_color()` for tinting

- **Create `cube` example**
  - Demonstrates retained mode 3D:
    - `load_mesh_indexed()` in init()
    - `draw_mesh()` in render()
    - Vertex format: POS_UV_NORMAL (format 5)
    - Camera setup: `camera_set()`, `camera_fov()`
    - Interactive rotation via analog stick
    - Mode 0 with normals (simple Lambert)

- **Create `lighting` example**
  - Demonstrates all render modes (0-3):
    - Toggle between modes with button press
    - Mode 0: Unlit/Lambert
    - Mode 1: Matcap (load 3 matcap textures)
    - Mode 2: PBR with 4 lights
    - Mode 3: Hybrid (1 light + matcap ambient)
    - Material properties: `material_metallic()`, `material_roughness()`, `material_emissive()`
    - Dynamic light positioning
    - `set_sky()` for procedural sky

- **Create `skinned-mesh` example**
  - Demonstrates GPU skinning:
    - Load skinned mesh with FORMAT_SKINNED | FORMAT_UV | FORMAT_NORMAL
    - Simple bone hierarchy (e.g., arm with 3 bones)
    - CPU-side bone animation (sine wave for demo)
    - `set_bones()` to upload bone matrices each frame
    - Shows workflow: CPU animation → GPU skinning

- **Create `billboard` example**
  - Demonstrates billboard drawing:
    - `draw_billboard()` with different modes (1-4)
    - Sprite-based character (cylindrical Y)
    - Particle system (spherical)
    - Tree/foliage sprites (cylindrical Y)
    - Comparison of billboard modes side-by-side

- **Create `platformer` example**
  - Full mini-game demonstrating multiple Z features:
    - 2D gameplay using 3D renderer
    - Textured sprites for player/enemies
    - Billboarded sprites in 3D space
    - Simple physics and collision (AABB)
    - Multiple players with analog stick input
    - 2D UI overlay with `draw_text()`, `draw_rect()`
    - Sky background with `set_sky()`
    - Demonstrates rollback-safe game state

### Phase 7: Testing & Documentation

- **Create unit tests for core framework**
  - WASM loading and execution tests
  - FFI function binding tests
  - Input serialization tests (bytemuck roundtrip)
  - State save/load tests

- **Create integration tests**
  - Full game lifecycle test (init → update → render)
  - Rollback simulation test (save → modify → load → verify)
  - Multi-player input synchronization test
  - Resource limit enforcement test (RAM/VRAM)

- **Create graphics tests**
  - Shader compilation tests (all 40 shaders)
  - Vertex format stride validation
  - Texture loading and binding tests
  - Render state switching tests

- **Update documentation**
  - Ensure docs/ffi.md matches implementation
  - Ensure docs/emberware-z.md matches implementation
  - Add troubleshooting section
  - Add performance tips section
  - API versioning documentation

- **Create developer guide**
  - Getting started tutorial
  - Best practices for rollback-safe code
  - Asset pipeline recommendations
  - Debugging tips

---
## In Progress

## Done

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