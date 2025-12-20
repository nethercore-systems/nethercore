# Example Games

The Emberware repository includes 28+ example games organized by category. Each example is a complete, buildable project.

## Location

```
emberware/examples/
```

## Learning Path

New to Emberware? Follow this progression:

1. **[hello-world](https://github.com/emberware/emberware/tree/main/examples/hello-world)** - 2D text and rectangles, basic input
2. **[triangle](https://github.com/emberware/emberware/tree/main/examples/triangle)** - Your first 3D shape
3. **[textured-quad](https://github.com/emberware/emberware/tree/main/examples/textured-quad)** - Loading and applying textures
4. **[cube](https://github.com/emberware/emberware/tree/main/examples/cube)** - Transforms and rotation
5. **[paddle](https://github.com/emberware/emberware/tree/main/examples/paddle)** - Complete game with the [tutorial](../tutorials/paddle/index.md)
6. **[platformer](https://github.com/emberware/emberware/tree/main/examples/platformer)** - Advanced example with physics, billboards, UI

## By Category

### Getting Started

| Example | Description |
|---------|-------------|
| **hello-world** | Basic 2D drawing, text, input handling |
| **triangle** | Minimal 3D rendering |
| **textured-quad** | Texture loading and binding |
| **cube** | Rotating textured cube with transforms |

### Graphics & Rendering

| Example | Description |
|---------|-------------|
| **lighting** | PBR rendering with 4 dynamic lights |
| **blinn-phong** | Classic specular and rim lighting |
| **billboard** | GPU-instanced billboards |
| **procedural-shapes** | Built-in mesh generators |
| **textured-procedural** | Textured procedural meshes |
| **dither-demo** | PS1-style dithering effects |
| **material-override** | Per-draw material properties |

### Render Mode Inspectors

| Example | Description |
|---------|-------------|
| **mode0-inspector** | Interactive Mode 0 (Unlit) explorer |
| **mode1-inspector** | Interactive Mode 1 (Matcap) explorer |
| **mode2-inspector** | Interactive Mode 2 (PBR) explorer |
| **mode3-inspector** | Interactive Mode 3 (Hybrid) explorer |

### Animation & Skinning

| Example | Description |
|---------|-------------|
| **skinned-mesh** | GPU skeletal animation basics |
| **animation-demo** | Keyframe playback from ROM |
| **ik-demo** | Inverse kinematics |
| **multi-skinned-procedural** | Multiple animated characters |
| **multi-skinned-rom** | ROM-based animation data |
| **skeleton-stress-test** | Performance testing |

### Complete Games

| Example | Description |
|---------|-------------|
| **paddle** | Classic 2-player game with AI |
| **platformer** | Full mini-game with physics, UI, multiplayer |

### Audio

| Example | Description |
|---------|-------------|
| **audio-demo** | Sound effects, panning, channels |

### Asset Loading

| Example | Description |
|---------|-------------|
| **datapack-demo** | ROM asset workflow |
| **font-demo** | Custom font loading |
| **level-loader** | Level data from ROM |
| **asset-test** | Pre-converted asset testing |

### Development Tools

| Example | Description |
|---------|-------------|
| **debug-demo** | Debug inspection system |

### Shared Libraries

| Example | Description |
|---------|-------------|
| **examples-common** | Reusable utilities (DebugCamera, math helpers) |

## Building Examples

Each example is a standalone Cargo project:

```bash
cd examples/paddle
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

Or build all examples:

```bash
cargo xtask build-examples
```

## Example Structure

All examples follow this pattern:

```
example-name/
├── Cargo.toml      # Project config
├── ember.toml      # Game manifest (optional)
├── src/
│   └── lib.rs      # Game code
└── assets/         # Assets (if needed)
```

## Learning by Reading Code

Each example includes comments explaining key concepts:

```rust
//! Example Name
//!
//! Description of what this example demonstrates.
//!
//! Controls:
//! - ...
//!
//! Note: Rollback state is automatic.
```

Browse the source on GitHub or read locally in your clone.
