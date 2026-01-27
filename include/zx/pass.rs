//! Render Pass Functions (Execution Barriers & Depth/Stencil Control)
//!
//! Render passes provide execution barriers and depth/stencil state control.
//! Commands within a pass are batched and drawn together before the next pass.
//!
//! Use cases:
//! - FPS viewmodels: begin_pass(1) clears depth so gun renders on top of world
//! - Scope overlays: begin_pass_stencil_write() creates mask, begin_pass_stencil_test() renders inside
//! - Portals: Stencil masking for portal windows with depth clear for separate view

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Begin a new render pass with optional depth clear.
    ///
    /// Provides an execution barrier - commands in this pass complete before
    /// the next pass begins. Use for layered rendering like FPS viewmodels.
    ///
    /// # Arguments
    /// * `clear_depth` — Non-zero to clear depth buffer at pass start
    ///
    /// # Example (FPS viewmodel rendering)
    /// ```rust,ignore
    /// // Draw world first (pass 0)
    /// draw_mesh(world_mesh);
    /// epu_set(env_config_ptr);
    /// draw_epu();
    ///
    /// // Draw gun on top (pass 1 with depth clear)
    /// begin_pass(1);  // Clear depth so gun renders on top
    /// draw_mesh(gun_mesh);
    /// ```
    pub fn begin_pass(clear_depth: u32);

    /// Begin a stencil write pass (mask creation mode).
    ///
    /// After calling this, subsequent draw calls write to the stencil buffer
    /// but NOT to the color buffer. Use this to create a mask shape.
    /// Depth testing is disabled to prevent mask geometry from polluting depth.
    ///
    /// # Arguments
    /// * `ref_value` — Stencil reference value to write (typically 1)
    /// * `clear_depth` — Non-zero to clear depth buffer at pass start
    ///
    /// # Example (scope mask)
    /// ```rust,ignore
    /// begin_pass_stencil_write(1, 0);  // Start mask creation
    /// draw_mesh(circle_mesh);          // Draw circle to stencil only
    /// begin_pass_stencil_test(1, 0);   // Enable testing
    /// epu_set(env_config_ptr);
    /// draw_epu();                      // Only visible inside circle
    /// begin_pass(0);                    // Back to normal rendering
    /// ```
    pub fn begin_pass_stencil_write(ref_value: u32, clear_depth: u32);

    /// Begin a stencil test pass (render inside mask).
    ///
    /// After calling this, subsequent draw calls only render where
    /// the stencil buffer equals ref_value (inside the mask).
    ///
    /// # Arguments
    /// * `ref_value` — Stencil reference value to test against (must match write pass)
    /// * `clear_depth` — Non-zero to clear depth buffer at pass start
    pub fn begin_pass_stencil_test(ref_value: u32, clear_depth: u32);

    /// Begin a render pass with full control over depth and stencil state.
    ///
    /// This is the "escape hatch" for advanced effects not covered by the
    /// convenience functions. Most games should use begin_pass, begin_pass_stencil_write,
    /// or begin_pass_stencil_test instead.
    ///
    /// # Arguments
    /// * `depth_compare` — Depth comparison function (see compare::* constants)
    /// * `depth_write` — Non-zero to write to depth buffer
    /// * `clear_depth` — Non-zero to clear depth buffer at pass start
    /// * `stencil_compare` — Stencil comparison function (see compare::* constants)
    /// * `stencil_ref` — Stencil reference value (0-255)
    /// * `stencil_pass_op` — Operation when stencil test passes (see stencil_op::* constants)
    /// * `stencil_fail_op` — Operation when stencil test fails
    /// * `stencil_depth_fail_op` — Operation when depth test fails
    pub fn begin_pass_full(
        depth_compare: u32,
        depth_write: u32,
        clear_depth: u32,
        stencil_compare: u32,
        stencil_ref: u32,
        stencil_pass_op: u32,
        stencil_fail_op: u32,
        stencil_depth_fail_op: u32,
    );
}
