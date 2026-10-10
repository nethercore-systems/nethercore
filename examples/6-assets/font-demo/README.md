# Font Demo

A real custom-font/menu lesson: one ROM texture, two font descriptors, measured
alignment, guest word wrapping, a focus cursor, and an explicit missing-glyph
comparison. It is not an atlas-only preview or a UI framework.

## Build and run

Requirements: Rust with `wasm32-unknown-unknown`, Python 3.11+ available as
`python`, and the repository-matched `nether` / `nethercore-zx` tools.

```sh
nether build
nether run --no-build
```

`nether build` invokes `python build.py --release`: generate the atlas/widths,
then compile and pack. `nether build --debug` omits `--release`; Cargo metadata
selects the corresponding WASM. Do not pack an old WASM after changing metrics.
No build-time Pillow, external font download, private sibling path or asset
server is required. The packed cartridge contains its texture and metrics.

## Controls

- **A**: fixed / variable advance, using `load_font` / `load_font_ex`.
- **B**: left / center / right alignment, calculated from `text_width`.
- **X**: 280 / 160 pixel wrap width. The vertical guide is the actual boundary.
- **D-pad Up/Down**: focus among the three illustrative menu entries; wraps.
- **Y**: exact reset. Holding a button does not repeat an edge-triggered action.

Use the player's existing mapped controller buttons; all state changes occur in
`update`, never `render`. The menu demonstrates focus, not game navigation.

## How it works / limits

`rom_texture("font_atlas")` loads the atlas, then both font descriptors are
created **once in init**. `font_bind(0)` restores the built-in font for labels.
The custom atlas covers ASCII 32-126 in a 128x48 texture, sixteen 8x8 cells per
row. The variable font's grid stride is **the maximum advance (8)**, not each
individual glyph's width; glyph ink is left-trimmed and measured independently.

`text_width` measures the bound font, including blank advances. Guest code uses
that result to align and wrap; there is no native wrap/alignment/focus API.
`layout.rs` advances on a too-wide UTF-8 character and breaks oversized words,
so even a zero-width line cannot hang. Its small fallback buffer is caller-owned;
excess input truncates at complete characters. This is not kerning, shaping,
localization or a general document layout system.

The `AΩB` row intentionally shows native custom-font behavior: an unsupported
codepoint is skipped **with zero advance**, so it draws `AB`. The next row runs
guest ASCII fallback and draws `A?B`. Both widths come from the real font API.
`rom_font` is not used: it returns an atlas handle, and the manifest does not
have a complete BMFont/fonts pipeline.

## Editable source and provenance

`build.py` reuses the repository's authored bitmaps in
`nethercore-zx/src/font.rs`, under the repository license. It crops blank left
columns, preserves glyph ink, and derives advances with a trailing blank column
when the 8-pixel cell permits it. Edit that source or the crop/advance recipe;
there is no OS-font or undocumented generated-asset dependency. Generated
`assets/font.png` and `assets/widths.bin` are intentionally ignored. The source
bitmap file is a build input and must be bound in qualification receipts.

## Runnable checks

```sh
rustc --edition 2021 --test src/layout.rs -o target/font-layout-tests
# Run the resulting host executable (with .exe on Windows).
nether replay run witness.ncrs --rom font-demo.nczx --headless --report report.json
python check_pixels.py --state report.json
```

Use the existing `examples/1-getting-started/check_foundations.py` with
`--projects 6-assets/font-demo --render`, explicit CLI/player paths and an empty
output directory outside the verification source export. Pillow is required only
for image QA. It builds, relocates, executes all scripted controls, rejects a
false state assertion, checks the actual native glyph pixels and runs forced
rollback. `check_pixels.py` independently checks every recorded state field and
exact nearest-filtered coverage: fixed/variable glyphs, all three alignments,
both wraps, missing/fallback behavior, focus wrap/held input, and exact reset.

Qualification/delivery status belongs in `docs/plans/zx-examples-development.md`.
Build or machine-image success does not imply human visual acceptance.
