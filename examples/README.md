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

### Basic Examples

- **hello-world** - Minimal example showing basic setup
- **triangle** - Single colored triangle
- **textured-quad** - Textured rectangle
- **cube** - Rotating 3D cube

### Graphics Examples

- **lighting** - PBR lighting demo
- **skinned-mesh** - Skeletal animation
- **billboard** - Billboard sprites
- **blinn-phong** - Blinn-Phong lighting demo
- **procedural-shapes** - Procedurally generated shapes
- **textured-procedural** - Textured procedural geometry

### Audio Example

- **audio-demo** - Audio playback demonstration

### Game Examples

- **platformer** - Simple platformer game

### Data Pack Examples

These examples demonstrate the ROM data pack system using `rom_*` FFI functions:

- **datapack-demo** - Full workflow: textures, meshes, and sounds from data pack
- **font-demo** - Loading and using bitmap fonts from data pack
- **level-loader** - Loading custom binary level data with `rom_data()`
- **asset-test** - Loading pre-converted `.ewzmesh` and `.ewztex` assets

## Building Individual Examples

To build a single example:

```bash
cd examples/hello-world
cargo build --target wasm32-unknown-unknown --release
```

The WASM file will be in `target/wasm32-unknown-unknown/release/`.

## Data Pack Workflow

Examples using data packs require the `ember` CLI tool:

```bash
# 1. Build the WASM
ember build

# 2. Bundle assets into data pack (reads ember.toml)
ember pack

# 3. Launch in emulator
ember run
```

See `ember.toml` in each data pack example for the manifest format.

### rom_* FFI Functions

Data pack assets bypass WASM memory and go directly to VRAM/audio:

```rust
fn init() {
    // Load from data pack (init-only)
    let tex = rom_texture(b"player".as_ptr(), 6);
    let mesh = rom_mesh(b"enemy".as_ptr(), 5);
    let sfx = rom_sound(b"jump".as_ptr(), 4);
    let font = rom_font(b"ui_font".as_ptr(), 7);

    // Raw data is copied to WASM memory
    let len = rom_data_len(b"level1".as_ptr(), 6);
    let mut buf = vec![0u8; len as usize];
    rom_data(b"level1".as_ptr(), 6, buf.as_mut_ptr(), len);
}
```
