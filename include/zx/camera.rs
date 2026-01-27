//! Camera Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Set the camera position and target (look-at point).
    ///
    /// Uses a Y-up, right-handed coordinate system.
    pub fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);

    /// Set the camera field of view.
    ///
    /// # Arguments
    /// * `fov_degrees` — Field of view in degrees (typically 45-90, default 60)
    pub fn camera_fov(fov_degrees: f32);

    /// Push a custom view matrix (16 floats, column-major order).
    pub fn push_view_matrix(
        m0: f32,
        m1: f32,
        m2: f32,
        m3: f32,
        m4: f32,
        m5: f32,
        m6: f32,
        m7: f32,
        m8: f32,
        m9: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m15: f32,
    );

    /// Push a custom projection matrix (16 floats, column-major order).
    pub fn push_projection_matrix(
        m0: f32,
        m1: f32,
        m2: f32,
        m3: f32,
        m4: f32,
        m5: f32,
        m6: f32,
        m7: f32,
        m8: f32,
        m9: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m15: f32,
    );
}
