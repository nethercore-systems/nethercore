# Console-Agnostic Library Architecture Specification

**Status:** REJECTED - in favor of combined-refactor-plan.md
**Author:** Claude (with Zerve)
**Version:** 1.0
**Last Updated:** December 2024

---

## Summary

Refactor Emberware from a Z-specific application into a console-agnostic library that can run games for any supported console. The console becomes a "black box" renderer that the library app orchestrates, rather than each console owning its own runtime and app implementation.

**Goals:**
- Single unified library app for all consoles
- Consoles are pure renderers with no app/runtime ownership
- Zero code duplication for rollback, sessions, UI
- Easy addition of future consoles (Classic, Y, X)
- Clean separation of concerns (DRY, no code smell)
- **Frontend-agnostic core** - anyone can build alternative frontends (mobile, web, console ports)

---

## Current Architecture Problems

### 1. Z-Specific App in Core
```
core/src/library/game.rs:12    → imports z_common::ZRom
core/src/library/cart.rs:10    → imports z_common
core/Cargo.toml:12             → depends on z-common
```

### 2. Each Console Owns Its Runtime
```
emberware-z/src/app/mod.rs     → 1242 lines of app logic
emberware_z::app::run(mode)    → Full app lifecycle per console
ConsoleType::launch_game()     → Routes to console's own app
```

### 3. Mixed Concerns in App
```rust
// emberware-z/src/app/mod.rs - mixes generic + Z-specific:
pub struct App {
    graphics: ZGraphics,           // Z-specific
    game: Option<GameSession<EmberwareZ>>,  // Generic (could be shared)
    mode: AppMode,                 // Generic (could be shared)
    library_ui: LibraryUi,         // Generic (could be shared)
}
```

---

## Proposed Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         library/                                │
│  ┌────────────────────────────────────────────────────────────┐│
│  │                    App (Generic)                           ││
│  │  - Window lifecycle        - Library UI                    ││
│  │  - Event loop              - Settings UI                   ││
│  │  - Mode transitions        - Debug panel                   ││
│  │  - Game session management                                 ││
│  └────────────────────────────────────────────────────────────┘│
│                              │                                  │
│              ┌───────────────┼───────────────┐                  │
│              ▼               ▼               ▼                  │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────┐      │
│  │  ConsoleRunner │ │  ConsoleRunner │ │  ConsoleRunner │      │
│  │  <EmberwareZ>  │ │  <Classic>     │ │  <Future>      │      │
│  └───────┬────────┘ └───────┬────────┘ └───────┬────────┘      │
│          │                  │                  │                │
└──────────┼──────────────────┼──────────────────┼────────────────┘
           │                  │                  │
           ▼                  ▼                  ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│   emberware-z    │ │ emberware-classic│ │   future-console │
│   (lib only)     │ │   (lib only)     │ │   (lib only)     │
│                  │ │                  │ │                  │
│ - ZGraphics      │ │ - ClassicGfx     │ │ - ...            │
│ - ZFFIState      │ │ - ClassicState   │ │                  │
│ - Z FFI funcs    │ │ - Classic FFI    │ │                  │
│ - ZInput mapping │ │ - Input mapping  │ │                  │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

### Key Principle: Console as Black Box

A console should only:
1. **Register FFI functions** - `register_ffi(linker)`
2. **Create graphics backend** - `create_graphics(window)`
3. **Create audio backend** - `create_audio()`
4. **Map input** - `map_input(raw) -> ConsoleInput`
5. **Execute draw commands** - Render one frame when told to

A console should NOT:
- Own or manage the runtime
- Handle rollback or networking
- Manage game sessions
- Contain app/UI logic
- Know about the library

---

## New Crate Structure

### Before (Current)
```
emberware/
├── core/                    # Framework + Z-specific leaks
├── shared/                  # Truly shared types
├── z-common/                # Z asset formats
├── emberware-z/             # Z console + FULL APP
│   └── src/
│       ├── main.rs          # Binary entry point
│       ├── app/             # Full app implementation
│       ├── console.rs       # Console trait impl
│       └── ...
└── src/
    ├── main.rs              # Thin launcher
    └── registry.rs          # Routes to Z app
```

### After (Proposed)
```
emberware/
├── core/                    # Framework (Z-free)
│   └── src/
│       ├── console.rs       # Console trait (unchanged)
│       ├── runtime.rs       # Runtime<C> (unchanged)
│       ├── runner.rs        # ConsoleRunner<C> + ActiveGame (NEW)
│       ├── library/         # Game library (generic via trait)
│       │   ├── game.rs      # Uses RomLoader trait
│       │   └── cart.rs      # Uses RomInstaller trait
│       └── ...
├── shared/                  # Truly shared types
├── z-common/                # Z asset formats (unchanged)
├── emberware-z/             # Z console LIBRARY ONLY
│   └── src/
│       ├── lib.rs           # No main.rs!
│       ├── console.rs       # Console trait impl
│       ├── graphics/        # ZGraphics
│       ├── ffi/             # Z FFI functions
│       └── state/           # ZFFIState
├── library/                 # NEW: Console-agnostic app with UI
│   └── src/
│       ├── main.rs          # Entry point
│       ├── app/             # Generic app logic
│       ├── ui/              # Library UI, settings, debug panel
│       └── ...
├── player/                  # NEW: Minimal standalone player (no UI)
│   └── src/
│       ├── main.rs          # Entry point
│       └── app.rs           # Minimal event loop
└── Cargo.toml               # Workspace with library as default
```

---

## Core Abstractions

### 1. ConsoleRunner<C: Console>

Shared by both library and player apps to run games without knowing console internals:

```rust
// core/src/runner.rs
pub struct ConsoleRunner<C: Console> {
    console: C,
    graphics: C::Graphics,
    audio: C::Audio,
    runtime: Runtime<C>,
    resource_manager: C::ResourceManager,
}

impl<C: Console> ConsoleRunner<C> {
    /// Create a new runner for a console
    pub fn new(console: C, window: Arc<Window>) -> Result<Self>;

    /// Load a game ROM
    pub fn load_game(&mut self, rom_path: &Path) -> Result<()>;

    /// Advance simulation by one frame (handles rollback internally)
    pub fn update(&mut self, input: &RawInput);

    /// Render the current frame
    pub fn render(&mut self);

    /// Resize the rendering surface
    pub fn resize(&mut self, width: u32, height: u32);

    /// Get debug info for the debug panel
    pub fn debug_info(&self) -> DebugInfo;

    /// Check if game requested quit
    pub fn quit_requested(&self) -> bool;
}
```

### 2. RomLoader Trait (Remove Z from Core)

```rust
// core/src/library/rom.rs
pub trait RomLoader: Send + Sync {
    /// File extension this loader handles (e.g., "ewz")
    fn extension(&self) -> &'static str;

    /// Magic bytes to identify this format
    fn magic_bytes(&self) -> &'static [u8];

    /// Load a ROM file, returning WASM bytes and optional data pack
    fn load(&self, path: &Path) -> Result<LoadedRom>;

    /// Install a ROM to the local game library
    fn install(&self, path: &Path, library_path: &Path) -> Result<LocalGame>;
}

pub struct LoadedRom {
    pub wasm: Vec<u8>,
    pub data_pack: Option<Vec<u8>>,  // Console-specific format
    pub manifest: LocalGameManifest,
}
```

### 3. Static Console Registry (Pure Static Dispatch)

```rust
// library/src/registry.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType {
    Z,
    // Future: Classic, Y, X
}

impl ConsoleType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "z" => Some(Self::Z),
            _ => None,
        }
    }

    pub fn rom_extension(&self) -> &'static str {
        match self {
            Self::Z => "ewz",
        }
    }
}

// Active game enum - pure static dispatch, no vtables
pub enum ActiveGame {
    Z(ConsoleRunner<EmberwareZ>),
    // Future: Classic(ConsoleRunner<EmberwareClassic>),
}

impl ActiveGame {
    pub fn update(&mut self, input: &RawInput) {
        match self {
            Self::Z(runner) => runner.update(input),
        }
    }

    pub fn render(&mut self) {
        match self {
            Self::Z(runner) => runner.render(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        match self {
            Self::Z(runner) => runner.resize(width, height),
        }
    }

    pub fn debug_stats(&self) -> Vec<DebugStat> {
        match self {
            Self::Z(runner) => runner.debug_stats(),
        }
    }

    pub fn quit_requested(&self) -> bool {
        match self {
            Self::Z(runner) => runner.quit_requested(),
        }
    }
}

// Factory function - compiler enforces exhaustiveness
pub fn create_game(
    console_type: ConsoleType,
    window: Arc<Window>,
    rom_path: &Path,
) -> Result<ActiveGame> {
    match console_type {
        ConsoleType::Z => {
            let console = EmberwareZ::new();
            let mut runner = ConsoleRunner::new(console, window)?;
            runner.load_game(rom_path)?;
            Ok(ActiveGame::Z(runner))
        }
    }
}
```

**Key Design:**
- `ActiveGame` enum provides pure static dispatch (zero vtables)
- Adding a new console = add enum variant + match arms
- Compiler enforces exhaustiveness - impossible to forget a console
- All hot-path code (update/render) is monomorphized

---

## App Architecture

### Generic App Structure

```rust
// library/src/app/mod.rs
pub struct App {
    // Window management
    window: Arc<Window>,

    // Mode state
    mode: AppMode,

    // UI (egui for library/settings, wgpu surface for game)
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    wgpu_state: WgpuState,  // Shared GPU state

    // Active game (pure static dispatch via enum)
    active_game: Option<ActiveGame>,

    // Library
    games: Vec<LocalGame>,

    // Input
    input_manager: InputManager,

    // Debug panel state
    debug_panel_open: bool,
}

pub enum AppMode {
    Library,
    Settings,
    Playing { game_id: String, console_type: ConsoleType },
}
```

### Game Launch Flow

```
User selects game in Library UI
        │
        ▼
App::launch_game(game_id)
        │
        ▼
Look up LocalGame → get console_type
        │
        ▼
ConsoleType::create_runner(window)
        │
        ▼
runner.load_game(rom_path)
        │
        ▼
App.active_runner = Some(runner)
App.mode = Playing { game_id, console_type }
        │
        ▼
Main loop calls runner.update() + runner.render()
```

### CLI/Direct Launch Mode

```rust
// library/src/main.rs
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mode = if args.len() > 1 {
        // Direct game launch (skip library UI)
        let game_id = &args[1];
        let game = resolve_game(game_id)?;
        AppMode::Playing {
            game_id: game.id.clone(),
            console_type: ConsoleType::from_str(&game.console_type)?,
        }
    } else {
        // Launch library UI
        AppMode::Library
    };

    App::run(mode)
}
```

---

## Standalone Player

A **minimal binary** for running games without any library UI code. Use cases:
- Ship with individual games for distribution
- Development iteration (faster startup)
- Embedding in other applications
- Kiosk/arcade deployments

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    player/ crate                        │
│  ┌───────────────────────────────────────────────────┐ │
│  │              Minimal App (no egui)                │ │
│  │  - Window + wgpu surface                          │ │
│  │  - Input capture                                  │ │
│  │  - ActiveGame enum (same as library)              │ │
│  │  - NO library UI, NO settings UI                  │ │
│  └───────────────────────────────────────────────────┘ │
│                          │                              │
│                          ▼                              │
│  ┌───────────────────────────────────────────────────┐ │
│  │  ConsoleRunner<C> (reused from library/core)      │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Crate Structure

```
player/
├── Cargo.toml          # Minimal deps: core, z, winit, wgpu (NO egui)
└── src/
    ├── main.rs         # Entry point
    └── app.rs          # Minimal app loop
```

### Usage

```bash
# Run ROM file directly
cargo run -p player -- path/to/game.ewz

# Run installed game by ID
cargo run -p player -- --id cube

# With window options
cargo run -p player -- game.ewz --fullscreen --scale 3
```

### Implementation

```rust
// player/src/main.rs
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// ROM file path or game ID
    game: String,

    /// Run in fullscreen
    #[arg(long)]
    fullscreen: bool,

    /// Integer scaling factor
    #[arg(long, default_value = "2")]
    scale: u32,

    /// Treat argument as game ID instead of path
    #[arg(long)]
    id: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom_path = if args.id {
        resolve_game_id(&args.game)?.rom_path
    } else {
        PathBuf::from(&args.game)
    };

    let console_type = ConsoleType::from_extension(
        rom_path.extension().unwrap_or_default()
    )?;

    PlayerApp::run(rom_path, console_type, args.fullscreen, args.scale)
}
```

```rust
// player/src/app.rs
pub struct PlayerApp {
    window: Arc<Window>,
    active_game: ActiveGame,  // Same enum as library - pure static dispatch
    input_manager: InputManager,
}

impl PlayerApp {
    pub fn run(rom_path: PathBuf, console_type: ConsoleType, fullscreen: bool, scale: u32) -> Result<()> {
        let event_loop = EventLoop::new()?;
        let window = create_window(&event_loop, fullscreen, scale)?;

        let active_game = create_game(console_type, window.clone(), &rom_path)?;

        let mut app = Self {
            window,
            active_game,
            input_manager: InputManager::new(),
        };

        event_loop.run(move |event, target| {
            app.handle_event(event, target);
        })?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) {
        match event {
            Event::AboutToWait => {
                self.active_game.update(&self.input_manager.raw_input());
                self.window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                self.active_game.render();
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                self.active_game.resize(size.width, size.height);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                target.exit();
            }
            _ => {
                self.input_manager.handle_event(&event);
            }
        }

        if self.active_game.quit_requested() {
            target.exit();
        }
    }
}
```

### Dependencies Comparison

| Crate | library | player |
|-------|---------|--------|
| emberware-core | ✓ | ✓ |
| emberware-z | ✓ | ✓ |
| winit | ✓ | ✓ |
| wgpu | ✓ | ✓ |
| egui | ✓ | ✗ |
| egui-winit | ✓ | ✗ |
| egui-wgpu | ✓ | ✗ |
| clap | ✗ | ✓ |

**Binary size savings:** ~2-3MB smaller without egui stack

### Code Reuse

Both `library` and `player` reuse:
- `ConsoleRunner<C>` - game lifecycle
- `ActiveGame` enum - static dispatch
- `create_game()` factory - console instantiation
- `InputManager` - input handling

These shared components can live in:
- **Option A:** `library/src/runner.rs` (player depends on library with `default-features = false`)
- **Option B:** `core/src/runner.rs` (both depend on core)
- **Option C:** New `runner/` crate (shared by both)

**Recommended: Option B** - Put `ConsoleRunner` and `ActiveGame` in core, keeping player truly minimal.

---

## Frontend Extensibility

The architecture explicitly separates **console implementation** from **frontend/UI**:

```
┌────────────────────────────────────────────────────────────────┐
│                    CORE (UI-agnostic)                          │
│  Console trait, ConsoleRunner<C>, Runtime<C>, DebugStat types  │
└─────────────────────────────┬──────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌────────────────┐   ┌────────────────────┐
│   library/    │   │  mobile-app/   │   │   web-frontend/    │
│   (egui)      │   │  (SwiftUI/     │   │   (React/WASM)     │
│               │   │   Compose)     │   │                    │
│ Desktop app   │   │ iOS/Android    │   │ Browser-based      │
└───────────────┘   └────────────────┘   └────────────────────┘
```

**What each frontend provides:**
- Window/surface management
- UI rendering (library browser, settings)
- Debug panel visualization
- Input capture and mapping

**What each frontend reuses from core:**
- `ConsoleRunner<C>` - game lifecycle
- `Runtime<C>` - rollback, networking
- Console implementations (Z, Classic, etc.)
- All FFI, WASM execution, state management

**Example: Building a mobile frontend:**
```rust
// mobile-app/src/lib.rs
use emberware_core::runner::ConsoleRunner;
use emberware_z::EmberwareZ;

// Create runner with platform-specific window
let console = EmberwareZ::new();
let runner = ConsoleRunner::new(console, mobile_window)?;

// Main loop - integrate with platform event loop
runner.load_game(rom_path)?;
loop {
    runner.update(&input);
    runner.render();
}
```

This means the `library/` crate we're creating is just **one** reference implementation. Others can build:
- **Console ports** (Switch, PlayStation) with native platform UI
- **Mobile apps** with SwiftUI/Jetpack Compose
- **Web players** with React/Vue frontend via wasm-bindgen
- **Headless test runners** for CI/CD

---

## Debug Panel Integration

**Architecture Decision:** Data types in `core/`, UI rendering in `library/`.

This separation allows:
- Core remains UI-agnostic (no egui dependency)
- Future frontends (web, headless) can consume same data, render differently
- Consoles just provide data, don't know about rendering

### Data Types (core/src/debug/stats.rs)

```rust
// core/src/debug/stats.rs - NEW FILE
/// Console-provided debug statistics
pub struct DebugStat {
    pub category: &'static str,  // e.g., "Memory", "Graphics"
    pub name: &'static str,
    pub value: DebugValue,
}

pub enum DebugValue {
    Bytes(usize),      // Format as KB/MB
    Count(usize),      // Raw number
    Percent(f32),      // 0.0 - 1.0
    Text(String),      // Free-form
}
```

### Console Trait Addition (core/src/console.rs)

```rust
pub trait Console: Send + 'static {
    // ... existing methods ...

    /// Get debug stats for the debug panel
    fn debug_stats(&self, state: &Self::State) -> Vec<DebugStat>;
}
```

### UI Rendering (library/src/ui/debug.rs)

```rust
// library/src/ui/debug.rs
pub fn render_debug_panel(ui: &mut egui::Ui, stats: &[DebugStat]) {
    // Groups stats by category, renders as collapsible sections
    // Formats DebugValue appropriately (bytes as KB/MB, etc.)
}
```

**Z Console Stats Example:**
- Memory: RAM usage, VRAM usage
- Graphics: Draw calls, vertices, textures loaded
- Audio: Active channels, sounds playing

---

## Migration Plan (4 Phases)

### Phase 1: Create Library + Player Crates with ConsoleRunner

**Goal:** New `library/` and `player/` crates with working ConsoleRunner in core, games launch via new path.

**Steps:**
1. Create `core/src/runner.rs` with `ConsoleRunner<C: Console>`, `ActiveGame` enum, `create_game()` factory
2. Create `library/Cargo.toml` with dependencies (emberware-core, emberware-z, egui, wgpu, winit)
3. Create `player/Cargo.toml` with minimal dependencies (emberware-core, emberware-z, winit, wgpu, clap)
4. Add `library` and `player` to workspace members in root `Cargo.toml`
5. Create `library/src/app/mod.rs` - App struct with library UI
6. Create `library/src/main.rs` - entry point that creates App
7. Create `player/src/app.rs` - minimal PlayerApp without UI
8. Create `player/src/main.rs` - CLI entry point with clap
9. Copy UI code from emberware-z to `library/src/ui/` (library.rs, settings.rs)
10. Wire up game launch in both: uses `create_game()` from core → `ActiveGame::Z(...)`

**Verification:**
- `cargo run -p library -- cube` launches cube with library
- `cargo run -p player -- path/to/cube.ewz` launches cube standalone

**Files Created:**
- `core/src/runner.rs`
- `library/Cargo.toml`
- `library/src/main.rs`
- `library/src/lib.rs`
- `library/src/app/mod.rs`
- `library/src/app/event_loop.rs`
- `library/src/ui/mod.rs`
- `library/src/ui/library.rs`
- `library/src/ui/settings.rs`
- `library/src/ui/debug.rs`
- `player/Cargo.toml`
- `player/src/main.rs`
- `player/src/app.rs`

---

### Phase 2: Remove Z-Specific Code from Core

**Goal:** Core has zero imports from z-common. ROM loading uses trait-based abstraction.

**Steps:**
1. Create `core/src/library/rom.rs` with `RomLoader` trait
2. Create `RomLoaderRegistry` struct to hold all loaders
3. Implement `ZRomLoader` in `z-common/src/loader.rs` (move logic from core)
4. Update `core/src/library/game.rs`:
   - Remove `use z_common::ZRom`
   - Use `RomLoaderRegistry` to detect/load ROMs by extension
5. Update `core/src/library/cart.rs`:
   - Remove `use z_common`
   - Use `RomLoader::install()` trait method
6. Remove `z-common` from `core/Cargo.toml` dependencies
7. Add `debug_stats()` method to Console trait
8. Create `core/src/debug/stats.rs` with `DebugStat`, `DebugValue` types

**Verification:** `cargo build -p emberware-core` succeeds with no z-common imports.

**Files Created:**
- `core/src/library/rom.rs`
- `core/src/debug/stats.rs`
- `z-common/src/loader.rs`

**Files Modified:**
- `core/Cargo.toml` (remove z-common dep)
- `core/src/library/mod.rs` (export rom module)
- `core/src/library/game.rs` (use RomLoader trait)
- `core/src/library/cart.rs` (use RomLoader trait)
- `core/src/console.rs` (add debug_stats method)
- `core/src/debug/mod.rs` (export stats module)
- `z-common/Cargo.toml` (if needed)
- `z-common/src/lib.rs` (export loader)

---

### Phase 3: Clean Up emberware-z

**Goal:** emberware-z becomes library-only crate (no binary, no app/).

**Steps:**
1. Ensure library crate has full feature parity with current emberware-z app
2. Remove `emberware-z/src/main.rs`
3. Remove `emberware-z/src/app/` directory entirely
4. Update `emberware-z/Cargo.toml`:
   - Remove `[[bin]]` section
   - Keep only `[lib]` section
5. Update `emberware-z/src/lib.rs`:
   - Remove `pub mod app`
   - Export only: `console`, `graphics`, `ffi`, `state`, `audio`
6. Implement `debug_stats()` for EmberwareZ console

**Verification:** `cargo build -p emberware-z` succeeds as library-only.

**Files Deleted:**
- `emberware-z/src/main.rs`
- `emberware-z/src/app/` (entire directory)
- `emberware-z/src/ui.rs` (moved to library)
- `emberware-z/src/settings_ui.rs` (moved to library)
- `emberware-z/src/game_resolver.rs` (if exists, moved to library)

**Files Modified:**
- `emberware-z/Cargo.toml`
- `emberware-z/src/lib.rs`
- `emberware-z/src/console.rs` (implement debug_stats)

---

### Phase 4: Workspace Finalization

**Goal:** library is the default binary, old launcher removed, docs updated.

**Steps:**
1. Update root `Cargo.toml`:
   - Set `default-members = ["library"]`
   - Or rename library to main package
2. Delete `src/main.rs` and `src/registry.rs` (replaced by library)
3. Update `CLAUDE.md` architecture diagram
4. Update `TASKS.md` - mark this task complete
5. Update any CI/CD scripts
6. Run all examples to verify nothing broke:
   ```
   cargo run -- cube
   cargo run -- platformer
   cargo run -- lighting
   cargo run -- billboard
   ```
7. Test CLI direct launch and library UI modes

**Verification:**
- `cargo run` launches library UI
- `cargo run -- cube` launches cube directly
- All examples work
- `cargo build --workspace` succeeds

**Files Deleted:**
- `src/main.rs`
- `src/registry.rs`
- `src/` directory (if empty)

**Files Modified:**
- `Cargo.toml` (workspace config)
- `CLAUDE.md` (architecture docs)
- `TASKS.md` (mark complete)

---

## Summary of All File Changes

### New Files (18 files)
```
library/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── lib.rs
    ├── app/
    │   ├── mod.rs          # App struct
    │   └── event_loop.rs   # Window events
    └── ui/
        ├── mod.rs
        ├── library.rs      # Library browser
        ├── settings.rs     # Settings page
        └── debug.rs        # Debug panel rendering

player/
├── Cargo.toml
└── src/
    ├── main.rs             # CLI entry point
    └── app.rs              # Minimal event loop

core/src/
├── runner.rs               # ConsoleRunner<C> + ActiveGame + create_game()
├── library/rom.rs          # RomLoader trait
└── debug/stats.rs          # DebugStat types

z-common/src/
└── loader.rs               # ZRomLoader implementation
```

### Modified Files (11 files)
| File | Changes |
|------|---------|
| `Cargo.toml` | Add library + player members, set default-members |
| `core/Cargo.toml` | Remove z-common dependency, add runner deps |
| `core/src/lib.rs` | Export runner module |
| `core/src/library/mod.rs` | Export rom module |
| `core/src/library/game.rs` | Use RomLoader trait instead of ZRom |
| `core/src/library/cart.rs` | Use RomLoader trait |
| `core/src/console.rs` | Add `debug_stats()` method |
| `core/src/debug/mod.rs` | Export stats module |
| `emberware-z/Cargo.toml` | Remove `[[bin]]` section |
| `emberware-z/src/lib.rs` | Remove app module export |
| `emberware-z/src/console.rs` | Implement `debug_stats()` |

### Deleted Files (~15 files)
| File/Directory | Reason |
|------|--------|
| `emberware-z/src/main.rs` | No longer a binary |
| `emberware-z/src/app/` | Entire directory moved to library |
| `emberware-z/src/ui.rs` | Moved to library |
| `emberware-z/src/settings_ui.rs` | Moved to library |
| `src/main.rs` | Replaced by library |
| `src/registry.rs` | Moved to library |

---

## Success Criteria

- [ ] `cargo run` launches library with all consoles' games
- [ ] `cargo run -- cube` directly launches game (no library UI)
- [ ] `cargo run -p player -- game.ewz` runs standalone player
- [ ] `core/` has zero imports from `z-common`
- [ ] `emberware-z/` has no `main.rs` or `app/` directory
- [ ] Adding a new console requires only:
  1. New crate implementing `Console`
  2. New variant in `ConsoleType` enum + `ActiveGame` enum
  3. Match arms in `create_game()` and `ActiveGame` impl
- [ ] All existing examples work unchanged
- [ ] Debug panel shows console-specific stats
- [ ] Zero dynamic dispatch in hot path (update/render)
- [ ] Player binary is ~2-3MB smaller than library (no egui)
