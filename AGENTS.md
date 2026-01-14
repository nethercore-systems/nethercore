# nethercore (Rust)

Fantasy console runtime/player with rollback netcode, console implementations (ZX), shared types, CLI tooling, and game-dev documentation.

## Start Here (Canonical)

- Runtime/player overview: `docs/architecture/overview.md`
- ZX rendering architecture: `docs/architecture/zx/rendering.md`
- ZX FFI ABI (signatures + docs): `include/zx.rs`
- ROM format + packing: `docs/architecture/rom-format.md`
- NCHS (handshake protocol + netplay model): `docs/architecture/nchs.md`

## Repo Map

- `core/` — Shared console framework (WASM runtime, rollback, inspection)
- `library/` — Main native player binary (library UI + launcher)
- `nethercore-zx/` — ZX console implementation
- `shared/` — Types shared with platform backend (`nethercore-platform`)
- `zx-common/` — ZX formats/ROM loader (also used by platform backend)
- `include/` — FFI surface; **canonical ZX FFI signatures live in `include/zx.rs`**
- `tools/` — `nether-cli` (build/pack/run), exporters, dev tooling
- `docs/book/` — mdBook game developer docs
- `docs/architecture/` — Internal architecture notes
- `examples/` — Example games
- `xtask/` — Build orchestration (`cargo xtask ...`)

## Quick Commands

- Install WASM target (for games/examples): `rustup target add wasm32-unknown-unknown`
- Build: `cargo build --release`
- Run player (library UI): `cargo run`
- Tests: `cargo test`
- Format: `cargo fmt`
- Build examples: `cargo xtask build-examples`
- Serve mdBook: `cd docs/book; mdbook serve` (requires `mdbook` + `mdbook-tabs` installed)

Handy cargo aliases (see `.cargo/config.toml`):
- `cargo xtask …` (alias to `run --package xtask --`)
- `cargo dev` (release build of `nethercore-zx` + `nether-cli`)

## Rollback / Determinism Guardrails

- Treat `update()` logic (game + simulation) as deterministic and rollback-safe.
- Avoid non-deterministic sources in simulation: wall-clock time, OS RNG, filesystem/network, thread timing, unordered iteration, floating-point drift without care.
- Keep render-only work in rendering paths; rollback can re-run `update()` many times.

## Cross-Repo Notes (Shared Types / FFI)

- `include/zx.rs` is the ABI source of truth; verify any FFI change against it and update docs/examples accordingly.
- `nethercore-platform/backend` depends on `shared/` and `zx-common/` via local path; coordinated changes must keep that repo building.
