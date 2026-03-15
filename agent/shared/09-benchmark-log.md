# Benchmark Log

Use this file to record benchmark-suite runs separately from the full showcase log.

## Format

For each benchmark pass, record:

- date
- run id or baseline id
- benchmark replay script used
- build preconditions
- determinism result
- which benchmark was the intended target
- what materially improved or failed to improve
- whether the change is promoted to a full showcase pass

Recommended benchmark review fields:

- `benchmark`
- `must-read result`
- `hard-fail trigger seen`
- `motion read`
- `artifact or flat-band notes`
- `promotion decision`

Benchmark review should judge metaphor/place strength and shipping utility for ambient/reflection/direct-view use, not literal prop-perfect clarity.

## Current State

- benchmark harness created on `2026-03-12`
- runtime scenes live in `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
- first dedicated replay script lives in `examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`

## Current Benchmark Truth

- `Projection Bay` remains the positive-control benchmark pass.
- `Front Mass` remains blocked, but the old hard seam/panel family is now materially reduced. The active architecture is still `MASS` for body plus `ADVECT` for subordinate motion, and the remaining blocker is scene ownership plus motion strength.
- `Frozen Bed` remains blocked, but the old direct seam-wedge class is now materially reduced. The remaining blocker is frozen-place identity rather than the earlier wrap artifact family.
- `Open Horizon`, `Region Isolation`, and `Transport Sweep` are still blocked and should be treated as system/benchmark work first, not rediscovered through full-showcase churn.

## Runs

### 2026-03-12 benchmark baseline

- replay script: `examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- build preconditions:
  - `cargo fmt`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- output:
  - 18 screenshots per run
  - first run timestamps: `2026-03-12 06:27:45` through `06:27:51`
  - second run timestamps: `2026-03-12 06:27:55` through `06:28:02`
  - paired hash comparison across the newest 36 PNGs found `0` mismatches out of `18` per-frame pairs
- smoke result:
  - benchmark mode toggled correctly from the replay
  - benchmark scenes cycled correctly
  - harness is deterministic enough to use as the first gate for future EPU surface work
- promotion decision:
  - benchmark harness itself is promoted and should be used before future full-showcase sweeps when the EPU surface changes
  - no benchmark quality verdicts are logged yet; this run was a harness smoke baseline

### 2026-03-12 benchmark baseline review

- baseline id:
  - deterministic smoke pair at `2026-03-12 06:27:45..06:28:02`
- benchmark:
  - `Open Horizon`
  - must-read result: fail
  - hard-fail trigger seen: yes; collapses into a pale chamber/bowl instead of open air
  - motion read: weak and not meaningfully scene-carrying
  - artifact or flat-band notes: broad pale wash dominates; lower-ground contract is too weak
  - promotion decision: blocked
- benchmark:
  - `Region Isolation`
  - must-read result: fail
  - hard-fail trigger seen: yes; floor/wall separation is not cleanly proven in direct view
  - motion read: not relevant as the primary blocker
  - artifact or flat-band notes: side wall dominates; floor proof is too weak and the scene feels smeared
  - promotion decision: blocked
- benchmark:
  - `Projection Bay`
  - must-read result: pass
  - hard-fail trigger seen: no
  - motion read: adequate for the benchmark purpose
  - artifact or flat-band notes: strongest current proof-of-life; reads as a world-integrated bay rather than HUD cards
  - promotion decision: promoted as the positive-control benchmark
- benchmark:
  - `Transport Sweep`
  - must-read result: fail
  - hard-fail trigger seen: yes; broad transport motion does not read as a convincing moving sheet
  - motion read: effectively no useful direct-view motion across the reviewed frames
  - artifact or flat-band notes: pale field with a magenta lower crescent; current carrier still collapses toward weak synthetic slabs
  - promotion decision: blocked
- benchmark:
  - `Front Mass`
  - must-read result: fail
  - hard-fail trigger seen: yes; no coherent scene-owning front body appears
  - motion read: lightning accent changes, but the body itself does not dominate
  - artifact or flat-band notes: still reads as a pale synthetic shell with weak wall ownership
  - promotion decision: blocked
- benchmark:
  - `Frozen Bed`
  - must-read result: fail
  - hard-fail trigger seen: yes; frozen identity is too weak and still drifts toward generic gloss/water behavior
  - motion read: minimal and not the deciding issue
  - artifact or flat-band notes: floor read is too faint; outdoor openness also remains weak around it
  - promotion decision: blocked
- benchmark baseline conclusion:
  - `Projection Bay` is the current positive control
  - the strongest next surface blockers are `Front Mass` first and `Frozen Bed` second
  - `Open Horizon` and `Region Isolation` remain structural/readability blockers that should be revisited alongside the same outdoor surface work rather than as isolated preset problems

### 2026-03-12 front-mass transport follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - added `ADVECT/FRONT` as a new `ADVECT` front-body variant
  - first benchmark wiring kept the carrier too constrained and did not materially improve the read
  - second benchmark wiring changed the front-body layer to `REGION_SKY | REGION_WALLS` with `BLEND_MULTIPLY`, which materially improved direct-view wall-front ownership
- build preconditions:
  - `cargo fmt`
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- determinism notes:
  - the overlapping dual-launch benchmark attempt at `2026-03-12 06:58:*` is invalid for pairwise comparison and should not be treated as a real determinism failure
  - the clean sequential pair at `2026-03-12 06:59:08..06:59:14` and `2026-03-12 06:59:36..06:59:43` produced `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: improved, but not a clean pass yet
  - hard-fail trigger seen: softened; the front body now reads as a dark wall/shelf instead of disappearing into a pale shell
  - motion read: lightning still carries most of the obvious frame-to-frame event read
  - artifact or flat-band notes: the storm body now owns the upper wall belt directionally, but the overall scene is still not yet a decisive 10/10 scene-owning front
  - promotion decision: promoted directionally to `Storm Front` showcase exploration, but not closed as a benchmark pass

### 2026-03-12 front-mass body-carrier follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - added `MASS` at `0x17` as a separate abstract body-carrier family with `BANK`, `SHELF`, `PLUME`, and `VEIL`
  - first benchmark promotion replaced the old primary body layer with `MASS/SHELF`
  - second benchmark promotion restored `ADVECT/FRONT` as subordinate motion under `MASS/SHELF`, which matches the intended architecture better than asking either opcode to do both jobs alone
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- determinism notes:
  - `MASS/SHELF` benchmark pair at `2026-03-12 19:00:19..19:00:28` and `19:00:36..19:00:43` was clean once compared against the correct content-matched pair
  - `MASS/SHELF + ADVECT/FRONT` benchmark pair at `2026-03-12 19:01:57..19:02:05` and `19:02:33..19:02:41` was also clean once compared against the correct content-matched pair
  - the intermediate `MismatchCount=3` reads in this block were invalid cross-state comparisons and should not be treated as real determinism failures
- benchmark review:
  - `Front Mass`
  - must-read result: improved structurally, still fail
  - hard-fail trigger seen: yes; the body is more coherent than the old `ADVECT`-only shelf, but it still does not become a decisive scene-owning front
  - motion read: stronger once `ADVECT` returns as support, but still too restrained to count as a benchmark pass
  - artifact or flat-band notes: the architecture is now clearer (`MASS` for body, `ADVECT` for motion), but the carrier still washes too pale and too soft in direct view
  - promotion decision: promoted experimentally to `Storm Front`, but benchmark remains blocked

### 2026-03-12 front-mass mass-tightening follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - tightened `MASS` occupancy and opacity in `15_mass.wgsl`
  - reduced the soft fallback shell in `MASS/BANK` and `MASS/SHELF`
  - increased alpha scaling so denser body regions would own more of the scene
- build preconditions:
  - `python -m py_compile tools/tmp/compare_epu_screenshot_batches.py`
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `target/release/nethercore-zx.exe examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- determinism notes:
  - the first `21:20` benchmark pair was invalid because both replays were launched in parallel and their screenshot windows overlapped
  - the clean sequential pair at `2026-03-12 21:21:13..21:21:20` and `21:21:30..21:21:36` produced `0` mismatches across all `18` frame pairs using explicit `--a-first` / `--b-first` comparison windows
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the front body remains a pale soft wall rather than a decisive scene-owning mass
  - motion read: minimal; the three reviewed frames still change too little to count as a benchmark pass
  - artifact or flat-band notes: the shelf is slightly denser, but the benchmark still reads as a soft gray wall with weak ownership rather than a front that materially shapes the scene and reflection read
  - promotion decision: blocked; do not promote this exact `MASS` tightening pass into `Storm Front`

### 2026-03-12 front-mass ownership-bias follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - gave `MASS/SHELF` and `MASS/BANK` a stronger dark-core color/alpha bias in `15_mass.wgsl`
  - changed `Front Mass` support `ADVECT/FRONT` from `BLEND_SCREEN` to `BLEND_LERP` so the support layer would stop washing the same wall band brighter
  - replaced the manual sequential pair procedure with `tools/tmp/run_epu_replay_pair.py`
- build preconditions:
  - `python -m py_compile tools/tmp/run_epu_replay_pair.py`
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_replay_pair.py C:\Users\rdave\AppData\Roaming\Nethercore\data\screenshots target\release\nethercore-zx.exe examples\3-inspectors\epu-showcase\epu-showcase.nczx examples\3-inspectors\epu-showcase\screenshot-benchmarks-anim3.ncrs 18 --cwd D:\Development\nethercore-project\nethercore`
- determinism notes:
  - the unattended pair runner produced a clean pair at `2026-03-12 21:40:48..21:40:54` and `21:40:56..21:41:02`
  - the helper reported `0` mismatches across all `18` frame pairs and emitted the exact batch windows directly
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the front body still reads as a pale soft wall rather than a decisive scene-owning mass
  - motion read: still too weak across the three reviewed frames to count as a benchmark pass
  - artifact or flat-band notes: the darker core-bias path and lower washout support did not materially change the failure class in direct review; this closes a second concrete weather-body branch beyond density-only tuning
  - promotion decision: blocked; do not promote this ownership-bias pass into `Storm Front`

### 2026-03-12 front-mass bank-profile follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - removed the `TRACE/LIGHTNING` accent from `Front Mass` so the body had to own the frame without help
  - changed the benchmark composition from `MASS/SHELF` to `MASS/BANK`
  - added a two-stage inner-belly / shoulder profile to `MASS_BANK` in `15_mass.wgsl`
  - executed the run through the unattended queue runner so the pair, copied screenshots, and review stub all landed under `agent/runs/20260312-220838-front-mass-body-ownership/`
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the queue runner produced a clean pair at `2026-03-12 22:12:35..22:12:42` and `22:12:43..22:12:50`
  - the copied run bundle under `agent/runs/20260312-220838-front-mass-body-ownership/` contains the exact `batch_a` / `batch_b` windows plus `manifest.json`, `summary.md`, and pair output
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the front body still does not become a decisive scene-owning wall/front mass
  - motion read: too weak; the three reviewed frames still do not show a strong front-body event
  - artifact or flat-band notes: removing lightning successfully proved the body is not being carried by an accent, but the benchmark still collapses into a pale soft wall in direct background and a generic light-top/dark-bottom split on the reflective probe; the two-stage `MASS_BANK` profile did not materially change that failure class
  - promotion decision: blocked; do not promote this pass into `Storm Front`

### 2026-03-12 frozen-bed mass-bank probe follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - replaced the old sky-only support emphasis with a `MASS/BANK` wall-body structural probe inserted ahead of the `SURFACE` floor layers
  - kept the frozen floor on the existing `SURFACE/GLAZE` + `SURFACE/CRUST` path so the test isolated whether stronger outdoor body/ownership could rescue the lane
  - executed the run through the unattended queue runner so the pair, copied screenshots, and review stub all landed under `agent/runs/20260312-222208-frozen-bed-identity/`
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- determinism notes:
  - the queue runner produced a clean pair at `2026-03-12 22:26:10..22:26:16` and `22:26:18..22:26:24`
  - the copied run bundle under `agent/runs/20260312-222208-frozen-bed-identity/` contains the exact `batch_a` / `batch_b` windows plus `manifest.json`, `notes.md`, and pair output
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not show an icy bed or crusted frozen floor in direct background
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: the new `MASS/BANK` structural probe did not materially improve outdoor openness or frozen identity; the background remains a near-uniform pale field with only a faint lower band, and the reflective probe still does not carry a convincing icy/crusted material read
  - promotion decision: blocked; do not promote this pass into `Frozen Tundra`

### 2026-03-12 outdoor rig-profile rerun

- benchmark targets:
  - `Front Mass`
  - `Frozen Bed`
- changes:
  - added per-benchmark outdoor rig profiles in the showcase harness so `Open Horizon`, `Transport Sweep`, `Front Mass`, and `Frozen Bed` use a slightly wider pullback, lower elevation, and slightly smaller reflective probe than the default indoor-oriented showcase rig
  - re-queued both outdoor benchmark lanes and drained them sequentially with `python tools/tmp/run_epu_loop_queue.py --until-idle --max-jobs 2`
- build preconditions:
  - `cargo fmt`
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --until-idle --max-jobs 2`
- determinism notes:
  - the unattended queue drain ran `front-mass-body-ownership` first and `frozen-bed-identity` second
  - the `Front Mass` rerun at `agent/runs/20260312-224135-front-mass-body-ownership/` produced a clean pair at `2026-03-12 22:41:39..22:41:45` and `22:41:48..22:41:54`
  - the `Frozen Bed` rerun at `agent/runs/20260312-224155-frozen-bed-identity/` produced a clean pair at `2026-03-12 22:41:59..22:42:05` and `22:42:08..22:42:14`
  - both pair results reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the scene is still a pale soft wall/bank rather than a coherent scene-owning front mass
  - motion read: still too weak to count as a benchmark pass
  - artifact or flat-band notes: the wider pullback made the framing slightly fairer, but the key read still survives only as a weak direct-background hint and still collapses to a generic bright-top/dark-bottom split on the reflective probe
  - promotion decision: blocked; do not promote this rerun into `Storm Front`
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not show an icy bed or crusted frozen floor in direct background
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: the new rig did not materially recover frozen-place identity; the scene remains a near-uniform pale field with a faint lower band, and the reflective probe still does not prove a convincing icy/crusted material read
  - promotion decision: blocked; do not promote this rerun into `Frozen Tundra`

### 2026-03-12 bounds-authority split follow-up

- benchmark targets:
  - `Front Mass`
  - `Frozen Bed`
- changes:
  - decoupled bounds structural authority from visible paint in the EPU runtime so bounds layers can keep full region retagging while no longer forcing full-opacity color bands
  - retuned the outdoor benchmark bounds alpha so the weather-body and frozen-surface features could own more of the scene without losing the world envelope
  - reused one shared benchmark replay bundle to review both `Front Mass` and `Frozen Bed` instead of paying for two identical replay captures
- build preconditions:
  - `cargo fmt`
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the shared run bundle lives at `agent/runs/20260312-230851-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-12 23:09:02..23:09:06` local time
  - batch B timestamps span `2026-03-12 23:09:08..23:09:14` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: improved slightly, still fail
  - hard-fail trigger seen: yes; the scene is darker and more centered, but still does not become a coherent scene-owning front body
  - motion read: still too weak for a benchmark pass
  - artifact or flat-band notes: the old pale-shell failure softened into a darker wall belt, but both direct background and reflective probe still collapse toward a generic hemispheric split rather than a dominant weather mass
  - promotion decision: blocked; do not promote this follow-up into `Storm Front`
- benchmark review:
  - `Frozen Bed`
  - must-read result: improved structurally, still fail
  - hard-fail trigger seen: yes; the scene no longer collapses into a near-uniform pale field, but it still does not read as an icy bed or crusted frozen floor
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: a broad cold ridge/bank now survives in both direct background and reflective probe, which is real progress, but the material identity is still snow shelf / cold berm rather than frozen-surface read
  - promotion decision: blocked; do not promote this follow-up into `Frozen Tundra`

### 2026-03-13 frozen-bed glaze-sheet follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - fixed live-workbench `MASS` decode in `nethercore-zx/src/debug/epu_panel/editor.rs` so local discovery no longer flattens `MASS` layers to `NOP`
  - added workbench CLI `status` and `list-scenes` support in `tools/epu_workbench.py` so overnight loops can reconnect and inspect scene ids from artifacts without guesswork
  - retuned `SURFACE/GLAZE` in `nethercore-zx/shaders/epu/features/14_surface.wgsl` toward a calmer sheet-like icy variation with higher continuous coverage
- build preconditions:
  - `cargo fmt`
  - `cargo test -p nethercore-zx --lib`
  - `python -m py_compile tools/epu_workbench.py`
  - `python tools/epu_workbench.py status --artifacts-dir tmp/epu-workbench-live`
  - `python tools/epu_workbench.py list-scenes --artifacts-dir tmp/epu-workbench-live`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- determinism notes:
  - the first queue rerun at `agent/runs/20260313-051613-frozen-bed-identity/` was useful as a fresh baseline but not treated as the final post-integration verdict because the wave-2 worker slice landed concurrently
  - the post-integration queue rerun at `agent/runs/20260313-052446-frozen-bed-identity/` is the authoritative wave-2 pair
  - batch A timestamps span `2026-03-13 05:29:30..05:29:37` local time
  - batch B timestamps span `2026-03-13 05:29:39..05:29:45` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not read as an icy bed or crusted frozen floor in either direct background or reflective probe
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: the calmer glaze path holds the shelf together more continuously, but the result is still a pale soft frozen berm with weak icy ownership rather than a shipping-strength frozen surface
  - promotion decision: blocked; do not promote this follow-up into `Frozen Tundra`

### 2026-03-13 front-mass shelf-body hold follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - reshaped `MASS/SHELF` in `nethercore-zx/shaders/epu/features/15_mass.wgsl` to replace the old underbelly-driven hemispheric split with a more local shelf-body hold
  - strengthened belly/shoulder occupancy, reduced rim influence, and pushed alpha/density toward a denser abstract front body on top of the improved bounds-authority envelope
- build preconditions:
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative post-change run bundle is `agent/runs/20260313-053823-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-13 05:42:19..05:42:25` local time
  - batch B timestamps span `2026-03-13 05:42:27..05:42:34` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: improved contrast, still fail
  - hard-fail trigger seen: yes; the front body is darker and denser, but it still reads as a broad shelf/wall instead of a decisive scene-owning front
  - motion read: still too weak across the reviewed frames to count as a benchmark pass
  - artifact or flat-band notes: the old pale soft wall is reduced, but the reflective probe still collapses toward a generic bright-top/dark-bottom split and the direct background still lacks a distinct weather-body event
  - promotion decision: blocked; do not promote this follow-up into `Storm Front`

### 2026-03-13 front-mass advect-front event follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - retuned `ADVECT_FRONT` support behavior in `nethercore-zx/shaders/epu/features/13_advect.wgsl`
  - increased internal pulse/event behavior, pushed support alpha harder, and reduced the remaining soft-rim bias so the subordinate transport layer could read as a clearer front-body event without becoming the primary body
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260313-055544-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-13 05:59:36..05:59:43` local time
  - batch B timestamps span `2026-03-13 05:59:45..05:59:51` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not produce a distinct scene-owning weather-body event
  - motion read: slightly livelier internally, but still too weak across the reviewed frames to count as a benchmark pass
  - artifact or flat-band notes: the front remains a broad shelf/wall read, and the reflective probe still collapses toward a generic hemispheric split rather than a strong front-body read
  - promotion decision: blocked; do not promote this follow-up into `Storm Front`

### 2026-03-13 front-mass authored-stack follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - changed the authored `Front Mass` benchmark stack in `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
  - switched the body layer to `MASS_SHELF` with a darker multiply path
  - moved `ADVECT_FRONT` later in the stack on `SKY | WALLS` with a stronger authored support profile so the live benchmark itself, not just shader-side support logic, carried the next lever
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260313-060402-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-13 06:07:58..06:08:04` local time
  - batch B timestamps span `2026-03-13 06:08:06..06:08:12` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: improved, still fail
  - hard-fail trigger seen: softened; the direct background now shows a more localized darker event instead of only a uniform shelf
  - motion read: still too weak overall to count as a benchmark pass
  - artifact or flat-band notes: this is the strongest recent composition improvement for direct background ownership, but the reflective probe still collapses toward a generic bright-top/dark-bottom split and the full scene still does not read as a decisive front-body event
  - promotion decision: blocked; do not promote this follow-up into `Storm Front` yet

### 2026-03-13 front-mass embedded-support follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - reordered the authored `Front Mass` stack in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` so `ADVECT_FRONT` sits under the owning `MASS_SHELF` layer instead of reading as a later layer on top
  - increased the authored Front Mass animation emphasis on the embedded-support path while keeping the body on a darker multiply shelf
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260313-061236-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-13 06:12:44..06:12:51` local time
  - batch B timestamps span `2026-03-13 06:12:53..06:12:59` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not achieve probe-side front-body ownership
  - motion read: still too weak overall to count as a benchmark pass
  - artifact or flat-band notes: the direct background keeps the localized darker event from the previous authored-stack pass, but embedding `ADVECT_FRONT` under `MASS_SHELF` does not materially improve the reflective probe; it still reads mostly as a generic hemisphere with a dark notch rather than a convincing front body
  - promotion decision: blocked; do not promote this follow-up into `Storm Front`

### 2026-03-13 front-mass probe-side shelf follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - retuned `MASS_SHELF` in `nethercore-zx/shaders/epu/features/15_mass.wgsl` to emphasize a tighter shelf band and crown trim intended to improve probe-side ownership and reduce the generic hemispheric split
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260313-061721-front-mass-body-ownership/`
  - batch A timestamps span `2026-03-13 06:21:10..06:21:14` local time
  - batch B timestamps span `2026-03-13 06:21:16..06:21:22` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark falls back toward a generic bright-top/dark-bottom split
  - motion read: still too weak overall to count as a benchmark pass
  - artifact or flat-band notes: this probe-side shelf pass erases the best recent directional win in direct background and does not recover a stronger reflective-probe read, so it should be treated as a closed regression branch rather than a base for further tuning
  - promotion decision: blocked; do not promote this follow-up into `Storm Front`

### 2026-03-13 frozen-bed pane-crust follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - retuned `SURFACE/GLAZE` and `SURFACE/CRUST` in `nethercore-zx/shaders/epu/features/14_surface.wgsl` toward pane-like glass, harder seams, and more crusted fracture contrast
  - retuned the authored `Frozen Bed` benchmark composition in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` so the floor layers own more of the read and the wall bank / sky spindrift compete less with the bed
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260313-063533-frozen-bed-identity/`
  - batch A timestamps span `2026-03-13 06:39:25..06:39:31` local time
  - batch B timestamps span `2026-03-13 06:39:33..06:39:39` local time
  - the pair result reported `0` mismatches across all `18` frame pairs
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still does not read as an icy/crusted frozen bed with enough direct-view or probe-side identity
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: the new pane/crust treatment is directionally coherent in code, but the reviewed frames still collapse toward the same generic cold shelf split instead of a memorable frozen-sheet or crusted-ice read
  - promotion decision: blocked; do not promote this follow-up into `Frozen Tundra`

### 2026-03-15 frozen-bed facet-bed promotion follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - promoted the live-workbench `frozen-bed-facet-bed` winner into `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
  - ran a fresh authoritative benchmark replay through the unattended queue
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- determinism notes:
  - the authoritative single-capture run bundle is `agent/runs/20260315-044745-frozen-bed-identity/`
  - the bundled summary completed cleanly and the review was performed separately by a fresh benchmark reviewer subagent
- benchmark review:
  - `Frozen Bed`
  - must-read result: does not meet brief; reads as a dark reflective water surface with a waterline, not a frozen bed or crusted icy floor
  - hard-fail trigger seen: yes; the surface still primarily reads as water
  - motion read: subtle water shimmer / refraction, which pushes the read further toward water instead of frozen crust
  - artifact or flat-band notes: strong flat dark floor mass, pronounced horizontal waterline across mid-frame, visible central vertical seam, and faint side banding / arcs
  - promotion decision: blocked; do not promote this branch into `Frozen Tundra`

### 2026-03-15 front-mass fifth-pass artifact validation

- benchmark target:
  - `Front Mass`
- changes:
  - rebuilt the live lane on the fifth shared artifact-mitigation shader pass
  - reloaded the best-known `front-mass-wall-core-event-post-fix` branch on the rebuilt binary
- build preconditions:
  - `cargo dev`
  - `python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench-live`
- capture path:
  - `python tools/epu_workbench.py select-scene --port 4581 --mode benchmark --scene-index 4`
  - `python tools/epu_workbench.py set-config --port 4581 --file tmp/epu-workbench-live/front-mass-fifthpass-config-only.json`
  - `python tools/epu_workbench.py capture --port 4581 --label front-mass-fifthpass-baseline`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark still reads as a generic hemispheric split with synthetic wall/panel shelving
  - motion read: unchanged decision-wise; no follow-up branch was justified
  - artifact or flat-band notes: the rebuilt binary redistributes/softens the panels slightly, but it does not create a stronger front-body owner
  - promotion decision: blocked; do not promote this branch into `Storm Front`

### 2026-03-15 front-mass source-reset live validation

- benchmark target:
  - `Front Mass`
- changes:
  - rewrote `BENCHMARK_FRONT_MASS` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` around a storm-body-first `MASS_PLUME` owner with subordinate rain-wall, squall, lightning, and minimal atmosphere support
  - rebuilt the example and relaunched the persistent live workbench on the new authored benchmark
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench-live`
  - `python tools/epu_workbench.py select-scene --port 4581 --mode benchmark --scene-index 4`
  - `python tools/epu_workbench.py capture --port 4581 --label front-mass-source-reset-baseline`
  - `python tools/epu_workbench.py capture --port 4581 --label front-mass-source-reset-branch-a-dark-core`
  - `python tools/epu_workbench.py capture --port 4581 --label front-mass-source-reset-branch-b-contrast`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail, but materially better than the old wall-core family
  - hard-fail trigger seen: yes; the benchmark is still too washed out and low-contrast to become a decisive storm-front owner
  - motion read: support is present but still too faint to organize the benchmark
  - artifact or flat-band notes: the new authored baseline finally leaves the synthetic shelf/wall family, but flat slab/panel artifacts remain visible in the direct frame
  - promotion decision: blocked; keep iterating on the new authored concept rather than promoting into `Storm Front`

### 2026-03-15 frozen-bed source-reset replay follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - rewrote `BENCHMARK_FROZEN_BED` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` around a direct-view floor owner led by `CELL_SHATTER`, `SURFACE_GLAZE`, `SURFACE_CRUST`, `TRACE_CRACKS`, and `MOTTLE_GRAIN`
  - live validation judged the authored baseline replay-worthy and exported it
  - ran a fresh authoritative benchmark replay through the queue runner
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id frozen-bed-identity`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260315-132701-frozen-bed-identity/`
  - the queue runner completed cleanly and left the bundle in `awaiting_review` before fresh subagent sign-off
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the benchmark now reads as bright cracked glass or polished pane ice on a white void rather than a grounded frozen bed
  - motion read: effectively absent across the reviewed frames
  - artifact or flat-band notes: the old waterline class is reduced, but the probe remains glossy/refractive and the direct frame still lacks grounded frozen-place ownership
  - promotion decision: blocked; do not promote this branch into `Frozen Tundra`

### 2026-03-15 front-mass source-refinement replay follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - tightened the authored storm-body rewrite in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` with denser/darker `MASS_PLUME`, subordinate `MOTTLE_RIDGE` breakup, reduced `ADVECT_SQUALL` / `ATMO_FULL`, and stronger `TRACE_LIGHTNING`
  - live validation judged the authored baseline replay-worthy and exported it
  - ran a fresh authoritative benchmark replay through the queue runner
- build preconditions:
  - `cargo test -p nethercore-zx --lib epu_capabilities`
  - `cargo dev`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- capture path:
  - `python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260315-134149-front-mass-body-ownership/`
  - the queue runner completed cleanly and left the bundle in `awaiting_review` before fresh subagent sign-off
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the reviewed frames still collapse toward a flat blue-gray field / washed slab with no decisive storm body owner
  - motion read: effectively absent across the reviewed frames
  - artifact or flat-band notes: the old explicit panel-wall family is reduced, but the benchmark still does not survive authoritative review as a scene-owning weather front
  - promotion decision: blocked; do not promote this branch into `Storm Front`

### 2026-03-15 front-mass squall-shelf replay follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - rewrote `BENCHMARK_FRONT_MASS` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` again around a darker squall-shelf composition with a hard horizon split, one dominant `MASS_SHELF` storm body, subordinate ridge breakup, in-world `TRACE/LIGHTNING`, floor `FLOW`, and support `ADVECT_FRONT`
  - live validation on the reused `4581` lane judged the authored baseline replay-worthy immediately and exported it as `tmp/epu-workbench-live/exports/20260315-150003-front-mass-liveiter-pass8-authored-baseline.json`
  - ran a fresh authoritative paired benchmark replay into a durable run bundle
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - the final run bundle also rebuilt the benchmark player path as part of its captured command sequence
- capture path:
  - `python tools/tmp/run_epu_replay_pair.py ... --json-out agent/runs/20260315-150518-front-mass-storm-owner/pair-result.json`
- determinism notes:
  - the authoritative run bundle is `agent/runs/20260315-150518-front-mass-storm-owner/`
  - the pair completed with `0` mismatches
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the reviewed frames still read as a generic pale split wall with a thin dark plume rather than a scene-owning storm front
  - motion read: negligible in the reviewed frames
  - artifact or flat-band notes: seam/panel guides still survive, and the probe still collapses into a hemispheric waterline split
  - promotion decision: blocked; this still does not unblock `Storm Front`

### 2026-03-15 frozen-bed grounded-floor live follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - rewrote `BENCHMARK_FROZEN_BED` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` around a grounded outdoor frozen-floor read with mountain horizon, narrow far seam, heavy `PLANE_STONE`, `SURFACE_CRUST`, `SURFACE_DUSTED`, one sky `ADVECT/SPINDRIFT` mover, and light atmospheric depth
  - rebuilt the example and relaunched the persistent live workbench on the new authored benchmark
- build preconditions:
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench-live`
  - `python tools/epu_workbench.py select-scene --port 4581 --mode benchmark --scene-index 5`
  - `python tools/epu_workbench.py capture --port 4581 --label frozen-bed-liveiter-pass9-baseline`
  - `python tools/epu_workbench.py capture --port 4581 --label frozen-bed-liveiter-pass9-branch-a-heavy-floor`
  - `python tools/epu_workbench.py capture --port 4581 --label frozen-bed-liveiter-pass9-branch-b-horizon-separation`
  - `python tools/epu_workbench.py capture --port 4581 --label frozen-bed-liveiter-pass9-branch-c-black-ice-grounding`
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; even the grounded rewrite still collapses into a pale low-ownership field rather than a convincing outdoor frozen-floor owner
  - motion read: still too weak to matter
  - artifact or flat-band notes: the older glossy pane / cracked-glass-on-white-void class is reduced, but the replacement still lacks grounded place ownership and horizon separation
  - promotion decision: blocked; no export or replay was justified from this live pass

### 2026-03-15 front-mass coast-under-squall live follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - rewrote `BENCHMARK_FRONT_MASS` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` again around a dark coast-under-squall concept with one dominant `MASS/BANK` storm body, a low mountain horizon anchor, stone ground instead of a waterline split, and narrow support layers for lightning, squall, and runoff
  - verified the authored benchmark with `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - verified the benchmark replay script with `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  - tested the rebuilt authored concept only on the reused live workbench lane; no authoritative replay was spent and no export was produced
- capture path:
  - `python tools/epu_workbench.py select-scene --artifacts-dir tmp/epu-workbench-live --mode benchmark --scene-index 4`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass10-baseline`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass10-branch-a-*`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass10-branch-b-*`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass10-branch-c-sky-only-owner`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the rebuilt concept still collapses into a pale curved split with panelized sky and dark lower hemisphere rather than a scene-owning storm body over grounded coast
  - motion read: still too weak to matter
  - artifact or flat-band notes: panelized sky structure and the same broad split geometry still survive; `branch-c-sky-only-owner` was the least-bad branch but still not promotion-worthy
  - promotion decision: blocked; no export and no replay promotion into `Storm Front`

### 2026-03-15 front-mass split-horizon shelf live follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - rewrote `BENCHMARK_FRONT_MASS` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` toward a split-horizon, mountain-grounded `MASS_SHELF` storm-body architecture
  - verified the authored benchmark with `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - verified the showcase replay script with `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs`
  - relaunched the persistent `tmp/epu-workbench-live` lane in place to pick up the new binary; final PID `40824` on `4581`
  - tested only on the live lane; no authoritative replay was spent and no export was produced
- capture path:
  - `python tools/epu_workbench.py select-scene --artifacts-dir tmp/epu-workbench-live --mode benchmark --scene-index 4`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass11-baseline`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass11-branch-a-*`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass11-branch-b-*`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label front-mass-liveiter-pass11-branch-c-ground-anchor`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; even with the mountain-grounded split-horizon shelf architecture, the storm body still does not own the frame decisively
  - motion read: still too weak to matter
  - artifact or flat-band notes: `branch-c-ground-anchor` is the least-bad branch, but the lane still does not escape the washed split-horizon / weak-owner class enough to count as near-pass or replay-worthy
  - promotion decision: blocked; no export and no replay promotion into `Storm Front`

### 2026-03-16 front-mass plume-stack live follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - tested a less-safe source reset that replaced the tier/shelf scaffold with a `RAMP + MASS_PLUME + TRACE_LIGHTNING` storm-body stack
  - backed the reset out after live validation and left the lane on the restored authored baseline
  - rebuilt and replay-validated after restore
  - final live PID `77228` on `4581`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; live capture still collapses into the same washed hemisphere/sheet class instead of a decisive localized front-body event
  - promotion decision: blocked; no replay-worthy state emerged
  - evidence is recorded in `tmp/epu-workbench-live/front-mass-liveiter-pass12-assessment.md` and `.json`

### 2026-03-16 front-mass downburst-cell live follow-up

- benchmark target:
  - `Front Mass`
- changes:
  - tested a downburst-cell reset explicitly outside the old split/shelf/plume family
  - backed the reset out after live validation and left the lane on the restored authored baseline
  - no durable source change was kept
  - final live PID `38992` on `4581`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the concept moved off the old family but still collapsed into the same capped hemisphere/probe class
  - promotion decision: blocked; no replay-worthy state emerged
  - evidence is recorded in `tmp/epu-workbench-live/front-mass-liveiter-pass13-assessment.md` and `.json`

### 2026-03-15 frozen-bed face-cut crust live follow-up

- benchmark target:
  - `Frozen Bed`
- changes:
  - rewrote `BENCHMARK_FROZEN_BED` in `examples/3-inspectors/epu-showcase/src/benchmarks.rs` toward a face-cut horizon, darker bed, crust-led floor owner, subordinate grain breakup, restrained dust, and a sky band instead of white atmospheric wash
  - verified the authored benchmark with `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - verified the showcase replay script with `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs`
  - relaunched the persistent `tmp/epu-workbench-live` lane in place; final PID `10624` on `4581`
  - tested only on the live lane; no authoritative replay was spent and no export was produced
- capture path:
  - `python tools/epu_workbench.py select-scene --artifacts-dir tmp/epu-workbench-live --mode benchmark --scene-index 5`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label frozen-bed-liveiter-pass10-baseline`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label frozen-bed-liveiter-pass10-branch-a-*`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label frozen-bed-liveiter-pass10-branch-b-horizon-owner`
  - `python tools/epu_workbench.py capture --artifacts-dir tmp/epu-workbench-live --label frozen-bed-liveiter-pass10-branch-c-*`
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail
  - hard-fail trigger seen: yes; the lane regressed into a glossy suspended mound with a cracked underside rather than a grounded frozen bed
  - motion read: still too weak to matter
  - artifact or flat-band notes: `branch-b-horizon-owner` moved the frame most, but it still stayed in the same suspended glossy-mound class rather than producing a grounded frozen-place owner
  - promotion decision: blocked; no export and no replay promotion into `Frozen Tundra`

### 2026-03-16 seam-isolation and 360-orbit validation follow-up

- benchmark targets:
  - `Front Mass`
  - `Frozen Bed`
- changes:
  - removed the direct-background dependence on the old 3-tap filtered path in `nethercore-zx/shaders/common/20_environment/90_sampling.wgsl` so background seam checks use the raw procedural evaluator again
  - added live-workbench camera angle/elevation control in `core/src/workbench/mod.rs`, `core/src/app/player/workbench.rs`, `nethercore-zx/src/debug/epu_panel/mod.rs`, and `tools/epu_workbench.py` so the lane can be checked through a full 360-degree orbit instead of one hero angle
  - fixed wrapped `SILHOUETTE` lattice indexing in `nethercore-zx/shaders/epu/bounds/02_silhouette.wgsl` so periodic horizon noise no longer tears at the cylinder seam
  - added a direct-view facing gate to the shared `DIRECT3D` path used by `ADVECT` and `MASS` in `nethercore-zx/shaders/epu/features/13_advect.wgsl` and `nethercore-zx/shaders/epu/features/15_mass.wgsl`, which removes the old grazing-angle panel/beam family instead of blurring it
  - captured full 360-degree background-only orbit sheets and isolated-layer checks under `tmp/epu-workbench-orbit/`, then ran a fresh paired benchmark replay
- build preconditions:
  - `cargo test -p nethercore-zx --lib`
  - `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
  - `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- capture path:
  - `python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4583 --artifacts-dir tmp/epu-workbench-orbit`
  - `python tools/epu_workbench.py select-scene --port 4583 --mode benchmark --scene-index 4`
  - `python tools/epu_workbench.py select-scene --port 4583 --mode benchmark --scene-index 5`
  - `python tools/epu_workbench.py set-view --port 4583 --show-ui false --show-probe false --show-background true --locked true`
  - repeated `python tools/epu_workbench.py set-view --port 4583 --camera-angle <deg> --camera-elevation 18` plus `capture` through a full `0..345` orbit, then repeated the worst-angle checks with layer isolation
  - `python tools/tmp/run_epu_replay_pair.py C:\Users\rdave\AppData\Roaming\Nethercore\data\screenshots target\debug\nethercore-zx.exe examples\3-inspectors\epu-showcase\epu-showcase.nczx examples\3-inspectors\epu-showcase\screenshot-benchmarks-anim3.ncrs 18 --cwd D:\Development\nethercore-project\nethercore --json-out tmp\benchmark-pair-v5.json`
- determinism notes:
  - the paired replay at `2026-03-16 03:53:04..03:53:10` and `03:53:13..03:53:20` produced `0` mismatches across all `18` frame pairs
  - the direct-background orbit evidence lives in `tmp/epu-workbench-orbit/front-mass-orbit-v4-background-contact.png` and `tmp/epu-workbench-orbit/frozen-bed-orbit-v4-background-contact.png`
  - the latest paired replay contact sheet is `tmp/benchmark-contact-20260316-pass5.png`
- benchmark review:
  - `Front Mass`
  - must-read result: still fail, but materially healthier
  - hard-fail trigger seen: yes; the benchmark still lacks a decisive scene-owning storm body and the motion is still too weak
  - motion read: support motion survives, but not strongly enough to organize the scene
  - artifact or flat-band notes: the old hard seam/panel wedge is no longer the dominant failure in the 360-degree direct-background orbit; the remaining miss is washed slab ownership, not the earlier seam family
  - promotion decision: benchmark remains blocked for `Storm Front`, but the seam-isolation branch is directionally closed enough that the next weather-body loop should target ownership/motion instead of blur, seam fade, or panel cleanup
- benchmark review:
  - `Frozen Bed`
  - must-read result: still fail, but materially healthier
  - hard-fail trigger seen: yes; the benchmark still does not achieve a convincing frozen-bed place identity
  - motion read: still minimal and not decision-moving
  - artifact or flat-band notes: the old bright seam wedge and pane crack family are no longer the dominant failure in the 360-degree direct-background orbit; the remaining miss is frozen material/place ownership rather than wrap tearing
  - promotion decision: benchmark remains blocked for `Frozen Tundra`, but the seam-isolation branch is directionally closed enough that the next frozen-floor loop should target grounded icy identity instead of seam hiding
