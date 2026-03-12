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
- `Front Mass` remains blocked, but the active architecture is now `MASS` for body plus `ADVECT` for subordinate motion.
- `Frozen Bed` remains blocked even after `SURFACE` landed; frozen-material identity is still not closed at shipping quality.
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
