## Mesh data is not being cleared correctly between games
- Load one game, it renders fine, close it.
- Open another game, meshes may be messed up.
- It doesn't happen 100% of the time, but when it does, the mesh just doesn't show. This usually happens during the Procedural Meshes example.

**Root Cause:** `ZGraphics` persists across game sessions (intentional, for GPU reinitialization cost), but mesh/texture resources are never cleared:
- `BufferManager::retained_meshes` HashMap accumulates across games
- `BufferManager::next_mesh_id` counter is never reset
- GPU buffer `used` counters are never reset (new data appends to old)
- `TextureManager` textures persist across games

**Proposed Fix:** Clear-on-Init pattern (clear resources when starting a new game, not when exiting)

Why clear-on-init is better than clear-on-exit:
- Survives game crashes during init (next game clears stale state)
- Survives failed init → retry scenarios
- Single cleanup point in `start_game()`
- First game load is harmless no-op

Implementation:
1. Add `ZGraphics::clear_game_resources()` in `graphics/mod.rs`
2. Add `TextureManager::clear_game_textures()` in `graphics/texture_manager.rs`
3. Add `BufferManager::clear_game_meshes()` in `graphics/buffer.rs`
4. Add `GrowableBuffer::reset_used()` in `graphics/buffer.rs`
5. Call `graphics.clear_game_resources()` in `app/game_session.rs:start_game()` before GameSession creation

## Hello World example doesn't work anymore
- Nothing is rendered to the screen!
- No text, no "box"
- Likely due to a conflict in how packed vertex's work now between quad renderer and the mesh rendering pipeline, while text rendering was not updated to include this.

**Root Cause:** Unit quad mesh in `init.rs` is uploaded as unpacked f32 data (32-byte stride) but GPU pipeline expects packed data (16-byte stride). GPU reads garbage for UVs/colors.

**Proposed Fix:** Pack unit quad vertices using `pack_vertex_data()` at init time, use `vertex_stride_packed()` for stride calculation.

Implementation in `graphics/init.rs` (lines 211-241):
```rust
// CURRENT (broken):
let vertex_bytes = bytemuck::cast_slice(&unit_quad_vertices);
let stride = vertex_stride(unit_quad_format);

// FIXED:
use crate::graphics::packing::pack_vertex_data;
let packed_vertices = pack_vertex_data(&unit_quad_vertices, unit_quad_format);
let stride = vertex_stride_packed(unit_quad_format);
// ...
retained_vertex_buf_mut.write(&queue, &packed_vertices);
```

## Lighting Example text doesn't render text or UI
- The main sphere renders
- Text and light indicators don't render at all
- Probably due to above issue, ie packed vertex data in the pipeline.
- Also applies to blinn phong example, in fact text doesnt render at all anymore

**Root Cause:** Same as Hello World - unit quad vertex format mismatch.

**Proposed Fix:** Same fix as Hello World.

## Textured Procedural Example is rendering the Default Text
- Likely a texture collision problem
- Default font texture is at index 0, first loaded texture is maybe also going to this address
- This causes the meshes to render with texture 0
- But could also just be a bug with the render pipeline, ie texture ids not being recorded/bound correctly
- Might be better to explore a separate system for text vs textures, such as a FontId and TextureId to prevent this kind of problem in the future.

**Root Cause:** `VRPCommand::Mesh/IndexedMesh` use `texture_slots: [TextureHandle; 4]` initialized to INVALID, with deferred remapping using `z_state.bound_textures` at frame end. But `bound_textures` changes during the frame (e.g., `draw_text()` sets it to font texture 0), so ALL meshes get remapped with wrong texture.

**Proposed Fix:** Unify texture handle pattern - all `VRPCommand` variants store `textures: [u32; 4]` (FFI handles captured at command creation). Resolution to `TextureHandle` happens at render time in `frame.rs`. This matches how `QuadBatch` already works correctly.

Implementation:

1. **Update VRPCommand enum** (`graphics/command_buffer.rs` lines 36-57)
   - Change `Mesh` and `IndexedMesh` variants: `texture_slots: [TextureHandle; 4]` → `textures: [u32; 4]`

2. **Update FFI functions to capture bound_textures** at command creation:
   - `ffi/mesh.rs:draw_mesh()` (line 523): `let textures = state.bound_textures;`
   - `ffi/draw_3d.rs:draw_triangles()` (line 34): capture `bound_textures`
   - `ffi/draw_3d.rs:draw_triangles_indexed()` (line 154): capture `bound_textures`

3. **Update command recording functions** (`graphics/command_buffer.rs`)
   - `record_triangles()` (line 125): accept `textures: [u32; 4]`
   - `record_triangles_indexed()` (line 158): accept `textures: [u32; 4]`
   - `record_mesh()` (line 197): accept `textures: [u32; 4]`

4. **Remove deferred remapping** (`graphics/draw.rs` lines 32-69)
   - Delete the loop that remaps INVALID placeholders using `z_state.bound_textures`

5. **Add render-time resolution** (`graphics/frame.rs` around lines 592-649)
   - When rendering Mesh/IndexedMesh commands, resolve FFI handles to TextureHandle:
   ```rust
   let resolved_textures = [
       texture_map.get(&cmd.textures[0]).copied().unwrap_or(TextureHandle::INVALID),
       // ... slots 1-3
   ];
   ```