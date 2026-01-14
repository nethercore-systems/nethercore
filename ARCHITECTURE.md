# Nethercore Architecture

> This file is a high-level orientation and code map.
> Deep dives live in `docs/architecture/` (start with `docs/architecture/overview.md`).

## TL;DR (what lives where)

- `core/` — console-agnostic runtime (WASM, rollback, netcode, inspection)
- `nethercore-zx/` — ZX console implementation (renderer, audio, ZX-specific FFI)
- `library/` — native player/launcher (UI + game discovery)
- `zx-common/` — ZX formats shared with build tools and the platform backend
- `shared/` — shared types used by `nethercore-platform/backend`
- Audio/tracker: `nether-tracker/`, `nether-it/`, `nether-xm/`, `nether-qoa/`
- Tooling: `tools/`, `xtask/`
- ABI surface: `include/` (canonical ZX FFI signatures in `include/zx.rs`)
- Docs: `docs/book/` (mdBook game-dev docs), `docs/architecture/` (internal architecture)

## Start here (canonical docs)

- Runtime/player architecture: `docs/architecture/overview.md`
- ZX FFI (ABI): `include/zx.rs`
- NCHS handshake protocol: `docs/architecture/nchs.md`
- ZX rendering architecture: `docs/architecture/zx/rendering.md`

## Quick lookups

| Topic | Pointer |
| --- | --- |
| Rollback + determinism guardrails | `core/rollback/`, `docs/architecture/overview.md` |
| WASM runtime + FFI registration | `core/wasm/`, `core/ffi/` |
| Netplay / handshake protocol | `core/net/nchs/`, `docs/architecture/nchs.md` |
| ZX FFI bindings | `nethercore-zx/ffi/`, `include/zx.rs` |
| ZX renderer implementation | `nethercore-zx/graphics/`, `docs/architecture/zx/rendering.md` |
| Replay record/playback | `core/replay/` |
| CLI/tools | `tools/`, `xtask/` |
| Shared types used by platform backend | `shared/`, `zx-common/` |

## Repo map (top level)

```
nethercore/
├── core/              # Shared console framework (WASM, rollback, netcode)
├── nethercore-zx/     # ZX console implementation
├── library/           # Native player UI + launcher
├── zx-common/         # ZX formats/ROM loader shared with tools/platform
├── shared/            # Types shared with platform backend
├── include/           # FFI surface; canonical ZX ABI in include/zx.rs
├── docs/              # mdBook + internal architecture notes
├── nether-tracker/    # Unified tracker engine (IT/XM playback)
├── nether-it/         # Impulse Tracker format parser/writer
├── nether-xm/         # FastTracker XM format parser
├── nether-qoa/        # QOA audio codec
└── tools/             # CLI tools and asset exporters
```

## Core crates

### `core/` — Console framework

Foundation for all Nethercore consoles. Console-agnostic infrastructure.

**Key types:**

- `Console` trait — implemented by each fantasy console
- `Runtime` — fixed-timestep game loop
- `GameInstance` — WASM game loaded via wasmtime
- `RollbackSession` — GGRS integration for netplay
- `WasmEngine` — shared wasmtime engine for compilation

**Key modules:**

- `app/` — windowed application framework (winit, egui)
- `rollback/` — state management, input synchronization, GGRS wrapper
- `net/nchs/` — Nethercore Handshake protocol for P2P connections
- `wasm/` — wasmtime runtime + FFI registration
- `ffi/` — common FFI functions (time, RNG, save data)
- `debug/` — debug panel, variable inspection, action system
- `replay/` — replay recording and playback

### `nethercore-zx/` — ZX console

ZX fantasy console implementation. Provides ZX-specific graphics, audio, and FFI.

**Key types:**

- `ZXConsole` — implements `core::Console`
- `ZXGraphics` — wgpu-based renderer
- `ZXAudio` / `ThreadedAudioOutput` — audio output with optional threading
- `TrackerEngine` — IT/XM module playback

**Key modules:**

- `graphics/` — pipeline, textures, buffers, unified shading
- `ffi/` — ZX FFI bindings (draw_2d, mesh, audio, environment)
- `tracker/` — tracker playback engine with rollback support
- `procedural/` — mesh generation (cube, sphere, capsule, etc.)
- `preview/` — asset viewers for development tools
- `state/` — per-frame FFI state management

### `zx-common/` — shared formats/loader

ZX binary formats used by both runtime and build tools.

**Key types:**

- `ZXDataPack` — packed asset container with lazy indexing
- `ZXRomLoader` — ROM loading and validation
- `PackedMesh`, `PackedTexture`, `PackedKeyframes`, …

**Used by:** `nethercore-zx`, tools, and the platform backend.

### `library/` — native launcher

Platform-specific launcher binary and game discovery.

**Key types:**

- `PlayerLauncher` — builder for launching games
- `ConsoleRegistry` — ROM loader factory
- `LocalGame` — discovered local game metadata

## Audio / tracker crates

### `nether-tracker/`

Unified tracker module representation for playback.

**Key types:**

- `TrackerModule` — format-agnostic module
- `TrackerInstrument`, `TrackerSample`, `TrackerPattern`
- `from_it_module()`, `from_xm_module()` — converters

### `nether-it/`

Impulse Tracker format support.

**Key types:**

- `ItModule` — parsed IT file
- `ItWriter` — IT file creation
- `parse_it()`, `pack_ncit()` — parsing and minimal packing

### `nether-xm/`

FastTracker XM format support.

**Key types:**

- `XmModule` — parsed XM file
- `parse_xm()`, `pack_xm_minimal()` — parsing and packing

## Dependency graph (conceptual)

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

## Entry points

| Entry point | Crate | Description |
| --- | --- | --- |
| `library/src/main.rs` | `library` | Native player binary |
| `run_standalone()` | `core` | Windowed game execution |
| `Console::run_frame()` | `core` | Per-frame console update |
| `GameInstance::call_update()` | `core` | WASM game tick |

## Key invariants (read before changing behavior)

- **Rollback-safe design:** anything reachable from game `update()` must be deterministic. Render-only work stays in render paths since rollback re-runs simulation.
- **FFI ABI stability:** `include/zx.rs` is the signature source of truth; keep console bindings and docs in sync with it.
- **Cross-repo shared types:** `nethercore-platform/backend` depends on `shared/` and `zx-common/` via local path; coordinated changes must keep it building.
