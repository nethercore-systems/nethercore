//! Keyframe Animation

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load keyframe animation data from WASM memory.
    ///
    /// Must be called during `init()`.
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to .nczxanim data in WASM memory
    /// * `byte_size` — Total size of the data in bytes
    ///
    /// # Returns
    /// Keyframe collection handle (>0) on success. Traps on failure.
    pub fn keyframes_load(data_ptr: *const u8, byte_size: u32) -> u32;

    /// Load keyframe animation data from ROM data pack by ID.
    ///
    /// Must be called during `init()`.
    ///
    /// # Arguments
    /// * `id_ptr` — Pointer to asset ID string in WASM memory
    /// * `id_len` — Length of asset ID string
    ///
    /// # Returns
    /// Keyframe collection handle (>0) on success. Traps on failure.
    pub fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32;

    /// Get the bone count for a keyframe collection.
    ///
    /// # Arguments
    /// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
    ///
    /// # Returns
    /// Bone count (0 on invalid handle)
    pub fn keyframes_bone_count(handle: u32) -> u32;

    /// Get the frame count for a keyframe collection.
    ///
    /// # Arguments
    /// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
    ///
    /// # Returns
    /// Frame count (0 on invalid handle)
    pub fn keyframes_frame_count(handle: u32) -> u32;

    /// Read a decoded keyframe into WASM memory.
    ///
    /// Decodes the platform format to BoneTransform format (40 bytes/bone):
    /// - rotation: [f32; 4] quaternion [x, y, z, w]
    /// - position: [f32; 3]
    /// - scale: [f32; 3]
    ///
    /// # Arguments
    /// * `handle` — Keyframe collection handle
    /// * `index` — Frame index (0-based)
    /// * `out_ptr` — Pointer to output buffer in WASM memory (must be bone_count × 40 bytes)
    ///
    /// # Traps
    /// - Invalid handle (0 or not loaded)
    /// - Frame index out of bounds
    /// - Output buffer out of bounds
    pub fn keyframe_read(handle: u32, index: u32, out_ptr: *mut u8);

    /// Bind a keyframe directly from the static GPU buffer.
    ///
    /// Points subsequent skinned draws to use pre-decoded matrices from the GPU buffer.
    /// No CPU decoding or data transfer needed at draw time.
    ///
    /// # Arguments
    /// * `handle` — Keyframe collection handle (0 to unbind)
    /// * `index` — Frame index (0-based)
    ///
    /// # Traps
    /// - Invalid handle (not loaded)
    /// - Frame index out of bounds
    pub fn keyframe_bind(handle: u32, index: u32);
}
