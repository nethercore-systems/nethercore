# Example Games

The Nethercore repository includes 46 working examples organized into 9 categories. Each example is a complete, buildable project.

## Location

Examples are organized by category:

```
nethercore/examples/
├── 1-getting-started/   (4 examples)
├── 2-graphics/          (6 examples)
├── 3-inspectors/        (12 examples)
├── 4-animation/         (6 examples)
├── 5-audio/             (2 examples)
├── 6-assets/            (4 examples)
├── 7-games/             (5 examples)
├── 8-advanced/          (5 examples)
├── 9-debug/             (5 examples)
└── _lib/                (support libraries)
```

## Learning Path

New to Nethercore? Follow this progression:

1. **[hello-world](https://github.com/nethercore/nethercore/tree/main/examples/1-getting-started/hello-world)** - 2D text and rectangles, basic input
2. **[triangle](https://github.com/nethercore/nethercore/tree/main/examples/1-getting-started/triangle)** - Your first 3D shape
3. **[textured-quad](https://github.com/nethercore/nethercore/tree/main/examples/2-graphics/textured-quad)** - Loading and applying textures
4. **[procedural-shapes](https://github.com/nethercore/nethercore/tree/main/examples/2-graphics/procedural-shapes)** - Procedural meshes with texture toggle
5. **[paddle](https://github.com/nethercore/nethercore/tree/main/examples/7-games/paddle)** - Complete game with the [tutorial](../tutorials/paddle/index.md)
6. **[platformer](https://github.com/nethercore/nethercore/tree/main/examples/7-games/platformer)** - Advanced example with physics, billboards, UI

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
| **mode0-inspector** | Interactive Mode 0 (Lambert) explorer |
| **mode1-inspector** | Interactive Mode 1 (Matcap) explorer |
| **mode2-inspector** | Interactive Mode 2 (PBR) explorer |
| **mode3-inspector** | Interactive Mode 3 (Blinn-Phong) explorer |
| **env-gradient-inspector** | Gradient environment with presets |
| **env-curtains-inspector** | Curtain-style environment effect |
| **env-lines-inspector** | Line-based environment effect |
| **env-rectangles-inspector** | Rectangle-based environment |
| **env-rings-inspector** | Ring-based environment effect |
| **env-room-inspector** | Room-style environment |
| **env-scatter-inspector** | Scatter-based environment |
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
| **tracker-demo** | XM tracker music playback |

### 6. Asset Loading

| Example | Description |
|---------|-------------|
| **datapack-demo** | Full ROM asset workflow (textures, meshes, sounds) |
| **font-demo** | Custom font loading with rom_font |
| **level-loader** | Level data loading with rom_data |
| **asset-test** | Pre-converted asset testing (.nczxmesh, .nczxtex) |

### 7. Complete Games

| Example | Description |
|---------|-------------|
| **paddle** | Classic 2-player paddle game with AI and rollback netcode |
| **platformer** | Full mini-game with 2D gameplay, physics, collision, UI |
| **prism-survivors** | Top-down shooter template (stub) |
| **lumina-depths** | Underwater exploration template (stub) |
| **neon-drift** | Arcade racer template (stub) |

### 8. Advanced Rendering

| Example | Description |
|---------|-------------|
| **stencil-demo** | All 4 stencil masking modes |
| **portal-demo** | Portal rendering using stencil masking |
| **viewport-test** | Split-screen rendering (2P, 4P) |
| **rear-mirror** | Rear-view mirror using viewport |
| **scope-shooter** | Sniper scope mechanic with stencil |

### 9. Debug & Development Tools

| Example | Description |
|---------|-------------|
| **debug-demo** | Debug inspection system (F3 panel) |
| **proc-gen-viewer** | Interactive procedural mesh viewer |
| **proc-gen-mode2** | Mode 2 asset preview (Neon Drift) |
| **proc-gen-mode3** | Mode 3 asset preview (Lumina Depths) |
| **proc-sounds-viewer** | Procedural sound effect viewer |

### Support Libraries

| Library | Description |
|---------|-------------|
| **examples-common** | Reusable utilities (DebugCamera, StickControl, math helpers) |
| **proc-gen-showcase-defs** | Shared definitions for procedural showcases |

## Building Examples

Each example is a standalone Cargo project:

```bash
cd examples/7-games/paddle
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

Or build all examples:

```bash
cargo xtask build-examples
```

## Example Structure

All examples follow this pattern:

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
