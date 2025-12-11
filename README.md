# Emberware

A fantasy console platform with built-in rollback netcode.

## Consoles

| Console | Generation | Aesthetic | Doc |
|---------|------------|-----------|-----|
| **Emberware Z** | 5th gen | PS1/N64/Saturn | [docs/reference/emberware-z.md](./docs/reference/emberware-z.md) |
| **Emberware Classic** | 4th gen | Genesis/SNES/Neo Geo | [docs/reference/emberware-classic.md](./docs/reference/emberware-classic.md) |

## What's Here

| Directory | Description |
|-----------|-------------|
| `/core` | Shared console framework (WASM runtime, GGRS rollback, debug inspection) |
| `/emberware-z` | Emberware Z runtime implementation |
| `/shared` | API types shared with platform backend, cart/ROM formats |
| `/tools` | Developer tools: `ember-cli` (build/pack/run), `ember-export` (asset conversion) |
| `/docs` | FFI documentation for game developers |
| `/examples` | Example games |

## For Game Developers

See [docs/reference/ffi.md](./docs/reference/ffi.md) for the shared FFI API, then check your target console's specific docs.

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

The `/examples` directory contains several example games demonstrating Emberware Z features. To build and test them:

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

The examples will appear in the Emberware Z game library. Use the refresh button if you add new games while the app is running.

**CLI Launch Features:**
- **Exact matching**: `cargo run -- cube` launches cube
- **Prefix matching**: `cargo run -- plat` launches platformer (if unique)
- **Case-insensitive**: `CUBE`, `Cube`, and `cube` all work
- **Error messages**: Invalid games show suggestions and available games list

**Available examples:** hello-world, triangle, textured-quad, cube, lighting, skinned-mesh, billboard, platformer, blinn-phong, procedural-shapes, audio-demo, textured-procedural, debug-demo, datapack-demo, font-demo, level-loader, asset-test

For more details, see [examples/README.md](./examples/README.md).

### Upload

Visit [emberware.io](https://emberware.io) to create an account and upload your game.

## Console Specs Comparison

| Spec | Emberware Z | Emberware Classic |
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
