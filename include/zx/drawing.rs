//! Immediate Mode 3D Drawing & Billboards

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Draw triangles immediately (non-indexed).
    ///
    /// # Arguments
    /// * `vertex_count` — Must be multiple of 3
    /// * `format` — Vertex format flags (0-15)
    pub fn draw_triangles(data_ptr: *const f32, vertex_count: u32, format: u32);

    /// Draw indexed triangles immediately.
    ///
    /// # Arguments
    /// * `index_count` — Must be multiple of 3
    /// * `format` — Vertex format flags (0-15)
    pub fn draw_triangles_indexed(
        data_ptr: *const f32,
        vertex_count: u32,
        index_ptr: *const u16,
        index_count: u32,
        format: u32,
    );

    /// Draw a billboard (camera-facing quad) with full texture.
    ///
    /// Uses the color set by `set_color()`.
    ///
    /// # Arguments
    /// * `w`, `h` — Billboard size in world units
    /// * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
    pub fn draw_billboard(w: f32, h: f32, mode: u32);

    /// Draw a billboard with a UV region from the texture.
    ///
    /// Uses the color set by `set_color()`.
    ///
    /// # Arguments
    /// * `w`, `h` — Billboard size in world units
    /// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
    /// * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
    pub fn draw_billboard_region(
        w: f32,
        h: f32,
        src_x: f32,
        src_y: f32,
        src_w: f32,
        src_h: f32,
        mode: u32,
    );
}
