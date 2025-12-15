# Architecture Refactor: Separate Library from Players

> **Note:** This plan supersedes `docs/pending/standalone-player-spec.md`. The key difference: instead of a shared player binary with `ActiveGame` enum dispatch, each console has its own player binary, eliminating cross-console coupling entirely.

## Overview

Refactor Emberware to separate the **library UI** (game browser/launcher) from **console players** (game runtime). This eliminates the `ActiveGame` enum, removes all console-specific code from the library, and enables standalone players.

## Current Architecture (Problem)

```
library/
├── App struct
│   ├── active_game: Option<ActiveGame>  ← enum with 30+ match arms
│   ├── input_manager: InputManager      ← Z-specific
│   └── ... graphics, audio, WASM ...    ← console-specific
```

**Issues:**
- Library is tightly coupled to console implementations
- Adding a new console requires updating 30+ match arms
- Can't have a standalone player without duplicating code
- Can't replace library UI without rewriting everything

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  emberware-library (binary)                 │
│                                                             │
│  • Scan ~/.emberware/games/ for ROMs (all console types)   │
│  • Display game list in egui UI                             │
│  • On "Play" → spawn correct player process                │
│  • Settings, downloads, game management                     │
│  • ZERO console-specific imports                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                    std::process::Command::new(player)
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│ emberware-z      │ │ emberware-classic│ │ emberware-y      │
│ (player binary)  │ │ (player binary)  │ │ (player binary)  │
│                  │ │                  │ │                  │
│ • ConsoleRunner  │ │ • ConsoleRunner  │ │ • ConsoleRunner  │
│ • Graphics/Audio │ │ • Graphics/Audio │ │ • Graphics/Audio │
│ • Input handling │ │ • Input handling │ │ • Input handling │
│ • GGRS rollback  │ │ • GGRS rollback  │ │ • GGRS rollback  │
│ • Debug overlay  │ │ • Debug overlay  │ │ • Debug overlay  │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

## Benefits

| Aspect | Before | After |
|--------|--------|-------|
| **Console coupling** | 30+ match arms in library | Zero console imports |
| **Adding console** | Update library + enum + 30 methods | Create new player binary |
| **Standalone player** | Not possible | `emberware-z game.ewz` works |
| **Crash isolation** | Game crash = library crash | Game crash = player crash only |
| **UI replacement** | Rewrite everything | Swap library, keep players |
| **Testing** | Complex integration | Test players independently |

## Implementation Plan

### Phase 1: Create Standalone Player for Emberware Z

**Goal:** `emberware-z.exe path/to/game.ewz` launches and plays the game.

#### 1.1 Add binary target to emberware-z

**File:** `emberware-z/Cargo.toml`

```toml
[[bin]]
name = "emberware-z"
path = "src/bin/main.rs"

[features]
default = ["player"]
player = ["dep:winit", "dep:egui", "dep:egui-winit", "dep:egui-wgpu", "dep:clap"]
```

#### 1.2 Create player binary

**File:** `emberware-z/src/bin/main.rs`

```rust
use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "emberware-z")]
#[command(about = "Emberware Z - PS1/N64 aesthetic fantasy console")]
struct Args {
    /// ROM file to play (.ewz)
    rom: PathBuf,

    /// Enable debug overlay
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    emberware_z::player::run(args.rom, args.debug)
}
```

#### 1.3 Create player module

**File:** `emberware-z/src/player.rs`

Move/adapt the game-running logic from `library/src/app/`:
- Window creation
- ConsoleRunner initialization
- Game loop (input → update → render)
- Debug overlay
- Frame controller

This is essentially what `library/src/app/game_session.rs` does, but standalone.

#### 1.4 Reuse from ember-cli

`tools/ember-cli/src/run.rs` already implements running a game. Extract shared logic:

**File:** `core/src/player.rs` (new)

```rust
/// Shared player logic used by standalone players and ember-cli
pub fn run_game<C: Console>(
    console: C,
    rom_path: &Path,
    debug: bool,
) -> Result<()> {
    // Window creation
    // ConsoleRunner setup
    // Game loop
    // Debug overlay
}
```

### Phase 2: Simplify Library

**Goal:** Library becomes pure launcher with zero console-specific code.

#### 2.1 Remove console dependencies from library

**File:** `library/Cargo.toml`

```toml
[dependencies]
# REMOVE these:
# emberware-z = { path = "../emberware-z" }
# z-common = { path = "../z-common" }

# KEEP these:
emberware-core = { path = "../core" }  # For LocalGame, DataDirProvider
egui = "0.30"
egui-winit = "0.30"
egui-wgpu = "0.30"
# ...
```

#### 2.2 Simplify App struct

**File:** `library/src/app/mod.rs`

```rust
pub struct App {
    // UI state
    pub(crate) egui_ctx: egui::Context,
    pub(crate) library_ui: LibraryUi,
    pub(crate) settings_ui: SettingsUi,

    // Data
    pub(crate) local_games: Vec<LocalGame>,
    pub(crate) config: Config,

    // Window
    pub(crate) window: Option<Arc<Window>>,

    // REMOVED: active_game, input_manager, debug_panel, frame_controller, etc.
}
```

#### 2.3 Replace game launching with process spawn

**File:** `library/src/app/game_session.rs` → **DELETE** (no longer needed)

**File:** `library/src/app/ui.rs`

```rust
fn launch_game(&self, game: &LocalGame) -> Result<()> {
    let player = self.get_player_binary(&game.console_type)?;
    let rom_path = self.get_rom_path(game)?;

    std::process::Command::new(player)
        .arg(&rom_path)
        .spawn()
        .context("Failed to launch player")?;

    Ok(())
}

fn get_player_binary(&self, console_type: &str) -> Result<PathBuf> {
    // Look for player binary in:
    // 1. Same directory as library executable
    // 2. PATH
    // 3. Known install locations

    let name = match console_type {
        "z" => "emberware-z",
        "classic" => "emberware-classic",
        _ => bail!("Unknown console type: {}", console_type),
    };

    // Try same directory first
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().unwrap();
        let player = dir.join(format!("{}{}", name, std::env::consts::EXE_SUFFIX));
        if player.exists() {
            return Ok(player);
        }
    }

    // Fall back to PATH
    Ok(PathBuf::from(name))
}
```

#### 2.4 Delete ActiveGame enum

**File:** `library/src/registry.rs`

Delete entire `ActiveGame` enum and all 30+ methods. Keep only:
- `ConsoleType` enum (for parsing manifest console_type field)
- `ConsoleRegistry` (for listing available consoles)
- `create_rom_loader_registry()` (for scanning ROMs)

### Phase 3: Update ember-cli

**File:** `tools/ember-cli/src/run.rs`

Use shared player logic from `core/src/player.rs`:

```rust
pub fn run(args: &RunArgs) -> Result<()> {
    // Detect console type from extension
    let console_type = detect_console_type(&args.rom)?;

    match console_type {
        "z" => {
            let console = EmberwareZ::new();
            emberware_core::player::run_game(console, &args.rom, args.debug)
        }
        _ => bail!("Unknown ROM type"),
    }
}
```

### Phase 4: File Association & Deep Links

#### 4.1 Register file associations (optional, platform-specific)

- `.ewz` → `emberware-z`
- `.ewc` → `emberware-classic`

#### 4.2 Deep links

`emberware://play/game-id` → Library looks up game, spawns correct player

---

## File Changes Summary

### New Files

| File | Purpose |
|------|---------|
| `emberware-z/src/bin/main.rs` | Standalone player entry point |
| `emberware-z/src/player.rs` | Player logic (window, game loop, debug) |
| `core/src/player.rs` | Shared player infrastructure |

### Modified Files

| File | Changes |
|------|---------|
| `emberware-z/Cargo.toml` | Add `[[bin]]` target |
| `library/Cargo.toml` | Remove emberware-z, z-common deps |
| `library/src/app/mod.rs` | Simplify App struct, remove game runtime |
| `library/src/app/ui.rs` | Replace direct play with process spawn |
| `library/src/registry.rs` | Delete ActiveGame enum |
| `tools/ember-cli/src/run.rs` | Use shared player logic |

### Deleted Files

| File | Reason |
|------|--------|
| `library/src/app/game_session.rs` | Logic moves to player |
| `library/src/app/debug.rs` | Logic moves to player |

---

## Migration Strategy

### Step 1: Create player without breaking library
1. Add `emberware-z/src/bin/main.rs`
2. Add `emberware-z/src/player.rs`
3. Test: `cargo run -p emberware-z -- examples/cube/cube.ewz`

### Step 2: Extract shared player logic
1. Create `core/src/player.rs`
2. Refactor emberware-z player to use it
3. Refactor ember-cli to use it

### Step 3: Simplify library
1. Add process spawning to library
2. Remove ActiveGame enum
3. Remove console-specific imports
4. Test library → player flow

### Step 4: Clean up
1. Delete dead code
2. Update documentation
3. Update CI/CD to build all binaries

---

## Future Considerations

### Adding Emberware Classic

1. Create `emberware-classic/` crate with `Console` implementation
2. Add `[[bin]]` target for `emberware-classic`
3. Library automatically detects `.ewc` files and spawns correct player
4. **Zero changes to library code**

### Web-based Library UI

1. Build new UI with web tech (Tauri, Electron, pure web)
2. Same mechanism: spawn player process on game select
3. Players unchanged

### Networked Multiplayer

1. Player handles all networking (GGRS)
2. Library could show online status, matchmaking UI
3. Launch player with network args: `emberware-z game.ewz --connect 192.168.1.5:7000`

---

## Player CLI Interface

Each player binary accepts consistent CLI arguments:

```bash
# Run ROM file directly
emberware-z path/to/game.ewz

# With options
emberware-z game.ewz --fullscreen
emberware-z game.ewz --scale 3
emberware-z game.ewz --debug

# Netplay (future)
emberware-z game.ewz --connect 192.168.1.5:7000
```

### CLI Arguments (clap)

```rust
#[derive(Parser)]
#[command(name = "emberware-z")]
#[command(about = "Emberware Z - PS1/N64 aesthetic fantasy console")]
struct Args {
    /// ROM file to play (.ewz)
    rom: PathBuf,

    /// Run in fullscreen mode
    #[arg(long, short = 'f')]
    fullscreen: bool,

    /// Integer scaling factor
    #[arg(long, short = 's', default_value = "2")]
    scale: u32,

    /// Enable debug overlay
    #[arg(long, short = 'd')]
    debug: bool,
}
```

### Keyboard Shortcuts (in player)

| Key | Action |
|-----|--------|
| ESC | Quit |
| F3 | Toggle debug overlay |
| F5 | Pause/Resume |
| F6 | Frame step (when paused) |
| F11 | Toggle fullscreen |

---

## Distribution Packaging

### Standalone Game Bundle

```
my-game/
├── emberware-z.exe        # Player binary
├── game.ewz               # ROM file
└── run.bat                # Optional launcher
```

### Library + Players Bundle

```
emberware/
├── emberware-library.exe  # Library UI
├── emberware-z.exe        # Z player
├── emberware-classic.exe  # Classic player (future)
└── data/
    └── games/             # Installed games
```

---

## Verification Checklist

- [ ] `emberware-z examples/cube/cube.ewz` - Standalone player works
- [ ] `ember run examples/cube/cube.ewz` - CLI still works
- [ ] Library shows games for all console types
- [ ] Library launches correct player for each console type
- [ ] Player crash doesn't affect library
- [ ] Debug overlay works in player
- [ ] Frame controller (pause/step) works in player
- [ ] ESC quits player
- [ ] F11 toggles fullscreen

---

## Cleanup

After implementation, delete the old spec:
- [ ] Delete `docs/pending/standalone-player-spec.md` (superseded by this document)
