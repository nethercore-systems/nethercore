# Nethercore Console Refactor / Cleanup TODO List

> Status: Working notes
> Last reviewed: 2026-01-06

Scope: `nethercore/` (runtime/player, ZX console implementation, shared types, tooling, docs). Module-by-module backlog aimed at long-term maintainability, deterministic correctness, and clean boundaries.

Legend: **P0** = foundational architecture/correctness, **P1** = high-ROI cleanup, **P2** = polish.

---

## P0 — Cross-Cutting (Console-Wide)

- [ ] Enforce determinism boundaries: anything that can run under rollback must not touch wall-clock time, OS RNG, filesystem/network, thread timing, or unordered iteration.
- [ ] Introduce explicit crate layering (no UI/network deps leaking into deterministic core); document and enforce with features + CI.
- [ ] Replace stringly IDs across runtime/shared/netplay with newtypes (e.g., `GameId`, `UserId`, `SessionId`) to prevent cross-ID mixups.
- [ ] Standardize timestamps: stop passing around ad-hoc RFC3339 `String`s; pick a single representation + helpers.
- [ ] Centralize configuration: one typed config loader with clear defaults and a “dev/prod” split; remove scattered `env` reads.
- [ ] Unify error strategy:
  - [ ] stable error codes for API/FFI surfaces
  - [ ] internal context for logs
  - [ ] avoid “stringly” error propagation in core paths.
- [x] Make logging consistent (prefer `tracing` end-to-end) and avoid logging secrets/PII.

---

## Repo Map — `nethercore/`

### `nethercore/core` (shared console framework)

#### `nethercore/core/src/app/*`

- [ ] Split mega modules into smaller, testable units:
  - [ ] `nethercore/core/src/app/player/mod.rs` (very large) into `state`, `ui`, `net_handshake`, `frame_loop`, `timing`, `errors`.
- [ ] Move any non-deterministic work out of rollback/simulation reachable code paths (audit `Instant`, random seeds, HashMap iteration).
- [x] Make tick-rate/fps derived from console specs (there’s a TODO to remove hardcoded fps).

#### `nethercore/core/src/ffi.rs`

- [ ] Reduce `unwrap()` density (currently high) and replace with structured errors + invariants.
- [x] Centralize safe WASM memory helpers (read/write slices/strings) and enforce bounds checks consistently.
- [ ] Ensure ABI parity with `nethercore/include/zx.rs` via generation + compile-time tests.

#### `nethercore/core/src/wasm/mod.rs`

- [ ] Break up the module (it’s large and central) into loader/instance/memory/imports/debug plumbing.
- [ ] Audit for determinism hazards (host functions must not use wall-clock or OS RNG unless explicitly “render-only”).

#### `nethercore/core/src/rollback/*`

- [ ] Replace remaining `panic!`/brittle invariants with explicit state machine transitions + errors.
- [x] Fix known drift: remove TODO hardcoded fps and derive from tick rate/specs.
- [x] Add a determinism audit checklist for any code reachable from rollback replays.

#### `nethercore/core/src/replay/*`

- [x] Make the headless runner real: `nethercore/core/src/replay/runtime/headless.rs` has TODOs to actually apply inputs and call `update()`.
- [x] Split script parsing/compiling responsibilities; avoid giant parser modules by separating AST, validation, and compilation stages.
- [x] Make replay output stable/diffable (deterministic ordering; avoid HashMap iteration order in outputs).

#### `nethercore/core/src/net/nchs/*`

- [ ] Isolate protocol/state machine from socket IO and timing (`Instant` usage is fine here, but keep it out of core sim code).
- [ ] Replace ad-hoc collections and implicit invariants with explicit types (peer handles, player slots, session state).

#### `nethercore/core/src/runtime.rs` / `nethercore/core/src/runner.rs`

- [ ] Clarify runtime responsibilities (timing, frame scheduling, stepping) vs app/player responsibilities.
- [ ] Make “one tick” semantics explicit and testable (inputs in, state out, no hidden IO).

---

### `nethercore/library` (main native player UI + launcher)

- [ ] Separate UI concerns from library state + persistence; make “library model” testable without UI.
- [ ] Consolidate update/install flows:
  - [x] `nethercore/library/src/update.rs` has a TODO for multi-console ROM support; define a console registry and remove hardcoded ZX assumptions.
- [ ] Ensure the launcher never depends on deterministic core internals directly; use a clean facade.

---

### `nethercore/nethercore-zx` (ZX console implementation)

#### High-risk “too large / too coupled” modules

- [ ] Split `nethercore/nethercore-zx/src/audio_thread.rs` (very large) into:
  - [ ] device/backend abstraction
  - [ ] mixing/scheduling
  - [ ] tracker integration
  - [ ] thread lifecycle + message protocol.
- [ ] Split `nethercore/nethercore-zx/src/tracker/engine.rs` into parser/engine/state/rendering, and add clear ownership boundaries.
- [ ] Split `nethercore/nethercore-zx/src/preview/viewers/mod.rs` into per-viewer modules; avoid a “registry god module”.

#### Graphics pipeline

- [ ] Break up `nethercore/nethercore-zx/src/graphics/unified_shading_state.rs` + `frame.rs` into smaller pieces (pipeline config, bind groups, frame graph, post).
- [ ] Reduce build-time codegen sprawl (`shader_gen.rs` + generated outputs): make the interface stable and test it.

#### FFI and state

- [ ] Audit `nethercore/nethercore-zx/src/state/ffi_state.rs` for clear separation:
  - [ ] deterministic state vs render caches vs IO handles.
- [ ] Ensure FFI entrypoints are thin adapters to safe internal APIs (no business logic in FFI glue).

---

### `nethercore/zx-common` (ZX formats / ROM loader)

- [ ] Split `nethercore/zx-common/src/formats/zx_data_pack.rs` into:
  - [ ] on-disk format structs
  - [ ] validation
  - [ ] encode/decode (streaming where possible).
- [ ] Add fuzz/property tests for all parsers (ROM, data pack, textures, skeletons, etc.).
- [ ] Define “trusted/untrusted” parsing tiers: anything that reads external bytes must be hardened and bounded.

---

### `nethercore/shared` (types shared with platform)

- [ ] Treat this as a public API surface:
  - [ ] version types explicitly
  - [ ] keep additive changes backwards compatible
  - [ ] document and test serialization formats.
- [ ] Centralize shared constants here (console specs, content ratings/tags, error codes) and generate JSON for the web frontend.

---

### `nethercore/include` (FFI surface)

- [ ] `nethercore/include/zx.rs` is huge: generate it (or generate wrappers) from a single canonical schema to prevent hand edits and drift.
- [ ] Add ABI conformance tests: host + wasm-side headers match in size/layout/signatures.

---

### `nethercore/tools/*` (CLI + exporters + generators)

- [x] Split `nethercore/tools/nether-cli/src/pack/mod.rs` into subcommands/modules (pack manifest building, asset ingestion, validation, output).
- [x] Deduplicate replay/compile shared logic (there’s an explicit TODO about deduping input layout docs).
- [ ] Ensure tools never reuse runtime-only internals in a way that creates accidental cyclic coupling; prefer `shared/` + `zx-common/` APIs.

---

### `nethercore/xtask`

- [ ] Make xtask steps declarative and reproducible; avoid “it works on my machine” env assumptions.
- [ ] Add “quick sanity” tasks: format, clippy (with allowlisted lints), deterministic replay smoke test.

---

### Docs (`nethercore/docs/*`)

- [ ] Keep docs in lockstep with code and generated headers (no drift):
  - [ ] console specs
  - [ ] FFI signatures
  - [ ] ROM format details.
- [ ] Fix/avoid non-UTF8 artifacts in generated docs/output so Windows terminals and mdBook render consistently.
