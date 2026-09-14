# EPU correctness and open authoring — implementation plan

Status: WORKING CHECKPOINT / PAUSED — see [epu-checkpoint.md](epu-checkpoint.md) for verified build/use commands, known limitations and restart ownership. The user authorized committing and pushing this checkpoint to `main`. `epu-progress.md` retains the execution ledger. Whole-goal technical and human visual acceptance remain open; performance acceptance is deferred.
Baseline inspected: nethercore e7040c1e6938d8843d0b2123141c8423e41e554f.
Workspace: D:/Development/nethercore-project/nethercore.
Related design checkout: D:/Development/nethercore-project/nethercore-design.

This document supersedes the environment-specific proposals in the preceding conversational plan. It is the task's durable scope, handoff, and progress home. Revalidate source facts against the current checkout before editing.

## 1. Product contract — authoritative user intent

Artists should be able to create experimental abstract directional backgrounds, animate their parameters, and obtain coherent environment lighting without painting skybox faces or understanding lighting mathematics. The console supplies predictable mechanisms, not an aesthetic or a world simulation.

- NON-OPINIONATED is a hard requirement for EPU and a guiding constraint for the console. Do not encode preferred compositions, moods, colors, scene types, or animation narratives in runtime behavior.
- Background and object lighting represent the same authored directional environment, with documented filtering differences.
- Keep the compact, bounded, deterministic, fantasy-console character of Nethercore. Improve existing mechanisms and tools before adding mechanisms.
- Expose only controls that actually work, with predictable units/ranges and honest limitations.
- Preserve artist freedom: an unconventional, flat, harsh, monochrome, or extremely bright composition is not a defect merely because an agent dislikes it.

The user's earlier references to time of day, lightning, interiors, and eerie worlds were examples of things arbitrary colors, shapes, parameter motion, and intensity changes might express. They are NOT required engine features or mandatory presets. "Fog" meant a greyish environment color, NOT fog affecting geometry.

### Expressive capability expansion — user authorized

Correctness repairs alone are not the product. Implement missing useful algorithms when the existing layered system cannot express needed game environments: indoor, outdoor, hybrid; real-world, sci-fi, cityscape, forest, fantasy, whimsy and horror. These are breadth requirements and review cases, not mandated engine presets or one opcode per genre. Abstract output must still be appropriate to the game; "non-opinionated" must not become an excuse for a palette of generic blobs/noise with no structural expressiveness.

Use the prior artistic example attempts to diagnose capability ceilings, not endlessly retune presets or launch another broad batch. For each demonstrated gap distinguish existing-but-broken, hidden/unusable authoring, genuinely missing algorithm, and representation limit. Reuse/repair first; add the smallest reusable shape/distribution/composition/motion mechanism when genuinely missing. Developer-controlled specialization is welcome; engine-prescribed palette, story, mood or world simulation is not. Preserve layer composition, public values and existing cartridges. Breaking changes still require an explicit contract decision.

Capability acceptance needs both a concrete behavioral test and a developer-facing composition demonstrating the missing capability. Reuse the same mechanism in materially different relevant contexts where practical; do not claim a universal world/style solution from one sample. Human judgment decides game-appropriateness. Cover the named breadth with a bounded selection/reuse of existing examples, not an arbitrary new20-render quota. Extend ordered correctness/authoring slices where a capability gap belongs; performance remains deferred.

**Required capability retention — regular and organic RADIAL:** Keep the shown regular RADIAL construction, which Robert says looks fine, and restore the useful organic/warped behavior represented by the old gas-giant-like comparison. This capability must return fixed alongside the regular construction before whole-goal completion; regular cells alone do not close it. Repair/reuse the retained warped construction and existing composable mechanisms first. Exact broken code or pixel-identical old scenes are not required, but comparable flowing bands and nested oval/vortex-like forms must actually be demonstrated. Add a minimal generic mechanism only if that reuse cannot provide the capability; do not hard-code a gas-giant preset or host animation controller. Retain old source/reference captures, reconcile full-source lineage before a new matched replay, and require coherent shape/ownership/boundaries, focused RED/GREEN, real guest/render verification and human comparison. This restoration remains OPEN, not delivered by the regular-grid technical pass.

### High-priority visual correctness — per-algorithm discontinuity gate

Unintended discontinuity seams are correctness defects, not optional polish or taste. Prioritize their reproduction and root-cause repair ahead of new expressive mechanisms and showcase expansion. Keep the algorithm ledger in `epu-progress.md` as the single durable checklist; do not add another tracking system.

- One algorithm, one explicit verification record. Inventory all current evaluators; split the record into subchecks where variants/domains use distinct coordinate, noise, support or phase paths. Newly added algorithms enter this gate before acceptance. A shared-helper fix does not automatically certify its callers.
- For each record identify applicable boundaries: angular wrap/poles, lattice/hash cells, chart/cube/octahedral folds, finite-support endpoints, region/layer composition and guest-driven cyclic phase. Mark genuinely inapplicable boundaries N/A with a source reason; preserve intentional authored hard edges rather than smoothing everything.
- Reuse existing tests/captures first. For a reported defect retain focused behavioral RED, the smallest shared root-cause repair, GREEN at a declared tolerance and a nonzero/interior-preservation check so deleting the effect cannot pass. No blur, retuning, clipping or masking to hide a seam.
- Verify actual assembled shared shaders and matched exported real-WASM guest/player inputs (configuration, camera, frame and phase), retaining source/build identities and before/after evidence. Include background/reflection paths where applicable. One reusable runner may cover many records; one screenshot or global test count may not mark them all verified.
- Track numeric status separately from real-player evidence and human visual review. Use OPEN, IN_PROGRESS, GPU_PASS, RUNTIME_PASS, or VERIFIED; VERIFIED requires all applicable technical checks and explicit human visual acceptance. Existing bounded evidence starts partial, not blanket VERIFIED.
- Keep distinct symptoms distinct: trail truncation and purple/grey FLOW field lines each require their own demonstrated cause and closure. Review corrected output with the user; do not relabel an objection as intentional without source evidence and human review.

Performance remains explicitly deferred; continuity correctness is not deferred with it. Parent Hermes owns reconciliation and resumes remaining algorithms after the current finite seam verification.

### Explicit exclusions

No day/night model, weather system, lightning controller, event scheduler, physical atmosphere, geometry fog, volumetric renderer, bloom pipeline, room simulation, parallax probes, shadowing system, general scene/node editor, automatic artistic correction, mandatory recipe taxonomy, or new PBR material model. No global console redesign under this task. No automatic palette/style selection. No new opcode family, ABI field, dependency, or framework without a demonstrated gap and the smallest justified proposal.

Existing themed effects/examples need not be deleted merely for having themes. They remain optional content; do not expand or hardwire them into the core. Content examples never define normative runtime semantics.

## 2. Dispatch and ownership

Suggested dispatch:

> Implement docs/plans/epu-correctness-and-open-authoring.md in order. Revalidate the current source, preserve unrelated work, use the existing checkout and tools, and obey the non-opinionated exclusions. Complete each bounded slice through real runtime checks before moving on. Repair recoverable failures autonomously. Do not silently break cartridge contracts or claim artistic/performance acceptance without its required evidence. Update the compact progress section in this document; do not create a parallel planning system.

- This document authorizes nothing until an implementation request is issued. Plan creation changes documentation only.
- Implement in the authoritative checkout; record starting revision and dirty state. Preserve concurrent/unrelated work. Do not reset, clean, or create disposable worktree copies.
- One source writer at a time. Delegate only bounded independent inspection/review or a non-overlapping result that saves work.
- Reuse relevant Nethercore, working-conventions, and evidence-gated-development guidance. Routine local slices use lean delivery. Changes to shared material behavior, public instruction semantics, or cartridge compatibility warrant risk-focused assured review because of downstream breakage.
- A worker summary is not acceptance. The owner inspects changes and exercises the real path. On recoverable failures, repair rather than leaving the task blocked because a worker stopped.
- No automatic commits, pushes, releases, or changes to sibling projects unless separately authorized. Propose needed sibling documentation synchronization rather than silently editing its independent contract.

## 3. Reconnaissance and technical decision rules

Before implementation, locate actual shader assembly, all evaluator/sampling callers, environment sources, cache lifetime, material modes, guest bindings, and existing tests/inspectors. Use canonical include/zx/mod.rs and the live host implementation to resolve authority; document discrepancies rather than choosing prose opportunistically.

Performance testing and optimization are deferred while the GPU is shared with AI rendering. Retain clean performance acceptance as an explicit later gate; do not block correctness repairs on a new timing baseline. When that gate resumes, compare source-bound before/after builds with matched hardware, settings, resolution and inputs across static, continuously changing and abrupt-change cases. A baseline measured after repairs is not a pre-repair baseline; label unavailable historical measurements honestly.

Audited leads (paths below are under nethercore-zx unless stated otherwise):

- shaders/epu/epu_compute_env.wgsl versus shaders/common/20_environment/90_sampling.wgsl: composed versus replacement bounds evaluation.
- shaders/epu/epu_common.wgsl: max-preserving bounds composition, layer clamping, placeholder HSV modification.
- shaders/epu/bounds/01_sector.wgsl and sibling bounds: source/modifier ambiguity.
- shaders/epu/epu_compute_blur.wgsl: projection-space downsampling and fold-side weighting.
- shaders/epu/epu_compute_irrad.wgsl: conventional SH basis/kernel plus custom L2 attenuation.
- shaders/blinnphong_common.wgsl: environment BRDF/Fresnel convention, material-dependent ambient multiplier, shell suppressions.
- build_support/generator.rs: direct-light roughness/shininess convention.
- shaders/common/10_unpacking.wgsl: normal-map diagnostic bypass.
- shaders/epu/bounds/05_patches.wgsl and features/02_scatter.wgsl: angular periodicity leads.
- src/graphics/epu, src/ffi/environment, build_support/sources.rs: runtime resources, dispatch, assembly, invalidation.
- tools/epu_workbench.py and examples/3-inspectors: existing authoring and verification surfaces.

These are source-audit findings, not fresh rendered reproduction. Earlier Python calculations were diagnostic transcriptions, not GPU tests. Reproduce the relevant defect before claiming it fixed. Do not blanket-rewrite every listed file.

### Documentation / website / API consistency — required deliverable

User explicitly requires EPU-related documentation to be updated or removed if obsolete, including website and API documentation. Inventory actual publishing source and generator paths, not just filenames containing EPU: book/API/architecture/tutorials/cheat-sheet, SDK bindings and Rustdoc, sample comments/snippets, inspector metadata/help, and obsolete design guidance. Current known book surfaces: docs/book/src/api/epu.md, guides/epu-environments.md, architecture/epu-overview.md; verify website deployment source rather than assuming the book is the only site.

Update implementation claims alongside each accepted behavior/control change. Remove superseded public instructions and repair incoming links/navigation; retain historical material only when it serves a real purpose and is unmistakably non-normative, not as competing current guidance. Do not delete the active plan/progress or unique unresolved requirements merely because they are old. Label proposed/unimplemented algorithms and visual hypotheses as such. Verify opcode/domain/variant inventories, packed fields, units, ranges, seed versus phase, layer/mask semantics, supported guest versus host APIs, import paths, and real limitations against source. Compile/run executable snippets and build/check local docs with existing tooling. Correct generated documentation at its source, not only emitted output.

Documentation acceptance includes website/API source coverage and working local build/links/examples, not only an added changelog note. Existing no-commit/no-push/no-public-hosting constraint remains: local source/build completion is distinct from public-site publication. Identify external/sibling publishing ownership and authorization before modifying another checkout or deploying; do not silently claim the live site updated. Documentation must not gate on deferred performance measurements, nor present contention-confounded timings as clean benchmarks.

### Compatibility and stop rules

Preserve the packed instruction format, opcode numbers, source routes, and public API by default. Before changing an instruction's meaning, inspect all callers/examples and identify affected cartridges. Correct an internal disagreement at its common root, not by separately tuning both paths.

Do not silently redefine published semantics, repurpose encoded values, remove supported modes, or change color interpretation globally. If a minimal correct fix genuinely requires a breaking contract, stop that portion with a concrete decision: old behavior, proposed behavior, affected consumers, migration/rollback options. Continue independent non-breaking work. Do not build a speculative compatibility framework.

Prefer one documented source/modifier rule over hidden mask-preservation heuristics. Select the exact rule from actual contract/caller evidence; this plan does not decree that every existing bounds opcode changes meaning.

## 4. Slice A — trustworthy evaluator and supported contracts

Outcome: a single instruction program means the same thing in background and lighting paths.

Work:
- Extract/reuse one shared WGSL evaluator in actual shader assembly; eliminate independently maintained semantic loops.
- Establish explicit bounds behavior. Initial default regions must not contaminate the first establishing source. Paint opacity and structural region ownership must have explicit, tested meanings.
- Include opcode-level paint/region remapping in the neutrality audit, specifically RAMP wall-paint softening and single-region coverage boosting. Resolve documented behavior and compatibility before changing either; shared evaluation alone does not remove aesthetic prescriptions, and this is not authorization to delete every shaping function.
- Restore normal-map sampling, preserving deliberate skip controls.
- Repair demonstrated domain seams and unsupported-value behavior. Validate legal domains/variants at the appropriate existing boundary; use documented fallback/NOP behavior rather than inventing a global policy.
- Resolve misleading capability metadata/placeholder claims by the smallest compatible fix. Do not implement a sophisticated HSV system merely to justify a label.

Acceptance (all five):
1. First-source and sequential-bounds fixtures agree between direct evaluation and cache sampling within a declared sampling tolerance.
2. Mask previews match subsequent feature placement; no phantom all-sky inheritance.
3. Normal-map on/off produces the expected distinction on real material shaders.
4. Supported seam-crossing/domain cases and invalid-value cases have focused executable checks.
5. Shared shaders compile and the existing real inspector/cartridge path runs without regression in exercised modes.

## 5. Slice B — coherent environment lighting

Outcome: rough materials respond to the authored environment rather than to its storage projection or symptom patches.

Work:
- Retain SH9 as the diffuse baseline. Extract SH from source radiance or a separately justified radiance reduction, not an already specular-convolved mip. The current SH binding reads the reflection mip chain; explicitly repair this dependency when changing that chain. Preserve constant radiance and verify a nonconstant low-order directional field against the diffuse reference. Remove arbitrary band/ambient corrections only against controlled comparisons.
- Replace rough-reflection projection-space downsampling with bounded direction-space lobe filtering. Keep octahedral storage initially; implement correct boundary handling. No fold avoidance weights. Increasing resolution or shrinking the final mip alone is not the fix.
- Define one roughness/shininess convention consistent with the existing Blinn–Phong identity. Before implementing Slice B, record the chosen kernel, normalization, roughness-to-filter mapping, and reference in this document. These are design decisions to justify, not already established source facts. Align direct-light response, environment filtering, and gain sufficiently to that reference.
- Correct the Fresnel/BRDF convention. Merely substituting F0 into a borrowed fit is not proof that the fit matches the selected lobe. Prefer an existing suitable calculation; a small fixed integration table is an option only if justified, not a mandated subsystem.
- Remove shell suppressions as their root causes are fixed. Preserve explicit artist controls such as rim lighting. Do not remove all heuristics blindly or retune per preset.
- Document color space, display conversion, and environment energy. Do not blindly remove every clamp or globally reinterpret RGB bytes. If current encoding cannot express a justified bounded intensity range compatibly, present that gap rather than introducing a silent ABI change.

Acceptance (all five):
1. Constant environment remains constant through filtering and diffuse reconstruction within documented numerical tolerance; a nonconstant low-order directional field matches the diffuse reference without unintended specular preconvolution.
2. Directional and seam-crossing test fields retain correct orientation and produce smooth broad-lobe response without projection-imprinted discontinuities.
3. Roughness sweeps on dielectric and metallic materials show no unexplained discontinuity or extra grazing boost against the chosen reference.
4. Background, diffuse, reflection, and normal-mapped paths remain coherent across supported procedural/imported sources and affected material modes.
5. Real GPU execution is checked; a small reference comparison and source-bound independent review cover the changed shared lighting surface.

### Slice B kernel decision — implementation candidate, acceptance pending

Retain the existing direct Blinn–Phong equation: `f_s = F0 * C(s) * max(N·H,0)^s`, `C(s)=0.0397436*s+0.0856832`; the outgoing environment contribution integrates `f_s * L_i * max(N·L,0)` over incoming directions. Mode2 uses `s=1+255*(1-roughness)`; Mode3 preserves its existing normalized-shininess mapping. No additional borrowed split-sum Fresnel term or shell gain is part of this reference. This preserves the declared direct model, not a claim of a new microfacet/PBR model or unit hemispherical gain at every view angle.

Candidate implementation integrates source mip0 with128 importance-sampled half-vectors per tangent frame, actual view direction and the `4*(V·H)` Jacobian. An actual GPU RED exposed discontinuity at the original abrupt frame switch (0.003905802 versus0.002). The repaired candidate blends Y/Z frames with squared cross-product weights, each vanishing at its singular axis; weight sum >=1. It uses up to256 fetches, not projection-space roughness mips. Actual finite GPU GREEN: angular pair residual0.0000006365, independent reference convergence0.0000055021; existing2880-case budget and imported directional gates unchanged. Full GPU suite20 passed,2 performance ignored. See epu-progress.md for exact logs and active source-bound review. This is bounded quadrature, not arbitrary-frequency or universal angular acceptance. Increased material cost and source-cache resolution limits remain explicit; performance testing and optimization remain deferred under the user override.

## 6. Slice C — neutral authoring and parameter animation

Outcome: users shape and animate an environment without learning packed bytes or accepting engine-prescribed content.

Work:
- Extend the existing inspector/workbench only where a usability gap is demonstrated. Reuse current save/load/export and metadata generation.
- Expose meaningful labels, ranges, and supported variants from existing authoritative metadata. Seed, phase, orientation, scale, opacity, and intensity are distinct concepts; show only the ones an operation actually supports.
- Reuse/add the minimum layer isolation and region/background/diffuse/reflection inspection controls necessary to understand composition. No mandatory graph editor, new application, or recipe architecture.
- Keep simulation-owned deterministic parameter state. Rendering must not advance gameplay time/RNG. No EPU-owned clock, event type, named mood, or scheduler.
- Exercise arbitrary parameter evolution: scalar ramp, cyclic phase, orientation change, and abrupt intensity/color change. These are generic test inputs, not new runtime animation controllers.
- Test existing phase resolution before proposing format changes. Advertise looping only where the function is genuinely periodic. Do not automatically smooth deliberate steps or randomize seeds to disguise discontinuities.
- Concrete user motion acceptance: columns of rain must admit a simple guest-owned wrapping counter driving a meaningful travel/phase control (conceptually `tick = (tick + 1) % 256`). Tracks/columns must remain coherent as rain travels, rather than reseeding, flickering, or merely pulsing; the wrap must be an ordinary motion step, not a reset/pop. Freeze unrelated seed/shape/color controls, exercise consecutive steps and the wrap in a real guest, and obtain human motion acceptance. A spatial/Z offset is an illustrative desired control, not authorization to invent an API field or reinterpret RGB Offset. Use simulation update, not render, to advance the driver; direction and speed remain developer-owned. Record the actual supported control and any resolution/steering limitations in the environment/effect capability mapping.
- Extend the motion acceptance cases with guest-owned phase +2 each simulation update and +1 every other update (held frames intentional). Verify skipped-step wrap as well as single-step wrap; speed changes must not reseed the pattern. For twinkling stars, freeze star positions/identity and drive smoothly periodic brightness with stable per-star variation rather than synchronized global flashing or position reshuffling. Noise sampled along a looping phase trajectory must close temporally; translated repeating fields must also close at their spatial tile boundaries. Spatial continuity and temporal looping are separate gates. Check wrap-adjacent motion/brightness against ordinary adjacent steps, not merely identical endpoint images. These are composable controls usable for space or night scenes, not engine-owned scene controllers; byte-phase stepping/resolution limits must be disclosed rather than hidden with automatic host interpolation.
- Concrete composition acceptance: user example SILHOUETTE-CITY plus a starry SCATTER layer with guest-driven twinkle. Treat 'STARRY SCATTER NIGHT' as the user's intended look, not a new opcode/preset name. Use existing ordered layers and region masks so stars occupy the authored sky/open region rather than painting through the city. Hold skyline geometry/region ownership and star positions/seed fixed while advancing only the star animation control; verify independent edits, consecutive phases, held/skipped steps and wrap in the composed real-WASM guest, then human-review the clip. Neither isolated CITY nor isolated SCATTER tests establish this composition. No special city-night controller or mandatory palette; the same star layer must remain reusable with other bounds/compositions.
- Optional example configurations must be editable ordinary instruction programs with neutral labels. No embedded preferred palettes, compositions, or named-effect dependencies.

Acceptance (all five):
1. A user can construct, inspect, edit, save, reload, and export an environment through the existing tool without hand-packing its fields.
2. Exposed controls map to real supported behavior; inactive controls are absent/clearly disabled, not misleading.
3. Exported configuration reproduces composition in a real guest cartridge; a small guest-owned deterministic parameter driver reproduces the specified motion from that configuration. Animation-controller serialization is out of scope; a configuration snapshot alone does not promise motion export.
4. Parameter scrubbing, periodic wrap where promised, abrupt changes, and rollback/replay show no stale environment state; no claim of cross-GPU bit-identical floating-point pixels.
5. Human review confirms useful creative control on a bounded sample; automated checks do not choose the aesthetic or declare abstract art beautiful.

## 7. Slice D — bounded cost, integration, and release decision

**User priority override:** defer performance work while the GPU is shared with AI image/video rendering. Retain earlier timestamp observations as contention-confounded evidence, not clean pre-change/final baselines; do not use them to drive optimization, caps or feature decisions. They do not invalidate deterministic correctness results, but numerical/visual acceptance remains scoped to its checks. Prioritize correctness, useful neutral controls, authoring flexibility and real guest round-trips. A later clean performance window is a release gate, not a blocker for that work.

Outcome: a verified supported system, not an empty-scene demo or an unsupported "bug free" claim.

Work:
- Compare the repaired implementation against the pre-change baseline captured during reconnaissance, using the same machine, build settings, scene, resolution, and configurations with actual timing/resource outputs. Read current console constraints instead of copying stale values.
- Cover static state, continuous parameter changes, abrupt changes, multiple environment IDs/viewports, and representative geometry/materials. Measure cache builds, background draw, material sampling cost, CPU updates, memory, and frame-time spikes.
- Retain existing procedural dirty-state caching and per-environment reuse. Imported environments currently rebuild without equivalent dirty-config filtering; measure that path separately rather than assuming caching parity. Add imported caching only for a demonstrated bottleneck with explicit invalidation checks. Confirm configuration/source changes and restored game state invalidate correctly. Do not key content only by a reused environment ID.
- Bound instruction/filter work. Avoid full procedural evaluation per material pixel where the cache demonstrably suffices. Do not sacrifice correctness with stale updates before profiling proves a need.
- No reduced-rate updates, temporal accumulation, or new cache layers unless a measured bottleneck requires them. If used, verify latency, ghosting, restored state, and abrupt changes explicitly.
- Correct touched host documentation/bindings/examples. Retain tiny regression inputs and reusable commands, not render dumps, transcripts, temporary roots, or duplicate builds.

Acceptance (all five):
1. All preceding technical criteria pass; no known blocking defect in the supported scope remains.
2. Procedural/imported paths, affected modes, multiple environments, and restored state are exercised in the real runtime.
3. Baseline-versus-final timings and resource use are reported with hardware/configuration, including worst exercised animated case.
4. Minimum target hardware and acceptable EPU budget are explicitly agreed before a universal performance/release claim; absence of that decision does not block earlier correctness slices.
5. Final human creative-control review and risk-focused compatibility review have explicit outcomes. Unmet gates are labeled, never silently waived.

## 8. Evidence, rollback, and completion

- Each slice uses the existing test infrastructure plus the smallest missing deterministic regression. Declare numerical tolerances before evaluating the result and distinguish numerical approximation from visible discontinuity.
- Reuse one compact real-runtime diagnostic scene: constant field, hemisphere split, broad lobe, seam-crossing detail, masks, and material probes. Add only cases needed for the defect being fixed. This is test content, not an enforced console aesthetic.
- Record actual commands/results below. CPU-only math and shader parse checks do not substitute for GPU integration or user-visible acceptance.
- Preserve a source baseline and use small reviewable diffs. Never revert unrelated edits. If a repair fails, revert only owned changes for that repair; no destructive checkout reset. Inspect status before and after verification.
- No hidden global shell tuning, disabled production features, or diagnostic bypasses left in release code. Remove temporary switches and generated QA artifacts after acceptance.
- "Complete" means finite supported contracts verified, no known blocking bugs, performance honestly scoped, and human creative-control judgment obtained. It does not mean all possible bugs disproved.
- Normal implementation flow is one bounded implementation wave and one focused repair/review wave per slice. Further work must address a concrete remaining defect, not optional process metadata.

## 9. Red–green execution and remote review

The user requests TDD/red–green refactoring and understandable before/after evidence accessible in this Telegram conversation, without logging into the development machine.

- For each concrete defect, state the expected behavior and add the smallest behavioral regression using existing infrastructure. Run it against unchanged production behavior and record the actual failing assertion/result (RED). A compile failure, missing dependency, or invented output is not a reproduced bug. If the suspected defect cannot be reproduced, mark it UNCONFIRMED and investigate rather than manufacturing a failure.
- Apply the smallest root-cause repair, rerun the same check (GREEN), then refactor only if it reduces duplication/complexity while keeping the check green. Exercise the actual shared shader/GPU and guest/inspector path; source-string assertions or independent formula transcriptions are not sufficient implementation acceptance.
- Before changing each visual behavior, preserve one small real baseline capture and its reproducible configuration. Compare the repaired output with the same camera, geometry, material, authored state, phase, resolution, and display settings. Record any necessary differences explicitly. Use short matched clips for temporal defects and images for static ones. Do not reconstruct a fictional before image after the fix. Keep baseline captures until the user's review completes; this is requested evidence, not a disposable render dump.
- Keep `docs/plans/epu-progress.md` as a concise evidence report, not a second scope/plan. This document remains authoritative. For each exercised issue record: plain-language symptom and cause, what to look at, source locations, exact test command, observed RED/GREEN result, baseline/candidate identity, capture references, compatibility/performance caveat, and human verdict if supplied. Distinguish SOURCE_CONFIRMED, RED_REPRODUCED, GREEN_VERIFIED, GPU_VERIFIED, AWAITING_HUMAN_REVIEW, and BLOCKED; never collapse these into an unsupported done label.
- Remote delivery defaults to this Telegram conversation: send short milestone summaries, the updated progress document as an attachment, and selected real before/after images or playable clips. The user explicitly requests these review artifacts. Deliver the smallest useful set per completed issue/slice rather than flooding the chat with every test run. Local paths alone are not remote delivery.
- If an interactive comparison materially improves understanding, reuse an existing authorized review surface or deliver a self-contained report attachment. Do not build a new dashboard/backend, alter unrelated Studio workflows, expose filesystem roots, open ports, or publish the repository to obtain a link. A hosted review link requires appropriate access controls and real route/media readback; localhost is not remotely reachable proof. Telegram attachments are the fallback, not a claim that interactive HTML works inside Telegram's preview.
- A fresh gateway session resumes from these documents and actual source/artifacts. Preserve unfinished and completed issue identities; do not rerun fixed work or duplicate producers merely because a notification was lost.

## 10. Compact progress / handoff

Update this section in place during implementation; do not duplicate raw logs here.

- Plan: IMPLEMENTATION AUTHORIZED; baseline and core Slice A started.
- Active milestone: core SH9/direct/ambient/specular, imported face-UV, signed normal and rotated-TBN checks have GPU evidence. Reflection continuity repair reviewed PASS_WITH_LIMITS. Exact static/motion export runtime parity, guest-owned animation/replay and neutral-material guest execution are verified in finite scope. Remaining non-performance technical criteria are being reconciled against existing evidence; owner integrates from `epu-progress.md`. Whole goal WORKING; no human or clean-performance acceptance.
- Independent review: gpt-6-astra / high, session 20260909_200833_a4aba8, returned READY_WITH_CORRECTIONS. Corrections incorporated: diffuse/specular source separation and kernel decision, snapshot-versus-motion export, opcode neutrality audit, pre-change performance baseline, and procedural/imported cache distinction. No fresh GPU/performance acceptance or second independent review is implied by this document update.
- Slice A: CORE FIXES GREEN; remaining seams, opcode/contract audit and full compatibility checks pending.
- Slice B: kernel/source filtering/BP/diffuse and finite imported/angular/face-UV tests GREEN; reflection review PASS_WITH_LIMITS. Broader pole/oblique-view claims remain unverified; do not restart completed repairs.
- Slice C: static/cyclic guest export and per-star SCATTER_PHASED animation/runtime/rollback are technically verified in their recorded finite scope. Broader creative-control, compatibility and human appearance acceptance remain open; do not restart the closed export/runtime work.
- Slice D: integration evidence accumulates with each repair; clean performance work is explicitly DEFERRED under the user priority override.
- Current source baseline: e7040c1e6938d8843d0b2123141c8423e41e554f (recheck at dispatch).
- Focused checks and real-path results: see `epu-progress.md`; actual GPU/guest core evidence recorded, not whole-plan acceptance.
- Performance target: as fast as reasonably possible, aiming at integrated GPUs from roughly the last decade. This is a target, not verified compatibility or an invented millisecond budget. Select representative supported hardware and record measured limits before final performance acceptance.
- Appearance policy override: the user authorizes breaking older EPU scenes for better correctness, quality or intuitive controls and delegates those choices to the owner. Uniform CELL Gap Alpha is selected. Preserve encoded cartridge/API layouts and unrelated work; do not preserve erroneous scene appearances through compatibility fades or reopen the resolved opacity question.
- Decisions remaining: representative minimum GPU/API capability and measured performance acceptance within that target; genuinely new ABI/layout decisions only if needed.
- Next action: resolve the newly reproduced VEIL AXIS_POLAR opposite-pole coverage defect. The existing near-pole support fade has no opposite-pole counterpart; mirroring it is the smallest candidate, but changes visibility and needs an explicit decision against the no seam-hiding fade/mask constraint. No pole repair has been applied. VEIL authoring is independently sealed: 39 editor, 165 GPU and 539 library passes; executable WGSL/packed contract/SDK helpers preserved, exact City/Grove/Arcade current-player replays, executable local docs. Eight existing breadth-review scenes are prepared without claiming useful scene identity. See `epu-progress.md` for the pole RED, ownership and exact receipts. Do not restart the closed showcase phase routing, SCATTER export/runtime, carrier loops, or SPLIT/SURFACE packed route. Human usefulness and clean performance remain separate open gates.

## Compatibility decisions — GRID and BAND

The user explicitly authorized regular whole-repeat GRID counts (even checker counts), whole-pattern-cycle scrolling, and the resulting change to older GRID layouts/rates. Its finite technical evidence is `tmp/epu-review/grid-regular-loop/closure.json`; overall and human acceptance remain separate.

The user also authorized separating BAND modulation depth from phase using previously unused Color B alpha: zero depth retains a plain ring, while phase is a cyclic position including zero. Old BAND appearance may change. Keep the packed layout and existing API signatures; expose any additive convenience API and the migration explicitly. Reproduce the zero-phase sentinel jump, preserve native before captures, and close actual guest/GPU, authoring, source/documentation and human gates separately. These decisions do not authorize controllers, geometry fog or unrelated API changes.
