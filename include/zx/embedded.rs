//! Embedded Asset API
//!
//! Load assets from NetherZ binary formats embedded via include_bytes!().
//! Use with: static DATA: &[u8] = include_bytes!("asset.nczxmesh");

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a mesh from .nczxmesh binary format.
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to .nczxmesh binary data
    /// * `data_len` — Length of the data in bytes
    ///
    /// # Returns
    /// Mesh handle (>0) on success, 0 on failure.
    pub fn load_zmesh(data_ptr: *const u8, data_len: u32) -> u32;

    /// Load a texture from .nczxtex binary format.
    ///
    /// # Returns
    /// Texture handle (>0) on success, 0 on failure.
    pub fn load_ztex(data_ptr: *const u8, data_len: u32) -> u32;

    /// Load a sound from .nczxsnd binary format.
    ///
    /// # Returns
    /// Sound handle (>0) on success, 0 on failure.
    pub fn load_zsound(data_ptr: *const u8, data_len: u32) -> u32;

    /// Load a skeleton from .nczxskel binary format.
    ///
    /// # Returns
    /// Skeleton handle (>0) on success, 0 on failure.
    pub fn load_zskeleton(data_ptr: *const u8, data_len: u32) -> u32;
}
