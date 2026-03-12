# Live EPU Workbench

Restart entrypoint:

```powershell
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm
```

That starts the showcase ROM with the local workbench HTTP service enabled and writes the session record under the selected artifacts directory.

If you do not pass `--artifacts-dir`, follow-up commands will read:

```text
tmp/epu-workbench/session.json
```

If you do pass a custom `--artifacts-dir`, follow-up commands can use `--artifacts-dir`, `--port`, or `--session-file`.

## Fast Path

Launch or reconnect:

```powershell
python tools/epu_workbench.py launch --rom examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm
python tools/epu_workbench.py health
python tools/epu_workbench.py session
python tools/epu_workbench.py status
python tools/epu_workbench.py list-scenes
```

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
2. Export the winning candidate with `python tools/epu_workbench.py export ...`.
3. Promote the Rust snippet into the showcase source by hand.
4. Run the existing replay validation path unchanged:

```powershell
cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs
target\debug\nethercore-zx.exe examples\3-inspectors\epu-showcase\epu-showcase.nczx --replay examples\3-inspectors\epu-showcase\screenshot-benchmarks-anim3.ncrs
```

5. Review the authoritative screenshots before promotion.

## Notes

- The local service binds to `127.0.0.1` only.
- The live editor still uses the existing ZX EPU debug panel as the source of truth.
- Scene selection currently targets the `epu-showcase` inspector ROM.
- `current-session.json` is the durable resume point for future agents.
