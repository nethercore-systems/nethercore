# Environment Processing Unit (EPU)

The Environment Processing Unit (EPU) is ZX's environment system. You can drive it from a packed 128-byte procedural configuration (8 x 128-bit instructions) or from six imported cube-face textures, then use the immediate-mode EPU API to:

- Render the environment background
- Drive ambient + reflection lighting for lit materials (computed on the GPU)

For canonical ABI docs, see `nethercore/include/zx.rs`. For the opcode catalog/spec, see `nethercore-design/specs/epu-feature-catalog.md`.

---

## FFI

### epu_set

Select the current EPU source from a procedural config (no background draw).

To switch environments in the same frame, call `epu_set(...)`, `epu_textures(...)`, or `epu_asset(...)` before the draws that should use that source.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Select the current EPU source from a procedural config.
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions x (hi u64, lo u64)
fn epu_set(config_ptr: *const u64);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Select the current EPU source from a procedural config.
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions x (hi u64, lo u64)
void epu_set(const uint64_t* config_ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Select the current EPU source from a procedural config.
///
/// config_ptr points to 16 u64 values (128 bytes):
/// 8 instructions x (hi u64, lo u64)
pub extern fn epu_set(config_ptr: [*]const u64) void;
```
{{#endtab}}

{{#endtabs}}

### epu_textures

Select the current EPU source from six already-loaded 2D textures interpreted as cubemap faces.

Face order is fixed: `px, nx, py, ny, pz, nz`.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Select the current EPU source from six texture handles.
fn epu_textures(px: u32, nx: u32, py: u32, ny: u32, pz: u32, nz: u32);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Select the current EPU source from six texture handles.
void epu_textures(uint32_t px, uint32_t nx, uint32_t py, uint32_t ny,
                  uint32_t pz, uint32_t nz);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Select the current EPU source from six texture handles.
pub extern fn epu_textures(px: u32, nx: u32, py: u32, ny: u32, pz: u32, nz: u32) void;
```
{{#endtab}}

{{#endtabs}}

### epu_asset

Select the current EPU source from a packed six-face environment asset in the ROM data pack.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust,ignore
/// Select the current EPU source from a packed EPU environment asset.
fn epu_asset(id_ptr: *const u8, id_len: u32);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Select the current EPU source from a packed EPU environment asset.
void epu_asset(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Select the current EPU source from a packed EPU environment asset.
pub extern fn epu_asset(id_ptr: [*]const u8, id_len: u32) void;
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
- `epu_set(...)`, `epu_textures(...)`, and `epu_asset(...)` all select the current EPU source.
- `draw_epu()` draws the currently selected source.

---

## Configuration Layout

Each environment is exactly **8 x 128-bit instructions** (128 bytes total). In memory, that is 16 `u64` values laid out as 8 `[hi, lo]` pairs.

| Slots | Kind | Authoring Model |
|------|------|-----------------|
| `0-7` | Mixed | The runtime evaluates all 8 instructions sequentially in authored order. Bounds opcodes (`0x01..0x07`) establish or reshape region weights; feature opcodes (`0x08+`) consume the current regions. A common cadence is `BOUNDS -> FEATURES -> BOUNDS -> FEATURES`, but it is not required. |

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

### Determinism (No Host Time)

The EPU has **no host-managed time input**. Any temporal variation (scrolling, pulsing, drifting, twinkling, etc.) must be driven explicitly by the game by changing instruction parameters as part of deterministic simulation.

In practice this often means incrementing an opcode-specific motion or modulation parameter each frame and re-calling `epu_set(...)` with the updated config. Many animated variants use `param_d` for this, but not all do; some variants use `param_d` as seed or waveform selection rather than smooth motion.

---

## Opcode Map (current shaders)

This is the opcode number. Some opcodes use `meta5` for domain/variant selection; when unused, set `meta5 = 0`.

| Code | Name | Notes |
|---|---|---|
| `0x00` | `NOP` | Disable layer |
| `0x01` | `RAMP` | Bounds gradient |
| `0x02` | `SECTOR` | Bounds modifier |
| `0x03` | `SILHOUETTE` | Bounds modifier |
| `0x04` | `SPLIT` | Bounds |
| `0x05` | `CELL` | Bounds |
| `0x06` | `PATCHES` | Bounds |
| `0x07` | `APERTURE` | Bounds |
| `0x08` | `DECAL` | Feature |
| `0x09` | `GRID` | Feature |
| `0x0A` | `SCATTER` | Feature |
| `0x0B` | `FLOW` | Feature |
| `0x0C` | `TRACE` | Feature |
| `0x0D` | `VEIL` | Feature |
| `0x0E` | `ATMOSPHERE` | Feature |
| `0x0F` | `PLANE` | Feature |
| `0x10` | `CELESTIAL` | Feature |
| `0x11` | `PORTAL` | Feature |
| `0x12` | `LOBE` | Feature |
| `0x13` | `BAND` | Feature |
| `0x14` | `MOTTLE` | Feature |
| `0x15` | `ADVECT` | Feature |
| `0x16` | `SURFACE` | Feature |

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

The region mask is consumed by feature opcodes: their contribution is multiplied by `region_weight(current_regions, mask)`.

`current_regions` comes from the most recent bounds opcode; every bounds opcode outputs updated `RegionWeights` for subsequent layers. Bounds opcodes do not use the region mask.

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
