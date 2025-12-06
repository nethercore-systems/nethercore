# Emberware Development Tasks

---

**Architecture Overview:** See [CLAUDE.md](./CLAUDE.md) for framework design and Console trait details.

---

## In Progress

---

## TODO

---

### **[CRITICAL] STABILITY: Refactor ffi/mod.rs (4262 lines → <1000 per file)**

**Current State:**
`emberware-z/src/ffi/mod.rs` is 4262 lines - the largest file in the codebase. It contains 67 FFI functions covering configuration, camera, transforms, rendering, textures, meshes, lighting, materials, audio, and more. This violates the project's stability requirement that no file exceed 2000 lines.

**Why This Matters:**
- Difficult to navigate and find specific functions
- Merge conflicts more likely with single large file
- Hard to understand function organization
- Slower compile times when any FFI function changes

**Proposed Module Structure:**
Split into focused submodules under `emberware-z/src/ffi/`:

```
ffi/
├── mod.rs (150 lines) - Module registration and re-exports
├── input.rs (500 lines) - EXISTING - Button/stick/trigger queries
├── config.rs (200 lines) - set_resolution, set_tick_rate, set_clear_color, render_mode
├── camera.rs (150 lines) - camera_set, camera_fov, push_view_matrix, push_projection_matrix
├── transform.rs (300 lines) - push_identity, transform_set, push_translate, rotate*, scale*
├── render_state.rs (200 lines) - set_color, depth_test, cull_mode, blend_mode, texture_filter
├── texture.rs (250 lines) - load_texture, texture_bind*, matcap_blend_mode
├── mesh.rs (400 lines) - load_mesh*, draw_mesh, draw_triangles*
├── draw_2d.rs (350 lines) - draw_sprite*, draw_rect, draw_text, load_font, font_size
├── billboard.rs (200 lines) - draw_billboard*
├── lighting.rs (300 lines) - light_set, light_color, light_intensity, light_ambient, sky_*
├── material.rs (250 lines) - material_metallic, material_roughness, material_emissive, material_rim
├── audio.rs (300 lines) - play_sound, stop_sound, play_music, stop_music, set_volume
└── skinning.rs (200 lines) - set_bones
```

**Implementation Steps:**
1. Create each submodule file under `emberware-z/src/ffi/`
2. Move related functions from `mod.rs` to appropriate submodules
3. Keep helper functions (e.g., `read_matrix_from_memory`) with their users
4. Update `mod.rs` to re-export all functions and call submodule registration
5. Test that all examples still compile and run correctly
6. Update `register_z_ffi()` to call submodule registration functions:
   ```rust
   pub fn register_z_ffi(linker: &mut Linker<...>) -> Result<()> {
       config::register(linker)?;
       camera::register(linker)?;
       transform::register(linker)?;
       // ... etc
   }
   ```

**Success Criteria:**
- ✅ No single file exceeds 1000 lines
- ✅ All 67 FFI functions still work
- ✅ All examples compile and run without changes
- ✅ Clear module organization by feature area

**Files to Create/Modify:**
- Create 13 new files in `emberware-z/src/ffi/`
- Modify `emberware-z/src/ffi/mod.rs` (4262 → ~150 lines)

---

### **[CRITICAL] STABILITY: Refactor graphics/mod.rs (2144 lines → <800 per file)**

**Current State:**
`emberware-z/src/graphics/mod.rs` is 2144 lines. It contains the entire ZGraphics implementation including initialization, resource management, draw command execution, and frame rendering. Many logical sections are already in separate files (vertex.rs, buffer.rs, pipeline.rs, texture_manager.rs) but the core remains too large.

**Why This Matters:**
- Hard to understand the rendering pipeline flow
- Mixes concerns: initialization, command execution, resource management
- Already has TODO comments like "TODO: Optimize matcap_blend_modes" (line 81 in unified_shading_state.rs suggests more refactoring needed)

**Current Module Structure:**
```
graphics/
├── mod.rs (2144 lines) - ZGraphics implementation, draw execution, frame rendering
├── buffer.rs (144 lines) - GrowableBuffer, MeshHandle
├── command_buffer.rs (exists) - CommandBuffer types
├── matrix_packing.rs (exists) - Matrix compression
├── pipeline.rs (499 lines) - Pipeline management
├── quad_instance.rs (exists) - Quad batching
├── render_state.rs (exists) - RenderState tracking
├── texture_manager.rs (exists) - TextureManager
├── unified_shading_state.rs (exists) - UnifiedShadingState
└── vertex.rs (573 lines) - Vertex formats
```

**Proposed Refactoring:**

Split `mod.rs` into:
```
graphics/
├── mod.rs (400 lines) - ZGraphics struct, initialization, public API
├── resources.rs (500 lines) - Mesh/texture loading and management
├── draw_executor.rs (600 lines) - Execute draw commands (immediate, retained, 2D)
├── frame.rs (400 lines) - Frame rendering, clear, present
└── [existing files remain]
```

**Key Sections to Extract:**

1. **resources.rs** - Move from mod.rs:
   - `create_texture()` and related texture creation
   - `create_mesh()` and mesh creation
   - `RetainedMesh` struct and mesh storage
   - Resource HashMap management

2. **draw_executor.rs** - Move from mod.rs:
   - `execute_draw_commands()` function
   - Immediate mode triangle batching
   - Retained mesh rendering
   - Billboard rendering
   - 2D sprite rendering

3. **frame.rs** - Move from mod.rs:
   - `render()` function
   - Frame setup and teardown
   - Clear operations
   - Render pass management

4. **mod.rs** keeps:
   - ZGraphics struct definition
   - `new()` initialization
   - Public interface methods
   - Re-exports from submodules

**Implementation Steps:**
1. Create `resources.rs`, `draw_executor.rs`, `frame.rs`
2. Move functions and keep them as `pub(crate)` for internal use
3. Update ZGraphics methods to call into submodules
4. Test rendering in all examples (triangle, cube, lighting, billboard, platformer)
5. Verify performance is unchanged (no extra allocations)

**Success Criteria:**
- ✅ No file exceeds 800 lines
- ✅ Clear separation of concerns
- ✅ All rendering modes work (unlit, matcap, PBR, hybrid)
- ✅ All examples render correctly

**Files to Create/Modify:**
- Create `emberware-z/src/graphics/resources.rs`
- Create `emberware-z/src/graphics/draw_executor.rs`
- Create `emberware-z/src/graphics/frame.rs`
- Modify `emberware-z/src/graphics/mod.rs` (2144 → ~400 lines)

---

### **[CRITICAL] STABILITY: Refactor app.rs (2079 lines → <700 per file)**

**Current State:**
`emberware-z/src/app.rs` is 2079 lines. It contains the main application loop, window management, UI rendering (library + settings + in-game), input handling, game lifecycle, and GGRS integration.

**Why This Matters:**
- Mixes UI, game logic, and platform integration
- Hard to test individual components
- UI code (library, settings) dominates the file

**Current Responsibilities:**
1. Window/event handling (winit)
2. Library UI (game browser, download manager)
3. Settings UI (controls, display, audio)
4. In-game UI (debug overlay, pause menu)
5. Game lifecycle (init, update, render)
6. GGRS session management
7. State transitions (Library → InGame → Settings)

**Proposed Module Structure:**
Split into focused modules under `emberware-z/src/`:

```
app/
├── mod.rs (500 lines) - App struct, event loop, state machine
├── game_session.rs (400 lines) - Game lifecycle, GGRS integration, update/render loop
├── ui_library.rs (400 lines) - Move from library.rs (currently 531 lines)
├── ui_settings.rs (400 lines) - Move from settings_ui.rs (currently 504 lines)
└── ui_ingame.rs (300 lines) - Debug overlay, pause menu, performance stats
```

**Key Extractions:**

1. **game_session.rs**:
   - `run_game_frame()` function
   - GGRS session handling
   - Game state management
   - Update/render dispatch

2. **ui_library.rs** (already exists as `library.rs`):
   - Rename and move to `app/` module
   - Game grid rendering
   - Download manager UI
   - Game launching

3. **ui_settings.rs** (already exists as `settings_ui.rs`):
   - Rename and move to `app/` module
   - Controls tab
   - Display tab
   - Audio tab

4. **ui_ingame.rs**:
   - Extract from app.rs
   - Debug overlay (FPS, frame time, rollback stats)
   - Pause menu
   - Performance graphs

5. **app/mod.rs**:
   - App struct and AppState enum
   - Event loop
   - State transitions
   - High-level orchestration

**Implementation Steps:**
1. Create `emberware-z/src/app/` directory
2. Move `library.rs` → `app/ui_library.rs`
3. Move `settings_ui.rs` → `app/ui_settings.rs`
4. Create `app/game_session.rs` and extract game loop code
5. Create `app/ui_ingame.rs` and extract debug overlay code
6. Move current `app.rs` → `app/mod.rs` and trim to orchestration only
7. Update imports throughout codebase
8. Test all state transitions (library → game → settings → library)

**Success Criteria:**
- ✅ No file exceeds 700 lines
- ✅ Clear separation: UI vs game logic vs platform
- ✅ All UI screens work (library, settings, in-game)
- ✅ Game launching and state transitions work
- ✅ GGRS netcode still functions

**Files to Create/Modify:**
- Create `emberware-z/src/app/` directory
- Rename `emberware-z/src/app.rs` → `emberware-z/src/app/mod.rs`
- Move `emberware-z/src/library.rs` → `emberware-z/src/app/ui_library.rs`
- Move `emberware-z/src/settings_ui.rs` → `emberware-z/src/app/ui_settings.rs`
- Create `emberware-z/src/app/game_session.rs`
- Create `emberware-z/src/app/ui_ingame.rs`
- Update `emberware-z/src/main.rs` imports

---

### **[FEATURE] Procedural Mesh API**

**User Request:** "Create a procedural mesh API. Allow users to call stuff like cube() or sphere() with parameters and return a mesh ID they can draw with."

**Current State:**
- Games must manually generate vertex data for common shapes (see `examples/cube/src/lib.rs`)
- No helper functions for primitive shapes
- Each game reimplements cube, sphere, cylinder, etc.
- `load_mesh()` FFI exists but requires manual vertex data

**What's Needed:**
FFI functions to generate common 3D primitives procedurally:

```rust
// Cube with configurable size
fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32  // Returns MeshId

// UV-mapped sphere (latitude/longitude grid)
fn sphere(radius: f32, segments: u32, rings: u32) -> u32

// Cylinder with separate top/bottom radii (cone if radii differ)
fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32

// Plane (quad with subdivisions)
fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32

// Torus (donut shape)
fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32

// Capsule (cylinder with hemispherical caps)
fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```

**Implementation Plan:**

1. **Create `emberware-z/src/procedural.rs`**:
   - Generate vertex data (positions, normals, UVs)
   - Use standard algorithms (icosphere, UV sphere, etc.)
   - Return `Vec<Vertex>` for each primitive

2. **Add FFI functions in `ffi/mesh.rs`** (after refactoring):
   ```rust
   fn cube(caller: Caller, size_x: f32, size_y: f32, size_z: f32) -> u32 {
       let vertices = procedural::cube(size_x, size_y, size_z);
       // Call existing load_mesh() logic
   }
   ```

3. **Vertex Format:**
   - All procedural meshes generate `FORMAT_UV | FORMAT_NORMAL` (vertex format 5)
   - Position (xyz), Normal (xyz), UV (xy)
   - Works with all render modes (unlit, matcap, PBR, hybrid)

4. **UV Mapping:**
   - Cube: Box unwrap (6 faces)
   - Sphere: Equirectangular (lat/lon)
   - Cylinder: Radial unwrap + cap UVs
   - Plane: Simple 0-1 grid
   - Torus: Wrap both axes
   - Capsule: Cylinder body + polar caps

5. **Normal Generation:**
   - Smooth normals for organic shapes (sphere, torus, capsule)
   - Flat normals for cube (per-face)
   - Smooth normals for cylinder sides, flat for caps

**Example Usage (from game code):**
```rust
// In init()
let cube_mesh = cube(2.0, 2.0, 2.0);
let sphere_mesh = sphere(1.0, 16, 16);

// In render()
draw_mesh(cube_mesh);
push_translate(5.0, 0.0, 0.0);
draw_mesh(sphere_mesh);
```

**Reference Implementations:**
- Bevy's `shape` module: https://docs.rs/bevy_render/latest/bevy_render/mesh/shape/
- Three.js geometries: https://threejs.org/docs/#api/en/geometries/BoxGeometry
- glTF primitive shapes

**Success Criteria:**
- ✅ 6 primitive shape functions implemented
- ✅ Correct normals for all shapes (verified visually in lighting example)
- ✅ Proper UV mapping (verified with textured-quad texture)
- ✅ Works in all render modes
- ✅ Document in `docs/emberware-z.md` under "Mesh Functions"
- ✅ Create `examples/procedural-shapes` demonstrating all primitives

**Files to Create/Modify:**
- Create `emberware-z/src/procedural.rs` (~500 lines)
- Modify `emberware-z/src/ffi/mesh.rs` (add 6 FFI wrappers)
- Update `docs/emberware-z.md` (document new functions)
- Create `examples/procedural-shapes/` (demo all shapes)

---

### **[FEATURE] Audio Panning Support**

**Current State:**
`emberware-z/src/audio.rs` has three TODO comments about panning (lines 235, 281, 310):
```rust
// TODO: Implement panning (rodio doesn't have built-in pan control)
```

The audio system is otherwise complete:
- 16 sound effect channels
- Dedicated music channel
- Volume control per channel
- Rollback-aware command buffering
- 22,050 Hz authentic retro audio

**What's Missing:**
- `pan` parameter accepted by `play_sound()` but ignored
- No spatial audio (left/right stereo positioning)
- All sounds play centered

**Why This Matters:**
- Spatial audio improves game feel (bullets whizzing past, enemies off-screen)
- Essential for 3D games and atmospheric 2D games
- Enables directional audio cues (footsteps, voices)

**Implementation Plan:**

Rodio doesn't have built-in panning, so we need to implement it manually:

1. **Pan Implementation (Manual Channel Mixing):**
   ```rust
   // In audio.rs, add a PannedSource wrapper
   struct PannedSource<S> {
       source: S,
       pan: f32,  // -1.0 (left) to 1.0 (right)
   }

   impl<S: Source<Item = i16>> Source for PannedSource<S> {
       type Item = i16;

       fn channels(&self) -> u16 {
           2  // Always output stereo
       }

       fn sample_rate(&self) -> u32 {
           self.source.sample_rate()
       }
   }

   impl<S: Source<Item = i16>> Iterator for PannedSource<S> {
       type Item = i16;

       fn next(&mut self) -> Option<i16> {
           let sample = self.source.next()?;

           // Convert pan -1..1 to gain multipliers
           // pan = -1: left=1.0, right=0.0
           // pan =  0: left=0.707, right=0.707 (equal power)
           // pan = +1: left=0.0, right=1.0
           let angle = (self.pan + 1.0) * 0.25 * PI;  // 0..PI/2
           let left_gain = angle.cos();
           let right_gain = angle.sin();

           // Alternate left/right samples for stereo
           static mut IS_LEFT: bool = true;
           unsafe {
               let gain = if IS_LEFT { left_gain } else { right_gain };
               IS_LEFT = !IS_LEFT;
               Some((sample as f32 * gain) as i16)
           }
       }
   }
   ```

2. **Update AudioChannel to Store Pan:**
   ```rust
   // In audio.rs
   struct AudioChannel {
       sink: Sink,
       sound_id: Option<u32>,
       volume: f32,
       pan: f32,  // ADD THIS
   }
   ```

3. **Apply Panning in play_sound():**
   ```rust
   // In audio.rs:219, replace TODO with:
   let source = sound.to_source();
   let panned = PannedSource { source, pan };
   sink.append(panned);
   ```

4. **Add set_pan FFI Function:**
   ```rust
   // In ffi/audio.rs (after refactoring) or ffi/mod.rs (current)
   fn set_pan(caller: Caller, channel: u32, pan: f32) {
       let state = caller.data_mut();
       state.console_state.audio_commands.push(AudioCommand::SetPan {
           channel,
           pan: pan.clamp(-1.0, 1.0),
       });
   }
   ```

5. **Document in docs/emberware-z.md:**
   ```markdown
   ### Audio Functions

   fn play_sound(sound_id: u32, volume: f32, pan: f32) -> u32
   fn set_pan(channel: u32, pan: f32)

   - `pan`: -1.0 (full left) to 1.0 (full right), 0.0 = center
   - Uses equal-power panning for smooth stereo image
   ```

**Alternative: Use Rodio's Spatial Sink (Future):**
If rodio adds spatial audio in the future, we can switch to `rodio::SpatialSink` with 3D positioning.

**Success Criteria:**
- ✅ Panning works (-1.0 = left, 0.0 = center, 1.0 = right)
- ✅ `set_pan()` FFI function implemented
- ✅ Equal-power panning (no volume drop at center)
- ✅ Works with existing sound playback
- ✅ Document in `docs/emberware-z.md`
- ✅ Update `examples/audio-test` to demonstrate panning (or create if missing)

**Files to Modify:**
- `emberware-z/src/audio.rs` (add PannedSource, update play_sound/set_pan)
- `emberware-z/src/ffi/audio.rs` or `ffi/mod.rs` (add set_pan FFI)
- `docs/emberware-z.md` (document pan parameter)
- `examples/audio-test/` or create if missing

---

### **[STABILITY] Reduce unwrap/expect Usage**

**Current State:**
Grep analysis shows 391 instances of `unwrap()` or `expect()` across 15 files:
- `core/src/ffi.rs`: 101 instances
- `core/src/integration.rs`: 107 instances
- `core/src/wasm/mod.rs`: 98 instances
- `emberware-z/src/graphics/pipeline.rs`: Many instances
- `emberware-z/src/shader_gen.rs`: 6 instances
- `emberware-z/src/app.rs`: 4 instances

**Why This Matters:**
- Unwraps can panic and crash the entire application
- Poor error messages for game developers debugging issues
- Violates Rust best practices for library code
- Hard to diagnose issues in WASM games

**Risk Assessment:**

**High Risk (MUST fix):**
- `core/src/ffi.rs` (101) - FFI calls from untrusted WASM
- `core/src/integration.rs` (107) - Core game loop integration
- `core/src/wasm/mod.rs` (98) - WASM module loading and execution

**Medium Risk (SHOULD fix):**
- `emberware-z/src/graphics/pipeline.rs` - Pipeline creation (can fail on GPU errors)
- `emberware-z/src/input.rs` (6) - Input handling

**Low Risk (COULD fix later):**
- `xtask/src/main.rs` (1) - Build tool, acceptable to panic
- `shared/src/lib.rs` (1) - Likely constants or tests
- `emberware-z/src/config.rs` (14) - Config parsing, fail early is OK

**Implementation Strategy:**

1. **Core FFI Functions (Highest Priority):**
   - Replace `unwrap()` with proper error handling
   - Return `Result<T, Trap>` for FFI functions (wasmtime Trap type)
   - Add descriptive error messages:
   ```rust
   // Before
   let data = memory.data(&caller).unwrap();

   // After
   let data = memory.data(&caller)
       .ok_or_else(|| Trap::new("Failed to access WASM memory"))?;
   ```

2. **WASM Module Loading:**
   - Return detailed errors for module instantiation failures
   - Validate WASM exports (init, update, render) with clear errors
   - Better error messages for linker failures

3. **Graphics Pipeline:**
   - Handle GPU resource creation failures gracefully
   - Fall back to error textures/meshes on load failure
   - Log warnings instead of panicking on non-critical failures

4. **Add Error Types:**
   ```rust
   // In core/src/error.rs (create new file)
   #[derive(Debug, thiserror::Error)]
   pub enum EmberwareError {
       #[error("WASM memory access failed: {0}")]
       MemoryAccess(String),

       #[error("Invalid FFI parameter: {0}")]
       InvalidParameter(String),

       #[error("Resource not found: {0}")]
       ResourceNotFound(String),

       #[error("GPU error: {0}")]
       GraphicsError(String),
   }
   ```

**Success Criteria:**
- ✅ Zero unwraps in `core/src/ffi.rs`
- ✅ Zero unwraps in `core/src/integration.rs`
- ✅ Zero unwraps in `core/src/wasm/mod.rs`
- ✅ All FFI functions return Result or handle errors gracefully
- ✅ Error messages are descriptive and actionable
- ✅ No panics during normal operation (even with bad WASM)

**Files to Modify:**
- Create `core/src/error.rs` (new error types)
- Modify `core/src/ffi.rs` (101 unwraps → Result)
- Modify `core/src/integration.rs` (107 unwraps → Result)
- Modify `core/src/wasm/mod.rs` (98 unwraps → Result)
- Modify `emberware-z/src/graphics/pipeline.rs` (handle GPU errors)

---

### **[DOCUMENTATION] Create rendering-architecture.md**

**Current State:**
- `CLAUDE.md` references `docs/rendering-architecture.md` (lines 17, 161)
- File does not exist (confirmed via Glob search)
- Rendering architecture is complex but undocumented:
  - 4 render modes (Unlit, Matcap, PBR-lite, Hybrid)
  - 8 vertex formats with 40 shader permutations
  - Template-based shader generation
  - Unified shading state
  - Sky system for ambient/reflection

**Why This Matters:**
- Broken link in main project documentation
- New contributors can't understand rendering architecture
- Game developers don't know when to use which render mode
- Shader generation is "magic" without documentation

**What Should Be Documented:**

1. **Rendering Pipeline Overview:**
   - CPU-side command buffering
   - GPU resource management
   - Frame rendering flow
   - Vertex buffer architecture (one per stride)

2. **Render Modes (0-3):**
   - **Mode 0 (Unlit)**: Vertex colors, textures, no lighting
   - **Mode 1 (Matcap)**: Matcap textures (3 slots), blend modes
   - **Mode 2 (Blinn-Phong)**: Metallic-roughness Blinn-Phong, dynamic lights
   - **Mode 3 (Hybrid)**: PBR + matcap ambient, best of both

3. **Vertex Formats:**
   - 8 base formats: POS, POS_UV, POS_COLOR, POS_UV_COLOR, POS_NORMAL, POS_UV_NORMAL, POS_COLOR_NORMAL, POS_UV_COLOR_NORMAL
   - 8 skinned variants: Add bone indices/weights
   - Automatic format detection in FFI

4. **Shader Generation:**
   - Template-based system in `shader_gen.rs`
   - 40 permutations (4 modes × 8 formats + variations)
   - Compile-time generation via build.rs
   - WGSL templates in `emberware-z/shaders/`

5. **Material System:**
   - Texture slots: 0=Albedo, 1=MRE (metallic/roughness/emissive), 2=Matcap
   - Material properties: metallic, roughness, emissive, rim
   - Blend modes for matcaps (multiply, add, HSV modulate)

6. **Lighting:**
   - Up to 8 dynamic point lights
   - Ambient lighting (uniform or sky-derived)
   - Sky gradient + sun for IBL-lite
   - Rim lighting (Gotanda 2010 model)

7. **Performance Characteristics:**
   - Immediate-mode: CPU-buffered, GPU-flushed per frame
   - Retained-mode: Persistent GPU meshes
   - One draw call per mesh (no batching yet)
   - VRAM limit: 4MB (tracked)

**Document Structure:**
```markdown
# Emberware Z Rendering Architecture

## Overview
[High-level architecture diagram]

## Rendering Pipeline
[CPU → GPU flow, command buffer pattern]

## Render Modes
### Mode 0: Unlit
[When to use, examples, screenshots]

### Mode 1: Matcap
[Matcap slots, blend modes, artistic use cases]

### Mode 2: Blinn-Phong (Metallic-Roughness)
[PBR workflow, lighting model, migration from old Mode 2]

### Mode 3: Hybrid
[Combining matcap + PBR, best practices]

## Vertex Formats
[Table of formats, memory layout, when to use each]

## Shader Generation System
[How templates work, adding new shaders, WGSL includes]

## Material System
[Texture slots, material properties, workflows]

## Lighting
[Light setup, sky system, ambient/IBL, rim lighting]

## Performance Considerations
[VRAM tracking, draw call optimization, vertex buffer architecture]

## Common Issues and Solutions
[Troubleshooting guide]
```

**Success Criteria:**
- ✅ File exists at `docs/rendering-architecture.md`
- ✅ All sections documented with examples
- ✅ Diagrams for complex concepts (pipeline flow, vertex formats)
- ✅ Code examples showing how to use each render mode
- ✅ Screenshots from examples demonstrating each mode
- ✅ Links to relevant source files for deep dives

**Files to Create:**
- `docs/rendering-architecture.md` (~800-1000 lines with examples)

**References for Content:**
- `emberware-z/src/graphics/mod.rs` (ZGraphics implementation)
- `emberware-z/src/shader_gen.rs` (shader permutation logic)
- `emberware-z/shaders/*.wgsl` (shader templates)
- `docs/emberware-z.md` (FFI reference)
- `examples/lighting/`, `examples/billboard/` (visual examples)

---

### **[TESTING] Add Integration Tests for FFI Functions**

**Current State:**
- Tests exist in 28 files (found via grep for `#[test]`)
- Most tests are unit tests in individual modules
- No integration tests for FFI contract between host and WASM
- Examples serve as manual integration tests
- No automated testing of FFI function behavior

**Why This Matters:**
- FFI is the most critical interface (games depend on it)
- Easy to break FFI contracts during refactoring
- No way to verify FFI behavior without running full examples
- Refactoring efforts (splitting ffi/mod.rs) risk breaking games

**What Should Be Tested:**

1. **Core FFI Functions** (from `core/src/ffi.rs`):
   - `random()` - Returns deterministic values with same seed
   - `save()` / `load()` / `delete()` - Save data persistence
   - `player_count()` / `local_player_mask()` - Multiplayer state
   - `get_ticks()` - Tick counter increments

2. **Graphics FFI** (from `emberware-z/src/ffi/`):
   - Texture loading and binding
   - Mesh creation (immediate and retained)
   - Transform stack (push/pop behavior)
   - Camera setup
   - Material properties
   - Render state (depth test, cull mode, blend mode)

3. **Audio FFI**:
   - Sound playback returns valid channel ID
   - Volume changes apply correctly
   - Stop sound clears channel
   - Music playback separate from sound effects

4. **Input FFI**:
   - Button state queries (held, pressed, released)
   - Analog stick values
   - Multi-player input routing

**Implementation Strategy:**

Create `core/tests/ffi_integration.rs`:
```rust
use emberware_core::test_utils::*;  // Helpers for WASM test harness

#[test]
fn test_random_deterministic() {
    let mut test_env = TestEnvironment::new();

    // Load minimal WASM module that calls random()
    test_env.load_wasm(include_bytes!("fixtures/random_test.wasm"));

    // Run twice with same seed
    let result1 = test_env.call_function("test_random");
    test_env.reset();
    let result2 = test_env.call_function("test_random");

    assert_eq!(result1, result2, "random() should be deterministic");
}

#[test]
fn test_save_load_roundtrip() {
    let mut test_env = TestEnvironment::new();
    test_env.load_wasm(include_bytes!("fixtures/save_test.wasm"));

    // Call WASM function that saves data
    test_env.call_function("save_data");

    // Reset WASM state
    test_env.reset();

    // Call WASM function that loads data
    let loaded_value = test_env.call_function("load_data");

    assert_eq!(loaded_value, 42, "save/load should persist data");
}
```

**Test Fixtures:**
Create small WASM modules in `core/tests/fixtures/`:
- `random_test.wasm` - Calls random() multiple times
- `save_test.wasm` - Calls save()/load()
- `transform_test.wasm` - Tests transform stack
- `texture_test.wasm` - Tests texture loading

**Test Helper Infrastructure:**
```rust
// In core/src/test_utils.rs (already exists, expand it)
pub struct TestEnvironment {
    runtime: Runtime<TestConsole>,
    // ... test harness
}

impl TestEnvironment {
    pub fn new() -> Self { /* ... */ }
    pub fn load_wasm(&mut self, bytes: &[u8]) { /* ... */ }
    pub fn call_function(&mut self, name: &str) -> i32 { /* ... */ }
    pub fn reset(&mut self) { /* ... */ }
}
```

**Success Criteria:**
- ✅ 20+ FFI integration tests covering core functions
- ✅ Tests run in CI (add to GitHub Actions if exists)
- ✅ Tests catch breaking changes to FFI contracts
- ✅ Test fixtures compile from Rust → WASM
- ✅ Clear error messages when tests fail

**Files to Create/Modify:**
- Create `core/tests/ffi_integration.rs`
- Create `core/tests/fixtures/*.rs` (compile to WASM)
- Expand `core/src/test_utils.rs` (test harness)
- Add `build.rs` to compile fixtures to WASM

---

### **[FEATURE] Add draw_sky() Function**

**Current State:**
- Emberware Z has a procedural sky system (gradient + sun)
- Sky provides ambient lighting and IBL-lite reflections in PBR modes
- Sky is configured via FFI: `sky_sun_color()`, `sky_gradient_top()`, `sky_gradient_bottom()`, `sky_sun_direction()`
- **No way to actually draw the sky as geometry** - it only affects lighting

**The Problem:**
Games that want a visible sky background have to:
1. Manually create a large sphere mesh
2. Texture it or use vertex colors to match sky gradient
3. Position it around the camera
4. Update it when sky settings change

This is tedious and error-prone. The sky configuration exists but isn't rendered.

**What's Needed:**
A simple FFI function to render the procedural sky as a background:

```rust
fn draw_sky()
```

**Expected Behavior:**
- Draws a full-screen quad or skybox using current sky settings
- Renders gradient from `sky_gradient_bottom()` (horizon) to `sky_gradient_top()` (zenith)
- Renders sun disc at `sky_sun_direction()` with `sky_sun_color()`
- Depth test disabled (always behind everything)
- No transform required (always fills viewport)
- Works in all render modes

**Implementation Plan:**

1. **Sky Rendering Approach:**
   Option A: Full-screen quad with gradient shader
   ```rust
   // Draw quad covering entire screen
   // Fragment shader interpolates gradient top→bottom
   // Add sun as bright disc based on view direction
   ```

   Option B: Skybox cube
   ```rust
   // Draw inverted cube around camera
   // Sample gradient based on Y coordinate
   // Add sun based on cube face
   ```

   **Recommendation:** Option A (full-screen quad) - simpler, faster, authentic to PS1/N64 era

2. **Add to FFI** (in `ffi/draw_3d.rs` or `ffi/mod.rs`):
   ```rust
   fn draw_sky(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>) {
       let state = caller.data_mut();

       // Add sky draw command to command buffer
       state.console_state.draw_commands.push(DrawCommand::Sky);
   }
   ```

3. **Execute in Graphics Backend:**
   ```rust
   // In ZGraphics::render() or draw_executor.rs
   DrawCommand::Sky => {
       // Disable depth test/write
       // Draw full-screen quad
       // Use sky gradient uniforms
       // Add sun disc
   }
   ```

4. **Sky Shader:**
   ```wgsl
   // New shader: sky.wgsl
   @fragment
   fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
       // Interpolate gradient based on UV.y
       let sky_color = mix(
           sky_uniforms.gradient_bottom,
           sky_uniforms.gradient_top,
           uv.y
       );

       // Add sun disc
       let view_dir = normalize(vec3(uv * 2.0 - 1.0, -1.0));
       let sun_dot = dot(view_dir, sky_uniforms.sun_direction);
       let sun_intensity = smoothstep(0.9995, 0.9999, sun_dot);
       let sun_contribution = sky_uniforms.sun_color * sun_intensity;

       return vec4(sky_color + sun_contribution, 1.0);
   }
   ```

5. **Usage Example:**
   ```rust
   // In game render()
   fn render() {
       // Configure sky
       sky_gradient_top(0x87CEEBFF);    // Sky blue
       sky_gradient_bottom(0xFFE4B5FF); // Moccasin (warm horizon)
       sky_sun_color(0xFFFAF0FF);       // Floral white
       sky_sun_direction(0.5, 0.707, 0.5); // 45° elevation

       // Draw sky first (before any geometry)
       draw_sky();

       // Draw scene geometry
       draw_mesh(terrain_mesh);
       draw_mesh(player_mesh);
   }
   ```

**Design Decisions:**

1. **Draw order:** Should be called first in render() (before geometry)
2. **Depth:** Always at infinite depth (depth = 1.0)
3. **Performance:** Single full-screen quad = 2 triangles (negligible cost)
4. **Compatibility:** Works with all render modes (unlit, matcap, PBR, hybrid)

**Success Criteria:**
- ✅ `draw_sky()` FFI function implemented
- ✅ Renders gradient from bottom to top color
- ✅ Renders sun disc at specified direction
- ✅ No depth conflicts with scene geometry
- ✅ Documented in `docs/emberware-z.md` under "Sky Functions"
- ✅ Update existing examples to use `draw_sky()` where appropriate
- ✅ Add to procedural-shapes example (once created)

**Files to Create/Modify:**
- Create `emberware-z/shaders/sky.wgsl` (sky rendering shader)
- Modify `emberware-z/src/ffi/mod.rs` or `ffi/draw_3d.rs` (add draw_sky FFI)
- Modify `emberware-z/src/graphics/mod.rs` or `draw_executor.rs` (execute sky draw command)
- Update `docs/emberware-z.md` (document draw_sky function)
- Update `examples/lighting/src/lib.rs` (demonstrate sky rendering)

---

### **[POLISH] Improve Error Messages in FFI Functions**

**Current State:**
Many FFI functions silently fail or have generic error messages:
- "Failed to load texture" (no details on why)
- "Invalid mesh ID" (which ID? what's the valid range?)
- Panics on invalid parameters instead of returning error codes

**Examples from Code:**
```rust
// emberware-z/src/ffi/mod.rs (various locations)
warn!("Invalid texture ID: {}", texture_id);  // Logged but game doesn't know
// Continues execution with undefined behavior
```

**Why This Matters:**
- Game developers waste time debugging issues
- No way to handle errors gracefully in WASM
- Poor developer experience compared to modern engines

**What's Needed:**

1. **Consistent Error Handling Pattern:**
   ```rust
   // Return error codes instead of logging and continuing
   fn load_texture(...) -> u32 {
       // Returns texture ID on success
       // Returns 0 on failure (reserved as error texture)
   }

   // Add get_last_error() FFI function
   fn get_last_error(buffer_ptr: u32, buffer_len: u32) -> u32 {
       // Writes error string to WASM memory
       // Returns error string length
   }
   ```

2. **Descriptive Error Messages:**
   ```rust
   // Before
   if texture_id >= textures.len() {
       warn!("Invalid texture ID");
       return;
   }

   // After
   if texture_id >= textures.len() {
       set_error(format!(
           "Invalid texture ID {}: only {} textures loaded (IDs 1-{})",
           texture_id,
           textures.len(),
           textures.len()
       ));
       return 0;  // Error texture
   }
   ```

3. **Error Categories:**
   - **Validation Errors:** Invalid parameters, out of range IDs
   - **Resource Errors:** File not found, out of memory, VRAM limit
   - **State Errors:** Called at wrong time (e.g., render_mode in update)
   - **GPU Errors:** Shader compilation, buffer creation failures

4. **Add Thread-Local Error Storage:**
   ```rust
   // In emberware-z/src/state.rs or ffi/mod.rs
   thread_local! {
       static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
   }

   pub fn set_error(msg: String) {
       LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
       error!("FFI Error: {}", msg);
   }

   pub fn get_last_error() -> Option<String> {
       LAST_ERROR.with(|e| e.borrow_mut().take())
   }
   ```

5. **Document Error Codes:**
   ```markdown
   // In docs/emberware-z.md

   ## Error Handling

   Most FFI functions return 0 or an invalid handle on error.
   Use `get_last_error()` to retrieve a detailed error message.

   fn get_last_error(buffer_ptr: *mut u8, buffer_len: u32) -> u32

   Example usage:
   let texture = load_texture(data_ptr, data_len, width, height);
   if texture == 0 {
       let mut buffer = [0u8; 256];
       let len = get_last_error(buffer.as_mut_ptr(), 256);
       let error = std::str::from_utf8(&buffer[..len]).unwrap();
       panic!("Failed to load texture: {}", error);
   }
   ```

**Success Criteria:**
- ✅ All FFI functions return error codes instead of silently failing
- ✅ `get_last_error()` FFI function implemented
- ✅ Error messages include:
  - What went wrong
  - Actual vs expected values
  - How to fix it
- ✅ Document error handling in `docs/emberware-z.md`
- ✅ Update examples to check return values

**Files to Modify:**
- `emberware-z/src/ffi/*.rs` (all FFI functions - after refactoring)
- `emberware-z/src/state.rs` (add error storage)
- `docs/emberware-z.md` (document error handling)
- `examples/*/src/lib.rs` (show error checking patterns)

---

### **[BUG] Fix "Launch by Command Argument" Bug**

**Current State:**
`emberware-z/src/game_resolver.rs` and `emberware-z/src/main.rs` implement game launching via command-line arguments:
```bash
cargo run -- platformer    # Should launch platformer game
cargo run -- plat          # Should match by prefix
```

**The Bug:**
The command-line argument game resolution is not working correctly. Need to investigate:
- Is the argument being parsed?
- Is the game_resolver matching correctly?
- Are there issues with the library lookup?
- Does it fail silently or show an error?

**Expected Behavior:**
1. Exact match: `cargo run -- platformer` launches platformer
2. Case-insensitive: `cargo run -- PLATFORMER` launches platformer
3. Prefix match: `cargo run -- plat` launches platformer (if unique)
4. Typo suggestions: `cargo run -- platformmer` suggests "platformer"
5. Clear error if no match or ambiguous

**Investigation Steps:**
1. Read `emberware-z/src/main.rs` to understand argument parsing
2. Read `emberware-z/src/game_resolver.rs` to understand matching logic
3. Test with actual commands to reproduce the bug
4. Add logging to trace execution flow
5. Check if library data is loaded correctly when using CLI args

**Files to Investigate:**
- `emberware-z/src/main.rs` (lines ~50-100, argument parsing)
- `emberware-z/src/game_resolver.rs` (entire file ~300 lines)
- `emberware-z/src/library.rs` (game loading logic)

**Success Criteria:**
- ✅ CLI argument parsing works for all match types
- ✅ Error messages are clear when game not found
- ✅ Typo suggestions work (Levenshtein distance)
- ✅ Add integration test for CLI argument parsing
- ✅ Document CLI usage in README.md

---

### **[CRITICAL] API: Unify FFI Color Parameters**

**Current State:**
FFI functions inconsistently use different color formats:
- Some use `color: u32` (packed 0xRRGGBBAA)
- Some use `r: f32, g: f32, b: f32` (separate components)
- Some use `r: f32, g: f32, b: f32, a: f32` (with alpha)

**Examples of Inconsistency:**
```rust
// Uses packed u32
fn set_clear_color(color: u32)          // 0xRRGGBBAA
fn set_color(color: u32)                // 0xRRGGBBAA
fn draw_rect(x, y, w, h, color: u32)    // 0xRRGGBBAA

// Uses separate f32 components (assumed, need to verify)
fn light_color(light_id: u32, r: f32, g: f32, b: f32)
fn material_emissive(r: f32, g: f32, b: f32)
fn sky_sun_color(r: f32, g: f32, b: f32)
```

**Why This Matters:**
- Inconsistent API is confusing for game developers
- Hard to remember which functions use which format
- Copy-paste errors when switching between functions
- Wastes WASM→host call overhead with multiple parameters

**Proposed Standard:**
**Use `u32` for all color parameters (0xRRGGBBAA format)**

**Benefits:**
- Single 4-byte value instead of 3-4 floats (12-16 bytes)
- Matches web/game dev conventions (hex colors)
- Easier to work with in code: `0xFF0000FF` (red) vs `1.0, 0.0, 0.0, 1.0`
- Consistent with existing functions like `set_color()`
- Less WASM function call overhead (1 param vs 3-4)

**Implementation Plan:**

1. **Audit All FFI Functions:**
   ```bash
   # Find all color-related FFI functions
   grep -n "fn.*color\|fn.*Color" emberware-z/src/ffi/mod.rs
   ```

2. **Create Color Utility Helpers:**
   ```rust
   // In emberware-z/src/ffi/mod.rs or ffi/utils.rs

   /// Convert packed u32 color to f32 components
   pub fn unpack_color(color: u32) -> (f32, f32, f32, f32) {
       let r = ((color >> 24) & 0xFF) as f32 / 255.0;
       let g = ((color >> 16) & 0xFF) as f32 / 255.0;
       let b = ((color >> 8) & 0xFF) as f32 / 255.0;
       let a = (color & 0xFF) as f32 / 255.0;
       (r, g, b, a)
   }

   /// Convert packed u32 color to RGB only (ignore alpha)
   pub fn unpack_color_rgb(color: u32) -> (f32, f32, f32) {
       let (r, g, b, _) = unpack_color(color);
       (r, g, b)
   }
   ```

3. **Update FFI Function Signatures:**
   ```rust
   // BEFORE
   fn light_color(caller: Caller, light_id: u32, r: f32, g: f32, b: f32) { ... }

   // AFTER
   fn light_color(caller: Caller, light_id: u32, color: u32) {
       let (r, g, b, _) = unpack_color_rgb(color);
       // ... rest of implementation
   }
   ```

4. **Functions to Update:**
   - `light_color(light_id, color)` - Currently uses f32 r, g, b
   - `light_ambient(color)` - Currently uses f32 r, g, b
   - `material_emissive(color)` - Currently uses f32 r, g, b
   - `sky_sun_color(color)` - Currently uses f32 r, g, b
   - `sky_gradient_top(color)` - Currently uses f32 r, g, b
   - `sky_gradient_bottom(color)` - Currently uses f32 r, g, b
   - Any other color functions found during audit

5. **Update Documentation:**
   - Update `docs/emberware-z.md` to show new signatures
   - Add color format documentation:
     ```markdown
     ## Color Format

     All colors use packed u32 format: 0xRRGGBBAA
     - Red: bits 24-31
     - Green: bits 16-23
     - Blue: bits 8-15
     - Alpha: bits 0-7

     Examples:
     - Red: 0xFF0000FF
     - Green: 0x00FF00FF
     - Blue: 0x0000FFFF
     - White: 0xFFFFFFFF
     - Black: 0x000000FF
     - Transparent: 0x00000000
     ```

6. **Update Examples:**
   - Find all examples using the old API
   - Update to use new packed u32 format
   - Verify they still render correctly

**Migration Path for Game Developers:**
```rust
// Old code (before)
light_color(0, 1.0, 0.5, 0.0);  // Orange light

// New code (after)
light_color(0, 0xFF8000FF);     // Orange light (same)

// Helper macro for easier migration
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        ((($r as u32) << 24) | (($g as u32) << 16) | (($b as u32) << 8) | 0xFF)
    };
}

light_color(0, rgb!(255, 128, 0));  // Orange
```

**Success Criteria:**
- ✅ ALL color parameters use u32 format
- ✅ Zero functions use separate r,g,b parameters
- ✅ Documentation updated with color format spec
- ✅ All examples updated and tested
- ✅ Color utility helpers provided
- ✅ Migration guide in docs

**Files to Modify:**
- Audit and update `emberware-z/src/ffi/*.rs` (all color functions)
- Update `docs/emberware-z.md` (add color format section)
- Update all `examples/*/src/lib.rs` (convert color calls)
- Add migration guide to `docs/ffi.md`

**Breaking Change:**
This is a **breaking API change**. Consider:
- Add to changelog
- Version bump (0.1 → 0.2)
- Provide migration script or helpers

---

### **[RESEARCH] Developer Tooling: Asset Exporters & Converters**

**Goal:**
Research and design developer tools to export 3D models, images, and other assets to Emberware's native formats.

**Current Situation:**
- Game developers must manually create vertex data in code
- No tools to convert glTF/OBJ/FBX → Emberware mesh format
- No image converter for optimal texture formats
- No font converter for bitmap fonts
- Asset pipeline is entirely manual

**Why This Matters:**
- Lowers barrier to entry for game developers
- Enables artists to create content without programming
- Improves iteration speed (export from Blender, test in game)
- Allows use of industry-standard tools (Blender, Aseprite, etc.)

**Dependencies:**
- **[POLISH] PERF: Pack vertex data to reduce memory/bandwidth** - Need to finalize vertex format before exporting
- **[FEATURE] Procedural Mesh API** - Defines what a "mesh" looks like

**Research Areas:**

### 1. **3D Mesh Exporter**

**Input Formats to Support:**
- glTF 2.0 (.gltf, .glb) - Industry standard, Blender default export
- Wavefront OBJ (.obj) - Simple, widely supported
- FBX (.fbx) - Common in game dev (Unity, Unreal)
- Collada (.dae) - Open format

**Output Format:**
Custom binary format optimized for Emberware:
```rust
// Proposed .embermesh format
struct EmberMeshFile {
    magic: [u8; 4],              // "EMSH"
    version: u32,                 // Format version
    vertex_format: u32,           // Flags: UV | COLOR | NORMAL | SKINNED
    vertex_count: u32,
    index_count: u32,
    vertices: Vec<u8>,            // Packed vertex data (format-specific)
    indices: Option<Vec<u16>>,    // Optional index buffer
    metadata: MeshMetadata,       // Name, bounds, etc.
}

struct MeshMetadata {
    name: String,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}
```

**Exporter Features:**
- Vertex attribute conversion (positions, normals, UVs, colors)
- Automatic normal generation if missing
- Vertex welding/deduplication
- Triangle mesh only (convert quads/ngons)
- Material baking (vertex colors from material)
- LOD generation (optional)
- Validation (vertex limits, format compatibility)

**Tool Implementation Options:**
1. **Blender Add-on** (Python):
   - Most popular tool for indie devs
   - Direct export from Blender UI
   - Can batch export entire scene

2. **Standalone CLI Tool** (Rust):
   - `ember-export mesh input.gltf output.embermesh`
   - Cross-platform
   - Can be integrated into build pipelines

3. **Web-based Converter:**
   - Drag-and-drop interface
   - No installation required
   - Preview before export

**Recommendation:** Start with CLI tool, add Blender add-on later.

### 2. **Texture/Image Converter**

**Input Formats:**
- PNG, JPG, TGA, BMP (via `image` crate)
- PSD, XCF (Photoshop, GIMP) via plugins

**Output Formats:**
- Raw RGBA8 (current, simple but large)
- Palette + index (256 colors, authentic retro)
- BC1/BC3 compression (GPU-accelerated, 4:1 or 6:1 compression)
- Custom packed formats (5551, 565, 4444)

**Converter Features:**
- Resize/downsample to console resolution limits
- Automatic palette generation (quantization)
- Dithering options (Floyd-Steinberg, Bayer, none)
- Mipmap generation
- Normal map generation from height maps
- Atlas/sprite sheet packing

**Tool:**
```bash
ember-export texture input.png output.embertex --format rgba8
ember-export texture input.png output.embertex --format palette --colors 256 --dither floyd
```

### 3. **Font Converter**

**Input Formats:**
- TrueType (.ttf) / OpenType (.otf)
- Bitmap fonts (.fnt, BMFont format)

**Output Format:**
Bitmap font atlas + metadata:
```rust
struct EmberFont {
    atlas_texture: TextureData,   // Single texture with all glyphs
    glyphs: Vec<GlyphMetadata>,   // Position, size, advance
    line_height: f32,
    baseline: f32,
}

struct GlyphMetadata {
    codepoint: u32,
    x: u16, y: u16,             // Position in atlas
    width: u16, height: u16,
    offset_x: i16, offset_y: i16,
    advance: f32,
}
```

**Converter Features:**
- Render TrueType → bitmap at specified size
- SDF (Signed Distance Field) generation for scalable rendering
- Subset selection (ASCII only, Latin-1, Unicode ranges)
- Kerning pairs
- Fallback glyph handling

**Tool:**
```bash
ember-export font input.ttf output.emberfont --size 16 --charset ascii
```

### 4. **Audio Converter**

**Input Formats:**
- WAV, MP3, OGG, FLAC

**Output Format:**
Raw PCM (current: 16-bit mono @ 22,050 Hz)

**Converter Features:**
- Resample to 22,050 Hz
- Convert stereo → mono
- Volume normalization
- Trim silence
- Loop point markers

**Tool:**
```bash
ember-export audio input.wav output.embersnd --normalize --trim-silence
```

### 5. **Asset Pipeline Integration**

**Build Script Integration:**
```toml
# In game Cargo.toml
[package.metadata.emberware]
assets = "assets/"

# Automatically converts:
# assets/models/*.gltf → rom/meshes/*.embermesh
# assets/textures/*.png → rom/textures/*.embertex
# assets/audio/*.wav → rom/sounds/*.embersnd
```

**Hot Reload Support:**
- Watch asset directory for changes
- Auto-reconvert and reload in running game
- Speeds up iteration for artists

**Research Deliverables:**
1. **Design Document** (`docs/asset-pipeline.md`):
   - File format specifications
   - Tool architecture
   - Integration guide
   - Performance considerations

2. **Proof-of-Concept:**
   - Simple CLI tool for one asset type (mesh exporter)
   - Demonstrate end-to-end workflow (Blender → Emberware)

3. **Prioritization:**
   - Which tools provide most value?
   - Which are easiest to implement?
   - Suggested implementation order

**Questions to Answer:**
1. Should assets be embedded in WASM or loaded at runtime?
2. What's the target workflow: export → copy files, or integrated build?
3. How do we handle versioning (format changes break old assets)?
4. Should we use existing formats (glTF as-is) or custom binary?
5. How to balance file size vs. load time vs. runtime performance?

**Success Criteria:**
- ✅ Comprehensive design document written
- ✅ File format specs defined
- ✅ At least one proof-of-concept tool implemented
- ✅ Workflow documented with examples
- ✅ Implementation roadmap created

**Files to Create:**
- `docs/asset-pipeline.md` (design document)
- `tools/ember-export/` (CLI tool crate)
- `tools/blender-addon/` (optional, future)

**Timeline:**
This is a research task - deliverable is the design document, not full implementation.
Implementation should be broken into separate tasks based on research findings.

---

### **[CRITICAL] STABILITY Codebase is huge and clunky**
- Lots of files are extremely long (2k+)
- We must go through the entire repository and clean this up. Some ways we can accomplish this are
1. Refactor heavily duplicated code to prevent copy paste
2. Split files into smaller focused ones.
- Any file which is longer than 2000 lines MUST be made smaller, preferrably under 1000 lines each.

---

### **CRITICAL POLISH: Matcap shaders should use perspective correct UV sampling **
- Currently, matcaps are using the simple uv lookup
```
// Compute matcap UV from view-space normal
fn compute_matcap_uv(view_normal: vec3<f32>) -> vec2<f32> {
    return view_normal.xy * 0.5 + 0.5;
}
```
- This should be adjusted to a perspective correct view space normal
- May need to calculate the view_space position
```
fn compute_matcap_uv(view_position: vec3<f32>, view_normal: vec3<f32>) -> vec2<f32> {
  let inv_depth = 1.0 / (1.0 + view_position.z);
  let proj_factor = -view_position.x * view_position.y * inv_depth;
  let basis1 = vec3(1.0 - view_position.x * view_position.x * inv_depth, proj_factor, -view_position.x);
  let basis2 = vec3(proj_factor, 1.0 - view_position.y * view_position.y * inv_depth, -view_position.y);
  let matcap_uv = vec2(dot(basis1, view_normal), dot(basis2, view_normal));

  return matcap_uv * vec2(0.5, -0.5) + 0.5;
}
```
- Function is provided as above

---

### **[POLISH] PERF: Pack vertex data to reduce memory/bandwidth**

**Status:** Future optimization

**Current State:**
All vertex attributes use f32 components (4 bytes each), resulting in large vertex buffers:
- Position: 3x f32 = 12 bytes
- Normal: 3x f32 = 12 bytes
- UV: 2x f32 = 8 bytes
- Color: 3x f32 = 12 bytes
- Bone indices: 4x u32 = 16 bytes (stored as f32 in shader)
- Bone weights: 4x f32 = 16 bytes

**Proposed Packed Format:**
Use hardware-accelerated packed formats for significant memory savings:

| Attribute    | Current   | Packed       | Savings     | Notes                              |
| ------------ | --------- | ------------ | ----------- | ---------------------------------- |
| Position     | f32x3     | f16x4        | 12→8 bytes  | Last component padded/ignored      |
| Normal       | f32x3     | snorm16x4    | 12→8 bytes  | Normalized to [-1,1], last ignored |
| UV           | f32x2     | unorm16x2    | 8→4 bytes   | Normalized to [0,1]                |
| Vertex color | f32x3     | unorm8x4     | 12→4 bytes  | Standard RGBA8                     |
| Bone indices | u32x4     | uint8x4      | 16→4 bytes  | Max 256 bones                      |
| Bone weights | f32x4     | unorm8x4     | 16→4 bytes  | Normalized to [0,1]                |

**Example Savings:**
- POS_UV_NORMAL: 32 bytes → 20 bytes (37% reduction)
- POS_UV_NORMAL_COLOR_SKINNED: 76 bytes → 32 bytes (58% reduction!)

**Benefits:**
- Reduced VRAM usage (important for low-end GPUs)
- Faster vertex fetch (less memory bandwidth)
- Authentic PS1/N64 precision (f16 positions match era)
- GPU automatically unpacks to f32 in shader (zero cost)

**Implementation Plan:**
1. Update `VertexFormatInfo` in `vertex.rs` with packed formats
2. Modify vertex buffer layout descriptors
3. Update FFI to accept packed data (or pack on upload)
4. Test precision loss is acceptable for retro aesthetic
5. Update examples to use new vertex formats

**Considerations:**
- f16 position precision: ±65504 range, good for typical game worlds
- snorm16 normal precision: 1/32767 ≈ 0.00003 angular precision (overkill)
- May need to adjust vertex data generation in examples

---

### **[POLISH] PERF: Store bone matrices as 3x4 instead of 4x4**

**Status:** Future optimization (dependent on GPU skinning)

**Current State:**
- Bone matrices stored as `mat4x4<f32>` (16 floats = 64 bytes each)
- 4th row always `[0, 0, 0, 1]` for affine transforms (wasted space)
- Storage buffer: `array<mat4x4<f32>, 256>` = 16 KB

**Proposed Optimization:**
Store as 3x4 matrices (row-major):
```wgsl
// CPU side: Upload as [f32; 12] per bone (48 bytes)
// GPU side: Reconstruct 4x4 in shader
struct BoneMatrix3x4 {
    row0: vec4<f32>,  // [m00, m01, m02, m03]
    row1: vec4<f32>,  // [m10, m11, m12, m13]
    row2: vec4<f32>,  // [m20, m21, m22, m23]
    // row3 is implicitly [0, 0, 0, 1]
}

fn expand_bone_matrix(bone: BoneMatrix3x4) -> mat4x4<f32> {
    return mat4x4<f32>(
        bone.row0.xyz, 0.0,
        bone.row1.xyz, 0.0,
        bone.row2.xyz, 0.0,
        bone.row0.w, bone.row1.w, bone.row2.w, 1.0
    );
}
```

**Savings:**
- Per bone: 64 bytes → 48 bytes (25% reduction)
- 256 bones: 16 KB → 12 KB (4 KB saved)
- GPU memory bandwidth reduced by 25% during skinning

**Benefits:**
- Standard practice in production engines (UE, Unity use 3x4)
- Negligible shader cost (one-time reconstruction per vertex)
- Allows more bones or higher vertex counts within bandwidth budget

**Implementation:**
1. Update `set_bones()` FFI to accept 12 floats per bone
2. Change storage buffer layout in shaders
3. Add expand_bone_matrix() helper in skinning code
4. Update skinned-mesh example to provide 3x4 data

---

### **[POLISH] PERF: Store MeshId, TextureId (and other ID)s in as a Vec<T> instead of a HashMap<usize, T>
- This task may need to be updated if ZGraphics is refactored to something else.
- Assets can never be unloaded
- Keys are always inserted via incrementing values
- No reason to waste CPU cycles with Hashing
- "Fallack" error mesh at index 0 will not cause out of bounds issues.


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
7. **spatial-audio** - A sound source rotating around a "listener", and audio pans around the object

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
