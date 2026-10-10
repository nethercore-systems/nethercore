# Sprite pivots: choose the anchor, not a moving top-left

Three views compare a centered arrow, an edge-hinged arrow, and two linked arrows.
White crosses mark the fixed screen anchors and the computed arm joints. The tiny
asymmetric arrow atlas includes an off-center red notch, making orientation and
UV-region errors visible. Source and procedural artwork use the repository
license; there are no external assets. Prerequisite: textured-quad.

## Build and controls

With the Rust `wasm32-unknown-unknown` target and matching tools, from this directory:

```sh
nether build -p . -o sprite-pivot-lab.nczx
nethercore-zx sprite-pivot-lab.nczx
```

- **A / Z:** advance 45 degrees clockwise; eight presses wrap to the start.
- **B / X:** switch the child's relative turn between +90 and -90 degrees.
  The centered and hinged sprites must not change.
- **X / C:** choose the cyan/gold atlas region, without changing origin or rotation.
- **Y / V:** reset angle, child turn and UV selection.
- **F3:** inspect angle step, child-turn flag, UV region, elbow and tip coordinates.

Controller button names appear before the slash, player-1 keyboard keys after it.
Holding a button does not repeat. The scene waits for input; deterministic angle
stops let you inspect exact cardinal and diagonal views without chasing motion.

## The actual coordinate contract

For `draw_sprite_ex`, **x/y is the screen location of the selected origin**.
The origin offsets are **pixels at the drawn sprite size**, not source texels,
normalized fractions or a second screen position. Thus a 160×80 draw uses:

- Center pivot: origin `(80,40)` at screen `(176,270)`.
- Left-edge pivot: origin `(0,40)` at screen `(480,270)`.

Do not first subtract a rotated origin yourself: the draw call already anchors it.
UVs still use normalized source regions; the two 16×8 tiles occupy halves of a
32×8 texture. The off-center red notch belongs to the image, not the anchor.

The arm is just two explicit guest calculations: place the elbow 80 pixels along
the parent angle, then place the tip 60 pixels along **parent + relative child**.
The child draw is anchored at that elbow. `geometry.rs` uses eight unit directions
to keep this finite lesson's math inspectable. Arbitrary continuous angles would
need a trig implementation; no scene graph, new console API or transform system
is introduced. Render reads state and draws; update owns all controls.

## Executable evidence

```sh
rustc --edition 2021 --test src/geometry.rs -o /absolute/outside/path/geometry-tests
/absolute/outside/path/geometry-tests
nethercore-zx sprite-pivot-lab.nczx --headless --replay witness.ncrs --report state.json --timeout 30
nethercore-zx sprite-pivot-lab.nczx --replay capture.ncrs --scale 1
python ../../1-getting-started/check_foundations.py --projects 2-graphics/sprite-pivot-lab --nether /absolute/path/nether --player /absolute/path/nethercore-zx --output /empty/directory/outside/checkout --render
```

Use `.exe` on Windows. Python 3.11+ and Pillow are required for the shared native
checker. It builds/packs outside the checkout, checks 68 ticks/19 assertions,
rejects a false assertion, validates fourteen native 960×540 captures and runs
120 input frames through forced rollback. The image oracle uses independent trig
with a declared three-pixel raster tolerance for shaft/anchor probes; it also
checks visible coverage, all eight angles, both UV regions, isolated child turns,
360-degree wrap and exact reset. The pure geometry test checks exact quarter-turn
joints, opposite child direction and diagonal unit length.

Copy only the cartridge elsewhere for relocation. Reports/captures are local QA
outputs. Technical checks are not human approval, online netplay or committed
remote delivery; those are separate gates.
