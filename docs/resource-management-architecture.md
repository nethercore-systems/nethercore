# Resource Management Architecture Analysis

## Executive Summary

After comprehensive codebase analysis and fixing the billboard texture mapping bug, **the current architecture is actually correct and clean**. The bug was caused by a redundant `z_state.texture_map` field that caused lookups in the wrong place. The fix (removing the redundant field) represents the proper architectural solution.

**Key Finding**: `GameSession` should own FFI→GPU mapping tables, which is exactly what the current architecture does.

## The Texture Mapping Bug (RESOLVED)

### What Happened
- Billboards were not rendering (47 quad instances created, all with `TextureHandle(0)` INVALID)
- Root cause: Code at `graphics/mod.rs:945-960` was using `z_state.texture_map.get()` to map textures
- Problem: `z_state.texture_map` was empty (never populated)
- Solution: The correct mapping was in `session.texture_map` passed as a parameter to `process_draw_commands()`

### The Fix
1. **Removed** redundant `z_state.texture_map` field from `state.rs:146`
2. **Updated** quad processing to use `texture_map` parameter (from session) instead of `z_state.texture_map`
3. **Changed** FFI functions to store `INVALID` placeholder handles (3 locations in ffi/mod.rs)
4. **Added** texture remapping in `process_draw_commands()` to translate INVALID → actual GPU handles

### Why This is the Right Architecture
Having `texture_map` only in `GameSession` (not duplicated in `ZFFIState`) ensures:
- Single source of truth for FFI→GPU mappings
- No ambiguity about which map to use
- Proper lifetime matching (mappings live as long as the game session)
- Clean separation of concerns

## Architecture Overview: Two-Handle System

Emberware uses a two-handle system to isolate game code from graphics internals:

```
┌─────────────────────┐         ┌─────────────────────┐
│   Game (WASM)       │         │  Graphics (GPU)     │
│                     │         │                     │
│  FFI Handle (u32)   │────────▶│  GPU Handle (typed) │
│  - texture_create() │  maps   │  - TextureHandle    │
│  - mesh_create()    │  ─────▶ │  - MeshHandle       │
└─────────────────────┘         └─────────────────────┘
         ▲                               ▲
         │                               │
         │      ┌─────────────────┐      │
         │      │  GameSession    │      │
         └──────│  - texture_map  │──────┘
                │  - mesh_map     │
                └─────────────────┘
                   Translation Layer
```

### Why Two Handle Spaces?
1. **Game isolation**: WASM code never sees GPU internals
2. **Validation**: Session can validate handles before GPU access
3. **Safety**: Invalid FFI handles can't corrupt GPU state
4. **Flexibility**: GPU implementation can change without affecting game API

## Component Ownership & Responsibilities

### 1. GameSession (`app.rs:79-86`)
**Lifetime**: One game's execution (from load until game exit)

**Owns**:
```rust
pub struct GameSession {
    pub runtime: Runtime<EmberwareZ>,
    texture_map: HashMap<u32, TextureHandle>,  // FFI → GPU texture mapping
    mesh_map: HashMap<u32, MeshHandle>,        // FFI → GPU mesh mapping
}
```

**Responsibilities**:
- Owns authoritative FFI→GPU mapping tables
- Processes pending resource requests from ZFFIState
- Translates game handles to GPU handles during rendering
- Lifetime matches game session (resources valid until game unloads)

### 2. ZGraphics (`graphics/mod.rs`)
**Lifetime**: Entire application (persists across game switches)

**Owns**:
```rust
pub struct ZGraphics {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_manager: TextureManager,      // Actual GPU textures
    buffer_manager: BufferManager,        // Actual GPU buffers/meshes
    pipeline_cache: PipelineCache,
    // ... shaders, command buffer, etc.
}
```

**Responsibilities**:
- Owns actual GPU resources (textures, buffers, pipelines)
- Provides resource creation methods (`load_texture()`, `upload_retained_mesh()`)
- Returns typed GPU handles (TextureHandle, MeshHandle)
- Completely isolated from FFI handle space
- **Issue**: Resources accumulate across game sessions (potential memory leak)

### 3. ZFFIState (`state.rs`)
**Lifetime**: Shared across game, cleared each frame

**Owns**:
```rust
pub struct ZFFIState {
    // Per-frame resource staging
    pending_textures: Vec<PendingTexture>,
    pending_meshes: Vec<PendingMesh>,

    // Per-frame drawing state
    bound_textures: [u32; 4],           // Current FFI texture bindings (slots 0-3)
    quad_instances: Vec<QuadInstance>,   // Accumulated this frame

    // Mesh metadata (for FFI queries)
    mesh_map: HashMap<u32, RetainedMesh>, // FFI handle → mesh info

    // Transform & shading state
    current_transform: Mat4,
    current_shading_state: ShadingState,
    shading_states: Vec<ShadingState>,

    // Command recording
    // (commands stored in ZGraphics.command_buffer)
}
```

**Responsibilities**:
- Stages resource creation requests (pending_textures, pending_meshes)
- Tracks current bindings and state for FFI calls
- Accumulates per-frame data (quad instances, shading states)
- Cleared after each frame
- **Note**: `mesh_map` here stores metadata (vertex/index counts, format) for FFI queries, not GPU handles

## Complete Resource Lifecycle

### Texture Loading Flow
```
1. Game calls texture_create(w, h, data)
   ↓
2. FFI function (ffi/mod.rs:1426-1459)
   - Generates unique handle (next_texture_handle)
   - Stages PendingTexture in z_state.pending_textures
   - Returns u32 handle to game
   ↓
3. App processes pending resources (app.rs:211-226)
   - For each z_state.pending_textures:
     - graphics.load_texture(width, height, data)
     - Receives TextureHandle (GPU handle)
     - session.texture_map.insert(ffi_handle, gpu_handle)
   - Clears pending_textures
   ↓
4. Game sets texture binding: texture_bind(handle)
   - FFI stores in z_state.bound_textures[slot] = handle
   ↓
5. Game issues draw: draw_billboard(...)
   - FFI creates command with INVALID placeholder textures
   ↓
6. Rendering: process_draw_commands()
   - Remaps INVALID → actual GPU handles using session.texture_map
   - Looks up z_state.bound_textures[0..3] in session.texture_map
   - Replaces placeholder with actual TextureHandle
   ↓
7. GPU execution
   - Bind actual GPU texture to pipeline
   - Render
```

### Why Delayed Mapping?
FFI functions don't have access to `session.texture_map` (different module, lifetime issues). Solution:
1. FFI stores `INVALID` placeholders in commands
2. `process_draw_commands()` has both `z_state` (bound FFI handles) and `texture_map` (FFI→GPU mapping)
3. Remapping happens just before GPU submission

## Architectural Recommendations

### ✅ KEEP: Session Owns Mappings
**Current**: `GameSession` has `texture_map` and `mesh_map`
**Recommendation**: Keep this design - it's correct

**Rationale**:
- Lifetime alignment: Mappings should live as long as game session
- Single source of truth: No ambiguity about where to look up handles
- Clean separation: Graphics layer doesn't know about FFI handles

### ✅ KEEP: Two-Phase Resource Loading
**Current**: FFI stages requests → App processes → Graphics uploads
**Recommendation**: Keep this design

**Rationale**:
- FFI can't directly access graphics (lifetime/borrowing issues)
- Batching opportunity (process all pending resources together)
- Error handling centralized in App

### ✅ KEEP: Delayed Texture Mapping
**Current**: FFI stores INVALID placeholders → process_draw_commands() remaps
**Recommendation**: Keep this design

**Rationale**:
- FFI functions don't have session.texture_map access
- Remapping at render time has all context available
- Avoids threading texture_map through FFI layer

### ⚠️ CONSIDER: z_state.mesh_map Redundancy
**Current**: `z_state.mesh_map: HashMap<u32, RetainedMesh>` stores mesh metadata
**Issue**: Potential redundancy with `session.mesh_map`

**Analysis**:
```rust
// Session owns FFI→GPU mapping
session.mesh_map: HashMap<u32, MeshHandle>  // u32 → MeshHandle

// ZFFIState owns FFI→metadata mapping
z_state.mesh_map: HashMap<u32, RetainedMesh> // u32 → (format, vert_count, etc.)
```

**Use cases for z_state.mesh_map**:
- FFI functions might query mesh properties (format, vertex count)
- `draw_mesh()` might need mesh data to construct commands

**Recommendation**: Keep if FFI needs mesh metadata queries. If not used, remove.

### 🔴 FIX REQUIRED: Resource Cleanup on Game Exit
**Issue**: `ZGraphics` persists across game sessions, accumulating resources

**Problem**:
```rust
// Game A creates 100 textures → stored in texture_manager
// User exits Game A, starts Game B
// Game A's 100 textures still in memory (leaked)
```

**Recommendation**: Add resource cleanup when game session ends

**Solution Options**:

**Option A**: Track resources by session
```rust
pub struct ZGraphics {
    texture_manager: TextureManager,
    // Add session-aware tracking
    session_resources: HashMap<SessionId, Vec<TextureHandle>>,
}

// On game exit:
graphics.cleanup_session(session_id);
```

**Option B**: Session owns GPU handles, drops them on exit
```rust
pub struct GameSession {
    texture_map: HashMap<u32, OwnedTextureHandle>, // Owns handle, drops on exit
    mesh_map: HashMap<u32, OwnedMeshHandle>,
}
```

**Option C**: Reference counting
```rust
// TextureHandle becomes Arc<Texture>
// Automatically freed when last reference (from session.texture_map) drops
```

**Recommended**: Option B (simplest) or Option C (cleanest)

### 🔴 FIX REQUIRED: Shader Compilation Must Panic
**User Requirement**: "NO SILENT FAILURES. shader compilation -> should PANIC."

**Current**: Shader compilation errors might be logged but not panic
**Recommendation**: Add `.expect()` or `.unwrap()` on shader compilation

**Location**: Check `shader_gen.rs` and pipeline creation

## Summary: Current State Assessment

### What's Working Well ✅
1. **Two-handle system**: Clean separation between game FFI handles and GPU handles
2. **Session owns mappings**: Single source of truth for FFI→GPU translation
3. **Delayed resource loading**: FFI stages requests, App processes them
4. **Delayed texture mapping**: FFI uses placeholders, rendering remaps them

### What Was Fixed 🔧
1. **Removed redundant z_state.texture_map**: Was causing lookups in wrong place
2. **Billboard rendering**: Now correctly uses session.texture_map via parameter

### What Needs Attention ⚠️
1. **Resource cleanup**: GPU resources accumulate across game sessions (memory leak)
2. **Shader compilation panics**: User wants compilation errors to panic, not fail silently
3. **z_state.mesh_map evaluation**: Determine if it's truly needed or redundant

### Architectural Health: 8/10
The core architecture is sound. The texture mapping bug was due to a redundant field, not a fundamental design flaw. Main gaps are resource lifecycle management (cleanup on game exit) and error handling policy (shader panics).

## Next Steps

1. **Immediate**: Verify billboard fix works (test billboard example) ✅ DONE
2. **Short-term**: Add shader compilation panic checks
3. **Short-term**: Evaluate z_state.mesh_map necessity
4. **Medium-term**: Implement resource cleanup on game session exit
5. **Long-term**: Consider reference-counted GPU resources for automatic cleanup
