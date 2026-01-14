# Nethercore - Claude Code Instructions

## Purpose

This repo contains the Nethercore runtime/player, shared types, console implementations (ZX), tooling, and documentation.

## TL;DR (How to Navigate)

- For **game-facing APIs / ABI**, treat `include/zx.rs` as the source of truth.
- For **ROM packing + asset formats**, start at `docs/architecture/rom-format.md` and `zx-common/`.
- For **rollback/determinism**, start at `docs/architecture/overview.md` and keep anything reachable from `update()` deterministic.

## Start Here (Canonical Docs)

- Game-dev docs (mdBook): `docs/book/`
- Runtime/player architecture: `docs/architecture/overview.md`
- ZX FFI signatures (ABI): `include/zx.rs`
- NCHS (handshake protocol): `docs/architecture/nchs.md`
- ZX rendering architecture: `docs/architecture/zx/rendering.md`

## Repo Map (High Level)

- `core/` — shared console framework (WASM runtime, rollback, netcode, inspection)
- `library/` — native player UI + launcher
- `nethercore-zx/` — ZX console implementation
- `zx-common/` — ZX formats/ROM loader (also used by platform backend)
- `shared/` — shared types used by platform backend (`nethercore-platform`)
- `tools/` — CLI + exporters
- `docs/` — mdBook + architecture notes
- `examples/` — example games

## Quick Commands

- Build: `cargo build`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Run player (library UI): `cargo run`
- Serve mdBook: `cd docs/book && mdbook serve` (requires `mdbook` + `mdbook-tabs`)

## Cross-Repo Dependency (Local Paths)

`../nethercore-platform/backend` depends on:

```toml
nethercore-shared = { path = "../../nethercore/shared" }
zx-common = { path = "../../nethercore/zx-common", default-features = false }
```

When changing shared types, keep platform building.

## Guardrails (Rollback/Determinism)

- Treat anything reachable from `update()` as deterministic and rollback-safe.
- Keep render-only work in rendering paths; rollback can re-run simulation many times.

## AI Plugins

See `../nethercore-ai-plugins/` (legacy plugin packs for Claude Code/Codex workflows).
