//! Render State Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Set the uniform tint color (multiplied with vertex colors and textures).
    ///
    /// # Arguments
    /// * `color` — Color in 0xRRGGBBAA format
    pub fn set_color(color: u32);

    /// Set the EPU environment index (`env_id`) used for subsequent draw calls.
    ///
    /// This selects which EPU environment textures are sampled for:
    /// - `draw_epu()` background rendering
    /// - Ambient lighting in lit render modes (0/2/3)
    /// - Reflections in lit render modes (1/2/3)
    ///
    /// Notes:
    /// - `env_id` is clamped to the supported range (0..255).
    /// - Default is 0.
    pub fn environment_index(env_id: u32);

    /// Set the face culling mode.
    ///
    /// # Arguments
    /// * `mode` — 0=none (default), 1=back, 2=front
    pub fn cull_mode(mode: u32);

    /// Set the texture filtering mode.
    ///
    /// # Arguments
    /// * `filter` — 0=nearest (pixelated), 1=linear (smooth)
    pub fn texture_filter(filter: u32);

    /// Set uniform alpha level for dither transparency.
    ///
    /// # Arguments
    /// * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15)
    ///
    /// Controls the dither pattern threshold for screen-door transparency.
    /// The dither pattern is always active, but with level=15 (default) all fragments pass.
    pub fn uniform_alpha(level: u32);

    /// Set dither offset for dither transparency.
    ///
    /// # Arguments
    /// * `x` — 0-3 pixel shift in X axis
    /// * `y` — 0-3 pixel shift in Y axis
    ///
    /// Use different offsets for stacked dithered meshes to prevent pattern cancellation.
    /// When two transparent objects overlap with the same alpha level and offset, their
    /// dither patterns align and pixels cancel out. Different offsets shift the pattern
    /// so both objects remain visible.
    pub fn dither_offset(x: u32, y: u32);

    /// Set z-index for 2D ordering control within a pass.
    ///
    /// # Arguments
    /// * `n` — Z-index value (0 = back, higher = front)
    ///
    /// Higher z-index values are drawn on top of lower values.
    /// Use this to ensure UI elements appear over game content
    /// regardless of texture bindings or draw order.
    ///
    /// Note: z_index only affects ordering within the same pass_id.
    /// Default: 0 (resets each frame)
    pub fn z_index(n: u32);
}
