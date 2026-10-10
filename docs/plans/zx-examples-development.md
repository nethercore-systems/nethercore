# ZX examples development contract: 90-project curriculum

Status: WORKING — the current execution request explicitly retains the current single development owner and removes the stale model/effort requirement. Foundation qualification and delivery reconciliation continue; required CI remains unresolved.
Plan review: 2026-10-10, owner source/contract review; not independent runtime certification.
Reviewed baseline: `ceed1becb4545313950e3ac089ff7ae4caae2a81`; local `main` and live `origin/main` agreed when inspected. Re-read live state before execution.

This is the sole implementation scope and per-example progress ledger. It replaces the exploratory scratch CSV/report as an execution contract. Everything needed to build the curriculum is below or in repository-relative sources. Do not require a chat transcript, transient scratch file or private sibling checkout.

## Outcome and authority

Deliver exactly **90 distinct, source-available, documented, native-player-verified example projects**: retain/rehabilitate the **44 existing** projects and add **46** named projects. They teach useful basics, individual mechanisms and medium/advanced combinations. Every item below is required; a source folder, screenshot, compile, static discussion of an unimplemented effect, or README-only placeholder does not qualify.

The human's later execution request authorizes implementation and ordinary commits/pushes of this curriculum to `nethercore-systems/nethercore`, target `origin/main`. Merely writing this contract does not start a goal, commit or publish work. Keep one authoritative checkout and one development owner. Model/session selection belongs to the explicit execution handoff, not a hidden repository configuration.

Ponytail full: read the actual path first, reuse existing examples/helpers/tools, choose small guest recipes, and change shared code only for a reproduced defect. No generic scene graph/ECS/UI/physics/animation framework, new verification platform, 90-way template expansion, or catalogue padding. Preserve the distinctive, visible learning objective of each entry.

## Review corrections that are now binding

1. **Source coverage is not qualification.** All entries start unchecked, including existing examples. Catalogue totals derive from tracked project source/manifests, not leftover directories or old ROMs. Current tracked source count is 42 Rust/Cargo + C + Zig. Existing code may need no changes if fresh acceptance evidence suffices.
2. **Dynamic resources have a real boundary.** Retained mesh/texture loading and procedural resource creation must obey current init-only guards. Keep CPU-known geometry where required; submit changing shadow/decal/rope/water geometry through bounded immediate triangles, not new retained allocations per frame. ROM mesh handles are not a CPU vertex-readback API. No mutable texture API is assumed.
3. **Six legal mesh routes, not a Cartesian product.** Retained float/indexed-float/packed/indexed-packed and immediate float/indexed-float. There is no packed-immediate guest API. Verify real vertex layout, RGB versus alpha, tangents, u16 indices and buffer ceilings; outdated comments are not the ABI.
4. **Stencil/depth must be proved on ZX.** Both faces share one stencil state per pass. Read/write masks are full byte, and the reference is shared with comparisons/replacement. Color writes are suppressed only for stencil-writing passes whose stencil comparison is ALWAYS; conditional mask writes can affect visible color. Depth clears affect the entire attachment, not only one viewport. The stencil-write convenience function ignores depth; visible receiver marking needs explicit full-pass depth state. Same-pass CPU call order can be reordered by batching: use execution barriers when the algorithm requires ordering.
5. **Dither is not framebuffer blending.** A radial blob needs texture alpha or explicit coverage geometry, not an invented per-vertex alpha field in an RGB format. Shadows, water and shafts must not claim multiply/additive/soft-alpha composition. No new renderer modes are silently authorized.
6. **Projection recipes are scoped honestly.** CPU orthographic markings and known-heightfield decals are in scope. Vertex-divided perspective UVs are an approximation with an explicit reference/error witness. Planar mirrors/linked portals are direct re-renders, one/two-level visual studies, not RTT or physical traversal. Clipping must match actual ZX/WebGPU conventions.
7. **Seven pilots are proof gates, not escape hatches.** Items 46, 77, 78, 79, 80, 86 and 87 need the stated discriminating native/audio proof before polish. An impossible or failing objective stays BLOCKED with evidence; do not replace it with a generic window, a static image, a tutorial about failure, or a cheaper unrelated item and still claim 90/90. Continue independent work; request a bounded scope/API decision only when genuinely necessary.
8. **Libraries are candidates, not compulsory architecture.** Reuse a suitable existing dependency only after public availability/license, wasm/no_std, determinism, CPU and memory checks. Pin reproducible inputs. No path to a sibling checkout or private game is a shipping dependency; no cross-repo implementation is authorized by this goal. Tiny existing guest code is acceptable when it is the better implementation.
9. **Explicit manifest behavior.** Set id/title/author/version, render mode, tick rate, max players and netplay support intentionally. Use single-player/offline defaults for isolated lessons; opt appropriate games/audio tests into their documented multiplayer contract. Do not inherit the packer's four-player/netplay-enabled defaults accidentally. Do not claim live joining merely from player_count or a UI label.
10. **Reproducible source, not hidden generated inputs.** The source/generator, licensing, asset IDs and regeneration command must be present. Respect `.gitignore`: compiled cartridges/WASM, generated preview media and caches are not source deliverables. Commit necessary authored source assets or small generators under intentional paths; do not force-add every generated PNG/GLB/audio file.
11. **Budget and timing claims are measured.** Respect the current canonical RAM/VRAM/ROM/tick limits from `shared/src/console.rs`; do not relax them to make examples fit. Count decoded resources separately from compressed files. Separate CPU timing from real GPU instrumentation. Compare culling with fixed LOD for equivalence, and assess LOD's quality/cost tradeoff separately.
12. **Current source and intended API are distinct.** Do not remove, expand or add examples dependent on `epu_asset`. Its existence does not establish product approval. Use the existing explicit `rom_texture` plus `epu_textures` route. Preserve unrelated EPU authoring/renderer work and accepted visuals; the curriculum does not authorize taking over that workstream.

## Scope, preservation and stopping rules

Primary scope: `examples/`, existing asset generators needed by examples, `examples/examples-common` only for demonstrated reuse, `xtask/`, the relevant example build/pack/preview wiring, `docs/book/`, `examples/README.md`, this ledger, and focused CI/README fixes needed to cover the complete catalogue.

A narrow runtime/packer repair is permitted only for a reproduced contract violation that blocks a named example, with one relevant regression witness, affected-caller inspection and preserved ABI. Changes to public APIs, cartridge formats, hardware limits, unrelated EPU algorithms, protected-branch rules, or other repositories require a separate decision. Never turn a failed pilot into open-ended engine redevelopment.

Before edits, record HEAD, branch/upstream, tracked/untracked dirty paths and exact overlapping diffs in a local ignored ownership receipt. Detect another live owner/build on the same target; do not compete with it. Do not stash, reset, clean, move to disposable worktrees, overwrite unfamiliar changes, regenerate dirty bindings wholesale, or commit another task's edits. The baseline already contains unrelated EPU changes; work around them and reconcile overlapping files without erasing them. No global formatting churn.

Human taste and engineering readiness are different. Produce actual native captures/audio for review and repair concrete visible/audible defects. Do not claim human ACCEPTED without an actual verdict. Do not invent a mandatory 90-click approval ceremony or a second review app. Pending optional taste review does not stop independent engineering or authorized commits; a known unresolved named defect does block the affected item's READY state.

Continue through ordinary tool/build failures by fixing the cause or using a supported alternative. A real authorization/toolchain/network/ownership blocker must name the exact failed step and needed action. Continue independent items; if nothing authorized remains, persist the exact state and report BLOCKED, not DONE. Do not run endless retries, silently omit a language/platform, or downgrade an objective to reach the total.

## Definition of ready (five gates)

**G1 — Complete, distinct source:** all 90 exact paths below are real independent buildable projects with the named teaching outcome, canonical version-matched bindings and intentional metadata. Existing useful material is preserved; no placeholders, empty stubs, palette clones or prose-only pilots.

**G2 — Reproducible build and packaging:** each project builds from repository source and documented, obtainable toolchains; each final `.nczx` includes required assets/configuration and launches after relocation without the checkout. All Rust projects participate in example discovery; C and Zig have explicitly exercised build/pack/native routes and are included in the aggregate catalogue gate rather than silently skipped. No secret/private/sibling dependency or untracked required input.

**G3 — Real-path behavior and regressions:** exercise the actual current native player and the specific witness below, including reset, control interaction and the failure-revealing edge case. Leave one small runnable check for nontrivial changed logic; reuse replay/sync-test and existing tests. Visual claims require actual native output, audio claims actual playback/listening, save claims restart/reload, and network claims real clients plus a separate forced-rollback/sync witness. A plain boot or test name is not evidence for these different claims. Define tolerances before judging math/image/performance checks. Revalidate affected evidence when shared code/assets change.

**G4 — Teachability and review-ready presentation:** readable 960x540 layout, discoverable controller/available mapped controls, reset, optional diagnostic view that exposes the mechanism, concise README describing why it works and where it fails, editable/license-traceable assets, and a representative captured result. No clipped instructions, unexplained flicker, stuck input, broken audio or default-looking placeholders. Tiny tutorials may use deliberate simple geometry; sophisticated examples must visually demonstrate the sophisticated technique. Deliver actual review media through the existing approved route; do not claim fun/art acceptance from a machine check.

**G5 — Shipped source and exact remote evidence:** review the staged diff, commit only task-owned source/docs/checks in coherent batches, deliver the completed set onto `origin/main`, and read back the exact remote commit/ref and applicable CI results. Fresh checkout/reconstruction of the committed tree must pass the source/asset/build gate so unpublished local changes cannot be carrying the results. No force-push, bypassed branch protection, open-but-unmerged PR as completion, fabricated CI, or binaries-only handoff. Final counts and evidence must match the same delivered source.

An item reaches READY after G1-G4; SHIPPED additionally requires G5. Only SHIPPED gets its top-level checkbox checked. Human verdict is an independent field; unreviewed is not accepted. The whole engineering goal closes only at 90 SHIPPED with no unresolved required gate. If a branch rule requires a PR, use the normal route; completion still means the commits are on main, not merely uploaded elsewhere.

## Implementation order and dependencies

Work in coherent bounded batches, one owner, not 90 concurrent jobs. Baseline-verify existing prerequisites rather than gratuitously rewriting them. Per-item prerequisites below take precedence over phase labels; phases are work lanes, not a reason to delay a prerequisite until a later wave. Keep actual dependencies acyclic.

- **Phase 0 — ownership and build contract:** reconcile the live baseline, inspect toolchains/CI/current flags, count tracked projects, repair manifest/discovery issues, define owned output locations, and prove clean source-to-cartridge on one representative Rust, C and Zig case. Before touching risky feature recipes, establish actual pass/geometry/clip semantics with the smallest native witness.
- **A — foundation:** first-game/input/time/RNG/UV/transforms/cameras/mesh routes/fonts plus the existing stencil/viewport/mirror-inset lessons. Establish conventions through a few real projects; do not generate boilerplate for all 90 up front.
- **B — advanced rendering:** prioritize 69, 71, 74, 75, 72, 73 and 84, while proving 78/79/77 with small scenes early. Promote proved recipes to polished examples, then build their dependent combinations (80, 83, 87). Treat image/error references as narrow tests, not renderer replacements.
- **C — feature depth:** rehabilitation of rendering inspectors, animation/import/attachments/IK, audio/tracker/adaptive/rollback, assets/fonts/compression/source-to-cartridge. Existing prerequisites here can be qualified during A/B.
- **D — complete combinations:** retain/qualify paddle/platformer/netplay; then build the one-room/one-track/one-puzzle system studies 60-65. Avoid adding a genre framework or an online requirement to a deliberately local lesson.
- **E — catalogue closure:** aggregate all 90, repair docs/discovery/preview inconsistencies, run source-clean/relocated output gates and relevant regressions, deliver review material, commit/push/read back main and applicable CI. Mark only verified entries shipped.

## Verification and delivery mechanics

Read live help and current repo scripts before using flags. Reuse `nether build`, `nether pack`, `nethercore-zx`, existing replay (`.ncrs`), `cargo xtask build-examples` and the actual CI workflows. Do not invent a `nether test` or assume a tool option exists. The repaired `xtask build-examples` discovers either Cargo or `nether.toml` projects, including C/Zig, and may clean stale installed examples; isolate the owned installation output and preserve unrelated user library entries. Discover the existing `--games-dir` behavior before executing a broad build.

For a change, run its nearest check, build/package and actual witness first. A final bounded run covers the affected crates and the advertised example matrix. Repository/workflow checks include Rust format, bindings, workspace clippy/tests and preview/example build; exact current commands live in `CLAUDE.md`, `xtask/src/main.rs` and `.github/workflows/`. Do not blindly run a binding generator over unrelated dirty FFI files. Add only the minimal existing-tool integration needed to make missing C/Zig/example failures fail the aggregate gate.

Retain local media/log evidence in an ignored, persistent project directory such as `tmp/zx-examples/<run-id>/`, not expiring assistant scratch or committed caches. Keep a compact text receipt per example/batch tied to source commit or tree/dirty hash, build/player/toolchain identity, exact command, exit/result, named fixture/replay and media/output identity. Do not serialize a second competing status database. This file owns progress; receipts prove it. Commit small tests/replays and result summaries where appropriate, not the generated build/media tree. A clone may regenerate evidence from the committed recipe; remote CI links provide shareable build evidence.

For final clean-source acceptance, an isolated disposable **verification export** of the committed tree with owned output directories is permitted; it is not a second development checkout/worktree. Use the committed player/source where affected; do not accidentally verify against an uncommitted renderer repair from another task. Remove only owned verification output when safe.

Git delivery:
1. Recheck branch/upstream, remote and new upstream commits before each delivery. Never auto-push unrelated local commits or absorb someone's dirty files to make a clean tree.
2. Stage explicit task paths/hunks, inspect `git diff --cached`, ensure no credentials/private audit records/generated artifacts, and commit a coherent tested batch. Include this plan and its doc-map link in the first task commit.
3. Use ordinary fast-forward integration/push to the verified `nethercore-systems/nethercore` main. If remote changed, reconcile without force/reset and rerun affected gates. Respect permissions/protected checks; report conflicts that require an owner decision.
4. Read back `refs/heads/main` and confirm the delivery commit is on it. Retrieve relevant CI for that commit (including example/preview and bindings when triggered), distinguish pending/failed/passed. Do not close on a successful push alone.
5. Update statuses/evidence honestly. Remote verification facts can be recorded in the next checkpoint/final response; do not create an infinite commit loop merely because a commit cannot name its own resulting hash. Final response supplies the final delivery SHA, remote readback, CI handles, 90/90 count and any separate human-review state.

If an unrelated pre-existing CI failure blocks required main delivery, record its exact failure/ownership and continue independent work. Do not bypass protection or fix arbitrary unrelated subsystems. Keep overall delivery blocked until the required gate is resolved or the user explicitly revises it.

## Source anchors and scope limits

Repository sources inspected for this review (re-read when changed):
- `include/zx.rs`, `include/zx/mod.rs`, `include/zx/{drawing,mesh,pass,constants,render,material,texture,transform,camera,viewport,text,music,system}.rs`: exposed guest calls and layouts.
- `nethercore-zx/src/ffi/mesh.rs`, `nethercore-zx/src/ffi/guards.rs`: retained init-only loading; immediate geometry lives under `ffi/draw_3d.rs`.
- `nethercore-zx/src/graphics/render_state.rs`, `graphics/frame/pass_execution.rs`, `graphics/pipeline/pipeline_creation.rs`, `graphics/command_buffer.rs`: actual pass/face/color/mask/clear/batching contracts.
- `nethercore-zx/shaders/quad_template.wgsl`, mode/common shaders and `shared/src/console.rs`: dither/UV path and resource limits.
- `tools/nether-cli/src/manifest.rs`, `tools/nether-cli/src/replay/`, `examples/replay/`, `xtask/src/main.rs`, `.gitignore`, `.github/workflows/{ci,ffi-check,tool-preview}.yml`: build/format/source/verification constraints.
- `examples/README.md`, `docs/book/src/reference/examples.md`, and the source paths named below: current examples, not assumed quality.

Not in this execution: guest RTT, new projector/decal ABI, explicit depth-bias ABI, general blend modes, stencil bitmask/independent-color extensions, engine animation/UI/physics frameworks, dynamic mid-session joining, all-device performance guarantees or an EPU redesign. A named example that proves such an addition necessary needs a separate decision; it does not grant blanket permission to extend the console.

## Live checkpoint

- Work state: WORKING. The latest execution request explicitly says to continue with the current single owner and removes model/effort requirements. No model migration, new worker or further owner gate is needed. Preserve all five gates and the exact 90-entry scope.
- Automatic continuation: the saved goal was last read back ACTIVE with the same current-owner clause now supplied by the continuation. The later attempted pause/old-contract restoration DID NOT RUN because its approval expired. Do not run `checkpoint_zx_owner_gate.py`: it would restore an obsolete requirement. No goal-control write or global configuration change is needed for this continuation.
- Qualified: 15/90 (01-03, 05-09, 16-17, 20, 49, 66-67, 69, G1-G4 against receipt-bound verified hosts); shipped: 0/90. The uncommitted mesh-storage host repair is NOT yet native-qualified; do not extend earlier receipts to a rebuilt host without revalidation. Required foundation CI failed. Fresh metadata readback for published `d2cb997` confirms CI run `38036826637` FAILED; Deploy Documentation, Latest Release and Tool Preview Packet succeeded. Evidence: `pivot-ci-readback-after-camera.json`; this is status metadata, not a retry of denied older log retrieval. Human review remains unreviewed.
- Active batch/item: 69 is READY locally at `4e3a6b032358d1d6a49383fb961fc62e4907866f`, tree `1d5c45fbd1db9722e13766c4af1fc7e826a38882`; fresh exact-export qualification passed (`boolean-committed-local-verification.json`, `boolean-committed-qualified/receipt.json`). It proves all 24 combinations with 26 native images, 7,000,032 interior classifications, borders/holes, typed snapshots and forced rollback. Ten task-owned paths were committed; 68 working-tree files were byte-preserved. Source-derived catalogue count is 51, not the stale 44 (`boolean-catalogue-inventory.json`); source presence is not readiness. Exact review item `zx-curriculum-69-e0a5c48870e1` passed durable record/media hash and a 960x540 LAN app-card preview after the initial queue-load race; no duplicate enqueue or vote. Eight project cards were pending, zero decided, and all 1010 earlier records were unchanged (`boolean-review-submission.json`). Live remote main readback remains `d2cb997b3119565e68343d2ae8d0fb9a78be8ccb`; no push or CI success is claimed.
- Previous delivery: 17 is delivered and exact-tree verified at `d2cb997b3119565e68343d2ae8d0fb9a78be8ccb`; 20 is committed locally at `7332a7ff6de9942c7888f426a07a9a88e477a86c` and passes fresh exact-export unit/build/native/negative/rollback verification (`transform-committed-verification/receipt.json`, `transform-committed/`). Its push command DID NOT RUN: approval expired. Do not retry/rephrase the push or publish this commit through another route without fresh consent. Local reversible curriculum work remains authorized; this is not whole-goal BLOCKED or completion. Save-slots real persistence and older CI-log retrieval remain separately authorization-blocked.
- Next action: qualify independent Phase B region-stack lesson 70 against prerequisite 69 and the unchanged verified viewport-only host. The scoped 68 formatting/frozen-oracle preflight remains paused after an expired approval, as do the earlier camera/mesh native diagnostic, publication and older CI-log/input probes; do not retry them through another route without fresh consent. Other independent work remains, so the whole goal is neither DONE nor globally BLOCKED. Human taste is unreviewed.
- Previous review delivery: 49's exact committed native image is queued as `zx-curriculum-49-ca44018bc42d`; loopback durable-record readback, LAN media SHA-256 and a decoded 960x540 browser card without horizontal overflow passed (`font-review-submission.json`). The app returned its documented HTTP 201; the exact target was read back without resubmitting after the client initially expected 200. At that readback five project cards were pending and zero decided. No service, lease, firewall, token, generation setting or review verdict was changed.
- New authorization event: the combined prior-transform receipt/font-directory/CLI-and-player-help terminal probe did not run because approval expired. Do not retry/rephrase that probe without fresh consent. The independent font build/native checks above were separately executed successfully using the established runner; this is not a camera repair retest.
- Owner/session: `20261010_100903_97bf0099`; routing key `agent:main:telegram:dm:2086218583:36859`. Current single Astra owner retained under the latest request. No worker, global model change, Hermes runtime/configuration change or replacement session was started.
- Ownership: retain `tmp/zx-examples/20261010-astra/ownership.json`, the initial 34-file preservation receipt and backups. Unrelated EPU source/bindings remain outside task commits. After the user's explicit continuation, fresh Git status/numstat confirmed the task-owned foundation/runtime edits and the preserved foreign EPU work. `proc_d8fdf3342d2f` is exited with code 0, not an active competing builder; no duplicate verifier was started.
- Completed build: the delayed `proc_8d6e2f467cf3` notification confirms the baseline player/CLI/xtask build exited 0. Do not infer current binary identity from that notification: the same export/target directories were subsequently used for an explicitly uncommitted runtime-repair probe.
- Completed committed-tree verifier: `proc_d8fdf3342d2f` exited 0. Its output reports format, host build, unit, Clippy, bindings, binding compile and first-cartridge checks passing for exact reconstruction `45836b8819b08eb5253a24374b9e5bae404e5ad0`. This is not a committed-tree pass for the later uncommitted foundation/runtime edits.
- Last verified delivery: `45836b8` and caption follow-up `09068cbf73aa74f06fc86dec3dc5cbef228122e8` were pushed to `origin/main`; exact remote readback matched the latter SHA at delivery. All three caption-fixed cartridges passed relocated replay, negative assertions, matching native pixels and rollback smoke; receipt: `tmp/zx-examples/20261010-astra/first-cartridges-caption-fixed/receipt.json`.
- Fresh hosted readback for that exact SHA: documentation `38022168501`, FFI bindings `38022168579`, Latest Release `38022168543` and Tool Preview Packet `38022168475` all completed successfully, including all listed jobs. CI `38022168502` failed job `114125306629` (Tests + Lint), step `Tests`. Full run/job results: `tmp/zx-examples/20261010-astra/first-batch-ci.json`. Retrieve the failed test log before classifying this failure as baseline or introduced; its cause has not been established by this checkpoint.
- CI log access: the attempted `fetch_zx_ci_log.py` invocation for job `114125306629` was blocked before execution by an approval timeout. No retry or alternate credential route was used. Fresh authorization is required before retrying that denied operation; the exact failure cause remains unclassified.
- Uncommitted repair evidence: `screen_space_geometry_preserves_requested_origins` exercises the actual WASM line/circle/outline/sprite imports. RED then GREEN logs are `origin-red.log` and `origin-green.log`; all six drawing tests passed after the shared origin correction. `origin-native-red-green.json` records the actual RNG plot changing from 27 disconnected components to one, with a centered runner. No public API or shader change was required. These local probes do not certify the eventual committed tree.
- Foundation evidence: `foundations-layer-fixed/receipt.json` records successful current-source host build, relocated pack/state/native checks and pixel checks for input-lab and triangle. All source hashes listed in that receipt matched the checkout at reconciliation. Explicit input UI layers and bounded labels repaired the earlier missing controls; triangle's checker covers visible/culled/edge-on states and exact reset pixels. The initial triangle oracle used the wrong replay frame mapping; that checker error was corrected against `capture.ncrs`, without changing the replay to manufacture a pass.
- Clock/RNG evidence: `foundations-state-checked/summary.json` records both checkers exiting 0. Clock headless/native states agree at host ticks 60/120/180, and one/four draws produce identical endpoint marker pixels. RNG seed 4919 repeats equal named state, seed 4920 changes the bounded route, and native generation hashes agree with headless state. Per-example receipts and actual captures are retained. These are UNCOMMITTED local probes, not G5 delivery or human acceptance.
- Remaining foundation checks: new README/control/build/witness documentation is present for 05-08, but final committed-source qualification and delivery remain open. The targeted shared HUD Cargo invocation exited 101 before testing because the support package is outside workspace membership; its failure remains in `hud-label-test.log`. Running the actual dependency-free module via `rustc --edition 2021 --test examples/examples-common/src/hud.rs` passed its bounded UTF-8/overflow test; log: `hud-module-tests.log`. No workspace manifest change was needed. Full source-bound negative/sync checks are being consolidated in `examples/1-getting-started/check_foundations.py`.
- Consolidated foundation qualification: `examples/1-getting-started/check_foundations.py --render` exited 0 for all four lessons; receipt/media: `tmp/zx-examples/20261010-astra/foundations-qualified-preflight/`. Triangle: 41 ticks/9 assertions/9 captures; input: 21/13/8; clock: 181/10/12; RNG: 91/10/6. Each rejects one deliberately false state assertion and executes the scripted controls through 120 input frames of forced rollback with check distance 2. Actual native layouts were inspected; input's README was corrected to describe the implemented radial hard deadzone. All source-listed lesson files match the tested verification export; unrelated dirty/line-ending-different live bindings were not used. This remains a pre-commit receipt, not G5.
- Foundation delivery: task-owned commit `936065db111c1be23b9c1460770e791dc969f7ad` is pushed to `origin/main`; exact remote readback matched. All fifteen committed-export commands exited 0, including format, host build, six draw tests, two quad-layout tests, host Clippy, HUD test, binding checks/compiles and all seven relocated lessons. The final raw-byte identity assertion initially failed on `epu_meta_gen.rs`: Git archive had materialized CRLF while the generator produced the exact LF committed blob. No EPU code was changed. Original failure receipt remains; `foundation-committed-verification/identity-reconciliation.json` verifies the exact blob and every preserved log, with all other exported source files unchanged.
- Current hosted CI: four applicable workflows started for `936065d`: CI `38031282733`, Tool Preview `38031282746`, Documentation `38031282747`, Latest Release `38031282770`. FFI workflow is path-filtered out for this commit; local binding generation/compile still passed. The bounded read-only watcher is `watch-foundation-ci.py`; retain its progress/final JSON and do not report pending workflows as passed. Required CI prevents SHIPPED until resolved.
- The foundation CI watch ended with exit 2 after its bounded wait: Documentation, Latest Release and Tool Preview succeeded; CI `38031282733` was still in progress. `foundation-ci-progress.json` records that observation; it is not a current-pass claim.
- Post-watch exact-SHA readback: CI `38031282733` completed FAILURE, job `114152576066` (Tests + Lint), step `Tests`; the other three applicable workflows passed. Read-only run/job metadata is retained in `foundation-ci-after-watch.json` and `foundation-ci-job-metadata.json`. No job logs or credentials were retrieved. The failed test's cause remains unclassified; this is not permission to waive G5 or retry the denied older log operation.
- Textured-quad local qualification: `texture-qualified/receipt.json` binds 48 replay ticks, 15 passing assertions, deliberate assertion rejection, ten actual captures and 120-frame scripted forced rollback. Its image gate checks four labeled corners/tiles, independent draw filters, green neighbor bleed at tile 1's edge, half-texel exclusion and exact reset. Initial title clipping was repaired and the fresh native title rechecked. `foundations-runner-generalized/receipt.json` proves the shared runner's unchanged default four-project route; the new `--projects` option avoids cloning verification scaffolding. Both receipts are under `tmp/zx-examples/20261010-astra/`. No source is yet shipped in this batch.
- Fresh graphics/foundation qualification: `graphics-six-qualified/receipt.json` records 09 (48 ticks/15 assertions/10 captures), 16 (241/52/25), and all four foundation passes with the final shared runner. Every lesson rejects a false assertion. Forced rollback now reaches each witness's final input, not an unconditional 120-frame cutoff: 16 uses 241 and 07 uses 181; the other four use 120. Flipbook's pure boundary test passes; its 312 recorded snapshot fields matched a separate absolute-time/table oracle. Real image review found crowded frame-duration captions; shortened labels and a corrected stretch atlas boundary are included in the fresh captures. Both graphics pixel gates reject deliberately repeated idle captures. The actual native layout repair was rechecked; no human acceptance was claimed.
- Graphics batch commit `7bc1cc55caede6ac6cfcc804902e4f5680ed00cb`, tree `8cdf3b08094795e41ee043ed11c19635134bda80`, contains exactly 20 task-owned paths. Ordinary push and exact `refs/heads/main` readback matched. `verify-graphics-commit.py` then passed workspace/guest formatting, compiled/runnable playback boundary test, all six graphics/foundation lessons, and all three Rust/C/Zig first cartridges from a fresh exact committed export. Receipt: `graphics-committed-verification/receipt.json` (PASSED); lesson media: `graphics-six-committed/` and `first-cartridges-graphics-committed/`. The unchanged committed host binaries match the verified foundation hashes, no live overlays were used, exported source identity and foreign-file hashes remained unchanged, and final remote readback matched again. CI is watched by `proc_c5d737ce8c72`; no G5/SHIPPED claim before its applicable conclusions are verified. Catalogue source count is now 48; the 90-entry contract is unchanged.
- Other constraints: keep the verified local Zig 0.13.0, Clang 18.1.8 and Rust 1.94.1 routes.
- Pivot local qualification: `pivot-qualified/receipt.json` binds 68 ticks/19 assertions, a false-assertion rejection, fourteen native images and 120 forced-rollback input frames. Pure quarter-turn/relative-child tests pass; 133 snapshot fields match independent Python trigonometry within the declared 0.02-pixel float tolerance. Native probes use a declared three-pixel raster tolerance and cover all eight stops, both UV tiles, fixed center/edge anchors, isolated child turns, wrap and exact reset. Repeated idle captures are rejected. Source and foreign-file preservation checks pass; the export-generated Cargo.lock is retained/hash-bound and ignored by repository policy, not an authored file missing from the live tree. Catalogue: 49 source projects, including nine graphics projects. This receipt alone does not establish delivery or human approval.
- Existing CI boundary: the separate pre-task baseline CI receipt reported 12 EPU GPU failures; preserve that workstream and do not bypass required CI. Advanced proof-first pilots remain unproved.
- Resume rule: read the latest execution request, this checkpoint and exact receipts, reconcile live work/artifacts and continue the unfinished dependency. The current-owner requirement is resolved; do not resurrect the obsolete Sol-only blocker. A denied CI-log operation is not permission to retry it, nor a reason to stop independent authorized work. Never infer SHIPPED from folders, a prior summary, compilation alone or model image review.

## Exact worklist

Keep IDs/paths stable. `existing` means preserve and qualify, not automatically done. `direct`, `guest` and `proof-first` describe the intended implementation route, not proven feasibility. B/M/A mean beginner/medium/advanced. Prerequisites identify learning/code evidence to reuse, not mandatory shared abstractions.

For each entry replace the progress line as work proceeds: PLANNED -> IMPLEMENTING -> VERIFYING -> READY -> SHIPPED, or BLOCKED with exact reason and next independent action. Keep evidence blank until real evidence exists. A READY line should point to the recipe/witness receipt and source identity; a SHIPPED line adds the delivery ref/CI or the linked verified batch. Keep `human: unreviewed` unless an actual review says otherwise.

### Portfolio totals

- Fundamentals: **8** (5 existing + 3 new).
- Graphics primitives: **13** (7 existing + 6 new).
- Inspectors and authoring: **7** (7 existing + 0 new).
- Animation: **10** (6 existing + 4 new).
- Audio: **8** (5 existing + 3 new).
- Assets and publishing: **10** (8 existing + 2 new).
- Small complete games and combinatorial systems: **9** (3 existing + 6 new).
- Advanced rendering and stencil composition: **25** (3 existing + 22 new).

**90 total: 44 existing + 46 new.** Difficulty: 19 beginner, 39 medium, 32 advanced.


### Fundamentals


#### 01. hello-world

- [ ] **01 SHIPPED** — `examples/1-getting-started/hello-world`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** First cartridge: text, rectangles and a moving object.
- **Build/repair:** Use the canonical Rust SDK and init/update/render; make every coordinate visible.
- **Witness:** Build, pack, launch outside the checkout and move the object.
- **Boundary:** No custom framework or allocator required.
- **Progress:** READY (G1-G4); evidence: `examples/1-getting-started/hello-world/check.py`, `movement.ncrs`, `capture.ncrs`; local batch receipt `tmp/zx-examples/20261010-astra/first-cartridges/receipt.json`. 91 ticks/15 assertions, negative control, five native frames, 120-frame rollback sync and cross-language pixel parity passed against the committed-baseline player. Delivery: pending exact committed export/remote/CI; human: unreviewed.


#### 02. hello-world-c

- [ ] **02 SHIPPED** — `examples/1-getting-started/hello-world-c`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The same first cartridge in C, not a second curriculum.
- **Build/repair:** Use the generated header and explicit WASM exports; repair the obsolete manifest.
- **Witness:** A clean C build and packed launch show the same controls as Rust.
- **Boundary:** Manifest repaired; no libc/WASI dependency. Source presence is not build acceptance.
- **Progress:** READY (G1-G4); evidence: shared first-cartridge check/receipt named under 01; same real control, clamp, negative-control, native-frame and sync witnesses passed for the C cartridge. Delivery: pending exact committed export/remote/CI; human: unreviewed.


#### 03. hello-world-zig

- [ ] **03 SHIPPED** — `examples/1-getting-started/hello-world-zig`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The same first cartridge in Zig.
- **Build/repair:** Use version-matched bindings and a freestanding build; repair the obsolete manifest.
- **Witness:** A clean Zig build and packed launch show the same controls as Rust.
- **Boundary:** Zig 0.13.0 route exercised; not a promise of every SDK feature/version.
- **Progress:** READY (G1-G4); evidence: shared first-cartridge check/receipt named under 01; same real witnesses passed for Zig. Native instantiation exposed generated `C::` imports; generator and generated Zig namespace repaired to `env::` with RED/GREEN unit/native evidence, without changing host ABI. Delivery: pending exact committed export/remote/CI; human: unreviewed.


#### 04. save-slots

- [ ] **04 SHIPPED** — `examples/1-getting-started/save-slots`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Controller-owned persistence without misleading disk-success claims.
- **Build/repair:** Save, load and delete slots 0-3; show local ownership and return codes.
- **Witness:** Restart the player, reload the saved counter, and reject invalid payloads.
- **Boundary:** Staging success is not confirmed disk persistence.
- **Progress:** IMPLEMENTING; obsolete slot-4..7 prose removed; staged/error/byte results, bounded versioned payload with legacy four-byte support and memory-only reset added. Payload unit test and 26-tick/19-assertion replay pass. Nine fresh native captures verify the repaired headings and unclipped explanatory text. Non-replay player launched with the same compiled guest under isolated metadata ID `zx-save-witness-ef28614d7a35`; initial live capture shows four empty slots. The first background keyboard action (`z`) hit an approval timeout and DID NOT RUN. Do not retry that input or use another injection route without fresh consent. Real restart/save-file/malformed-on-load proof remains blocked for this lesson only; continue independent curriculum work. Sources/evidence remain uncommitted; human: unreviewed.


#### 05. triangle

- [ ] **05 SHIPPED** — `examples/1-getting-started/triangle`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The smallest legible 3D scene.
- **Build/repair:** Label winding, camera, vertices and model transform; retain one triangle.
- **Witness:** A/B winding and camera changes have predictable visible results.
- **Boundary:** Keep this a tiny example, not a lighting showcase.
- **Progress:** READY (local G1-G4); combined `check_foundations.py --render` passes build/relocation, 41 ticks/9 assertions, a rejected false expectation, nine native views, winding/culling/rotation/reset pixels and 120-frame scripted rollback. Receipt: `foundations-qualified-preflight/receipt.json`; committed-tree/remote/CI G5 pending; human: unreviewed.


#### 06. input-lab

- [ ] **06 SHIPPED** — `examples/1-getting-started/input-lab`
- **Class:** new; B; direct; phase A. **Prerequisites:** 01 `hello-world`.
- **Outcome:** Understand all controller inputs, dead zones and button edges.
- **Build/repair:** Visualize held/pressed/released buttons, both sticks, triggers and local-player slots.
- **Witness:** Replay exact press/hold/release sequences; each edge appears once.
- **Boundary:** Expose controller input, not an invented keyboard/mouse guest API.
- **Progress:** READY (local G1-G4); explicit layers repair the observed panel defect. Combined checker passes build/relocation, 21 ticks/13 assertions, rejected false expectation, eight native captures/all stick markers and four-player 120-frame scripted rollback. All panels/labels fit in actual native output; radial-deadzone docs corrected. Receipt: `foundations-qualified-preflight/receipt.json`; committed-tree/remote/CI G5 pending; human: unreviewed.


#### 07. fixed-clock-lab

- [ ] **07 SHIPPED** — `examples/1-getting-started/fixed-clock-lab`
- **Class:** new; B; direct; phase A. **Prerequisites:** 01 `hello-world`.
- **Outcome:** Teach simulation time versus render frequency.
- **Build/repair:** Advance state only in update; compare tick_count, delta_time and repeated renders.
- **Witness:** The same replay gives the same endpoint when render frequency changes.
- **Boundary:** Render interpolation is optional and must not advance simulation.
- **Progress:** READY (local G1-G4); combined checker passes build/relocation, 181 ticks/10 assertions, rejected false expectation, zero-render/native state at ticks 60/120/180, one/four-draw identical endpoint pixels, twelve native views and 120-frame scripted rollback. README explains frame pre/post alignment and reset scope. Receipt: `foundations-qualified-preflight/receipt.json`; committed-tree/remote/CI G5 pending; human: unreviewed.


#### 08. seeded-random-lab

- [ ] **08 SHIPPED** — `examples/1-getting-started/seeded-random-lab`
- **Class:** new; B; direct; phase A. **Prerequisites:** 07 `fixed-clock-lab`.
- **Outcome:** A repeatable small procedural playground.
- **Build/repair:** Use the host RNG only in init/update; replay with a fixed recorded seed to place objects and generate a bounded route. Explain that guest reset does not expose a host reseed API.
- **Witness:** Two runs of the same recorded seed/input sequence produce equal named guest state; a different replay seed changes the route without exceeding the generation bound.
- **Boundary:** Do not infer cross-platform determinism solely from equal images.
- **Progress:** READY (local G1-G4); combined checker passes build/relocation, 91 ticks/10 assertions, rejected false expectation, repeated seed 4919 equal named state, changed seed 4920 bounded route, native/headless generation agreement, six native captures and 120-frame scripted rollback. Receipt: `foundations-qualified-preflight/receipt.json`; committed-tree/remote/CI G5 pending; human: unreviewed.


### Graphics primitives


#### 09. textured-quad

- [ ] **09 SHIPPED** — `examples/2-graphics/textured-quad`
- **Class:** existing; B; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Texture coordinates, binding and filtering made obvious.
- **Build/repair:** Use an asymmetric checker/label atlas; toggle nearest/linear and UV regions.
- **Witness:** Each corner stays correctly oriented; filtering changes only the intended draw.
- **Boundary:** Do not hide sampling or atlas gutters behind a generic texture wrapper.
- **Progress:** READY locally (G1-G4); numbered asymmetric atlas, per-draw nearest/linear, full/four-tile UV selection, explicit half-texel inset, hold-safe controls and reset implemented. Source-bound state/negative/native/pixel/rollback recipe passes; evidence: `texture-qualified/receipt.json` in the run directory above. Representative native capture: `textured-quad-native-0.png`; delivery: pending; human: unreviewed.


#### 10. procedural-shapes

- [ ] **10 SHIPPED** — `examples/2-graphics/procedural-shapes`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Understand generated geometry, normals and face culling.
- **Build/repair:** Reuse the existing primitive gallery; add front/back/none and wire-like edge aids.
- **Witness:** Every shape has predictable scale, winding and visible lighting response.
- **Boundary:** One catalogue is enough; not one project per primitive.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 11. lighting

- [ ] **11 SHIPPED** — `examples/2-graphics/lighting`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The actual dynamic-light budget and material response.
- **Build/repair:** Animate and label up to four supported lights; isolate each contribution.
- **Witness:** Moving a light changes the correct surfaces; disabled lights contribute nothing.
- **Boundary:** No claim of shadow maps, unlimited lights or physically exact PBR.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 12. billboard

- [ ] **12 SHIPPED** — `examples/2-graphics/billboard`
- **Class:** existing; M; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Compare billboard axes, foliage and deterministic particles.
- **Build/repair:** Keep its four modes and fixed-array particles; actually call draw_billboard_region.
- **Witness:** Sprite-sheet frame regions, camera turns and particle replay behave correctly.
- **Boundary:** Particles already exist here; do not add another basic particle project.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 13. dither-demo

- [ ] **13 SHIPPED** — `examples/2-graphics/dither-demo`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Show what ZX transparency really means.
- **Build/repair:** Show textured/uniform material override draws in A/B/A order. Keep tangent/normal-map setup in mode2-inspector and mesh-submission-lab instead of duplicating a full normal-mapping curriculum.
- **Witness:** Moving views reveal declared coverage and overlap behavior without false blending claims.
- **Boundary:** Dither coverage is not conventional smooth alpha compositing.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 14. material-override

- [ ] **14 SHIPPED** — `examples/2-graphics/material-override`
- **Class:** existing; M; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Prevent material-state leaks between unrelated draws.
- **Build/repair:** Show textured and uniform MRE draws in A/B/A order; include a real tangent-space normal-map swatch.
- **Witness:** The third A matches the first; toggling an override affects only its target.
- **Boundary:** A fixed cartridge render mode is not a per-object shader switch.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 15. epu-multi-reflections

- [ ] **15 SHIPPED** — `examples/2-graphics/epu-multi-reflections`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Select different environment sources before different objects.
- **Build/repair:** Reuse material probes under distinct EPU configs and label source ownership.
- **Witness:** Swapping draw order does not accidentally replace every object's environment.
- **Boundary:** Environment reflection is not a position-correct mirror of scene geometry.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 16. atlas-flipbook

- [ ] **16 SHIPPED** — `examples/2-graphics/atlas-flipbook`
- **Class:** new; B; guest; phase A. **Prerequisites:** 09 `textured-quad`, 07 `fixed-clock-lab`.
- **Outcome:** A practical 2D sprite animation recipe.
- **Build/repair:** Read a small frame table; use draw_sprite_region and update-owned loop/once/ping-pong timing.
- **Witness:** Boundary ticks and replay restart reproduce the same selected frames.
- **Boundary:** This is guest playback metadata, not a newly promised console animation API.
- **Progress:** READY locally (G1-G4); editable procedural four-pose atlas, unequal-duration frame table, loop/once/ping-pong, pause/single-tick/rewind/reset and update-only time ownership implemented. Pure boundary test, 241-tick/52-assertion state replay, negative control, 25 native pose/selection/endpoint views and full 241-input-frame rollback pass; evidence: `graphics-six-qualified/receipt.json` in the run directory above. Delivery: pending exact committed source and CI; human: unreviewed.


#### 17. sprite-pivot-lab

- [ ] **17 SHIPPED** — `examples/2-graphics/sprite-pivot-lab`
- **Class:** new; B; direct; phase A. **Prerequisites:** 09 `textured-quad`.
- **Outcome:** Make draw_sprite_ex useful for weapons, gauges and spinning sprites.
- **Build/repair:** Show origin markers, rotation, subregions and a composed hierarchy using guest coordinates.
- **Witness:** Rotation around center and edge pivots matches marked reference points.
- **Boundary:** A handful of transforms is enough; do not invent a scene graph.
- **Progress:** READY locally (G1-G4); center/edge origins, eight clockwise stops, atlas subregions, marked reference points and explicit two-link composition implemented. Unit, 68-tick/19-assertion replay, negative control, fourteen native images, independent endpoint/coverage probes, reset and forced rollback pass. Evidence: `pivot-qualified/receipt.json` in the run directory above. Delivery: `d2cb997b3119565e68343d2ae8d0fb9a78be8ccb`, exact committed export/relocated native checks and remote main readback pass; required CI `38036826637` failed in the `Tests + Lint` job's `Tests` step, while documentation/release/preview workflows succeeded. Human: unreviewed.


#### 18. ui-layering-lab

- [ ] **18 SHIPPED** — `examples/2-graphics/ui-layering-lab`
- **Class:** new; M; guest; phase C. **Prerequisites:** 49 `font-demo`, 17 `sprite-pivot-lab`.
- **Outcome:** A controller-first menu over mixed 2D/3D content.
- **Build/repair:** Use z_index, explicit passes, text measurement and a simple focus list.
- **Witness:** A/B/A textures, overlapping labels, dialogue and focus never reorder unexpectedly.
- **Boundary:** Layout/focus are guest code; no retained widget framework is implied.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 19. camera-projection-lab

- [ ] **19 SHIPPED** — `examples/2-graphics/camera-projection-lab`
- **Class:** new; M; direct; phase A. **Prerequisites:** 05 `triangle`, 20 `transform-composition-lab`.
- **Outcome:** Perspective, orthographic and off-axis camera recipes.
- **Build/repair:** Use custom view/projection matrices with a measured calibration grid.
- **Witness:** Known world points land at expected screen locations in each projection.
- **Boundary:** Honor column-major matrices and WebGPU depth conventions.
- **Progress:** G3 FAILED, not READY. Pure matrix test, build/pack and 93-tick/23-assertion replay pass, including 138 independent scalar snapshot fields. Real native capture has rings but zero of fifteen cyan 3D markers and no magenta IN witnesses; `camera-probe2/camera-projection-lab-pixels.log` rejects it. Static caller tracing found the shared cause: command execution inferred storage from indexed topology, routing retained non-indexed vertices to the immediate buffer and immediate indexed draws to retained buffers. A two-file internal repair now carries the existing `BufferSource` on both mesh commands. The real focused Rust regression passes all four storage/indexing combinations; `mesh-routing-unit.json` binds source/log hashes and verifies the old CLI/player plus 32 foreign files unchanged. This is CPU evidence only, not a native GREEN. The earlier retained-versus-immediate native diagnostic DID NOT RUN because approval expired; do not retry/rephrase native verification without fresh consent. Live guest remains on the retained route; only the diagnostic export uses immediate drawing. Title/label fixes also await rerender. No public API, SDK or EPU changes. Human: unreviewed.


#### 20. transform-composition-lab

- [ ] **20 SHIPPED** — `examples/2-graphics/transform-composition-lab`
- **Class:** new; B; guest; phase A. **Prerequisites:** 05 `triangle`.
- **Outcome:** Teach composition order and a tiny articulated hierarchy.
- **Build/repair:** Use push_identity and explicit guest matrices to build a two-joint crane.
- **Witness:** Translate-then-rotate differs from rotate-then-translate exactly as labeled.
- **Boundary:** The API's push names do not provide a generic push/pop hierarchy stack.
- **Progress:** READY locally (G1-G4); explicit T*R/R*T, API/matrix comparison, eight angles, relative-child crane and reset implemented. Pure matrix checks, 108-tick/25-assertion replay, negative assertion, 22 native captures, independent scalar box projections/225 snapshot fields and 120-frame forced rollback pass. Evidence: `transform-qualified/receipt.json`; repeated idle captures rejected. The initial image classifier also counted cyan-lit crane surfaces in the cyan bar: panel-local classification fixed this oracle defect without changing guest code or the declared three-pixel tolerance. Delivery: local `7332a7ff6de9942c7888f426a07a9a88e477a86c`; fresh exact-tree format/unit/relocated build/native/negative/rollback checks pass in `transform-committed-verification/receipt.json`. Push approval expired without execution; last confirmed remote was `d2cb997`, so no CI or remote delivery claim for this commit. Human: unreviewed.


#### 21. mesh-submission-lab

- [ ] **21 SHIPPED** — `examples/2-graphics/mesh-submission-lab`
- **Class:** new; A; direct; phase A. **Prerequisites:** 09 `textured-quad`, 20 `transform-composition-lab`.
- **Outcome:** Explain every mesh submission/storage route without folklore.
- **Build/repair:** Render one asymmetric mesh through exactly the six exposed routes: retained float/indexed-float/packed/indexed-packed, and immediate float/indexed-float. Cover format/stride/index validation, UVs/colors/normals and one tangent swatch.
- **Witness:** Each legal route agrees on bounds/topology and expected quantization tolerance; deliberate bad stride/index witnesses are rejected without unsafe reads.
- **Boundary:** There is no guest packed-immediate or mutable-retained-mesh API. Indices are u16; obey current buffer/resource limits.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Inspectors and authoring


#### 22. mode0-inspector

- [ ] **22 SHIPPED** — `examples/3-inspectors/mode0-inspector`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A readable Lambert/flat-shading inspector.
- **Build/repair:** Retain its controls; show normal/no-normal examples with explanatory labels.
- **Witness:** Each control has one obvious effect and reset restores the initial state.
- **Boundary:** Do not call mode 0 universally unlit when normals enable diffuse lighting.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 23. mode1-inspector

- [ ] **23 SHIPPED** — `examples/3-inspectors/mode1-inspector`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Matcap layering with a convincing stylized asset.
- **Build/repair:** Compare multiply/add/HSV matcap combinations in the existing fixed-mode cartridge.
- **Witness:** Layer order and blend choices match the labeled material examples.
- **Boundary:** Matcap add is not general framebuffer additive blending.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 24. mode2-inspector

- [ ] **24 SHIPPED** — `examples/3-inspectors/mode2-inspector`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Metallic/roughness, emissive and normal-map material authoring.
- **Build/repair:** Use real UVs/tangents and source-authored material maps; retain existing controls.
- **Witness:** Normal-map toggles and roughness changes remain legible from multiple views.
- **Boundary:** Label the actual PBR-inspired shading model rather than implying a generic engine.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 25. mode3-inspector

- [ ] **25 SHIPPED** — `examples/3-inspectors/mode3-inspector`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Specular/shininess authoring with legacy-style materials.
- **Build/repair:** Retain the inspector and show clear differences in exponent and specular color.
- **Witness:** A controlled moving light produces the intended highlight width and tint.
- **Boundary:** No runtime mode switching or external shader injection is assumed.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 26. epu-inspector

- [ ] **26 SHIPPED** — `examples/3-inspectors/epu-inspector`
- **Class:** existing; A; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Packed instruction literacy plus semantic editor/export workflow.
- **Build/repair:** Keep the eight-layer explorer; connect native F4 LOCK/edit/export to a minimal guest.
- **Witness:** An exported program boots with the exact layers, and no hidden host clock.
- **Boundary:** An editor export is not proof of attractive environment art.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 27. epu-showcase

- [ ] **27 SHIPPED** — `examples/3-inspectors/epu-showcase`
- **Class:** existing; A; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Convincing environment recipes, distinct from opcode coverage.
- **Build/repair:** Retain the composition/capability suites and guest-owned animation; label provisional scenes honestly.
- **Witness:** Check fixed-camera world motion, frozen-world look-around and material response separately.
- **Boundary:** Current structural/artistic work is provisional; preserve accepted scenes and human verdicts.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 28. debug-demo

- [ ] **28 SHIPPED** — `examples/3-inspectors/debug-demo`
- **Class:** existing; M; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Use the debug inspector to understand a real stateful scene.
- **Build/repair:** Add representative watch/edit/range/vector values to the existing project.
- **Witness:** Edits affect the correct variables and reset/replay semantics are documented.
- **Boundary:** Debug editing can alter determinism; do not present it as normal multiplayer input.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Animation


#### 29. skinned-mesh

- [ ] **29 SHIPPED** — `examples/4-animation/skinned-mesh`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The smallest skeletal deformation example.
- **Build/repair:** Show bind pose, explicit bone matrices and clear colored bone influence regions.
- **Witness:** Known bone transforms bend only the expected vertices.
- **Boundary:** Document raw skinning versus skeleton-bound matrix contracts.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 30. animation-demo

- [ ] **30 SHIPPED** — `examples/4-animation/animation-demo`
- **Class:** existing; M; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Stamped versus interpolated keyframe sampling.
- **Build/repair:** Keep existing keyframe_bind and keyframe_read paths with side-by-side playback.
- **Witness:** Exact frames match; between-frame interpolation is smooth and replay-owned.
- **Boundary:** Interpolation already exists; it is not evidence of a full animation graph.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 31. ik-demo

- [ ] **31 SHIPPED** — `examples/4-animation/ik-demo`
- **Class:** existing; M; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A legible reachable/unreachable IK target.
- **Build/repair:** Retain the solver; show target, joint limits and pole direction; add a manifest.
- **Witness:** Reachable endpoints agree; unreachable targets remain finite and stable.
- **Boundary:** IK is guest math, not an engine controller.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 32. multi-skinned-procedural

- [ ] **32 SHIPPED** — `examples/4-animation/multi-skinned-procedural`
- **Class:** existing; M; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Independent procedural poses on shared mesh resources.
- **Build/repair:** Retain its multiple skeletons; add a manifest and explicit per-character binds.
- **Witness:** Characters stay independent when their draw order is swapped.
- **Boundary:** Do not assume skin state automatically follows the mesh handle.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 33. multi-skinned-rom

- [ ] **33 SHIPPED** — `examples/4-animation/multi-skinned-rom`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Independent imported characters/clips in one scene.
- **Build/repair:** Reuse the existing generated assets and show different phases for each actor.
- **Witness:** ROM IDs, bone counts and inverse binds match; swapping actors does not leak poses.
- **Boundary:** Import limitations and resource ceilings must be documented.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 34. skeleton-stress-test

- [ ] **34 SHIPPED** — `examples/4-animation/skeleton-stress-test`
- **Class:** existing; A; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** An honest bone/actor cost envelope, not a pretty FPS counter.
- **Build/repair:** Retain the stress scene; add a manifest, fixed modes and separate CPU/GPU labels.
- **Witness:** Measure repeatable scaling at named actor/bone counts and catch buffer ceilings.
- **Boundary:** Exported animation and runtime skeleton ceilings need not be identical.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 35. locomotion-transitions

- [ ] **35 SHIPPED** — `examples/4-animation/locomotion-transitions`
- **Class:** new; M; guest; phase C. **Prerequisites:** 30 `animation-demo`, 52 `glb-inline`.
- **Outcome:** Idle/walk/run with explicit clip-to-clip transitions.
- **Build/repair:** Build on animation-demo sampling; blend two imported clips from update-owned state.
- **Witness:** Start/stop/reverse transitions and replay endpoints remain continuous and correct.
- **Boundary:** No generalized blend tree; two clips and a tiny state machine suffice.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 36. bone-attachments

- [ ] **36 SHIPPED** — `examples/4-animation/bone-attachments`
- **Class:** new; M; guest; phase C. **Prerequisites:** 29 `skinned-mesh`, 30 `animation-demo`.
- **Outcome:** A held weapon, helmet and muzzle marker that follow a rig.
- **Build/repair:** Reuse sampled model-space bones and explicit attachment offsets.
- **Witness:** Attachments stay aligned through rotation, animation, scaling and bind-pose reset.
- **Boundary:** Guests own hierarchy math; there is no promised attachment API.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 37. clip-plus-ik

- [ ] **37 SHIPPED** — `examples/4-animation/clip-plus-ik`
- **Class:** new; A; guest; phase C. **Prerequisites:** 31 `ik-demo`, 35 `locomotion-transitions`.
- **Outcome:** Combine authored motion with procedural aiming or foot placement.
- **Build/repair:** Start with a sampled clip, modify only an arm aiming chain using guest math, then submit final bones. Ground/foot IK is an optional later extension, not required scope.
- **Witness:** The unmodified limbs retain the clip while the aiming arm tracks a moving/reachable target and remains finite at unreachable targets.
- **Boundary:** Use a known floor/target model; no general terrain IK framework.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 38. animation-event-timeline

- [ ] **38 SHIPPED** — `examples/4-animation/animation-event-timeline`
- **Class:** new; M; guest; phase C. **Prerequisites:** 30 `animation-demo`, 07 `fixed-clock-lab`.
- **Outcome:** A compact cutscene with camera motion, gestures and sound cues.
- **Build/repair:** Sequence a short camera move, gesture and sound cue from deterministic update ticks; use sampled keyframes and explicit transforms. No general cutscene graph.
- **Witness:** An event fires once per intended tick and a replay reproduces camera/actor state.
- **Boundary:** No hidden callback/event API; audio side effects need the dedicated rollback gate.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Audio


#### 39. audio-demo

- [ ] **39 SHIPPED** — `examples/5-audio/audio-demo`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Audible samples, dedicated channels, volume, pan and loop control.
- **Build/repair:** Retain the existing tone/sample controls and clearly label channel ownership.
- **Witness:** Listen to start/stop/loop/pan behavior and verify no unintended channel stealing.
- **Boundary:** channel_set adjusts volume/pan, not a nonexistent pitch control.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 40. tracker-demo-xm

- [ ] **40 SHIPPED** — `examples/5-audio/tracker-demo-xm`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The packed embedded-sample XM workflow.
- **Build/repair:** Retain actual import/pack/play, position/seek/tempo diagnostics and source track.
- **Witness:** Known song sections, loops and seeks are audible and survive the declared replay.
- **Boundary:** Do not infer every XM effect is supported from one song.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 41. tracker-demo-xm-split

- [ ] **41 SHIPPED** — `examples/5-audio/tracker-demo-xm-split`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** XM patterns with explicit reusable sample assets.
- **Build/repair:** Retain split names/IDs and compare with the equivalent embedded sample version.
- **Witness:** The same passage agrees and missing/colliding sample IDs fail clearly.
- **Boundary:** This is a distinct asset workflow, not a duplicate music player skin.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 42. tracker-demo-it

- [ ] **42 SHIPPED** — `examples/5-audio/tracker-demo-it`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The packed embedded-sample IT workflow.
- **Build/repair:** Retain actual IT conversion/playback and explain format-specific behaviors.
- **Witness:** Known instruments, effect fixtures, loop and seek transitions are audible.
- **Boundary:** Do not claim universal IT parity without a named finite corpus.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 43. tracker-demo-it-split

- [ ] **43 SHIPPED** — `examples/5-audio/tracker-demo-it-split`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** IT patterns with separately managed sample assets.
- **Build/repair:** Retain split import/pack/play and side-by-side source mapping diagnostics.
- **Witness:** Missing samples are visible errors; equivalent embedded/split passages agree.
- **Boundary:** Sharing sample assets is not simultaneous multi-track music mixing.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 44. world-audio-panning

- [ ] **44 SHIPPED** — `examples/5-audio/world-audio-panning`
- **Class:** new; M; guest; phase C. **Prerequisites:** 39 `audio-demo`.
- **Outcome:** A convincing positional-audio recipe for a small game.
- **Build/repair:** Compute stereo pan and bounded distance attenuation from a deterministic in-game listener/emitter. Show left/right reversal under listener turning without claiming a platform spatializer.
- **Witness:** Turning the camera and swapping left/right emitters produce the correct audible result.
- **Boundary:** Local single-listener recipe first. Do not use client-local camera-dependent FFI audio state in a netplay claim without checking rollback/checksum semantics; no HRTF/reverb/pitch API.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 45. adaptive-tracker-score

- [ ] **45 SHIPPED** — `examples/5-audio/adaptive-tracker-score`
- **Class:** new; A; guest; phase C. **Prerequisites:** 40 `tracker-demo-xm`.
- **Outcome:** Explore/combat/outro music driven by a small game state.
- **Build/repair:** Use update-owned music state, music_jump and explicit position/order/row checks for authored explore/combat/outro transitions. Establish query timing from the current audio path; do not promise sample-accurate transitions.
- **Witness:** Repeated transition requests do not constantly restart; each intended section/cue is audibly reached, including missing-boundary fallback and reset.
- **Boundary:** One music source is active; do not promise native stem mixing or crossfades.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 46. rollback-audio-lab

- [ ] **46 SHIPPED** — `examples/5-audio/rollback-audio-lab`
- **Class:** new; A; proof-first; phase C. **Prerequisites:** 39 `audio-demo`, 40 `tracker-demo-xm`, 59 `netplay-demo`.
- **Outcome:** Prove audio behavior under prediction, replay and state changes.
- **Build/repair:** Add audible event IDs/loops to a minimal real netplay scenario; use existing replay/sync-test tooling.
- **Witness:** Compare offline and actual induced rollback for duplicate/stale attacks, stuck loops, seeks and discontinuities. Record an audible witness and a state witness; a sync-test label alone is insufficient.
- **Boundary:** An audible correctness gate, not an assertion that current audio already satisfies every case.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Assets and publishing


#### 47. asset-test

- [ ] **47 SHIPPED** — `examples/6-assets/asset-test`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Understand packed versus source assets and expected failures.
- **Build/repair:** Retain packed texture/mesh tests and show malformed/unsupported format diagnostics.
- **Witness:** A relocated final cartridge renders every expected asset; bad input fails visibly.
- **Boundary:** A parser success is not appearance or compatibility acceptance.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 48. datapack-demo

- [ ] **48 SHIPPED** — `examples/6-assets/datapack-demo`
- **Class:** existing; B; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A complete texture/mesh/sound cartridge with stable IDs.
- **Build/repair:** Keep the existing pack workflow and make every loaded resource observable.
- **Witness:** Build from clean source assets, relocate the cartridge and verify draw/play.
- **Boundary:** Do not rely on the development checkout's asset paths at runtime.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 49. font-demo

- [ ] **49 SHIPPED** — `examples/6-assets/font-demo`
- **Class:** existing; M; direct; phase A. **Prerequisites:** 09 `textured-quad`.
- **Outcome:** A real custom-font/menu example instead of an atlas preview.
- **Build/repair:** Turn the atlas preview into fixed/variable-width custom text via rom_texture, load_font/load_font_ex, font_bind and text_width. Implement a tiny guest wrap/alignment/focus example, not an assumed native layout API.
- **Witness:** Glyphs, measured alignment, missing glyphs, wrapping and focus are legible and correct.
- **Boundary:** rom_font returns an atlas; the current manifest has no fonts section. Do not imply a complete BMFont pipeline.
- **Progress:** READY locally (G1-G4) from exact committed source `bf18f14132fb0dc589aa0d7fec5ea75e9c11b7aa`. `font-committed-local-verification.json` binds fresh release/debug build and headless replay, 88 ticks/19 assertions, 190 independent snapshot fields, nineteen native captures with 237690 exact glyph pixels, false-assertion and repeated-idle rejection, 120-input-frame rollback, reproducible asset regeneration, warning-free Clippy, unchanged full source export/pinned tools and preserved worktree. The native oracle caught a real separator-at-boundary wrap defect; the focused unit failed before and passed after the shared guest-helper repair, without relaxing the pixel contract. ROM-backed fixed/variable text, all three alignments, both wrap widths, missing-glyph fallback, focus wrap/held input and reset are implemented. Reuses repository-authored bitmap source; no new API/dependency. Delivery: local commit only, remote/CI unverified; human: unreviewed.


#### 50. level-loader

- [ ] **50 SHIPPED** — `examples/6-assets/level-loader`
- **Class:** existing; M; guest; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A useful textured map, not only colored tile rectangles.
- **Build/repair:** Retain rom_data and visible-cell iteration; add tileset regions, layers and a small collision visualization.
- **Witness:** Authored cells, camera bounds and edge tiles agree with the source data.
- **Boundary:** A guest tilemap recipe is not a native tilemap asset/playback API.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 51. gltf-test

- [ ] **51 SHIPPED** — `examples/6-assets/gltf-test`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A truthful GLTF import validation scene.
- **Build/repair:** Show axes, scales, mesh primitive/material selection, skeleton and keyframe IDs.
- **Witness:** Compare known source bounds, UVs, transforms and bone counts to the packed result.
- **Boundary:** Document first-mesh/primitive and other real importer limitations instead of promising entire-scene import.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 52. glb-inline

- [ ] **52 SHIPPED** — `examples/6-assets/glb-inline`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Direct GLB references with several named animation clips.
- **Build/repair:** Retain wildcard/name/prefix selection and show source-to-ROM identity.
- **Witness:** Every intended clip can be selected and wildcard IDs do not collide.
- **Boundary:** Do not silently accept the newest unrelated WASM or first unintended clip.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 53. glb-rigid

- [ ] **53 SHIPPED** — `examples/6-assets/glb-rigid`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Rigid node animation without pretending it is skeletal deformation.
- **Build/repair:** Retain the imported moving prop and label node/transform playback.
- **Witness:** Source transforms and animation phase match the cartridge's moving prop.
- **Boundary:** This overlaps asset import, but is a distinct rigid-animation contract.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 54. epu-textures-demo

- [ ] **54 SHIPPED** — `examples/6-assets/epu-textures-demo`
- **Class:** existing; M; direct; phase C. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Artist-authored cube faces in the immediate EPU route.
- **Build/repair:** Keep six rom_texture handles and epu_textures; show orientation, seams and material probes.
- **Witness:** An asymmetric axis scene remains correctly oriented through a full look-around.
- **Boundary:** Do not require epu_asset; its intent is unconfirmed by the user.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 55. asset-compression-budget

- [ ] **55 SHIPPED** — `examples/6-assets/asset-compression-budget`
- **Class:** new; A; direct; phase C. **Prerequisites:** 47 `asset-test`, 48 `datapack-demo`.
- **Outcome:** An honest comparison of cartridge size, decoded textures and appearance.
- **Build/repair:** Build matched RGBA8/BC7 variants with the current packer; inspect atlas/alpha/material swatches.
- **Witness:** Report measured cartridge/texture costs and decoded-image differences, with controlled inputs.
- **Boundary:** PNG bytes are not VRAM; no unsupported compression, mip or timing claim.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 56. source-to-cartridge-kit

- [ ] **56 SHIPPED** — `examples/6-assets/source-to-cartridge-kit`
- **Class:** new; M; guest; phase C. **Prerequisites:** 48 `datapack-demo`.
- **Outcome:** A small reproducible authoring pipeline others can actually reuse.
- **Build/repair:** Regenerate one small mesh, atlas and sound from editable, licensed source using existing repository tools; pack and launch from a relocated cartridge. Extend existing generators only as needed.
- **Witness:** A fresh checkout can regenerate the named assets and play the final cartridge.
- **Boundary:** No mandatory external generative model, private game source, sibling checkout, new asset DSL or binary-only input. Essential source assets are not disposable QA output.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Small complete games and combinatorial systems


#### 57. paddle

- [ ] **57 SHIPPED** — `examples/7-games/paddle`
- **Class:** existing; B; guest; phase D. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The short complete-game learning path.
- **Build/repair:** Retain movement, ball rules, AI, audio, scoring and two-player play.
- **Witness:** A real round finishes and replay/local/netplay routes are documented.
- **Boundary:** Already a complete game; no need for a second Pong.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 58. platformer

- [ ] **58 SHIPPED** — `examples/7-games/platformer`
- **Class:** existing; M; guest; phase D. **Prerequisites:** none beyond Phase 0.
- **Outcome:** A bounded 2.5D platformer with camera, collision and HUD.
- **Build/repair:** Retain the existing rules; make sprite presentation, restart and win/loss legible.
- **Witness:** Jumps, edges, collisions and reset are exercised through real controls.
- **Boundary:** Do not count presentation alone as good game feel.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 59. netplay-demo

- [ ] **59 SHIPPED** — `examples/7-games/netplay-demo`
- **Class:** existing; A; guest; phase D. **Prerequisites:** none beyond Phase 0.
- **Outcome:** The existing 1-4-player arena plus actual network proof.
- **Build/repair:** Retain lobby, deterministic projectiles and local-only stats; add two-instance instructions/replays.
- **Witness:** Two real clients finish a round with agreed state; sync tests and save ownership are separately checked.
- **Boundary:** A local player_count label is not proof of online connection or live join support.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 60. third-person-room

- [ ] **60 SHIPPED** — `examples/7-games/third-person-room`
- **Class:** new; M; guest; phase D. **Prerequisites:** 35 `locomotion-transitions`, 19 `camera-projection-lab`, 50 `level-loader`.
- **Outcome:** A small playable 3D room combining camera, locomotion and interactions.
- **Build/repair:** One controlled 3D room: imported actor, idle/walk transition, yaw/follow camera, simple guest XZ wall collision, one collectible/door objective, HUD and restart. No general physics.
- **Witness:** Walk to an objective, collide with the walls, animate and reset consistently.
- **Boundary:** One room and one actor, not a character-controller framework.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 61. hoverkart-split-screen

- [ ] **61 SHIPPED** — `examples/7-games/hoverkart-split-screen`
- **Class:** new; A; guest; phase D. **Prerequisites:** 67 `viewport-test`, 68 `rear-mirror`, 06 `input-lab`.
- **Outcome:** A short 2-4-player driving loop that earns the split-screen feature.
- **Build/repair:** One short XZ-plane track with two/four local controllers, checkpoint/lap rules, simple guest collision, viewport-relative HUDs and a rear-view inset. Reuse bumpbox only after a standalone dependency/license/target check.
- **Witness:** Each viewport/HUD follows its assigned player; an actual lap and deterministic replay complete. Online racing is not a requirement of this local split-screen lesson.
- **Boundary:** Not full 3D vehicle physics; local split-screen and real netplay are separate acceptance claims.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 62. seeded-dungeon

- [ ] **62 SHIPPED** — `examples/7-games/seeded-dungeon`
- **Class:** new; A; guest; phase D. **Prerequisites:** 08 `seeded-random-lab`, 50 `level-loader`.
- **Outcome:** Generation plus traversal and simple encounters in one tiny game.
- **Build/repair:** A small tile dungeon with a bounded generation algorithm, connected start/exit, one encounter and restart. Evaluate tessera reuse first; pin a publicly obtainable no_std dependency only if it fits the current limits.
- **Witness:** The same seed/replay builds the same connected level; failed generation has a bounded fallback.
- **Boundary:** No compulsory new library integration if a tiny existing repo algorithm suffices. Fixed generation bound and deterministic connected fallback; no private path dependency.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 63. patrol-tactics-puzzle

- [ ] **63 SHIPPED** — `examples/7-games/patrol-tactics-puzzle`
- **Class:** new; A; guest; phase D. **Prerequisites:** 50 `level-loader`, 08 `seeded-random-lab`.
- **Outcome:** Pathfinding and decision-making interacting with game rules.
- **Build/repair:** One grid puzzle with a patrol, routefinding and a decoy/blocked route. Evaluate existing grid/AI code or suitable public libraries, but use only the minimum required systems.
- **Witness:** A patrol reacts to blockage/decoys and repeatable turns reach win/fail states.
- **Boundary:** A bounded route search plus a tiny state/decision routine is acceptable. No general ECS, navmesh, AI editor or mandatory pair of libraries.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 64. rope-bridge-contraption

- [ ] **64 SHIPPED** — `examples/7-games/rope-bridge-contraption`
- **Class:** new; A; guest; phase D. **Prerequisites:** 07 `fixed-clock-lab`.
- **Outcome:** Deterministic constraints plus visible interaction.
- **Build/repair:** One bounded rope/bridge manipulated by the player with fixed-step constraints and explicit iteration counts. Evaluate softy reuse first; prove wasm/no_std/determinism/resource fit or reuse a small existing implementation.
- **Witness:** Pull/release/reset stays finite and the same replay produces the same configuration.
- **Boundary:** No blanket cross-platform bitwise/online physics claim; require the named replay/sync gate and keep the system finite.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 65. rhythm-action

- [ ] **65 SHIPPED** — `examples/7-games/rhythm-action`
- **Class:** new; A; guest; phase D. **Prerequisites:** 40 `tracker-demo-xm`, 07 `fixed-clock-lab`.
- **Outcome:** Music, input windows and visual cues in a tiny playable challenge.
- **Build/repair:** One authored short rhythm chart, visible note/input windows, scoring, end/retry, and packed tracker playback. Authoritative scoring uses simulation ticks; music position is a diagnostic.
- **Witness:** Known early/on-time/late inputs score correctly; expose measured audiovisual offset.
- **Boundary:** Expose the measured audio/display offset and keep it configurable. Calibration is not sample-accurate synchronization or proof of competitive online rhythm play.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


### Advanced rendering and stencil composition


#### 66. stencil-demo

- [ ] **66 SHIPPED** — `examples/8-advanced/stencil-demo`
- **Class:** existing; M; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Keep the current four-mode introduction as the stencil entry point.
- **Build/repair:** Retain inside-circle, inverted mask, diagonal split and alternate-camera portal; visualize pass order.
- **Witness:** Each mask and depth-clear toggle produces the intended region with no UI leakage.
- **Boundary:** Current coverage is four demonstrations, not a complete advanced stencil curriculum.
- **Progress:** READY locally (G1-G4); four modes, pass-order labels, mask ink, hold-edge controls, pause/rotation/reset and meaningful 3D portal depth-clear counterexample implemented. Fresh exact-export qualification at local `e013bf436bb4dc23ea73eb57cf375bdac434a69a` passes release/debug pack, Clippy `-D warnings`, scalar state/tint test, 103-frame/17-assertion replay, 204 typed snapshot fields, false-assertion rejection, 17 native images, 2,124,578 mask classifications, 37,363 depth-change pixels confined to the aperture, idle-negative and 120-frame forced rollback. Evidence: `stencil-committed-local-verification.json`, `stencil-committed-qualified/receipt.json`. Initial closure-typing/static-reference compiler failures and invalid replay `l1` alias remain in their logs; canonical `l` is used without changing the console. A source-bound guard catches the repaired maximum-tick HUD overflow. Exact committed review image `zx-curriculum-66-d8a9f31dcf4c` passed durable queue/media/LAN/browser readback (`stencil-review-submission.json`). Delivery: local commit only; no push or CI claimed. Human: unreviewed.


#### 67. viewport-test

- [ ] **67 SHIPPED** — `examples/8-advanced/viewport-test`
- **Class:** existing; M; direct; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Explain per-viewport cameras, HUD coordinates and pass state.
- **Build/repair:** Retain 2P/4P layouts; draw asymmetric labels and shared geometry in every viewport.
- **Witness:** No content, masks or UI spill into a neighboring viewport.
- **Boundary:** A depth clear affects the attachment, not just the active viewport.
- **Progress:** READY locally (G1-G4); independent four-slot cameras, 2P/4P layouts, asymmetric geometry/labels, clipped masks/ink, pause and reset. Baseline native RED exposed doubled viewport mapping; the bounded render-pass fix selects fullscreen GPU mapping for already-absolute screen quads, retains the requested scissor and caches the mesh/quad route. Exact local commit `b0c83a823f77a0059f97452408f615ff8d1d9000` passed fresh host build, six draw-FFI tests, host/guest Clippy `-D warnings`, state test, release/debug pack, 83 replay frames/18 assertions, 324 typed fields, 18 native images, 6,185,764 viewport/mask classifications, negative controls and 120-frame four-slot rollback. Evidence: `viewport-committed-host-verification.json`, `viewport-committed-local-verification.json`, `viewport-committed-qualified/receipt.json`; all 36 fullscreen font/stencil regression frames are byte-identical (`viewport-committed-fullscreen-regression/receipt.json`). Mesh-storage repair excluded; old pinned tools restored. Exact queued media `zx-curriculum-67-dac49b4c0201` passed LAN hash and read-only app card decoding; no vote (`viewport-review-submission.json`). Delivery: local commit only; remote/CI unresolved. Human: unreviewed.


#### 68. rear-mirror

- [ ] **68 SHIPPED** — `examples/8-advanced/rear-mirror`
- **Class:** existing; M; guest; phase A. **Prerequisites:** none beyond Phase 0.
- **Outcome:** Keep the existing rear-view inset as a different camera use case.
- **Build/repair:** Reuse its second viewport and explicit camera state; show forward/rear orientation.
- **Witness:** An asymmetric track/room proves the inset is rear-facing and isolated.
- **Boundary:** This is a viewport inset, not a reflection or render-to-texture monitor.
- **Progress:** PAUSED before verification: source, README, pure state test, 18 capture cases and the independent state/projection/isolation checker are authored but unqualified. The combined scoped `rustfmt` and frozen-oracle preflight DID NOT RUN because approval expired; do not retry/rephrase either outcome without fresh consent. No build/native/rollback success is claimed. Source inspection also identified that the planned host Cargo unit route must isolate the WASM-only examples-common dependency before use; no unrelated helper changes are authorized by that observation. Ownership baseline: `mirror-ownership.json`. Human: unreviewed.


#### 69. stencil-boolean-masks

- [ ] **69 SHIPPED** — `examples/8-advanced/stencil-boolean-masks`
- **Class:** new; M; guest; phase B. **Prerequisites:** 66 `stencil-demo`.
- **Outcome:** Combine masks: union, intersection, subtraction and parity.
- **Build/repair:** Use explicit full-pass REPLACE/ZERO/INVERT/count operations to demonstrate region union/intersection/subtraction/parity. Build masks before final color or prove a sequence that preserves visible color.
- **Witness:** Overlapping shapes match independent expected region masks, including holes.
- **Boundary:** Stencil comparison uses the same reference as replacement; no guest read/write bit masks. Conditional stencil operations can write color. Do not hide these restrictions behind assumed GL helpers.
- **Progress:** READY locally (G1-G4); local `4e3a6b032358d1d6a49383fb961fc62e4907866f` passes fresh exact-export release/debug pack, state test/Clippy, 98 replay frames/26 assertions, 156 typed pre/post fields, 26 native captures, 7,000,032 interior classifications, 188,448 border pixels, 25,600 hole pixels, false-assertion/idle-negative controls and 120-frame forced rollback. Evidence: `boolean-committed-local-verification.json`, `boolean-committed-qualified/receipt.json`. All 24 operation/layout/order combinations prove unions, intersections, subtraction, parity, commutativity and holes with no pixel-tolerance relaxation. Initial manifest/compound-assertion failures and the clipped-footer repair remain recorded; the corrected footer has a source-bound width guard and a fresh decoded native image. Exact review item `zx-curriculum-69-e0a5c48870e1` passed durable/LAN/app preview readback with no vote (`boolean-review-submission.json`). Delivery: local commit only; remote/CI unresolved. Human: unreviewed.


#### 70. stencil-region-stack

- [ ] **70 SHIPPED** — `examples/8-advanced/stencil-region-stack`
- **Class:** new; A; guest; phase B. **Prerequisites:** 69 `stencil-boolean-masks`.
- **Outcome:** Budget and reuse nested stencil regions without leaks.
- **Build/repair:** Use a fixed two/three-level region stack with explicit full-reference/count allocation, an eight-bit capacity ceiling, and rasterized ZERO cleanup. Explain how repaint order avoids conditional-mask color pollution.
- **Witness:** Nested regions unwind correctly across multiple viewports and consecutive frames.
- **Boundary:** 8-bit full masks; no direct per-bit read/write control or independent stencil clear call.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 71. texture-shaped-stencil

- [ ] **71 SHIPPED** — `examples/8-advanced/texture-shaped-stencil`
- **Class:** new; M; direct; phase B. **Prerequisites:** 66 `stencil-demo`, 09 `textured-quad`.
- **Outcome:** Stencil silhouettes from textured sprites and foliage.
- **Build/repair:** Write a patterned alpha/cutout sprite into stencil, then cover with different content.
- **Witness:** Transparent holes and dithered edges affect the mask exactly as the fragment path dictates.
- **Boundary:** Texture discard/dither participates; this is not automatic high-quality antialiasing.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 72. selection-outline

- [ ] **72 SHIPPED** — `examples/8-advanced/selection-outline`
- **Class:** new; M; guest; phase B. **Prerequisites:** 66 `stencil-demo`, 20 `transform-composition-lab`.
- **Outcome:** A useful selected-object outline, not a floating duplicate mesh.
- **Build/repair:** Mark the original silhouette, draw a expanded authored hull under NOT_EQUAL, and preserve depth intent.
- **Witness:** Foreground occluders and hollow/concave examples demonstrate where the outline should appear.
- **Boundary:** A scaled hull is not a constant-pixel-width edge detector; use suitable assets and disclose thickness variation.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 73. occluded-xray

- [ ] **73 SHIPPED** — `examples/8-advanced/occluded-xray`
- **Class:** new; A; guest; phase B. **Prerequisites:** 66 `stencil-demo`.
- **Outcome:** Highlight only hidden portions of an actor or objective.
- **Build/repair:** Combine depth GREATER, silhouette masks and a dithered/flat highlight pass.
- **Witness:** An actor moving behind a wall reveals only the occluded portions; visible surfaces stay normal.
- **Boundary:** Not a depth-texture postprocess; self-overlap and equal-depth behavior need explicit witnesses.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 74. blob-shadows

- [ ] **74 SHIPPED** — `examples/8-advanced/blob-shadows`
- **Class:** new; M; guest; phase B. **Prerequisites:** 09 `textured-quad`, 20 `transform-composition-lab`.
- **Outcome:** The practical cheap shadow recipe users asked for.
- **Build/repair:** Use a reusable preloaded radial-alpha texture on a ground disk below a moving/jumping actor. Draw in a defined depth-tested/depth-write-off pass with a small receiver-normal geometry offset and explicit dither state.
- **Witness:** The blob stays attached, behaves at floor edges and never covers foreground actors incorrectly.
- **Boundary:** The vertex-color route is RGB, not a per-vertex alpha gradient. A textured blob supplies radial coverage. Known receiver plane, dithered coverage, no multiply blend or native depth bias.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 75. receiver-clipped-shadows

- [ ] **75 SHIPPED** — `examples/8-advanced/receiver-clipped-shadows`
- **Class:** new; A; guest; phase B. **Prerequisites:** 74 `blob-shadows`, 66 `stencil-demo`.
- **Outcome:** Make cheap shadows usable around gaps, platforms and multiple actors.
- **Build/repair:** Render opaque depth, then mark only visible known receiver geometry with depth EQUAL or a proved equivalent full-pass configuration. Clip blob/silhouette coverage to that mask and define an idempotent overlap policy.
- **Witness:** Shadows stop at receiver boundaries, do not climb props and overlap according to the declared policy.
- **Boundary:** The convenience stencil-write pass ignores depth and is not sufficient for receiver visibility. No runtime depth readback, arbitrary projection or hidden smooth blending.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 76. planar-silhouette-shadows

- [ ] **76 SHIPPED** — `examples/8-advanced/planar-silhouette-shadows`
- **Class:** new; A; guest; phase B. **Prerequisites:** 75 `receiver-clipped-shadows`, 21 `mesh-submission-lab`.
- **Outcome:** A caster-shaped shadow using projection math rather than a canned blob.
- **Build/repair:** Project CPU-known caster vertices onto a plane in guest math, explicitly perform any homogeneous divide, and submit the bounded result with immediate triangles. Stencil-mask known receivers, avoid zero denominators and use a declared geometry offset.
- **Witness:** Moving the light changes the projected silhouette; near-parallel light is handled safely.
- **Boundary:** Do not assume arbitrary projective model transforms or runtime load_mesh calls. One flat receiver, bounded geometry, no native depth bias or shadow mapping.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 77. stencil-shadow-volumes

- [ ] **77 SHIPPED** — `examples/8-advanced/stencil-shadow-volumes`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 66 `stencil-demo`, 21 `mesh-submission-lab`.
- **Outcome:** A hardcore single-light shadow-volume study, not an unsupported engine promise.
- **Build/repair:** Single closed low-poly caster, finite light/extrusion, explicit caps and CPU-generated immediate volume triangles. Separate front/back culled wrap-count passes; visualise the stencil result with a deliberately unlit cheap shadow-overlay interpretation.
- **Witness:** First prove each pass against a simple expected region witness. Then exercise camera outside/inside, near/far boundaries, reversed winding and intersecting casters. Required in-domain cases must render correctly, not merely explain failure.
- **Boundary:** Same face state per pass; finite-volume and clipping assumptions must be explicit. This teaches shadow membership/cheap dark coverage, not correct separation of arbitrary direct/ambient lights.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 78. planar-mirror

- [ ] **78 SHIPPED** — `examples/8-advanced/planar-mirror`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 66 `stencil-demo`, 19 `camera-projection-lab`.
- **Outcome:** A genuine scene reflection rendered through a stencil aperture.
- **Build/repair:** One flat mirror and bounded room: stencil the visible mirror against scene depth, reflect camera/geometry, correct winding/culling, and prove plane clipping. Rebuild needed depth after global clears before later normal-scene/HUD draws.
- **Witness:** An asymmetric scene and foreground occluder verify reflection, handedness and clipping.
- **Boundary:** One bounce, direct re-render, no RTT. The mirror plane cannot be entered; crossing is clamped/documented. CPU clipping or a ZX/WebGPU-correct oblique projection must be proved.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 79. linked-room-portal

- [ ] **79 SHIPPED** — `examples/8-advanced/linked-room-portal`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 66 `stencil-demo`, 19 `camera-projection-lab`.
- **Outcome:** A portal whose camera mapping actually follows the viewer.
- **Build/repair:** Two fixed door frames with viewer-relative camera mapping, receiver/occluder masking and proved destination-plane clipping. Treat as a visual portal example; display correct parallax without requiring game-entity teleportation.
- **Witness:** Lateral movement and rotation produce correct parallax; source/destination foreground objects clip/occlude correctly. Approaching the portal plane is handled with an explicit bounded camera rule.
- **Boundary:** No native clip-plane API or RTT. Visual linked camera is required; physical traversal/teleport gameplay is deliberately excluded. Derive the correct WebGPU projection or clip CPU geometry.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 80. nested-portals

- [ ] **80 SHIPPED** — `examples/8-advanced/nested-portals`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 79 `linked-room-portal`, 70 `stencil-region-stack`.
- **Outcome:** Explicit bounded recursion and stencil ownership for advanced developers.
- **Build/repair:** After the single-link portal is proved, add exactly two visual recursion levels with explicit stencil allocation/cleanup, depth reconstruction and a clear terminal view.
- **Witness:** Nested apertures, intersecting doors and termination never leak previous-room imagery.
- **Boundary:** Fixed two-level visual recursion, not a general recursive renderer or gameplay traversal. Failure of this objective remains blocked, not replaced by two unrelated windows.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 81. scope-and-viewmodel

- [ ] **81 SHIPPED** — `examples/8-advanced/scope-and-viewmodel`
- **Class:** new; A; guest; phase B. **Prerequisites:** 66 `stencil-demo`, 19 `camera-projection-lab`.
- **Outcome:** Combine a magnified scoped view, reticle and close-up weapon rendering.
- **Build/repair:** Use a stencil aperture, separate projection/depth pass, then a restored HUD/viewmodel pass.
- **Witness:** Zoom affects only the aperture and the viewmodel never clips against distant world depth.
- **Boundary:** Direct re-render; no scope RTT or automatic camera/viewport state restoration.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 82. cutaway-diorama

- [ ] **82 SHIPPED** — `examples/8-advanced/cutaway-diorama`
- **Class:** new; A; guest; phase B. **Prerequisites:** 66 `stencil-demo`.
- **Outcome:** Reveal an interior while keeping a coherent exterior shell.
- **Build/repair:** A finite authored shell and interior mesh set with a movable reveal window, stencil region and explicit depth sequence. Show the combined result and a labeled pass diagnostic.
- **Witness:** Camera rotation preserves the cut boundary and hidden walls do not leak through.
- **Boundary:** An authored interior reveal, not arbitrary geometric CSG, a watertight generated slice or an unimplemented user clip plane.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 83. split-screen-worlds

- [ ] **83 SHIPPED** — `examples/8-advanced/split-screen-worlds`
- **Class:** new; A; guest; phase B. **Prerequisites:** 67 `viewport-test`, 79 `linked-room-portal`, 54 `epu-textures-demo`.
- **Outcome:** A combinatorial scene that exercises viewports, portals, EPU and HUD together.
- **Build/repair:** Two local views of different rooms, each with its own simple portal, explicit EPU source and HUD. Reuse the proved linked-portal sequence; isolate/reset stencil and schedule or rebuild depth around attachment-wide clears.
- **Witness:** Both players see their own room, portal, lighting and HUD with no cross-contamination.
- **Boundary:** Mid-frame depth clears are global; render order/rebuild must account for other viewports.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 84. orthographic-projected-textures

- [ ] **84 SHIPPED** — `examples/8-advanced/orthographic-projected-textures`
- **Class:** new; A; guest; phase B. **Prerequisites:** 21 `mesh-submission-lab`, 66 `stencil-demo`.
- **Outcome:** Real receiver-relative projected markings without inventing a projector API.
- **Build/repair:** Compute orthographic projector coordinates on CPU-known receiver triangles, clip the projection footprint, and submit bounded overlay triangles with a small normal offset and explicit depth/stencil. Preload texture resources in init.
- **Witness:** Move/rotate the projection basis; markings track the surface and stay within the projection footprint.
- **Boundary:** Ordinary projected surface marking, not general perspective projection, additive light, or automatic projection onto arbitrary ROM mesh geometry.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 85. terrain-conforming-decals

- [ ] **85 SHIPPED** — `examples/8-advanced/terrain-conforming-decals`
- **Class:** new; A; guest; phase B. **Prerequisites:** 84 `orthographic-projected-textures`.
- **Outcome:** Spell circles, roads or landing marks that follow a known heightfield.
- **Build/repair:** Predefine one bounded heightfield; generate a tessellated moving decal patch from known CPU height/normal data and submit immediate triangles. Clip its footprint and document offset/tessellation budget.
- **Witness:** A mark across a slope/edge follows the terrain without floating or bleeding onto other receivers.
- **Boundary:** Requires CPU-known surface data and tessellation; geometric offset replaces unavailable native bias.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 86. perspective-projector-approximation

- [ ] **86 SHIPPED** — `examples/8-advanced/perspective-projector-approximation`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 84 `orthographic-projected-textures`.
- **Outcome:** Teach the difference between vertex UV projection and true projective texturing.
- **Build/repair:** Compute projector-divided UVs on a subdivided known receiver; draw via immediate float triangles. Compare coarse/fine subdivisions to an independent offline reference image/analytic UV error witness.
- **Witness:** Declare a geometric domain and numeric/image error tolerance before evaluation. Demonstrate converging bounded error with tessellation and show the excluded grazing failure mode; source labels and images must agree.
- **Boundary:** Current guest vertex UVs are ordinary 2D; do not advertise exact fragment-level projective division.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 87. water-plane-reflections

- [ ] **87 SHIPPED** — `examples/8-advanced/water-plane-reflections`
- **Class:** new; A; proof-first; phase B. **Prerequisites:** 78 `planar-mirror`, 84 `orthographic-projected-textures`.
- **Outcome:** Combine a proved planar reflection with a stylized animated water surface.
- **Build/repair:** Reuse the proved one-plane mirror and add a preloaded surface texture with guest-generated moving UVs and declared dither coverage. Explicitly order reflected scene, water surface, opaque foreground and HUD.
- **Witness:** Reflection remains correctly oriented while shoreline occlusion and surface motion are legible.
- **Boundary:** A stylized planar reflecting surface, not physical blended water, refraction, SSR, arbitrary waves or a distortion shader.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 88. light-cone-impostors

- [ ] **88 SHIPPED** — `examples/8-advanced/light-cone-impostors`
- **Class:** new; A; guest; phase B. **Prerequisites:** 12 `billboard`, 13 `dither-demo`.
- **Outcome:** Cheap shafts/searchlights with honest rendering semantics.
- **Build/repair:** Use preloaded textured cones/cards with dithered coverage and explicit depth testing in a small searchlight scene. Demonstrate placement and camera-crossing limitations without additive light claims.
- **Witness:** Camera crossing, opaque foreground geometry and cone intersections expose artifacts clearly.
- **Boundary:** Not additive volumetric lighting or participating-media scattering; document the illusion.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 89. layered-effects-lab

- [ ] **89 SHIPPED** — `examples/8-advanced/layered-effects-lab`
- **Class:** new; A; guest; phase B. **Prerequisites:** 12 `billboard`, 13 `dither-demo`, 66 `stencil-demo`.
- **Outcome:** Make shields, smoke cards and holograms usable together.
- **Build/repair:** Compose a small shield, a smoke card emitter and a hologram with explicit draw/pass order, depth-write choices and dither offsets. Keep resources preloaded and per-frame emission bounded.
- **Witness:** Overlap tests and camera turns show the declared coverage, occlusion and draw ordering.
- **Boundary:** No general smooth transparency, additive particle pass, distortion or bloom is assumed.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


#### 90. culling-lod-budget-scene

- [ ] **90 SHIPPED** — `examples/8-advanced/culling-lod-budget-scene`
- **Class:** new; A; guest; phase B. **Prerequisites:** 21 `mesh-submission-lab`, 19 `camera-projection-lab`.
- **Outcome:** A practical large-scene recipe rather than an empty stress loop.
- **Build/repair:** A fixed seeded scene and repeatable camera path with CPU guest bounds/frustum tests and authored LOD meshes. Record actual draw counts, source/device/build identity and CPU timing; GPU timing only if real instrumentation provides it.
- **Witness:** For culling A/B, keep identical LOD and require equal visible content plus measured costs. For LOD A/B, permit declared image differences and quantify the separate quality/cost tradeoff.
- **Boundary:** Culling equivalence and LOD approximation are different claims. No invented GPU timings or occlusion-query API; warm/cold costs remain distinct.
- **Progress:** PLANNED; evidence: none; delivery: none; human: unreviewed.


## Change and decision record

- 2026-10-10 — Promoted the 90-entry exploratory catalogue into this reviewed execution contract. Preserved all 44 existing identities and all 46 proposed objectives. Corrected dynamic-geometry/submission assumptions, explicit manifest defaults, private dependency risks, shader/blend/stencil/clip contracts, pilot and completion semantics, scope of visual portals/IK/physics, and source-clean commit/remote acceptance. No example implementation was created or certified during this review.
- 2026-10-10 — User explicitly authorized Astra instead of the earlier Sol-only owner requirement. Same single-owner scope and all five acceptance gates remain in force.

Append only consequential scope/acceptance decisions here. Routine progress belongs to the checkpoint and per-item lines. Changes to required objectives, project count, blocked-pilot substitutions or console API scope require explicit authorization and a visible diff; do not quietly edit the finish line.
