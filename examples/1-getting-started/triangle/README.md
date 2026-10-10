# One triangle: winding, camera and transform

One unlit POS_COLOR triangle is enough to distinguish three common causes of
"my mesh disappeared." There are no external assets or runtime file reads.
The vertex data is original example source, under the repository license.

## Build and run

Requires the Rust `wasm32-unknown-unknown` target and matching `nether` and
`nethercore-zx` executables on PATH. From this directory:

```sh
nether build -p . -o triangle.nczx
nethercore-zx triangle.nczx
nethercore-zx triangle.nczx --headless --replay witness.ncrs --report triangle-state.json --timeout 30
nethercore-zx triangle.nczx --replay capture.ncrs --scale 1
```

For relocation, copy only `triangle.nczx` to a directory outside this checkout
and launch it there. Use absolute executable/replay paths for automated checks.
Keep generated cartridges, reports and captures as local QA outputs, not source.

## Controls and experiment

- **A / Z:** swap the G and B vertices, reversing winding.
- **B / X:** move the camera between +Z and -Z, still looking at the origin.
- **X / C:** cycle model Y rotation through 0, 45 and 90 degrees.
- **L1 / Q:** toggle back-face culling versus no culling.
- **Y / V:** reset every choice to the initial visible triangle.
- **F3:** inspect the named state without changing it.

Start with R,G,B order, a +Z camera and back culling. Reversing the winding hides
the face; viewing it from -Z reveals it again. Swapping back hides it until culling
is disabled. Reset, then rotate: 45 degrees narrows the projected face; 90 degrees
is edge-on, not an absent vertex buffer. Rotation happens in render, but the
selected rotation index changes only in update. No simulation state is mutated
by rendering.

## Evidence and limitations

`witness.ncrs` checks the named state transitions. `capture.ncrs` requests nine
native views at frames 0,5,10,15,20,25,30,35,40. The last two resets must restore
the initial frame. The native player prints the screenshot paths.

With Pillow installed, pass those nine **scale-1** captures in that order to
`python check_pixels.py <nine image paths>`. The check requires empty geometry
regions for culled/edge-on states, nonempty visible states, a narrower 45-degree
view and identical reset pixels. Headless assertions alone do not establish
visibility, and this pixel check does not establish human visual acceptance.

Replay bumper names are `l` and `r`, although the on-screen labels say L1/R1.
This deliberately teaches one triangle, not lighting, retained meshes or asset
loading; continue to the graphics-primitives lessons for those subjects.
