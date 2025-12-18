# Emberware ZX Examples

Example games demonstrating Emberware ZX features. Each example is a standalone WASM game showcasing specific APIs and techniques.

## Building and Running

**Prerequisites:**
```bash
rustup target add wasm32-unknown-unknown
```

**Build all examples:**
```bash
cargo xtask build-examples
```

This will compile each example to WASM and install them to `~/.emberware/games/`.

**Run examples:**
```bash
# Launch library UI
cargo run

# Or launch directly by name
cargo run -- hello-world
cargo run -- platformer
cargo run -- lighting
```

---

## Available Examples

### Getting Started

| Example | Description |
|---------|-------------|
| **hello-world** | Basic 2D drawing with `draw_text`, `draw_rect`, input handling |
| **triangle** | Minimal 3D — single colored triangle |
| **textured-quad** | Textured sprite rendering with `load_texture`, `texture_bind` |
| **cube** | Rotating textured cube with transforms |

### Graphics & Rendering

| Example | Description |
|---------|-------------|
| **lighting** | PBR lighting (mode 2) with 4 dynamic lights, sky, materials |
| **blinn-phong** | Classic Blinn-Phong (mode 3) with specular and rim lighting |
| **billboard** | GPU-instanced billboards and sprites |
| **procedural-shapes** | Built-in generators: `cube()`, `sphere()`, `cylinder()`, etc. |
| **textured-procedural** | Procedural shapes with texture mapping |
| **dither-demo** | PS1-style ordered dithering effects |
| **material-override** | Per-draw material property overrides |

### Render Mode Inspectors

| Example | Description |
|---------|-------------|
| **mode0-inspector** | Interactive inspector for Unlit mode |
| **mode1-inspector** | Interactive inspector for Matcap mode (blend modes) |
| **mode2-inspector** | Interactive inspector for MR-Blinn-Phong mode |
| **mode3-inspector** | Interactive inspector for Blinn-Phong mode |

### Animation & Skinning

| Example | Description |
|---------|-------------|
| **skinned-mesh** | GPU skeletal animation basics with `set_bones` |
| **animation-demo** | Keyframe animation playback from ROM data |
| **ik-demo** | Inverse kinematics (procedural animation) |
| **multi-skinned-procedural** | Multiple animated characters (procedural bones) |
| **multi-skinned-rom** | Multiple animated characters (ROM data) |
| **skeleton-stress-test** | Performance testing with many skeletons |

### Audio

| Example | Description |
|---------|-------------|
| **audio-demo** | Sound effects, panning, channels, looping |

### Asset Loading (Data Packs)

| Example | Description |
|---------|-------------|
| **datapack-demo** | Full `rom_*` workflow: textures, meshes, sounds |
| **font-demo** | Custom font loading with `rom_font` |
| **level-loader** | Loading level data with `rom_data` |
| **asset-test** | Pre-converted `.ewzmesh` and `.ewztex` assets |

### Complete Games

| Example | Description |
|---------|-------------|
| **platformer** | Full mini-game: physics, collision, multiple players |

### Development Tools

| Example | Description |
|---------|-------------|
| **debug-demo** | Debug inspection system (F3 panel, frame control) |

### Shared Library

| Directory | Description |
|-----------|-------------|
| **examples-common** | Shared utilities: `DebugCamera`, `StickControl`, math |

---

## Example Details

### hello-world
Basic 2D drawing with text and rectangles. Controls: D-pad moves square, A resets.

### lighting
Full PBR demo with interactive material and light controls:
- Left stick: Rotate sphere
- Right stick: Move light
- Triggers: Metallic (LT) / Roughness (RT)
- D-pad: Light intensity
- A/B/X/Y: Toggle lights

### skinned-mesh
GPU skeletal animation with 3x4 bone matrices and smooth weight blending.
- Left stick: Rotate view
- A: Toggle animation
- D-pad: Animation speed

### audio-demo
Audio system demo with panning and channel control:
- Left/Right: Adjust pan
- A: Play sound
- B: Stop sound

### platformer
Complete mini-game demonstrating:
- 2D gameplay using 3D renderer
- Billboarded sprites
- Simple physics (gravity, friction)
- AABB collision detection
- Multiple players
- 2D UI overlay

### debug-demo
Debug inspection system demo:
- F3: Toggle debug panel
- F5: Pause/resume
- F6: Step frame
- F7/F8: Time scale

---

## Data Pack Workflow

Examples using data packs require the `ember` CLI:

```bash
# 1. Build WASM
ember build

# 2. Bundle assets (reads ember.toml)
ember pack

# 3. Launch game
ember run
```

### rom_* FFI Functions

Data pack assets bypass WASM memory, going directly to VRAM/audio:

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

See `ember.toml` in each data pack example for the manifest format.

---

## Building Individual Examples

```bash
cd examples/hello-world
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/hello_world.wasm`

---

## Further Reading

- [FFI Reference](../docs/architecture/ffi.md) — Shared API documentation
- [Game Developer Book](../docs/book/) — Full API documentation (mdBook)
- [Rendering Architecture](../docs/architecture/zx/rendering.md) — ZX graphics deep dive
