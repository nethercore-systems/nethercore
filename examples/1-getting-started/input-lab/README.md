# Input lab: four session slots

See the controller state delivered by ZX: button edges, both sticks, both
triggers and local-session ownership. All visuals are authored in source; there
are no external assets. The code follows the repository license.

## Build and run

Requires Rust with `wasm32-unknown-unknown` and matching `nether`/`nethercore-zx`
executables on PATH. From this directory:

```sh
nether build -p . -o input-lab.nczx
nethercore-zx input-lab.nczx
nethercore-zx input-lab.nczx --headless --replay witness.ncrs --report input-state.json --timeout 30
nethercore-zx input-lab.nczx --replay capture.ncrs --scale 1
```

Copy only the cartridge outside the checkout and launch it from that directory
to check relocation. Automated replay/executable paths must then be absolute.
Keep reports and screenshots outside tracked source.

## Controls and reading the display

The normal host keyboard mapping is shown in the footer: arrows for the D-pad,
ZXCV for A/B/X/Y, Q/E for L1/R1, WASD and IJKL for the sticks, U/O for triggers,
and R/Y for L3/R3. Select/Shift resets press counters; `SEL` is the compact button
label. F3 exposes exact watched values.

- Blue button fill means held; green means a new press; pink means release.
  An edge overrides the held color for that tick. Holding does not add presses.
- Blue stick dots are the values **already delivered by the host API**. They are
  not raw hardware samples. Gold rings apply the guest's additional radius-0.20
  hard radial deadzone in `deadzone`; magnitudes outside that circle are preserved,
  not rescaled. Both stick diagrams use the same filter.
- Trigger bars and numbers show values in 0..1. Stick components use -1..1, with
  positive Y down in this display.
- "Local", "remote" and "inactive" describe session slots, not whether a physical
  controller is plugged in. The included four-player replay makes all four local.

Rendering reads a snapshot populated in update. Card backgrounds use z-index 0,
controls 1 and labels 2. Same-pass commands can be texture-sorted, so submission
order alone must not be used to layer overlapping UI.

## Runnable witnesses

`witness.ncrs` exercises press/hold/release, simultaneous buttons on four slots,
reset, sub-deadzone and nonzero sticks, and trigger endpoints. Replay bumper
names are `l`/`r`, not the display labels L1/R1. Native `capture.ncrs` requests
eight corresponding views; it does not execute the headless assertions.

With Pillow installed, run `python check_pixels.py <capture paths>` on the
scale-1 native screenshots. It requires both raw/filtered markers for both sticks
in every panel, and fails on the original missing-panel layering defect.
This does not certify physical hardware, online netplay, text legibility or human
taste. Inspect the actual views too; real network behavior belongs to netplay-demo.
