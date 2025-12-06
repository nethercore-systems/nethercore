# Emberware Console Architecture Refactoring Plan

## Executive Summary

**Problem:** The current architecture will require ~480 lines of duplicated code per console (app.rs event loop, egui integration, encoder management) when building Emberware Classic, Y, X, etc.

**Root Cause:** The Console trait abstracts the *components* (Graphics, Audio, Input) but not the *application framework*. Each console binary reimplements the entire application layer.

**Solution:** Extract the application framework into core as a parameterized `ConsoleApp<C: Console>` that handles windowing, event loop, egui, and rendering orchestration. Console binaries reduce to ~100 lines: instantiate console + run framework.

**Impact:**
- **Code reduction:** 1,300 lines → 100 lines per console binary
- **Consistency:** All consoles get performance optimizations automatically
- **Maintainability:** Bug fixes and features propagate to all consoles
- **Flexibility:** Consoles can still override framework behavior if needed

---

## Current Architecture Analysis

### What Exists in Core (Shared Infrastructure)

```
emberware-core/
├── Console trait (abstract interface)
├── Runtime<C> (game loop orchestration)
├── WasmEngine (WASM execution)
├── GGRS integration (rollback netcode)
├── GameState<I> (core execution state)
├── GameInstance<I, S> (loaded game wrapper)
├── Common FFI (timing, RNG, saves, multiplayer)
└── Input abstraction (RawInput → Console::Input)
```

**Strengths:**
- ✅ Generic over Console type
- ✅ Rollback works with any Input/State types
- ✅ FFI registration is extensible
- ✅ WASM execution is shared

**Weaknesses:**
- ❌ No application framework (event loop, windowing, UI)
- ❌ Each console reimplements app.rs (~1,300 lines)
- ❌ Egui integration duplicated per console
- ❌ Rendering orchestration duplicated per console

### What Gets Duplicated Per Console

From analysis of `emberware-z/src/app.rs` (1,432 lines):

| Component | Lines | Status | Issue |
|-----------|-------|--------|-------|
| Event loop (ApplicationHandler) | ~130 | Generic | Would be copy-pasted |
| Egui integration + caching | ~250 | Generic | Would be copy-pasted |
| Window management | ~80 | Generic | Would be copy-pasted |
| Mode state machine (Library/Playing/Settings) | ~100 | Generic | Would be copy-pasted |
| Network session handling | ~200 | Generic | Would be copy-pasted |
| Debug overlay | ~100 | Generic | Would be copy-pasted |
| Game loading/starting | ~100 | Generic | Would be copy-pasted |
| Encoder orchestration | ~50 | Generic | Would be copy-pasted |
| **Console-specific code** | ~280 | Specific | Must change per console |
| **Total duplication risk** | **~1,010** | | **70% of file** |

**Console-specific code breakdown:**
- Type parameters (`Runtime<EmberwareZ>` → `Runtime<Classic>`)
- State access (`ZFFIState` → `ClassicFFIState`)
- Graphics calls (`ZGraphics::new()` → `ClassicGraphics::new()`)
- Resource loading patterns (textures, meshes, audio)
- Window title

---

## Proposed Architecture

### Option 1: Shared App Framework (Recommended)

Move the application framework into core as a parameterized type.

#### New Core Structure

```rust
// core/src/app/mod.rs
pub struct ConsoleApp<C: Console> {
    // Mode state machine
    mode: AppMode,

    // Console instance
    console: C,

    // Runtime (game loop + GGRS)
    game_session: Option<GameSession<C>>,

    // Graphics backend (from console)
    graphics: Option<C::Graphics>,

    // Audio backend (from console)
    audio: Option<C::Audio>,

    // Framework components (console-agnostic)
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    window: Option<Arc<Window>>,
    input_manager: Option<InputManager>,

    // Caching for performance
    cached_egui_shapes: Vec<egui::epaint::ClippedShape>,
    cached_egui_tris: Vec<egui::ClippedPrimitive>,
    needs_redraw: bool,

    // Config and state
    config: Config,
    local_games: Vec<LocalGame>,
    debug_overlay: bool,
    debug_stats: DebugStats,
}

pub struct GameSession<C: Console> {
    pub runtime: Runtime<C>,
    // Console-specific resource maps (handled via trait)
}
```

#### Console Trait Extensions

Add methods to the Console trait for framework integration:

```rust
pub trait Console: Send + 'static {
    // Existing methods...
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;
    type State: Default + Send + 'static;

    // New: Resource management integration
    type ResourceManager: ConsoleResourceManager;

    fn create_resource_manager(&self) -> Self::ResourceManager;
    fn window_title(&self) -> &'static str;
}

/// Trait for console-specific resource loading
pub trait ConsoleResourceManager {
    fn process_pending_resources(&mut self, graphics: &mut dyn Graphics, state: &mut dyn Any);
    fn execute_draw_commands(&mut self, graphics: &mut dyn Graphics, state: &mut dyn Any);
}
```

#### ConsoleApp Implementation

```rust
impl<C: Console> ConsoleApp<C> {
    pub fn new(console: C) -> Self {
        // Initialize framework with console instance
    }

    pub fn run(mut self) -> Result<(), AppError> {
        // Generic event loop
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    // Generic methods (work for any console)
    fn run_game_frame(&mut self) -> Result<bool, RuntimeError> { }
    fn handle_session_events(&mut self) -> Result<(), RuntimeError> { }
    fn start_game(&mut self, game_id: &str) -> Result<(), RuntimeError> { }
    fn render(&mut self) { }
    fn request_redraw_if_needed(&mut self) { }
}

impl<C: Console> ApplicationHandler for ConsoleApp<C> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, ...) { }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) { }
}
```

#### Console Binary (emberware-z/src/main.rs)

Reduces to ~100 lines:

```rust
use emberware_core::app::ConsoleApp;
use emberware_z::EmberwareZ;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create console instance
    let console = EmberwareZ::new();

    // Run framework
    let app = ConsoleApp::new(console);
    app.run()?;

    Ok(())
}
```

**That's it.** The entire app.rs moves to core.

---

### Option 2: Optional Helper Crates (More Flexible)

Create opt-in helper crates that consoles can use or replace.

```
emberware-app-framework/  (new crate)
├── WgpuRenderOrchestrator  (encoder management)
├── EguiRenderer            (caching optimization)
├── WinitEventLoop          (event-driven redraw)
└── ConsoleAppBuilder       (app construction)
```

**Usage:**

```rust
// emberware-z/src/main.rs
use emberware_app_framework::ConsoleAppBuilder;

fn main() {
    ConsoleAppBuilder::new()
        .with_console(EmberwareZ::new())
        .with_wgpu_renderer()
        .with_egui()
        .with_winit()
        .run()
        .unwrap();
}
```

**Pros:**
- More modular
- Easier to customize per console
- Core stays minimal

**Cons:**
- Another crate to maintain
- More indirection
- Still requires some boilerplate per console

---

### Option 3: Trait Extensions (Most Complex)

Create specialized trait hierarchies for different capabilities.

```rust
pub trait WgpuConsole: Console {
    fn render_frame(&mut self, encoder: &mut wgpu::CommandEncoder, ...);
}

pub trait EguiConsole: Console {
    fn build_ui(&mut self, ctx: &egui::Context);
}

pub trait WinitConsole: Console {
    fn handle_window_event(&mut self, event: &WindowEvent);
}
```

**Pros:** Maximum flexibility, pay for what you use

**Cons:** Complex trait hierarchy, harder to reason about

---

## Recommendation: Option 1 (Shared App Framework)

**Rationale:**

1. **Pragmatic reality check:**
   - Will Classic/Y/X really *not* use winit/wgpu/egui?
   - Unlikely - these are the best Rust tools for this use case
   - Even a retro console benefits from modern windowing/UI

2. **YAGNI principle:**
   - Don't over-engineer for hypothetical futures
   - If we need custom event loop later, refactor then
   - Current problem: 1,000 lines of duplication *now*

3. **Concrete benefits:**
   - GPU optimizations (unified encoder, egui caching) automatic for all consoles
   - Bug fixes propagate everywhere
   - New features (save state UI, replays, etc.) benefit all consoles
   - Consistency across consoles (good UX)

4. **Escape hatch:**
   - If a console truly needs custom app logic, it can:
     - Opt out by not using ConsoleApp
     - Override specific methods
     - Use core building blocks directly

---

## Implementation Plan

### Phase 1: Move Core App Logic to Core

**Create new modules in core:**

```
core/src/
├── app/
│   ├── mod.rs           (ConsoleApp<C> struct and impl)
│   ├── session.rs       (GameSession<C> management)
│   ├── events.rs        (ApplicationHandler impl)
│   ├── render.rs        (Rendering orchestration)
│   └── ui.rs            (Egui integration)
└── lib.rs               (Re-export app module)
```

**Add dependencies to core/Cargo.toml:**

```toml
[dependencies]
# Existing dependencies...
winit = "0.30"
wgpu = "23"
egui = "0.30"
egui-winit = "0.30"
egui-wgpu = "0.30"
```

**Changes:**
- Extract `AppMode`, `RuntimeError`, `DebugStats` to core
- Move `ConsoleApp` struct to core (generic over C)
- Move all event loop logic to core
- Move egui integration to core
- Move rendering orchestration to core

**Files to create:**
- `core/src/app/mod.rs` (~500 lines)
- `core/src/app/session.rs` (~200 lines)
- `core/src/app/events.rs` (~200 lines)
- `core/src/app/render.rs` (~200 lines)
- `core/src/app/ui.rs` (~200 lines)

**Total new code in core:** ~1,300 lines (moved from emberware-z)

---

### Phase 2: Extend Console Trait

**Add to Console trait:**

```rust
pub trait Console: Send + 'static {
    // Existing...
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;
    type State: Default + Send + 'static;

    // New: Framework integration
    type ResourceManager: ConsoleResourceManager;

    fn name(&self) -> &'static str;
    fn window_title(&self) -> String {
        format!("Emberware {}", self.name())
    }
    fn create_resource_manager(&self) -> Self::ResourceManager;

    // Existing methods...
    fn specs(&self) -> &'static ConsoleSpecs;
    fn register_ffi(&self, linker: &mut Linker<GameStateWithConsole<Self::Input, Self::State>>) -> Result<()>;
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;
    fn create_audio(&self) -> Result<Self::Audio>;
    fn map_input(&self, raw: &RawInput) -> Self::Input;
}
```

**New trait for resource management:**

```rust
/// Console-specific resource loading and processing
pub trait ConsoleResourceManager: Send {
    /// Process pending resources (textures, meshes, audio) from game state
    fn process_pending_resources(
        &mut self,
        graphics: &mut dyn Any,  // Downcast to Console::Graphics
        audio: &mut dyn Any,     // Downcast to Console::Audio
        state: &mut dyn Any,     // Downcast to Console::State
    );

    /// Execute accumulated draw commands
    fn execute_draw_commands(
        &mut self,
        graphics: &mut dyn Any,
        state: &mut dyn Any,
    );
}
```

**Implementation for EmberwareZ:**

```rust
pub struct ZResourceManager {
    texture_map: HashMap<u32, TextureHandle>,
    mesh_map: HashMap<u32, MeshHandle>,
}

impl ConsoleResourceManager for ZResourceManager {
    fn process_pending_resources(&mut self, graphics: &mut dyn Any, audio: &mut dyn Any, state: &mut dyn Any) {
        let graphics = graphics.downcast_mut::<ZGraphics>().unwrap();
        let state = state.downcast_mut::<ZFFIState>().unwrap();

        // Existing logic from app.rs::process_pending_resources()
    }

    fn execute_draw_commands(&mut self, graphics: &mut dyn Any, state: &mut dyn Any) {
        let graphics = graphics.downcast_mut::<ZGraphics>().unwrap();
        let state = state.downcast_mut::<ZFFIState>().unwrap();

        // Existing logic from app.rs::execute_draw_commands_new()
    }
}

impl Console for EmberwareZ {
    type ResourceManager = ZResourceManager;

    fn name(&self) -> &'static str { "Z" }
    fn create_resource_manager(&self) -> Self::ResourceManager {
        ZResourceManager {
            texture_map: HashMap::new(),
            mesh_map: HashMap::new(),
        }
    }
    // ... existing methods
}
```

---

### Phase 3: Refactor Emberware Z

**Simplify emberware-z/src/main.rs:**

```rust
use emberware_core::app::ConsoleApp;
use emberware_z::EmberwareZ;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let console = EmberwareZ::new();
    let app = ConsoleApp::new(console);
    app.run()?;

    Ok(())
}
```

**Move console-specific logic:**

- `emberware-z/src/app.rs` → DELETE (moved to core)
- Keep `emberware-z/src/console.rs` (implements Console trait)
- Keep `emberware-z/src/graphics/` (ZGraphics implementation)
- Keep `emberware-z/src/audio.rs` (ZAudio implementation)
- Keep `emberware-z/src/ffi/` (Z-specific FFI functions)
- Keep `emberware-z/src/state.rs` (ZFFIState definition)

**Add resource manager:**

- `emberware-z/src/resource_manager.rs` (~300 lines from app.rs)

**Result:**
- Binary size: ~100 lines (main.rs)
- Console-specific code: ~3,000 lines (graphics, audio, FFI, state)
- Framework code: 0 lines (all in core)

---

### Phase 4: Build Emberware Classic

**New console binary:**

```
emberware-classic/
├── Cargo.toml
└── src/
    ├── main.rs           (~100 lines)
    ├── console.rs        (implements Console for EmberwareClassic)
    ├── graphics.rs       (8-bit retro renderer)
    ├── audio.rs          (8-bit retro audio)
    ├── ffi/              (Classic-specific FFI)
    ├── state.rs          (ClassicFFIState)
    └── resource_manager.rs
```

**main.rs:**

```rust
use emberware_core::app::ConsoleApp;
use emberware_classic::EmberwareClassic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let console = EmberwareClassic::new();
    let app = ConsoleApp::new(console);
    app.run()?;

    Ok(())
}
```

**No duplication of:**
- Event loop
- Egui integration
- Window management
- Network session handling
- Debug overlay
- Game loading
- Encoder orchestration
- Performance optimizations

**Console implements:**
- `ClassicInput` (6-button retro input)
- `ClassicGraphics` (8-bit tile-based renderer)
- `ClassicFFIState` (tile maps, sprite lists, palettes)
- `ClassicResourceManager` (tile/palette loading)
- Classic-specific FFI (set_tile, draw_sprite, set_palette, etc.)

**Development time:**
- With current architecture: 2-3 weeks (1,000 lines of copy-paste + debugging)
- With refactored architecture: 3-5 days (just console-specific code)

---

## Migration Strategy

### Step 1: Create core/src/app/ (No Breaking Changes)

1. Create new modules in core:
   - `core/src/app/mod.rs`
   - `core/src/app/session.rs`
   - `core/src/app/events.rs`
   - `core/src/app/render.rs`
   - `core/src/app/ui.rs`

2. Copy code from `emberware-z/src/app.rs` and make generic over `C: Console`

3. Add wgpu/egui dependencies to core

4. Do NOT delete emberware-z/src/app.rs yet

**Result:** Core has the framework, Z still works with old app.rs

---

### Step 2: Extend Console Trait (Additive Only)

1. Add `ConsoleResourceManager` trait to `core/src/console.rs`
2. Add associated type `ResourceManager` to Console trait with default impl
3. Add `name()` and `window_title()` methods with defaults

**Backward compatibility:**

```rust
pub trait Console: Send + 'static {
    // New optional methods with defaults
    fn name(&self) -> &'static str { "Unknown" }
    fn window_title(&self) -> String { format!("Emberware {}", self.name()) }

    // New associated type with default
    type ResourceManager: ConsoleResourceManager = DefaultResourceManager;
    fn create_resource_manager(&self) -> Self::ResourceManager {
        Default::default()
    }
}
```

**Result:** EmberwareZ still compiles without changes

---

### Step 3: Implement ZResourceManager

1. Create `emberware-z/src/resource_manager.rs`
2. Move resource loading logic from app.rs
3. Implement `ConsoleResourceManager` for `ZResourceManager`
4. Update `EmberwareZ` impl to use it

**Result:** Z has resource manager, still using old app.rs

---

### Step 4: Migrate Emberware Z to ConsoleApp

1. Replace `emberware-z/src/main.rs` with new version using `ConsoleApp`
2. Delete `emberware-z/src/app.rs`
3. Delete UI modules (library_ui.rs, settings_ui.rs) - moved to core
4. Test thoroughly

**Result:** Z uses framework, 1,200 lines deleted

---

### Step 5: Build Emberware Classic

1. Create new `emberware-classic` crate
2. Implement Console trait for EmberwareClassic
3. Use ConsoleApp in main.rs
4. Develop console-specific code only

**Result:** Classic built with ~500 lines instead of ~1,500

---

## Testing Strategy

### Unit Tests

**In core:**
- `ConsoleApp` with `TestConsole` (already exists in test_utils.rs)
- Event loop state transitions
- Egui caching behavior
- Render orchestration

**In emberware-z:**
- `ZResourceManager` resource loading
- Console trait implementation
- Graphics/Audio backends (existing tests)

### Integration Tests

**Full application flow:**
1. Library → Playing → Library (mode transitions)
2. Window resize, fullscreen toggle
3. Egui caching (hover triggers redraw, idle doesn't)
4. Network session (local P2P, desync handling)
5. Game loading errors
6. Save/load functionality

### Regression Tests

**Before/after comparison:**
- Performance (FPS, GPU usage, frame times)
- Visual output (screenshot comparison)
- Network behavior (rollback correctness)
- Input latency
- Audio quality

---

## Performance Impact

### Expected Improvements

**All consoles automatically benefit from:**
- Unified encoder (single submit per frame)
- Event-driven redraw (idle GPU <5%)
- Egui caching (skip tessellation when unchanged)
- Future optimizations propagate automatically

### Measurement Plan

**Metrics to track:**
- Build time (expect slight increase for core, decrease for consoles)
- Binary size (expect similar, framework is shared at link time)
- Runtime performance (expect identical or better)
- Developer productivity (expect 5-10x faster to build new consoles)

---

## Risks and Mitigation

### Risk 1: Core becomes too heavy

**Mitigation:**
- Feature flags for optional components (egui, wgpu)
- Keep Console trait minimal
- Move heavy code to app module, not console module

### Risk 2: Less flexibility per console

**Mitigation:**
- Provide override hooks in ConsoleApp
- Allow consoles to opt out and use core building blocks directly
- Use trait methods for console-specific behavior

### Risk 3: Breaking changes for existing code

**Mitigation:**
- Incremental migration (steps 1-5 above)
- Keep both old and new code working during transition
- Use default trait implementations for backward compatibility

### Risk 4: Complexity in core crate

**Mitigation:**
- Clear module structure (app/, rollback/, wasm/)
- Comprehensive documentation
- Examples for each console type

---

## Success Criteria

### Code Metrics

- [ ] emberware-z binary: <150 lines (currently ~1,400)
- [ ] emberware-classic binary: <150 lines (new)
- [ ] Shared framework code: ~1,300 lines in core
- [ ] Zero duplication of event loop, egui, or orchestration

### Functional Requirements

- [ ] All existing Emberware Z features work
- [ ] Performance unchanged or improved
- [ ] No visual regressions
- [ ] Network play still works
- [ ] Can build Emberware Classic in <1 week

### Developer Experience

- [ ] Clear documentation for building new consoles
- [ ] Example console (Classic) demonstrates the pattern
- [ ] Tests pass for both Z and Classic
- [ ] CI/CD builds both consoles

---

## Timeline Estimate

**Phase 1:** Create core/src/app/ - 2-3 days
**Phase 2:** Extend Console trait - 1 day
**Phase 3:** Implement ZResourceManager - 1 day
**Phase 4:** Migrate Emberware Z - 1-2 days
**Phase 5:** Build Emberware Classic - 3-5 days

**Total:** 8-12 days (2 weeks)

**ROI:** Every future console saves 1-2 weeks of duplication work

---

## Future Enhancements

Once the framework is in place:

1. **Renderer abstraction** - Make wgpu optional (for headless testing)
2. **UI customization** - Let consoles add custom UI panels
3. **Plugin system** - Allow third-party extensions
4. **Hot reload** - Reload shaders/assets without restart
5. **Replay system** - Record/playback using GGRS state snapshots
6. **Cloud saves** - Sync saves across devices
7. **Achievements** - Framework-level achievement tracking

All of these benefit all consoles automatically.

---

## Conclusion

**Current state:** Each console is a standalone 1,500-line application with 70% duplication.

**Proposed state:** Each console is a 100-line thin wrapper around a shared 1,300-line framework.

**Benefits:**
- 93% code reduction per console
- Automatic feature/optimization propagation
- Consistent UX across consoles
- 5-10x faster to build new consoles

**Recommendation:** Implement Option 1 (Shared App Framework) with the phased migration plan. This gives maximum code reuse while maintaining flexibility for console-specific needs.

---

## Appendix: File Structure After Refactoring

```
emberware/
├── core/
│   ├── src/
│   │   ├── app/
│   │   │   ├── mod.rs        (ConsoleApp<C>)
│   │   │   ├── session.rs    (GameSession<C>)
│   │   │   ├── events.rs     (ApplicationHandler impl)
│   │   │   ├── render.rs     (Rendering orchestration)
│   │   │   └── ui.rs         (Egui integration)
│   │   ├── console.rs        (Console trait + ResourceManager trait)
│   │   ├── runtime.rs        (Runtime<C>)
│   │   ├── wasm/
│   │   ├── rollback/
│   │   └── ffi.rs
│   └── Cargo.toml            (Add wgpu, egui deps)
│
├── emberware-z/
│   ├── src/
│   │   ├── main.rs           (100 lines - just instantiate + run)
│   │   ├── console.rs        (EmberwareZ impl)
│   │   ├── resource_manager.rs (ZResourceManager)
│   │   ├── graphics/         (ZGraphics implementation)
│   │   ├── audio.rs          (ZAudio implementation)
│   │   ├── ffi/              (Z-specific FFI)
│   │   └── state.rs          (ZFFIState)
│   └── Cargo.toml
│
└── emberware-classic/        (New!)
    ├── src/
    │   ├── main.rs           (100 lines - just instantiate + run)
    │   ├── console.rs        (EmberwareClassic impl)
    │   ├── resource_manager.rs (ClassicResourceManager)
    │   ├── graphics.rs       (Retro 8-bit renderer)
    │   ├── audio.rs          (Retro 8-bit audio)
    │   ├── ffi/              (Classic-specific FFI)
    │   └── state.rs          (ClassicFFIState)
    └── Cargo.toml
```

**Key insight:** The framework (core/src/app/) is written once, used by all consoles. Console-specific code is isolated to console crates.
