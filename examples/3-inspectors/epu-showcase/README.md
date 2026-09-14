# EPU Showcase

Editable guest instruction programs, not engine-owned presets or world simulation.
Twenty optional compositions and seven small capability scenes use all 24 non-NOP
opcodes. This is a mechanism catalogue, not approval of every style/variant.

## Build and run

Run these from the repository root with the current `nether` and `nethercore-zx` on PATH:

Automatic Rust output lookup ignores host build scripts, tests, benches and examples; it follows Cargo metadata (shared targets, workspace/config target directories, and the crate's actual output name), not a stale project-local `target/` or another game's newest file. Set `[build].wasm` explicitly for multiple outputs or custom scripts that override Cargo's target/profile/output conventions.

```sh
nether build -p examples/3-inspectors/epu-showcase -o tmp/epu-showcase.nczx
nethercore-zx tmp/epu-showcase.nczx
```

Gamepad inputs:

- **Start:** switch composition/capability suites and reset the scene index.
- **A / B:** next / previous scene, with wraparound.
- **X:** sphere / cube / torus material probe.
- **Y:** show/hide the on-screen labels.
- **Left stick:** orbit; capability scenes deliberately hold their rig elevation.
- **F4:** native debug inspector. Scene/probe/background, metallic and roughness
  controls are registered there. Use the EPU panel's **LOCK** for frozen semantic
  layer editing; it overrides all environments. Unlock for guest-animation checks.

`src/presets.rs` lists the compositions and rates; `src/benchmarks.rs` lists the
capability scenes and their fixed camera rigs. Both contain ordinary eight-layer
programs. Names and palettes are editable example choices, not runtime rules.

## Phase ownership

`update()` advances the rollback-covered guest counter. `render()` copies the
selected program and calls `constants::animate_phases`; repeated rendering does
not advance it. The helper adds to the authored starting phase instead of erasing
it, and preserves every other field:

- `SCATTER_PHASED`: `param_c` phase, `param_d` seed; fixed point identity.
- `FLOW` supported patterns, `GRID`, `DECAL`, modulated `LOBE`, `BAND`,
  `MOTTLE`, `ADVECT`, `SURFACE`, `MASS`, `VEIL/RAIN_WALL`, `PLANE/WATER`,
  and rough `PORTAL/VORTEX`: `param_d` phase.
- Legacy `SCATTER`, bounds and inactive variants stay unchanged. A nonzero rate
  is not permission to overwrite their seed, orientation or structural controls.

Rates are byte increments per simulation update; zero holds the authored phase.
A repeating byte sequence is not a universal promise of smooth visual looping.
The Sky Ownership capability scene composes CITY bounds with fixed-seed stars;
Toxic Wasteland reuses phased points for fallout brightness, not falling geometry.
These are examples; games may deliberately animate any other authored parameters.

## Small regression check

```sh
rustc --edition=2024 --test examples/3-inspectors/epu-showcase/src/constants.rs -o tmp/epu-phase-test.exe
tmp/epu-phase-test.exe
```

The check covers active/inactive variants, phase offsets, zero rates, counter
wrap, seed and sibling-bit preservation. It is a host logic check, not GPU,
cartridge, rollback, human visual or performance acceptance.
