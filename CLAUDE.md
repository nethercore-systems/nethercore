# Nethercore

Runtime/player, shared types, console implementations (ZX), tooling, and game-dev documentation.

## Hard Rules

- Anything reachable from `update()` must be deterministic and rollback-safe.
- Avoid non-deterministic sources in simulation: wall-clock time, OS RNG, filesystem/network, thread timing, unordered iteration, floating-point drift.
- Keep render-only work in rendering paths; rollback can re-run `update()` many times.
- `include/zx.rs` is the ABI source of truth; verify any FFI change against it.
- Cross-repo: `nethercore-platform/backend` depends on `shared/` and `zx-common/` via local path; changes must keep that repo building.

## Verify

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Navigate

| I need to...                | Start here                              |
|-----------------------------|-----------------------------------------|
| Understand game-facing APIs | `include/zx.rs`                         |
| ROM packing + asset formats | `docs/architecture/rom-format.md`, `zx-common/` |
| Rollback / determinism      | `docs/architecture/overview.md`         |
| ZX rendering architecture   | `docs/architecture/zx/rendering.md`     |
| NCHS handshake protocol     | `docs/architecture/nchs.md`             |
| Game-dev docs (mdBook)      | `docs/book/`                            |
| Repo map + full commands    | `AGENTS.md`                             |

## SSOT

| What               | Where                              |
|--------------------|------------------------------------|
| ZX FFI signatures  | `include/zx.rs`                    |
| Runtime architecture | `docs/architecture/overview.md`  |
| Game-dev docs      | `docs/book/`                       |
| Repo map           | `AGENTS.md`                        |
