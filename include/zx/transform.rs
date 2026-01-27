//! Transform Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Push identity matrix onto the transform stack.
    pub fn push_identity();

    /// Set the current transform from a 4x4 matrix pointer (16 floats, column-major).
    pub fn transform_set(matrix_ptr: *const f32);

    /// Push a translation transform.
    pub fn push_translate(x: f32, y: f32, z: f32);

    /// Push a rotation around the X axis.
    ///
    /// # Arguments
    /// * `angle_deg` — Rotation angle in degrees
    pub fn push_rotate_x(angle_deg: f32);

    /// Push a rotation around the Y axis.
    ///
    /// # Arguments
    /// * `angle_deg` — Rotation angle in degrees
    pub fn push_rotate_y(angle_deg: f32);

    /// Push a rotation around the Z axis.
    ///
    /// # Arguments
    /// * `angle_deg` — Rotation angle in degrees
    pub fn push_rotate_z(angle_deg: f32);

    /// Push a rotation around an arbitrary axis.
    ///
    /// # Arguments
    /// * `angle_deg` — Rotation angle in degrees
    /// * `axis_x`, `axis_y`, `axis_z` — Rotation axis (will be normalized)
    pub fn push_rotate(angle_deg: f32, axis_x: f32, axis_y: f32, axis_z: f32);

    /// Push a non-uniform scale transform.
    pub fn push_scale(x: f32, y: f32, z: f32);

    /// Push a uniform scale transform.
    pub fn push_scale_uniform(s: f32);
}
