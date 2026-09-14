# Environment Processing Unit (EPU)

The Environment Processing Unit (EPU) is ZX's environment system. You can drive it from a packed 128-byte procedural configuration (8 x 128-bit instructions) or from six imported cube-face textures, then use the immediate-mode EPU API to:

- Render the environment background
- Drive ambient + reflection lighting for lit materials (computed on the GPU)

For canonical ABI docs, see `include/zx/mod.rs`. For the opcode catalog/spec, see `nethercore-zx/shaders/epu/`.

Known limitation: TRACE's POLAR chart is not continuous at its opposite pole for
some variants. See the [bounded TRACE warning](../guides/epu-environments.md#known-trace-polar-limitation)
before relying on full-sphere line continuity.

---

## FFI

### epu_set

Select the current EPU source from a procedural config (no background draw).

The three region-mask bits are consumed only by features (`0x08..0x18`). Bounds
(`0x01..0x07`) ignore those bits, replace region weights and blend their paint in
sequence. Their inspector region checkboxes are disabled without rewriting
storage. A later opaque bounds layer can cover an earlier floor or decal, so
place required features after it. A downward PLANE with `REGION_ALL` projects
below the viewer without depending on the current bounds' floor weights.

To switch environments in the same frame, call `epu_set(...)`, `epu_textures(...)`, or `epu_asset(...)` before the draws that should use that source.

Animation remains guest-owned: advance rollback-covered state in `update()` and emit its current configuration in `render()`. For `VEIL/RAIN_WALL`, `param_d` is cyclic phase (`raw/256`); each streak travels one or two vertical-chart circuits per cycle, including seamless byte wrap. Holding the phase holds the rain. This changes historical drop motion, not the packed API. See [variant-specific motion and compatibility](../guides/epu-environments.md#variant-specific-motion-reality) before animating other opcodes.

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

### CELL paint and lattice contract

For `CELL` (`0x05`), `alpha_a` is **Gap Alpha**: `0` makes the gap-color contribution transparent and `15` makes it opaque. It scales both gaps between solids and wholly unfilled cells, once rather than twice. It does not change the geometry-owned SKY/WALL/FLOOR regions. `alpha_b` controls outline opacity. Zero Gap Width disables the additional outline contribution; the geometry-owned wall band remains. Packed density decoding keeps true fractional GRID steps while representing integral steps exactly, without a phantom terminal column.

`HEX` (variant `1`) uses a regular periodic hex lattice. Density maps to 4..64 and is rounded to a whole column count; the same count scales both axes. Fill selects occupied polygons, including their shared antialiased edges. This deliberately changes older HEX patterns. CELL's bit-stable integer hashing also changes older seeded layouts while preserving fractional hash inputs. Neither correction changes the 128-bit encoding or introduces host-owned animation.

`VORONOI` and `SHATTER` also round density to a whole periodic circumference rather than inserting a partial terminal cell. GRID retains its existing fractional circumference and partial terminal rectangle. Filled rectangles retain antialiased support into neighbouring openings, including zero/narrow Gap Width. The jittered variants use shortest periodic site distance without duplicate wrapped sites. Their filled-site edge bands continue into adjacent unfilled cells: changing the nearest owner no longer discards a neighbouring outline or region contribution. Site positions, seeded occupancy, and filled interiors retain the same geometry; Gap Alpha remains independent of region ownership. `BRICK` is a regular staggered grid with half as many rows as columns; seed changes occupancy/paint, not brick geometry. Filled bricks contribute their antialiased edge fields to neighbouring openings, as HEX does. Real mortar boundaries remain structural edges.

`RADIAL` (variant `3`) uses regular annular sectors in the plane perpendicular to the authored axis, with one centre cell at each pole. Rounded density selects the spoke count; the number of radial bands, including the centre, is half that count rounded up. Seed changes occupancy, not geometry. Filled polar cells retain their antialiased edge support into neighbouring openings, so occupancy and SKY/WALL/FLOOR regions follow the same boundary. Rings, spokes and the projected outer rim remain structural edges. This is the regular construction; use the separate `WARPED_RADIAL` variant for warped cells rather than changing what regular RADIAL means.

`WARPED_RADIAL` (variant `6`) restores flowing, warped ring/spoke cells as a separate construction. It keeps CELL's Density, Fill, Gap Width, Seed, outline and alpha meanings. Density controls cell scale; the underlying distortion frequency is bounded rather than growing quadratically with it. Seed changes occupancy, not geometry. Periodic warping fades near the poles, and occupancy and filled-neighbour edge support use the same warped coordinates. It is a geometric primitive, not an environment preset or a time controller.

Variant `6` is now defined, not a reserved GRID fallback; explicitly select `GRID` (`0`) if that was intended. Regular `RADIAL` remains `3`, the remaining undefined selector `7` retains the existing GRID fallback, and the `0x05` opcode and 128-bit layout are unchanged.

Density remains a guest-owned packed byte. Rounding can map adjacent byte values to the same lattice; changing the count reconstructs it and can change cell identity. These corrections do not promise smooth density animation or universal polar continuity.

### GRID repeats and cyclic scrolling

`GRID` (`0x09`) is the line/checker feature, not `CELL/GRID` (`0x05`, variant `0`).

- `param_a` targets `1 + 63 * byte / 255` repeats. Stripes/grid round to the nearest whole count **1..64**; checker rounds to the nearest even count **2..64**, with half-up ties. Adjacent bytes can select the same count. Changing count reconstructs the pattern; it is not smooth density animation.
- `param_c` high nibble selects **0=stripes, 1=grid, 2=checker**; other values retain the stripes fallback. The low nibble is **0..15 whole pattern cycles per phase loop**, not the former fractional scroll speed. Zero is stationary. A checker cycle spans two cells so its alternating colors close.
- `param_d / 256` is the guest-owned cyclic phase. Byte 255 is the last step before zero, not a duplicate endpoint. Hold it or advance with `wrapping_add`; there is no engine clock. The [exact phase helper](../guides/epu-environments.md#grid-count-and-phase-authoring) changes only that byte.
- `param_b` remains line half-thickness in cell coordinates, `0.001..0.1`; checker ignores it. Stripes/grid retain hard authored edges. Checker alternates Color A at 60% and 100% linear intensity; Color B, its alpha, and direction remain unused.

This deliberately changes old fractional layouts and scrolling rates while preserving the opcode, packed fields and API signatures. It repairs the cylindrical chart cut and phase wrap, not the projection itself: azimuth is undefined at the world-Y poles. Use a wall/bounded region away from those poles when a unique polar limit is required. Do not confuse this feature's whole counts with CELL/GRID's fractional terminal-cell contract.

### LOBE centre-bright gain

`LOBE` (`0x12`) is a directional cosine-power glow. Its **weight** peaks on the authored axis without the old hard-coded centre suppression; arbitrary core/edge colors still control the resulting appearance. Use `BAND` for an explicit ring. Existing LOBE centres become brighter; the opcode, field packing, colors and waveform functions are unchanged.

- Brightness: `2 * intensity / 255`; Color A alpha and the current region mask multiply coverage. Normal blend clamping still applies: this is not a separate HDR exposure control.
- Exponent (`param_a`): `1 + 63 * raw / 255`; increasing it narrows the directional lobe.
- Color falloff (`param_b`): `0.5 + 3.5 * raw / 255`; shapes the A-to-B color mix, not the coverage exponent.
- Waveform (`param_c`): **0 steady** (modulation off, not light off), **1 sine**, **2 triangle**, **3 four-pulse strobe**. Reserved byte values use the existing sine fallback and are preserved until explicitly edited.
- Phase (`param_d`): **raw / 256 cycles**, with 255 immediately preceding wrap to zero. Steady mode ignores phase. The guest owns held state, advancement and restoration; the engine has no clock.
- Color B alpha, domain and variant are unused. Stored values are retained; the native inspector disables B alpha and inactive phase rather than silently rewriting them.

## Carrier loop closure

ADVECT and MASS separate their cyclic time from deterministic shape offsets. Every authored variant in DIRECT3D, AXIS_CYL and AXIS_POLAR closes over the 256-step phase cycle. Phase zero retains the prior shape; nonzero phases deliberately replace the old nonperiodic motion. No engine clock or new packed field is involved. MOTTLE already has cyclic motion and a single world-space coordinate path: its stored domain bits are ignored, not alternative projections.

## ADVECT/MASS transport charts

ADVECT and MASS share DIRECT3D, AXIS_CYL and AXIS_POLAR domains. The cylindrical and polar charts now close their azimuth smoothly near either pole, within transverse radius 0.05. This is a coordinate continuation, not an opacity fade: the normal body and density evaluation still runs. Coordinates outside that neighborhood and DIRECT3D are unchanged. Older pole-adjacent detail can change; packed fields, signatures and guest-owned phase do not.

## Cyclic phase units

DECAL, VEIL, MOTTLE, ADVECT, SURFACE and MASS decode their guest-owned loop phase as **raw / 256 turns**. Their inspector fields span 0..0.99609375 turns: byte 255 is the last step before wrap, not a second copy of zero. All 256 stored byte values remain available. This corrects the displayed units only; motion, packed fields and shader execution are unchanged. A phase field does not guarantee motion in every variant. CELESTIAL's phase-angle convention is separately documented; do not treat every field named phase as interchangeable.

## PATCHES chart pole coordinates

`AXIS_CYL` and `AXIS_POLAR` close their undefined azimuth using the existing
ADVECT/MASS coordinate closure inside transverse radius `0.05`. The original
circle embedding remains outside that cap. This changes noise coordinates,
not the noise family, seed, thresholds or coverage formula. No opacity/support
fade is added: region patterns, colors and their region-weighted alpha may
change inside the cap. DIRECT3D and the reserved-domain fallback are unchanged.
STATIC retains its intentional hard lattice boundaries; max sharpness also
retains the authored hard region threshold. Neither is mislabeled smooth noise.

## VEIL variant-aware authoring

VEIL keeps its existing ribbon algorithms and stored fields. The native editor shows an **integer base count**: `2 + floor(param_a * 30 / 255)`, from 2 to 32. RAIN_WALL emits twice that count. Editing Count chooses the lowest byte for the selected count; loading or leaving the control idle preserves every original byte.

**Thickness ratio** is `param_b / 255`. Its actual chart-space base width is `mix(0.002, max(0.05, 0.5 / count), ratio)`, also displayed by the native editor. It is not an angle or world-space distance; variant core/glow shapes further scale it.

- CURTAINS (0, also fallback selectors 5..7): `param_c` is static sway; phase is unused.
- PILLARS (1): shape, phase, Color B and Color B alpha are unused; all remain stored and their native controls are disabled.
- LASER_BARS (2): shape and phase are unused; Color B and its alpha still control glow.
- RAIN_WALL (3): `param_c` controls wind slant; only this variant reads guest phase `param_d / 256`.
- SHARDS (4): `param_c` controls static tilt; phase is unused.

The guest inspector labels unused bytes rather than discarding them. No renderer, motion, opcode, packed layout or clock changes with these authoring corrections.

AXIS_POLAR uses the existing support fade at **both poles**: zero contribution within normalized angular distance 0.05, reaching full contribution at 0.2. The middle chart (radius 0.2..0.8) and other domains are unchanged. Mirroring the former one-ended support removes the opposite-pole singularity; older polar scenes lose that endpoint contribution. This is the explicitly authorized visibility change, not a new fade width, hidden controller or packed-field change.

## PLANE variant-aware authoring

PLANE uses its authored blend mode. `intensity` is layer gain (it multiplies Color A alpha), not a separate texture-contrast control. `param_a` scales every pattern. The inspector preserves unused stored bytes while disabling their controls.

Color A paints the surface; Color B paints the gaps. Increasing `param_b` exposes more gap, not more surface. HEX uses regular hexagonal cells with equal neighbour spacing. GRATING maps the nominal gap range 0..0.2 (raw 0..255) from full bars to all gaps. This corrects the reversed HEX/GRATING coverage and irregular HEX layout; older images and HEX cell-color variation change, but packed fields and API signatures do not.

- **TILES / HEX / GRATING:** Color A, Color B, scale and `param_b` gap width; no variation or phase.
- **STONE / PAVEMENT:** also use `param_c` surface/color variation.
- **SAND / GRASS:** Color A, scale and variation only; Color B and gap width are inactive.
- **WATER:** Color A, scale and guest-owned `param_d` ripple phase only. Phase is **raw / 256 turns**: 255 is 0.99609375 turns, then wraps to zero without a duplicated endpoint. No other PLANE variant reads phase.
- **Direction points toward the visible plane:** use down for a floor and up for a ceiling. It is not an outward-facing mesh normal; rays facing away from it contribute nothing.
- **All variants ignore Color B alpha.** The stored nibble survives editing and export. Pattern variation is not object-material roughness.

## ATMOSPHERE: directional tint, not geometry fog

ATMOSPHERE operates on environment directions only. It has no scene depth, clock, weather controller or intrinsic cyclic phase. Strength and Color A alpha scale the result once; the shared blend clamps its weight and RGB. Color B alpha and domain are unused and retained in storage.

- **ABSORPTION (0):** gradient tint weighted by `(1-t) * strength`; use Multiply to darken an existing environment.
- **RAYLEIGH (1):** altitude gradient between Color B (horizon) and Color A (zenith).
- **MIE (2):** Color A directional halo; concentration, exponent and the layer direction are active. Falloff, horizon shift and Color B are inactive.
- **FULL (3):** gradient plus half-strength Mie tint, with both control groups active.
- **ALIEN (4):** sinusoidal altitude color gradient; the name denotes a mathematical shape, not an engine-owned mood.
- **5–7:** reserved, no output.

Gradient variants use the current bounds up vector, with `t = pow(clamp((dot(dir,up)-horizon_y+1)/2,0,1),falloff)`. Horizon Y is a signed shift of that full-sphere ramp, not a hard horizon cutoff. Falloff spans 0.5–8. Mie concentration spans 0–2 and exponent 4–128; its halo uses the forward half-space only. Direction, concentration and exponent are unused outside MIE/FULL. Disabled native controls preserve their stored bytes, including reserved values; changing variants does not normalize them. Colors are authored RGB tints, not a spectral scattering simulation.

## PORTAL paint and cyclic VORTEX

PORTAL uses a front-facing gnomonic tangent chart. Size is a tangent-space scale (0.05–0.8), not an angle in radians; its base angular half extent is `atan(size)`. Individual shapes apply their documented proportions. Glow width is also measured in that chart (0.01–0.3).

Color A and alpha A control the interior; Color B and alpha B control the edge glow. Intensity scales the glow, not the interior. Straight component paint is combined with coverage once by the shared blend. Setting an individual alpha to zero removes that paint component, including at antialiased edges.

VORTEX reuses the existing rough TEAR contour under the existing spiral coordinate warp. Roughness controls contour variation; zero roughness gives a circle with no meaningful phase motion. TEAR, VORTEX, CRACK and RIFT use roughness; CIRCLE and RECT ignore it. VORTEX alone uses phase: `param_d / 256` turns, with wrap from 255 to 0. The guest advances or holds this byte; the EPU owns no clock or effect controller. Unused fields remain stored, not reset by the inspector.

This corrects formerly double-applied paint coverage and an ineffective rotation of a circular SDF. Existing PORTAL images can change; opcodes, packed fields and public signatures do not.

## DECAL front-facing support

`DECAL` paints only the front-facing hemisphere of its authored centre direction. RECT and LINE no longer repeat behind that direction. Front-facing projection, size, softness, fill/glow paint and pulse remain unchanged. LINE remains an unbounded vertical stripe within that front chart, not a finite segment; this repair adds neither a length control nor a grazing fade. The hemisphere boundary is explicit support, not a promise of a smooth wrap through the back of the chart. Packed fields and API signatures are unchanged.

## DECAL paint coverage

DECAL returns straight fill/glow paint plus a separate coverage weight, like the shared layer blend contract. Fill alpha, glow alpha and antialiased shape support are applied once, not squared by being present in both returned RGB and blend weight. Zero coverage produces finite zero paint. The correction makes partially covered/transparent paint brighter than older renders, while preserving every geometric coverage weight, shape, Size, direction, phase, brightness multiplier and packed field. It does not change the separately tracked RECT/LINE antipodal-support behavior.

### CELESTIAL MOON/PLANET phase lighting

MOON and PLANET use a unit-sphere normal reconstructed from the projected body disk. The same disk point has the same phase illumination at every Size; the night side has zero direct surface illumination, rather than half-Lambert fill. Limb darkening and surface detail remain separate factors. Color B atmosphere/rim light is separately authored and can remain visible on a dark body; set its alpha to zero to inspect surface lighting alone.

Phase (`param_c`) retains `byte / 255 * 360` degrees: 0/360 is full, 90/270 is half, and 180 is new. These are angle landmarks; the byte lattice need not land exactly on each angle. Byte 255 duplicates the full-phase endpoint, unlike GRID/BAND's `/256` cyclic phase. For uniform guest-driven rotation, iterate bytes 0..254 and wrap to 0; holding the byte holds lighting. This repair changes older MOON/PLANET shading, not phase angles, angular size, textures, other variants, packed fields or API signatures.

### CELESTIAL ring tilt

RINGED (variant 4) uses `param_d / 255 * 90` degrees: 0 is edge-on with zero ring coverage; 90 is face-on with a complete annulus. Intermediate values flatten the annulus by `sin(tilt)`. Its angular radii remain 1.5–2.5 times the body's authored radius, with the existing soft edges and disk cutoff: rings do not cover the disk at `r <= 1.05`. Bands and the internal gap follow the same elliptical radius. This is a directional angular construction, not scene geometry.

Color B and its alpha author the rings. Phase (`param_c`) is unused for RINGED and retained in storage. Tilt is inclination, not cyclic position: 255 and 0 are different endpoints. The guest owns changes and any repeated sweep. Older RINGED renders change because the former plane test cancelled Tilt out of the coverage calculation; opcode, packed fields and API signatures are unchanged.

### CELESTIAL surface texture footprint

MOON, PLANET and GAS_GIANT detail uses the fixed projected body radius, `sin(angular_size_rad)`, rather than each pixel's normalized angular distance. Texture coordinates span the projected unit disk from centre to limb, so Size no longer collapses their radial footprint. This correction leaves noise functions, phase lighting and tilt unchanged.

### CELESTIAL angular distance

CELESTIAL measures the full 0..180-degree separation from an authored body direction,
using a signed cosine clamped to -1..1. Directions beyond 90 degrees must not flatten
to 90 degrees: that previously produced incorrect back-hemisphere corona and ring
contributions. Tangent projection retains the same signed dot product. This corrects
four shared evaluator sites; angular-size units, gain, UV mapping, tilt conventions,
phase bytes, opcode, packed layout and API signatures are not changed by this repair.

### CELESTIAL contribution strength

`CELESTIAL` (`0x10`) decodes intensity as `2 * raw / 255` and applies that gain **once in the layer weight**, alongside Color A alpha and the selected region weight. Like other radiance layers, the common blend clamps weight to `0..1`; this is not a separate HDR exposure control. On black with ADD, below clipping, doubling intensity doubles the contribution. Zero intensity disables the feature under every blend.

Previously, the evaluator multiplied both RGB and weight by intensity, unintentionally squaring low-strength contributions and adding a second RGB gain at high strength. The shared-return correction affects all seven body variants; older renders can change. Body geometry, UV mapping, colors, phase controls, field layout and API signatures are unchanged by this correction. Their broader continuity/appearance gates remain separate.

### BAND depth and cyclic phase

`BAND` (`0x13`) is a ring around its encoded axis, with center/edge RGB and optional four-wave azimuthal modulation.

- **Color B alpha (`alpha_b`) is modulation depth, 0..15**, not paint opacity. Zero gives a plain ring at every phase. Depth 15 retains the original four-wave **40..100%** modulation profile; intermediate depths blend linearly toward the plain ring. Color A alpha remains layer opacity.
- `param_d / 256` is cyclic phase, including zero. Advance it in deterministic guest `update()`; keep depth fixed to move the accents without switching them off at wrap. One phase cycle rotates the pattern once; its four identical waves also repeat every 64 byte steps. This is accent motion, not translation of the ring.
- Width (`param_a`, 0.005..0.5 in axis-projection coordinates), offset (`param_b`, -0.5..0.5), edge softness (`param_c`, 0..1), brightness and axis keep their existing meaning. The ring's existing cap fade and support are retained; domain and variant remain unused.

**Migration:** old BAND used phase zero as a modulation-off sentinel and ignored Color B alpha. Set depth 0 for a plain ring or 15 for the old nonzero-phase profile. Animated programs now need explicit nonzero depth; phase zero no longer turns them off. Older renders may change, but opcode, packed fields and existing API signatures do not. The host `band_radiance(p)` builder writes zero depth; the additive `band_radiance_with_depth(p, depth)` exposes depth clamped to 0..15 without adding a field to `BandRadianceParams`. WASM guests use the [small exported-layer helpers](../guides/epu-environments.md#band-depth-and-phase-authoring), not the host builder API.

### Determinism (No Host Time)

The EPU has **no host-managed time input**. Any temporal variation (scrolling, pulsing, drifting, twinkling, etc.) must be driven explicitly by the game by changing instruction parameters as part of deterministic simulation.

Advance parameters in simulation `update()`, store them in rollback-covered game state, and submit that state with `epu_set(...)` in `render()`; rendering must not advance time or RNG. Many animated variants use `param_d`, but some use it as seed or waveform selection rather than smooth motion. A wrapping byte alone does not guarantee a visually seamless loop. An exported configuration is a snapshot, not an exported animation driver.

---

## Inactive common controls

Color B alpha is not a universal opacity control. **SECTOR, SILHOUETTE, SPLIT,
APERTURE, GRID, legacy SCATTER, FLOW, MOTTLE, ADVECT, SURFACE and MASS ignore it**,
as do RAMP, ATMOSPHERE, PLANE and LOBE. Their native Alpha B control is disabled, with its stored nibble preserved.
SCATTER_PHASED and BAND instead use it for modulation depth; their controls remain
active. Other opcodes retain their documented paint/region-specific meanings.

GRID additionally ignores Color B RGB and direction: it uses Color A and a fixed
Y-up cylindrical chart. The native Color B picker and direction editor are
disabled rather than changing stored values. This does not alter rendering,
packing, or exported cartridge fields.

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
| `0x17` | `MASS` | Feature: broad directional bodies |
| `0x18` | `SCATTER_PHASED` | Feature: guest-phased point brightness |

For full per-opcode packing/algorithm details, see `nethercore-zx/shaders/epu/` in this checkout.

`SCATTER_PHASED` (`0x18`) shares SCATTER geometry but uses `param_c / 256` for guest-owned cyclic phase and `alpha_b / 15` for modulation depth (0 steady, 15 fully off/on). All seven shapes and four domains retain their existing roles; variant 7 remains unsupported/no-output. `alpha_a` remains layer opacity. The new opcode requires a supporting player; older players treat it as NOP. Legacy `SCATTER` (`0x0A`) and the 128-byte layout remain unchanged. See [independent point brightness](../guides/epu-environments.md#independent-point-brightness) for executable guest-side phase editing.

---

## Host-side typed construction

Native Rust tools using `nethercore_zx::graphics::epu` can construct MASS layers with `EpuBuilder::mass(MassParams)`. `MassVariant` exposes `Bank`, `Shelf`, `Plume`, and `Veil`; the helper packs the existing `0x17` instruction rather than introducing a new shader or format.

`MassParams` exposes both colors, region, blend, direction, density (`intensity`), scale, coverage, breakup, phase and alpha. Parameter bytes retain their packed ranges; alpha is `0..15`, and supported MASS domain IDs are `0` (DIRECT3D), `1` (AXIS_CYL), and `2` (AXIS_POLAR). These are the supported input values, not a promise that the builder validates arbitrary values. Phase changes remain caller-controlled.

This helper belongs to the **native host library**, not the WASM guest ABI. Cartridges still submit the packed configuration through `epu_set`; adding this helper does not add a guest-side `mass` function or export an animation controller.

### SILHOUETTE: static wall depth, not drift

For native Rust construction, `EpuBuilder::silhouette_bounds(SilhouetteParams)` retains its historical public fields and exact packing:

| Native field | Packed field | Current meaning |
|---|---|---|
| `roughness` | `param_b` | Height span / relief: `0..255` maps to `0.1..1.0` |
| `octaves_q` | `param_c[7:4]` | MOUNTAINS octave count: `1 + (octaves_q & 15) / 2` (integer, `1..8`); inactive for other variants |
| `drift_amount_q` | `param_c[3:0]` | Reserved/inactive, not drift amount; low four bits are preserved |
| `drift_speed` | `param_d` | Wall depth below the roofline: `0.05 + 1.15 * raw / 255`, not speed |

Use `drift_speed: 160` to author a wall-depth byte of 160 in an otherwise default `SilhouetteParams`; see its compiled rustdoc example. When editing an existing value, assign only `params.drift_speed = depth_byte`, then pass it to `silhouette_bounds`. Do not clear `drift_amount_q`: choose zero for new layers, but preserve existing reserved bits. Existing nibble masking and defaults are unchanged.

Depth is in signed up-axis height space, not world distance or parallax; the wall base is clamped to `-1..1`. Neither historical drift field supplies time or phase. The native inspector labels `param_d` as `wall_depth`; its `param_c` control is explicitly advanced packed/raw, not a normalized octave slider. Editing that whole-byte control deliberately changes the stored byte; idle, load and export do not normalize it. Guest SDKs still submit these same packed bytes through `epu_set`; no new guest function or animation driver is introduced.

## RAMP paint and rendering compatibility

RAMP combines authored sky, wall and floor RGB using their structural ownership weights: `sky*w_sky + wall*w_wall + floor*w_floor`. Full wall ownership therefore returns the authored wall color. `alpha_a` controls paint opacity independently of structural ownership; thresholds, softness and direction still define the regions.

**Rendering compatibility note:** the hidden wall-color softening has been removed. Existing RAMP scenes can change in wall bands and transitions, including derived diffuse and reflection lighting. Packed fields, opcode IDs and paint-opacity semantics are unchanged, but historical pixels are not preserved. Lowering wall RGB can approximate the old full-wall color; it does not reproduce the old transition-dependent mixing everywhere. Review existing scenes rather than assuming a universal migration.

### SPLIT blend width

SPLIT `param_a` (Rust `SplitParams::blend_width`) remains a raw byte. Its nominal blend width is `param_a / 255 * 0.2`, displayed by the native editor as `0.0..0.2`. The evaluator floors this nominal width at `0.001` for AA stability before variant-specific use; nominal zero is not an effective zero-width edge. This metadata correction changes neither packed bytes nor shader behavior.

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
| 0 | `0b000` | `NONE` | Zero feature coverage; not a bounds-layer disable switch |

The region mask is consumed by feature opcodes: their contribution is multiplied by `region_weight(current_regions, mask)`. This is the sum of the selected ownership weights, clamped to `0..1`; adding a region with zero weight does not change the result.

**Rendering compatibility note:** the correctness repair removes the former `weight^0.72` boost for single-region masks. Partially covered single-region features therefore become weaker; zero/full coverage and multi-region weighting are unchanged. Packed instructions and API identifiers are unchanged, but existing scenes that relied on the boost need visual review. Adjust authored bounds or feature strength deliberately rather than assuming pixel compatibility.

`current_regions` comes from the most recent bounds opcode; every bounds opcode outputs updated `RegionWeights` for subsequent layers. Bounds opcodes do not use the region mask.

APERTURE/MULTI (variant 5) uses a regular N-by-N rectangular opening grid, with Param D clamped to 1..8. N=1 matches RECT. Half Width/Height set the envelope; Frame Thickness controls the shared outer rectangle and internal mullions stay WALL-owned. Older seeded/warped MULTI appearances intentionally change, not the packed API. See the [environment guide](../guides/epu-environments.md) for the geometry contract.

---

## Blend Modes (3-bit, 8 modes)

Here `src` and `a` are the opcode's output RGB and weight, not necessarily its raw color/alpha fields. The runtime clamps `a` to `0..1` and clamps the result of every blend to `0..1` per RGB component. Use `NOP` to disable a layer; a region mask is not a general opcode-disable switch.

| Value | Name | Formula |
|-------|------|---------|
| 0 | `ADD` | `dst + src * a` |
| 1 | `MULTIPLY` | `dst * mix(1, src, a)` |
| 2 | `MAX` | `max(dst, src * a)` |
| 3 | `LERP` | `mix(dst, src, a)` |
| 4 | `SCREEN` | `1 - (1-dst)*(1-src*a)` |
| 5 | `HSV_MOD` (legacy identifier) | RGB Offset: `clamp(dst + (src - 0.5) * clamp(a, 0, 1) * 2, 0, 1)`, per RGB component; not HSV modulation. Source 0.5 is neutral. |
| 6 | `MIN` | `min(dst, mix(1, src, a))` |
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
- `examples/3-inspectors/epu-showcase/src/presets.rs`
- `examples/3-inspectors/epu-showcase/src/constants.rs`

These snippets show the render body; import the canonical SDK (`include/zx/mod.rs`, `include/zx.h`, or `include/zx.zig`) and export the guest lifecycle functions in your cartridge. The all-NOP template is intentionally black; replace it with an authored/exported configuration.

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
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
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
- Opcode implementation and parameter decoding: `nethercore-zx/shaders/epu/` in the source checkout.
- Guest FFI declarations: `include/zx/mod.rs` in the source checkout.
