# Nethercore Examples

**40 working examples** organized into 8 categories to help you learn game development with Nethercore.

## 📁 Organization

```
examples/
├── 1-getting-started/   →  4 examples   (FFI basics, languages)
├── 2-graphics/          →  6 examples   (Rendering, meshes, materials)
├── 3-inspectors/        → 13 examples   (Debug inspector, render modes, environments)
├── 4-animation/         →  6 examples   (Skeletal animation, IK)
├── 5-audio/             →  2 examples   (Sound effects, music)
├── 6-assets/            →  4 examples   (ROM loading, data packs)
├── 7-games/             →  2 examples   (Complete games)
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
# From repository root - builds all examples and installs to ~/.nethercore/games/
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

### 1. Getting Started (4 examples)

Learn the basics across multiple languages.

| Example | Description | Difficulty | Language |
|---------|-------------|------------|----------|
| **hello-world** | 2D drawing, text, rectangles, basic input | 🟢 Beginner | Rust |
| **hello-world-c** | Identical to hello-world, demonstrates C FFI | 🟢 Beginner | C |
| **hello-world-zig** | Identical to hello-world, demonstrates Zig FFI | 🟢 Beginner | Zig |
| **triangle** | Minimal 3D rendering with a single colored triangle | 🟢 Beginner | Rust |

---

### 2. Graphics & Rendering (6 examples)

Core rendering techniques and procedural meshes.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **textured-quad** | Texture loading and sprite rendering | 🟢 Beginner | `load_texture()`, `texture_bind()` |
| **procedural-shapes** | 7 built-in mesh generators with texture toggle | 🟡 Intermediate | B button toggles textured/plain modes |
| **lighting** | Full PBR lighting with 4 dynamic lights | 🟡 Intermediate | Mode 2 PBR, sky system, metallic/roughness |
| **billboard** | GPU-instanced billboards, camera-facing sprites | 🟡 Intermediate | Instancing, orientation |
| **dither-demo** | PS1-style ordered dithering effects | 🟡 Intermediate | Retro aesthetic |
| **material-override** | Per-draw material property overrides | 🟡 Intermediate | Dynamic materials |

---

### 3. Inspectors (13 examples)

Interactive debuggers for render modes and environment effects.

#### Render Mode Inspectors (4)

| Example | Mode | Description |
|---------|------|-------------|
| **mode0-inspector** | Mode 0 | Lambert rendering with interactive controls |
| **mode1-inspector** | Mode 1 | Matcap/image-based lighting explorer |
| **mode2-inspector** | Mode 2 | Metallic-Roughness PBR (Mode 2) explorer |
| **mode3-inspector** | Mode 3 | Specular-Shininess Blinn-Phong explorer |

#### Environment Inspectors + Debug (9)

| Example | Effect Type | Description |
|---------|-------------|-------------|
| **debug-demo** | Debug inspection system (F4 panel) | Frame controls, inspection |
| **env-gradient-inspector** | Gradient | 4-point gradient with 4 presets, debug controls |
| **env-curtains-inspector** | Curtains | Curtain-style environment effect |
| **env-lines-inspector** | Lines | Line-based procedural environment |
| **env-rectangles-inspector** | Rectangles | Rectangle pattern environment |
| **env-rings-inspector** | Rings | Concentric ring environment |
| **env-room-inspector** | Room | Room-style enclosed environment |
| **env-scatter-inspector** | Scatter | Particle scatter environment |
| **env-silhouette-inspector** | Silhouette | Silhouette-based environment |

**Difficulty:** 🟡 Intermediate | **Key Features:** F4 debug panels, real-time parameter tweaking

---

### 4. Animation & Skinning (6 examples)

GPU skeletal animation and inverse kinematics.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **skinned-mesh** | GPU skeletal animation basics | 🟡 Intermediate | `set_bones()`, basic transforms |
| **animation-demo** | Keyframe animation playback from ROM | 🟡 Intermediate | ROM-based anim data |
| **ik-demo** | Inverse kinematics (procedural animation) | 🔴 Advanced | Runtime IK solving |
| **multi-skinned-procedural** | Multiple animated characters (procedural bones) | 🟡 Intermediate | Procedural skeleton |
| **multi-skinned-rom** | Multiple animated characters (ROM data) | 🟡 Intermediate | ROM skeleton + anim |
| **skeleton-stress-test** | Performance test with many skeletons | 🔴 Advanced | Stress testing |

---

### 5. Audio (2 examples)

Sound effects and music playback.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **audio-demo** | Sound effects, panning, channels, looping | 🟢 Beginner | `play_sound()`, channels, panning |
| **tracker-demo** | XM tracker music playback with controls | 🟡 Intermediate | Tracker music, interactive controls |

---

### 6. Asset Loading (4 examples)

ROM-based asset workflows and data packs.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **datapack-demo** | Full ROM workflow: textures, meshes, sounds | 🟡 Intermediate | `rom_texture()`, `rom_mesh()`, `rom_sound()` |
| **font-demo** | Custom font loading with `rom_font()` | 🟢 Beginner | Bitmap fonts, text rendering |
| **level-loader** | Level data loading with `rom_data()` | 🟡 Intermediate | Binary data, custom formats |
| **asset-test** | Pre-converted asset testing (.nczxmesh, .nczxtex) | 🟡 Intermediate | Asset pipeline validation |

---

### 7. Complete Games (2 examples)

Fully playable games demonstrating complete game loops.

| Example | Description | Difficulty | Key Features |
|---------|-------------|------------|--------------|
| **paddle** | Classic 2-player paddle game | 🟢 Beginner | AI, rollback netcode, sound, [tutorial](../docs/book/src/tutorials/paddle/index.md) |
| **platformer** | 2D platformer mini-game | 🟡 Intermediate | Physics, collision, billboards, UI |

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

- **Total Examples:** 41 working examples
- **Beginner-Friendly:** 10 examples (🟢)
- **Intermediate:** 23 examples (🟡)
- **Advanced:** 12 examples (🔴)
- **Languages:** Rust, C, Zig
- **Complete Games:** 2

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
nether run target/wasm32-unknown-unknown/release/lighting.wasm
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
