# Stencil region stack (70)

Two mirrored viewports use two or three nested full-byte stencil references.
After colored rings are painted, white strokes prove that popping a child has
restored its parent's test region. Finally, reference 1 is reused for a green
sibling. Reusing it without releasing the old parent visibly floods that parent.

## Controls

- **A:** two/three levels; held input does not repeatedly toggle.
- **B:** deliberately omit/restore root cleanup. The bad case paints the old
  parent green, demonstrating the leak rather than hiding it.
- **X:** run/pause alternating child and sibling positions. Starts paused.
- **Y:** reset depth, cleanup, pause and simulation tick; takes priority.

The right viewport mirrors the geometry and uses the opposite motion phase.
Both views share only the rollback-covered scalar state, not stencil ownership.

## Fixed stack recipe

This is a bounded rectangle recipe, not an arbitrary scene-graph allocator:

1. Rasterize ZERO over the selected viewport.
2. Push nested references 1, 2 and optionally 3 with unconditional REPLACE.
   Authored rectangles are strictly contained; unit and native tests check that
   invariant. Arbitrary overlapping/concave geometry needs an intersection recipe.
3. Paint the disjoint equal-reference rings. Reference 3 is the deepest child.
4. Pop the deepest child by rasterizing ZERO over it, then REPLACE the parent's
   whole rectangle with its original reference. Paint a white parent-test stroke.
5. Repeat for the next parent. Existing colored child pixels remain visible:
   these unconditional stencil writes do not write color.
6. ZERO the root, then reuse reference 1 for a disjoint green sibling. **B** skips
   only root cleanup to expose stale-reference leakage. Finally ZERO the sibling.

Do not substitute conditional EQUAL + DECREMENT after painting: the current
renderer can write color during that pass and damage already painted children.
The silent ZERO/REPLACE restoration avoids that problem. All references use the
full byte, with background 0 and nonzero values 1–255; this example peaks at 3
and reuses released values. There are no per-bit masks or separate clear API.

The scissor is retained for every mask, paint and border. A normal pass and the
full-screen viewport are explicitly restored before global labels. Simulation
is changed only by update; rendering consumes a copy.

## Reproduce

```sh
nether build -p examples/8-advanced/stencil-region-stack -o region-stack.nczx
nethercore-zx region-stack.nczx
rustc --edition 2021 --test examples/8-advanced/stencil-region-stack/src/state.rs -o region-state
./region-state
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/stencil-region-stack --render
python examples/8-advanced/stencil-region-stack/check_pixels.py --state OUTSIDE_SOURCE/stencil-region-stack.json
```

The independent oracle checks every interior pixel, parent strokes, sibling
reuse, both depths, the deliberately broken cleanup case, borders, adjacent
moving frames and exact reset/pause/history equivalence. No geometric edge band
is discarded. Typed pre/post snapshots model update separately, including active
ticks. The shared runner also checks relocation, false-assertion rejection and
120 forced-rollback input frames; repeated idle images must fail. Source-bound
label checks include the maximum tick width. Pillow is QA-only; there are no
external assets or private dependencies. Actual readiness, committed-source,
remote/CI and human-review evidence belong to the curriculum ledger.
