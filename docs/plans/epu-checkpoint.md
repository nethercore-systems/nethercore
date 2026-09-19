# EPU working checkpoint

Status: the **bounded SCATTER pole follow-up is locally verified**, including a fresh two-case native rerun. On 2026-09-20 Robert authorized small validated commit/push checkpoints so work can pause safely; SCATTER is the first such checkpoint on base `b3628e59bb6f2d09574962c33c9ca3ddc220d799`. Hosted Vulkan/llvmpipe acceptance and the broader EPU plan remain open. Keep subsequent failure investigations separate until validated.

## Bounded SCATTER pole follow-up 2026-09-18

Three finite tasks completed: reproduce one failure family, repair only its cause,
and verify/document the local result. No new algorithms, scene changes or performance work.

- **Reproduction:** the unchanged `domain_boundary_scatter_angular_limits` check
  and a new direct-coordinate regression fail on Microsoft Basic Render Driver
  (DX12, driver `10.0.26100.9278`), exposing `[NaN, 1, NaN, 0]` at an exact pole.
  The old boundary check passes on NVIDIA RTX 3080 / Vulkan (`591.86`). This is
  backend-dependent undefined zero-vector normalization, not a tolerance problem.
- **Repair:** `scatter_cyl_uv` and `scatter_polar_uv` return canonical azimuth zero
  only when their projection is exactly zero. Existing height/radius, support fades,
  all non-singular equations, seed/phase handling and packed API remain unchanged.
  Both legacy SCATTER and SCATTER_PHASED use these shared helpers.
- **Focused checks:** both software-DX12 regressions pass; the NVIDIA SCATTER filter
  reports **12 passed / 0 failed / 1 ignored** (the pre-existing hardware cost probe).
  The added check covers **192 pole rows, 960 non-pole mapping pairs and 1,920
  rejected shifted-azimuth controls**, comparing non-pole GPU f32 before half storage.
  Default adapter selection passes; an explicitly nonexistent adapter fails closed,
  with no silent fallback. The test harness uses wgpu's existing `WGPU_BACKEND` and
  `WGPU_ADAPTER_NAME` controls; these are not new cartridge/runtime settings.
- **Real route:** inspector export → canonical SDK → WASM → packed cartridge →
  native player passes for legacy CYL and phased POLAR. **1,036,800 pixels**, zero
  failed/excluded/changed pixels, maximum channel error **1** at fixed limit **2**;
  both wrong-direction controls reject. All **six owned capture players** exited,
  including two final-player requalification runs after Cargo relinked it.
- **Other checks:** workspace formatting, focused GPU-target Clippy with warnings
  denied, source/helper preservation and the retained-artifact verifier pass.
  SDK, example scenes and other production shaders were not changed.

Evidence: `tmp/epu-review/scatter-pole-20260918-144339/closure.json`.
Recompute it without launching a GPU test/player:

```bash
python tmp/epu-review/scatter-pole-20260918-144339/verify.py
```

To reproduce the focused software-backend check, use this guide's canonical
Windows target/temp environment, then:

```bash
WGPU_BACKEND=dx12 WGPU_ADAPTER_NAME='Microsoft Basic Render Driver'   cargo test --locked --offline -p nethercore-zx --test epu_gpu   scatter::scatter_pole_coordinates_are_finite_and_preserve_nonpoles   -- --exact --nocapture --test-threads=1
```

Retained harness failures are not renderer RED or hidden passes. The generic
compute surrogate disagreed at six legacy-CYL pixels; the literal-input raster
surrogate still disagreed at two (maximum error 14 in both attempts). Matching the
native triangle interpolation **and storage-backed camera/instruction inputs**
passes at the original tolerance, without changing any guest/camera values or
excluding pixels. These observations do not establish a particular compiler/FMA
cause. A shader-edit typo and the post-test player relink were caught and repaired;
original failure logs/receipts and all earlier captures remain intact.

**Remaining:** exact hosted Vulkan/llvmpipe rerun, other hosted failure families
(including reflection), then the plan's remaining engineering, human visual/
usefulness review and deferred performance gate. This batch does not reduce the
historical CI failure count by assumption or claim whole-EPU completion. The next
bounded engineering batch should classify another hosted failure family before
TRACE/new scene work; human acceptance remains Robert's decision.


## Hosted GPU CI blocker

**Not CI-green; cross-backend technical acceptance remains open.** The code at
`1c2416ac281e84ea1d68fc724d1b43561d7f5ae7` passed hosted format, generated-binding
synchronization and Clippy. Its [completed CI run](https://github.com/nethercore-systems/nethercore/actions/runs/34871214298)
failed the GPU target on Vulkan / `llvmpipe (LLVM 20.1.2, 256 bits)`, Mesa
`25.2.8-0ubuntu0.24.04.2 (LLVM 20.1.2)`: **153 passed, 13 failed, 2 ignored**.
The ZX library passed **539 tests, with 1 ignored**. Cargo stopped at the failed
GPU target; this is not a successful full hosted workspace run.

Exact failing tests:

- `celestial_uv::celestial_uv_caller_maps_the_projected_disk_and_rejects_per_pixel_radius`
- `cell_continuity::cell_grid_partial_boundary_oracle`
- `cell_grid_zero::cell_grid_zero_width_archived_outline_rejects`
- `cell_grid_zero::cell_grid_zero_width_archived_wrap_rejects`
- `cell_hex_2d_high::cell_hex_2d_high_fixed_owner_falsifier`
- `cell_hex_2d_medium::cell_hex_2d_medium_small_gap`
- `cell_uniform::cell_hex_periodic_gate_rejects_injected_seam`
- `cell_uniform::cell_hex_periodic_wrap_and_rows_all_density_bytes`
- `domain_boundary::domain_boundary_patches_hard_ownership`
- `domain_boundary::domain_boundary_scatter_angular_limits`
- `grid_regular::grid_chart_seam_control_rejected`
- `grid_regular::grid_regular_chart_wrap`
- `reflection::reflection_imported_runtime_basis_switch_continuity`

At the recorded run these failures were unclassified. The bounded follow-up
above classifies the SCATTER pole NaNs as a production defect; the other families
and the exact hosted rerun remain open. Concrete retained examples include:

- CELESTIAL's current UV evaluator has zero failures, but the archived negative
  control rejects 36,732 records rather than the asserted 36,744.
- A CELL boundary oracle observes `0.49975586` where it asserts exact `0.5`.
- HEX fixtures straddle 17,874 of 18,432 requested boundaries, failing their
  own coverage precondition; excluding the missed boundaries is not a repair.
- The SCATTER exact-pole and imported-reflection continuity checks report NaNs.
  Do not dismiss these as harmless rounding or claim their runtime path is safe.

No assertion, tolerance, negative control, fixture, shader or runtime code was
changed to bypass these failures. The earlier Windows/native receipts remain
valid for their source, inputs and adapter, not as proof of Vulkan portability.
Full job evidence is linked above; local copies and a machine-readable failure
inventory are in `tmp/epu-review/checkpoint-20260915/` (`ci-job-104067272395-clean.log`
and `hosted-gpu-blocker.json`). The exact cases are committed here so the restart
does not depend on those ignored local copies.

**Disposition:** the standing goal remains paused for Robert's usage break.
Hermes resumes this GPU-portability investigation only on explicit instruction,
before TRACE or any broader rendering work. Reproduce against the logged adapter
and exact inputs; distinguish harness defects from production behavior before
changing either. Preserve all thresholds, negative controls and pixel coverage.
Do not turn failed checks into ignored tests or use an unobserved fallback adapter
as evidence. This documentation-only update does not rerun the unchanged failing
GPU suite; its skipped CI is not a passing result.

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

1. **Hermes, only after Robert resumes: close the hosted GPU portability blocker above.** Preserve the exact failing cases and all existing thresholds; establish each root cause before changing probes or production. Do not start TRACE or new scene/mechanism work first.
2. **Hermes: TRACE polar opposite-pole defect.** The retained audit found 15 failing pole rows for TRACE/POLAR variants 1–3, not a universal TRACE failure. A known case is opcode 12, domain 2, variant 1, direction 0 (TRACE's +Y default), parameters `[64,20,128,37]`. Packed GPU words are `[620757239,3225425024,572671180,1735519931]`. At -Y, 16 azimuthal approaches plus the exact pole at shrinking transverse radii `.01,.001,.0001,.00001` retain a premultiplied RGBA span `0.7529296875` against the declared `.005` limit. Start at `trace_polar_uv` and `trace_segment_distance` in `04_trace.wgsl`; turn this into the focused RED before changing production. Do not silently mirror a fade: a visibility change needs Robert's decision. Other TRACE domains are not interchangeable coordinate systems.
3. **Hermes: reconcile remaining finite acceptance rows** in `epu-progress.md`, not an unbounded sweep of every byte combination. Preserve sealed source/input/tolerance identities. Add a mechanism only for a demonstrated expressive gap, not because a scene is unattractive.
4. **Robert: visual/usefulness review.** Review a finite indoor/outdoor/hybrid set across the requested themes. Grove/Arcade approvals are scoped; city identity, WARPED_RADIAL coarseness and overall composition usefulness are not automatically accepted. Technical tests and model inspection cannot grant this verdict.
5. **Hermes, after an explicit quiet-GPU window: performance acceptance.** Timestamp probes remain opt-in/ignored during shared AI rendering. No performance claim accompanies this checkpoint.

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

## Hosted CI compatibility follow-up

The first hosted run for `fd586093a75f9f4f916c0ed1f6c21f50d82e7b7b`
passed format and binding synchronization, then stopped before tests on Rust
1.98's new `chunks_exact_to_as_chunks` style lint. The local toolchain is Rust
1.94.1; adding a blanket lint allowance would introduce an unknown-lint error
there. The follow-up instead uses the standard-library fixed-array chunk views
at exactly the 45 diagnosed sites in 19 GPU-test source files.

Only test-harness iteration syntax changes: shader literals, assertions,
thresholds, fixtures, runtime/SDK code and cartridge inputs are unchanged. The
old/new iterators match on all 22 affected chunk sizes over 174 boundary-length
cases, including empty inputs, remainder handling, shared storage and mutable
writes. Workspace Clippy passes locally, and 8 representative real GPU tests
pass (reflection/SH9 mutation, large PLANE groups, PATCHES and phased SCATTER).
Full hosted acceptance remains the exact pushed revision's CI result below;
no lint/test suppression, new visual repair, or performance acceptance is added.

## Verified checkpoint results

**Scope: the following are retained local Windows receipts, not hosted Vulkan acceptance.**

- Full workspace: **1,766 tests passed, 0 failed, 50 ignored** across unit, integration and documentation targets. Included: **167 GPU tests** (2 timing probes ignored) and **539 ZX library tests** (1 timing probe ignored). Other ignored documentation tests are not performance evidence.
- Final Cargo target-selection fix: **82 CLI tests passed**. Its focused regression first failed on host-helper discovery, then passed; real-output ambiguity, stale output rejection and explicit overrides remain covered.
- Workspace Clippy with warnings denied and generated ZX FFI synchronization pass. Final formatting and staged-diff hygiene are checked before committing.
- The only post-workspace shader edit removes a final blank line from `16_scatter_phased.wgsl`; executable text is identical. Its **3 focused GPU tests** and a fresh native build pass. No numerical threshold or pixel exclusion changed.
- Both documented `nether build` commands compile and pack real WASM. The rebuilt player runs three showcase replays and the inspector for 32 updates each. City, Grove and Arcade match their retained native reference frames exactly; all four owned players are reaped. No host EPU configuration is injected.
- Local book and executable phase snippet pass: **7 changed pages, 389 local links, 187 anchors; all 256 phase values**. Inspector metadata/docs also compile.
- Active GPU test source/runtime inputs are durable. SURFACE's formerly ignored reference is now a source fixture. Hash-bound WGSL fixture bytes are preserved through Git attributes and verified against index blobs.

Full-workspace receipts precede the isolated CLI fix and shader EOF cleanup; their affected paths were re-tested afterwards. Historical verifiers may reject later formatting/docs changes: preserve original receipts and reconcile exact drift rather than bypassing hash checks.

Local logs and native images: `tmp/epu-review/checkpoint-20260915/`. Rebuilt book/snippet receipts: `tmp/epu-review/checkpoint-docs-20260915/`. These are optional audit outputs, not build dependencies. Hosted CI is a separate post-push gate; its [commit-linked run](https://github.com/nethercore-systems/nethercore/actions/workflows/ci.yml), not local success, is authoritative.
