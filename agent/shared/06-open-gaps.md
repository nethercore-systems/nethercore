# Open Gaps

This file is the current mutable punch list.

## Workflow Note

- One writer agent per file at a time. Reassign ownership explicitly before another agent edits that file.
- Keep authoring separate from verification, capture, and review. The worker that makes a change does not also sign off on that same change.
- Prove the delegated path with one minimal proof-of-life task before assigning a larger batch.
- Do not interrupt long-running workers unless the user asks or a real stop condition from `agent/session-protocol.md` is hit.
- Run dependent stages sequentially. Only independent read-only audit may run in parallel.
- Implement and fix workers should re-read the EPU guide, rendering architecture context, and `src/constants.rs` before targeted visual tuning.
- `agent/shared/07-preset-briefs.md` is now the canonical visual contract for every preset. Review and fix work should cite it directly.
- Review and benchmark verdicts must treat EPU as metaphor-first ambient/reflection/direct-view world art, not literal prop rendering.
- Determinism checks must use content-matched screenshot windows. Do not treat naive latest-batch comparisons as authoritative when benchmark and showcase runs were interleaved.
- The unattended deterministic-pair helper now exists at `tools/tmp/run_epu_replay_pair.py`; prefer it over manual dual-launch capture when running overnight benchmark/showcase pairs.
- The live workbench is now part of the standard workflow. Use `agent/shared/12-live-workbench.md` plus `tools/epu_workbench.py` for rapid local discovery, then replay-promote only the winning candidates.

## Standing Review Traps

- Intended animation only passes when motion is obvious across the reviewed frames.
- Looping or repeated patterning is a standing defect category; do not assume it is authored intent.
- Giant flat bands or broad solid-color shelves are also a standing defect category unless a preset intentionally uses a banded feature as the hero read.
- Engine-driven EPU or rendering bugs are plausible. If an artifact survives across presets, opcode swaps, or domain swaps, log `suspected engine bug` and stop content-only churn until isolated.
- Treat EPU work as procedural/generative world art, not literal scene modeling. Bounds should establish the world envelope; later feature layers should carry the readable motifs and most of the motion.
- Treat animation as variant-specific reality, not an opcode-family wish. `PORTAL/RECT`, `TRACE/LIGHTNING`, `SCATTER`, `APERTURE`, and `BAND` all have practical limits that should shape authoring and review.
- If a target clearly needs missing directionality, variant families, or behavior outside the current opcode surface, log an opcode-surface gap and consider engine/opcode work instead of repeating the same content churn.

## Current State

- Current preset count in code: 12
- `screenshot-all-anim3.ncrs` now matches the current 12-preset sweep and toggles the UI off before capture.
- Replay syntax proof-of-life passed: `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs` exited 0.
- Targeted format check passed for the touched Rust preset files.
- Example WASM build passed after wiring presets 11-12.
- Real-player capture runs now reach the current authoritative deterministic baseline: the latest 36-PNG batch is `2026-03-12-epu-showcase-12preset-replay-81`, written to `%APPDATA%\\Nethercore\\data\\screenshots` from `2026-03-12 03:30:50` through `2026-03-12 03:31:03`.
- Replay screenshot determinism is fixed for the current pipeline: runs `2026-03-11-epu-showcase-12preset-replay-28` and `2026-03-11-epu-showcase-12preset-replay-29` produced `0` hash mismatches across all `36` per-frame pairs when rechecked locally.
- Full deterministic adversarial review of run08 failed all 12 current shipping presets against `agent/shared/07-preset-briefs.md`; the main blocker is now visual contract quality, not screenshot nondeterminism.
- Newer deterministic pairs are now confirmed for the current content state as well: runs `2026-03-12-epu-showcase-12preset-replay-42` / `...-43`, `...-44` / `...-45`, `...-46` / `...-47`, `...-48` / `...-49`, `...-50` / `...-51`, `...-52` / `...-53`, `...-54` / `...-55`, `...-56` / `...-57`, `...-58` / `...-59`, `...-60` / `...-61`, `...-62` / `...-63`, `...-64` / `...-65`, `...-66` / `...-67`, `...-68` / `...-69`, `...-70` / `...-71`, `...-72` / `...-73`, `...-74` / `...-75`, `...-76` / `...-77`, `...-78` / `...-79`, and `...-80` / `...-81` each produced `0` hash mismatches across all `36` per-frame pairs.
- A real systemic bug is now closed: `SPLIT` documented `blend_width` as `0.0..0.2` but the shader was applying `0.0..1.0`. Fixing that bug materially improved both `Frozen Tundra` and `Storm Front`, which means at least part of the outdoor churn really was a surface bug rather than only weak authoring.
- Two additional engine-level theories are now closed by deterministic review. Sequential bounds composition was corrected so later bounds no longer hard-overwrite earlier region ownership, and single-region feature masks now get a focused boost under softly mixed bounds. Both changes are likely correct runtime behavior, but neither materially improves `Frozen Tundra` or `Storm Front` in direct review. The remaining blockers are therefore not just overwrite semantics or diluted one-region mask math.
- Three focused fix loops now exist across the current proof-of-life work, all fully captured and reviewed on deterministic pairs. `Combat Lab` now clears run29 and becomes the first indoor proof-of-life pass in the roster. `Ocean Depths` failed in two different ways on the same day: run19 overcorrected into abstract teal debris, while run21 recovered the underwater shaft-based mood but still lacks a trench-floor anchor and obvious motion. The newest outdoor structural retry on `Frozen Tundra` and `Storm Front` still fails on run35, so the remaining gate is outdoor proof-of-life.
- Read-only capability audit now explains part of the churn: `PORTAL/RECT` is static, `TRACE/LIGHTNING` is a static strike shape, `SCATTER` is seed-driven shimmer, `APERTURE` is a bounds remapper, and `BAND` phase is azimuthal rather than a true horizon-scroll. Future content loops must plan around those limits instead of assuming all variants animate from `param_d`.
- New outdoor loop evidence now narrows a second capability issue: current `VEIL/RAIN_WALL` domains each fail differently for `Storm Front`. `DIRECT3D` turns the sheet into giant side-arc fans, `TANGENT_LOCAL` hides too much of the storm in direct view, and `AXIS_CYL` finally exposes a weather event but only as giant geometric pillar slabs. That is now a demonstrated surface-vocabulary limitation, not just preset churn.
- Two more post-`SPLIT` loops are now closed. First, tightening `ADVECT` shaping for `SPINDRIFT` / `SQUALL` plus a light preset tune produced no material direct-view change at all on run75. Second, a stronger preset-only region retargeting pass on run77 proved that `Frozen Tundra` does not recover simply by moving support breakup off the walls, and `Storm Front` does not recover simply by reducing competing wall breakup around `ADVECT`. Those are useful closures because they shrink the remaining search space instead of reopening stale hopeful theories.
- The current `Storm Front` `ADVECT` track is now much more tightly bounded. A continuity-heavy `SQUALL` shader pass still failed on run79, and an even harder preset-side isolation (`BLEND_LERP` plus `REGION_WALLS` only) still failed on run81. That is enough evidence to stop pretending the remaining miss is just the wrong blend mode, tint, or sky mask. The current transport carrier surface still lacks the forceful wall-front behavior this weather brief needs.
- The first `ADVECT/BANK` pass also failed on run83. That closes the next obvious variant-level theory for `Storm Front`: even a banked transport profile does not make the storm front dominate direct view. The remaining miss now looks like a real surface-vocabulary gap, not more preset-side mask/blend churn.
- The refined `ADVECT/BANK` follow-up also failed on run85 after thickening the bank depth behavior and moving the finishing haze back to `SKY` only. That closes the obvious “maybe the bank is just too thin / too washed out” theory too. The current transport surface still cannot produce a scene-owning storm-front body in this brief.
- A benchmark-first harness now exists. The showcase app now contains a separate benchmark scene set, and `examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs` has already been validated on a deterministic paired run. Future EPU surface work should hit that benchmark gate before burning a full 12-preset showcase sweep.
- A single durable restart entrypoint now exists in `agent/start-here.md`, with the canonical long-term program plan in `agent/shared/10-program-runbook.md`. Future sessions should restart from those files instead of reconstructing the workflow from scattered logs or chat memory.
- The prompt pack and restart surface now explicitly include the live workbench path. `start-here`, the runbook, the validation playbook, and the job prompts all teach the same split: live workbench for fast local discovery, replay capture for authoritative promotion.
- The first qualitative benchmark baseline is now logged in `agent/shared/09-benchmark-log.md`. Current benchmark truth is: `Projection Bay` is the positive control; `Front Mass` is the strongest next surface blocker; `Frozen Bed` is second; `Open Horizon`, `Region Isolation`, and `Transport Sweep` also remain blocked and should be treated as benchmark-gated surface problems rather than full-showcase churn targets.
- The current capability audit and next-surface spec now live in `docs/architecture/epu-surface-expansion-plan.md`. Treat that document as the current system-level plan for EPU surface expansion work before proposing new opcodes or reopening broad outdoor churn.
- Target-specific implication from that audit:
  - `Combat Lab` should be judged and authored as world-integrated projection architecture, not HUD/UI. Keep motion on `GRID`, `VEIL/RAIN_WALL`, `LOBE`, and room-anchored emitters; treat `PORTAL/RECT` as a static bay/frame.
  - `Frozen Tundra` should carry its world read with ridge + `PLANE/WATER` ice-sheet structure first, then use `FLOW`, `PLANE/WATER`, `LOBE`, or another proven mover for sheen/drift. Do not let `SCATTER` or `BAND` become the primary motion carrier.
- Code-backed audit of the current 12 presets now shows `22/23` opcode families in use; `PATCHES` is the only remaining inventory gap in the current roster, while `SURFACE` and `MASS` are now present in the frozen/weather lane.
- Authored domain usage in current code now covers `DIRECT3D`, `AXIS_CYL`, `AXIS_POLAR`, and `TANGENT_LOCAL`.
- Newly closed from code: `SCATTER_SNOW`, `SCATTER_RAIN`, `TRACE_LIGHTNING`, `ATMO_RAYLEIGH`, `ATMO_MIE`, `ATMO_FULL`, `SILHOUETTE_MOUNTAINS`, `SILHOUETTE_WAVES`, and `PLANE_WATER`.
- Remaining thin spots confirmed from code: `SECTOR` beyond `BOX`, `CELL` beyond `SHATTER`, `APERTURE` beyond `ROUNDED_RECT`, `TRACE` beyond `CRACKS` / `LIGHTNING`, `ATMOSPHERE_ALIEN`, `PLANE_HEX`, `SILHOUETTE_INDUSTRIAL`, and broader `CELESTIAL` / `PORTAL` variety.
- Current shipping roster in code:
  - `Neon Metropolis`
  - `Sakura Shrine`
  - `Ocean Depths`
  - `Void Station`
  - `Desert Mirage`
  - `Enchanted Grove`
  - `Astral Void`
  - `Hell Core`
  - `Sky Ruins`
  - `Combat Lab`
  - `Frozen Tundra`
  - `Storm Front`

## Immediate Gaps

- `batch-wide scene readability failure`: `Neon Metropolis`, `Sakura Shrine`, `Desert Mirage`, `Enchanted Grove`, and `Sky Ruins` all miss their named-scene contract in the deterministic run08 review. Their core skyline, shrine, dune, grove, or ruin reads collapse into generic fog, speckle, or guide-line abstraction.
- `remaining interior readability failure`: `Void Station` is now the only current indoor preset still failing a critical scene-read contract. `Combat Lab` cleared on run29 and should be protected from regression rather than sent back into active fix churn.
- `Combat Lab` polish target under corrected brief: even though run29 closes the indoor proof-of-life gate, future work on preset 10 must stay world-integrated and non-HUD. If it is revisited, the projection read should come from room-anchored emitters, scan walls, and a rectangular test-field bay, not overlay-like cards.
- `space and depth failure`: `Astral Void` keeps stars but loses the required hero celestial hierarchy. `Ocean Depths` now has two failed repair directions on disk: the run19 `PATCHES` pass destroyed scene readability, and the run21 retreat restored underwater mood but still misses the trench-floor anchor, biolum focal point, and motion contract.
- `hell structure failure`: `Hell Core` preserves infernal mood, but the lava fissures are not the dominant read, so the scene still fails its own hard rule.
- `current Frozen Tundra blocker`: the run54-run73 sequence is now more precise. The organizer swaps and the first bounds/mask runtime fixes did not help much, but the `SPLIT` blend-width bug fix did materially improve the lower-scene read. `Frozen Tundra` is still failing, but it is no longer fair to call it only an abstract systemic wall. The remaining blocker is a narrower one: the preset still lacks a convincing outdoor horizon contract and stronger obvious motion even after the `SPLIT` fix and grounded-breakup retunes.
- `current Storm Front blocker`: the latest run73-run85 review chain says the structure side is much healthier after the `SPLIT` bug fix, but the transport surface is still not there. Runs `79`, `81`, `83`, and `85` together show that the issue is no longer just `SQUALL` shaping, blend mode, region mask, thin bank depth, or wall-haze washout. The current transport surface still lacks a forceful banked wall-front weather behavior that can own the horizon belt in direct view.
- `current Front Mass / Storm Front direction`: the benchmark lane has now moved past `ADVECT`-only thinking. `ADVECT/FRONT` was a useful bridge, but the active architecture is now `MASS` for the front body plus subordinate `ADVECT` for motion. The benchmark pair is deterministic, the body read is healthier, and the remaining blocker is scene ownership, occupancy, and motion strength rather than missing transport vocabulary alone.
- `current MASS-tightening closure`: the latest engine-side `MASS` occupancy/opacity tightening pass is now closed by a clean benchmark pair and direct image review. It made the front shelf slightly denser but did not materially change the failure class: `Front Mass` still reads as a pale soft wall with weak ownership. That means the next weather-body loop should not be another blind retry of the same density/fallback lever; it needs a stronger change in body-color/ownership behavior or benchmark composition.
- `current MASS ownership-bias closure`: the next benchmark-first follow-up is now also closed. Giving `MASS/SHELF` a darker core-bias path and reducing support washout by changing benchmark `ADVECT/FRONT` to `BLEND_LERP` still did not materially change the direct-review failure class. `Front Mass` remains a pale soft wall with weak ownership, so the next weather-body loop should not recycle this exact color/alpha-support lever either.
- `current MASS bank-profile closure`: the next `Front Mass` benchmark follow-up is now also closed. Removing the lightning confound, switching the body to `MASS/BANK`, and adding a two-stage inner-belly / shoulder profile still did not make the body own the frame. The latest benchmark stays weak in both direct background and reflective probe, so the next weather-body loop needs a more fundamental body/ownership rethink than another soft wall-band refinement.
- `current MASS shelf-body closure`: the latest `MASS/SHELF` local-body-hold pass is now also closed by a clean benchmark pair at `agent/runs/20260313-053823-front-mass-body-ownership/`. It materially darkened and densified the body, but the benchmark still reads as a broad shelf/wall rather than a scene-owning front, and the reflective probe still collapses toward a generic bright-top/dark-bottom split. That closes another concrete `MASS/SHELF` shaping lever without reopening the already-closed pale-shell theory.
- `current ADVECT_FRONT event closure`: the latest subordinate transport pass is now also closed by a clean benchmark pair at `agent/runs/20260313-055544-front-mass-body-ownership/`. It made the internal support motion slightly livelier, but the benchmark still lacks a distinct scene-owning front-body event and the probe still reads as a generic hemispheric split. That closes the first stronger-support-motion lever without changing the failure class.
- `current Front Mass authored-stack improvement`: the next authored benchmark pass is now logged at `agent/runs/20260313-060402-front-mass-body-ownership/`. It is the first recent Front Mass change that materially improves direct background ownership: the benchmark now shows a localized darker event instead of only a uniform shelf. It still fails because the reflective probe remains too generic and overall motion is still too weak, but this is a real directional improvement rather than another dead branch.
- `current Front Mass embedded-support closure`: the next authored stack-order follow-up is now also closed at `agent/runs/20260313-061236-front-mass-body-ownership/`. Reordering `ADVECT_FRONT` under `MASS_SHELF` preserves the darker localized background event, but it does not materially improve probe-side ownership. That closes the first embedded-support ordering theory without changing the failure class enough to promote.
- `current Front Mass probe-side shelf closure`: the next `MASS_SHELF` probe-facing pass is now also closed at `agent/runs/20260313-061721-front-mass-body-ownership/`. It regresses the best recent composition win by collapsing back toward a more generic hemispheric split without materially improving probe ownership. Treat this as a closed regression branch, not the new base state for Front Mass.
- `current outdoor rig-profile closure`: the shared showcase rig was worth testing, but it is no longer the leading excuse. The new per-benchmark outdoor rig profiles made the framing slightly fairer, yet both `Front Mass` and `Frozen Bed` still failed cleanly. That means the remaining blocker is still primarily EPU surface / scene-read quality, not just one oversized reflective probe or one overly indoor camera angle.
- `current bounds-authority split`: a new engine-side cleanup is now closed directionally. Bounds no longer have to use the same weight for structural region authority and visible paint, which means they can keep organizing the world envelope without forcing giant full-opacity color bands. This materially improved the outdoor benchmarks, so it was a real system fix rather than another no-op.
- `current Front Mass bounds-authority follow-up`: the new split softened the old pale-shell failure into a darker, more centered wall belt in both direct background and reflective probe, but it still does not become a coherent scene-owning front mass. The next weather-body loop should build on this cleaner envelope rather than reopening the old flat-band cleanup theory.
- `current Frozen Bed bounds-authority follow-up`: the same change materially improved broad outdoor structure. `Frozen Bed` no longer collapses into a nearly blank pale field; a broad cold ridge/bank now survives in both direct background and reflective probe. It still fails because the scene reads as snow shelf / cold berm rather than icy or crusted frozen floor, so the next loop should focus on frozen material identity on top of the improved envelope.
- `current Frozen Bed glaze-sheet closure`: the latest `SURFACE/GLAZE` continuity pass is now also closed by a clean wave-2 benchmark pair at `agent/runs/20260313-052446-frozen-bed-identity/`. It made the shelf read slightly more continuous, but the benchmark still presents as a pale soft berm with weak icy ownership in both direct background and reflective probe. That closes the first calmer-sheet/material lever without reopening full `Frozen Tundra` showcase churn.
- `current Frozen Bed pane-crust closure`: the latest pane/crust material pass is now also closed at `agent/runs/20260313-063533-frozen-bed-identity/`. It is coherent in shader intent, but the reviewed frames still collapse toward the same generic cold shelf split rather than a memorable icy/crusted bed. That closes another frozen-material lever without materially changing the failure class.
- `current unattended-loop progress`: the repo now has a deterministic replay-pair helper plus a restartable queue/documentation surface. That closes one real process gap: future long-running loops no longer need manual timestamp reconstruction just to prove benchmark/showcase determinism.
- `current unattended-loop progress 2`: the unattended queue runner now has real proof-of-life. It can execute targeted queue jobs, copy exact batch windows into `agent/runs/*`, write summary/review stub artifacts, and drain multiple runnable jobs with `--until-idle` / `--max-jobs` instead of stopping after one benchmark.
- `current unattended-loop progress 3`: one fresh benchmark replay bundle can now be treated as evidence for multiple benchmark targets when the replay and content state are identical. The latest `front-mass-body-ownership` run was intentionally reused to review both `Front Mass` and `Frozen Bed`, which closes one more wasteful process pattern: paying for duplicate captures just because the queue modeled the targets separately.
- `current live-workbench progress`: the machine-driven live authoring path is healthier now. The workbench editor correctly decodes `MASS`, and `tools/epu_workbench.py` can now summarize saved/live session state with `status` and enumerate benchmark/showcase scene ids with `list-scenes`. That closes one more process gap for overnight agent loops, even though it does not by itself improve outdoor benchmark quality.
- `current orchestration-pack progress`: the restart and orchestration prompts now explicitly describe this as a long-running validated program, not a one-shot task. `start-here`, the runbook, and the orchestrate pack all now state that the work should continue in validated waves until the full 20-scene program is complete or a real blocker is logged.
- `current workflow simplification`: determinism is now treated as solved infrastructure, not an active creative blocker. The validation playbook, unattended-loop doc, and capture pack now tell agents to focus routine waves on beautiful benchmark/showcase authoring and only reopen paired determinism checks if a fresh capture-path regression appears.
- `current process blocker`: before the benchmark harness, the workflow kept using the full showcase as both engine R&D and final art validation. That was too slow and too lossy. The new benchmark suite should now absorb capability discovery first so the full showcase loop is reserved for near-final validation and regression checks.
- `current review-calibration blocker`: some earlier review language leaned too hard toward literal scene clarity. The durable prompt pack now needs to keep the bar on shippable metaphor, ambient/reflection utility, and world-place read instead of accidental feature-film expectations.
- `current benchmark blocker`: the first qualitative benchmark review confirms that only `Projection Bay` currently passes its own benchmark contract. `Front Mass` is the clearest next EPU-surface target, `Frozen Bed` is the next outdoor/material target, and `Open Horizon` plus `Region Isolation` remain structural outdoor/readability failures that should be revisited from the benchmark lane, not rediscovered through full-roster showcase churn.
- `current Frozen Bed mass-bank closure`: the latest `Frozen Bed` benchmark follow-up is now also closed. Adding a `MASS/BANK` structural probe ahead of the `SURFACE` floor layers did not materially improve outdoor openness or frozen material identity; the benchmark still reads as a near-uniform pale field with a faint lower band and no convincing icy/crusted read in either direct background or reflective probe.
- `motion contract failure after determinism fix`: across most of the roster, the three deterministic run08 review frames show only tiny deltas or effectively frozen imagery. Screenshot nondeterminism is no longer the explanation; some of this is authored amplitude, but some of it is also capability mismatch from relying on non-phase-driven variants for motion.
- `PATCHES` still has no authored usage in current code.
- Remaining variant gaps from code: `SECTOR_TUNNEL`, `SECTOR_CAVE`, non-`SHATTER` `CELL`, non-`ROUNDED_RECT` `APERTURE`, `TRACE_LEAD_LINES`, `TRACE_FILAMENTS`, `ATMOSPHERE_ALIEN`, `PLANE_HEX`, `SILHOUETTE_INDUSTRIAL`, additional `CELESTIAL` bodies, and additional `PORTAL` shapes.
- 8 additional presets still need to be designed, implemented, captured, reviewed, and fixed as needed, but expansion should stay blocked until at least one outdoor preset from the current 12 can pass the review bar cleanly. `Combat Lab` is now the protected indoor proof-of-life pass; the gate is outdoor benchmark/showcase health, not more indoor expansion.

## Recommended Next Batch

Recommended next fix work:

- `Front Mass`
  Target: keep this as the first gating weather benchmark, but stop spending every immediate wave on it. The newest evidence says `ADVECT` alone was insufficient, `MASS` alone is also insufficient, and the right architecture is `MASS` as body plus `ADVECT` as subordinate motion. The occupancy/opacity pass, darker core-bias/support-washout pass, `MASS/BANK` bank-profile pass, `MASS/SHELF` local-body-hold pass, the stronger `ADVECT_FRONT` support-motion pass, the first embedded-support ordering pass, and now the probe-side `MASS_SHELF` pass are all closed. The best remaining Front Mass state is still the authored-stack run with a localized darker background event; probe ownership remains the gating miss. Keep this lane alive, but pull the frozen-material lane forward in parallel instead of burning every wave on Front Mass alone.
- `Frozen Bed`
  Target: keep this as the second benchmark-gated surface track and pull it forward more aggressively. The current floor identity is still too close to generic gloss/water or cold shelf, even after the calmer `SURFACE/GLAZE` sheet pass and the newer pane/crust material pass. The next lever should move beyond current glaze/crust tuning and target a clearer frozen-place read through authored composition and stronger probe-side material response before any `Frozen Tundra` promotion. Do not reopen `Frozen Tundra` showcase churn until the benchmark lane materially improves again.
- `Storm Front`
  Target: only promote after `Front Mass` materially improves again. Stay on the `MASS + ADVECT` track instead of reopening `SQUALL` / `BANK` churn. The remaining miss is body occupancy/contrast and scene ownership.
- `Frozen Tundra`
  Target: only promote after `Frozen Bed` and related outdoor-structure benchmark work materially improve. The current blocker is no longer just preset-only microtuning.
- `Combat Lab`
  Target: keep the run29 pass state and protect it from regression, not as an active failure lane. Treat preset 10 as a room-integrated projection bay, not a HUD showcase. If revisited for polish, keep cadence on `GRID`, `VEIL/RAIN_WALL`, `LOBE`, and room-anchored emitters while using `PORTAL/RECT` only as a static frame.

Reason:

- determinism remains closed through run84/run85, so the blocker is still content-read quality rather than capture drift
- `Combat Lab` now closes the indoor proof-of-life requirement on run29, but the corrected brief still matters because future polish/review must stop drifting back toward HUD/UI language
- `Frozen Tundra` is still the explicit outdoor proof-of-life candidate, but the newest run54-run67 evidence now says the failure is drifting from "bad preset" toward "real EPU outdoor far-field / region-read gap" even after targeted runtime fixes
- `Storm Front` still matters for weather coverage, and the latest loops show the new opcode path is healthier than the old `VEIL` path even though the current shaping still leaves an obviously synthetic weather body
- the newest body-carrier loops close one more false hope cleanly: adding `MASS` was the right structural direction, but the first two deterministic showcase promotions still failed, so the remaining blocker is not “we just needed one more opcode.” It is now specifically the combined `MASS + ADVECT` read quality in direct view.
- `MOTTLE` already closes the generic anti-flat-fill need, so future outdoor work should not reopen "we need basic noise" unless the missing behavior is clearly stronger than current support breakup
- `Frozen Tundra` now has six separate evidence-backed lessons: transport-only tuning was not enough, frozen-surface/material-response work was necessary but not sufficient, a second early bounds layer materially changed the failure class, repeated post-run53 organizer swaps (`SPLIT/TIER`, darker `SPLIT/TIER`, `UP`-oriented `SPLIT/FACE`) still fail to recover a real outdoor read, correcting sequential bounds composition still does not rescue the preset, and even boosting single-region feature masks does not materially recover direct-view floor identity
- the outdoor rig-profile rerun closes a process theory too: even with a fairer pullback and slightly smaller probe, both active outdoor benchmark lanes still fail. The next move should therefore be deeper surface/scene-vocabulary work, not more camera-only churn.
- the new bounds-authority split closes another process and engine theory too: some of the outdoor flat-band / weak-feature problem really was caused by bounds having to use one weight for both visible paint and structural authority. Fixing that did not solve the benchmarks, but it materially improved both active outdoor lanes and narrowed the next work more honestly.
- current loop health depends on matching briefs and authoring to the carriers that actually animate/read, then updating the shared docs as soon as the surface changes so the prompt pack does not rot
- review must keep using both direct background and reflective-probe reads as first-class evidence. The latest benchmark closures are stronger specifically because the failure now survives in both, not just one.

## Audit Drift

- `agent/shared/00-charter.md` still lists the full-sweep replay comment/UI-toggle issues as known drift, but the script on disk is now synced to 12 presets and toggles UI off first.
- `Neon Metropolis` comments describe a `SCATTER/RAIN` layer, but current code uses `VEIL_RAIN_WALL` plus `FLOW`; `SCATTER_RAIN` is now covered only by `Storm Front`.
- `Hell Core` authors `TRACE/CRACKS` with `DOMAIN_DIRECT3D`, while `src/constants.rs` documents `TRACE` as a domain-tagged family intended for `AXIS_CYL`, `AXIS_POLAR`, or `TANGENT_LOCAL`.

## Exit Checklist

- [ ] 20 presets exist in code
- [ ] every preset has been screenshot-reviewed
- [x] full-sweep replay script matches the real preset count
- [x] full-sweep replay script toggles UI off
- [ ] all major opcodes are covered
- [x] all domains are covered
- [ ] variant gaps are either closed or explicitly justified
- [ ] no preset is a weak duplicate
- [ ] no preset has a known technical flaw
- [ ] shared logs and planning docs reflect final reality
