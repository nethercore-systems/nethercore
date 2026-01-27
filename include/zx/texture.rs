//! Texture Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a texture from RGBA pixel data.
    ///
    /// # Arguments
    /// * `width`, `height` — Texture dimensions
    /// * `pixels_ptr` — Pointer to RGBA8 pixel data (width × height × 4 bytes)
    ///
    /// # Returns
    /// Texture handle (>0) on success, 0 on failure.
    pub fn load_texture(width: u32, height: u32, pixels_ptr: *const u8) -> u32;

    /// Bind a texture to slot 0 (albedo).
    pub fn texture_bind(handle: u32);

    /// Bind a texture to a specific slot.
    ///
    /// # Arguments
    /// * `slot` — 0=albedo, 1=MRE/matcap, 2=reserved, 3=matcap
    pub fn texture_bind_slot(handle: u32, slot: u32);

    /// Set matcap blend mode for a texture slot (Mode 1 only).
    ///
    /// # Arguments
    /// * `slot` — Matcap slot (1-3)
    /// * `mode` — 0=Multiply, 1=Add, 2=HSV Modulate
    pub fn matcap_blend_mode(slot: u32, mode: u32);

    /// Bind a matcap texture to a slot (Mode 1 only).
    ///
    /// # Arguments
    /// * `slot` — Matcap slot (1-3)
    pub fn matcap_set(slot: u32, texture: u32);
}
