# Emberware - Claude Code Instructions

## Project Overview

Emberware is a 5th-generation fantasy console platform. This public repo contains:

- **Emberware Z** — Native game runtime (Rust, wgpu, wasmtime)
- **Shared** — Types shared with the platform backend
- **Docs** — FFI documentation for game developers
- **Examples** — Example games

## Tech Stack

### Emberware Z
- Rust, wgpu (graphics), wasmtime (WASM), winit (windowing), rodio (audio), egui (minimal UI)
- GGRS (rollback netcode), matchbox_socket (WebRTC P2P), reqwest (HTTP)

### Shared
- serde for serialization

## Project Structure

- `/emberware-z` — Native runtime binary
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
