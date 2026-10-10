# Nethercore Examples

**50 source example projects** organized into 8 categories. Source presence is not runtime qualification; the [90-project development ledger](../docs/plans/zx-examples-development.md) records verified coverage and work still pending.

## 📁 Organization

```
examples/
├── 1-getting-started/   →  8 examples   (FFI basics, languages, input/time/RNG)
├── 2-graphics/          → 10 examples   (Rendering, atlases, pivots, transforms, meshes)
├── 3-inspectors/        →  7 examples   (Debug inspector, render modes, environments)
├── 4-animation/         →  6 examples   (Skeletal animation)
├── 5-audio/             →  5 examples   (Sound effects, tracker music)
├── 6-assets/            →  8 examples   (ROM loading, data packs, GLTF/GLB pipeline)
├── 7-games/             →  3 examples   (Complete games)
├── 8-advanced/          →  3 examples   (Stencils, viewports, mirrors)
└── examples-common/     →  Support library
```

## 🚀 Quick Start

### For Game Developers

Build individual examples using the `nether` CLI:

```bash
# Navigate to an example
cd examples/7-games/paddle

# Build and run
nether run

# Or just build the ROM
nether build
```

### For Nethercore Contributors

Build all examples at once:

```bash
# From repository root - builds Rust, C and Zig examples and installs them to your Nethercore data directory under `games/` (prints the exact path)
cargo xtask build-examples

# Run any example from the library
cargo run -- <example-name>

# Example: Run the paddle game
cargo run -- paddle
```

## 📚 Learning Path

**New to Nethercore?** Follow this progression:

| # | Example | Category | What You'll Learn |
|---|---------|----------|-------------------|
| 1 | **hello-world** | 1-getting-started | 2D drawing, text, rectangles, input handling |
| 2 | **triangle** | 1-getting-started | Your first 3D shape, minimal rendering |
| 3 | **textured-quad** | 2-graphics | Loading textures, sprite rendering |
| 4 | **procedural-shapes** | 2-graphics | 7 built-in mesh generators, texture toggle |
| 5 | **lighting** | 2-graphics | PBR materials, dynamic lights, sky system |
| 6 | **paddle** | 7-games | Complete game with [tutorial](../docs/book/src/tutorials/paddle/index.md) |

---

## 📂 All Examples by Category

### 1. Getting Started (8 examples)

Learn the basics across multiple languages.

| Example | Description | Difficulty | Language |
|---------|-------------|------------|----------|
| **[hello-world](1-getting-started/hello-world/README.md)** | 2D drawing, text, rectangles, basic input | 🟢 Beginner | Rust |
| **save-slots** | Persistent save slots (save/load/delete) | 🟢 Beginner | Rust |
| **[hello-world-c](1-getting-started/hello-world-c/README.md)** | Identical to hello-world, demonstrates C FFI | 🟢 Beginner | C |
| **[hello-world-zig](1-getting-started/hello-world-zig/README.md)** | Identical to hello-world, demonstrates Zig FFI | 🟢 Beginner | Zig |
| **[triangle](1-getting-started/triangle/README.md)** | One triangle: winding, camera, culling and model rotation | 🟢 Beginner | Rust |
| **[input-lab](1-getting-started/input-lab/README.md)** | Four input slots, button edges, sticks, radial deadzone and triggers | 🟢 Beginner | Rust |
| **[fixed-clock-lab](1-getting-started/fixed-clock-lab/README.md)** | Simulation ticks versus zero/repeated draws | 🟢 Beginner | Rust |
| **[seeded-random-lab](1-getting-started/seeded-random-lab/README.md)** | Repeatable bounded routes from recorded RNG seeds | 🟢 Beginner | Rust |

#### Foundation checks

With Python 3.11+, Pillow, Rust's WASM target and matching native tools, from the
repository root:

```sh
python examples/1-getting-started/check_foundations.py --nether /absolute/path/to/nether --player /absolute/path/to/nethercore-zx --output /absolute/path/outside/checkout/foundation-evidence --render
```

Use real executable paths (including `.exe` on Windows) and an empty output
directory. This builds and runs all four lessons after relocation, rejects a
wrong assertion in each, checks native pixels/state and runs the same controls
through 120 input frames of forced rollback. Reports bind source, ROM, tools,
logs and captured pixels. Without `--render`, only headless checks run; that
subset does not certify presentation or rollback. Human review and remote/CI
delivery remain separate. The shared allocation-free label formatter is directly
testable without changing Cargo workspace membership:

```sh
rustc --edition 2021 --test examples/examples-common/src/hud.rs -o /absolute/path/outside/checkout/hud-test
/absolute/path/outside/checkout/hud-test
```

---

### 2. Graphics & Rendering (7 examples)

Core rendering techniques and procedural meshes.

`epu-multi-reflections` now demonstrates immediate-mode `epu_set()` switching per draw instead of public environment IDs.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **textured-quad** | Numbered atlas, UV regions and per-draw sampling | 🟢 Beginner | Nearest/linear comparison, half-texel inset, reset and native witness |
| **atlas-flipbook** | Guest-owned sprite animation timing | 🟢 Beginner | Loop/once/ping-pong, unequal frame durations, pause/tick/rewind and native witness |
| **sprite-pivot-lab** | Center/edge origins and explicit two-link composition | 🟢 Beginner | Eight rotation stops, UV regions, marked anchors and independent native probes |
| **transform-composition-lab** | T*R versus R*T and a two-joint crane | 🟢 Beginner | Eight angles, API/matrix parity, scalar projected-bound checks and explicit hierarchy |
| **procedural-shapes** | 7 built-in mesh generators with texture toggle | 🟡 Intermediate | B button toggles textured/plain modes |
| **lighting** | Full PBR lighting with 4 dynamic lights | 🟡 Intermediate | Mode 2 PBR, sky system, metallic/roughness |
| **epu-multi-reflections** | Two shiny spheres switching immediate-mode EPU sources | 🟡 Intermediate | `epu_set()`, `epu_textures()`, `draw_epu()` |
| **billboard** | GPU-instanced billboards, camera-facing sprites | 🟡 Intermediate | Instancing, orientation |
| **dither-demo** | PS1-style ordered dithering effects | 🟡 Intermediate | Retro aesthetic |
| **material-override** | Per-draw material property overrides | 🟡 Intermediate | Dynamic materials |

---

### 3. Inspectors (7 examples)

Interactive debuggers for render modes and environment effects.

#### Render Mode Inspectors (4)

| Example | Mode | Description |
|---------|------|-------------|
| **mode0-inspector** | Mode 0 | Lambert rendering with interactive controls |
| **mode1-inspector** | Mode 1 | Matcap/image-based lighting explorer |
| **mode2-inspector** | Mode 2 | Metallic-Roughness PBR (Mode 2) explorer |
| **mode3-inspector** | Mode 3 | Specular-Shininess Blinn-Phong explorer |

#### Debug + EPU (3)

| Example | Description |
|---------|-------------|
| **debug-demo** | Debug inspection system (F4 panel) |
| **epu-showcase** | EPU environment presets + inspector |
| **epu-inspector** | Packed eight-layer EPU instruction explorer |

---

### 4. Animation & Skinning (6 examples)

GPU skeletal animation and keyframe playback.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **skinned-mesh** | GPU skeletal animation basics | 🟡 Intermediate | `set_bones()`, basic transforms |
| **animation-demo** | Keyframe animation playback from ROM | 🟡 Intermediate | ROM-based anim data |
| **multi-skinned-rom** | Multiple animated characters (ROM data) | 🟡 Intermediate | ROM skeleton + anim |
| **ik-demo** | Inverse kinematics | 🟡 Intermediate | Guest-side joint solving |
| **multi-skinned-procedural** | Multiple procedural characters | 🟡 Intermediate | Per-instance bones |
| **skeleton-stress-test** | Skeleton budget inspection | 🔴 Advanced | Instancing and bone budgets |

---

### 5. Audio (5 examples)

Sound effects and music playback.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **audio-demo** | Sound effects, panning, channels, looping | 🟢 Beginner | `play_sound()`, channels, panning |
| **tracker-demo-xm** | XM tracker music playback (three songs) | 🟡 Intermediate | XM modules, embedded sample extraction |
| **tracker-demo-xm-split** | XM tracker music with split sample workflow | 🟡 Intermediate | Split samples (WAV), explicit `[[assets.sounds]]` |
| **tracker-demo-it** | IT tracker music playback (three songs) | 🟡 Intermediate | IT modules, embedded sample extraction |
| **tracker-demo-it-split** | IT tracker demo with separate sample assets | 🟡 Intermediate | Split assets, explicit `[[assets.sounds]]` |

---

### 6. Asset Loading (8 examples)

ROM-based asset workflows and data packs.

`epu-textures-demo` is the face-texture validation example for `rom_texture()` + `epu_textures()`.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **datapack-demo** | Full ROM workflow: textures, meshes, sounds | 🟡 Intermediate | `rom_texture()`, `rom_mesh()`, `rom_sound()` |
| **font-demo** | Texture-atlas text demo (not a `rom_font()` loader demo) | 🟢 Beginner | `rom_texture()`, atlas regions |
| **level-loader** | Level data loading with `rom_data()` | 🟡 Intermediate | Binary data, custom formats |
| **asset-test** | Pre-converted asset testing (.nczxmesh, .nczxtex) | 🟡 Intermediate | Asset pipeline validation |
| **gltf-test** | Tests GLTF import (mesh, skeleton, animation) | 🟡 Intermediate | GLTF pipeline, conversion validation |
| **glb-inline** | Raw GLB references with multiple animations | 🟡 Intermediate | Direct `.glb` in `nether.toml`, animation selectors |
| **glb-rigid** | Rigid transform animation imported from GLB | 🟡 Intermediate | `keyframe_read()`, multi-mesh transforms |
| **epu-textures-demo** | Six-face environment textures | 🟡 Intermediate | `rom_texture()`, `epu_textures()` |

---

### 7. Complete Games (3 examples)

Fully playable games demonstrating complete game loops.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **paddle** | Classic 2-player paddle game | 🟢 Beginner | AI, rollback netcode, sound, [tutorial](../docs/book/src/tutorials/paddle/index.md) |
| **platformer** | 2D platformer mini-game | 🟡 Intermediate | Physics, collision, billboards, UI |
| **netplay-demo** | Multiplayer arena and session feedback | 🟡 Intermediate | Player/session input |

---

### 8. Advanced Rendering (3 examples)

Stencil buffers, viewports, and advanced techniques.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **stencil-demo** | All 4 stencil masking modes | 🔴 Advanced | Circle, inverted, diagonal, multiple masks |
| **viewport-test** | Split-screen rendering (2P, 4P) | 🟡 Intermediate | Multiple viewports |
| **rear-mirror** | Rear-view mirror for racing | 🔴 Advanced | Secondary viewport |

---

## 🛠️ Support Library

| Library | Description | Used By |
|---------|-------------|---------|
| **examples-common** | Reusable utilities (DebugCamera, StickControl, math helpers) | Multiple inspectors |
| **assets/** | Shared assets used by multiple examples | Various |

---

## 📊 Statistics

- **Source Projects:** 44
- **Languages:** Rust, C, Zig
- **Game Projects:** 3

---

## 🎮 Running Examples

### Option 1: From the Library (Recommended)

```bash
# Build all examples once
cargo xtask build-examples

# Launch the library browser
cargo run

# Select any game from the grid
```

### Option 2: Direct Launch

```bash
# Run by name (no path needed)
cargo run -- paddle
cargo run -- platformer
cargo run -- triangle
```

### Option 3: Manual Build

```bash
# Navigate to example
cd 2-graphics/lighting

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Run with nether CLI
nethercore-zx target/wasm32-unknown-unknown/release/lighting.wasm
```

---

## 📖 Documentation

- **API Reference:** See [`../docs/book/`](../docs/book/src/SUMMARY.md)
- **Tutorial:** [Build Paddle from Scratch](../docs/book/src/tutorials/paddle/index.md)
- **FFI Cheat Sheet:** [`../docs/book/src/cheat-sheet.md`](../docs/book/src/cheat-sheet.md)
- **Example Reference:** [`../docs/book/src/reference/examples.md`](../docs/book/src/reference/examples.md)

---

## 🤝 Contributing

Want to add an example? Follow the structure:

```
<category>/<your-example>/
├── Cargo.toml       # Standard WASM project
├── nether.toml      # Game manifest (title, description, assets)
├── src/
│   └── lib.rs       # Game code with FFI exports
└── assets/          # Optional: textures, sounds, etc.
```

See [`../docs/contributing/`](../docs/contributing/) for guidelines.

---

**Questions?** Check the [book](../docs/book/) or open an issue on GitHub!

## Isolated build verification

`cargo xtask build-examples` discovers Cargo projects and manifest-declared C/Zig
builds. All three language toolchains are required; failures are not skipped.
For self-contained test fixtures, keep installation and stale-example cleanup
away from your normal library:

```bash
cargo xtask build-examples --examples-dir <fixture-directory> --games-dir <disposable-install-directory>
```

The custom source directory skips workspace asset generators; prepare its assets
first. Any failed example makes the command exit nonzero. A successful build is
not a runtime, graphics, audio, or controller acceptance result.
