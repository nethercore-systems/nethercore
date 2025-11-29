# Emberware - Claude Code Instructions

## Project Overview

Emberware is a 5th-generation fantasy console platform designed to support multiple fantasy consoles with shared rollback netcode infrastructure.

- **Core** — Shared console framework (WASM runtime, GGRS rollback, game loop)
- **Emberware Z** — PS1/N64 aesthetic fantasy console (first implementation)
- **Shared** — API types shared with the platform backend
- **Docs** — FFI documentation for game developers
- **Examples** — Example games

See [TASKS.md](./TASKS.md) for current development status and implementation plan.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    emberware-z                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ ZGraphics   │  │ ZAudio      │  │ Z-specific FFI  │ │
│  │ (wgpu)      │  │ (rodio)     │  │ (draw_*, etc)   │ │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         │                │                   │          │
│         └────────────────┼───────────────────┘          │
│                          │ implements Console trait     │
├──────────────────────────┼──────────────────────────────┤
│                    emberware-core                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ Console     │  │ Runtime<C>  │  │ Common FFI      │ │
│  │ trait       │  │ game loop   │  │ (input, save,   │ │
│  │             │  │ GGRS        │  │  random, etc)   │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │ WasmEngine  │  │ Rollback    │                      │
│  │ (wasmtime)  │  │ state mgmt  │                      │
│  └─────────────┘  └─────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

### Console Trait

Each fantasy console implements the `Console` trait:

```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;  // Console-specific input layout

    fn name(&self) -> &'static str;
    fn specs(&self) -> &ConsoleSpecs;
    fn register_ffi(&self, linker: &mut Linker<GameState>) -> Result<()>;
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;
    fn create_audio(&self) -> Result<Self::Audio>;
    fn map_input(&self, raw: &RawInput) -> Self::Input;
}

// Must be POD for GGRS serialization
pub trait ConsoleInput: Clone + Copy + Default + bytemuck::Pod + bytemuck::Zeroable {}
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
- matchbox_socket (WebRTC P2P networking)
- winit (windowing)

### Emberware Z
- wgpu (graphics with PS1/N64 aesthetic)
- glam (math: vectors, matrices, quaternions)
- rodio (audio)
- egui (library UI)
- reqwest (ROM downloads)

### Shared
- serde for serialization

## Project Structure

- `/core` — `emberware-core` crate with Console trait, WASM runtime, GGRS integration
- `/emberware-z` — `emberware-z` binary implementing Console for PS1/N64 aesthetic
- `/shared` — `emberware-shared` crate with API types
- `/docs/ffi.md` — FFI reference for game developers
- `/examples/hello-world` — Minimal example game

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
- `transform_set()` takes 16 floats in column-major order: `[col0, col1, col2, col3]`

### Resource Management
- All graphics resources (textures, palettes, tilemaps) created in `init()`
- No `*_free` functions — resources auto-cleaned on game shutdown
- Vertex buffers: one buffer per stride, grows dynamically during init
- Immediate-mode draws buffered on CPU, flushed once per frame

### Local Storage
```
~/.emberware/
├── config.toml
├── games/{game_id}/
│   ├── manifest.json
│   ├── rom.wasm
│   └── saves/
```

## Deep Links
`emberware://play/{game_id}` — Download if needed, then play

## Related
- `emberware-platform` (private) — Backend API, web frontend
