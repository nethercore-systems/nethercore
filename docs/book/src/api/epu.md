# Environment Processing Unit (EPU)

The Environment Processing Unit (EPU) is ZX’s instruction-based procedural environment system. You provide a packed 128-byte configuration (8 × 128-bit instructions) and use `epu_set(config_ptr)` + `draw_epu()` to:

- Render the environment background
- Drive ambient + reflection lighting for lit materials (computed on the GPU)

For canonical ABI docs, see `nethercore/include/zx.rs`. For the opcode catalog/spec, see `nethercore-design/specs/epu-feature-catalog.md`.

---

## FFI

### environment_index

Select which EPU environment (`env_id`) subsequent draw calls will sample for ambient + reflections.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Select the EPU environment ID for subsequent draws (0..255).
fn environment_index(env_id: u32);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Select the EPU environment ID for subsequent draws (0..255).
void environment_index(uint32_t env_id);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Select the EPU environment ID for subsequent draws (0..255).
pub extern fn environment_index(env_id: u32) void;
```
{{#endtab}}

{{#endtabs}}

### epu_set

Store the environment config for the currently selected `environment_index(...)` (no background draw).

To configure multiple environments in the same frame, call `environment_index(env_id)` then `epu_set(config_ptr)` for each `env_id` you use.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Store the EPU config for the current environment_index(...).
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions × (hi u64, lo u64)
fn epu_set(config_ptr: *const u64);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Store the EPU config for the current environment_index(...).
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions × (hi u64, lo u64)
void epu_set(const uint64_t* config_ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Store the EPU config for the current environment_index(...).
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions × (hi u64, lo u64)
pub extern fn epu_set(config_ptr: [*]const u64) void;
```
{{#endtab}}

{{#endtabs}}

### draw_epu

Draw the environment background for the current viewport/pass.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Draw the EPU background for the current viewport/pass.
fn draw_epu();
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Draw the EPU background for the current viewport/pass.
void draw_epu(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Draw the EPU background for the current viewport/pass.
pub extern fn draw_epu() void;
```
{{#endtab}}

{{#endtabs}}

Call `draw_epu()` **after** your 3D geometry so the environment only fills background pixels.

Notes:
- For split-screen, set `viewport(...)` and call `draw_epu()` per viewport.
- The EPU compute pass runs automatically before rendering.
- Ambient lighting is computed and applied entirely on the GPU; there is no CPU ambient query.
- `epu_set(...)` stores a config for the currently selected `environment_index(...)`.

---

## Configuration Layout

Each environment is exactly **8 × 128-bit instructions** (128 bytes total). In memory, that’s 16 `u64` values laid out as 8 `[hi, lo]` pairs.

| Slot | Kind | Recommended Use |
|------|------|------------------|
| 0 | Enclosure | `RAMP` (base enclosure + region weights) |
| 1 | Enclosure | `SECTOR` |
| 2 | Enclosure | `SILHOUETTE` |
| 3 | Enclosure | `SPLIT` / `CELL` / `PATCHES` / `APERTURE` |
| 4–7 | Radiance | `DECAL` / `GRID` / `SCATTER` / `FLOW` + radiance ops (`0x0C..0x13`) |

---

## Instruction Bit Layout (128-bit)

Each instruction is packed as two `u64` values:

### High Word (bits 127..64)

```
bits 127..123: opcode     (5)  - Which algorithm to run
bits 122..120: region     (3)  - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
bits 119..117: blend      (3)  - How to combine layer output (8 modes)
bits 116..112: meta5      (5)  - (domain_id<<3)|variant_id; use 0 when unused
bits 111..88:  color_a    (24) - RGB24 primary color
bits 87..64:   color_b    (24) - RGB24 secondary color
```

### Low Word (bits 63..0)

```
bits 63..56:   intensity  (8)  - Layer brightness
bits 55..48:   param_a    (8)  - Opcode-specific
bits 47..40:   param_b    (8)  - Opcode-specific
bits 39..32:   param_c    (8)  - Opcode-specific
bits 31..24:   param_d    (8)  - Opcode-specific
bits 23..8:    direction  (16) - Octahedral-encoded direction (u8,u8)
bits 7..4:     alpha_a    (4)  - color_a alpha (0=transparent, 15=opaque)
bits 3..0:     alpha_b    (4)  - color_b alpha (0=transparent, 15=opaque)
```

---

## Opcode Map (current shaders)

This is the opcode number. Some opcodes use `meta5` for domain/variant selection; when unused, set `meta5 = 0`.

| Code | Name | Notes |
|---|---|---|
| `0x00` | `NOP` | Disable layer |
| `0x01` | `RAMP` | Enclosure gradient |
| `0x02` | `SECTOR` | Enclosure modifier |
| `0x03` | `SILHOUETTE` | Enclosure modifier |
| `0x04` | `SPLIT` | Enclosure |
| `0x05` | `CELL` | Enclosure |
| `0x06` | `PATCHES` | Enclosure |
| `0x07` | `APERTURE` | Enclosure |
| `0x08` | `DECAL` | Radiance |
| `0x09` | `GRID` | Radiance |
| `0x0A` | `SCATTER` | Radiance |
| `0x0B` | `FLOW` | Radiance |
| `0x0C` | `TRACE` | Radiance |
| `0x0D` | `VEIL` | Radiance |
| `0x0E` | `ATMOSPHERE` | Radiance |
| `0x0F` | `PLANE` | Radiance |
| `0x10` | `CELESTIAL` | Radiance |
| `0x11` | `PORTAL` | Radiance |
| `0x12` | `LOBE_RADIANCE` | Radiance (region-masked) |
| `0x13` | `BAND_RADIANCE` | Radiance (region-masked) |

For full per-opcode packing/algorithm details, see:
- `nethercore-design/specs/epu-feature-catalog.md`
- `nethercore/nethercore-zx/shaders/epu/`

---

## Region Mask (3-bit bitfield)

Regions are combinable using bitwise OR:

| Value | Binary | Name | Meaning |
|-------|--------|------|---------|
| 7 | `0b111` | `ALL` | Apply to sky + walls + floor |
| 4 | `0b100` | `SKY` | Sky/ceiling only |
| 2 | `0b010` | `WALLS` | Wall/horizon belt only |
| 1 | `0b001` | `FLOOR` | Floor/ground only |
| 6 | `0b110` | `SKY_WALLS` | Sky + walls |
| 5 | `0b101` | `SKY_FLOOR` | Sky + floor |
| 3 | `0b011` | `WALLS_FLOOR` | Walls + floor |
| 0 | `0b000` | `NONE` | Layer disabled |

---

## Blend Modes (3-bit, 8 modes)

| Value | Name | Formula |
|-------|------|---------|
| 0 | `ADD` | `dst + src * a` |
| 1 | `MULTIPLY` | `dst * mix(1, src, a)` |
| 2 | `MAX` | `max(dst, src * a)` |
| 3 | `LERP` | `mix(dst, src, a)` |
| 4 | `SCREEN` | `1 - (1-dst)*(1-src*a)` |
| 5 | `HSV_MOD` | HSV shift dst by src |
| 6 | `MIN` | `min(dst, src * a)` |
| 7 | `OVERLAY` | Photoshop-style overlay |

---

## meta5

The 5-bit `meta5` field (hi bits 116..112) is interpreted as:

- `meta5 = (domain_id << 3) | variant_id`
- `domain_id = (meta5 >> 3) & 0b11`
- `variant_id = meta5 & 0b111`

---

## Quick Start

The easiest reference implementation is the EPU showcase presets:
- `nethercore/examples/3-inspectors/epu-showcase/src/presets.rs`
- `nethercore/examples/3-inspectors/epu-showcase/src/constants.rs`

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
// 8 x [hi, lo]
static ENV: [[u64; 2]; 8] = [
    [0, 0], [0, 0], [0, 0], [0, 0],
    [0, 0], [0, 0], [0, 0], [0, 0],
];

fn render() {
    unsafe {
        epu_set(ENV.as_ptr().cast());
        // ... draw scene geometry
        draw_epu();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static const uint64_t env_config[16] = {
    /* hi0, lo0, hi1, lo1, ... */
};

void render(void) {
    epu_set(env_config);
    // ... draw scene geometry
    draw_epu();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const env_config: [16]u64 = .{
    // hi0, lo0, hi1, lo1, ...
};

export fn render() void {
    epu_set(&env_config);
    // ... draw scene geometry
    draw_epu();
}
```
{{#endtab}}

{{#endtabs}}

---

## See Also

- [EPU Environments Guide](../guides/epu-environments.md) - Recipes and examples
- [EPU Architecture Overview](../architecture/epu-overview.md) - Compute pipeline details
- [EPU Feature Catalog](../../../../../nethercore-design/specs/epu-feature-catalog.md) - Opcode catalog + packing details
- [ZX FFI Bindings](../../../../../nethercore/include/zx.rs) - Canonical ABI docs
