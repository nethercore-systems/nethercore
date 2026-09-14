# EPU Environments

The Environment Processing Unit (EPU) is ZX's GPU-driven procedural background and ambient environment system.

- It renders an infinite environment when you call `draw_epu()` after providing a config with `epu_set(config_ptr)` (packed 128-byte config).
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact FFI signatures and instruction encoding, see the [Environment (EPU) API](../api/epu.md).

For the full specification (opcode catalog, packing rules, WGSL details), see:
- `nethercore-zx/shaders/epu/`
- `include/zx/mod.rs` (canonical ABI docs)


Authoring note: the current runtime and Rust builder evaluate all 8 instructions sequentially in authored order. The real model is mixed `bounds` and `features`, with bounds rewriting region weights for later feature layers.

### Known TRACE polar limitation

`TRACE/POLAR` (`domain_id=2`) can still show an azimuth-dependent discontinuity
at the pole opposite its authored axis, observed for RIBBON, BOLT and FILAMENT.
Do not rely on that chart for a continuous line through the opposite pole.
Choose another domain deliberately if it fits the composition; domains are not
drop-in coordinate equivalents. The existing first-pole fade has not been silently
mirrored to hide this defect. This is a known rendering limitation, not an ABI change.

### Rendering compatibility notes

SPLIT/BANDS compatibility: Both integer and reverse half-integer transitions are now wall-mediated, retaining alternating sky/floor ownership and minimum-width sharpness (the nominal width still has a `0.001` floor). The integer-center inner-quarter profile is retained; broad overlapping supports increase wall weight and alter the outer sky/floor shoulders. No API or packed-byte change is involved. This is not global appearance parity; review broad blends and quarter-band shoulders in authored scenes.

SPLIT/PRISM's cap repair removes angular sector holes from the ceiling/floor caps and completes wide cap transitions at the reachable sphere endpoint. Cap appearance therefore changes, including broad blends; side-sector modulation remains, and sampled off-cap output is unchanged. Packed controls and guest ownership are unchanged. Pole continuity has bounded GPU and matched exported-guest evidence, not whole-variant or human appearance acceptance. The separate fractional-wrap correction is described below.

SPLIT/PRISM continues fractional sector distances across the angular wrap instead of rounding side counts. It retains the original region evaluation for integral counts and wherever continuation cannot contribute; only the affected wall blend changes near the wrap. Ordinary sector boundaries and axial cap controls are preserved in the checked cases. The shared bounds evaluation fixes both background radiance and later region-masked features, rather than hiding the jump with a background-only blend. Packed controls and guest-owned parameter updates are unchanged; human appearance acceptance remains separate.

CELL/HEX now uses a regular periodic hexagonal lattice, not rectangular row ownership. Density maps to 4..64 and is rounded to whole cell columns; the same count scales both axes. Filled polygons contribute their shared antialiased edge fields across angular and row joins. **Gap Alpha controls every opening, including missing cells, and is applied once**; it does not reclassify SKY/WALL/FLOOR regions. Zero Gap Width disables extra outline brightness rather than evaluating a zero-width transition; the ordinary wall band remains. CELL uses integer hashing of canonical float input bits so direct and complete-program evaluation agree, retaining fractional seed inputs. HEX patterns and seeded CELL layouts intentionally differ from older versions; the packed instruction layout is unchanged. The chart-wrap and filled-neighbour corrections below also apply. Pole appearance, human visual review, and clean performance acceptance remain separate open gates.

- CELL/VORONOI and SHATTER now round the packed density scale `4 + 60 * byte / 255` to a whole periodic circumference. They select jittered sites using shortest periodic distance and do not count wrapped duplicates as different sites. Filled-site edge support continues across empty ownership, including zero/narrow Gap Width, instead of disappearing at the nearest-site boundary. The support repair preserves site identities, seeded occupancy, filled interiors and unrelated variants; it adds no clock or packed fields.
- CELL/GRID retains the fractional circumference `4 + 60 * byte / 255` and its partial terminal rectangle. The terminal edge remains a real boundary between different cell owners; GRID has not been migrated to rounded density. Filled rectangles retain antialiased support across owner boundaries, including zero and narrow Gap Width. Packed decoding agrees for constant and dynamic inputs, so integral density steps do not create an extra column.
- CELL/BRICK uses the same whole circumference and a regular staggered rectangular grid with half as many rows as columns. Seed changes fill/paint, not brick geometry. As with HEX, filled bricks contribute their antialiased edge field into adjacent unfilled cells; selecting only the current owner must not abruptly discard a neighbouring edge. Real mortar boundaries and T-junctions remain structural edges, not unwanted seams.
- CELL/RADIAL uses regular annular sectors around the authored axis. Density rounds to whole spokes; radial bands, including one centre cell at each pole, use half that count rounded up. Seed changes occupancy rather than warping rings or spokes. Filled-neighbour support continues across openings, and the circular rim remains a real edge. The pole has a single centre-cell limit instead of azimuth-dependent slivers. Older RADIAL patterns intentionally change; no clock, opcode or packed fields are added.
- CELL/WARPED_RADIAL (variant `6`) restores warped, flowing ring/spoke cells alongside regular RADIAL (`3`). Use it for organic enclosures or layered band/storm-like structures without a themed preset. Density, Fill, Gap Width, Seed, outline and alpha keep their CELL meanings; Seed selects occupancy, not smooth motion. Warping is periodic and fades near the poles, and occupied cells and their edge support share one coordinate field. This explicitly defines former fallback selector `6`; choose GRID (`0`) if that was intended. There is no built-in clock: animation remains guest-owned.
- These consistency-first corrections intentionally change older CELL patterns without changing the packed instruction layout. Guest-controlled density is byte-stepped. In rounded-period variants, some adjacent values select the same lattice, while a count change rebuilds its cells. This does not promise stable population or smooth density animation. RADIAL has separate polar-geometry and shrinking-pole checks; these do not certify other variants' pole appearance or substitute for human visual review.

- APERTURE's local hash now materializes integer lattice/bar/cell IDs before hashing to remove observed lattice cuts. ABI, packed fields, variant IDs, and parameter meanings are unchanged, but seeded IRREGULAR/BARS layouts and common horizon relief may change; recheck authored scenes rather than assuming historical pixel identity. This is not a cross-GPU pixel-identity guarantee.

- APERTURE/BARS now evaluates the seeded bar distance on both sides of the rectangle boundary and uses `max(base_sdf, -bar_dist)` for a continuous signed-distance subtraction. Bar count, seeded offsets, bow, taper and segmentation are retained. Opening/occluder sign controls pass, but the distance shoulders change: softness, frame weights, painted RGB and later region-masked features can visibly differ near bars and caps. The fixed-orientation GPU gate passes unchanged limits; this is not global appearance parity.
- APERTURE/MULTI is a regular grid of rectangular openings in one rectangular frame. Param D selects N per axis (clamped to 1..8); N=1 matches RECT. Half Width/Height set the outer opening envelope; internal mullions use 10% of the smaller cell spacing and stay wall-owned. Frame Thickness expands the outer rectangular frame. Seeded offsets, shear and cell-ownership distance patches are removed: old MULTI scenes intentionally change under the consistency-first policy, without changing opcodes, packed fields or guest-owned animation. The regular-grid contour gate includes real zero contours, solid mullions and a phantom-zero negative control; human appearance acceptance remains separate.

- MOTTLE, ADVECT and MASS now materialize integer lattice corners at their shared noise-hash boundary. This removes the tested carrier-cell cuts without changing fields, phase drivers, scale or variant meanings. Local noise values and shaded pixels can change; this is not historical pixel parity or whole-algorithm continuity. The existing PATCHES integer-corner path remains unchanged in the matched control. Recheck authored scenes visually.
- FLOW's 3D lattice hash now forms its dot product in integer space to remove cell-boundary cuts. Existing seeded noise, turbulence, and caustic patterns can change; the separate 2D streak hash is unchanged. This is not a cross-GPU pixel-identity guarantee.
- VEIL/RAIN_WALL retains the smooth finite streak envelope and separate core/glow maximum unions. Loop-safe travel now uses one or two vertical-chart circuits per phase cycle and circular streak distance. The authored domain/pole fade still controls visibility. This approved motion change intentionally changes existing nonzero-phase drop positions and speed distribution; it is not historical pixel compatibility. Packed fields, opcode IDs, colors and other VEIL variants are unchanged.

- Direct procedural rendering and cached lighting now use the same layer evaluator. Existing direct/background ownership is retained: each bounds result supplies the regions consumed by later features. Cached lighting no longer merges those regions with the initial all-sky default. Lighting may change where it previously disagreed with the visible program; instruction packing and source order are unchanged.
- Normal mapping again samples its BC5 texture; the explicit skip flag still disables it. Materials that relied on the temporary global bypass will visibly change.
- RAMP now uses authored wall RGB without hidden softening, and feature masks sum selected ownership without the former single-region boost. See the [RAMP compatibility note](../api/epu.md#ramp-paint-and-rendering-compatibility) and [region-mask contract](../api/epu.md#region-mask-3-bit-bitfield); old pixels are not preserved.
- PATCHES angular domains now sample a periodic circular embedding, and PATCHES reuses the shared non-trigonometric noise hash to avoid observed lattice-boundary discontinuities. **Existing PATCHES seeds can produce different patterns in every domain** after this repair; DIRECT3D coordinates and the packed parameter meanings are unchanged, but the hash values are not. Recheck authored PATCHES configurations rather than assuming pixel-identical output across this renderer update. The STATIC variant intentionally remains discontinuous noise; it is not a smooth-variant seam guarantee.


---

## Quick Start

1. Create a packed EPU config: 8 × 128-bit instructions (stored as 16 `u64` values as 8 `[hi, lo]` pairs).
2. Call `epu_set(config_ptr)` near the start of `render()`, then call `draw_epu()` after your 3D geometry so the environment fills only background pixels.

**Determinism note:** The EPU has no host-managed time. To animate an environment, keep a deterministic `u8 phase` in your game state (e.g. `phase = phase.wrapping_add(1)` in each simulation `update()`, never in `render()`), write it into an opcode parameter that the authored variant actually consumes for motion (often `param_d`, but not universally), and call `epu_set(...)` again with the updated config.

**Authoring reality:** EPU work is procedural/generative world art. It is best at strong metaphorical place reads and ambient structural cues, not literal scene modeling or screen-space UI.

The following render bodies require the canonical SDK (`include/zx/mod.rs`, `include/zx.h`, or `include/zx.zig`) and exported guest lifecycle functions. The all-NOP template is intentionally black; substitute an authored/exported configuration.

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
        epu_set(ENV.as_ptr().cast()); // Set environment config
        // ... draw scene geometry
        draw_epu(); // Draw environment background
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
    epu_set(env_config);  // Set environment config
    // ... draw scene geometry
    draw_epu();           // Draw environment background
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
    epu_set(&env_config);  // Set environment config
    // ... draw scene geometry
    draw_epu();            // Draw environment background
}
```
{{#endtab}}

{{#endtabs}}

Reference presets and packing helpers:
- `examples/3-inspectors/epu-showcase/src/presets.rs`
- `examples/3-inspectors/epu-showcase/src/constants.rs`

---

## Architecture Overview

The EPU uses a 128-byte instruction-based configuration:

| Slot | Kind | Recommended Use |
|------|------|------------------|
| 0–7 | Mixed | Any authored sequence of bounds (`0x01..0x07`) and features (`0x08..0x18`), with `NOP` for unused slots. |

**Bounds** defines the low-frequency envelope and region weights (sky/walls/floor).

**Features** add higher-frequency motifs (decals, grids, stars, clouds, etc.).

Bounds replace the current region weights for subsequent features; they do not retroactively re-mask earlier paint. Bounds can themselves paint, including consecutive bounds layers. **The region-mask bits only affect features.** Bounds ignore those bits; setting a bounds layer to WALLS does not crop its paint to a wall. The inspector disables these inactive checkboxes without rewriting their stored bits. Put details after the last bounds paint that would cover them. A PLANE floor can use `REGION_ALL` plus a downward direction to project below the viewer independently of the current region weights.

---

## Opcode Overview

| Opcode | Name | Best For | Notes |
|--------|------|----------|-------|
| 0x01 | RAMP | Base bounds | Often used first to explicitly set `up/ceil/floor/softness`, but any bounds opcode can be layer 0. |
| 0x02 | SECTOR | Opening wedge / interior cues | Bounds modifier |
| 0x03 | SILHOUETTE | Skyline / horizon cutout | Bounds modifier |
| 0x04 | SPLIT | Geometric divisions | Bounds |
| 0x08 | DECAL | Sun disks, signage, portals | Feature |
| 0x09 | GRID | Panels, architectural lines | Feature |
| 0x0A | SCATTER | Stars, dust, particles | Feature |
| 0x0B | FLOW | Looping noise, streaks, caustics | Feature |
| 0x18 | SCATTER_PHASED | Independent fixed-point twinkle / fallout brightness | Feature |
| 0x12 | LOBE | Sun glow, lamps, neon spill | Feature |
| 0x13 | BAND | Horizon bands / rings | Feature |

---

## Authoring Workflow

- Start with a bounds layer and the features you need, or adapt an ordinary program from `epu-showcase`; the examples are optional, not engine presets.
- Open F4's EPU panel, select the game environment and enable **LOCK** to edit a frozen configuration with the semantic controls. Lock replaces **all** game environments for inspection; disable it before verifying the guest's own output or animation.
- Save/reload or export that configuration through the existing inspector/workbench. Copy the exported Rust into your guest, call `epu_set(config_ptr)`, then `draw_epu()`. Exported configuration is a snapshot, not an animation-controller export.
- The showcase has 20 optional compositions and 7 capability scenes covering all 24 non-NOP opcodes. See `examples/3-inspectors/epu-showcase/README.md` for controls and the source-level phase regression; opcode presence does not imply every variant or aesthetic is accepted.

### Slot Conventions

| Slots | Kind | Recommended Use |
|------|------|------------------|
| 0-7 | Mixed | Bounds and features are evaluated in authored order. Use early bounds to establish the envelope, then spend later slots on readable feature carriers unless you deliberately need a later bounds remap. |

### Bounds/Feature Cadence (Don\'t Waste Slots)

Bounds opcodes don\'t just draw color; they also rewrite the **region weights** (`SKY/WALLS/FLOOR`) that later feature opcodes use for masking.

- Stack bounds when their paint or region remapping is useful; no intervening feature is required.
- Prefer a cadence like: `BOUNDS (define/reshape regions) -> FEATURES (use regions) -> BOUNDS (carve/retag: APERTURE/SPLIT) -> FEATURES (decorate + animate)`.
- If you insert a bounds opcode later in the 8-layer program, it only affects features **after** it (it cannot retroactively re-mask earlier features).

### Openings and projected floors

Region names describe the current bounds mask, not guaranteed physical locations. For `APERTURE`, `SKY` is the opening, `WALLS` is its frame, and `FLOOR` is the surrounding area outside the frame on the front hemisphere. That surrounding area can be above or beside the opening as well as below it.

`PLANE` contributes only toward its authored normal: it rejects directions with `dot(view_direction, plane_normal) <= 0.05` and fades in toward `0.2`. Use a downward normal for a floor below the viewer; an upward normal paints above instead. A `FLOOR` mask alone does not correct an incorrectly oriented plane. After APERTURE, a downward PLANE masked to `FLOOR` can texture the lower surround while preserving the opening and frame; partial region boundaries still blend according to their weights.

To check a composition, freeze the camera and earlier layers, change only the plane's gap or scale, and confirm that the intended lower region responds while the opening, frame and upper surround do not. Keep the preceding bounds when isolating the feature's mask behavior. This is directional environment composition, not room geometry, parallax, or a connected portal; a technically localized texture is not by itself proof of a convincing interior.

### Bounds vs Features in Practice

- Bounds are your scene envelope: horizon, enclosure, wedges, openings, and region weights.
- Features are where most readable world detail actually lives: signage, scan planes, rain curtains, stars, caustics, water reads, glow accents, and projection planes.
- If a preset keeps collapsing in direct view while the reflection still looks better, it often means the bounds are doing too much visual work and the later feature layers are not carrying enough world-readable structure.

### Variant-Specific Motion Reality

Do not assume `param_d` means smooth animation for every opcode or every variant.

Keep the driver in rollback-covered simulation state; `render()` only emits its current parameters. Byte wrap is not proof of visual periodicity for every variant. Save/export captures the configuration, not the game-owned animation driver. The showcase `animate_phases` helper copies the authored phase offset, adds the guest counter, and edits only the supported phase byte: `param_c` for SCATTER_PHASED, otherwise `param_d` for its phase-driven variants. It preserves seeds and structural fields. Games remain free to author other deterministic parameter changes.

- Phase-driven movers in current practice include `FLOW`, `GRID`, `LOBE`, `DECAL`, `PLANE/WATER`, and `PORTAL/VORTEX`; inspect the intended variant rather than assuming every phase byte has identical motion semantics.
- `VEIL/RAIN_WALL` uses `param_d / 256` as cyclic phase. Each seeded streak travels one or two vertical-chart circuits per cycle; circular distance preserves its smooth finite envelope through wrap. A guest can advance the byte by one or two, hold it on selected ticks, or drive it from other deterministic state. The approved loop repair changes the old fractional-speed distribution and drop positions, not the seed offsets, packed fields or other VEIL variants. There is no hidden host clock or automatic phase driver.
- `SCATTER` uses `param_d` as seed. Animating that byte replaces points rather than moving or twinkling a fixed field. Keep the seed fixed. Endpoint color modulation preserves positions but is not independent per-star twinkle; see [stable-point brightness](#stable-point-brightness).
- SCATTER angular-domain correction: AXIS_CYL/AXIS_POLAR cells are now reconstructed around the authored axis, rather than normalized as world-space XYZ coordinates. Existing angular-domain point layouts therefore change; DIRECT3D and packed fields are unchanged. Angular cell identities also wrap periodically, repairing the sampled azimuth seam without added blur; this changes angular seeded layouts. TANGENT_LOCAL now distributes/reconstructs points in its authored axis frame rather than normalizing seeded world coordinates; its layouts and brightness change. TANGENT_LOCAL now searches the point footprint rather than a fixed neighborhood, repairing tested cell-boundary cuts; large-size/density cost remains unaccepted, and this does not establish artifact-free output for all settings. Pole continuity and complete large-point neighborhood coverage remain unverified.
- `PORTAL/RECT` is a static SDF shape; it can read as a frame or backplate, but not as a self-animating volumetric field.
- `TRACE/LIGHTNING` is a static strike shape; use other layers for storm cadence.
- `APERTURE` is a bounds remapper, not a feature-layer detail primitive.
- `BAND` phase is azimuthal modulation, so it is better as support than as a general scrolling horizon effect.

If a scene depends on behavior the current opcode surface cannot supply, treat that as a real opcode-family or directionality gap. That is a legitimate signal to improve the engine/opcode surface instead of forcing more content churn into an impossible target.

### RAMP inspector controls

- **Floor threshold** and **Ceiling threshold** independently edit 16 signed steps from `-1` to `1` along the authored up vector. The display shows the quantized value; exact zero is not representable. Floor occupies the low nibble of `param_d`, ceiling the high nibble; editing either preserves the other and all unrelated fields.
- Reversed authored endpoints remain unchanged in storage. The renderer sorts them for evaluation: the lower value acts as floor and the higher as ceiling. The inspector reports the effective sorted values when reversed.
- **Color B** RGB remains editable as the floor/ground color. Its **Alpha** control is disabled for RAMP because RAMP ignores `alpha_b`; the stored value is preserved. `alpha_a` controls visible paint opacity.
- Generic metadata exposes `param_d` only as an explicitly **advanced packed** byte (`0..255`), not a normalized threshold. Native inspector controls require no bit editing. Packing, shader math, and saved cartridge encoding are unchanged.

### Inactive common controls

Color B alpha is not a universal opacity control. **SECTOR, SILHOUETTE, SPLIT,
APERTURE, GRID, legacy SCATTER, FLOW, MOTTLE, ADVECT, SURFACE and MASS ignore it**,
as do RAMP, ATMOSPHERE, PLANE and LOBE. Their native Alpha B control is disabled,
with its stored nibble preserved. SCATTER_PHASED and BAND instead use it for
modulation depth; their controls remain active. Other opcodes retain their
documented paint/region-specific meanings.

GRID additionally ignores Color B RGB and direction: it uses Color A and a fixed
Y-up cylindrical chart. The native Color B picker and direction editor are
disabled rather than changing stored values. This does not alter rendering,
packing, or exported cartridge fields.

### FLOW inspector controls

- **Frequency** selects an integer from `1..16`, matching `1 + floor(param_a * 15 / 255)`. An edit writes the lowest byte for that frequency, `(frequency - 1) * 17`; idle display and edits to other fields preserve noncanonical raw bytes.
- **Pattern** offers named **Noise**, **Streaks**, and **Caustic** choices. These edit only the low nibble of `param_c`, preserving the octave nibble. They are not `meta5` variants; FLOW ignores `meta5` and Color B alpha.
- **Octaves** independently edits the high nibble with effective values `0..4`, preserving the pattern. The shader clamps stored values `5..15` to `4`; display and pattern changes do not rewrite them. Zero is valid: Noise becomes a constant pattern value of `0.5`. Streaks uses the clamped value for lane variation, not an octave loop. Caustic and raw patterns `3..15` ignore octaves, so the native octave control is disabled for them.
- Unsupported/raw patterns `3..15` remain stored until explicitly selecting a named pattern. Their existing shader behavior is static single-noise fallback, not animated Noise. Phase and direction controls are disabled for these patterns; stored values are retained.
- **Cyclic phase** edits 256 steps and displays `raw/256` turns: `0/256 = 0.000000`, `128/256 = 0.500000`, `255/256 = 0.996094` (rounded display). The next byte wraps to zero; there is no duplicated `1.0` endpoint. This control does not add an animation driver.
- Generic metadata uses explicitly **advanced raw/packed** identity-byte fallbacks for frequency, octaves/pattern, and phase, with decoding rules in the labels. Native controls require no hand-packed bytes. Shader algorithms and cartridge encoding are unchanged.

### SCATTER inspector controls

- **Base radius (radians)** shows the variant-adjusted size. Its range depends on density and variant; changing either can change the displayed radius without changing the stored size byte. RAIN and EMBERS additionally show their extended falloff radius.
- **Brightness variation (static)** edits the high four bits of `param_c` in 16 steps while preserving its reserved low bits. It controls seeded brightness variation, not an independent animation clock. WINDOWS ignores it, so that control is disabled for WINDOWS.
- **Seed** is the raw `0..255` value. Changing it selects a different field; it does not smoothly move the existing points.

These inspector controls preserve the packed cartridge format. SCATTER's authoritative metadata now labels size as an **advanced raw** byte with density/variant-dependent radius, brightness variation as an **advanced packed** byte (high nibble divided by 15, reserved low nibble, inactive for WINDOWS), and seed as a **raw byte**. None advertises a fixed universal radius or normalized whole-byte twinkle/seed. The unchanged native radius display follows `mix(0.001, max(0.05, 0.5 / (1 + density_raw)), size_raw / 255) * variant_multiplier`; external generic consumers get honest raw controls rather than a fabricated semantic mapping.

### Stable-point brightness

SCATTER mixes its two RGB endpoints using a stable per-point hash. A guest can cycle those endpoints without changing point placement or the packed format, but every point remains a mixture of the same two signals. This is **not independent per-star twinkle**: it can visibly pulse the entire field even when peak times differ. Use it for coordinated endpoint-color modulation, not as a substitute for per-point phase control. Legacy SCATTER (`0x0A`) has no such phase input; its packed brightness variation remains static. For independent modulation use the opt-in instruction below.

### Independent point brightness

Choose **SCATTER_PHASED (`0x18`)** for one fixed point field whose points brighten independently. It reuses SCATTER's shapes, domains, density, size, RGB endpoints and seed; only the modulation controls differ:

- **Phase (turns, guest-owned)** uses the whole `param_c` byte as `raw / 256`. The next byte after 255 wraps to zero, without a duplicate 1.0 endpoint. Do not interpret this byte as the legacy variation nibble.
- **Mod depth** uses Color B's `alpha_b` nibble: `0` is steady, `15` enables fully off-to-on modulation; intermediate values blend toward steady brightness. Color A's `alpha_a` still controls layer opacity.
- Each point uses its stable seed-derived phase offset and an integer rate from 1 through 4. The brightness coefficient is `smoothstep(0.05, 0.95, 0.5 + 0.5 * cos(TAU * (phase * rate + offset)))`. Brief dark/bright holds reach exact 0 and 1; overlapping points or background light need not make the entire pixel black.
- Keep seed fixed. The guest owns phase advance, reverse, hold, skipped steps and update cadence. This is brightness modulation, not particle movement, a clock, or a weather controller.

Export the new instruction from the inspector with modulation depth 15. Advance a wrapping `u8` phase only in rollback-covered guest `update()`; a step of 4 gives the faster example, 2 gives half that rate, and 1 on alternate updates gives slower motion without a new field. Render from a fresh copy of the export and set only its phase byte:

<!-- scatter-phase-driver:start -->
```rust
pub fn set_scatter_phase(layer: &mut [u64; 2], phase: u8) {
    assert_eq!(layer[0] >> 59, 0x18, "requires SCATTER_PHASED");
    layer[1] = (layer[1] & !(255u64 << 32)) | (u64::from(phase) << 32);
}
```
<!-- scatter-phase-driver:end -->

Call `set_scatter_phase(&mut layers[point_layer], phase)` before the unchanged `epu_set` submission. Export contains a phase snapshot, not game update rules. Native Rust tooling can use the existing `EpuLayer` with `EpuOpcode::ScatterPhased`; no additional parameter wrapper is needed.

**Compatibility:** the 128-byte program and FFI signatures are unchanged. This is a new opt-in instruction: older players treat `0x18` as NOP. Existing supported SCATTER encodings retain their old interpretation; do not silently reinterpret an old layer's packed variation byte or unused alpha as phase/depth. Author the new layer deliberately.

### Composition and legacy endpoint modulation

For a two-layer example, put SILHOUETTE/CITY first and SCATTER_PHASED/STARS second. Select **SKY** for the point layer (the canonical mask is `0b100`, not `0b001`, which is FLOOR). Points are then masked out of opaque buildings. The city dimensions and point brightness remain independent edits. CITY has a finite wall depth below each roof height; if its lower edge enters the view, the floor region appears there. Increase the authored wall-depth byte when the wall should continue below the framing; the renderer does not automatically extend it. This is an example composition, not a required palette or themed preset.

Use `set_scatter_phase` for independent point twinkle in that composition. The optional legacy helper below instead scales the exported endpoint colors. Its legacy `twinkle` name does not imply independent star animation: same-hue endpoints give correlated brightness modulation; different hues allow color cycling. The wave and phase offset are example guest choices, not engine policy.

<!-- scatter-brightness-driver:start -->
```rust
// Guest-side integer smoothstep wave. No renderer clock, seed changes or float RNG.
pub fn brightness(phase: u8) -> u8 {
    let p = u32::from(phase);
    let x = p.min(256 - p);
    (48 + (x * x * (384 - 2 * x) * 192 + (1 << 20)) / (1 << 21)) as u8
}

fn scale_rgb(rgb: u32, amount: u8) -> u32 {
    let mut scaled = 0;
    for shift in [0, 8, 16] {
        scaled |= ((((rgb >> shift) & 255) * u32::from(amount) + 127) / 255) << shift;
    }
    scaled
}

// Start from the immutable inspector export each render, not last frame's tint.
// Only RGB24 endpoints change; brightness signals remain shared across points.
pub fn twinkle(layer: &mut [u64; 2], phase: u8) {
    let a = u64::from(scale_rgb(
        ((layer[0] >> 24) & 0xFF_FFFF) as u32,
        brightness(phase),
    ));
    let b = u64::from(scale_rgb(
        (layer[0] & 0xFF_FFFF) as u32,
        brightness(phase.wrapping_add(85)),
    ));
    layer[0] = (layer[0] & 0xFFFF_0000_0000_0000) | (a << 24) | b;
}
```
<!-- scatter-brightness-driver:end -->

For coordinated endpoint-color modulation (not independent twinkle), keep a wrapping `u8` phase in rollback-covered game state and advance it only in `update()`. In `render()`, start with a fresh copy of the immutable inspector export (`let mut layers = SCENE;`), apply `twinkle(&mut layers[1], phase)`, and submit it through the usual `epu_set` / `environment_index` calls. Do not scale last frame's colors repeatedly: that would accumulate darkening. Holding the phase holds the output; stepping it by two skips one authored sample without reseeding. Export saves the configuration, not these game-owned update rules.

SCATTER's shared angular-distance calculation uses `atan2(length(cross(a, b)), dot(a, b))`. Unlike `acos(dot(a, b))`, it retains the core of a tiny point when float rounding puts its self-dot just below one. This precision repair can brighten small point cores; it does not change seeded positions or the authored radius/profile. Broad pole/large-neighborhood coverage and visual acceptance remain separate gates.

### meta5 Behavior

- `meta5` encodes `(domain_id << 3) | variant_id` for opcodes that support domain/variant selection.
- For opcodes that do not use domain/variant, set `meta5 = 0`.

---

## Split-Screen / Multiple Viewports

Call `viewport(...)` and then `draw_epu()` per viewport/pass where you want an environment background.

---

## See Also

- [EPU API Reference](../api/epu.md) - FFI signatures and instruction encoding
- [EPU Architecture Overview](../architecture/epu-overview.md) - Compute pipeline details
- Opcode implementation and parameter decoding: `nethercore-zx/shaders/epu/` in the source checkout.

## GRID count and phase authoring

The separate `GRID (0x09)` feature now uses whole repeat counts, even for checker, and whole pattern cycles per phase loop. The native inspector shows **Repeats**, **Pattern**, **Cycles / loop** and the exact **phase byte / 256**. It preserves the original raw count byte and unused fields when idle; checker disables the inactive thickness control. The guest inspector's `repeat raw`, `pat+cycles` and `phase/256` labels are packed-byte controls; see the [complete field contract](../api/epu.md#grid-repeats-and-cyclic-scrolling).

Keep count, pattern and cycle count fixed for scrolling. Store phase in rollback-covered simulation state; update it with `phase.wrapping_add(step)` and emit the current configuration in `render()`. Step zero holds, one advances normally, and two skips a step without inventing an engine controller. High cycle counts retain sharp patterns and may move quickly between frames; byte wrapping is periodic, not an antialiasing promise. Zero cycles is stationary regardless of phase.

<!-- grid-phase-driver:start -->
```rust
/// Change only GRID's guest-owned phase byte; keep layout, cycles and paint intact.
pub fn set_grid_phase(layer: &mut [u64; 2], phase: u8) {
    assert_eq!(layer[0] >> 59, 0x09, "expected a GRID instruction");
    layer[1] = (layer[1] & !(0xffu64 << 24)) | (u64::from(phase) << 24);
}
```
<!-- grid-phase-driver:end -->

Apply this to the exported GRID layer before calling the canonical SDK `epu_set`. The opcode assertion rejects an accidental write to another algorithm. This is a guest-side example, not a new cartridge API or host-owned animation system. Old GRID layouts and speeds intentionally change; CELL/GRID is a different primitive and retains its separate contract. Cylindrical pole singularities remain a projection limitation, not a claim of universal polar continuity.

## LOBE directional lights

Use LOBE for centre-peaking directional gain and BAND for an intentional ring. Choose core Color A and edge Color B, point the axis, then narrow the footprint with Exponent. Color falloff changes the palette transition, not the footprint. The removed centre suppression intentionally brightens older LOBE renders; lower authored Brightness if that is your desired result, rather than reintroducing an engine-owned style.

The native Waveform selector names steady, sine, triangle and four-pulse strobe. Steady means modulation is disabled; its Phase field is inactive but preserved. Animated modes use `param_d / 256` cycles. A guest may advance, hold or restore that byte in rollback-covered simulation state; rendering only emits the state. Strobe has four deliberately sharp pulses per cycle, not an interpolation guarantee. Color B alpha remains stored but unused.

## Carrier loop closure

ADVECT and MASS separate their cyclic time from deterministic shape offsets. Every authored variant in DIRECT3D, AXIS_CYL and AXIS_POLAR closes over the 256-step phase cycle. Phase zero retains the prior shape; nonzero phases deliberately replace the old nonperiodic motion. No engine clock or new packed field is involved. MOTTLE already has cyclic motion and a single world-space coordinate path: its stored domain bits are ignored, not alternative projections.

The guest selects the layer, updates its own phase, and can hold or skip steps. This helper changes only the phase byte:

```rust
/// Change only an ADVECT/MASS layer's guest-owned cyclic phase byte.
pub fn set_carrier_phase(layer: &mut [u64; 2], phase: u8) {
    assert!(matches!(layer[0] >> 59, 0x15 | 0x17), "expected ADVECT or MASS");
    layer[1] = (layer[1] & !(0xffu64 << 24)) | (u64::from(phase) << 24);
}
```

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

## ATMOSPHERE authoring

Choose the mathematical contribution first: ABSORPTION with Multiply for directional darkening, RAYLEIGH for an altitude tint, MIE for a directional halo, FULL for both, or ALIEN for a sinusoidal color band. There is no geometry fog or automatic time/weather behavior.

Gradient controls use the enclosure's up direction. MIE/FULL's separate layer direction points the halo. Horizon Y shifts the full-sphere altitude ramp; it is not an occlusion plane. Only MIE/FULL use Mie concentration/exponent; MIE ignores the gradient controls and Color B. Color B alpha is always unused. The native inspector disables inactive scalar/direction controls without rewriting their bytes, so switching variants preserves authored alternatives. There is no phase field: `param_d` is a Mie exponent, not an animation byte. A guest can explicitly update strength, tint or direction in its own simulation state.

## PORTAL: paint, shape and guest phase

Use the interior and glow alphas independently. PORTAL now stores straight component paint; the shared blend applies coverage once. Intensity affects the edge glow only. The size and glow-width sliders use tangent-chart units; the inspector also shows the base angular half extent.

For a moving VORTEX contour, use nonzero roughness and advance its phase from deterministic guest state. This composes the existing rough TEAR shape with the existing spiral warp, rather than adding a new preset or controller. Zero roughness remains circular and phase has no meaningful visible effect. Other PORTAL variants ignore phase; inactive stored bytes are preserved.

<!-- portal-phase-driver:start -->
```rust
/// Change only a VORTEX portal's guest-owned cyclic phase byte.
pub fn set_portal_phase(layer: &mut [u64; 2], phase: u8) {
    assert_eq!(layer[0] >> 59, 0x11, "expected PORTAL");
    assert_eq!((layer[0] >> 48) & 7, 3, "phase is used by VORTEX");
    layer[1] = (layer[1] & !(0xffu64 << 24)) | (u64::from(phase) << 24);
}
```
<!-- portal-phase-driver:end -->

Call `set_portal_phase` on the VORTEX instruction and submit the resulting array through `epu_set`. Increment with `wrapping_add` in guest update state; repeating a byte holds the frame. The period is 256 steps, not 255. Old PORTAL renders can change because paint coverage is no longer applied twice and a rough VORTEX now actually moves.

## DECAL direction and sidedness

Aim a DECAL at the direction where it should appear. A backward duplicate of RECT or LINE is no longer drawn. Front-facing appearance and the existing pulse are preserved. LINE still extends vertically within the front hemisphere; compose an authored bound when it must be clipped to a smaller region. This is a support correction, not a new shape or hidden fade.

## DECAL opacity

DECAL fill/glow alpha now applies once through the layer blend, not twice. For example, lowering fill alpha lowers its contribution proportionally rather than squaring it. Antialiased edges follow the same rule, and zero coverage remains zero. Existing partially transparent DECAL renders change; shapes, coverage, brightness, phase and cartridge fields do not. This paint correction is separate from the sidedness correction described above.

## CELESTIAL phase authoring

MOON/PLANET phase lighting now follows the projected unit sphere: Size does not change illumination at the same disk point, and the night side receives no direct surface light. Color B rim/atmosphere light remains independent; set Color B alpha to zero when inspecting the terminator.

The existing phase angle is `param_c / 255 * 360` degrees: full at 0/360, half at 90/270, new at 180. Byte 255 repeats the full endpoint. A guest can advance 0..254 and wrap to zero for equal angular steps, or hold/reverse/skip steps explicitly. Advance only in simulation `update()` and emit the current phase in `render()`; there is no engine clock. Use the existing instruction parameter setter to change only Phase, not Size, direction or colors. Older MOON/PLANET shading changes; phase angles and cartridge layout do not.

## CELESTIAL ring tilt

For RINGED, adjust Angular size and then Ring tilt (0–90 degrees). A low tilt produces flattened ring lobes around the disk; 90 degrees exposes the full annulus. Color B and its alpha control the rings. The inspector shows the effective degrees without normalizing the stored byte, and marks the unused Phase field as retained rather than active.

Tilt is not a cyclic phase. A guest that wants a smooth repeating inclination should reverse its sweep rather than wrap 255 directly to 0; hold or update it in the guest's rollback-covered state. There is no host clock. The ellipse correction changes older RINGED appearances, not cartridge fields or SDK calls.

## CELESTIAL surface texture footprint

MOON, PLANET and GAS_GIANT detail covers the projected body disk at every authored Size. The texture projection uses the fixed projected body radius, not a varying per-pixel radius. Noise definitions and phase lighting are unchanged; this is a texture-footprint correction, not a new surface preset.

## CELESTIAL angular distance

Body direction is a full-sphere position: separation spans 0..180 degrees, not a
plateau after 90 degrees. The signed-cosine repair corrects back-hemisphere corona
and ring contributions while retaining the authored size, gain, UV and phase fields.
It does not introduce a host controller or certify remaining ring/UV/phase geometry.

## CELESTIAL gain authoring

CELESTIAL brightness is a single layer-weight gain (`2 * raw / 255`), not two multipliers or a separate exposure control. Color A alpha and region masking still determine coverage, and the ordinary blend clamp still applies. Low-strength ADD contributions no longer square the authored gain; old low-strength scenes brighten and duplicated high-strength RGB boosting is removed. Geometry and phase are unchanged in this bounded correction. See the [field contract](../api/epu.md#celestial-contribution-strength).

## BAND depth and phase authoring

The native inspector labels Color B alpha **Mod depth** and displays the exact **phase byte / 256**. At zero depth, phase stays stored but its inactive control is disabled. Depth 15 uses the existing four-wave 40..100% profile, not SCATTER_PHASED's full off/on brightness range. The guest inspector keeps raw alpha fields: BAND `alpha_b` is depth, `alpha_a` is opacity, and `phase/256` is position.

Keep depth and shape fixed; update a wrapping phase byte in rollback-covered simulation state, then emit it in `render()`. Step 0 holds, 1 advances, and 2 skips a sample, including wrap. Byte stepping is finite-resolution motion, not automatic smoothing. These helpers operate on the exact exported BAND layer and reject another opcode or an out-of-range depth before changing it:

<!-- band-phase-driver:start -->
```rust
/// Change only BAND's guest-owned phase byte; depth and all sibling bits stay intact.
pub fn set_band_phase(layer: &mut [u64; 2], phase: u8) {
    assert_eq!(layer[0] >> 59, 0x13, "expected a BAND instruction");
    layer[1] = (layer[1] & !(0xffu64 << 24)) | (u64::from(phase) << 24);
}

/// Depth is explicit: 0 = plain ring; 15 = the full four-wave profile.
pub fn set_band_depth(layer: &mut [u64; 2], depth: u8) {
    assert_eq!(layer[0] >> 59, 0x13, "expected a BAND instruction");
    assert!(depth <= 15, "BAND depth must be 0..15");
    layer[1] = (layer[1] & !15u64) | u64::from(depth);
}
```
<!-- band-phase-driver:end -->

Call `set_band_depth(&mut config[layer], 15)` once when selecting modulation; call `set_band_phase(&mut config[layer], phase)` when emitting the current guest state before SDK `epu_set`. Zero depth retains a plain ring. The [migration contract](../api/epu.md#band-depth-and-cyclic-phase) describes the deliberate change from phase-zero-as-off; no host clock or new packed fields are added.
