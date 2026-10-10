# Viewport camera lab (67)

Two side-by-side or four tiled views show one shared cube, floor, four colored
spheres and an off-axis tower. Each view has its own camera. P1/P2/P3/P4 and the
bottom-right BR labels use **viewport-local** pixel coordinates, not screen-wide
fractions. The asymmetric world and per-player camera labels expose orientation.

## Controls

- **P1 A:** toggle 2P / 4P. Holding does not repeatedly toggle.
- **P1 B:** circle masks, deliberately extending beyond viewport bounds.
- **P1 L1:** replace the world with solid diagnostic ink, also oversized.
- **P1 X:** play/pause shared animation. Starts paused.
- **P1 Y:** reset mode, masks, ink, time and all camera angles.
- **Each player's left stick:** rotate only that camera, proportionally up to
  2 degrees per tick; normalized magnitudes at or below 0.1 are ignored.

All four local controller slots are exercised by the scripts. This is not a
networking, joining or four-client connectivity demonstration.

## Why the rendering is isolated

The 960x540 frame reserves y=0..63 and y=492..539 for a full-screen HUD. The
remaining 960x428 area is partitioned into two 480x428 or four 480x214 viewports.
The camera aspect follows those dimensions. 3D clip coordinates map through that
viewport; 2D positions are local pixels and retain their pixel size. Viewport
scissors apply to scene geometry, text and stencil writes alike.

With masks enabled, all masks are written **before** scenes, using distinct
references. Unclipped later masks would overwrite earlier views. Each view then
tests its own reference; a normal pass explicitly removes the test for local UI.
The final HUD restores the full-screen viewport and normal pass. Screen quads
use painter ordering, so UI has explicit layers. A depth clear would affect the
whole attachment, **not** merely the current viewport; disjoint views here do
not require one. The SDK circle is 16 radial rectangles, not an exact disk.

## Reproduce

```sh
nether build -p examples/8-advanced/viewport-test -o viewport-test.nczx
nethercore-zx viewport-test.nczx
rustc --edition 2021 --test examples/8-advanced/viewport-test/src/state.rs -o viewport-state
./viewport-state
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/viewport-test --render
python examples/8-advanced/viewport-test/check_pixels.py --state OUTSIDE_SOURCE/viewport-test.json
```

No external assets, private repositories or runtime resource allocations are
needed. Public source/toolchains suffice. Pillow is a QA-only dependency.
`witness.ncrs` / `capture.ncrs` cover both layouts, masks and ink, held A,
independent cameras, below-deadzone input, animation, freeze and reset. The shared
runner checks build/pack/relocated launch, rejects a false assertion and replays
all controls through at least 120 forced-rollback input frames.

The native gate tests all viewport borders, local label positions and all
non-UI ink/mask pixels. Its analytic circle boundary band is one pixel; camera
state tolerance is 0.001 degree. Unchanged views, resets, freeze and global HUD
are byte-exact. Color classes are fixed in advance, not shader-RGB parity.
Repeated idle captures must fail. Local readiness, committed-source proof,
publication/CI and human acceptance remain separate gates.
