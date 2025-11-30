Console-Agnostic Refactoring Walkthrough
Goal
Decouple core/src/wasm from Emberware Z (3D console) to support future 2D consoles.

Changes Made
1. Created ZState in Emberware Z ✅
File: 
emberware-z/src/state.rs

Moved ALL Z-specific rendering types from core:

CameraState - 3D camera (position, target, FOV)
LightState - PBR lighting
ZDrawCommand - 3D draw commands (DrawTriangles, DrawMesh, SetSky)
PendingTexture, PendingMesh - Resource loading
ZInitConfig - Z-specific init config (render mode)
ZState - Complete Z rendering state (300+ lines)
2. Stripped GameState to Minimal Core ✅
File: 
core/src/wasm/state.rs

Before: 210 lines with camera, transforms, render state, draw commands
After: 120 lines - ONLY core gameplay state

pub struct GameState<I: ConsoleInput> {
    pub memory: Option<Memory>,
    pub tick_count: u64,
    pub input_prev: [I; MAX_PLAYERS],
    pub input_curr: [I; MAX_PLAYERS],
    pub rng_state: u64,
    pub save_data: [Option<Vec<u8>>; MAX_SAVE_SLOTS],
    // NO camera, render_state, transforms!
}
Created wrapper:

pub struct GameStateWithConsole<I, S> {
    pub game: GameState<I>,      // Core
    pub console: S,               // Z-specific rendering
}
3. Updated Console Trait ✅
File: 
core/src/console.rs

Added State associated type:

pub trait Console {
    type Input: ConsoleInput;
    type State: Default + Send + 'static;  // NEW
    
    fn register_ffi(
        &self,
        linker: &mut Linker<GameStateWithConsole<Self::Input, Self::State>>,
    ) -> Result<()>;
}
Removed to_input_state() from ConsoleInput trait (no longer needed).

4. Deleted Old Core Files ✅
Removed from core/src/wasm/:

❌ camera.rs → moved to emberware-z/src/state.rs
❌ render.rs → moved to emberware-z/src/state.rs
❌ draw.rs → moved to emberware-z/src/state.rs
❌ input.rs → deleted (replaced by generic I)
5. Updated EmberwareZ Console Impl ✅
File: 
emberware-z/src/console.rs

impl Console for EmberwareZ {
    type Input = ZInput;
    type State = ZState;  // NEW
    
    fn register_ffi(&self, linker: &mut Linker<GameStateWithConsole<ZInput, ZState>>) {
        crate::ffi::register_z_ffi(linker)?;
    }
}
Completed Steps
6. Update ALL FFI Functions ✅
Files: emberware-z/src/ffi/mod.rs

All FFI functions updated to use GameStateWithConsole<ZInput, ZState>.
Functions access Z-specific state via caller.data_mut().console.
Fixed imports - removed old core types, imported from crate::state.
Replaced all render_state.field with direct field access on ZState.
Added MAX_TRANSFORM_STACK constant to state.rs.

7. Update GameInstance/WasmEngine ✅
Files: core/src/wasm/mod.rs, emberware-z/src/app.rs

Added console_state_mut() and console_state() methods to GameInstance.
Updated app.rs to use console_state_mut() for accessing ZState.
Fixed all references to pending resources, draw commands, camera, etc.

8. Fix Graphics Backend ✅
Updated app.rs to properly separate core state and console state.
Graphics now consumes ZState fields via console_state_mut().

Status: COMPLETE ✅ + ARCHITECTURE REFINED ✅

The refactor is fully working with a clean staging area pattern!

## Architecture: Option A (Staging Area Pattern)

**ZFFIState** (FFI staging) → **ZGraphics** (GPU execution)

1. **FFI functions write to ZFFIState** - Draw commands, transforms, render state
2. **ZGraphics consumes ZFFIState** - Processes commands and executes on GPU
3. **ZFFIState is cleared each frame** - It's ephemeral, not part of rollback
4. **ZGraphics owns GPU resources** - Textures, meshes, buffers, pipelines

### Key Files Changed
- **core/src/wasm/state.rs** - Minimal GameState + GameStateWithConsole wrapper
- **core/src/console.rs** - Added State associated type (documented as FFI staging)
- **core/src/wasm/mod.rs** - Added console_state_mut() and console_state() methods
- **emberware-z/src/state.rs** - Renamed ZState → ZFFIState (clarifies it's staging)
- **emberware-z/src/ffi/mod.rs** - All FFI functions write to ZFFIState
- **emberware-z/src/app.rs** - Consumes FFI state, clears after each frame
- **emberware-z/src/graphics/mod.rs** - Owns actual GPU state and resources
- **core/src/ffi.rs** - Generic common FFI over GameStateWithConsole<I, S>

### Data Flow

```
Game WASM
  ↓ FFI calls (draw_triangle, set_color, etc.)
ZFFIState (staging)
  ↓ app.rs executes commands
ZGraphics (GPU)
  ↓ wgpu rendering
GPU / Screen
```

**After each frame**: `z_state.clear_frame()` - ready for next frame's commands

### Why This Matters

**Before**: Core assumed 3D rendering → couldn't add 2D consoles
**After**: Core is truly agnostic → any console can plug in

**Key Insight**: FFI State is NOT rolled back! Only GameState (input, memory, RNG, saves) is rolled back. This makes rollback simpler and faster.

### Example - Future 2D Console

```rust
/// FFI staging for 2D console (no camera, no 3D!)
pub struct ClassicFFIState {
    pub palette: [Color; 256],
    pub sprite_commands: Vec<SpriteCommand>,
    pub tilemap_updates: Vec<TilemapUpdate>,
}

impl Console for EmberwareClassic {
    type Input = ClassicInput;
    type State = ClassicFFIState;  // Just 2D commands!
    type Graphics = ClassicGraphics;  // Owns 2D GPU state
}
```