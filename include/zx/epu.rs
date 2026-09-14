//! Environment Processing Unit (EPU) — Instruction-Based API

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Store an EPU configuration (128-byte) as the current immediate-mode EPU source.
    ///
    /// Reads a 128-byte (8 x 128-bit = 16 x u64) environment configuration from WASM memory
    /// and stores it for the current render frame. The EPU compute pass runs automatically before
    /// rendering to build environment textures (EnvRadiance + SH9) for the internal slots
    /// referenced by the current frame's draws.
    ///
    /// # Arguments
    /// * `config_ptr` — Pointer to 16 u64 values (128 bytes total) in WASM memory
    ///
    /// # Configuration Layout
    /// Each environment is exactly 8 x 128-bit instructions (each stored as [hi, lo]):
    /// All eight slots are evaluated in authored order; there are no fixed bounds/feature ranges.
    /// Bounds (`0x01..0x07`) rewrite region weights consumed by subsequent features (`0x08+`).
    /// Only features consume the region mask. Bounds ignore those stored bits,
    /// write their own region weights, and blend their paint in authored order.
    /// Put features after the bounds paint they must survive. For a projected
    /// floor independent of those weights, use PLANE directed down with ALL.
    ///
    /// # Instruction Bit Layout (per 128-bit = 2 x u64)
    /// ```text
    /// u64 hi [bits 127..64]:
    ///   63..59  opcode     (5)   Which algorithm to run (32 opcodes)
    ///   58..56  region     (3)   Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
    ///   55..53  blend      (3)   8 blend modes
    ///   52..48  meta5      (5)   (domain_id<<3)|variant_id; use 0 when unused
    ///   47..24  color_a    (24)  RGB24 primary color
    ///   23..0   color_b    (24)  RGB24 secondary color
    ///
    /// u64 lo [bits 63..0]:
    ///   63..56  intensity  (8)   Layer brightness
    ///   55..48  param_a    (8)   Opcode-specific
    ///   47..40  param_b    (8)   Opcode-specific
    ///   39..32  param_c    (8)   Opcode-specific
    ///   31..24  param_d    (8)   Opcode-specific
    ///   23..8   direction  (16)  Octahedral-encoded direction
    ///   7..4    alpha_a    (4)   color_a alpha (0-15)
    ///   3..0    alpha_b    (4)   color_b alpha (0-15)
    /// ```
    ///
    /// # Opcodes (common)
    /// - 0x00: NOP (disable layer)
    /// - 0x01: RAMP (bounds gradient)
    /// - 0x02: SECTOR (bounds modifier)
    /// - 0x03: SILHOUETTE (bounds modifier)
    /// - 0x04: SPLIT (bounds source)
    /// - 0x05: CELL (bounds source)
    /// - 0x06: PATCHES (bounds source)
    /// - 0x07: APERTURE (bounds modifier)
    /// - 0x08: DECAL (sharp SDF shape)
    /// - 0x09: GRID (repeating lines/panels)
    /// - 0x0A: SCATTER (point field)
    /// - 0x0B: FLOW (animated noise/streaks)
    /// - 0x0C..0x13: radiance opcodes (TRACE/VEIL/ATMOSPHERE/PLANE/CELESTIAL/PORTAL/LOBE_RADIANCE/BAND_RADIANCE)
    /// # ADVECT/MASS transport charts
    /// AXIS_CYL and AXIS_POLAR close their azimuth smoothly near either pole,
    /// within transverse radius 0.05. This continues coordinates, not opacity:
    /// normal body/density evaluation still runs. DIRECT3D and coordinates
    /// outside that neighborhood are unchanged. Older pole detail may change;
    /// packed fields, signatures and guest-owned phase do not.
    /// ADVECT/MASS time is cyclic in every variant/domain; deterministic shape
    /// offsets are separate from the one-turn clock. Phase zero keeps its prior
    /// shape; nonzero phases replace the old nonperiodic motion. No clock or
    /// packed field is added. MOTTLE already loops and ignores its domain bits.
    ///
    /// # PATCHES chart coordinates
    ///
    /// CYL/POLAR close their azimuth coordinates inside transverse radius 0.05
    /// using the existing ADVECT/MASS closure. The old circle embedding remains
    /// outside the cap; direct and reserved domains are unchanged. No opacity
    /// fade is added. Local region patterns and their weighted alpha may change.
    /// STATIC lattice edges and max-sharpness region edges remain intentional.
    ///
    /// # VEIL variant controls (`0x0D`)
    /// Base count is 2 + floor(param_a*30/255); RAIN_WALL emits twice that count.
    /// Width is mix(0.002, max(0.05, 0.5/count), param_b/255) in chart units,
    /// before variant-specific scaling. The native editor shows integer count,
    /// thickness ratio and actual base width; idle keeps all original bytes.
    /// param_c is static sway (CURTAINS), wind slant (RAIN_WALL), or tilt (SHARDS).
    /// PILLARS/LASER_BARS ignore param_c. Only RAIN_WALL reads param_d/256 phase.
    /// PILLARS also ignore Color B and its alpha; other variants use their glow.
    /// Selectors 5..7 fall back to CURTAINS. Inactive controls retain stored values.
    ///
    /// AXIS_POLAR mirrors its existing support fade at both poles: zero within
    /// angular radius 0.05, full at 0.2 (normalized by pi). Radius 0.2..0.8 and
    /// other domains are unchanged. Old opposite-pole contribution is removed
    /// by explicit authorization; no new fade width or packed field is added.
    ///
    /// # PLANE variant controls (`0x0F`)
    /// Color A fills surfaces; increasing gap exposes more Color B. HEX uses regular
    /// hexagonal cells. GRATING runs from full bars at raw gap 0 to all gaps at 255.
    /// Older HEX/GRATING images change; packed fields and API signatures do not.
    /// Gain multiplies Color A alpha; scale is shared. Gap width / Color B apply to
    /// TILES, HEX, STONE, GRATING and PAVEMENT. Variation applies to STONE, SAND,
    /// GRASS and PAVEMENT, not material roughness. Only WATER reads phase: raw/256
    /// turns, with no duplicated endpoint. Direction points toward the visible plane:
    /// down for a floor, up for a ceiling. SAND/WATER/GRASS ignore Color B; all ignore
    /// its alpha. Inactive values remain stored; the inspector disables their controls.
    /// # CELL variants (`0x05`, domain 0)
    /// Variant IDs are GRID=0, HEX=1, VORONOI=2, RADIAL=3, SHATTER=4,
    /// BRICK=5, WARPED_RADIAL=6. RADIAL is regular; WARPED_RADIAL restores
    /// periodic warped ring/spoke cells with the same CELL field meanings.
    /// Selector 6 is now defined rather than a GRID fallback; selector 7 keeps
    /// that fallback. The opcode, packed layout and guest-owned time are unchanged.
    /// # GRID repeats and phase (`0x09`, not CELL/GRID)
    /// param_a selects nearest whole repeats 1..64, or nearest even 2..64 for checker.
    /// param_c high nibble selects stripes=0, grid=1, checker=2 (other values: stripes).
    /// Its low nibble is 0..15 whole pattern cycles per phase loop; zero is stationary.
    /// param_d / 256 is guest phase; checker cycles span two cells. Old layouts/rates
    /// intentionally change, not packed fields. Thickness is unused by checker;
    /// Color B and direction remain unused. Cylindrical poles have undefined azimuth.
    /// # BAND depth and phase (`0x13`)
    /// Color B alpha is modulation depth 0..15: zero is a plain ring; 15 retains
    /// the four-wave 40..100% profile. Color A alpha remains layer opacity.
    /// param_d / 256 is cyclic position, including zero, not an off sentinel.
    /// Old animated BAND programs need nonzero depth; old renders may change.
    /// The opcode, layout and guest API are unchanged; time stays guest-owned.
    /// # CELESTIAL strength (`0x10`)
    /// Intensity is 2 * raw / 255, applied once in layer weight (not again in RGB).
    /// Color A alpha and region weight still apply; the normal blend clamps coverage.
    /// Zero disables the feature; this is not separate HDR exposure. Older renders
    /// change where the former duplicate RGB gain applied. Geometry/phase are unchanged.
    /// CELESTIAL direction distance covers 0..180 degrees; negative cosines retain
    /// their sign instead of flattening the back hemisphere to 90 degrees.
    /// CELESTIAL surface detail uses the fixed projected body radius for unit-disk UVs.
    /// RINGED (variant 4): param_d / 255 * 90 is ring inclination in degrees,
    /// from edge-on (zero ring coverage) to face-on (complete annulus). Intermediate
    /// tilts form an ellipse; bands follow that radius. Phase (param_c) is unused.
    /// Tilt is not cyclic: the guest owns any held state or reversing sweep.
    /// This repairs older ring geometry without changing the layout or SDK calls.
    /// # LOBE centre-bright glow (`0x12`)
    /// Cosine-power directional gain has no centre suppression; use BAND for rings.
    /// param_a controls exponent 1..64, param_b controls the A-to-B color curve.
    /// param_c: 0 steady (modulation off), 1 sine, 2 triangle, 3 four-pulse strobe;
    /// reserved values use sine. param_d / 256 is guest-owned cyclic phase,
    /// inactive in steady mode. Color A alpha controls coverage; B alpha is unused.
    /// Existing LOBE centres become brighter; packed fields and calls are unchanged.
    /// # DECAL front-facing support
    /// A stamp paints only the front hemisphere of its centre direction; RECT/LINE
    /// no longer repeat behind it. Front-facing shape/pulse is unchanged. LINE
    /// remains an unbounded front-chart stripe, not a finite segment or hidden fade.
    /// Packed fields and API signatures are unchanged.
    /// # ATMOSPHERE controls (`0x0E`)
    /// Directional environment tint only: no geometry fog or intrinsic phase.
    /// Variants: ABSORPTION=0, RAYLEIGH=1, MIE=2, FULL=3, ALIEN=4; 5..7 emit nothing.
    /// Only MIE/FULL use direction and param_c/d (Mie concentration/exponent).
    /// MIE ignores param_a/b (gradient falloff/horizon shift) and Color B RGB.
    /// Color B alpha and domain are unused; stored fields/signatures stay unchanged.
    /// Strength and Color A alpha apply once; the shared blend clamps weight/RGB.
    /// # PORTAL paint and VORTEX phase
    /// Interior/glow alphas select their own straight paint; coverage applies once.
    /// Intensity scales glow only. Size/glow width use tangent-chart units.
    /// VORTEX composes the rough TEAR contour with the spiral warp: roughness zero
    /// is circular; nonzero roughness gives visible guest-owned phase motion.
    /// Phase is the cyclic param_d byte / 256; other variants ignore it.
    /// Packed fields/signatures stay unchanged; older PORTAL images may change.
    ///
    /// # DECAL paint coverage
    /// Fill/glow alpha and antialiased support apply once via the layer blend.
    /// Partially covered paint changes from older squared-coverage renders;
    /// geometric coverage, shapes, phase and packed fields are unchanged.
    /// # CELESTIAL MOON/PLANET phase lighting
    /// A projected unit-sphere normal makes phase illumination independent of Size.
    /// Night-side direct surface light is zero; Color B rim light is separate.
    /// param_c / 255 * 360 degrees: 0/360 full, 90/270 half, 180 new.
    /// Byte 255 duplicates full; guest cycles may use 0..254 for equal angle steps.
    /// This changes older surface shading, not phase angles or cartridge fields.
    /// # Blend Modes
    /// - 0: ADD (dst + src * a)
    /// - 1: MULTIPLY (dst * mix(1, src, a))
    /// - 2: MAX (max(dst, src * a))
    /// - 3: LERP (mix(dst, src, a))
    /// - 4: SCREEN (1 - (1-dst)*(1-src*a))
    /// - 5: HSV_MOD (legacy identifier for RGB Offset, not HSV modulation:
    ///   clamp(dst + (src - 0.5) * clamp(a, 0, 1) * 2, 0, 1), per RGB component)
    /// - 6: MIN (min(dst, mix(1, src, a)))
    /// - 7: OVERLAY (Photoshop-style overlay)
    /// All modes clamp the opcode output weight a and the resulting RGB to 0..1.
    ///
    /// Use this to set the current procedural EPU source for this frame without
    /// doing a fullscreen background draw.
    ///
    /// # Usage
    /// ```rust,ignore
    /// fn render() {
    ///     // Set environment configuration at the start of the pass/frame
    ///     epu_set(config.as_ptr());
    ///
    ///     // Draw scene geometry
    ///     draw_mesh(terrain);
    ///     draw_mesh(player);
    ///
    ///     // Draw environment background last (fills only background pixels)
    ///     draw_epu();
    /// }
    /// ```
    ///
    /// # Notes
    /// - The EPU compute pass runs automatically before rendering
    /// - To switch environments in a frame: call `epu_set(...)`, `epu_textures(...)`, or `epu_asset(...)`
    ///   before the draws that should use that source
    /// - Determinism: no host-managed time. Advance animation in deterministic update(),
    ///   store it in rollback-covered game state, and emit the resulting parameters in render().
    pub fn epu_set(config_ptr: *const u64);

    /// Set the current EPU source from six already-loaded cube face textures.
    ///
    /// Face order is `px, nx, py, ny, pz, nz`.
    pub fn epu_textures(px: u32, nx: u32, py: u32, ny: u32, pz: u32, nz: u32);

    /// Set the current EPU source from a packed ROM cubemap-face asset.
    pub fn epu_asset(id_ptr: *const u8, id_len: u32);

    /// Draw the environment background for the current viewport/pass.
    ///
    /// This draws a fullscreen background using the current EPU source selected by
    /// `epu_set(...)`, `epu_textures(...)`, or `epu_asset(...)`.
    ///
    /// For split-screen / multi-pass, set `viewport(...)` and call `draw_epu()`
    /// once per viewport/pass where you want an environment background.
    pub fn draw_epu();
}
