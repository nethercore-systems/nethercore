# nethercore -- Agent Notes

Detailed context for agents working in this repo. See `CLAUDE.md` for hard rules and verification commands.

## Repo Map

- `core/` -- Shared console framework (WASM runtime, rollback, netcode, inspection)
- `library/` -- Main native player binary (library UI + launcher)
- `nethercore-zx/` -- ZX console implementation
- `shared/` -- Types shared with platform backend (`nethercore-platform`)
- `zx-common/` -- ZX formats/ROM loader (also used by platform backend)
- `include/` -- FFI surface; **canonical ZX FFI signatures live in `include/zx.rs`**
- `tools/` -- `nether-cli` (build/pack/run), exporters, dev tooling
- `docs/book/` -- mdBook game developer docs
- `docs/architecture/` -- Internal architecture notes
- `examples/` -- Example games
- `xtask/` -- Build orchestration (`cargo xtask ...`)

## Commands

```bash
# Build + run
cargo build --release
cargo run                     # Player (library UI)

# Testing + quality
cargo test
cargo fmt
cargo clippy --all-targets -- -D warnings

# WASM target (for games/examples)
rustup target add wasm32-unknown-unknown

# xtask orchestration
cargo xtask build-examples

# mdBook (requires mdbook + mdbook-tabs)
cd docs/book && mdbook serve
```

Cargo aliases (see `.cargo/config.toml`):
- `cargo xtask ...` -- alias to `run --package xtask --`
- `cargo dev` -- release build of `nethercore-zx` + `nether-cli`

## Cross-Repo Notes

- `nethercore-platform/backend` depends on `shared/` and `zx-common/` via local path.
- Coordinated changes must keep that repo building.
- `include/zx.rs` is the ABI source of truth; update docs/examples when changing FFI.

## Coding Conventions

- Determinism boundary: `update()` and everything it calls must be pure. No wall-clock time, OS RNG, filesystem, network, or unordered iteration in simulation paths.
- Rendering code lives in rendering paths only -- never mix simulation state mutations into render.
- Prefer `xtask` for build orchestration over ad-hoc scripts.

## Common Pitfalls

- Forgetting `wasm32-unknown-unknown` target when building examples.
- Changing `shared/` or `zx-common/` types without verifying `nethercore-platform/backend` still compiles.
- Introducing non-determinism in code reachable from `update()`.
