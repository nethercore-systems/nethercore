# Emberware

A fantasy console platform with built-in rollback netcode.

## Consoles

| Console | Generation | Aesthetic | Status | Doc |
|---------|------------|-----------|--------|-----|
| **Emberware ZX** | 5th gen | PS1/N64/Saturn | Available | [docs/book/](./docs/book/) |
| **Emberware Chroma** | 4th gen | Genesis/SNES/Neo Geo | Coming Soon | — |

## What's Here

| Directory | Description |
|-----------|-------------|
| `/core` | Shared console framework (WASM runtime, GGRS rollback, debug inspection) |
| `/emberware-zx` | Emberware ZX runtime implementation |
| `/shared` | API types shared with platform backend, cart/ROM formats |
| `/tools` | Developer tools: `ember-cli` (build/pack/run), `ember-export` (asset conversion) |
| `/docs/book` | Game developer documentation (mdBook) |
| `/docs/architecture` | Internal architecture (FFI, rendering, ROM format) |
| `/docs/contributing` | Contributor guides |
| `/examples` | Example games |

## For Game Developers

See [docs/architecture/ffi.md](./docs/architecture/ffi.md) for the shared FFI API, or browse the [mdBook documentation](./docs/book/).

### Quick Start

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Called once at startup
}

#[no_mangle]
pub extern "C" fn update() {
    // Called every tick — game logic (deterministic!)
}

#[no_mangle]
pub extern "C" fn render() {
    // Called every frame — draw calls (skipped during rollback)
}
```

### Build Your Game

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Testing with Examples

The `/examples` directory contains several example games demonstrating Emberware ZX features. To build and test them:

**Prerequisites:**
```bash
rustup target add wasm32-unknown-unknown
```

**Build all examples:**
```bash
cargo xtask build-examples
```

This will:
1. Compile each example to WASM (targeting `wasm32-unknown-unknown`)
2. Install them to `~/.emberware/games/` (or platform-specific equivalent)
3. Generate a manifest for each game

**Run the examples:**
```bash
# Launch game library UI
cargo run

# Or launch a game directly (faster for development)
cargo run -- platformer      # Launch by full name
cargo run -- plat            # Launch by prefix match
cargo run -- CUBE            # Case-insensitive
```

The examples will appear in the Emberware ZX game library. Use the refresh button if you add new games while the app is running.

**CLI Launch Features:**
- **Exact matching**: `cargo run -- cube` launches cube
- **Prefix matching**: `cargo run -- plat` launches platformer (if unique)
- **Case-insensitive**: `CUBE`, `Cube`, and `cube` all work
- **Error messages**: Invalid games show suggestions and available games list

The `/examples` directory contains 28 example games covering graphics, animation, audio, and more.

See [examples/README.md](./examples/README.md) for the complete list with descriptions.

## Screen Capture

Emberware includes built-in screenshot and GIF recording:

| Key | Default | Description |
|-----|---------|-------------|
| Screenshot | **F9** | Save PNG to `~/.emberware/Emberware/screenshots/` |
| GIF Toggle | **F10** | Start/stop GIF recording, saves to `~/.emberware/Emberware/gifs/` |

Filenames include the game name and timestamp: `platformer_screenshot_2025-01-15_14-30-45.png`

### Configuration

All capture keys are configurable in `~/.emberware/config.toml`:

```toml
[capture]
screenshot = "F9"
gif_toggle = "F10"
gif_fps = 30          # Recording framerate (default: 30)
gif_max_seconds = 60  # Max duration before auto-stop
```

## Documentation

Searchable API documentation is available as an mdBook:

```bash
# Install mdBook
cargo install mdbook

# Build and serve locally
cd docs/book
mdbook serve
# Opens at http://localhost:3000
```

**Quick References:**
- [Cheat Sheet](./docs/book/src/cheat-sheet.md) — All ~116 functions at a glance
- [Getting Started](./docs/book/src/getting-started.md) — Your first game
- [API Reference](./docs/book/src/) — Full documentation

### Upload

Visit [emberware.io](https://emberware.io) to create an account and upload your game.

## Console Specs Comparison

*Note: Emberware Classic is documented but not yet implemented.*

| Spec | Emberware ZX | Emberware Classic |
|------|-------------|-------------------|
| Generation | 5th (PS1/N64) | 4th (Genesis/SNES) |
| Target audience | Experienced devs | Beginners, students |
| Resolution | 360p-1080p | 8 options (16:9 + 4:3, pixel-perfect) |
| RAM | 4MB | 4MB |
| VRAM | 4MB | 2MB |
| ROM size | 12MB | 16MB |
| 3D support | Yes | No |
| Analog input | Sticks + triggers | D-pad only |
| Face buttons | 4 (A/B/X/Y) | 6 (A/B/C/X/Y/Z) |
| Tilemap layers | No | Yes (4 layers) |
| Sprite flip/priority | No | Yes |
| Palette swapping | No | Yes |
| Tick rate | 24-120 fps | 24-120 fps |
| Max players | 4 | 4 |
| Netcode | Rollback (GGRS) | Rollback (GGRS) |

## License

Dual-licensed under MIT OR Apache-2.0 (your choice).
