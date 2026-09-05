# Executable ZX replay automation

## Run the real game

```bash
cargo build -p nether-cli -p nethercore-zx
nether replay run tests/scenario.ncrs --rom game.nczx --headless --report report.json --timeout 30
```

Use the matching `nether` and `nethercore-zx` binaries from the build. The CLI prefers a sibling player. `.wasm` is also supported, but has no packed ROM assets.

To discover state and actions, start with:

```toml
console = "zx"
seed = 4401
players = 2

[[frames]]
f = 0
snap = true
```

Reports expose typed `registered_variables` (canonical name, full registry path and unambiguous aliases), `registered_actions` (label, full path, parameter types/defaults), actual assertion results, and snapshots. Games must register relevant state/actions during `init()`. Empty state is not semantic coverage. Register only stable-memory watches; the runtime uses bounds-checked reads.

## Semantics

- Frame zero is the first update. Unspecified frames/players are idle; inputs do not persist. Repeat held input on every intended tick.
- One frame entry/action per frame. Duplicate frames, unknown fields/buttons, malformed input byte lengths and invalid analog ranges are rejected. Dense scripts are bounded to 1,000,000 ticks.
- Seed and player count apply before `init()`. Persistent saves are not loaded or written in replay mode. The default tick interval is available at init; subsequent updates use the game's selected tick rate.
- Order: action → inputs → `pre` snapshot → update/audio-state advance → `post` snapshot → assertions. No graphics or audio device is created by the headless path.
- `$prev_name` refers to the preceding tick's post-update value, even across ticks with no snapshot. It is unavailable on frame zero. Assertions compare scalar numbers (f32 precision when either operand is f32, otherwise exact integers); vectors remain typed snapshot data.
- `match/score_p1` becomes `$match_score_p1`. A unique leaf alias such as `$score_p1` is offered only if it cannot shadow another canonical name. Canonical normalization collisions fail explicitly.
- Use exact action labels or full paths. Only registered i32/f32 action parameters are supported; omitted parameters use registered defaults. Unknown parameters or wrong types fail.

Example **only if the game registers this action/state**:

```toml
[[frames]]
f = 1
p1 = "idle"
action = "Force Point"
action_params = { scorer = 0 }
snap = true
assert = "$match_score_p1 == 1"
```

## Failure contract

Success requires exit code zero, report status `PASSED`, completed frame count, and your expected assertions/nonempty state. Failed assertions report `FAILED`; startup, trap and runtime errors report `ERROR` with context. Both exit nonzero. `--fail-fast` stops at the first failed assertion. `--timeout` bounds the headless player invocation, including ROM loading, compilation, initialization, actions and updates; it does not cover building a missing player through Cargo fallback. Use an already-built player for automation.

A requested report is replaced on ordinary replay failures; it may not overwrite the ROM/script. Invalid CLI arguments fail before execution and may not write a report: always check the exit code, never trust an old JSON alone. A watchdog timeout may have only startup-level context rather than the exact interrupted frame.

## Rendered playback and boundaries

`nether run --no-build --replay tests/visual.ncrs` supports **inputs and `screenshot = true` only**. Remove actions, assertions and `snap` directives; they are rejected before window creation, not silently skipped. Headless runs reject screenshots. Rendered replay and sync-test can be paired; replay/netplay combinations are unsupported. Runtime errors in automated rendered runs exit nonzero.

Semantic snapshots are **not restorable save states**. Binary `.ncrp` compilation/recording stores inputs, not an executable semantic test with actions/assertions. This is not a rewind/seek debugger. Identical local runs establish repeatability, not cross-machine/platform determinism or multiplayer correctness. Pixels, audio and game feel need separate acceptance.

Runtime regressions: `cargo test -p nethercore-zx --lib replay::` and `cargo test -p nethercore-zx --test replay_cli`. Cross-game proof lives in sibling `nethercore-ai-plugins`: `python scripts/verify_agent_workflow.py --rebuild-games` builds/re-packs temporary Volley Fighter and Scrapheap Saints carts without replacing checked-in ROMs, then compares semantic states and deliberate negatives.
