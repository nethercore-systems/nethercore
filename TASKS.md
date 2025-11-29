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
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
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

### Phase 1: Core Framework Foundation

- [ ] **Create `core` crate with workspace configuration**
  - Add `core/Cargo.toml` with wasmtime, ggrs, matchbox_socket, winit
  - Update root `Cargo.toml` workspace members
  - Define core module structure

- [ ] **Define `Console` trait and associated types**
  - `Console` trait with specs, FFI registration, graphics/audio factory methods
  - `Graphics` trait for rendering backend abstraction
  - `Audio` trait for audio backend abstraction
  - `ConsoleSpecs` struct (resolutions, tick rates, RAM/VRAM limits)

- [ ] **Implement `GameState` for WASM instance**
  - Wasmtime `Store` data structure
  - Memory management (RAM limits)
  - FFI context (graphics commands buffer, audio commands, RNG state)

- [ ] **Implement WASM runtime wrapper**
  - `WasmEngine` — shared wasmtime `Engine`
  - `GameInstance` — loaded game with `Module`, `Instance`, `Store`
  - Export function bindings: `init()`, `update()`, `render()`
  - Export function bindings: `save_state()`, `load_state()` (for rollback)
  - Memory access helpers for FFI string/buffer passing

- [ ] **Implement common FFI host functions**
  - System: `delta_time`, `elapsed_time`, `tick_count`, `log`, `quit`
  - Rollback: `random` (deterministic seeded RNG)
  - Save data: `save`, `load`, `delete`
  - Session: `player_count`, `local_player_mask` (input functions are console-specific)

- [ ] **Implement game loop orchestration**
  - Fixed timestep update loop (tick rate)
  - Variable render rate with interpolation support
  - Frame timing and delta time calculation
  - Update/render separation (render skipped during rollback)

### Phase 2: GGRS Rollback Integration

- [ ] **Define GGRS config and input types**
  - `GGRSConfig` implementing `ggrs::Config`
  - `PlayerInput` struct (buttons, stick axes)
  - Input serialization for network

- [ ] **Implement rollback state management**
  - `save_game_state()` — call WASM `save_state`, store snapshot
  - `load_game_state()` — call WASM `load_state`, restore snapshot
  - State buffer pool for efficient rollback

- [ ] **Integrate GGRS session into runtime**
  - Local session (single player or local multiplayer, no rollback)
  - P2P session with matchbox_socket
  - `advance_frame()` with GGRS requests handling
  - TODO [needs clarification]: Spectator session support

- [ ] **Handle GGRS events**
  - `GGRSRequest::SaveGameState` → serialize WASM state
  - `GGRSRequest::LoadGameState` → deserialize WASM state
  - `GGRSRequest::AdvanceFrame` → run `update()` with confirmed inputs
  - Audio muting during rollback replay

### Phase 3: Emberware Z Implementation

- [ ] **Create Emberware Z `Console` implementation**
  - Implement `Console` trait for PS1/N64 aesthetic
  - Define Z-specific specs (540p default, 60fps, 16MB RAM, 8MB VRAM)
  - RGBA8 color output (easy to change later if needed)

- [ ] **Implement wgpu graphics backend**
  - `ZGraphics` implementing `Graphics` trait
  - wgpu device/queue/surface setup
  - Render pipeline for 3D (no PS1 effects for now, can add later)
  - Render pipeline for 2D sprites/UI
  - Command buffer pattern (game FFI queues commands, rendered at frame_end)
  - VRAM budget tracking (no texture count limit, just memory)
  - **Vertex buffer architecture:**
    - Single vertex buffer per stride (e.g., pos+uv+color = one buffer, pos+uv+color+normal = another)
    - Buffers grow dynamically as meshes are loaded during `init()`
    - Immediate-mode drawing (`draw_triangle`, `draw_sprite`, etc.) buffers on CPU side
    - CPU buffer flushed to GPU once per frame to minimize draw calls
    - No manual resource cleanup — all graphics resources freed on game shutdown

- [ ] **Implement Z-specific FFI functions**
  - Graphics: `camera_set`, `camera_fov`, `set_clear_color` (init-only)
  - Textures: `load_texture` (raw RGBA pixels), `texture_bind`, `texture_bind_slot`
  - Meshes: `load_mesh(data, count, format)`, `load_mesh_indexed(data, count, indices, index_count, format)`, `draw_mesh(handle)`
  - Immediate 3D: `draw_triangles(data, count, format)`, `draw_triangles_indexed(data, count, indices, index_count, format)`
  - 2D: `draw_sprite`, `draw_rect`, `draw_text` (built-in font, UTF-8)
  - Transform: `transform_identity/translate/rotate/scale/push/pop/set`
  - Render state: `set_color`, `depth_test`, `cull_mode`, `blend_mode`, `texture_filter`
  - Input: `button_held`, `button_pressed`, `button_released`, `left_stick_x/y`, `right_stick_x/y`, `trigger_left`, `trigger_right`
  - Vertex formats: 3-bit bitmask (`FORMAT_UV`, `FORMAT_COLOR`, `FORMAT_NORMAL`) producing 8 combinations
  - NO `*_free` functions — resources auto-cleaned on shutdown

- [ ] **Implement GPU skinning**
  - Add `FORMAT_SKINNED` flag (adds 20 bytes: 4×u8 bone indices + 4×f32 bone weights)
  - Implement `set_bones(matrices, count)` — upload bone transforms (max 256 bones)
  - Shader support for weighted bone transform in vertex shader
  - Works with both retained mode (`load_mesh` + `draw_mesh`) and immediate mode (`draw_triangles`)
  - CPU-side animation (keyframes, blend trees, IK) left to developers

- [ ] **Implement built-in font for draw_text**
  - Static embedded bitmap/SDF font
  - Full UTF-8 support for localization
  - TODO [needs clarification]: Custom font loading in the future

- [ ] **Audio system**
  - TODO [needs clarification]: Audio architecture, formats, sample rates, channel count, etc.
  - Shelved for initial implementation

### Phase 4: Application Shell

- [ ] **Implement winit window management**
  - Window creation with configurable resolution
  - Fullscreen toggle
  - Event loop integration

- [ ] **Implement egui integration for library UI**
  - egui-wgpu renderer setup
  - Library screen (game list, play/delete)
  - Settings screen (video, controls)
  - Download progress UI

- [ ] **Implement application state machine**
  - Library mode → Downloading → Playing → back to Library
  - Error handling: CPU exceeded → skip frame, OOM → crash, panic → return to library

- [ ] **Implement keyboard/gamepad input**
  - Keyboard mapping to virtual controller
  - Gamepad support via gilrs or similar
  - Multiple local players (e.g., keyboard + gamepad on same instance)
  - Input config persistence

- [ ] **Implement debug overlay (console-wide)**
  - FPS counter
  - Memory usage (RAM/VRAM)
  - Network stats (ping, rollback frames)
  - Toggle via hotkey

### Phase 5: Networking & Polish

- [ ] **Implement multiplayer player model**
  - Max 4 players total (any mix of local + remote)
  - Examples: 4 local, 1 local + 3 remote, 2 local + 2 remote
  - Each local player maps to a physical input device
  - GGRS handles all players uniformly

- [ ] **Implement matchbox signaling connection**
  - Connect to matchbox signaling server
  - Peer connection establishment
  - TODO [needs clarification]: Matchmaking handled by platform service

- [ ] **Implement netplay session management**
  - Host/join game flow via platform
  - Connection quality display
  - Disconnect handling

- [ ] **Implement local network testing**
  - Multiple instances on same machine via localhost
  - Connect via 127.0.0.1:port for testing

- [ ] **Add input delay configuration**
  - Local input delay setting
  - Frame delay vs rollback tradeoff UI

- [ ] **Performance optimization**
  - State serialization optimization
  - Render batching
  - Memory pooling

### Phase 6: Emberware Z Examples

- [ ] **Create `triangle` example**
  - Demonstrates `draw_triangle` (immediate mode 3D)
  - Spinning colored triangle
  - Shows transform stack usage (`transform_rotate`)
  - Minimal no_std WASM game

- [ ] **Create `textured-quad` example**
  - Demonstrates `load_texture` and `texture_bind`
  - Embed PNG via `include_bytes!()`, decode, upload
  - Draw textured sprite with `draw_sprite`
  - Shows texture coordinates and color tinting

- [ ] **Create `cube` example**
  - Demonstrates `load_mesh_indexed` and `draw_mesh` (retained mode)
  - Load cube vertices/indices in `init()`
  - Draw by handle in `render()`
  - Camera setup with `camera_set` and `camera_fov`
  - Interactive rotation via analog stick input

- [ ] **Create `lighting` example**
  - Demonstrates render modes 1-3 (Matcap, PBR-lite, Hybrid)
  - Toggle between modes with button press
  - Show material properties (metallic, roughness, emissive)
  - Dynamic light positioning

- [ ] **Create `skinned-mesh` example**
  - Demonstrates GPU skinning with `FORMAT_SKINNED`
  - Load skinned mesh with bone indices/weights in `init()`
  - Simple bone hierarchy (e.g., arm with 3 bones)
  - Animate bones on CPU each frame (sine wave for demo)
  - Upload bone matrices with `set_bones()`
  - Shows workflow: CPU animation → GPU skinning

- [ ] **Create `platformer` example**
  - Full mini-game demonstrating multiple Z features
  - Textured sprites for player/enemies
  - Simple physics and collision
  - Multiple players with analog stick input
  - Background and foreground layers

---

## IN PROGRESS

(None currently)

---

## DONE

- [x] **Create repository structure**
  - Root Cargo.toml workspace
  - README.md with project overview
  - CLAUDE.md with development instructions
  - .gitignore and LICENSE

- [x] **Create `shared` crate**
  - API types: Game, Author, User, Auth responses
  - Request/response types for platform API
  - LocalGameManifest for downloaded games
  - Error types and codes

- [x] **Create `emberware-z` crate skeleton**
  - Cargo.toml with dependencies
  - main.rs entry point
  - app.rs application state
  - config.rs configuration management
  - deep_link.rs URL parsing
  - download.rs ROM fetching
  - library.rs local game management
  - ui.rs egui library interface
  - runtime/mod.rs module declaration (stubs)

- [x] **Create FFI documentation**
  - docs/ffi.md with complete API reference
  - All function signatures and examples
  - Console specs and lifecycle documentation

- [x] **Create hello-world example**
  - Minimal no_std WASM game
  - Demonstrates init/update/render lifecycle
  - Basic input and rendering

- [x] **Initialize git repository and push to GitHub**

---

## DEFERRED (Emberware Classic)

These tasks are deferred until Emberware Z is complete. Classic shares the core framework but has its own console implementation.

### Classic Console Implementation

- [ ] **Create Emberware Classic `Console` implementation**
  - Implement `Console` trait for SNES/Genesis aesthetic
  - Define Classic-specific specs (384×216 default, 60fps, 4MB RAM, 2MB VRAM)
  - 8 resolution options (4× 16:9 + 4× 4:3, pixel-perfect to 1080p)

- [ ] **Implement Classic graphics backend**
  - `ClassicGraphics` implementing `Graphics` trait
  - 2D-only rendering pipeline (no 3D transforms)
  - Sprite layers (4 layers, back-to-front)
  - Tilemap system (4 layers with parallax scrolling)
  - Palette swapping (256-color indexed textures)

- [ ] **Implement Classic-specific FFI functions**
  - Textures: `load_texture`, `texture_bind`
  - Sprites: `draw_sprite`, `draw_sprite_region`, `draw_sprite_ex` (with flip)
  - Sprite control: `sprite_layer`, `draw_sprite_flipped`
  - Tilemaps: `tilemap_create`, `tilemap_set_texture`, `tilemap_set_tile`, `tilemap_set_tiles`, `tilemap_scroll`
  - Palettes: `palette_create`, `palette_bind`
  - Input: `button_held`, `button_pressed`, `button_released`, `dpad_x`, `dpad_y`
  - Render state: `blend_mode`, `texture_filter`

### Classic Examples

- [ ] **Create `sprites` example (Classic)**
  - Demonstrates Classic-specific 2D features
  - Sprite sheets with `draw_sprite_region`
  - Sprite flipping with `draw_sprite_ex`
  - Sprite layers and priority
  - D-pad input for movement

- [ ] **Create `tilemap` example (Classic)**
  - Demonstrates `tilemap_create` and `tilemap_scroll`
  - Multiple parallax layers
  - Tile animation via `tilemap_set_tile`
  - Sprite/tilemap layer interleaving

- [ ] **Create `palette-swap` example (Classic)**
  - Demonstrates `palette_create` and `palette_bind`
  - Enemy color variants from single sprite
  - Damage flash effect
  - Dynamic palette cycling

- [ ] **Create `platformer` example (Classic)**
  - Full mini-game demonstrating Classic features
  - Tilemap-based levels with scrolling
  - Animated sprite character
  - Parallax background layers
  - 6-button input scheme
