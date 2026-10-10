# Stencil boolean masks (69)

Three panels show FIRST, SECOND and the actual stencil result. Four operations
use the existing eight-bit stencil attachment, not a Boolean geometry library.

## Controls

- **A:** union → intersection → FIRST minus SECOND → parity/XOR → union.
- **B:** overlap → nested → disjoint placement of the authored shapes.
- **X:** swap operands. Union/intersection/parity are unchanged; subtraction is not.
- **Y:** reset operation, layout and operand order. Wins over other held buttons.

All toggles are press edges. State is rollback-covered; rendering cannot advance
it. The nested subtraction/parity cases produce holes; disjoint intersection is
correctly empty, not a failed draw.

## Actual recipes

First rasterize ZERO over the result viewport; this is not an independent
stencil-clear API. Build the complete mask before drawing the final purple ink:

- **Union:** write 1 for each shape, then test EQUAL 1.
- **Intersection:** write 1 for FIRST; draw SECOND with EQUAL 1 and
  INCREMENT_CLAMP, then test EQUAL 2.
- **Subtraction:** write 1 for FIRST; rasterize ZERO for SECOND, then test EQUAL 1.
- **Parity:** INVERT once per shape, then test EQUAL 255. Overlap is inverted
  twice and returns to zero.

Comparison and REPLACE share the same reference. EQUAL 1 plus REPLACE cannot
write 2; conditional increment is intentional. Conditional mask passes **can
write color**, so intersection uses the exact flat panel background color and
all final ink is drawn afterward. Unconditional stencil-writing passes suppress
color in the current renderer. Read/write masks cover the full byte; there is
no invented per-bit masking feature here.

Each shape is one quad. Reusing a tessellated circle built from overlapping
rectangles would invert/increment some pixels multiple times. For other shapes,
prove their fragment coverage before copying the parity/count recipe. Normal
passes restore non-stenciled borders, labels and the full-screen HUD.

## Reproduce and falsify

```sh
nether build -p examples/8-advanced/stencil-boolean-masks -o boolean.nczx
nethercore-zx boolean.nczx
rustc --edition 2021 --test examples/8-advanced/stencil-boolean-masks/src/state.rs -o boolean-state
./boolean-state
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/stencil-boolean-masks --render
python examples/8-advanced/stencil-boolean-masks/check_pixels.py --state OUTSIDE_SOURCE/stencil-boolean-masks.json
```

The frozen native oracle tests every interior pixel and all panel borders using
an independent Boolean truth table and integer pixel-center membership. It
covers all operations, layouts and operand orders, exact reset/hold/freeze,
commutativity and noncommutative subtraction. There is no fitted tolerance or
ignored shape-boundary band. The input oracle checks typed pre/post snapshots.
The shared runner also rejects a false assertion, relocates the ROM and runs
at least 120 frames of forced rollback. Repeated idle images must fail.

There are no external assets or private runtime dependencies. Pillow is only
used for QA. Readiness, committed-source proof, remote delivery/CI and human
review are recorded separately in the curriculum ledger.
