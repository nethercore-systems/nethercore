# Emberware Examples

Example games demonstrating Emberware Z features.

## Building and Installing Examples

To build all examples and install them to your local Emberware library:

```bash
cargo xtask build-examples
```

This will:
1. Compile each example to WASM (targeting `wasm32-unknown-unknown`)
2. Install them to `~/.emberware/games/` (or platform-specific equivalent)
3. Generate a manifest for each game

### Prerequisites

Make sure you have the WASM target installed:

```bash
rustup target add wasm32-unknown-unknown
```

## Running Examples

After building, simply run Emberware Z:

```bash
cargo run
```

The examples will appear in the game library.

## Available Examples

- **hello-world** - Minimal example showing basic setup
- **triangle** - Single colored triangle
- **textured-quad** - Textured rectangle
- **cube** - Rotating 3D cube
- **lighting** - PBR lighting demo
- **skinned-mesh** - Skeletal animation
- **billboard** - Billboard sprites
- **platformer** - Simple platformer game
- **blinn-phong** - Blinn-Phong lighting demo
- **procedural-shapes** - Procedurally generated shapes
- **audio-demo** - Audio playback demonstration
- **textured-procedural** - Textured procedural geometry

## Building Individual Examples

To build a single example:

```bash
cd examples/hello-world
cargo build --target wasm32-unknown-unknown --release
```

The WASM file will be in `target/wasm32-unknown-unknown/release/`.
