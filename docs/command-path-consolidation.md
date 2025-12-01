# Command Path Consolidation

## Current Architecture (Problem)

```
FFI Function (ffi/mod.rs)
    │
    ▼
ZDrawCommand (state.rs)         ← Vec<f32> allocated per draw call
    │                              State fields duplicated
    │
    ▼
process_draw_commands (graphics/mod.rs)
    │                           ← Vec<f32> copied to VirtualRenderPass
    │                              State converted (u8 → enum)
    │                              Handle mapping (u32 → TextureHandle)
    ▼
VRPCommand (command_buffer.rs)
    │
    ▼
GPU Render Loop
```

### Overhead Analysis

**Per `draw_triangles` call:**
1. `Vec<f32>` allocation in ZDrawCommand::DrawTriangles
2. Copy from ZDrawCommand.vertex_data → VirtualRenderPass.vertex_data
3. State field copying (6 fields)
4. Enum conversions: `u8 → CullMode`, `u8 → BlendMode`
5. Handle mapping: `[u32; 4] → [TextureHandle; 4]`

**Estimated overhead:** ~200-500 bytes copied per draw call, plus allocator overhead

---

## Target Architecture (Solution)

```
FFI Function (ffi/mod.rs)
    │
    ▼
VirtualRenderPass.record_*()    ← Direct write to vertex buffers
    │                              No intermediate allocation
    ▼
VRPCommand
    │
    ▼
GPU Render Loop
```

---

## Implementation Plan

### Step 1: Move VirtualRenderPass to ZFFIState

**File: `state.rs`**

Replace:
```rust
pub draw_commands: Vec<ZDrawCommand>,
```

With:
```rust
pub render_pass: VirtualRenderPass,
```

Add import:
```rust
use crate::graphics::VirtualRenderPass;
```

Update `Default` impl and `reset()` method.

### Step 2: Add Recording Methods to VirtualRenderPass

**File: `graphics/command_buffer.rs`**

Add these methods to `impl VirtualRenderPass`:

```rust
/// Record a non-indexed triangle draw (called from FFI)
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    transform: Mat4,
    color: u32,
    depth_test: bool,
    cull_mode: CullMode,
    blend_mode: BlendMode,
    texture_slots: [TextureHandle; 4],
    matcap_blend_modes: [MatcapBlendMode; 4],
) {
    let format_idx = format as usize;
    let stride = vertex_stride(format) as usize;
    let vertex_count = (vertex_data.len() * 4) / stride;
    let base_vertex = self.vertex_counts[format_idx];

    // Write directly to buffer (no intermediate Vec)
    let byte_data = bytemuck::cast_slice(vertex_data);
    self.vertex_data[format_idx].extend_from_slice(byte_data);
    self.vertex_counts[format_idx] += vertex_count as u32;

    self.commands.push(VRPCommand {
        format,
        transform,
        vertex_count: vertex_count as u32,
        index_count: 0,
        base_vertex,
        first_index: 0,
        texture_slots,
        color,
        depth_test,
        cull_mode,
        blend_mode,
        matcap_blend_modes,
    });
}

/// Record an indexed triangle draw (called from FFI)
pub fn record_triangles_indexed(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    index_data: &[u32],
    transform: Mat4,
    color: u32,
    depth_test: bool,
    cull_mode: CullMode,
    blend_mode: BlendMode,
    texture_slots: [TextureHandle; 4],
    matcap_blend_modes: [MatcapBlendMode; 4],
) {
    let format_idx = format as usize;
    let stride = vertex_stride(format) as usize;
    let vertex_count = (vertex_data.len() * 4) / stride;
    let base_vertex = self.vertex_counts[format_idx];
    let first_index = self.index_counts[format_idx];

    // Write directly to buffers
    let byte_data = bytemuck::cast_slice(vertex_data);
    self.vertex_data[format_idx].extend_from_slice(byte_data);
    self.vertex_counts[format_idx] += vertex_count as u32;

    self.index_data[format_idx].extend_from_slice(index_data);
    self.index_counts[format_idx] += index_data.len() as u32;

    self.commands.push(VRPCommand {
        format,
        transform,
        vertex_count: vertex_count as u32,
        index_count: index_data.len() as u32,
        base_vertex,
        first_index,
        texture_slots,
        color,
        depth_test,
        cull_mode,
        blend_mode,
        matcap_blend_modes,
    });
}

/// Record a mesh draw (called from FFI)
pub fn record_mesh(
    &mut self,
    mesh_format: u8,
    mesh_vertex_count: u32,
    mesh_index_count: u32,
    mesh_vertex_offset: u64,
    mesh_index_offset: u64,
    transform: Mat4,
    color: u32,
    depth_test: bool,
    cull_mode: CullMode,
    blend_mode: BlendMode,
    texture_slots: [TextureHandle; 4],
    matcap_blend_modes: [MatcapBlendMode; 4],
) {
    let stride = vertex_stride(mesh_format) as u64;
    let base_vertex = (mesh_vertex_offset / stride) as u32;
    let first_index = if mesh_index_count > 0 {
        (mesh_index_offset / 4) as u32
    } else {
        0
    };

    self.commands.push(VRPCommand {
        format: mesh_format,
        transform,
        vertex_count: mesh_vertex_count,
        index_count: mesh_index_count,
        base_vertex,
        first_index,
        texture_slots,
        color,
        depth_test,
        cull_mode,
        blend_mode,
        matcap_blend_modes,
    });
}
```

### Step 3: Add Handle Mapping Helpers

The challenge is that FFI uses game handles (`u32`) but VRPCommand uses graphics handles (`TextureHandle`). We need mapping at record time.

**Option A: Pass texture_map to FFI context**

Add to `ZFFIState`:
```rust
pub texture_map: HashMap<u32, TextureHandle>,
pub mesh_map: HashMap<u32, MeshHandle>,
```

These are populated when textures/meshes are loaded and used during recording.

**Option B: Defer mapping (keep u32 in VRPCommand)**

Change VRPCommand to use `[u32; 4]` for texture slots, map during render. This is simpler but adds work to render loop.

**Recommendation: Option A** - Map at record time, cleaner render loop.

### Step 4: Update FFI Functions

**File: `ffi/mod.rs`**

Change `draw_triangles`:

```rust
fn draw_triangles(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    float_count: u32,
    format: u32,
) {
    // ... validation code stays the same ...

    // Read vertex data from WASM memory
    let vertex_data: Vec<f32> = /* existing code */;

    let state = &mut caller.data_mut().console;

    // Map texture handles
    let texture_slots = [
        state.texture_map.get(&state.bound_textures[0]).copied().unwrap_or(TextureHandle::INVALID),
        state.texture_map.get(&state.bound_textures[1]).copied().unwrap_or(TextureHandle::INVALID),
        state.texture_map.get(&state.bound_textures[2]).copied().unwrap_or(TextureHandle::INVALID),
        state.texture_map.get(&state.bound_textures[3]).copied().unwrap_or(TextureHandle::INVALID),
    ];

    let matcap_blend_modes = [
        convert_matcap_blend_mode(state.matcap_blend_modes[0]),
        convert_matcap_blend_mode(state.matcap_blend_modes[1]),
        convert_matcap_blend_mode(state.matcap_blend_modes[2]),
        convert_matcap_blend_mode(state.matcap_blend_modes[3]),
    ];

    // Record directly to VirtualRenderPass
    state.render_pass.record_triangles(
        format,
        &vertex_data,
        state.current_transform,
        state.color,
        state.depth_test,
        convert_cull_mode(state.cull_mode),
        convert_blend_mode(state.blend_mode),
        texture_slots,
        matcap_blend_modes,
    );
}
```

### Step 5: Update Graphics Processing

**File: `graphics/mod.rs`**

Remove or simplify `process_draw_commands()`:

```rust
pub fn process_draw_commands(
    &mut self,
    z_state: &mut ZFFIState,
    _texture_map: &HashMap<u32, TextureHandle>,  // No longer needed
    _mesh_map: &HashMap<u32, MeshHandle>,        // No longer needed
) {
    // Apply init config
    self.set_render_mode(z_state.init_config.render_mode);

    // Swap render pass (take ownership, give empty one back)
    std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

    // Process special commands that need graphics context
    // (billboards, sprites, text - these generate geometry)
    // ... keep the billboard/sprite/text/sky processing ...
}
```

### Step 6: Delete ZDrawCommand

**File: `state.rs`**

Remove the entire `ZDrawCommand` enum (lines 147-234).

---

## Files to Modify

| File | Changes |
|------|---------|
| `state.rs` | Replace `draw_commands: Vec<ZDrawCommand>` with `render_pass: VirtualRenderPass`, delete ZDrawCommand enum |
| `graphics/command_buffer.rs` | Add `record_triangles()`, `record_triangles_indexed()`, `record_mesh()` methods |
| `ffi/mod.rs` | Update `draw_triangles`, `draw_triangles_indexed`, `draw_mesh` to call render_pass.record_*() |
| `graphics/mod.rs` | Simplify `process_draw_commands()` - remove ZDrawCommand matching, just swap render_pass |
| `app.rs` | Update calls to pass render_pass instead of draw_commands |

---

## Special Cases: Billboards, Sprites, Text

These commands generate geometry at render time (need camera info for billboards, screen size for sprites). Keep them as enum variants but in a smaller enum:

```rust
pub enum DeferredCommand {
    DrawBillboard { ... },
    DrawSprite { ... },
    DrawRect { ... },
    DrawText { ... },
    SetSky { ... },
}
```

These are processed in `process_draw_commands()` and generate VRPCommands there.

---

## Testing Checklist

1. [ ] `cargo check` passes
2. [ ] `cargo test` passes
3. [ ] Visual: Draw triangles with various formats
4. [ ] Visual: Draw indexed triangles
5. [ ] Visual: Draw retained meshes
6. [ ] Visual: Billboards face camera correctly
7. [ ] Visual: Sprites render at correct screen positions
8. [ ] Visual: Text renders correctly
9. [ ] Performance: Reduced allocations per frame (measure with logging)

---

## Estimated Effort

- Step 1-2: 30 minutes
- Step 3: 15 minutes
- Step 4: 1 hour (multiple FFI functions)
- Step 5: 30 minutes
- Step 6: 5 minutes
- Testing: 1 hour

**Total: ~3-4 hours**
