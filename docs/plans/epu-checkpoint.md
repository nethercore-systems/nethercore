# EPU working checkpoint

Status: paused for the requested usage break. This is a working-product checkpoint, not completion of the full EPU correctness/open-authoring plan. Resume only when Robert asks. The commit-linked CI result is authoritative for hosted checks.

## Use the product

From the repository root, with the Rust WASM target installed:

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked -p nethercore-zx --bin nethercore-zx -p nether-cli --bin nether
```

Put those freshly built binaries on PATH (use Cargo's actual target directory, not an assumed project-local target), then:

```sh
mkdir -p tmp
nether build -p examples/3-inspectors/epu-showcase -o tmp/epu-showcase.nczx
nethercore-zx tmp/epu-showcase.nczx
nether build -p examples/3-inspectors/epu-inspector -o tmp/epu-inspector.nczx
nethercore-zx tmp/epu-inspector.nczx
```

See the showcase README for A/B scene navigation, Start suite switching, X mesh switching and F4 inspection. The showcase has 20 editable compositions and seven capability scenes, not runtime-owned themes. Use inspector LOCK only for host editing; unlock it when checking guest-owned animation.

## Retained work

- Lighting/reflection/normal-map and SH9 repairs, truthful variant-aware editor controls, corrected field units, raw /256 cyclic phases, and executable SDK/book guidance.
- Opt-in `SCATTER_PHASED` at `0x18`; legacy `SCATTER` remains `0x0A`. Guest state owns animation and rollback.
- Existing mechanism repairs, regular plus organic CELL authoring, non-fading PATCHES cap coordinates, authorized VEIL polar support, and bounded cartridge/native evidence recorded in the progress ledger.
- Preserved editable scene structure changes, including Grove, Arcade and Neon Metropolis. No engine-owned weather, moods or geometry fog.
- Durable GPU fixtures and regressions in `nethercore-zx/tests/epu_gpu/`; no ordinary active test requires the ignored review archive as an input. Historical unregistered experiments remain diagnostic-only.

## Known rough edges and next steps

1. **Hermes: TRACE polar opposite-pole defect.** The retained audit found 15 failing pole rows for TRACE/POLAR variants 1–3, not a universal TRACE failure. A known case is opcode 12, domain 2, variant 1, direction 0 (TRACE's +Y default), parameters `[64,20,128,37]`. Packed GPU words are `[620757239,3225425024,572671180,1735519931]`. At -Y, 16 azimuthal approaches plus the exact pole at shrinking transverse radii `.01,.001,.0001,.00001` retain a premultiplied RGBA span `0.7529296875` against the declared `.005` limit. Start at `trace_polar_uv` and `trace_segment_distance` in `04_trace.wgsl`; turn this into the focused RED before changing production. Do not silently mirror a fade: a visibility change needs Robert's decision. Other TRACE domains are not interchangeable coordinate systems.
2. **Hermes: reconcile remaining finite acceptance rows** in `epu-progress.md`, not an unbounded sweep of every byte combination. Preserve sealed source/input/tolerance identities. Add a mechanism only for a demonstrated expressive gap, not because a scene is unattractive.
3. **Robert: visual/usefulness review.** Review a finite indoor/outdoor/hybrid set across the requested themes. Grove/Arcade approvals are scoped; city identity, WARPED_RADIAL coarseness and overall composition usefulness are not automatically accepted. Technical tests and model inspection cannot grant this verdict.
4. **Hermes, after an explicit quiet-GPU window: performance acceptance.** Timestamp probes remain opt-in/ignored during shared AI rendering. No performance claim accompanies this checkpoint.

## Recheck before resuming

```sh
git status --short
cargo fmt --all -- --check
cargo xtask ffi check --console zx
cargo clippy --workspace --all-targets --keep-going -- -D warnings
cargo test --workspace -- --test-threads=1
```

The GPU regression target requires a real available wgpu adapter; the headless host needs the platform dependencies configured in CI. Do not add `--ignored` to the ordinary run: that would run deferred timing probes. For a narrow next slice, use `cargo test -p nethercore-zx --test epu_gpu <filter> -- --nocapture --test-threads=1`.

On this Windows checkout, retain the existing native TEMP/TMP/TMPDIR and `CARGO_TARGET_DIR=C:/Users/rdave/AppData/Local/Temp/nethercore-target` convention. Use one authoritative checkout and stop only test players you own. Do not delete ignored `tmp/epu-review/`: it contains historical source-bound receipts and before/after media, not required production dependencies or a substitute for the committed tests.

## Verified checkpoint results

- Full workspace: **1,766 tests passed, 0 failed, 50 ignored** across unit, integration and documentation targets. Included: **167 GPU tests** (2 timing probes ignored) and **539 ZX library tests** (1 timing probe ignored). Other ignored documentation tests are not performance evidence.
- Final Cargo target-selection fix: **82 CLI tests passed**. Its focused regression first failed on host-helper discovery, then passed; real-output ambiguity, stale output rejection and explicit overrides remain covered.
- Workspace Clippy with warnings denied and generated ZX FFI synchronization pass. Final formatting and staged-diff hygiene are checked before committing.
- The only post-workspace shader edit removes a final blank line from `16_scatter_phased.wgsl`; executable text is identical. Its **3 focused GPU tests** and a fresh native build pass. No numerical threshold or pixel exclusion changed.
- Both documented `nether build` commands compile and pack real WASM. The rebuilt player runs three showcase replays and the inspector for 32 updates each. City, Grove and Arcade match their retained native reference frames exactly; all four owned players are reaped. No host EPU configuration is injected.
- Local book and executable phase snippet pass: **7 changed pages, 389 local links, 187 anchors; all 256 phase values**. Inspector metadata/docs also compile.
- Active GPU test source/runtime inputs are durable. SURFACE's formerly ignored reference is now a source fixture. Hash-bound WGSL fixture bytes are preserved through Git attributes and verified against index blobs.

Full-workspace receipts precede the isolated CLI fix and shader EOF cleanup; their affected paths were re-tested afterwards. Historical verifiers may reject later formatting/docs changes: preserve original receipts and reconcile exact drift rather than bypassing hash checks.

Local logs and native images: `tmp/epu-review/checkpoint-20260915/`. Rebuilt book/snippet receipts: `tmp/epu-review/checkpoint-docs-20260915/`. These are optional audit outputs, not build dependencies. Hosted CI is a separate post-push gate; its [commit-linked run](https://github.com/nethercore-systems/nethercore/actions/workflows/ci.yml), not local success, is authoritative.
