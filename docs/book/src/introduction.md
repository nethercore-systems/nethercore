# Nethercore ZX API Reference

Nethercore ZX is a 5th-generation fantasy console targeting PS1/N64/Saturn aesthetics with modern conveniences like deterministic rollback netcode.

## Console Specs

| Spec | Value |
|------|-------|
| **Aesthetic** | PS1/N64/Saturn (5th gen) |
| **Resolution** | 960×540 (fixed, upscaled to display) |
| **Color depth** | RGBA8 |
| **Tick rate** | 24, 30, 60 (default), 120 fps |
| **ROM (Cartridge)** | 16MB (WASM code + data pack assets) |
| **RAM** | 4MB (WASM linear memory for game state) |
| **VRAM** | 4MB (GPU textures and mesh buffers) |
| **Compute budget** | WASM GAS metering |
| **Netcode** | Deterministic rollback via GGRS |
| **Max players** | 4 (any mix of local + remote) |

## Game Lifecycle

Games export three functions:

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Called once at startup
    // Load resources, configure console settings
}

#[no_mangle]
pub extern "C" fn update() {
    // Called every tick (deterministic for rollback)
    // Update game state, handle input
}

#[no_mangle]
pub extern "C" fn render() {
    // Called every frame (skipped during rollback replay)
    // Draw to screen
}
```

## Memory Model

Nethercore ZX uses a **16MB ROM + 4MB RAM** memory model:

- **ROM (16MB)**: WASM bytecode + data pack (textures, meshes, sounds)
- **RAM (4MB)**: WASM linear memory for game state
- **VRAM (4MB)**: GPU textures and mesh buffers

Assets loaded via `rom_*` functions go directly to VRAM/audio memory, keeping RAM free for game state.

## API Categories

| Category | Description |
|----------|-------------|
| [System](./api/system.md) | Time, logging, random, session info |
| [Input](./api/input.md) | Buttons, sticks, triggers |
| [Graphics](./api/graphics.md) | Resolution, render mode, state |
| [Camera](./api/camera.md) | View and projection |
| [Transforms](./api/transforms.md) | Matrix stack operations |
| [Textures](./api/textures.md) | Loading and binding textures |
| [Meshes](./api/meshes.md) | Loading and drawing meshes |
| [Materials](./api/materials.md) | PBR and Blinn-Phong properties |
| [Lighting](./api/lighting.md) | Directional and point lights |
| [Skinning](./api/skinning.md) | Skeletal animation |
| [Animation](./api/animation.md) | Keyframe playback |
| [Procedural](./api/procedural.md) | Generated primitives |
| [2D Drawing](./api/drawing-2d.md) | Sprites, text, rectangles |
| [Billboards](./api/billboards.md) | Camera-facing quads |
| [Environment (EPU)](./api/epu.md) | Procedural environments |
| [Audio](./api/audio.md) | Sound effects and music |
| [Save Data](./api/save-data.md) | Persistent storage |
| [ROM Loading](./api/rom-loading.md) | Data pack access |
| [Debug](./api/debug.md) | Runtime value inspection |

## Screen Capture

The host application includes screenshot and GIF recording capabilities:

| Key | Default | Action |
|-----|---------|--------|
| Screenshot | **F9** | Save PNG to screenshots folder |
| GIF Toggle | **F10** | Start/stop GIF recording |

Files are saved to:
- Screenshots: `~/.nethercore/Nethercore/screenshots/`
- GIFs: `~/.nethercore/Nethercore/gifs/`

Filenames include game name and timestamp (e.g., `platformer_screenshot_2025-01-15_14-30-45.png`).

**Configuration** (`~/.nethercore/config.toml`):
```toml
[capture]
screenshot = "F9"
gif_toggle = "F10"
gif_fps = 30          # GIF framerate
gif_max_seconds = 60  # Max duration
```

## Quick Links

- [Cheat Sheet](./cheat-sheet.md) - All functions on one page
- [Getting Started](./getting-started.md) - Your first game
- [Render Modes](./guides/render-modes.md) - Mode 0-3 explained
- [Rollback Safety](./guides/rollback-safety.md) - Writing deterministic code

---

## Building These Docs

These docs are built with [mdBook](https://rust-lang.github.io/mdBook/).

```bash
# Install mdBook
cargo install mdbook

# Build static HTML (outputs to docs/book/book/)
cd docs/book
mdbook build

# Or serve locally with live reload
mdbook serve
```
