//! GPU Skinning

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a skeleton's inverse bind matrices to GPU.
    ///
    /// Call once during `init()` after loading skinned meshes.
    /// The inverse bind matrices transform vertices from model space
    /// to bone-local space at bind time.
    ///
    /// # Arguments
    /// * `inverse_bind_ptr` — Pointer to array of 3×4 matrices (12 floats per bone, column-major)
    /// * `bone_count` — Number of bones (max 256)
    ///
    /// # Returns
    /// Skeleton handle (>0) on success, 0 on error.
    pub fn load_skeleton(inverse_bind_ptr: *const f32, bone_count: u32) -> u32;

    /// Bind a skeleton for subsequent skinned mesh rendering.
    ///
    /// When bound, `set_bones()` expects model-space transforms and the GPU
    /// automatically applies the inverse bind matrices.
    ///
    /// # Arguments
    /// * `skeleton` — Skeleton handle from `load_skeleton()`, or 0 to unbind (raw mode)
    ///
    /// # Behavior
    /// - skeleton > 0: Enable inverse bind mode. `set_bones()` receives model transforms.
    /// - skeleton = 0: Disable inverse bind mode (raw). `set_bones()` receives final matrices.
    pub fn skeleton_bind(skeleton: u32);

    /// Set bone transform matrices for skeletal animation.
    ///
    /// # Arguments
    /// * `matrices_ptr` — Pointer to array of 3×4 matrices (12 floats per bone, column-major)
    /// * `count` — Number of bones (max 256)
    ///
    /// Each bone matrix is 12 floats in column-major order:
    /// ```text
    /// [col0.x, col0.y, col0.z]  // X axis
    /// [col1.x, col1.y, col1.z]  // Y axis
    /// [col2.x, col2.y, col2.z]  // Z axis
    /// [tx,     ty,     tz    ]  // translation
    /// // implicit 4th row [0, 0, 0, 1]
    /// ```
    pub fn set_bones(matrices_ptr: *const f32, count: u32);

    /// Set bone transform matrices for skeletal animation using 4x4 matrices.
    ///
    /// Alternative to `set_bones()` that accepts full 4x4 matrices instead of 3x4.
    ///
    /// # Arguments
    /// * `matrices_ptr` — Pointer to array of 4×4 matrices (16 floats per bone, column-major)
    /// * `count` — Number of bones (max 256)
    ///
    /// Each bone matrix is 16 floats in column-major order:
    /// ```text
    /// [col0.x, col0.y, col0.z, col0.w]  // X axis + w
    /// [col1.x, col1.y, col1.z, col1.w]  // Y axis + w
    /// [col2.x, col2.y, col2.z, col2.w]  // Z axis + w
    /// [tx,     ty,     tz,     tw    ]  // translation + w
    /// ```
    pub fn set_bones_4x4(matrices_ptr: *const f32, count: u32);
}
