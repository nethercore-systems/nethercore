# Emberware - Claude Code Instructions

## Project Overview

Emberware is a fantasy console platform with built-in rollback netcode, designed to support multiple fantasy consoles (Emberware Z, Classic, etc.) with a shared framework.

**Console Status:**
- **Emberware Z** — Fully implemented (PS1/N64 aesthetic)
- **Emberware Classic** — Coming Soon (documented but not yet implemented)

**Repository Structure:**
- `/core` — Console trait, WASM runtime, GGRS rollback, ConsoleRunner, debug inspection
- `/library` — Main binary with library UI, console registry, game launcher
- `/emberware-z` — PS1/N64 aesthetic console implementation (library, no binary)
- `/z-common` — Z-specific formats, ROM loader
- `/shared` — API types for platform backend, cart/ROM formats
- `/tools` — Developer tools (ember-cli, ember-export)
- `/docs` — Game developer documentation
- `/examples` — Example games

**Key Documentation:**
- [TASKS.md](./TASKS.md) — Development status and implementation plan
- [docs/reference/ffi.md](./docs/reference/ffi.md) — Shared FFI API reference
- [docs/reference/emberware-z.md](./docs/reference/emberware-z.md) — Z-specific API
- [docs/reference/rendering-architecture.md](./docs/reference/rendering-architecture.md) — Graphics deep dive

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
│          │           emberware-z (lib)                      │
│  ┌───────▼───────┐  ┌─────────────┐  ┌─────────────────┐   │
│  │ EmberwareZ    │  │ ZGraphics   │  │ Z-specific FFI  │   │
│  │ Console impl  │  │ (wgpu)      │  │ (draw_*, etc)   │   │
│  └───────┬───────┘  └─────────────┘  └─────────────────┘   │
│          │          ┌─────────────┐  ┌─────────────────┐   │
│          │          │ ZAudio      │  │ ZRollbackState  │   │
│          │          │ (cpal)      │  │ (audio state)   │   │
│          │          └─────────────┘  └─────────────────┘   │
├──────────┼──────────────────────────────────────────────────┤
│          │              emberware-core                      │
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
- Easy addition of future consoles (Emberware Y, X, etc.)

### Input Abstraction

Each console defines its own input struct:

```rust
// Emberware Z (PS2/Xbox style)
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

// Emberware Classic (6-button retro)
#[repr(C)]
pub struct ClassicInput {
    pub buttons: u16,  // D-pad + A/B/C/X/Y/Z + L/R + start/select
}
```

The core handles GGRS serialization of whatever input type the console uses.

## Tech Stack

### Core
- wasmtime (WASM execution)
- GGRS (rollback netcode)
- winit (windowing)

### Emberware Z
- wgpu (graphics with PS1/N64 aesthetic)
- glam (math: vectors, matrices, quaternions)
- cpal + ringbuf (per-frame audio generation with rollback support)

### Library
- egui (library UI)
- reqwest (ROM downloads)

### Shared
- serde for serialization

## Project Structure

- `/core` — `emberware-core` crate with Console trait, ConsoleRunner, WASM runtime, GGRS integration, debug inspection
- `/library` — `emberware-library` binary (default workspace member) with library UI, console registry
- `/emberware-z` — `emberware-z` library implementing Console for PS1/N64 aesthetic
- `/z-common` — Z-specific formats, ZRomLoader implementing RomLoader trait
- `/shared` — `emberware-shared` crate with API types, cart formats, asset formats
- `/tools/ember-cli` — Build, pack, and run games (`ember build`, `ember pack`, `ember run`)
- `/tools/ember-export` — Convert assets to Emberware formats (meshes, textures, audio, skeletons, animations)
- `/docs/reference/` — FFI reference and API documentation for game developers
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

**ROM Data Pack (recommended):** Assets bundled in `.ewz` ROM file, loaded by string ID:
```rust
// In init() - assets go directly to VRAM, bypass WASM memory
let texture = rom_texture(b"player".as_ptr(), 6);
let mesh = rom_mesh(b"level".as_ptr(), 5);
let sound = rom_sound(b"jump".as_ptr(), 4);
```

**Embedded Assets:** Use `include_bytes!()` with binary formats:
```rust
static MESH: &[u8] = include_bytes!("player.ewzmesh");
let handle = load_zmesh(MESH.as_ptr() as u32, MESH.len() as u32);
```

### Game Manifest (ember.toml)
Games are packaged using `ember.toml`:
```toml
[game]
id = "my-game"
title = "My Game"
author = "Developer"
version = "1.0.0"
render_mode = 2  # 0=Unlit, 1=Matcap, 2=PBR, 3=Hybrid

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.meshes]]
id = "level"
path = "assets/level.ewzmesh"
```

### Debug Inspection System
Runtime value editing for development (F3 to open panel):
- Register values with `debug_register_*()` functions
- Organize with `debug_group_begin/end()`
- Read-only values with `debug_watch_*()` functions
- Frame control: F5=pause, F6=step, F7/F8=time scale
- Zero overhead in release builds (compiles out)

### Rendering Architecture (Emberware Z)

**Summary:**
- **4 render modes**: Unlit, Matcap, Metallic-Roughness (MR), Specular-Shininess (SS) — all use Blinn-Phong (set once in `init()`)
- **16 vertex formats**: Position + optional UV/Color/Normal + optional Skinned (runtime permutations)
- **40 shader permutations**: Generated from templates at compile-time
- **One vertex buffer per stride**: Avoids padding waste
- **Procedural sky**: Gradient + sun, provides ambient/reflection in all modes
- **2D/3D drawing**: Screen space sprites, world space billboards, immediate triangles, retained meshes

### Local Storage
```
~/.emberware/
├── config.toml
├── games/{game_id}/
│   ├── manifest.json
│   ├── rom.wasm
│   └── saves/
```

## Launching Games

### Deep Links
`emberware://play/{game_id}` — Download if needed, then play

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

## Related
- `emberware-platform` (private) — Backend API, web frontend
