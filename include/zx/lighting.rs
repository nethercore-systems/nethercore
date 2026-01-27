//! Lighting Functions (Mode 2/3)

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Set light direction (and enable the light).
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `x`, `y`, `z` — Direction rays travel (from light toward surface)
    ///
    /// For a light from above, use (0, -1, 0).
    pub fn light_set(index: u32, x: f32, y: f32, z: f32);

    /// Set light color.
    ///
    /// # Arguments
    /// * `color` — Light color (0xRRGGBBAA, alpha ignored)
    pub fn light_color(index: u32, color: u32);

    /// Set light intensity multiplier.
    ///
    /// # Arguments
    /// * `intensity` — Typically 0.0-10.0
    pub fn light_intensity(index: u32, intensity: f32);

    /// Enable a light.
    pub fn light_enable(index: u32);

    /// Disable a light (preserves settings for re-enabling).
    pub fn light_disable(index: u32);

    /// Convert a light to a point light at world position.
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `x`, `y`, `z` — World-space position
    ///
    /// Enables the light automatically. Default range is 10.0 units.
    pub fn light_set_point(index: u32, x: f32, y: f32, z: f32);

    /// Set point light falloff distance.
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `range` — Distance at which light reaches zero intensity
    ///
    /// Only affects point lights (ignored for directional).
    pub fn light_range(index: u32, range: f32);
}
