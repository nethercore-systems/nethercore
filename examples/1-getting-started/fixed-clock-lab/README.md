# Fixed clock: update owns simulation time

Compare a simulation that advances in `update()` with a render callback that
only reads it. The moving marker and labels are original procedural source,
under the repository license; no external assets are required.

## Build and run

Requires Rust's `wasm32-unknown-unknown` target and matching `nether` and
`nethercore-zx` executables on PATH. From this directory:

```sh
nether build -p . -o fixed-clock-lab.nczx
nethercore-zx fixed-clock-lab.nczx
nethercore-zx fixed-clock-lab.nczx --headless --replay witness.ncrs --report clock-state.json --timeout 30
nethercore-zx fixed-clock-lab.nczx --replay capture.ncrs --scale 1
```

Copy just the cartridge outside this checkout and launch it there to check
relocation. Use absolute replay/executable paths when running outside the source.

## Controls and time model

- **A / Z:** pause/resume the simulation. Host ticks continue while paused.
- **B / X:** draw the marker once or four times per render callback. This changes
  draw calls, not callback frequency and not the simulated position.
- **Y / V:** reset the simulation clock and unpause. This does not reset the host
  tick counter or the selected draw count.
- **F3:** inspect host tick, delta time, simulation ticks, pause and position.

At the declared 60 Hz tick rate, movement is four pixels per simulation tick and
wraps every 200 simulation ticks. State uses integer ticks; seconds are a label
computed from ticks and `delta_time()`, not a wall-clock accumulator. The render
loop must not increment a simulation counter or consume the RNG.

## Executable witness

`witness.ncrs` checks pause, resume, draw-count selection, reset and the simulated
endpoint. `capture.ncrs` contains the same inputs and requests six native views.

After building, with Python 3.11+ and Pillow:

```sh
python check.py --player /absolute/path/to/nethercore-zx --rom /absolute/path/to/fixed-clock-lab.nczx --output /absolute/path/to/clock-evidence
```

Use the executable's actual path (including `.exe` on Windows). This check runs
zero-render headless replays and native one/four-draw variants, checks exact named
state at host ticks 60, 120 and 180, and compares the actual endpoint marker pixels.
It requires a native display. Generated reports/logs/media are local evidence.

**Compare host ticks, not screenshot ordinal numbers.** A headless replay frame
contains pre- and post-update snapshots; frame 180 has host tick 180 before its
update and 181 afterward. The native checkpoint logged at tick 180 must be compared
to tick 180, not to that extra headless post-update. At tick 180 the scripted
simulation is at tick 139, position 556. Repeated logs during rollback are expected;
these diagnostics are not persistence/audio events.

Passing the check is technical evidence, not human visual approval or real network
verification. Interpolation is optional; adding it must not advance simulation.
