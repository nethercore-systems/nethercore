# Nethercore - Claude Code Instructions (Archived Snapshot)

> Status: Archived snapshot
> Last reviewed: 2026-01-06

## Project Overview

Nethercore is a fantasy console platform with built-in rollback netcode, designed to support multiple fantasy consoles (Nethercore ZX, Chroma, etc.) with a shared framework.

**Console Status:**
- **Nethercore ZX** — Fully implemented (5th generation aesthetic)
- **Nethercore Chroma** — Coming Soon (documented but not yet implemented)

**Repository Structure:**
- `/core` — Console trait, WASM runtime, GGRS rollback, ConsoleRunner, debug inspection
- `/library` — Main binary with library UI, console registry, game launcher
- `/nethercore-zx` — 5th generation aesthetic console implementation (library, no binary)
- `/z-common` — Z-specific formats, ROM loader
- `/shared` — API types for platform backend, cart/ROM formats
- `/tools` — Developer tools (nether-cli, nether-export)
- `/docs` — Game developer documentation
- `/examples` — Example games

**Key Documentation:**
- [TASKS.md](./TASKS.md) — Development status and implementation plan
- [docs/architecture/ffi.md](./docs/architecture/ffi.md) — Shared FFI API reference
- [../nethercore-design/consoles/zx-spec.md](../nethercore-design/consoles/zx-spec.md) — ZX console specification (source of truth)
- [docs/architecture/zx/rendering.md](./docs/architecture/zx/rendering.md) — ZX graphics deep dive
- [docs/architecture/rom-format.md](./docs/architecture/rom-format.md) — ROM/cart format specification
- [docs/architecture/multiplayer-testing.md](./docs/architecture/multiplayer-testing.md) — Multiplayer and GGRS testing guide

## Game Developer Documentation (Book)

The [docs/book/](./docs/book/) directory contains comprehensive game developer documentation in mdBook format:

**Quick Reference:**
- [Cheat Sheet](./docs/book/src/cheat-sheet.md) — Quick FFI function lookup

**Getting Started:**
- [Prerequisites](./docs/book/src/getting-started/prerequisites.md)
- [Your First Game](./docs/book/src/getting-started/first-game.md)
- [Understanding the Game Loop](./docs/book/src/getting-started/game-loop.md)

**Tutorials:**
- [Build Paddle](./docs/book/src/tutorials/paddle/index.md) — Complete 8-part tutorial building a paddle game

**API Reference** (FFI functions with examples):
- [System](./docs/book/src/api/system.md) — Time, logging, quit
- [Input](./docs/book/src/api/input.md) — Buttons, sticks, triggers
- [Graphics](./docs/book/src/api/graphics.md) — General graphics overview
- [Camera](./docs/book/src/api/camera.md) — Camera setup
- [Transforms](./docs/book/src/api/transforms.md) — Matrix stack operations
- [Textures](./docs/book/src/api/textures.md) — Texture loading and binding
- [Meshes](./docs/book/src/api/meshes.md) — Mesh loading and drawing
- [Materials](./docs/book/src/api/materials.md) — PBR material properties
- [Lighting](./docs/book/src/api/lighting.md) — Directional and point lights
- [Skinning](./docs/book/src/api/skinning.md) — Skeletal animation
- [Animation](./docs/book/src/api/animation.md) — Keyframe animation
- [Procedural Meshes](./docs/book/src/api/procedural.md) — Shape generation
- [2D Drawing](./docs/book/src/api/drawing-2d.md) — Sprites, text, rectangles
- [Billboards](./docs/book/src/api/billboards.md) — Camera-facing quads
- [Sky](./docs/book/src/api/sky.md) — Sky gradients and sun
- [Audio](./docs/book/src/api/audio.md) — Sound and music playback
- [Save Data](./docs/book/src/api/save-data.md) — Persistent storage
- [ROM Loading](./docs/book/src/api/rom-loading.md) — Asset loading from ROM data pack
- [Debug](./docs/book/src/api/debug.md) — Debug inspection system

**Guides:**
- [Render Modes](./docs/book/src/guides/render-modes.md) — Choosing and configuring render modes
- [Rollback Safety](./docs/book/src/guides/rollback-safety.md) — Writing rollback-compatible code
- [Asset Pipeline](./docs/book/src/guides/asset-pipeline.md) — Converting and bundling assets
- [Publishing Your Game](./docs/book/src/guides/publishing.md) — Packaging and distribution

**Reference:**
- [Button Constants](./docs/book/src/reference/buttons.md) — Input button mappings
- [Dither Patterns](./docs/book/src/reference/dither-patterns.md) — Available dither patterns
- [Example Games](./docs/book/src/reference/examples.md) — Overview of all examples

**Contributing:**
- [Getting Started](./docs/contributing/getting-started.md) — Contributing to Nethercore
- [Distributing Games](./docs/contributing/distributing-games.md) — Publishing guidelines

## Canonical References

| Reference | File | Purpose |
|-----------|------|---------|
| FFI Source of Truth | [zx.rs](./include/zx.rs) | All ZX FFI function signatures |
| Shared FFI | [core/src/ffi.rs](./core/src/ffi.rs) | System, input, save, ROM functions |
| ZX FFI Implementation | [nethercore-zx/src/ffi/mod.rs](./nethercore-zx/src/ffi/mod.rs) | ZX-specific FFI registration |
| Console Trait | [core/src/console.rs](./core/src/console.rs) | Console abstraction |

## Key Source Files

### Core Crate
- [core/src/lib.rs](./core/src/lib.rs) — Public API exports
- [core/src/console.rs](./core/src/console.rs) — Console trait definition
- [core/src/wasm/mod.rs](./core/src/wasm/mod.rs) — WASM runtime
- [core/src/rollback/mod.rs](./core/src/rollback/mod.rs) — GGRS integration

### Nethercore ZX
- [nethercore-zx/src/lib.rs](./nethercore-zx/src/lib.rs) — ZX public API
- [nethercore-zx/src/console.rs](./nethercore-zx/src/console.rs) — Console impl
- [nethercore-zx/src/graphics/mod.rs](./nethercore-zx/src/graphics/mod.rs) — wgpu rendering

### Library
- [library/src/main.rs](./library/src/main.rs) — Entry point
- [library/src/registry.rs](./library/src/registry.rs) — Console registry

## Testing

```bash
# Run all tests
cargo test

# Build all examples (for Nethercore contributors)
cargo xtask build-examples

# Run specific example from library
cargo run -- platformer
cargo run -- hello-world

# Build a game using nether CLI (for game developers)
cd examples/platformer
nether build
nether run
```

## Example Games

Located in `examples/` — 28 examples organized by category:

| Category | Examples |
|----------|----------|
| Getting Started | `hello-world`, `triangle`, `textured-quad`, `cube` |
| Graphics | `lighting`, `blinn-phong`, `billboard`, `procedural-shapes` |
| Animation | `skinned-mesh`, `animation-demo`, `ik-demo` |
| Audio | `audio-demo` |
| Complete Game | `platformer` |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      library (binary)                       │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │ ConsoleRegistry│ │ ActiveGame   │  │ Library UI       │ │
│  │ (static dispatch│ │ enum dispatch│  │ (egui)           │ │
│  └───────┬───────┘  └──────┬───────┘  └──────────────────┘ │
│          │                 │                                │
├──────────┼─────────────────┼────────────────────────────────┤
│          │           nethercore-zx (lib)                     │
│  ┌───────▼───────┐  ┌─────────────┐  ┌─────────────────┐   │
│  │ NethercoreZ    │  │ ZXGraphics   │  │ Z-specific FFI  │   │
│  │ Console impl  │  │ (wgpu)      │  │ (draw_*, etc)   │   │
│  └───────┬───────┘  └─────────────┘  └─────────────────┘   │
│          │          ┌─────────────┐  ┌─────────────────┐   │
│          │          │ ZXAudio      │  │ ZRollbackState  │   │
│          │          │ (cpal)      │  │ (audio state)   │   │
│          │          └─────────────┘  └─────────────────┘   │
├──────────┼──────────────────────────────────────────────────┤
│          │              nethercore-core                      │
│  ┌───────▼───────┐  ┌─────────────┐  ┌─────────────────┐   │
│  │ Console trait │  │ConsoleRunner│  │ Common FFI      │   │
│  │ + RollbackState│ │ <C: Console>│  │ (input, save,   │   │
│  │               │  │ game loop   │  │  random, etc)   │   │
│  └───────────────┘  └─────────────┘  └─────────────────┘   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐     │
│  │ WasmEngine  │  │ Rollback    │  │ RomLoader trait │     │
│  │ (wasmtime)  │  │ state mgmt  │  │ (console-agnostic)│   │
│  └─────────────┘  └─────────────┘  └─────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Console Trait

Each fantasy console implements the `Console` trait:

```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;           // Console-specific input layout
    type State: Default + Send;          // Per-frame FFI state (not rolled back)
    type RollbackState: ConsoleRollbackState;  // Rolled back state (e.g., audio)
    type ResourceManager: ConsoleResourceManager;

    fn name(&self) -> &'static str;
    fn specs(&self) -> &ConsoleSpecs;
    fn register_ffi(&self, linker: &mut Linker<WasmGameContext<...>>) -> Result<()>;
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;
    fn create_audio(&self) -> Result<Self::Audio>;
    fn map_input(&self, raw: &RawInput) -> Self::Input;
    fn debug_stats(&self, state: &Self::State) -> Vec<DebugStat>;
}

// Must be POD for GGRS serialization
pub trait ConsoleInput: Clone + Copy + Default + bytemuck::Pod + bytemuck::Zeroable {}

// Console-specific rollback state (e.g., audio playhead positions)
pub trait ConsoleRollbackState: Pod + Zeroable + Default + Send + 'static {}
```

This allows:
- Shared WASM execution, rollback netcode, and game loop
- Console-specific rendering, audio, FFI functions, and input layouts
- Easy addition of future consoles (Nethercore Y, X, etc.)

### Input Abstraction

Each console defines its own input struct:

```rust
// Nethercore ZX (PS2/Xbox style)
#[repr(C)]
pub struct ZInput {
    pub buttons: u16,        // D-pad + face + shoulders + start/select
    pub left_stick_x: i8,    // -128..127
    pub left_stick_y: i8,
    pub right_stick_x: i8,
    pub right_stick_y: i8,
    pub left_trigger: u8,    // 0..255 analog
    pub right_trigger: u8,
}

// Nethercore Chroma (6-button retro)
#[repr(C)]
pub struct ChromaInput {
    pub buttons: u16,  // D-pad + A/B/C/X/Y/Z + L/R + start/select
}
```

The core handles GGRS serialization of whatever input type the console uses.

## Tech Stack

### Core
- wasmtime (WASM execution)
- GGRS (rollback netcode)
- winit (windowing)

### Nethercore ZX
- wgpu (graphics with 5th generation aesthetic)
- glam (math: vectors, matrices, quaternions)
- cpal + ringbuf (per-frame audio generation with rollback support)

### Library
- egui (library UI)
- reqwest (ROM downloads)

### Shared
- serde for serialization

## Project Structure

- `/core` — `nethercore-core` crate with Console trait, ConsoleRunner, WASM runtime, GGRS integration, debug inspection
- `/library` — `nethercore-library` binary (default workspace member) with library UI, console registry
- `/nethercore-zx` — `nethercore-zx` library implementing Console for 5th generation aesthetic
- `/z-common` — Z-specific formats, ZXRomLoader implementing RomLoader trait
- `/shared` — `nethercore-shared` crate with API types, cart formats, asset formats
- `/tools/nether-cli` — Build, pack, and run games (`nether build`, `nether pack`, `nether run`)
- `/tools/nether-export` — Convert assets to Nethercore formats (meshes, textures, audio, skeletons, animations)
- `/docs/architecture/` — FFI reference and internal architecture
- `/docs/contributing/` — Contributor guides
- `/docs/book/` — Game developer documentation (mdBook)
- `/examples/` — Example games demonstrating various features

## Conventions

### FFI Functions
- No prefix (e.g., `clear`, `draw_triangle`)
- Use C ABI: `extern "C" fn`

### Game Lifecycle
Games export: `init()`, `update()`, `render()`

- `init()` — Called once at startup
- `update()` — Called every tick (deterministic, used for rollback)
- `render()` — Called every frame (skipped during rollback replay)

### Rollback Netcode (GGRS)
The console uses GGRS for deterministic rollback netcode. This means:
- `update()` MUST be deterministic (same inputs → same state)
- Game state must be serializable for save/load during rollback
- No external randomness — use seeded RNG from host
- Tick rate is separate from frame rate (update can run multiple times per frame during catchup)

### Math Conventions
- **glam** for all math (vectors, matrices, quaternions)
- **Column-major** matrix storage (compatible with WGSL/wgpu)
- **Column vectors**: `v' = M * v`
- **Y-up**, right-handed coordinate system
- FFI angles in **degrees** (convert to radians internally)

**Matrix FFI formats (all column-major):**
- **4×4 matrices** (16 floats): `[col0.xyzw, col1.xyzw, col2.xyzw, col3.xyzw]`
  - Used by: `transform_set()`, `view_matrix_set()`, `proj_matrix_set()`
- **3×4 matrices** (12 floats): `[col0.xyz, col1.xyz, col2.xyz, col3.xyz]`
  - Used by: `set_bones()`, `load_skeleton()` (bone/inverse-bind matrices)
  - Implicit 4th row `[0, 0, 0, 1]` (affine transforms only)
  - Saves 25% memory vs 4×4 (48 bytes vs 64 bytes per matrix)

### Resource Management
- All graphics resources (textures, palettes, tilemaps) created in `init()`
- No `*_free` functions — resources auto-cleaned on game shutdown
- Vertex buffers: one buffer per stride, grows dynamically during init
- Immediate-mode draws buffered on CPU, flushed once per frame

### Asset Loading
Two approaches for loading assets:

**ROM Data Pack (recommended):** Assets bundled in `.nczx` ROM file, loaded by string ID:
```rust
// In init() - assets go directly to VRAM, bypass WASM memory
let texture = rom_texture(b"player".as_ptr(), 6);
let mesh = rom_mesh(b"level".as_ptr(), 5);
let sound = rom_sound(b"jump".as_ptr(), 4);
```

**Embedded Assets:** Use `include_bytes!()` with binary formats:
```rust
static MESH: &[u8] = include_bytes!("player.nczxmesh");
let handle = load_zmesh(MESH.as_ptr() as u32, MESH.len() as u32);
```

### Game Manifest (nether.toml)
Games are packaged using `nether.toml`:
```toml
[game]
id = "my-game"
title = "My Game"
author = "Developer"
version = "1.0.0"
render_mode = 2  # 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.meshes]]
id = "level"
path = "assets/level.nczxmesh"
```

### Debug Inspection System
Runtime value editing for development (F3 to open panel):
- Register values with `debug_register_*()` functions
- Organize with `debug_group_begin/end()`
- Read-only values with `debug_watch_*()` functions
- Frame control: F5=pause, F6=step, F7/F8=time scale
- Zero overhead in release builds (compiles out)

### Rendering Architecture (Nethercore ZX)

**Summary:**
- **4 render modes**: Lambert, Matcap, Metallic-Roughness (MR), Specular-Shininess (SS) — all use Blinn-Phong (set once in `init()`)
- **16 vertex formats**: Position + optional UV/Color/Normal + optional Skinned (runtime permutations)
- **40 shader permutations**: Generated from templates at compile-time
- **One vertex buffer per stride**: Avoids padding waste
- **Procedural sky**: Gradient + sun, provides ambient/reflection in all modes
- **2D/3D drawing**: Screen space sprites, world space billboards, immediate triangles, retained meshes
- **State-based color**: Use `set_color(0xRRGGBBAA)` before draw calls (2D functions don't take color params)
- **Z-ordering for 2D**: Use `z_index(n)` to control 2D draw order (0-255, higher draws on top)
- **Split-screen viewports**: Use `viewport(x, y, w, h)` and `viewport_clear()` for multiplayer

### Local Storage
```
~/.nethercore/
├── config.toml
├── games/{game_id}/
│   ├── manifest.json
│   ├── rom.wasm
│   └── saves/
```

## Launching Games

### Deep Links
`nethercore://play/{game_id}` — Download if needed, then play

### Command-Line Arguments
Games can be launched directly from the command line, bypassing the library UI:

```bash
# Launch by exact game ID
cargo run -- platformer

# Launch by prefix (case-insensitive, must be unique)
cargo run -- plat        # Matches "platformer"
cargo run -- CUBE        # Matches "cube"

# No argument launches the library UI
cargo run
```

**Priority order:** Deep links → CLI arguments → Library UI

**Game resolution features:**
- Exact case-sensitive matching (fast path)
- Case-insensitive matching
- Prefix matching (if unique)
- Levenshtein distance suggestions for typos
- Helpful error messages listing available games

**Implementation:**
- `library/src/registry.rs` - Game ID resolution and console registry
- `library/src/main.rs` - CLI argument parsing and entry point

## License

Dual-licensed under MIT OR Apache-2.0 (your choice).

## Claude Code Plugins

This repository works with Claude Code plugins for ZX game development assistance.

**Location:** `../nethercore-ai-plugins/`

### Available Plugins

| Plugin | Purpose |
|--------|---------|
| `ai-game-studio` | Intelligent request routing, GDD tracking, quality analysis, completion verification |
| `creative-direction` | Art, sound, tech directors - coherence and quality across disciplines |
| `game-design` | Platform-agnostic design - world building, narrative, characters, replayability |
| `sound-design` | Platform-agnostic audio - sonic style language, synthesis, music, SFX |
| `tracker-music` | Tracker module generation (XM/IT) - pattern design, end-to-end song generation |
| `zx-dev` | Core game development - FFI specs, project templates, rollback safety review |
| `zx-game-design` | ZX-specific design - GDDs, constraints, multiplayer patterns |
| `zx-procgen` | Procedural asset generation - textures, meshes, sounds, animations |
| `zx-publish` | Publishing workflow - ROM packaging, platform upload |
| `zx-orchestrator` | Meta-orchestration - coordinates full development pipeline |
| `zx-test` | Testing and QA - sync tests, replay regression, determinism |
| `zx-optimize` | Optimization - resource budgets, performance tuning |
| `zx-cicd` | CI/CD automation - GitHub Actions, quality gates |

### Plugin Structure

```
../nethercore-ai-plugins/
├── ai-game-studio/              # Intelligent game development studio
├── creative-direction/          # Art, sound, tech directors
├── game-design/                 # Platform-agnostic design
├── sound-design/                # Audio design and composition
├── tracker-music/               # Tracker module generation
├── zx-dev/                      # Core game development
├── zx-game-design/              # ZX-specific design
├── zx-procgen/                  # Procedural asset generation
├── zx-publish/                  # Publishing workflow
├── zx-orchestrator/             # Meta-orchestration
├── zx-test/                     # Testing and QA
├── zx-optimize/                 # Optimization
├── zx-cicd/                     # CI/CD automation
├── LICENSE-MIT
├── LICENSE-APACHE
└── README.md
```

Each plugin contains:
- `.claude-plugin/plugin.json` - Plugin manifest
- `skills/` - Auto-triggering knowledge skills
- `commands/` - Slash commands
- `agents/` - Specialized sub-agents

### Installation

Add to your global Claude settings (`~/.claude/settings.json`) or project settings:

```json
{
  "extraKnownMarketplaces": {
    "nethercore-ai-plugins": {
      "source": {
        "source": "github",
        "repo": "nethercore-systems/nethercore-ai-plugins"
      }
    }
  },
  "enabledPlugins": {
    "ai-game-studio@nethercore-ai-plugins": true,
    "creative-direction@nethercore-ai-plugins": true,
    "game-design@nethercore-ai-plugins": true,
    "sound-design@nethercore-ai-plugins": true,
    "tracker-music@nethercore-ai-plugins": true,
    "zx-dev@nethercore-ai-plugins": true,
    "zx-game-design@nethercore-ai-plugins": true,
    "zx-procgen@nethercore-ai-plugins": true,
    "zx-publish@nethercore-ai-plugins": true,
    "zx-orchestrator@nethercore-ai-plugins": true,
    "zx-test@nethercore-ai-plugins": true,
    "zx-optimize@nethercore-ai-plugins": true,
    "zx-cicd@nethercore-ai-plugins": true
  }
}
```

### Key Commands

**Project Setup (Token-Efficient):**
- `/init-procgen-infrastructure all` - Copy all procgen parsers (saves 95% tokens)
- `/init-tracker-music` - Copy XM/IT music writers (saves 85% tokens)

**Traditional Setup:**
- `/new-game [language] [name]` - Scaffold a new ZX game project
- `/design-game` - Interactive GDD builder wizard
- `/setup-project` - Full GDD + creative direction wizard

**Asset Generation:**
- `/generate-asset [type] [description]` - Quick asset generation
- `/publish-game` - Full publishing workflow

**Performance Note:**
Prefer `/init-procgen-infrastructure` and `/init-tracker-music` for new projects - they use native file copying instead of tokenized Read/Write operations, completing 10-20x faster with 85-95% fewer tokens.

### Example Queries

- "Create a new Nethercore ZX game in Rust"
- "How do I handle input in ZX?"
- "Generate a procedural texture for a brick wall"
- "Check my game for rollback issues"
- "What FFI functions are available for 3D rendering?"

The plugins reference actual source files for FFI specs, ensuring they stay current as the codebase evolves.

## Related
- `nethercore-platform` (private) — Backend API, web frontend
