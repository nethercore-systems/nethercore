# Example Games

The Nethercore repository includes **46 working examples** organized into 8 categories. Each example is a complete, buildable project (Rust, C, or Zig).

## Location

Examples are organized by category:

```
examples/
├── 1-getting-started/   (4 examples)
├── 2-graphics/          (6 examples)
├── 3-inspectors/        (13 examples)
├── 4-animation/         (6 examples)
├── 5-audio/             (5 examples)
├── 6-assets/            (7 examples)
├── 7-games/             (2 examples)
├── 8-advanced/          (3 examples)
└── examples-common/     (support library)
```

## Learning Path

New to Nethercore? Follow this progression:

1. **[hello-world](https://github.com/nethercore-systems/nethercore/tree/main/examples/1-getting-started/hello-world)** - 2D text and rectangles, basic input
2. **[triangle](https://github.com/nethercore-systems/nethercore/tree/main/examples/1-getting-started/triangle)** - Your first 3D shape
3. **[textured-quad](https://github.com/nethercore-systems/nethercore/tree/main/examples/2-graphics/textured-quad)** - Loading and applying textures
4. **[procedural-shapes](https://github.com/nethercore-systems/nethercore/tree/main/examples/2-graphics/procedural-shapes)** - Procedural meshes with texture toggle
5. **[paddle](https://github.com/nethercore-systems/nethercore/tree/main/examples/7-games/paddle)** - Complete game with the [tutorial](../tutorials/paddle/index.md)
6. **[platformer](https://github.com/nethercore-systems/nethercore/tree/main/examples/7-games/platformer)** - Advanced example with physics, billboards, UI

## By Category

### 1. Getting Started

| Example | Description |
|---------|-------------|
| **hello-world** | Basic 2D drawing, text, input handling |
| **hello-world-c** | Same example in C (demonstrates C FFI) |
| **hello-world-zig** | Same example in Zig (demonstrates Zig FFI) |
| **triangle** | Minimal 3D rendering |

### 2. Graphics & Rendering

| Example | Description |
|---------|-------------|
| **textured-quad** | Texture loading and sprite rendering |
| **procedural-shapes** | Built-in mesh generators with texture toggle (B button) |
| **lighting** | PBR rendering with 4 dynamic lights |
| **billboard** | GPU-instanced billboards |
| **dither-demo** | PS1-style dithering effects |
| **material-override** | Per-draw material properties |

### 3. Inspectors (Mode & Environment)

| Example | Description |
|---------|-------------|
| **debug-demo** | Debug inspection system (F4 panel) |
| **mode0-inspector** | Interactive Mode 0 (Lambert) explorer |
| **mode1-inspector** | Interactive Mode 1 (Matcap) explorer |
| **mode2-inspector** | Interactive Mode 2 (PBR) explorer |
| **mode3-inspector** | Interactive Mode 3 (Blinn-Phong) explorer |
| **env-gradient-inspector** | Gradient environment with presets |
| **env-veil-inspector** | Veil environment effect |
| **env-lines-inspector** | Line-based environment effect |
| **env-nebula-inspector** | Nebula (fog/clouds/aurora) environment |
| **env-rings-inspector** | Ring-based environment effect |
| **env-room-inspector** | Room-style environment |
| **env-cells-inspector** | Cells (particles/tiles) environment |
| **env-silhouette-inspector** | Silhouette-based environment |

### 4. Animation & Skinning

| Example | Description |
|---------|-------------|
| **skinned-mesh** | GPU skeletal animation basics |
| **animation-demo** | Keyframe playback from ROM |
| **ik-demo** | Inverse kinematics |
| **multi-skinned-procedural** | Multiple animated characters (procedural) |
| **multi-skinned-rom** | Multiple animated characters (ROM data) |
| **skeleton-stress-test** | Performance testing with many skeletons |

### 5. Audio

| Example | Description |
|---------|-------------|
| **audio-demo** | Sound effects, panning, channels, looping |
| **tracker-demo-xm** | XM tracker music playback |
| **tracker-demo-xm-split** | XM tracker music with split sample workflow |
| **tracker-demo-it** | IT tracker music playback |
| **tracker-demo-it-split** | IT tracker demo with separate sample assets |

### 6. Asset Loading

| Example | Description |
|---------|-------------|
| **datapack-demo** | Full ROM asset workflow (textures, meshes, sounds) |
| **font-demo** | Custom font loading with rom_font |
| **level-loader** | Level data loading with rom_data |
| **asset-test** | Pre-converted asset testing (.nczxmesh, .nczxtex) |
| **gltf-test** | GLTF import pipeline validation (mesh, skeleton, animation) |
| **glb-inline** | Direct `.glb` references with multiple animations |
| **glb-rigid** | Rigid transform animation imported from GLB |

### 7. Complete Games

| Example | Description |
|---------|-------------|
| **paddle** | Classic 2-player paddle game with AI and rollback netcode |
| **platformer** | Full mini-game with 2D gameplay, physics, collision, UI |

### 8. Advanced Rendering

| Example | Description |
|---------|-------------|
| **stencil-demo** | All 4 stencil masking modes |
| **viewport-test** | Split-screen rendering (2P, 4P) |
| **rear-mirror** | Rear-view mirror using viewport |

### Support Library

| Library | Description |
|---------|-------------|
| **examples-common** | Reusable utilities (DebugCamera, StickControl, math helpers) |

## Building Examples

### Using nether CLI (Recommended for Game Developers)

Many examples include a `nether.toml` manifest:

```bash
cd examples/7-games/paddle
nether build   # Build WASM and create .nczx ROM
nether run     # Build and launch in emulator
```

### Building All Examples (For Nethercore Contributors)

To build and install all examples at once:

```bash
# From nethercore repository root
cargo xtask build-examples
```

This builds the Cargo-based examples and installs them into your Nethercore data directory under `games/` (the command prints the exact path).

## Example Structure

Most Rust examples follow this pattern:

```
category/example-name/
├── Cargo.toml       # Project config
├── nether.toml      # Game manifest (optional)
├── src/
│   └── lib.rs       # Game code
└── assets/          # Assets (if needed)
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

Browse the source on GitHub or navigate to `examples/` in your local clone.
