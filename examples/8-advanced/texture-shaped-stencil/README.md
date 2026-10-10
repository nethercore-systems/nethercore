# Texture-shaped stencil (71)

The cyan source sprite has a transparent central window, an asymmetric small hole,
and three fractional-alpha edge bands. On the right the same fragment coverage
writes stencil 1 without drawing color. Oversized gold and purple content then
passes only where that reference survived. The source and result intentionally
have different colors so a texture merely drawn on top cannot masquerade as a
mask.

## Controls

- **A / Z:** cycle uniform opacity 15/15, 8/15, 0/15.
- **B / X:** cycle four Bayer offsets: (0,0), (1,2), (2,0), (3,2).
- **X / C:** replace only the right mask with its bounding rectangle, an explicit
  bad example that fills the transparent window and loses the fractional fringe.
- **Y / V:** reset all three controls; reset wins over simultaneous input.

The letter after each slash is the default keyboard mapping. Held buttons do
not repeatedly toggle. The mask and label state live in rollback-covered memory;
render only reads them.

## Why this works

`load_texture` loads an authored 32x32 RGBA8 tile once during init. `state::pixels`
is its complete editable generator: no private source, asset download or external
art license is needed. The texture occupies 4096 RGBA8 bytes before any host
allocation overhead; it is deliberately tiny, not an invitation to embed large
assets in snapshotted guest RAM.

An unconditional ZERO pass first clears the right viewport's stencil footprint.
An ALWAYS/REPLACE pass then draws the textured sprite. Color writes are suppressed
for that stencil-writing pass, but the normal quad fragment shader still runs:
texture alpha multiplied by material alpha and `uniform_alpha / 15` must exceed
the pixel's 4x4 Bayer threshold. Discarded fragments do not write stencil. The
current quad shader and `common/00_bindings.wgsl` define this contract.

Nearest sampling makes texel boundaries exact. The source and mask are ten screen
pixels per texel and their viewport origins differ by a multiple of four, so they
share Bayer phase. Alpha values 64, 128 and 192 are genuine RGBA8 fractions, not
ideal quarter/half/three-quarter values; threshold equality discards. The checker
uses an exact rational comparison rather than a fitted image threshold.

After marking, uniform opacity returns to 15 and dither offset to zero: only the
stored mask clips the replacement content, with no second dither multiplication.
Separate passes keep gold before purple despite batching. The viewport and normal
pass are restored before labels. The empty 0/15 case and box-mask toggle expose
missing alpha handling or unwanted state carry-over.

This is screen-door coverage, not framebuffer blending or automatic high-quality
antialiasing. There is no smooth alpha composite or new stencil API.

## Reproduce

```sh
nether build -p examples/8-advanced/texture-shaped-stencil -o texture-stencil.nczx
nethercore-zx texture-stencil.nczx
rustc --edition 2021 --test examples/8-advanced/texture-shaped-stencil/src/state.rs -o texture-stencil-state
./texture-stencil-state
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/texture-shaped-stencil --render
python examples/8-advanced/texture-shaped-stencil/check_pixels.py --state OUTSIDE_SOURCE/texture-shaped-stencil.json
```

The frozen oracle checks every panel pixel, including holes, dither samples,
texture boundaries and borders, across all three opacities, four phases and both
mask types. It also checks exact reset/held state and unchanged labels. The shared
runner supplies relocation, typed replay snapshots, deliberate false assertions
and at least 120 scripted forced-rollback frames. Repeated idle captures and
substituting the native bounding-box captures for the sprite masks must fail.
Pillow is QA-only. Actual receipts, committed-source, remote/CI and separate human
review status belong to the curriculum ledger; these commands are a recipe, not
a claim that an unexecuted check passed.
