# Building a New Fantasy Console

This guide explains how to create a new fantasy console for the Emberware platform (e.g., Emberware Classic, Emberware Y, etc.).

## Architecture Overview

Emberware uses a **trait-based console abstraction** that separates:
- **Core framework** (`emberware-core`) - WASM runtime, rollback netcode, shared utilities
- **Console implementations** (`emberware-z`, `emberware-classic`, etc.) - Graphics, audio, FFI, UI

Each console implements the `Console` trait and provides its own:
1. Graphics backend (rendering pipeline, shaders, vertex formats)
2. Audio backend (sound playback, effects)
3. Input layout (button mapping, analog sticks, etc.)
4. FFI functions (console-specific API for games)
5. Resource manager (texture/mesh loading)
6. UI (library and settings screens)

## Quick Start: Minimal Console

Here's the minimum viable console structure:

```
emberware-my-console/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point (~50 lines)
│   ├── console.rs        # Console trait impl
│   ├── graphics/         # Your rendering backend
│   ├── audio.rs          # Your audio backend
│   ├── ffi/              # Console-specific FFI functions
│   ├── state.rs          # FFI staging state
│   ├── resource_manager.rs  # Resource loading
│   ├── ui.rs             # Library UI
│   └── settings_ui.rs    # Settings UI
```

## Step 1: Console Trait Implementation

Create `src/console.rs`:

```rust
use emberware_core::console::{Console, ConsoleSpecs, RawInput, ConsoleResourceManager};
use std::sync::Arc;
use winit::window::Window;

pub struct MyConsole;

impl MyConsole {
    pub fn new() -> Self {
        Self
    }
}

impl Console for MyConsole {
    type Graphics = MyGraphics;
    type Audio = MyAudio;
    type Input = MyInput;
    type State = MyFFIState;
    type ResourceManager = MyResourceManager;

    fn specs(&self) -> &'static ConsoleSpecs {
        &ConsoleSpecs {
            name: "My Console",
            resolutions: &[(320, 240), (640, 480)],
            default_resolution: 0,
            tick_rates: &[30, 60],
            default_tick_rate: 1,
            ram_limit: 16 * 1024 * 1024,
            vram_limit: 8 * 1024 * 1024,
            rom_limit: 32 * 1024 * 1024,
            cpu_budget_us: 4000,
        }
    }

    fn register_ffi(
        &self,
        linker: &mut wasmtime::Linker<GameStateWithConsole<Self::Input, Self::State>>,
    ) -> anyhow::Result<()> {
        // Register your console-specific FFI functions here
        Ok(())
    }

    fn create_graphics(&self, window: Arc<Window>) -> anyhow::Result<Self::Graphics> {
        MyGraphics::new(window)
    }

    fn create_audio(&self) -> anyhow::Result<Self::Audio> {
        Ok(MyAudio::new())
    }

    fn map_input(&self, raw: &RawInput) -> Self::Input {
        // Map raw input to your console's input format
        MyInput::from_raw(raw)
    }

    fn create_resource_manager(&self) -> Self::ResourceManager {
        MyResourceManager::new()
    }

    fn window_title(&self) -> &'static str {
        "My Fantasy Console"
    }
}
```

## Step 2: Input Type

Define your console's input layout in `src/console.rs`:

```rust
use bytemuck::{Pod, Zeroable};
use emberware_core::console::ConsoleInput;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct MyInput {
    pub buttons: u16,  // Bitmask for buttons
    pub stick_x: i8,   // -128..127
    pub stick_y: i8,
}

unsafe impl Pod for MyInput {}
unsafe impl Zeroable for MyInput {}
impl ConsoleInput for MyInput {}

impl MyInput {
    pub fn from_raw(raw: &RawInput) -> Self {
        let mut buttons = 0u16;
        if raw.button_a { buttons |= 0x01; }
        if raw.button_b { buttons |= 0x02; }
        // ... map other buttons

        Self {
            buttons,
            stick_x: (raw.left_stick_x * 127.0) as i8,
            stick_y: (raw.left_stick_y * 127.0) as i8,
        }
    }
}
```

## Step 3: Resource Manager

Create `src/resource_manager.rs`:

```rust
use emberware_core::console::{ConsoleResourceManager, Audio};
use crate::graphics::MyGraphics;
use crate::state::MyFFIState;

pub struct MyResourceManager {
    pub texture_map: hashbrown::HashMap<u32, TextureHandle>,
    pub mesh_map: hashbrown::HashMap<u32, MeshHandle>,
}

impl MyResourceManager {
    pub fn new() -> Self {
        Self {
            texture_map: hashbrown::HashMap::new(),
            mesh_map: hashbrown::HashMap::new(),
        }
    }
}

impl ConsoleResourceManager for MyResourceManager {
    type Graphics = MyGraphics;
    type State = MyFFIState;

    fn process_pending_resources(
        &mut self,
        graphics: &mut Self::Graphics,
        _audio: &mut dyn Audio,
        state: &mut Self::State,
    ) {
        // Load pending textures from state.pending_textures
        // Load pending meshes from state.pending_meshes
        // Store handles in texture_map and mesh_map
    }

    fn execute_draw_commands(
        &mut self,
        graphics: &mut Self::Graphics,
        state: &mut Self::State,
    ) {
        // Execute draw commands from state using texture_map
        graphics.execute_draws(state, &self.texture_map);
    }
}
```

## Step 4: Main Entry Point

Create `src/main.rs`:

```rust
use emberware_core::app::AppMode;

mod console;
mod graphics;
mod audio;
mod ffi;
mod state;
mod resource_manager;
mod ui;
mod settings_ui;
mod library;  // Game discovery

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    // Simple CLI: ./my-console [game_id]
    let mode = if args.len() > 1 {
        AppMode::Playing { game_id: args[1].clone() }
    } else {
        AppMode::Library
    };

    if let Err(e) = app::run(mode) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

## Shared Framework Utilities

The `emberware_core::app` module provides reusable utilities:

### Debug Overlay
```rust
use emberware_core::app::{render_debug_overlay, DebugStats};

// In your render loop:
if debug_overlay_enabled {
    render_debug_overlay(
        &egui_ctx,
        &debug_stats,
        is_playing,
        frame_time_ms,
        render_fps,
        game_tick_fps,
    );
}
```

### FPS Calculation
```rust
use emberware_core::app::calculate_fps;

let fps = calculate_fps(&frame_times);
```

### Frame Time Tracking
```rust
use emberware_core::app::{update_frame_times, FRAME_TIME_HISTORY_SIZE};

let mut frame_times = VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE);
update_frame_times(&mut frame_times, frame_time_ms);
```

### Game Session Management
```rust
use emberware_core::app::session::GameSession;

// Create session
let console = MyConsole::new();
let runtime = Runtime::new(console);
runtime.load_game(game_instance);
runtime.init_game()?;

let resource_manager = console.create_resource_manager();
let session = GameSession::new(runtime, resource_manager);

// In render loop (with proper borrow scoping):
if let Some(game) = session.runtime.game_mut() {
    let state = game.console_state_mut();
    session.resource_manager.process_pending_resources(graphics, audio, state);
    session.resource_manager.execute_draw_commands(graphics, state);
}
```

## What You Need to Implement

### Required

1. **Graphics Backend** - Your rendering pipeline:
   - Implement `Graphics` trait
   - Define vertex formats, shaders
   - Handle texture and mesh loading
   - Execute draw commands

2. **Audio Backend** - Sound playback:
   - Implement `Audio` trait
   - Load and play sounds
   - Support rollback mode (mute during replay)

3. **FFI State** - Staging area for FFI calls:
   - `Default + Send + 'static`
   - Contains pending resources, draw commands, etc.
   - NOT serialized (game state is separate)

4. **FFI Functions** - Console-specific API:
   - Register with `linker.func_wrap()`
   - Write to FFI state, not game state
   - See `docs/ffi.md` for guidelines

5. **Resource Manager** - Handle loading:
   - Map game handles (u32) to backend handles
   - Process pending textures/meshes
   - Execute draw commands

### Optional

6. **UI** - Library and settings screens:
   - Use egui for immediate-mode UI
   - Return `UiAction` enum for user actions
   - See `emberware-z/src/ui.rs` for example

7. **Game Discovery** - Find local games:
   - Scan `~/.emberware/games/`
   - Parse manifest.json files
   - See `emberware-z/src/library.rs`

## Code Reuse vs Console-Specific

### Reuse from Core ✅
- Config management (`emberware_core::app::Config`)
- Input handling (`emberware_core::app::InputManager`)
- Debug overlay (`emberware_core::app::render_debug_overlay`)
- Session management (`emberware_core::app::session::GameSession`)
- Runtime (`emberware_core::runtime::Runtime`)
- Rollback netcode (`emberware_core::rollback`)

### Implement Per-Console 🔧
- Graphics pipeline and shaders
- Audio backend
- FFI functions (draw_*, set_*, etc.)
- UI styling and branding
- Resource loading specifics

## Testing

```bash
# Build your console
cargo build --release --bin my-console

# Run with library UI
./target/release/my-console

# Launch a specific game
./target/release/my-console platformer

# Run tests
cargo test --all
```

## Example Consoles

- **Emberware Z** - Full-featured PS1/N64-style console with:
  - wgpu 3D rendering
  - 4 render modes (Unlit, Matcap, PBR, Hybrid)
  - 8 vertex formats with runtime permutations
  - GPU skinning
  - Procedural sky
  - See `emberware-z/` for complete implementation

- **Test Console** - Minimal console for testing:
  - No-op graphics and audio
  - See `core/src/test_utils.rs`

## Next Steps

1. Copy the Emberware Z structure as a template
2. Implement your `Console` trait
3. Create your graphics and audio backends
4. Design your FFI API (see `docs/ffi.md`)
5. Add UI and game discovery
6. Test with example games

For questions, see:
- `docs/ffi.md` - FFI design patterns
- `docs/console-abstraction-plan.md` - Architecture details
- `emberware-z/` - Reference implementation
