# Live EPU Workbench

Reuse-first restart path:

Before launching anything new, audit the existing lane and attach if it is healthy:

```powershell
Get-Process nethercore-zx -ErrorAction SilentlyContinue
Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -ge 4580 -and $_.LocalPort -le 4690 }
python tools/epu_workbench.py status --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py health --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py session --artifacts-dir tmp/epu-workbench-live
```

If that session is healthy, reuse it. Do not spin up another HTTP workbench instance for the same lane.

Only relaunch when there is no healthy session for the lane, or when the existing session is stale and needs to be replaced in place:

```powershell
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench-live
```

That starts the showcase ROM with the local workbench HTTP service enabled and writes the session record under the selected artifacts directory.

If you do not pass `--artifacts-dir`, follow-up commands will read:

```text
tmp/epu-workbench/session.json
```

If you do pass a custom `--artifacts-dir`, follow-up commands can use `--artifacts-dir`, `--port`, or `--session-file`.

## Fast Path

Attach to the current lane or relaunch it in place:

```powershell
python tools/epu_workbench.py status --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py health --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py session --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py list-scenes --artifacts-dir tmp/epu-workbench-live
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --port 4581 --artifacts-dir tmp/epu-workbench-live
```

Operational rule:

- Keep one persistent workbench HTTP session per active lane.
- Reuse the same artifacts dir and port while iterating on that lane.
- If workers would contend for one live session, serialize them or give them clearly separate ports and artifacts dirs on purpose.
- The live worker is expected to inspect its own captures and iterate in place.
- Turn one meaningful knob at a time, recapture, and keep going until the candidate is clearly strong or clearly blocked.
- Reserve fresh adversarial review for exported or replay-promoted candidates, not every micro-iteration.
- Loading JSON into the editor while unlocked does not make that JSON the active live render source.
- For truthful evaluation of an editor-loaded candidate, lock the editor override before capture or judgment.

Load a benchmark or showcase scene into the live editor and lock it for authoring:

```powershell
python tools/epu_workbench.py select-scene --mode benchmark --scene-index 2
python tools/epu_workbench.py select-scene --mode showcase --scene-index 4
```

Read and patch the live 8-layer config:

```powershell
python tools/epu_workbench.py get-config
python tools/epu_workbench.py patch-layer --layer 3 --field intensity --value-json 196
python tools/epu_workbench.py patch-layer --layer 3 --field color_a --value-json "[196,228,255]"
python tools/epu_workbench.py set-view --isolated-layer 3
python tools/epu_workbench.py set-view --show-probe false
python tools/epu_workbench.py set-view --show-probe true
python tools/epu_workbench.py set-view --show-ui false
```

Authority note:

- `set-config --file ...` and other editor loads update editor/session state immediately.
- In unlocked mode, the frame can still be owned by the scene's live runtime config.
- Use `python tools/epu_workbench.py set-view --locked true` when you need the loaded editor config to become the rendered truth.

Capture review images from the live session:

```powershell
python tools/epu_workbench.py capture --label front-mass-pass-01
```

Each capture writes:

- full frame
- background crop
- probe crop

under the active artifacts directory, usually `tmp/epu-workbench/captures/`.
Use `python tools/epu_workbench.py status` to confirm the resolved artifacts/session paths.

Export a candidate for promotion:

```powershell
python tools/epu_workbench.py export --label front-mass-pass-01 --rust-const-name FRONT_MASS_LAYERS --include-rust-text
```

That writes both JSON and Rust snippet artifacts into `agent/workbench/<timestamp>/exports/`.

## Sweep Loop

Simple one-field sweep with automatic captures:

```powershell
python tools/epu_workbench.py sweep-layer --layer 4 --field intensity --values-json "[96,128,160,192]" --capture --label-prefix front-mass-intensity
```

The sweep client is intentionally thin. For multi-parameter sweeps, script repeated `patch-layer` and `capture` calls around the same API.

## Promotion Workflow

1. Author live in the workbench until the background read and probe read both hold up.
   For editor-loaded JSON candidates, that live judgment must happen in locked mode.
2. Inspect your own live captures as you iterate. The point of the lane is fast local knob-turning with immediate visual feedback.
3. Export the winning candidate with `python tools/epu_workbench.py export ...`.
4. Promote the Rust snippet into the showcase source by hand.
5. Run the existing replay validation path unchanged:

```powershell
cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs
target\debug\nethercore-zx.exe examples\3-inspectors\epu-showcase\epu-showcase.nczx --replay examples\3-inspectors\epu-showcase\screenshot-benchmarks-anim3.ncrs
```

6. Review the authoritative screenshots before promotion.

## Notes

- The local service binds to `127.0.0.1` only.
- The live editor still uses the existing ZX EPU debug panel as the source of truth.
- Scene selection currently targets the `epu-showcase` inspector ROM.
- `current-session.json` is the durable resume point for future agents.
