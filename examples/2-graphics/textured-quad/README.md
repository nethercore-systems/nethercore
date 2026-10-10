# Textured quad: UVs and sampling

One procedural 16×16 RGBA texture contains four numbered checker tiles:
**1 red top-left, 2 green top-right, 3 blue bottom-left, 4 gold bottom-right**.
The left reference stays nearest-filtered with the full atlas. The right subject
changes independently; asymmetric colors and digits expose flipped UVs that a
symmetric checker alone would conceal. No external assets or private dependencies;
the generated texture and code use the repository license.

## Build and controls

Requires the Rust `wasm32-unknown-unknown` target and matching `nether` / `nethercore-zx`:

```sh
nether build -p . -o textured-quad.nczx
nethercore-zx textured-quad.nczx
```

Copy only the cartridge elsewhere and run it there to verify relocation.

- **A / Z:** nearest ↔ linear on the subject only. Holding A does not repeat.
- **B / X:** cycle full atlas → tiles 1, 2, 3, 4 → full atlas.
- **L1 / Q:** toggle half-texel UV inset.
- **Y / V:** restore nearest, full atlas and no inset.
- **F3:** inspect `filter`, `region` and `inset`.

`load_texture` runs once in init. Every render rebinds the atlas, sets each draw's
filter and explicitly restores nearest for labels. Mutable controls live in
rollback-covered memory; render only reads them. The manifest selects one local
player, 60 Hz, mode 0, uncompressed texture data, and disables netplay.

## Try the discriminating experiment

Press A, then B. Linear sampling at tile 1's right edge includes green texels
from tile 2. Press L1: moving each UV endpoint inward by **half one source texel**
removes that neighboring color. The UI prints the actual UV origin and extent.
This deliberately narrows/rescales the sampled area; it is not a hidden sampler
clamp or a substitute for padded atlas gutters. Nearest preserves sharp source
texels; linear interpolates adjacent values. Neither toggling filtering nor this
simple inset supplies mipmaps or per-tile wrap/clamp behavior.

## Reproducible evidence

From this directory:

```sh
nethercore-zx textured-quad.nczx --headless --replay witness.ncrs --report state.json --timeout 30
nethercore-zx textured-quad.nczx --replay capture.ncrs --scale 1
```

The shared qualification runner discovers all ten captures from actual player logs
and passes their ordered paths to `check_pixels.py`:

```sh
python ../../1-getting-started/check_foundations.py --projects 2-graphics/textured-quad --nether /absolute/path/nether --player /absolute/path/nethercore-zx --output /empty/directory/outside/checkout --render
```

Python 3.11+ and Pillow are required for the native pixel checks. The runner
builds/packs into the outside directory, executes 48 ticks and 15 state assertions,
rejects a deliberately false assertion, checks ten actual 960×540 native images,
and executes the same controls through a 120-frame forced-rollback run. Its receipt
binds sources, tools, cartridge and captures. Pixel checks cover fixed reference,
four corners/tiles, numeral ink, filter change, edge bleed/inset and exact reset.

That is technical evidence, not online transport, human approval, or delivery
of uncommitted changes. Native media/reports remain local QA artifacts.
