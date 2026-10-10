# Seeded random: a bounded route

A 32-point walk demonstrates the host RNG, deterministic replay and bounded
procedural generation without an asset pack. All route/visual source is original
example code under the repository license.

## Build and run

Requires Rust's `wasm32-unknown-unknown` target and matching `nether` and
`nethercore-zx` executables on PATH. From this directory:

```sh
nether build -p . -o seeded-random-lab.nczx
nethercore-zx seeded-random-lab.nczx
nethercore-zx seeded-random-lab.nczx --headless --replay witness.ncrs --report rng-state.json --timeout 30
nethercore-zx seeded-random-lab.nczx --replay capture.ncrs --scale 1
```

Copy only the cartridge to a directory outside the checkout and start it there
for relocation. Automated executable/replay paths must then be absolute.

## Controls and generation contract

- **Left/Right:** move along the current route, clamped to points 0..31.
- **A / Z:** reset the runner and stop autorun. The route and RNG stream stay intact.
- **B / X:** generate the next route, advancing the existing RNG stream.
- **X / C:** toggle autorun, advancing every sixth host tick.
- **F3:** inspect the generation, cursor, point count, bound and route hash.

Each route performs exactly 32 `random_range(-1, 2)` calls. Every new lane is
clamped to -3..3, so generation has no rejection/retry loop. A route hash covers
the generated lanes; it is diagnostic, not a cryptographic identifier.

RNG calls occur only in init/update. Render reads the route and never consumes
randomness. The guest API exposes no host reseed operation: A resets the runner,
not the seed. To reproduce the entire generation sequence, restart the cartridge
with the same recorded replay seed and inputs. Do not teach B as "reset seed."

## Executable witness

With Python 3.11+, pass a built relocated ROM and native player:

```sh
python check.py --player /absolute/path/to/nethercore-zx --rom /absolute/path/to/seeded-random-lab.nczx --output /absolute/path/to/rng-evidence --render
```

Use real absolute paths and `.exe` where required. The checker executes seed 4919
twice and seed 4920 once, compares all named pre/post guest state for equal runs,
requires changed route hashes for the different seed, and checks the 32-point/lane
bounds. Native mode separately compares the logged generation hashes with headless
state and preserves the six actual captures. Omit `--render` only for the headless
subset; it is not visual qualification. Reports and media are local QA outputs.

Equal images alone are not a determinism test; neither this local check nor its
hashes establish cross-platform determinism or human visual acceptance.
