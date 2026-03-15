# Live EPU Workbench

This is the restartable local control loop for fast EPU authoring on one machine.

Use `agent/start-here.md` as the overall program entrypoint. Use this file when the next task is live tuning rather than replay-only validation.

## Purpose

The live workbench is for:

- selecting existing benchmark and showcase scenes without editing Rust
- reading and writing the current 8-layer EPU config
- patching semantic layer fields quickly
- capturing fresh full/probe/background review images
- exporting a winning candidate to durable JSON and Rust snippet form
- running scripted parameter sweeps with durable manifests
- seeing the result immediately and iterating knob-by-knob in the same lane until the direction is strong enough to hand off

The authoritative promotion path is still replay-based validation. The live workbench is the fast local discovery loop before promotion.

Critical authority rule:

- Loading or setting a workbench JSON config while unlocked updates the editor and exported/session state, but it does not make that config render-authoritative in the live frame.
- Unlocked live rendering still comes from the game-owned runtime `epu_frame_configs`.
- If you need truthful live validation of an editor-loaded candidate, lock the editor override first.

## Core Files

- `tools/epu_workbench.py`
  Thin machine-drivable CLI over the local HTTP API.
- `core/src/app/player/workbench.rs`
  Local HTTP server and capture/export plumbing.
- `core/src/workbench/mod.rs`
  Stable JSON schema for workbench commands and responses.
- `nethercore-zx/src/console.rs`
  ZX console bridge into the existing EPU panel/editor.
- `examples/3-inspectors/epu-showcase/src/lib.rs`
  Benchmark/showcase scene selection and probe/background debug toggles.

## Reuse Before Launch

For orchestration runs, treat the workbench as a persistent lane resource, not a disposable per-worker process.

Before launching a new HTTP server:

1. Audit existing player processes and listening workbench ports.
2. Check for existing `tmp/epu-workbench*` artifacts dirs and `session.json` files.
3. Attach to a healthy existing session first.
4. Only launch a new session if no healthy one fits the lane.

Useful checks from the repo root:

```powershell
Get-Process nethercore-zx -ErrorAction SilentlyContinue
Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -ge 4580 -and $_.LocalPort -le 4690 }
Get-ChildItem tmp -Directory -Filter 'epu-workbench*'
```

Operational rules:

- Do not spawn one fresh workbench server per worker by default.
- Prefer one persistent session per active lane.
- If two workers must not contend for the same live session, serialize them or give them explicitly different ports and artifacts dirs.
- When a session is stale, either relaunch in the same artifacts dir or shut it down before creating another.
- Record the artifacts dir and port in the worker result so later workers can reuse the same session instead of relaunching blindly.
- The live worker should inspect its own captures and keep iterating in-lane. Do not wait for a separate reviewer after every small knob turn.
- Change one meaningful lever at a time when possible so the cause of improvement or regression stays legible.
- Hand off only when the candidate is strong enough to export or when the lane has hit a concrete blocker.

## Start A Session

Build the latest showcase ROM code first when the showcase source changed:

```powershell
cd examples/3-inspectors/epu-showcase
cargo build --target wasm32-unknown-unknown --release
cd ../../..
```

Preferred restartable command:

```powershell
python tools/epu_workbench.py launch `
  --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm `
  --port 4581 `
  --artifacts-dir tmp/epu-workbench
```

The raw `.wasm` launch path walks up to the nearest ancestor `nether.toml` and inherits that game's `id`, `title`, and `render_mode`. That matters for the showcase because the probe depends on `render_mode = 2` for the intended reflection and IBL read.

Equivalent manual player launch:

```powershell
cargo run -p nethercore-zx --bin nethercore-zx -- `
  examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm `
  --epu-workbench-port 4581 `
  --epu-workbench-dir tmp/epu-workbench
```

After startup, the session writes durable state to:

- `tmp/epu-workbench/session.json`
- `tmp/epu-workbench/launch.json`
- `tmp/epu-workbench/player.stdout.log`
- `tmp/epu-workbench/player.stderr.log`

Future commands can use `--port 4581` directly or omit `--port` and let the CLI read `tmp/epu-workbench/session.json`.

## Security Model

The workbench server is opt-in.

- The player only starts it when `--epu-workbench-port` is passed.
- Without that flag, no workbench server is created.
- The listener binds `127.0.0.1` only, not `0.0.0.0`.
- This is a local authoring tool, not a general remote control surface.

## Live Tuning Loop

Read the current session:

```powershell
python tools/epu_workbench.py session
```

Select a scene by id:

```powershell
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 0
python tools/epu_workbench.py select-scene --mode showcase --scene-index 3
python tools/epu_workbench.py select-scene --mode showcase --scene-index 3 --no-lock-editor
```

By default `select-scene` locks the editor override after loading the scene into the workbench. That is correct for machine-driven authoring, but if you then cycle scenes manually in the window, the render will stay pinned to the locked override. Use `--no-lock-editor` or `set-view --locked false` when you want manual scene cycling to drive the live output.

Current lock behavior note:

- Locked workbench editing now preserves the live scene's current `param_d` phase bytes when replacing `env_id 0` with the editor override, so motion proof is valid in locked mode when the selected scene actually animates.
- If a scene still saves byte-identical captures after that fix, treat it as a scene-specific authored-motion/read blocker, not immediate proof that the whole live capture path is dead.
- When validating live captures, compare decoded pixels for the full frame, probe crop, and background crop separately. One weak or near-static crop is not proof that the whole saved capture path is stale.

Scene ids are array indices in:

- `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
- `examples/3-inspectors/epu-showcase/src/presets.rs`

Read the current 8-layer config:

```powershell
python tools/epu_workbench.py get-config
```

Patch one layer field:

```powershell
python tools/epu_workbench.py patch-layer --layer 2 --field intensity --value-json 208
python tools/epu_workbench.py patch-layer --layer 2 --field color_a --value-json "[220,160,96]"
python tools/epu_workbench.py patch-layer --layer 2 --field blend --value-json "\"screen\""
```

Replace the full config from JSON:

```powershell
python tools/epu_workbench.py set-config --file tmp/epu-workbench/candidate.json
```

Important:

- `set-config` in unlocked mode is an editor-state load, not a live render takeover.
- Do not judge a loaded candidate from the unlocked frame and assume the JSON failed to apply.
- For truthful candidate validation after `set-config`, use `set-view --locked true` before capture or review.

Change editor-facing view state:

```powershell
python tools/epu_workbench.py set-view --selected-layer 2 --isolated-layer 2 --locked true
python tools/epu_workbench.py set-view --clear-layer-isolation
python tools/epu_workbench.py set-view --show-probe false
python tools/epu_workbench.py set-view --show-background false
python tools/epu_workbench.py set-view --show-background true --show-probe true
python tools/epu_workbench.py set-view --camera-angle 330 --camera-elevation 18
```

Review guidance:

- `show-probe false` gives a background-only direct place read.
- `show-background false` gives a probe-first read against the clear background.
- Keep both reads healthy before promotion.
- Use those reads during live iteration yourself. Fresh adversarial review comes later, after export or replay promotion.

Camera note:

- `set-view` now also accepts `--camera-angle` and `--camera-elevation`.
- `session` / `capture` summaries now report those camera values, so a saved seam check can be reproduced exactly.

## Orbit Seam Validation

When the suspected defect is a seam, guide line, pane edge, or wrap artifact, do not rely on a single hero angle.

Recommended lane setup:

```powershell
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 4
python tools/epu_workbench.py set-view --show-ui false --show-probe false --show-background true --locked true
python tools/epu_workbench.py set-view --camera-angle 0 --camera-elevation 18
```

Recommended method:

- Sweep one full orbit at fixed elevation, for example `0..345` degrees in `15` degree steps.
- Capture each stop, then build a contact sheet from the background crops.
- If a seam survives, repeat the same orbit with `--isolated-layer` set to the suspected layer until the offending opcode family is isolated.
- Only call the seam class closed when the full orbit no longer shows the old hard seam family in the direct background.

## Animation Loop Validation

When the suspected defect is a phase wrap, end-of-loop flash, or animation snap, do not rely on three nearby replay frames.

Recommended lane setup:

```powershell
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 3
python tools/epu_workbench.py set-view --show-ui false --show-probe false --show-background true --locked true
python tools/epu_workbench.py set-view --camera-angle 0 --camera-elevation 18
```

Recommended method:

- Capture one full authored loop cycle, including frames immediately before and after the phase wraps back to the loop start.
- Repeat that same wrap check from a full fixed-elevation camera orbit, or at minimum from enough orbit stops to prove the flash is not view-specific.
- If the wrap flash survives, rerun the same check with `--isolated-layer` set until the offending opcode family is isolated.
- Only call the animation-loop class closed when the loop boundary no longer flashes in the direct background around the orbit.

## Troubleshooting

If the probe suddenly looks flat, dark, or "non-reflective", check session state before assuming the shaders or builder are broken.

Common causes:

- `isolated_layer` is active, so only one layer is rendering.
- The isolated layer is a bounds-only layer, which can make the probe look almost dead.
- `show-probe` was turned off.
- `show-background` was turned off.
- The player binary is stale and a raw `.wasm` launch is falling back to render mode `0` instead of the example's `nether.toml` render mode.
- The probe material override flags are missing, so fallback texture values can push the probe toward a flat white read.

Recovery steps:

```powershell
python tools/epu_workbench.py set-view --clear-layer-isolation --show-background true --show-probe true
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 0
```

`select-scene` now resets the editor to a clean baseline by clearing layer isolation, selecting layer `0`, and applying the requested lock state. That change exists specifically to avoid false "reflections are broken" sessions.

If the probe still looks dead after a restart, rebuild the player and showcase, then relaunch the workbench session so the raw `.wasm` path picks up the current loader fix and the current probe material override fix.

## Capture And Export

Capture the live frame plus background/probe crops:

```powershell
python tools/epu_workbench.py capture --label benchmark-0-pass-a
```

Export a candidate:

```powershell
python tools/epu_workbench.py export `
  --label benchmark-0-pass-a `
  --rust-const-name BENCHMARK_0_PASS_A `
  --include-json-text `
  --include-rust-text
```

Artifacts land under:

- `tmp/epu-workbench/captures/`
- `tmp/epu-workbench/exports/`

## Sweep Exploration

Sweep a numeric range:

```powershell
python tools/epu_workbench.py sweep-layer `
  --layer 2 `
  --field intensity `
  --start 96 `
  --stop 224 `
  --step 32 `
  --capture `
  --label-prefix benchmark-0-layer2-intensity
```

Sweep explicit JSON values:

```powershell
python tools/epu_workbench.py sweep-layer `
  --layer 4 `
  --field blend `
  --values-json "[\"add\",\"screen\",\"overlay\"]" `
  --capture `
  --label-prefix showcase-3-layer4-blend
```

Sweep manifests land under:

- `tmp/epu-workbench/sweeps/`

## Promotion Path

After a live candidate looks healthy:

1. Export the candidate from the live workbench.
2. Promote the winning config into the appropriate showcase or benchmark Rust source.
3. Run replay validation with the existing authoritative path.
4. Log the result in the durable agent artifacts.

Keep replay validation unchanged. The live workbench is a precursor, not a replacement.

## Proof-Of-Life Example

This is the minimal end-to-end loop to demonstrate the system:

```powershell
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 0
python tools/epu_workbench.py patch-layer --layer 0 --field intensity --value-json 192
python tools/epu_workbench.py capture --label benchmark-0-intensity-192
python tools/epu_workbench.py export --label benchmark-0-intensity-192 --rust-const-name BENCHMARK_0_INTENSITY_192 --include-rust-text
```

Expected durable outputs:

- updated `tmp/epu-workbench/session.json`
- one capture triplet in `tmp/epu-workbench/captures/`
- one export file in `tmp/epu-workbench/exports/`





