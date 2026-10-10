# Example Games

The repository contains **53 source example projects** in 8 categories (Rust, C and Zig). The [development ledger](../../../plans/zx-examples-development.md) distinguishes source presence from runtime qualification and the planned 90-project curriculum.

## Location

Examples are organized by category:

```
examples/
├── 1-getting-started/   (8 examples)
├── 2-graphics/          (10 examples)
├── 3-inspectors/        (7 examples)
├── 4-animation/         (6 examples)
├── 5-audio/             (5 examples)
├── 6-assets/            (8 examples)
├── 7-games/             (3 examples)
├── 8-advanced/          (6 examples)
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
| **hello-world** | Basic 2D drawing, text, input handling; [source and replay checks](https://github.com/nethercore-systems/nethercore/tree/main/examples/1-getting-started/hello-world) |
| **hello-world-c** | Same example in C (demonstrates C FFI) |
| **hello-world-zig** | Same example in Zig (demonstrates Zig FFI) |
| **save-slots** | Persistent save slots |
| **triangle** | Winding, camera side, culling and model rotation with one triangle |
| **input-lab** | Four controller slots, button edges, radial deadzone, sticks and triggers |
| **fixed-clock-lab** | Simulation ticks versus zero/repeated draws; native/headless endpoint checks |
| **seeded-random-lab** | Bounded route generation, repeated/different seeds and runner reset |

The four foundation lessons include controls, build/run instructions and replay
witnesses. `examples/1-getting-started/check_foundations.py --help` describes the
combined build, relocation, negative-assertion, native and forced-rollback check.
Native checks require a display and Pillow; neither headless success nor captured
images establish human acceptance.

### 2. Graphics & Rendering

| Example | Description |
|---------|-------------|
| **textured-quad** | Numbered atlas: UV corners/regions, nearest versus linear, half-texel inset and reset |
| **atlas-flipbook** | Tick-owned loop/once/ping-pong sprite playback; pause, single tick, rewind and native boundary checks |
| **sprite-pivot-lab** | Marked center/edge pivots, rotated atlas regions and explicit parent/child coordinates |
| **transform-composition-lab** | Labeled T*R/R*T, API/matrix equivalence and an explicit two-joint crane |
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
| **epu-showcase** | Optional editable compositions + seven capability benchmarks; all 24 active EPU opcodes and guest-owned phase |
| **epu-inspector** | Packed eight-layer instruction explorer; use the native F4 EPU editor for semantic controls and LOCK/export |

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
| **font-demo** | ROM texture + fixed/variable font metrics, measured alignment, guest wrap/fallback/focus |
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
| **stencil-demo** | Four stencil introductions: pass order, mask ink, portal depth-clear counterexample and reset |
| **viewport-test** | 2P/4P cameras, local HUD coordinates, scissored masks/ink and independent input |
| **rear-mirror** | Rear-view mirror using viewport |
| **stencil-boolean-masks** | Union/intersection/subtraction/parity with nested holes and swapped operands |
| **texture-shaped-stencil** | Transparent sprite holes and Bayer-dithered edges define the stencil silhouette |
| **blob-shadows** | Radial texture coverage, floor clipping, foreground occlusion and depth-write diagnostics |

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

This builds Rust, C and Zig examples through their project manifests and installs them into your Nethercore data directory under `games/` (the command prints the exact path). All three language toolchains are required; a missing toolchain is a failure, not a skipped success.

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
