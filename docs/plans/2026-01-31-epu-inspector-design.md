# EPU Inspector - Editor Playground Design

## Overview

Refactor the `epu-inspector` example into a live EPU editing playground. Developers can tweak layer values in real-time via the debug panel, isolate individual layers for inspection, and export hex values for preset authoring.

## Location

New project at `examples/3-inspectors/epu-inspector/`

## Core Interaction Model

### Layer Selection & Editing

- Debug panel has a `layer_index` slider (1-8) selecting which layer to edit
- When layer index changes, all edit fields pull data from that layer's current `[hi, lo]` values
- Edits write back to the selected layer in real-time (live preview)

### Isolation Mode

- Boolean `isolate_layer` toggle in debug panel
- When ON: only the selected layer renders (other 7 layers set to NOP)
- When OFF: full 8-layer composition renders normally
- Useful for seeing exactly what one layer contributes without interference

### Data Flow

```
Edit field changed
    → Pack updated values into [hi, lo]
    → Write to layers[layer_index - 1]
    → EPU re-renders with new config
    → Visible immediately in viewport
```

## Debug Panel Fields

All fields registered at init, operating on the selected layer's unpacked state.

### Control Fields

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| layer_index | u8_range | 1-8 | Which layer to edit |
| isolate_layer | bool | - | Render only selected layer |
| show_hints | bool | - | Toggle param hint text |

### Hi Word Fields (bits 127-64)

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| opcode | u8_range | 0-31 | Layer operation type |
| region_sky | bool | - | Apply to sky/ceiling region |
| region_walls | bool | - | Apply to walls/horizon region |
| region_floor | bool | - | Apply to floor/ground region |
| blend | u8_range | 0-7 | Compositing mode |
| domain_id | u8_range | 0-3 | Coordinate domain (unpacked from meta5) |
| variant_id | u8_range | 0-7 | Opcode variant (unpacked from meta5) |
| color_a | color | RGB24 | Primary color |
| color_b | color | RGB24 | Secondary color |

### Lo Word Fields (bits 63-0)

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| intensity | u8 | 0-255 | Layer brightness |
| param_a | u8 | 0-255 | Opcode-specific parameter |
| param_b | u8 | 0-255 | Opcode-specific parameter |
| param_c | u8 | 0-255 | Opcode-specific parameter |
| param_d | u8 | 0-255 | Phase/animation parameter |
| azimuth | f32_range | 0-360 | Direction horizontal angle (converted) |
| elevation | f32_range | -90 to 90 | Direction vertical angle (converted) |
| alpha_a | u8_range | 0-15 | color_a alpha |
| alpha_b | u8_range | 0-15 | color_b alpha |

### Actions

| Field | Type | Description |
|-------|------|-------------|
| export | action | Print all 8 layers as hex to console |

## In-Game Hint Text

- Rendered via `draw_text()` as overlay (not in debug panel)
- Dynamic: shows param meanings for the **currently selected opcode only**
- Controlled by `show_hints` toggle
- Updated each frame based on current opcode value

Example hints per opcode:
```
SCATTER: a=count, b=size, c=twinkle
FLOW: a=scale, b=speed, c=octaves
DECAL: a=shape, b=size, c=feather
SILHOUETTE: a=height, b=jitter, c=density
```

## Export Format

Button triggers export of all 8 layers. Output format (printed to debug console):

```
[0x08C0FFAABB00FF00, 0xFF201040800000FF],
[0x..., 0x...],
[0x..., 0x...],
[0x..., 0x...],
[0x..., 0x...],
[0x..., 0x...],
[0x..., 0x...],
[0x..., 0x...],
```

Each line is one layer's `[hi, lo]` pair. Developer copies and formats as needed.

## Helper Functions

### Packing/Unpacking

```rust
/// Unpack 128-bit layer into editor-friendly struct
fn unpack_layer(hi: u64, lo: u64) -> EditorState

/// Pack editor state back into 128-bit layer
fn pack_layer(state: &EditorState) -> (u64, u64)
```

### Direction Conversion

```rust
/// Convert octahedral-encoded direction to human angles
fn octahedral_to_angles(dir16: u16) -> (f32, f32)  // (azimuth, elevation)

/// Convert human angles to octahedral encoding
fn angles_to_octahedral(azimuth: f32, elevation: f32) -> u16
```

## EditorState Struct

```rust
struct EditorState {
    // Hi word
    opcode: u8,
    region_sky: bool,
    region_walls: bool,
    region_floor: bool,
    blend: u8,
    domain_id: u8,
    variant_id: u8,
    color_a: u32,  // RGB24 stored as u32 for color picker
    color_b: u32,

    // Lo word
    intensity: u8,
    param_a: u8,
    param_b: u8,
    param_c: u8,
    param_d: u8,
    azimuth: f32,    // Converted from direction
    elevation: f32,  // Converted from direction
    alpha_a: u8,
    alpha_b: u8,
}
```

## Implementation Notes

### Init

1. Create 8-layer EPU config array (start with simple RAMP or NOP)
2. Initialize EditorState from layer 0
3. Register all debug fields pointing to EditorState fields
4. Register export action

### Update Loop

1. Detect if `layer_index` changed → unpack new layer into EditorState
2. Detect if any EditorState field changed → repack into layers array

### Render Loop

1. If `isolate_layer`: create temp config with only selected layer, others NOP
2. Call `epu_set()` with config
3. Draw 3D object (sphere/cube for reference)
4. Call `draw_epu()`
5. If `show_hints`: draw param hints for current opcode

## Project Files

```
examples/3-inspectors/epu-inspector/
├── Cargo.toml
├── Cargo.lock
├── nether.toml
└── src/
    └── lib.rs
```

## Out of Scope

- Animation speed editing (users can manually tweak param_d)
- Dynamic per-opcode field labels (init-time registration constraint)
- Preset save/load system (export hex only)
- Multiple preset slots (single 8-layer config)

## References

- EPU constants: `examples/3-inspectors/epu-showcase/src/constants.rs`
- Debug API: `include/zx/debug.rs`
- Similar inspector: `examples/3-inspectors/mode0-inspector/`
