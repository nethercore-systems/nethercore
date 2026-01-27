//! Environment Processing Unit (EPU) — Instruction-Based API

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Store an EPU configuration (128-byte) for the current `environment_index(...)`.
    ///
    /// Reads a 128-byte (8 x 128-bit = 16 x u64) environment configuration from WASM memory
    /// and stores it for the current render frame. The EPU compute pass runs automatically before
    /// rendering to build environment textures (EnvRadiance + SH9) for any referenced `env_id`.
    ///
    /// # Arguments
    /// * `config_ptr` — Pointer to 16 u64 values (128 bytes total) in WASM memory
    ///
    /// # Configuration Layout
    /// Each environment is exactly 8 x 128-bit instructions (each stored as [hi, lo]):
    /// - Slots 0-3: Enclosure/bounds layers (`0x01..0x07`)
    /// - Slots 4-7: Radiance/feature layers (`0x08..0x1F`)
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
    /// - 0x01: RAMP (enclosure gradient)
    /// - 0x02: SECTOR (enclosure modifier)
    /// - 0x03: SILHOUETTE (enclosure modifier)
    /// - 0x04: SPLIT (enclosure source)
    /// - 0x05: CELL (enclosure source)
    /// - 0x06: PATCHES (enclosure source)
    /// - 0x07: APERTURE (enclosure modifier)
    /// - 0x08: DECAL (sharp SDF shape)
    /// - 0x09: GRID (repeating lines/panels)
    /// - 0x0A: SCATTER (point field)
    /// - 0x0B: FLOW (animated noise/streaks)
    /// - 0x0C..0x13: radiance opcodes (TRACE/VEIL/ATMOSPHERE/PLANE/CELESTIAL/PORTAL/LOBE_RADIANCE/BAND_RADIANCE)
    ///
    /// # Blend Modes
    /// - 0: ADD (dst + src * a)
    /// - 1: MULTIPLY (dst * mix(1, src, a))
    /// - 2: MAX (max(dst, src * a))
    /// - 3: LERP (mix(dst, src, a))
    /// - 4: SCREEN (1 - (1-dst)*(1-src*a))
    /// - 5: HSV_MOD (HSV shift dst by src)
    /// - 6: MIN (min(dst, src * a))
    /// - 7: OVERLAY (Photoshop-style overlay)
    ///
    /// Store the environment configuration for the current `environment_index(...)`.
    ///
    /// Use this to set the active environment config for this frame without
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
    /// - To set up multiple environments in a frame: call `environment_index(env_id)`, then `epu_set(config_ptr)`
    /// - Determinism: the EPU has no host-managed time; animate by changing instruction parameters from the game
    pub fn epu_set(config_ptr: *const u64);

    /// Draw the environment background for the current viewport/pass.
    ///
    /// This draws a fullscreen background using the config selected by
    /// `environment_index(...)` (and previously provided via `epu_set(...)`).
    ///
    /// For split-screen / multi-pass, set `viewport(...)` and call `draw_epu()`
    /// once per viewport/pass where you want an environment background.
    pub fn draw_epu();
}
