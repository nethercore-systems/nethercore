# Emberware Development Tasks

---

**Architecture Overview:** See [CLAUDE.md](./CLAUDE.md) for framework design and Console trait details.

---

## In Progress

---

## TODO

---

### **[CRITICAL] STABILITY Codebase is huge and clunky**
- Lots of files are extremely long (2k+)
- We must go through the entire repository and clean this up. Some ways we can accomplish this are
1. Refactor heavily duplicated code to prevent copy paste
2. Split files into smaller focused ones.
- Any file which is longer than 2000 lines MUST be made smaller, preferrably under 1000 lines each.

### **CRITICAL MISSING FEATURE: Shaders mode1, mode2, and mode3 don't use sky lambert shading **
- Currently, only mode0_unlit.wgsl is properly using sky lambert.
- Lambert shading using sun as a directional light should be implemented for mode1, mode2, and mode3 as well.
```
// Simple Lambert shading using sky sun (when normals available)
fn sky_lambert(normal: vec3<f32>) -> vec3<f32> {
    let n_dot_l = max(0.0, dot(normal, sky.sun_direction.xyz));
    let direct = sky.sun_color_and_sharpness.xyz * n_dot_l;
    let ambient = sample_sky(normal) * 0.3;
    return direct + ambient;
}
```
- Function already exists and defined as in mode0_unlit.wgsl (same as above)
- Implement and include this lambert shading for the mode1, mode2, mode3 shaders.
- CAUTION: Actual shaders are generated via shader_gen.rs, hence the string tags like //VS_POSITION around the functions. These tags will be replaced with relevant shader code to implement the actual variants that are used at runtime.

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

### **CRITICAL PERFORMANCE TASK: Optimize Render Loop & Reduce Idle GPU Usage (WGPU + Egui)**

**Context**

Our application currently consumes **~30% GPU** even when idle (e.g., in the game library UI).
Profiling and review of the render loop indicate **multiple systemic issues** that cause unnecessary GPU work each frame:

* Multiple command encoders and command buffer submissions per frame
* Possible double presentation of the surface texture
* Game rendering and UI rendering split across separate encoders
* Forced maximum-framerate redraw loop (`window.request_redraw()` every frame)
* Egui meshes and buffers rebuilt every frame even when unchanged
* Potential redundant clear/load passes
* CPU-side debug/UI work performed even when invisible

These collectively cause high GPU load when nothing is happening.

---

**Goal**

Reduce idle GPU utilization in the library UI from ~30% → **<5%** (target matching typical WGPU idle loads) by restructuring the rendering pipeline.

---

**Action Plan**

**1. Unify GPU work into a single CommandEncoder per frame**

Currently:

* `graphics.render_frame()` likely creates its own encoder + submit
* UI rendering creates another encoder + submit

Required:

* Main frame loop creates **ONE** `CommandEncoder`
* Pass this encoder to all rendering subsystems
* Remove internal submits from `graphics.render_frame()`
* Game rendering becomes a *pass* inside this single encoder
* UI/egui rendering becomes another pass inside the same encoder
* Only the main render loop may call `queue.submit()`

**Deliverables:**

* Updated API for `graphics.render_frame(&mut encoder, &view, ...)`

**2. Ensure exactly ONE surface texture present() per frame**

Currently the surface texture may be used or presented by the game renderer.
Required:

* The main render loop obtains the surface texture
* Pass the texture view into game and UI render passes
* Only the main render loop calls `surface_texture.present()`

**3. Remove per-frame forced redraw (`window.request_redraw()`)**

Currently the loop redraws at maximum (hundreds/thousands FPS).
Required:

* Redraw only when:

  * Input events arrive
  * Game is running (Playing mode)
  * Egui reports needing animation
* Implement Winit redraw scheduling (not forced loops)

*4. Avoid rebuilding Egui meshes/buffers every frame when static**

Currently we always do:

```rust
let tris = ctx.tessellate(...)
egui_renderer.update_buffers(...)
```

Even when no UI changed.
Required:

* Only update buffers when `full_output.shapes` or textures changed
* Cache previous meshes and reuse when unchanged

5. Reduce redundant CPU-side debug/UI work**

Examples:

* Cloning `frame_times` every frame
* Repainting debug graphs constantly
* Rebuilding local vectors

Required:

* Update debug UI only when overlay is visible
* Avoid cloning large vectors constantly
* Use ring-buffer references instead of copies

**6. Confirm render passes do not double-clear the frame**

* Game renderer may clear the screen
* UI pass may clear again (when not in Play mode)

Required:

* Only clear once per frame
* If game rendered first, UI must use `LoadOp::Load`

**Acceptance Criteria**

A pull request is complete when all of the following are true:

Rendering Architecture

* [ ] Only **one** `CommandEncoder` is created per frame
* [ ] Only **one** `queue.submit()` is called per frame
* [ ] Only **one** `surface_texture.present()` is called per frame
* [ ] Game renderer no longer manages its own encoder or present
* [ ] Game and UI rendering occur sequentially inside the same encoder

Egui Improvements

* [ ] Egui meshes are only rebuilt when shapes change
* [ ] Egui buffers updated only when mesh changes
* [ ] UI redraw is event-driven, not frame-driven

Performance

* [ ] Idle GPU usage in library mode is reduced from ~30% to **<5%** on midrange hardware
* [ ] No visual glitches (tests: Library UI → Settings → Playing → back to Library)

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

### **[FEATURE] Direct game launch via command-line argument**

**Status:** Not yet implemented

**Current State:**
- `cargo run` always launches to the game library UI
- Users must click on a game to launch it
- No way to directly launch a specific game from command line

**What's Needed:**
Add command-line argument support to launch games directly, skipping the library screen.

**Usage Examples:**
```bash
cargo run platformer    # Launch platformer directly
cargo run cube          # Launch cube example directly
cargo run -- lighting   # Alternative syntax
```

**Implementation Plan:**

1. **Parse command-line arguments** in `main.rs`:
   - Check for first argument after program name
   - If argument provided, treat as game name
   - If no argument, launch library as normal

2. **Game name resolution**:
   - Match argument against game IDs in library
   - Support both full game IDs and partial matches (e.g., "platform" matches "platformer")
   - Show error if game not found or multiple matches

3. **Direct launch flow**:
   - Skip `UiMode::Library` and go straight to `UiMode::Loading`
   - Use provided game name to construct ROM path
   - Load and run game immediately

4. **Error handling**:
   - Game not found: Print error and show library
   - ROM missing: Print error and show library
   - Invalid game name: Show available games and exit

**Files to Modify:**
- `emberware-z/src/main.rs` - Parse command-line args, implement game resolution logic
- `emberware-z/src/app.rs` - Support initial mode as Loading instead of Library

**User Benefit:**
- Faster iteration during development (no UI navigation)
- Scriptable game launching for testing
- Better developer experience for example testing

---

### **[POLISH] PERF: Store MeshId, TextureId (and other ID)s in ZGraphics as a Vec<T> instead of a HashMap<usize, T>
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
let quad_indices: &[u16] = &[0, 1, 2, 0, 2, 3];
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
