//! Mesh Functions (Retained Mode)

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a non-indexed mesh.
    ///
    /// # Vertex format flags
    /// - 1 (FORMAT_UV): Has UV coordinates (2 floats)
    /// - 2 (FORMAT_COLOR): Has per-vertex color (3 floats RGB)
    /// - 4 (FORMAT_NORMAL): Has normals (3 floats)
    /// - 8 (FORMAT_SKINNED): Has bone indices/weights
    ///
    /// # Returns
    /// Mesh handle (>0) on success, 0 on failure.
    pub fn load_mesh(data_ptr: *const f32, vertex_count: u32, format: u32) -> u32;

    /// Load an indexed mesh.
    ///
    /// # Returns
    /// Mesh handle (>0) on success, 0 on failure.
    pub fn load_mesh_indexed(
        data_ptr: *const f32,
        vertex_count: u32,
        index_ptr: *const u16,
        index_count: u32,
        format: u32,
    ) -> u32;

    /// Load packed mesh data (power user API, f16/snorm16/unorm8 encoding).
    pub fn load_mesh_packed(data_ptr: *const u8, vertex_count: u32, format: u32) -> u32;

    /// Load indexed packed mesh data (power user API).
    pub fn load_mesh_indexed_packed(
        data_ptr: *const u8,
        vertex_count: u32,
        index_ptr: *const u16,
        index_count: u32,
        format: u32,
    ) -> u32;

    /// Draw a retained mesh with current transform and render state.
    pub fn draw_mesh(handle: u32);
}
