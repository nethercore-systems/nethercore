# Validation Playbook

Use this file for the actual build, replay, screenshot, and review loop.

## Final Validation Principle

Final validation must use the real player with `--replay`.

Do not use `tools/nether-cli replay execute` as a substitute for final screenshot validation.

The authoritative showcase and benchmark captures are not flat background plates:

- the scene contains a large reflective hero probe mesh in the foreground
- the EPU is drawn after 3D geometry and only fills background pixels
- review must judge both the direct background read and the probe reflection read

In current default replay captures, that probe is the sphere unless the replay explicitly changes shape.

EPU is not a literal feature-film skybox renderer. Validate it as a metaphor-first fantasy-console environment signal:

- strong enough to ship as ambient lighting, reflection/IBL input, and world-integrated background mood
- clear enough that the intended place metaphor survives direct view and reflection
- not expected to render every prompt noun as a literal prop or perfectly clear panoramic matte painting

## Minimum Loop

1. Inspect current code and replay scripts.
2. Re-check opcode/domain capability reality in code before assuming a layer can carry motion, horizon structure, or world-space depth.
3. If the next task is rapid local discovery, scripted sweeps, or high-frequency tuning on one machine, use the live workbench first.
4. If the change touched EPU runtime/opcode/capability behavior, run the benchmark replay first.
5. Make a small batch of changes.
6. Run `cargo fmt`.
7. Rebuild the correct binaries for the scope of change.
8. Validate replay script syntax.
9. Run the real player with `--replay`.
10. Inspect the generated PNGs directly.
11. Log findings.
12. Only then promote healthy changes to the full showcase sweep.
13. Fix and repeat until pass.

## Build Scope Rules

- `cargo ba` is not sufficient for showcase validation. It is only a workspace native build.
- `cargo xtask build-examples` is for bulk installing examples into the library games directory. Use it when validating the installed library set, not as the default single-example replay step.
- If any engine, renderer, shader generation, player, or CLI code changed, run `cargo dev` first so `target/release/nethercore-zx.exe` and the release CLI are fresh.
- If only the showcase example changed, rebuild the example ROM explicitly with the CLI before capture.
- Final screenshot validation should prefer an explicit player binary path over launcher discovery so the exact executable is known.

## Preferred Commands

If engine or CLI code changed:

```bash
cargo dev
```

For single-example showcase validation from the repo root:

```bash
cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase
cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs
target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs
```

For benchmark-first validation after EPU runtime/opcode changes:

```bash
cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase
cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs
target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs
```

For live local tuning and export from the repo root:

```bash
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench
python tools/epu_workbench.py session
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 0
python tools/epu_workbench.py capture --label benchmark-0-baseline
```

For unattended deterministic benchmark pairs from the repo root:

```bash
python tools/tmp/run_epu_replay_pair.py C:\Users\rdave\AppData\Roaming\Nethercore\data\screenshots target\release\nethercore-zx.exe examples\3-inspectors\epu-showcase\epu-showcase.nczx examples\3-inspectors\epu-showcase\screenshot-benchmarks-anim3.ncrs 18 --cwd D:\Development\nethercore-project\nethercore
```

For queued unattended benchmark/showcase runs from the repo root:

```bash
python tools/tmp/run_epu_loop_queue.py
```

To drain multiple runnable queue items in one unattended session:

```bash
python tools/tmp/run_epu_loop_queue.py --until-idle --max-jobs 2
```

For library-wide example installation:

```bash
cargo xtask build-examples
```

Record which path was used. Do not describe a capture as authoritative if the player executable is ambiguous.

## Replay Script Rules

- Full-sweep scripts must match the real preset count.
- Benchmark-first scripts should stay smaller and capability-focused; do not silently turn them into another full showcase sweep.
- Full-sweep scripts must toggle the UI off before capture.
- Default expectation is 3 screenshots per preset at spaced frames.
- Use focused scripts for suspicious presets, different shapes, or extra animation checks.

## Screenshot Review Rules

- Clean screenshots only for quality review.
- Compare all captured frames for animated presets.
- Judge readability as shippable procedural world art. The bar is strong scene metaphor, useful region/light separation, and convincing world integration, not literal prop-perfect illustration.
- Do not fail a preset only because it is abstract. Fail it when the intended place metaphor, ambient/reflection utility, or world-anchored motion does not survive.
- Remember that the reflective probe is part of the contract, not a distraction. A scene can benefit from strong reflection utility, but it still fails if the intended world event only survives on the probe and disappears from the direct background when the brief requires a direct-view read.
- All showcase presets are expected to be visibly animated in review. Theoretical animation or tiny deltas fail.
- Treat bounds as the world envelope and feature layers as the main detail/motion carriers, but do not assume slot index enforces that layout.
- Treat `ANIM_SPEEDS` hypotheses as variant-specific. Before blaming capture or reviewers, verify that the authored variant actually consumes `param_d` as phase.
- Before escalating to an engine bug, rule out capability mismatch and unsupported authoring assumptions in the current shader surface. Coverage or metadata names alone are not proof.
- If the same failure survives multiple deterministic passes, log whether the blocker is a content miss, an engine/render bug, or an opcode-surface/tooling gap. Do not collapse all repeated failures into `suspected engine bug`.
- Known limits that should block bad assumptions: `APERTURE` is structural, `TRACE/LIGHTNING` is a static strike shape, `SCATTER` is seed-driven shimmer, `PORTAL/RECT` is a static local frame, and `BAND` is support-only rather than a general horizon scroller.
- Strong motion/world carriers in current practice: `FLOW`, `GRID`, `LOBE`, `VEIL/RAIN_WALL`, `PLANE/WATER`, and `PORTAL/VORTEX`.
- Visible artifacts or obvious rendering errors are automatic fails, even if the overall art direction is strong.
- If a preset can be made to loop cleanly with phase-driven motion, prefer that over one-shot or chaotic motion.
- Treat loopability as a validation target, not an assumption. When `param_d` or phase-driven motion is authored to loop, verify the result and escalate any shared failure as a likely engine or render-path bug.
- Baseline expectation is deterministic screenshots across repeated runs from the same rebuilt binaries and ROM. If repeated runs diverge, stop and log a renderer or capture bug.
- Treat looping or repeated patterning as a defect category and name it directly in review notes.
- Treat giant flat bands or broad solid-color fields as a defect category unless the brief explicitly calls for a banded feature read.
- If an artifact looks engine-driven or persists across presets, opcodes, or domains, log a suspected EPU/rendering bug and stop content-only churn until isolated.
- If a preset looks borderline, treat it as failing until proven otherwise.
- For benchmark or showcase determinism checks, compare content-matched capture windows. Do not blindly compare the latest `72` or `36` PNGs if benchmark and full-showcase runs were interleaved.
- `tools/tmp/compare_epu_screenshot_batches.py` defaults to the latest two batches only. Use `--a-first`, `--b-first`, and `--batch-size` when captures are interleaved or when reviewing benchmark windows.
- `tools/tmp/run_epu_replay_pair.py` is the preferred unattended path for deterministic pair capture because it runs both replays sequentially and emits the exact batch windows automatically.
- `tools/tmp/run_epu_loop_queue.py` is the preferred unattended path when you want queue progression plus a durable `agent/runs/*` artifact bundle instead of a one-off pair.
- `tools/epu_workbench.py` is the preferred local path when the goal is rapid EPU discovery, scripted sweeps, or candidate export rather than authoritative replay promotion.
- Do not declare a pass from code inspection alone.

## Screenshot Location

The player saves screenshots according to `core/src/capture.rs`.

For this example, the sanitized filename prefix should begin with:

```text
epu_showcase_screenshot_
```

Locate the newest batch and inspect those PNGs directly.

## Review Scorecard

Every reviewed preset should be scored on:

- `brief / metaphor fidelity`
- `visual identity`
- `spatial envelope / hierarchy`
- `ambient / IBL / reflection read`
- `metaphor / shippability fit`
- `technical cleanliness`
- `animation quality`
- `loop quality or loopability readiness`
- `novelty versus roster`
- `verdict`

Do not pass a preset if any category is below 9.
