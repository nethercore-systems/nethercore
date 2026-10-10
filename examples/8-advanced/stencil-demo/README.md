# Stencil demo (66)

Four modes reuse one cube/floor/orbiting-sphere scene. The on-screen row lists
pass order; **L1** replaces geometry with solid mask ink through the same stencil
passes. This is a four-mode introduction, not an advanced stencil curriculum.

## Controller and reset

- **A:** next mode, wrapping through inside-circle, outside-circle vignette,
  diagonal warm/cool split and alternate-camera portal. Holding does not repeat.
- **B:** toggle portal depth clear. In **3D scene** view, leaving the near camera's
  depth in place incorrectly hides parts of the farther alternate camera.
- **X:** play/pause. Starts paused at an exactly repeatable state.
- **Y:** reset everything, including mode, angle, time, view and depth-clear flag.
- **L1:** 3D scene / solid mask ink. Ink tests stencil regions; screen-space quads
  always pass depth comparison, so the clear toggle does not change their pixels.
- **D-pad left/right:** rotate 15 degrees per press; X also animates the scene.

## What the passes actually do

1. The circle writer disables color and depth writes, then EQUAL admits the scene.
2. NOT_EQUAL paints an **opaque** dark vignette outside a larger circle. This is
   not framebuffer alpha blending.
3. A non-indexed screen-projected triangle writes one diagonal half; EQUAL and
   NOT_EQUAL draw the same scene with different tints in disjoint regions.
4. The base camera draws first, including its depth. The ring writes no depth;
   one interior stencil write defines the aperture. An alternate camera then
   uses that mask, with optional depth clear. The two-camera depth values are not
   interchangeable. The clear affects the **whole attachment**, not just the mask.
5. A normal pass removes the stencil test before an explicitly layered HUD.

`draw_circle` is the SDK's union of 16 radial rectangles, not an exact disk.
This portal is a camera-composition example, not RTT, recursive portals or a
proof of arbitrary near-plane clipping. No console API or renderer change is
needed by this lesson. Meshes are generated only during init; render never
advances simulation, and controller-edge state is in rollback-covered memory.

## Reproduce

Use `nether` and `nethercore-zx` built from the public repository. No asset
producer, private sibling repository or downloaded content is required.

```sh
nether build -p examples/8-advanced/stencil-demo -o stencil-demo.nczx
nethercore-zx stencil-demo.nczx
rustc --edition 2021 --test examples/8-advanced/stencil-demo/src/state.rs -o stencil-state
./stencil-state
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/stencil-demo --render
python examples/8-advanced/stencil-demo/check_pixels.py --state OUTSIDE_SOURCE/stencil-demo.json
```

`witness.ncrs` / `capture.ncrs` exercise all four modes, held A/right, both views,
depth clear off/on, mode wrap, rotation, animation, freeze and reset. The shared
runner builds/packs, launches from a relocated working directory, rejects a
false assertion, checks original-resolution native captures and runs all inputs
through a forced-rollback run (at least 120 input frames). Pillow is used only
for QA. The small state test also checks reset precedence and integer wrap.

The pixel gate declares a one-pixel analytic raster boundary band and broad,
fixed color classes before native execution; it does not claim RGB shader parity.
It checks the full non-HUD scene region, exact unchanged header/control pixels,
exact reset/freeze frames and a visible 3D depth-clear difference confined to the
aperture. Repeated idle frames must fail. Native review media, committed-source
qualification, publication/CI and human acceptance remain separate gates.
