# Standalone Player Specification

**Status:** Pending (deferred from combined refactor)
**Author:** Claude
**Last Updated:** December 2025
**Prerequisites:** Combined refactor (console-agnostic architecture) must be completed first

---

## Summary

A minimal binary for running Emberware games without any library UI. Ships as a lightweight player that can be bundled with individual games or used for development.

**Key benefits:**
- ~2-3MB smaller than full library (no egui stack)
- Faster startup for development iteration
- Can be bundled with games for standalone distribution
- Suitable for kiosk/arcade deployments
- Embeddable in other applications

---

## Use Cases

### 1. Game Distribution
Bundle `emberware-player` + `game.ewz` as a standalone download. Players don't need the full library installed.

### 2. Development Iteration
Faster startup than the full library. Run `cargo run -p player -- game.ewz` during development.

### 3. Kiosk/Arcade Mode
No UI to distract - just launches straight into the game. Perfect for arcade cabinets or demo stations.

### 4. Embedding
Other applications can embed the player as a library to run Emberware games within their own UI.

---

## Architecture

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
│  │  ConsoleRunner<C> (reused from core)              │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

**Key principle:** Reuses all game-running infrastructure from the combined refactor. Only the app shell is different (no egui).

---

## Crate Structure

```
player/
├── Cargo.toml
└── src/
    ├── main.rs         # CLI entry point with clap
    └── app.rs          # Minimal event loop
```

---

## Dependencies

```toml
[package]
name = "emberware-player"
version = "0.1.0"
edition = "2021"

[dependencies]
emberware-core = { path = "../core" }
emberware-z = { path = "../emberware-z" }
winit = "0.30"
wgpu = "24"
clap = { version = "4", features = ["derive"] }
log = "0.4"
env_logger = "0.11"

# NO egui - that's the whole point
```

### Dependency Comparison

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

---

## CLI Interface

```bash
# Run ROM file directly
emberware-player path/to/game.ewz

# Run installed game by ID
emberware-player --id cube

# With window options
emberware-player game.ewz --fullscreen
emberware-player game.ewz --scale 3
emberware-player game.ewz --windowed --width 1280 --height 720

# Help
emberware-player --help
```

### CLI Arguments

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "emberware-player")]
#[command(about = "Minimal Emberware game player")]
struct Args {
    /// ROM file path or game ID (with --id flag)
    game: String,

    /// Treat argument as installed game ID instead of file path
    #[arg(long)]
    id: bool,

    /// Run in fullscreen mode
    #[arg(long, short = 'f')]
    fullscreen: bool,

    /// Integer scaling factor (ignored in fullscreen)
    #[arg(long, short = 's', default_value = "2")]
    scale: u32,

    /// Window width (ignored if scale is set)
    #[arg(long)]
    width: Option<u32>,

    /// Window height (ignored if scale is set)
    #[arg(long)]
    height: Option<u32>,

    /// Start paused (for debugging)
    #[arg(long)]
    paused: bool,
}
```

---

## Implementation

### main.rs

```rust
use clap::Parser;
use std::path::PathBuf;

mod app;

#[derive(Parser)]
#[command(name = "emberware-player")]
#[command(about = "Minimal Emberware game player")]
struct Args {
    game: String,

    #[arg(long)]
    id: bool,

    #[arg(long, short = 'f')]
    fullscreen: bool,

    #[arg(long, short = 's', default_value = "2")]
    scale: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let rom_path = if args.id {
        resolve_game_id(&args.game)?
    } else {
        PathBuf::from(&args.game)
    };

    let console_type = ConsoleType::from_extension(
        rom_path.extension().and_then(|s| s.to_str()).unwrap_or("")
    )?;

    app::PlayerApp::run(rom_path, console_type, args.fullscreen, args.scale)
}

fn resolve_game_id(id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Look up in ~/.emberware/games/{id}/
    let library_path = emberware_core::library::library_path()?;
    let game = emberware_core::library::find_game(&library_path, id)?;
    Ok(game.rom_path)
}
```

### app.rs

```rust
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use emberware_core::input::InputManager;
// ActiveGame and create_game come from library crate or a shared location
use crate::registry::{ActiveGame, ConsoleType, create_game};

pub struct PlayerApp {
    window: Option<Arc<Window>>,
    active_game: Option<ActiveGame>,
    input_manager: InputManager,
    rom_path: PathBuf,
    console_type: ConsoleType,
    fullscreen: bool,
    scale: u32,
}

impl PlayerApp {
    pub fn run(
        rom_path: PathBuf,
        console_type: ConsoleType,
        fullscreen: bool,
        scale: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;

        let mut app = Self {
            window: None,
            active_game: None,
            input_manager: InputManager::new(),
            rom_path,
            console_type,
            fullscreen,
            scale,
        };

        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl ApplicationHandler for PlayerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let attrs = WindowAttributes::default()
            .with_title("Emberware Player")
            .with_inner_size(winit::dpi::LogicalSize::new(
                320 * self.scale,
                240 * self.scale,
            ));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        if self.fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        // Create game
        let game = create_game(
            self.console_type,
            window.clone(),
            &self.rom_path,
        ).expect("Failed to create game");

        self.window = Some(window);
        self.active_game = Some(game);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(game) = &mut self.active_game {
                    game.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(game) = &mut self.active_game {
                    game.render();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_manager.handle_keyboard(&event);

                // ESC to quit
                if event.physical_key == winit::keyboard::PhysicalKey::Code(
                    winit::keyboard::KeyCode::Escape
                ) && event.state.is_pressed() {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(game) = &mut self.active_game {
            game.update(&self.input_manager.raw_input());

            if game.quit_requested() {
                std::process::exit(0);
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
```

---

## Code Reuse

Both `library` and `player` reuse from the combined refactor:

| Component | Location | Description |
|-----------|----------|-------------|
| `ConsoleRunner<C>` | `core/src/runner.rs` | Game lifecycle management |
| `ActiveGame` | `library/src/registry.rs` or shared | Static dispatch enum |
| `create_game()` | `library/src/registry.rs` or shared | Console instantiation |
| `InputManager` | `core/src/input.rs` | Input handling |
| `ConsoleType` | `library/src/registry.rs` or shared | Console type enum |

### Sharing Strategy

**Option A: Player depends on library (feature-gated)**
```toml
# player/Cargo.toml
[dependencies]
emberware = { path = "../library", default-features = false, features = ["core-only"] }
```

**Option B: Extract shared types to core**
Move `ConsoleType`, `ActiveGame`, `create_game()` to `core/src/runner.rs` so both library and player can use them without circular dependencies.

**Recommended: Option B** - Keeps player truly minimal and avoids pulling in any library dependencies.

---

## Window Behavior

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ESC | Quit |
| F11 | Toggle fullscreen |
| F3 | Toggle debug overlay (optional) |

### Fullscreen

- `--fullscreen` flag starts in borderless fullscreen
- F11 toggles fullscreen at runtime
- ESC exits (doesn't just exit fullscreen)

### Scaling

- Default: 2x scale (640x480 for 320x240 base)
- `--scale N` sets integer scaling factor
- Window is resizable, game scales to fit

---

## Debug Overlay (Optional)

A minimal debug overlay can be added without egui using simple text rendering:

```
FPS: 60 | Frame: 12345 | Rollback: 0
```

This would use the existing font rendering system from emberware-z, not egui.

**Implementation:** Deferred - can be added later if needed.

---

## Distribution Packaging

### Single-Game Bundle

```
my-game/
├── emberware-player.exe    # or emberware-player on Linux/Mac
├── game.ewz
└── run.bat                 # or run.sh
```

**run.bat:**
```batch
@echo off
emberware-player.exe game.ewz %*
```

**run.sh:**
```bash
#!/bin/bash
./emberware-player game.ewz "$@"
```

### Multi-Game Bundle

```
emberware-arcade/
├── emberware-player.exe
├── games/
│   ├── game1.ewz
│   ├── game2.ewz
│   └── game3.ewz
└── launcher.bat            # Simple menu or auto-rotates games
```

---

## Implementation Plan

### Prerequisites
- [ ] Combined refactor completed (Phase 1-4)
- [ ] `ConsoleRunner<C>` and `ActiveGame` working in library

### Steps

1. **Create crate structure**
   - Create `player/Cargo.toml`
   - Create `player/src/main.rs` and `player/src/app.rs`
   - Add to workspace members

2. **Extract shared types (if needed)**
   - Move `ConsoleType`, `ActiveGame`, `create_game()` to core
   - Update library to use from core
   - Update player to use from core

3. **Implement CLI**
   - Add clap argument parsing
   - Implement game ID resolution
   - Implement ROM path handling

4. **Implement minimal app**
   - Window creation (winit)
   - Event loop without egui
   - Game update/render calls
   - Input handling

5. **Test**
   - `cargo run -p player -- examples/cube/cube.ewz`
   - `cargo run -p player -- --id cube`
   - `cargo run -p player -- game.ewz --fullscreen`
   - Verify binary size is smaller than library

6. **Update workspace**
   - Add player to workspace members
   - Document in README

---

## Success Criteria

- [ ] `cargo run -p player -- game.ewz` launches game
- [ ] `cargo run -p player -- --id cube` launches installed game
- [ ] `--fullscreen` flag works
- [ ] `--scale N` flag works
- [ ] ESC quits the game
- [ ] Binary size is 2-3MB smaller than library
- [ ] No egui dependencies in player crate
- [ ] All games that work in library also work in player

---

## Future Enhancements

- **Controller support** - Already handled by InputManager
- **Debug overlay** - Simple FPS/frame counter without egui
- **Save state shortcuts** - F5/F9 for quick save/load
- **Screenshot** - F12 to save PNG
- **Recording** - Built-in replay recording
- **Netplay CLI** - `--connect peer-id` for direct P2P games
