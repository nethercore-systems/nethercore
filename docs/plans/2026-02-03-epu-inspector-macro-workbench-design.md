#+#+#+#+## EPU Inspector - Macro Workbench + WGSL Contract

### Overview

The current `epu-inspector` example exposes packed EPU fields (`param_a..d`, `meta5`, etc.) directly through the debug panel. This is powerful but too abstract for iterative preset authoring, and it makes it easy for UI hints/macros to drift from the real WGSL implementation.

This plan upgrades `examples/3-inspectors/epu-inspector` into a macro-first “workbench” while preserving raw editing for debugging. It also introduces a strict, parseable WGSL metadata contract so the inspector UI stays correct as opcode implementations evolve.

### Goals

- Macro-first editing: intentful, named controls per opcode/variant/domain.
- Keep raw editing as an escape hatch for low-level debugging.
- Always-correct parameter semantics: the inspector’s labels/ranges/mappings are derived from WGSL, not duplicated by hand.
- Fast iteration: browse opcode/variant, reset-to-good-defaults, and easy export.

### Non-Goals (Initial Scope)

- Persistent preset storage (slots on disk).
- A custom in-game editor UI; the host debug panel remains the primary surface.
- General-purpose debug-panel upgrades for all games (possible follow-up).

### Constraints

- Debug panel registrations are init-time; labels and widget types are effectively static per run.
- `epu-inspector` is `#![no_std]`; codegen must emit no-std-compatible Rust.

---

## Macro-First Editor Model

### State

- `LAYERS: [[u64; 2]; 8]` remains canonical for `epu_set()`.
- `RAW[layer]`: decoded/unpacked layer fields (opcode/region/blend/meta/colors/params/dir/alphas).
- `MACROS[layer][opcode]`: opcode-specific macro structs holding human units and typed choices.
- `EDIT_MODE: Macro | Raw` determines the source of truth.

### Sync Rules

- Macro mode:
  - Debug panel edits update the opcode’s macro struct.
  - The editor derives `RAW[layer]` via `macro_to_raw(...)`.
  - Pack into `LAYERS[layer]`.
- Raw mode:
  - Debug panel edits update `RAW[layer]` directly.
  - Pack into `LAYERS[layer]`.
  - Optionally resync macros via `raw_to_macro(...)` when switching layers/opcodes.

### Debug Panel Layout (High Level)

- `control/`: layer select, isolate toggle, hints toggle, `edit_mode`.
- `browse/`: actions for next/prev opcode, next/prev variant/domain, reset layer, copy/paste layer (in-memory).
- `macro/<OPCODE>/...`: semantic controls for the currently selected opcode (implemented as always-visible groups; only one is “active”).
- `raw/`: full low-level fields (existing fields), for debugging and escape hatch.
- `export/`: actions to print a ready-to-paste Rust preset literal.

---

## WGSL Contract: Parseable Opcode Metadata

### Why

Handwritten hints/macros will drift as WGSL changes. The fix is to treat WGSL as the canonical definition of opcode semantics and generate inspector mapping code from a strict metadata block embedded alongside each opcode.

### Contract Format

Add exactly one metadata block per opcode WGSL file under `nethercore-zx/shaders/epu/{bounds,features}/*.wgsl`:

```wgsl
// @epu_meta_begin
// opcode = 0x0F
// name = PLANE
// kind = radiance
// variants = [TILES, HEX, STONE, SAND, WATER, GRATING, GRASS, PAVEMENT]
// domains = []
// field intensity = { label="contrast", map="u8_01" }
// field param_a   = { label="pattern_scale", unit="x", map="u8_lerp", min=0.5, max=16.0 }
// field param_b   = { label="gap_width", unit="tangent", map="u8_lerp", min=0.0, max=0.2 }
// field param_c   = { label="roughness", map="u8_01" }
// field param_d   = { label="phase", map="u8_01" }
// field direction = { label="normal", map="dir16_oct" }
// field alpha_a   = { label="coverage", map="u4_01" }
// @epu_meta_end
```

Notes:

- This block is machine-parsed; freeform comments remain allowed outside the block.
- Mappings are expressed using a small vocabulary (`u8_01`, `u8_lerp`, `u4_01`, `dir16_oct`, and opcode-specific nibble/bitfield maps as needed).
- The block must declare variant/domain labels if the opcode uses them.

---

## Code Generation Pipeline

### Producer

- Add `examples/3-inspectors/epu-inspector/build.rs`.
- The build script reads the EPU WGSL opcode files and extracts `@epu_meta_*` blocks.
- It emits a generated Rust module (no-std) into `OUT_DIR` (or `src/epu_meta_gen.rs`) containing:
  - Opcode tables: names, kind, variant names, domain names.
  - Per-opcode field specs: label/unit, mapping kind, recommended min/max/step.
  - Convenience helpers for showing semantics in the in-game overlay.

### Consumer

- `epu-inspector` includes the generated module and uses it to:
  - Drive the on-screen “parameter semantics” overlay (authoritative labels/ranges).
  - Provide macro-to-raw defaults and clamping.
  - Validate user edits (e.g., clamp variant_id to available variants).

### Build Failures (Drift Prevention)

The build script must fail (panic) if:

- Any opcode file is missing a metadata block.
- An opcode has multiple metadata blocks.
- Any field mapping is unknown/invalid.
- Variant/domain label counts exceed representable IDs (variant 0..7, domain 0..3).

---

## Testing Strategy

- Host-side parser tests (in `nethercore-zx` or a small tool crate): ensure every opcode file has a valid metadata block.
- Roundtrip invariants (unit tests in host code):
  - `raw -> pack -> unpack == raw` for representative values.
  - `macro -> raw -> macro` stability for macro defaults and common edits.
- Manual QA checklist:
  - For each opcode, flip through variants and confirm macro labels match on-screen semantics overlay.
  - Exported preset compiles when pasted into `examples/3-inspectors/epu-showcase/src/presets.rs`.

---

## Follow-Ups

- Clipboard-friendly export (host debug panel enhancement) if log-copy is still too slow.
- Persistent preset slots using `env save/load/delete`.
