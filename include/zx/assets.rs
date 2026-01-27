//! ROM Data Pack API (init-only)
//!
//! Load assets from the bundled ROM data pack by string ID.
//! Assets go directly to VRAM/audio memory, bypassing WASM linear memory.
//! All `rom_*` functions can only be called during `init()`.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a texture from ROM data pack by ID.
    ///
    /// # Arguments
    /// * `id_ptr` — Pointer to asset ID string in WASM memory
    /// * `id_len` — Length of asset ID string
    ///
    /// # Returns
    /// Texture handle (>0) on success. Traps on failure.
    pub fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;

    /// Load a mesh from ROM data pack by ID.
    ///
    /// # Returns
    /// Mesh handle (>0) on success. Traps on failure.
    pub fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;

    /// Load skeleton inverse bind matrices from ROM data pack by ID.
    ///
    /// # Returns
    /// Skeleton handle (>0) on success. Traps on failure.
    pub fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32;

    /// Load a font atlas from ROM data pack by ID.
    ///
    /// # Returns
    /// Texture handle for font atlas (>0) on success. Traps on failure.
    pub fn rom_font(id_ptr: *const u8, id_len: u32) -> u32;

    /// Load a sound from ROM data pack by ID.
    ///
    /// # Returns
    /// Sound handle (>0) on success. Traps on failure.
    pub fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    /// Get the byte size of raw data in the ROM data pack.
    ///
    /// Use this to allocate a buffer before calling `rom_data()`.
    ///
    /// # Returns
    /// Byte count on success. Traps if not found.
    pub fn rom_data_len(id_ptr: *const u8, id_len: u32) -> u32;

    /// Copy raw data from ROM data pack into WASM linear memory.
    ///
    /// # Arguments
    /// * `id_ptr`, `id_len` — Asset ID string
    /// * `dst_ptr` — Pointer to destination buffer in WASM memory
    /// * `max_len` — Maximum bytes to copy (size of destination buffer)
    ///
    /// # Returns
    /// Bytes written on success. Traps on failure.
    pub fn rom_data(id_ptr: *const u8, id_len: u32, dst_ptr: *const u8, max_len: u32) -> u32;
}
