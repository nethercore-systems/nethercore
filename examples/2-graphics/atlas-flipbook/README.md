# Atlas flipbook: time belongs to update

A small guest frame table drives four original squash/stretch poses from one
procedural 64×16 RGBA atlas. The big sprite and filmstrip use
`draw_sprite_region`; the yellow border identifies the selected table entry.
`src/atlas.rs` is the editable, dependency-free image generator. The code and
original character texture use the repository license; there are no external
assets. Prerequisites: textured-quad and fixed-clock-lab.

## Build and use

With the Rust `wasm32-unknown-unknown` target and matching tools, from this directory:

```sh
nether build -p . -o atlas-flipbook.nczx
nethercore-zx atlas-flipbook.nczx
```

The lesson starts paused to expose frame zero and the table before motion begins.
The footer pairs **controller button / player-1 keyboard key**:

- **A / Z:** toggle play/pause. Holding the button does not repeat.
- **B / X:** cycle **loop → once → ping-pong → loop**, restarting at frame zero
  while retaining the current play/pause choice.
- **X / C:** advance **one update tick**, not one whole frame, while paused.
- **L1 / Q:** rewind the current mode to its beginning, preserving pause state.
- **Y / V:** reset to loop, frame zero, tick zero and paused.
- **F3:** inspect mode, frame, frame age, pause, finished and direction.

`8t` and `12t` below the filmstrip mean update ticks, not milliseconds. At the
explicit 60 Hz rate, the four durations are 8, 12, 8 and 12 ticks. The selected
frame changes when its duration expires; leftover elapsed time is never advanced
by render. Once holds the final pose and reports END after its full duration.
Ping-pong visits 0,1,2,3,2,1,0,1 without duplicating either endpoint.

All mutable playback state lives in WASM linear memory. `update` owns both input
edges and time; `render` copies state and only draws. Loading happens once in
init, and nearest sampling keeps this tiny unpadded atlas crisp. This is guest
metadata and playback logic, **not a new console animation API**, texture-update
API, interpolation/blending system, or online-netplay demonstration.

## Checks and actual native witnesses

The pure playback boundary check needs no emulator or test dependency:

```sh
rustc --edition 2021 --test src/playback.rs -o /absolute/outside/path/playback-tests
/absolute/outside/path/playback-tests
```

Use `.exe` on Windows. It covers just-before/exact boundaries, all three modes,
finished-state stability and restart. The actual cartridge witnesses are:

```sh
nethercore-zx atlas-flipbook.nczx --headless --replay witness.ncrs --report state.json --timeout 30
nethercore-zx atlas-flipbook.nczx --replay capture.ncrs --scale 1
python ../../1-getting-started/check_foundations.py --projects 2-graphics/atlas-flipbook --nether /absolute/path/nether --player /absolute/path/nethercore-zx --output /empty/directory/outside/checkout --render
```

Python 3.11+ and Pillow are needed for the shared qualification runner. It builds
and packs outside the checkout, checks 241 ticks/52 state assertions, rejects a
false assertion, validates 25 actual native captures and runs the **whole
241-input-frame** witness through forced rollback. The pixel checker compares
the enlarged pose with its filmstrip entry, all four distinct silhouettes,
selection borders, reverse visits, pause, once/restart endpoints and exact reset.

Only the `.nczx` is needed for relocation. Captures/reports are QA outputs, not
source assets. Technical checks do not constitute human approval or delivery;
committed-source, remote-ref and applicable CI checks remain separate gates.
