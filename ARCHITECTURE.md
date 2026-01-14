# Nethercore Architecture

## Crate Overview

```
nethercore/
├── core/              # Shared console framework (WASM, rollback, netcode)
├── nethercore-zx/     # ZX console implementation
├── library/           # Native player UI + launcher
├── zx-common/         # ZX formats shared with build tools
├── shared/            # Types shared with platform backend
├── nether-tracker/    # Unified tracker engine (IT/XM playback)
├── nether-it/         # Impulse Tracker format parser/writer
├── nether-xm/         # FastTracker XM format parser
├── nether-qoa/        # QOA audio codec
└── tools/             # CLI tools and asset exporters
```

## Core Crates

### `core/` - Console Framework
Foundation for all Nethercore consoles. Console-agnostic infrastructure.

**Key types:**
- `Console` trait - implemented by each fantasy console
- `Runtime` - game loop with fixed timestep
- `GameInstance` - WASM game loaded via wasmtime
- `RollbackSession` - GGRS integration for netplay
- `WasmEngine` - shared wasmtime engine for compilation

**Modules:**
- `app/` - windowed application framework (winit, egui)
- `rollback/` - state management, input synchronization, GGRS wrapper
- `net/nchs/` - Nethercore Handshake protocol for P2P connections
- `wasm/` - wasmtime runtime, FFI registration
- `ffi/` - common FFI functions (time, RNG, save data)
- `debug/` - debug panel, variable inspection, action system
- `replay/` - scripted replay recording and playback

### `nethercore-zx/` - ZX Console
The ZX fantasy console implementation. Provides all ZX-specific graphics, audio, and FFI.

**Key types:**
- `ZXConsole` - implements `core::Console`
- `ZXGraphics` - wgpu-based renderer
- `ZXAudio` / `ThreadedAudioOutput` - audio output with optional threading
- `TrackerEngine` - IT/XM module playback

**Modules:**
- `graphics/` - pipeline, textures, buffers, unified shading
- `ffi/` - ZX FFI bindings (draw_2d, mesh, audio, environment)
- `tracker/` - tracker playback engine with rollback support
- `procedural/` - mesh generation (cube, sphere, capsule, etc.)
- `preview/` - asset viewers for development tools
- `state/` - per-frame FFI state management

### `zx-common/` - Shared Formats
ZX binary formats used by both runtime and build tools.

**Key types:**
- `ZXDataPack` - packed asset container with lazy indexing
- `ZXRomLoader` - ROM loading and validation
- `PackedMesh`, `PackedTexture`, `PackedKeyframes`, etc.

**Used by:** nethercore-zx, nether-cli, nether-export, platform backend

### `library/` - Native Launcher
Platform-specific launcher binary and game discovery.

**Key types:**
- `PlayerLauncher` - builder for launching games
- `ConsoleRegistry` - ROM loader factory
- `LocalGame` - discovered local game metadata

## Audio Crates

### `nether-tracker/`
Unified tracker module representation for playback.

**Key types:**
- `TrackerModule` - format-agnostic module
- `TrackerInstrument`, `TrackerSample`, `TrackerPattern`
- `from_it_module()`, `from_xm_module()` - converters

### `nether-it/`
Impulse Tracker format support.

**Key types:**
- `ItModule` - parsed IT file
- `ItWriter` - IT file creation
- `parse_it()`, `pack_ncit()` - parsing and minimal packing

### `nether-xm/`
FastTracker XM format support.

**Key types:**
- `XmModule` - parsed XM file
- `parse_xm()`, `pack_xm_minimal()` - parsing and packing

## Dependency Graph

```
┌─────────────┐
│   library   │ ─────────────────────────────────┐
└──────┬──────┘                                  │
       │                                         │
       ▼                                         ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│nethercore-zx│ ───▶│    core     │     │   shared    │
└──────┬──────┘     └──────┬──────┘     └─────────────┘
       │                   │
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│  zx-common  │     │    ggrs     │
└──────┬──────┘     └─────────────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│nether-tracker│───▶│  nether-it  │
└──────┬──────┘     │  nether-xm  │
       │            └─────────────┘
       ▼
┌─────────────┐
│  nether-qoa │
└─────────────┘
```

## Entry Points

| Entry Point | Crate | Description |
|-------------|-------|-------------|
| `library/src/main.rs` | library | Native player binary |
| `run_standalone()` | core | Windowed game execution |
| `Console::run_frame()` | core | Per-frame console update |
| `GameInstance::call_update()` | core | WASM game tick |

## Key Patterns

**Rollback-safe design:** Anything reachable from `update()` must be deterministic. Render-only work stays in render paths since rollback re-runs simulation.

**FFI registration:** Each console registers FFI functions via `Linker`. Core provides common FFI (time, RNG, save), console provides specific FFI (graphics, audio).

**State pools:** Pre-allocated buffers for rollback snapshots avoid allocation during gameplay.

**Threaded audio:** Optional `AudioGenThread` for predictive audio generation, reducing main thread load.
