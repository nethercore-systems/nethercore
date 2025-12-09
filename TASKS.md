# Emberware Development Tasks

**Architecture Overview:** See [CLAUDE.md](./CLAUDE.md) for framework design and Console trait details.


## TODO

### **[PERFORMANCE FEATURE] Implement Pipeline Caching
- Small speed up when launching the program

### **[Feature] Add UV-enabled procedural shapes**
- Add new FFI commands for UV enabled procedural shapes. 
- Use a smart mapping, like a UV sphere, or a Box cube (not dice faces)
- These must be different functions, in addition to the provided position + normal ones, these should have position + uv + normals

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
   - **Mode 3 (Hybrid)**: Specular-Shininess Blinn-Phong, dynamic lights

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

### Mode 3: Blinn-Phong (Specular-Shininess)
[Retro workflow, lighting model]

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
- ❌ **Mode 1 (Matcap)** example - No matcap rendering demo, no demo of multiply/add/hsv modulate
- ❌ **Mode 3 (SS BP)** example - No demo
- ❌ **Custom fonts** - No font loading demo

**Recommended New Examples:**
1. **matcap-showcase** - All 3 matcap slots with blend modes (mode 1)
2. **unlit-lowpoly** - PS1-style low-poly with vertex colors (mode 0)
3. **hybrid-character** - Character with matcap ambient + PBR lighting (mode 3)
4. **audio-test** - Sound effects, music, channels (CRITICAL - audio undocumented!)
5. **custom-font** - Load bitmap font, render styled text
6. **particle-system** - Hundreds of sprites, blend modes
7. **spatial-audio** - A sound source rotating around a "listener", and audio pans around the object
8. **rendering mode examples** - Show different scenes in from various rendering modes. Explain the material system (textures and channels)

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


