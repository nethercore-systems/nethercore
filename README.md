# Nethercore

A fantasy console platform with built-in rollback netcode.

## Installation

### Pre-built Binaries (Recommended)

Download the latest build for your platform from the [Releases page](https://github.com/nethercore-systems/nethercore/releases/latest):

- **Windows**: `nethercore-windows-x86_64.zip`
- **macOS (Intel)**: `nethercore-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `nethercore-macos-aarch64.tar.gz`
- **Linux**: `nethercore-linux-x86_64.tar.gz`

Each package contains:
- `nethercore` — Library app for browsing and playing games
- `nether` — CLI tool for building, packing, and running games

Releases are published via CI; see the Releases page for the most recent build.

**After extracting:**

```bash
# Add to PATH (recommended)
# Linux/macOS
export PATH="$PATH:/path/to/nethercore"

# Windows
# Add the extracted folder to your PATH environment variable

# Or run directly
./nethercore  # Launch library
./nether --help  # CLI tool
```

### Build from Source

If you prefer to build from source:

```bash
# Clone and build
git clone https://github.com/nethercore-systems/nethercore
cd nethercore
cargo build --release -p nethercore-library -p nether-cli

# Binaries will be in target/release/
```

**To build and run the examples:**

```bash
# Add WASM target (needed for compiling games)
rustup target add wasm32-unknown-unknown

# Build all examples
cargo xtask build-examples

# Run the library to browse examples
cargo run
```

## Consoles

| Console | Generation | Aesthetic | Status | Doc |
|---------|------------|-----------|--------|-----|
| **Nethercore ZX** | 5th gen | PS1/N64/Saturn | Available | [docs/book/](./docs/book/) |
| **Nethercore Chroma** | 4th gen | Genesis/SNES/Neo Geo | Coming Soon | — |

## What's Here

| Directory | Description |
|-----------|-------------|
| `/core` | Shared console framework (WASM runtime, GGRS rollback, debug inspection) |
| `/nethercore-zx` | Nethercore ZX runtime implementation |
| `/shared` | API types shared with platform backend, cart/ROM formats |
| `/tools` | Developer tools: `nether-cli` (build/pack/run), `nether-export` (asset conversion) |
| `/docs/book` | Game developer documentation (mdBook) |
| `/docs/architecture` | Internal architecture (FFI, rendering, ROM format) |
| `/docs/contributing` | Contributor guides |
| `/examples` | Example games |

## For Game Developers

Docs index: [docs/README.md](./docs/README.md).

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
# Create a manifest (first time only)
nether init

# Build and package your game
nether build

# Build and run in emulator
nether run
```

### Testing with Examples

The `/examples` directory contains several example games demonstrating Nethercore ZX features. To build and test them:

**Prerequisites:**
```bash
rustup target add wasm32-unknown-unknown
```

**Build all examples:**
```bash
cargo xtask build-examples
```

This will:
1. Build the Rust examples under `/examples` (directories with `Cargo.toml`)
2. Install them into your Nethercore data directory under `games/` (the command prints the exact path)
3. Generate a manifest for each installed example

**Run the examples:**
```bash
# Launch game library UI
cargo run

# Or launch a game directly (faster for development)
cargo run -- platformer      # Launch by full name
cargo run -- plat            # Launch by prefix match
cargo run -- PADDLE          # Case-insensitive
```

The examples will appear in the Nethercore ZX game library. Use the refresh button if you add new games while the app is running.

**CLI Launch Features:**
- **Exact matching**: `cargo run -- paddle` launches paddle
- **Prefix matching**: `cargo run -- plat` launches platformer (if unique)
- **Case-insensitive**: `PADDLE`, `Paddle`, and `paddle` all work
- **Error messages**: Invalid games show suggestions and available games list

The `/examples` directory contains dozens of example games covering graphics, animation, audio, and more.

See [examples/README.md](./examples/README.md) for the complete list with descriptions.

## Preview Mode

Browse ROM assets without running game code:

```bash
# Using nether CLI
nether preview paddle

# Using library launcher
cargo run -- paddle --preview

# Focus on specific asset
nether preview paddle --asset player_texture
cargo run -- paddle --preview --asset player_texture
```

Preview mode supports:
- **Textures** - View with zoom/pan controls
- **Sounds** - Waveform visualization with playback
- **Meshes** - 3D mesh inspection
- **Animations** - Skeletal animation preview
- **Fonts** - Font character inspection
- **Trackers** - Music tracker data
- **Data** - Raw data inspection

Smart ROM lookup works in preview mode:
- `nether preview pad` matches `paddle.nczx` (prefix)
- `nether preview PADDLE` works (case-insensitive)
- Typo suggestions: `nether preview padde` suggests `paddle`

## Screen Capture

Nethercore includes built-in screenshot and GIF recording:

| Key | Default | Description |
|-----|---------|-------------|
| Screenshot | **F9** | Save PNG to your Nethercore data directory under `screenshots/` |
| GIF Toggle | **F10** | Start/stop GIF recording, saves to your Nethercore data directory under `gifs/` |

Filenames include the game name and timestamp: `platformer_screenshot_2025-01-15_14-30-45.png`

### Configuration

All capture keys are configurable in `config.toml` (stored in your platform-specific config directory):

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
cargo install mdbook mdbook-tabs

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

Visit [nethercore.systems](https://nethercore.systems) to create an account and upload your game.

## Console Specs Comparison

*Note: Nethercore Chroma is documented but not yet implemented.*

| Spec | Nethercore ZX | Nethercore Chroma |
|------|-------------|-------------------|
| Generation | 5th (PS1/N64) | 4th (Genesis/SNES) |
| Target audience | Experienced devs | Beginners, students |
| Resolution | 960×540 (fixed) | 256×192 (fixed) |
| RAM | 4MB | 1MB |
| VRAM | 4MB | — |
| ROM size | 16MB | 4MB |
| 3D support | Yes | No |
| Analog input | Sticks + triggers | D-pad only |
| Face buttons | 4 (A/B/X/Y) | 4 (A/B/X/Y) |
| Tilemap layers | No | Yes (2 layers) |
| Sprite flip/priority | No | Yes |
| Palette swapping | No | No (64-color fixed) |
| Tick rate | 24-120 fps | 60 fps |
| Max players | 4 | 4 |
| Netcode | Rollback (GGRS) | Rollback (GGRS) |

## Community

- [Discord](https://discord.gg/EaSuKMDF) — Chat, get help, share your games
- [GitHub Discussions](https://github.com/nethercore-systems/nethercore/discussions) — Longer-form discussions

## License

Dual-licensed under MIT OR Apache-2.0 (your choice).
