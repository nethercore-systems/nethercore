//! Nethercore ZX FFI Bindings
//!
//! This file provides all FFI function declarations for Nethercore ZX games.
//! Import this module to access the complete Nethercore ZX API.
//!
//! # Usage
//!
//! ```rust,ignore
//! #![no_std]
//! #![no_main]
//!
//! // Include the FFI bindings
//! mod ffi;
//! use ffi::*;
//!
//! #[no_mangle]
//! pub extern "C" fn init() {
//!     set_clear_color(0x1a1a2eFF);
//!     render_mode(2); // PBR mode
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn update() {
//!     // Game logic here
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn render() {
//!     draw_sky();
//!     // Draw your scene
//! }
//! ```
//!
//! # Game Lifecycle
//!
//! All Nethercore games must export three functions:
//! - `init()` — Called once at startup
//! - `update()` — Called every tick (deterministic for rollback netcode)
//! - `render()` — Called every frame (skipped during rollback replay)

#![allow(unused)]

// =============================================================================
// EXTERN FUNCTION DECLARATIONS
// =============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // =========================================================================
    // System Functions
    // =========================================================================

    /// Returns the fixed timestep duration in seconds.
    ///
    /// This is a **constant value** based on the configured tick rate, NOT wall-clock time.
    /// - 60fps → 0.01666... (1/60)
    /// - 30fps → 0.03333... (1/30)
    ///
    /// Safe for rollback netcode: identical across all clients regardless of frame timing.
    pub fn delta_time() -> f32;

    /// Returns total elapsed game time since start in seconds.
    ///
    /// This is the **accumulated fixed timestep**, NOT wall-clock time.
    /// Calculated as `tick_count * delta_time`.
    ///
    /// Safe for rollback netcode: deterministic and identical across all clients.
    pub fn elapsed_time() -> f32;

    /// Returns the current tick number (starts at 0, increments by 1 each update).
    ///
    /// Perfectly deterministic: same inputs always produce the same tick count.
    /// Safe for rollback netcode.
    pub fn tick_count() -> u64;

    /// Logs a message to the console output.
    ///
    /// # Arguments
    /// * `ptr` — Pointer to UTF-8 string data
    /// * `len` — Length of string in bytes
    pub fn log(ptr: *const u8, len: u32);

    /// Exits the game and returns to the library.
    pub fn quit();

    // =========================================================================
    // Rollback Functions
    // =========================================================================

    /// Returns a deterministic random u32 from the host's seeded RNG.
    /// Always use this instead of external random sources for rollback compatibility.
    pub fn random() -> u32;

    /// Returns a random i32 in range [min, max).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_range(min: i32, max: i32) -> i32;

    /// Returns a random f32 in range [0.0, 1.0).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_f32() -> f32;

    /// Returns a random f32 in range [min, max).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_f32_range(min: f32, max: f32) -> f32;

    // =========================================================================
    // Session Functions
    // =========================================================================

    /// Returns the number of players in the session (1-4).
    pub fn player_count() -> u32;

    /// Returns a bitmask of which players are local to this client.
    ///
    /// Example: `(local_player_mask() & (1 << player_id)) != 0` checks if player is local.
    pub fn local_player_mask() -> u32;

    // =========================================================================
    // Save Data Functions
    // =========================================================================

    /// Saves data to a slot.
    ///
    /// # Arguments
    /// * `slot` — Save slot (0-7)
    /// * `data_ptr` — Pointer to data in WASM memory
    /// * `data_len` — Length of data in bytes (max 64KB)
    ///
    /// # Returns
    /// 0 on success, 1 if invalid slot, 2 if data too large.
    pub fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32;

    /// Loads data from a slot.
    ///
    /// # Arguments
    /// * `slot` — Save slot (0-7)
    /// * `data_ptr` — Pointer to buffer in WASM memory
    /// * `max_len` — Maximum bytes to read
    ///
    /// # Returns
    /// Bytes read (0 if empty or error).
    pub fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32;

    /// Deletes a save slot.
    ///
    /// # Returns
    /// 0 on success, 1 if invalid slot.
    pub fn delete(slot: u32) -> u32;

    // =========================================================================
    // Configuration Functions (init-only)
    // =========================================================================

    /// Set the tick rate. Must be called during `init()`.
    ///
    /// # Arguments
    /// * `rate` — Tick rate index: 0=24fps, 1=30fps, 2=60fps (default), 3=120fps
    pub fn set_tick_rate(rate: u32);

    /// Set the clear/background color. Must be called during `init()`.
    ///
    /// # Arguments
    /// * `color` — Color in 0xRRGGBBAA format (default: black)
    pub fn set_clear_color(color: u32);

    /// Set the render mode. Must be called during `init()`.
    ///
    /// # Arguments
    /// * `mode` — 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid
    pub fn render_mode(mode: u32);

    // =========================================================================
    // Camera Functions
    // =========================================================================

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
        m0: f32, m1: f32, m2: f32, m3: f32,
        m4: f32, m5: f32, m6: f32, m7: f32,
        m8: f32, m9: f32, m10: f32, m11: f32,
        m12: f32, m13: f32, m14: f32, m15: f32,
    );

    /// Push a custom projection matrix (16 floats, column-major order).
    pub fn push_projection_matrix(
        m0: f32, m1: f32, m2: f32, m3: f32,
        m4: f32, m5: f32, m6: f32, m7: f32,
        m8: f32, m9: f32, m10: f32, m11: f32,
        m12: f32, m13: f32, m14: f32, m15: f32,
    );

    // =========================================================================
    // Transform Functions
    // =========================================================================

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

    // =========================================================================
    // Input Functions — Buttons
    // =========================================================================

    /// Check if a button is currently held.
    ///
    /// # Button indices
    /// 0=UP, 1=DOWN, 2=LEFT, 3=RIGHT, 4=A, 5=B, 6=X, 7=Y,
    /// 8=L1, 9=R1, 10=L3, 11=R3, 12=START, 13=SELECT
    ///
    /// # Returns
    /// 1 if held, 0 otherwise.
    pub fn button_held(player: u32, button: u32) -> u32;

    /// Check if a button was just pressed this tick.
    ///
    /// # Returns
    /// 1 if just pressed, 0 otherwise.
    pub fn button_pressed(player: u32, button: u32) -> u32;

    /// Check if a button was just released this tick.
    ///
    /// # Returns
    /// 1 if just released, 0 otherwise.
    pub fn button_released(player: u32, button: u32) -> u32;

    /// Get bitmask of all held buttons.
    pub fn buttons_held(player: u32) -> u32;

    /// Get bitmask of all buttons just pressed this tick.
    pub fn buttons_pressed(player: u32) -> u32;

    /// Get bitmask of all buttons just released this tick.
    pub fn buttons_released(player: u32) -> u32;

    // =========================================================================
    // Input Functions — Analog Sticks
    // =========================================================================

    /// Get left stick X axis value (-1.0 to 1.0).
    pub fn left_stick_x(player: u32) -> f32;

    /// Get left stick Y axis value (-1.0 to 1.0).
    pub fn left_stick_y(player: u32) -> f32;

    /// Get right stick X axis value (-1.0 to 1.0).
    pub fn right_stick_x(player: u32) -> f32;

    /// Get right stick Y axis value (-1.0 to 1.0).
    pub fn right_stick_y(player: u32) -> f32;

    /// Get both left stick axes at once (more efficient).
    ///
    /// Writes X and Y values to the provided pointers.
    pub fn left_stick(player: u32, out_x: *mut f32, out_y: *mut f32);

    /// Get both right stick axes at once (more efficient).
    ///
    /// Writes X and Y values to the provided pointers.
    pub fn right_stick(player: u32, out_x: *mut f32, out_y: *mut f32);

    // =========================================================================
    // Input Functions — Triggers
    // =========================================================================

    /// Get left trigger value (0.0 to 1.0).
    pub fn trigger_left(player: u32) -> f32;

    /// Get right trigger value (0.0 to 1.0).
    pub fn trigger_right(player: u32) -> f32;

    // =========================================================================
    // Render State Functions
    // =========================================================================

    /// Set the uniform tint color (multiplied with vertex colors and textures).
    ///
    /// # Arguments
    /// * `color` — Color in 0xRRGGBBAA format
    pub fn set_color(color: u32);

    /// Enable or disable depth testing.
    ///
    /// # Arguments
    /// * `enabled` — 0 to disable, non-zero to enable (default: enabled)
    pub fn depth_test(enabled: u32);

    /// Set the face culling mode.
    ///
    /// # Arguments
    /// * `mode` — 0=none (default), 1=back, 2=front
    pub fn cull_mode(mode: u32);

    /// Set the texture filtering mode.
    ///
    /// # Arguments
    /// * `filter` — 0=nearest (pixelated), 1=linear (smooth)
    pub fn texture_filter(filter: u32);

    /// Set uniform alpha level for dither transparency.
    ///
    /// # Arguments
    /// * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15)
    ///
    /// Controls the dither pattern threshold for screen-door transparency.
    /// The dither pattern is always active, but with level=15 (default) all fragments pass.
    pub fn uniform_alpha(level: u32);

    /// Set dither offset for dither transparency.
    ///
    /// # Arguments
    /// * `x` — 0-3 pixel shift in X axis
    /// * `y` — 0-3 pixel shift in Y axis
    ///
    /// Use different offsets for stacked dithered meshes to prevent pattern cancellation.
    /// When two transparent objects overlap with the same alpha level and offset, their
    /// dither patterns align and pixels cancel out. Different offsets shift the pattern
    /// so both objects remain visible.
    pub fn dither_offset(x: u32, y: u32);

    /// Set draw layer for 2D ordering.
    ///
    /// # Arguments
    /// * `n` — Layer value (0 = back, higher = front)
    ///
    /// Higher layer values are drawn on top. Use this to ensure
    /// UI elements appear over game content regardless of texture bindings.
    /// Default: 0 (resets each frame)
    pub fn layer(n: u32);

    // =========================================================================
    // Viewport Functions (Split-Screen)
    // =========================================================================

    /// Set the viewport for subsequent draw calls.
    ///
    /// All 3D and 2D rendering will be clipped to this region.
    /// Camera aspect ratio automatically adjusts to viewport dimensions.
    /// 2D coordinates (draw_sprite, draw_text, etc.) become viewport-relative.
    ///
    /// # Arguments
    /// * `x` — Left edge in pixels (0-959)
    /// * `y` — Top edge in pixels (0-539)
    /// * `width` — Width in pixels (1-960)
    /// * `height` — Height in pixels (1-540)
    ///
    /// # Example (2-player horizontal split)
    /// ```rust,ignore
    /// // Player 1: left half
    /// viewport(0, 0, 480, 540);
    /// camera_set(p1_x, p1_y, p1_z, p1_tx, p1_ty, p1_tz);
    /// draw_env();
    /// draw_mesh(scene);
    ///
    /// // Player 2: right half
    /// viewport(480, 0, 480, 540);
    /// camera_set(p2_x, p2_y, p2_z, p2_tx, p2_ty, p2_tz);
    /// draw_env();
    /// draw_mesh(scene);
    ///
    /// // Reset for HUD
    /// viewport_clear();
    /// draw_text_str("PAUSED", 400.0, 270.0, 32.0, 0xFFFFFFFF);
    /// ```
    pub fn viewport(x: u32, y: u32, width: u32, height: u32);

    /// Reset viewport to fullscreen (960×540).
    ///
    /// Call this at the end of split-screen rendering to restore full-screen
    /// coordinates for HUD elements or between frames.
    pub fn viewport_clear();

    // =========================================================================
    // Stencil Functions (Masked Rendering)
    // =========================================================================

    /// Begin writing to the stencil buffer (mask creation mode).
    ///
    /// After calling this, subsequent draw calls will write to the stencil buffer
    /// but NOT to the color buffer. Use this to create a mask shape.
    ///
    /// # Example (circular scope mask)
    /// ```rust,ignore
    /// stencil_begin();           // Start mask creation
    /// draw_mesh(circle_mesh);    // Draw circle to stencil only
    /// stencil_end();             // Enable testing
    /// draw_env();                // Only visible inside circle
    /// draw_mesh(scene);          // Only visible inside circle
    /// stencil_clear();           // Back to normal rendering
    /// ```
    pub fn stencil_begin();

    /// End stencil mask creation and begin stencil testing.
    ///
    /// After calling this, subsequent draw calls will only render where
    /// the stencil buffer was written (inside the mask).
    ///
    /// Must be called after stencil_begin() has created a mask shape.
    pub fn stencil_end();

    /// Clear stencil state and return to normal rendering.
    ///
    /// Disables stencil operations. The stencil buffer itself is cleared
    /// at the start of each frame during render pass creation.
    ///
    /// Call this when finished with masked rendering to restore normal behavior.
    pub fn stencil_clear();

    /// Enable inverted stencil testing.
    ///
    /// After calling this, subsequent draw calls will only render where
    /// the stencil buffer was NOT written (outside the mask).
    ///
    /// Use this for effects like vignettes or rendering outside portals.
    ///
    /// # Example (vignette effect)
    /// ```rust,ignore
    /// stencil_begin();           // Start mask creation
    /// draw_mesh(rounded_rect);   // Draw center area to stencil
    /// stencil_invert();          // Render OUTSIDE the mask
    /// set_color(0x000000FF);     // Black vignette color
    /// draw_rect(0.0, 0.0, 960.0, 540.0, 0x000000FF);  // Fill outside
    /// stencil_clear();           // Back to normal
    /// ```
    pub fn stencil_invert();

    // =========================================================================
    // Texture Functions
    // =========================================================================

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

    // =========================================================================
    // Mesh Functions (Retained Mode)
    // =========================================================================

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

    // =========================================================================
    // Procedural Mesh Generation (init-only)
    // =========================================================================
    // All procedural mesh functions must be called during init().
    // They queue meshes for GPU upload which must happen before the game loop.

    /// Generate a cube mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `size_x`, `size_y`, `size_z` — Half-extents along each axis
    pub fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a UV sphere mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius` — Sphere radius
    /// * `segments` — Longitudinal divisions (3-256)
    /// * `rings` — Latitudinal divisions (2-256)
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a cylinder or cone mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius_bottom`, `radius_top` — Radii (>= 0.0, use 0 for cone tip)
    /// * `height` — Cylinder height
    /// * `segments` — Radial divisions (3-256)
    pub fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a plane mesh on the XZ plane. **Init-only.**
    ///
    /// # Arguments
    /// * `size_x`, `size_z` — Dimensions
    /// * `subdivisions_x`, `subdivisions_z` — Subdivisions (1-256)
    pub fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a torus (donut) mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `major_radius` — Distance from center to tube center
    /// * `minor_radius` — Tube radius
    /// * `major_segments`, `minor_segments` — Segment counts (3-256)
    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    /// Generate a capsule (pill shape) mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius` — Capsule radius
    /// * `height` — Height of cylindrical section (total = height + 2*radius)
    /// * `segments` — Radial divisions (3-256)
    /// * `rings` — Divisions per hemisphere (1-128)
    pub fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // UV-enabled variants (Format 5: POS_UV_NORMAL) — also init-only

    /// Generate a UV sphere mesh with equirectangular texture mapping. **Init-only.**
    pub fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a plane mesh with UV mapping. **Init-only.**
    pub fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a cube mesh with box-unwrapped UV mapping. **Init-only.**
    pub fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a cylinder mesh with cylindrical UV mapping. **Init-only.**
    pub fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a torus mesh with wrapped UV mapping. **Init-only.**
    pub fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    /// Generate a capsule mesh with hybrid UV mapping. **Init-only.**
    pub fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // =========================================================================
    // Immediate Mode 3D Drawing
    // =========================================================================

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

    // =========================================================================
    // Billboard Drawing
    // =========================================================================

    /// Draw a billboard (camera-facing quad) with full texture.
    ///
    /// # Arguments
    /// * `w`, `h` — Billboard size in world units
    /// * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
    /// * `color` — Color tint (0xRRGGBBAA)
    pub fn draw_billboard(w: f32, h: f32, mode: u32, color: u32);

    /// Draw a billboard with a UV region from the texture.
    ///
    /// # Arguments
    /// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
    pub fn draw_billboard_region(
        w: f32, h: f32,
        src_x: f32, src_y: f32, src_w: f32, src_h: f32,
        mode: u32, color: u32,
    );

    // =========================================================================
    // 2D Drawing (Screen Space)
    // =========================================================================

    /// Draw a sprite with the bound texture.
    ///
    /// # Arguments
    /// * `x`, `y` — Screen position in pixels (0,0 = top-left)
    /// * `w`, `h` — Sprite size in pixels
    /// * `color` — Color tint (0xRRGGBBAA)
    pub fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32);

    /// Draw a region of a sprite sheet.
    ///
    /// # Arguments
    /// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
    pub fn draw_sprite_region(
        x: f32, y: f32, w: f32, h: f32,
        src_x: f32, src_y: f32, src_w: f32, src_h: f32,
        color: u32,
    );

    /// Draw a sprite with full control (rotation, origin, UV region).
    ///
    /// # Arguments
    /// * `origin_x`, `origin_y` — Rotation pivot point (in pixels from sprite top-left)
    /// * `angle_deg` — Rotation angle in degrees (clockwise)
    pub fn draw_sprite_ex(
        x: f32, y: f32, w: f32, h: f32,
        src_x: f32, src_y: f32, src_w: f32, src_h: f32,
        origin_x: f32, origin_y: f32, angle_deg: f32,
        color: u32,
    );

    /// Draw a solid color rectangle.
    pub fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);

    /// Draw text with the current font.
    ///
    /// # Arguments
    /// * `ptr` — Pointer to UTF-8 string data
    /// * `len` — Length in bytes
    /// * `size` — Font size in pixels
    /// * `color` — Text color (0xRRGGBBAA)
    pub fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    /// Measure the width of text when rendered.
    ///
    /// # Arguments
    /// * `ptr` — Pointer to UTF-8 string data
    /// * `len` — Length in bytes
    /// * `size` — Font size in pixels
    ///
    /// # Returns
    /// Width in pixels that the text would occupy when rendered.
    pub fn text_width(ptr: *const u8, len: u32, size: f32) -> f32;

    /// Draw a line between two points.
    ///
    /// # Arguments
    /// * `x1`, `y1` — Start point in screen pixels
    /// * `x2`, `y2` — End point in screen pixels
    /// * `thickness` — Line thickness in pixels
    /// * `color` — Line color (0xRRGGBBAA)
    pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: u32);

    /// Draw a filled circle.
    ///
    /// # Arguments
    /// * `x`, `y` — Center position in screen pixels
    /// * `radius` — Circle radius in pixels
    /// * `color` — Fill color (0xRRGGBBAA)
    ///
    /// Rendered as a 16-segment triangle fan.
    pub fn draw_circle(x: f32, y: f32, radius: f32, color: u32);

    /// Draw a circle outline.
    ///
    /// # Arguments
    /// * `x`, `y` — Center position in screen pixels
    /// * `radius` — Circle radius in pixels
    /// * `thickness` — Line thickness in pixels
    /// * `color` — Outline color (0xRRGGBBAA)
    ///
    /// Rendered as 16 line segments.
    pub fn draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32, color: u32);

    /// Load a fixed-width bitmap font.
    ///
    /// # Arguments
    /// * `texture` — Texture atlas handle
    /// * `char_width`, `char_height` — Glyph dimensions in pixels
    /// * `first_codepoint` — Unicode codepoint of first glyph
    /// * `char_count` — Number of glyphs
    ///
    /// # Returns
    /// Font handle (use with `font_bind()`).
    pub fn load_font(
        texture: u32,
        char_width: u32,
        char_height: u32,
        first_codepoint: u32,
        char_count: u32,
    ) -> u32;

    /// Load a variable-width bitmap font.
    ///
    /// # Arguments
    /// * `widths_ptr` — Pointer to array of char_count u8 widths
    pub fn load_font_ex(
        texture: u32,
        widths_ptr: *const u8,
        char_height: u32,
        first_codepoint: u32,
        char_count: u32,
    ) -> u32;

    /// Bind a font for subsequent draw_text() calls.
    ///
    /// Pass 0 for the built-in 8×8 monospace font.
    pub fn font_bind(font_handle: u32);

    // =========================================================================
    // Environment Rendering
    // =========================================================================

    /// Render the configured environment. Call first in render(), before any geometry.
    pub fn draw_env();

    /// Bind a matcap texture to a slot (Mode 1 only).
    ///
    /// # Arguments
    /// * `slot` — Matcap slot (1-3)
    pub fn matcap_set(slot: u32, texture: u32);

    // =========================================================================
    // Environment Processing Unit (EPU) — Multi-Environment v3
    // =========================================================================

    /// Configure gradient environment (Mode 0).
    ///
    /// Creates a 4-color gradient background with vertical blending.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `zenith` — Color directly overhead (0xRRGGBBAA)
    /// * `sky_horizon` — Sky color at horizon level (0xRRGGBBAA)
    /// * `ground_horizon` — Ground color at horizon level (0xRRGGBBAA)
    /// * `nadir` — Color directly below (0xRRGGBBAA)
    /// * `rotation` — Rotation around Y axis in radians
    /// * `shift` — Horizon vertical shift (-1.0 to 1.0, 0.0 = equator)
    ///
    /// The gradient interpolates: zenith → sky_horizon (Y > 0), sky_horizon → ground_horizon (at Y = 0 + shift), ground_horizon → nadir (Y < 0).
    ///
    /// You can configure the same mode on both layers with different parameters for creative effects.
    pub fn env_gradient(layer: u32, zenith: u32, sky_horizon: u32, ground_horizon: u32, nadir: u32, rotation: f32, shift: f32);

    /// Configure scatter environment (Mode 1: stars, rain, warp).
    ///
    /// Creates a procedural particle field.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `variant` — 0=Stars, 1=Vertical (rain), 2=Horizontal, 3=Warp
    /// * `density` — Particle count (0-255)
    /// * `size` — Particle size (0-255)
    /// * `glow` — Glow/bloom intensity (0-255)
    /// * `streak_length` — Elongation for streaks (0-63, 0=points)
    /// * `color_primary` — Main particle color (0xRRGGBB00)
    /// * `color_secondary` — Variation/twinkle color (0xRRGGBB00)
    /// * `parallax_rate` — Layer separation amount (0-255)
    /// * `parallax_size` — Size variation with depth (0-255)
    /// * `phase` — Animation phase (0-65535, wraps for seamless looping)
    pub fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak_length: u32,
                       color_primary: u32, color_secondary: u32, parallax_rate: u32,
                       parallax_size: u32, phase: u32);

    /// Configure lines environment (Mode 2: synthwave grid, racing track).
    ///
    /// Creates an infinite procedural grid.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `variant` — 0=Floor, 1=Ceiling, 2=Sphere
    /// * `line_type` — 0=Horizontal, 1=Vertical, 2=Grid
    /// * `thickness` — Line thickness (0-255)
    /// * `spacing` — Distance between lines in world units
    /// * `fade_distance` — Distance where lines start fading in world units
    /// * `color_primary` — Main line color (0xRRGGBBAA)
    /// * `color_accent` — Accent line color (0xRRGGBBAA)
    /// * `accent_every` — Make every Nth line use accent color
    /// * `phase` — Scroll phase (0-65535, wraps for seamless looping)
    pub fn env_lines(layer: u32, variant: u32, line_type: u32, thickness: u32, spacing: f32, fade_distance: f32,
                     color_primary: u32, color_accent: u32, accent_every: u32, phase: u32);

    /// Configure silhouette environment (Mode 3: mountains, cityscape).
    ///
    /// Creates layered terrain silhouettes with procedural noise.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `jaggedness` — Terrain roughness (0-255, 0=smooth hills, 255=sharp peaks)
    /// * `layer_count` — Number of depth layers (1-3)
    /// * `color_near` — Nearest silhouette color (0xRRGGBBAA)
    /// * `color_far` — Farthest silhouette color (0xRRGGBBAA)
    /// * `sky_zenith` — Sky color at zenith behind silhouettes (0xRRGGBBAA)
    /// * `sky_horizon` — Sky color at horizon behind silhouettes (0xRRGGBBAA)
    /// * `parallax_rate` — Layer separation amount (0-255)
    /// * `seed` — Noise seed for terrain shape
    pub fn env_silhouette(layer: u32, jaggedness: u32, layer_count: u32, color_near: u32, color_far: u32,
                          sky_zenith: u32, sky_horizon: u32, parallax_rate: u32, seed: u32);

    /// Configure rectangles environment (Mode 4: city windows, control panels).
    ///
    /// Creates rectangular light sources like windows or screens.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `variant` — 0=Scatter, 1=Buildings, 2=Bands, 3=Panels
    /// * `density` — How many rectangles (0-255)
    /// * `lit_ratio` — Percentage of rectangles lit (0-255, 128=50%)
    /// * `size_min` — Minimum rectangle size (0-63)
    /// * `size_max` — Maximum rectangle size (0-63)
    /// * `aspect` — Aspect ratio bias (0-3, 0=square, 3=very tall)
    /// * `color_primary` — Main window/panel color (0xRRGGBBAA)
    /// * `color_variation` — Color variation for variety (0xRRGGBBAA)
    /// * `parallax_rate` — Layer separation (0-255)
    /// * `phase` — Flicker phase (0-65535, wraps for seamless animation)
    pub fn env_rectangles(layer: u32, variant: u32, density: u32, lit_ratio: u32, size_min: u32, size_max: u32,
                          aspect: u32, color_primary: u32, color_variation: u32, parallax_rate: u32, phase: u32);

    /// Configure room environment (Mode 5: interior spaces).
    ///
    /// Creates interior of a 3D box with directional lighting.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `color_ceiling` — Ceiling color (0xRRGGBB00)
    /// * `color_floor` — Floor color (0xRRGGBB00)
    /// * `color_walls` — Wall color (0xRRGGBB00)
    /// * `panel_size` — Size of wall panel pattern in world units
    /// * `panel_gap` — Gap between panels (0-255)
    /// * `light_dir_x`, `light_dir_y`, `light_dir_z` — Light direction
    /// * `light_intensity` — Directional light strength (0-255)
    /// * `corner_darken` — Corner/edge darkening amount (0-255)
    /// * `room_scale` — Room size multiplier
    /// * `viewer_x`, `viewer_y`, `viewer_z` — Viewer position in room (-128 to 127 = -1.0 to 1.0)
    pub fn env_room(layer: u32, color_ceiling: u32, color_floor: u32, color_walls: u32, panel_size: f32, panel_gap: u32,
                    light_dir_x: f32, light_dir_y: f32, light_dir_z: f32, light_intensity: u32,
                    corner_darken: u32, room_scale: f32, viewer_x: i32, viewer_y: i32, viewer_z: i32);

    /// Configure curtains environment (Mode 6: pillars, trees, vertical structures).
    ///
    /// Creates vertical structures arranged around the viewer.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `layer_count` — Depth layers (1-3)
    /// * `density` — Structures per cell (0-255)
    /// * `height_min` — Minimum height (0-63)
    /// * `height_max` — Maximum height (0-63)
    /// * `width` — Structure width (0-31)
    /// * `spacing` — Gap between structures (0-31)
    /// * `waviness` — Organic wobble (0-255, 0=straight)
    /// * `color_near` — Nearest structure color (0xRRGGBBAA)
    /// * `color_far` — Farthest structure color (0xRRGGBBAA)
    /// * `glow` — Neon/magical glow intensity (0-255)
    /// * `parallax_rate` — Layer separation (0-255)
    /// * `phase` — Horizontal scroll phase (0-65535, wraps for seamless)
    pub fn env_curtains(layer: u32, layer_count: u32, density: u32, height_min: u32, height_max: u32,
                        width: u32, spacing: u32, waviness: u32, color_near: u32, color_far: u32,
                        glow: u32, parallax_rate: u32, phase: u32);

    /// Configure rings environment (Mode 7: portals, tunnels, vortex).
    ///
    /// Creates concentric rings for portals or vortex effects.
    ///
    /// # Arguments
    /// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
    /// * `ring_count` — Number of rings (1-255)
    /// * `thickness` — Ring thickness (0-255)
    /// * `color_a` — First alternating color (0xRRGGBBAA)
    /// * `color_b` — Second alternating color (0xRRGGBBAA)
    /// * `center_color` — Bright center color (0xRRGGBBAA)
    /// * `center_falloff` — Center glow falloff (0-255)
    /// * `spiral_twist` — Spiral rotation in degrees (0=concentric)
    /// * `axis_x`, `axis_y`, `axis_z` — Ring axis direction (normalized)
    /// * `phase` — Rotation phase (0-65535 = 0°-360°, wraps for seamless)
    pub fn env_rings(layer: u32, ring_count: u32, thickness: u32, color_a: u32, color_b: u32,
                     center_color: u32, center_falloff: u32, spiral_twist: f32,
                     axis_x: f32, axis_y: f32, axis_z: f32, phase: u32);

    /// Set the blend mode for combining base and overlay layers.
    ///
    /// # Arguments
    /// * `mode` — Blend mode (0-3)
    ///
    /// # Blend Modes
    /// * 0 — Alpha: Standard alpha blending
    /// * 1 — Add: Additive blending
    /// * 2 — Multiply: Multiplicative blending
    /// * 3 — Screen: Screen blending
    ///
    /// Controls how the overlay layer composites onto the base layer.
    /// Use this to create different visual effects when layering environments.
    pub fn env_blend(mode: u32);

    // =========================================================================
    // Material Functions (Mode 2/3)
    // =========================================================================

    /// Bind an MRE texture (Metallic-Roughness-Emissive) to slot 1.
    pub fn material_mre(texture: u32);

    /// Bind an albedo texture to slot 0.
    pub fn material_albedo(texture: u32);

    /// Set material metallic value (0.0 = dielectric, 1.0 = metal).
    pub fn material_metallic(value: f32);

    /// Set material roughness value (0.0 = smooth, 1.0 = rough).
    pub fn material_roughness(value: f32);

    /// Set material emissive intensity (0.0 = no emission, >1.0 for HDR).
    pub fn material_emissive(value: f32);

    /// Set rim lighting parameters.
    ///
    /// # Arguments
    /// * `intensity` — Rim brightness (0.0-1.0)
    /// * `power` — Falloff sharpness (0.0-32.0, higher = tighter)
    pub fn material_rim(intensity: f32, power: f32);

    /// Set shininess (Mode 3 alias for roughness).
    pub fn material_shininess(value: f32);

    /// Set specular color (Mode 3 only).
    ///
    /// # Arguments
    /// * `color` — Specular color (0xRRGGBBAA, alpha ignored)
    pub fn material_specular(color: u32);

    // =========================================================================
    // Lighting Functions (Mode 2/3)
    // =========================================================================

    /// Set light direction (and enable the light).
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `x`, `y`, `z` — Direction rays travel (from light toward surface)
    ///
    /// For a light from above, use (0, -1, 0).
    pub fn light_set(index: u32, x: f32, y: f32, z: f32);

    /// Set light color.
    ///
    /// # Arguments
    /// * `color` — Light color (0xRRGGBBAA, alpha ignored)
    pub fn light_color(index: u32, color: u32);

    /// Set light intensity multiplier.
    ///
    /// # Arguments
    /// * `intensity` — Typically 0.0-10.0
    pub fn light_intensity(index: u32, intensity: f32);

    /// Enable a light.
    pub fn light_enable(index: u32);

    /// Disable a light (preserves settings for re-enabling).
    pub fn light_disable(index: u32);

    /// Convert a light to a point light at world position.
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `x`, `y`, `z` — World-space position
    ///
    /// Enables the light automatically. Default range is 10.0 units.
    pub fn light_set_point(index: u32, x: f32, y: f32, z: f32);

    /// Set point light falloff distance.
    ///
    /// # Arguments
    /// * `index` — Light index (0-3)
    /// * `range` — Distance at which light reaches zero intensity
    ///
    /// Only affects point lights (ignored for directional).
    pub fn light_range(index: u32, range: f32);

    // =========================================================================
    // GPU Skinning
    // =========================================================================

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

    // =========================================================================
    // Audio Functions
    // =========================================================================

    /// Load raw PCM sound data (22.05kHz, 16-bit signed, mono).
    ///
    /// Must be called during `init()`.
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to i16 PCM samples
    /// * `byte_len` — Length in bytes (must be even)
    ///
    /// # Returns
    /// Sound handle for use with playback functions.
    pub fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;

    /// Play sound on next available channel (fire-and-forget).
    ///
    /// # Arguments
    /// * `volume` — 0.0 to 1.0
    /// * `pan` — -1.0 (left) to 1.0 (right), 0.0 = center
    pub fn play_sound(sound: u32, volume: f32, pan: f32);

    /// Play sound on a specific channel (for managed/looping audio).
    ///
    /// # Arguments
    /// * `channel` — Channel index (0-15)
    /// * `looping` — 1 = loop, 0 = play once
    pub fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);

    /// Update channel parameters (call every frame for positional audio).
    pub fn channel_set(channel: u32, volume: f32, pan: f32);

    /// Stop a channel.
    pub fn channel_stop(channel: u32);

    // =========================================================================
    // Unified Music API (PCM + Tracker)
    // =========================================================================
    //
    // A single API for both PCM music and XM tracker modules.
    // The handle type is detected automatically:
    // - PCM sound handles (from load_sound/rom_sound) have bit 31 = 0
    // - Tracker handles (from load_tracker/rom_tracker) have bit 31 = 1
    //
    // Starting one type automatically stops the other (mutually exclusive).
    // Supports rollback netcode: state is snapshotted and restored.

    /// Load a tracker module from ROM data pack by ID.
    ///
    /// Must be called during `init()`.
    /// Returns a handle with bit 31 set (tracker handle).
    ///
    /// # Arguments
    /// * `id_ptr` — Pointer to tracker ID string
    /// * `id_len` — Length of tracker ID string
    ///
    /// # Returns
    /// Tracker handle (>0) on success, 0 on failure.
    pub fn rom_tracker(id_ptr: u32, id_len: u32) -> u32;

    /// Load a tracker module from raw XM data.
    ///
    /// Must be called during `init()`.
    /// Returns a handle with bit 31 set (tracker handle).
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to XM file data
    /// * `data_len` — Length of XM data in bytes
    ///
    /// # Returns
    /// Tracker handle (>0) on success, 0 on failure.
    pub fn load_tracker(data_ptr: u32, data_len: u32) -> u32;

    /// Play music (PCM sound or tracker module).
    ///
    /// Automatically stops any currently playing music of the other type.
    /// Handle type is detected by bit 31 (0=PCM, 1=tracker).
    ///
    /// # Arguments
    /// * `handle` — Sound handle (from load_sound) or tracker handle (from rom_tracker)
    /// * `volume` — 0.0 to 1.0
    /// * `looping` — 1 = loop, 0 = play once
    pub fn music_play(handle: u32, volume: f32, looping: u32);

    /// Stop music (both PCM and tracker).
    pub fn music_stop();

    /// Pause or resume music (tracker only, no-op for PCM).
    ///
    /// # Arguments
    /// * `paused` — 1 = pause, 0 = resume
    pub fn music_pause(paused: u32);

    /// Set music volume (works for both PCM and tracker).
    ///
    /// # Arguments
    /// * `volume` — 0.0 to 1.0
    pub fn music_set_volume(volume: f32);

    /// Check if music is currently playing.
    ///
    /// # Returns
    /// 1 if playing (and not paused), 0 otherwise.
    pub fn music_is_playing() -> u32;

    /// Get current music type.
    ///
    /// # Returns
    /// 0 = none, 1 = PCM, 2 = tracker
    pub fn music_type() -> u32;

    /// Jump to a specific position (tracker only, no-op for PCM).
    ///
    /// Use for dynamic music systems (e.g., jump to outro pattern).
    ///
    /// # Arguments
    /// * `order` — Order position (0-based)
    /// * `row` — Row within the pattern (0-based)
    pub fn music_jump(order: u32, row: u32);

    /// Get current music position.
    ///
    /// For tracker: (order << 16) | row
    /// For PCM: sample position
    ///
    /// # Returns
    /// Position value (format depends on music type).
    pub fn music_position() -> u32;

    /// Get music length.
    ///
    /// For tracker: number of orders in the song.
    /// For PCM: number of samples.
    ///
    /// # Arguments
    /// * `handle` — Music handle (PCM or tracker)
    ///
    /// # Returns
    /// Length value.
    pub fn music_length(handle: u32) -> u32;

    /// Set music speed (tracker only, ticks per row).
    ///
    /// # Arguments
    /// * `speed` — 1-31 (XM default is 6)
    pub fn music_set_speed(speed: u32);

    /// Set music tempo (tracker only, BPM).
    ///
    /// # Arguments
    /// * `bpm` — 32-255 (XM default is 125)
    pub fn music_set_tempo(bpm: u32);

    /// Get music info.
    ///
    /// For tracker: (num_channels << 24) | (num_patterns << 16) | (num_instruments << 8) | song_length
    /// For PCM: (sample_rate << 16) | (channels << 8) | bits_per_sample
    ///
    /// # Arguments
    /// * `handle` — Music handle (PCM or tracker)
    ///
    /// # Returns
    /// Packed info value.
    pub fn music_info(handle: u32) -> u32;

    /// Get music name (tracker only, returns 0 for PCM).
    ///
    /// # Arguments
    /// * `handle` — Music handle
    /// * `out_ptr` — Pointer to output buffer
    /// * `max_len` — Maximum bytes to write
    ///
    /// # Returns
    /// Actual length written (0 if PCM or invalid handle).
    pub fn music_name(handle: u32, out_ptr: *mut u8, max_len: u32) -> u32;

    // =========================================================================
    // ROM Data Pack API (init-only)
    // =========================================================================
    //
    // Load assets from the bundled ROM data pack by string ID.
    // Assets go directly to VRAM/audio memory, bypassing WASM linear memory.
    // All `rom_*` functions can only be called during `init()`.

    /// Load a texture from ROM data pack by ID.
    ///
    /// # Arguments
    /// * `id_ptr` — Pointer to asset ID string in WASM memory
    /// * `id_len` — Length of asset ID string
    ///
    /// # Returns
    /// Texture handle (>0) on success. Traps on failure.
    pub fn rom_texture(id_ptr: u32, id_len: u32) -> u32;

    /// Load a mesh from ROM data pack by ID.
    ///
    /// # Returns
    /// Mesh handle (>0) on success. Traps on failure.
    pub fn rom_mesh(id_ptr: u32, id_len: u32) -> u32;

    /// Load skeleton inverse bind matrices from ROM data pack by ID.
    ///
    /// # Returns
    /// Skeleton handle (>0) on success. Traps on failure.
    pub fn rom_skeleton(id_ptr: u32, id_len: u32) -> u32;

    /// Load a font atlas from ROM data pack by ID.
    ///
    /// # Returns
    /// Texture handle for font atlas (>0) on success. Traps on failure.
    pub fn rom_font(id_ptr: u32, id_len: u32) -> u32;

    /// Load a sound from ROM data pack by ID.
    ///
    /// # Returns
    /// Sound handle (>0) on success. Traps on failure.
    pub fn rom_sound(id_ptr: u32, id_len: u32) -> u32;

    /// Get the byte size of raw data in the ROM data pack.
    ///
    /// Use this to allocate a buffer before calling `rom_data()`.
    ///
    /// # Returns
    /// Byte count on success. Traps if not found.
    pub fn rom_data_len(id_ptr: u32, id_len: u32) -> u32;

    /// Copy raw data from ROM data pack into WASM linear memory.
    ///
    /// # Arguments
    /// * `id_ptr`, `id_len` — Asset ID string
    /// * `dst_ptr` — Pointer to destination buffer in WASM memory
    /// * `max_len` — Maximum bytes to copy (size of destination buffer)
    ///
    /// # Returns
    /// Bytes written on success. Traps on failure.
    pub fn rom_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) -> u32;

    // =========================================================================
    // Embedded Asset API
    // =========================================================================
    //
    // Load assets from NetherZ binary formats embedded via include_bytes!().
    // Use with: static DATA: &[u8] = include_bytes!("asset.nczxmesh");

    /// Load a mesh from .nczxmesh binary format.
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to .nczxmesh binary data
    /// * `data_len` — Length of the data in bytes
    ///
    /// # Returns
    /// Mesh handle (>0) on success, 0 on failure.
    pub fn load_zmesh(data_ptr: u32, data_len: u32) -> u32;

    /// Load a texture from .nczxtex binary format.
    ///
    /// # Returns
    /// Texture handle (>0) on success, 0 on failure.
    pub fn load_ztex(data_ptr: u32, data_len: u32) -> u32;

    /// Load a sound from .nczxsnd binary format.
    ///
    /// # Returns
    /// Sound handle (>0) on success, 0 on failure.
    pub fn load_zsound(data_ptr: u32, data_len: u32) -> u32;

    // =========================================================================
    // Debug Inspection System
    // =========================================================================
    //
    // Runtime value inspection and editing for development.
    // Press F3 to open panel. Zero overhead in release builds (compiles out).
    // All debug functions use length-prefixed strings: (name_ptr, name_len).

    // --- Primitive Type Registration (Editable) ---

    /// Register an i8 value for debug inspection.
    pub fn debug_register_i8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register an i16 value for debug inspection.
    pub fn debug_register_i16(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register an i32 value for debug inspection.
    pub fn debug_register_i32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a u8 value for debug inspection.
    pub fn debug_register_u8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a u16 value for debug inspection.
    pub fn debug_register_u16(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a u32 value for debug inspection.
    pub fn debug_register_u32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register an f32 value for debug inspection.
    pub fn debug_register_f32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a bool value for debug inspection.
    pub fn debug_register_bool(name_ptr: u32, name_len: u32, ptr: u32);

    // --- Range-Constrained Registration (Slider UI) ---

    /// Register an i32 with min/max range constraints.
    pub fn debug_register_i32_range(name_ptr: u32, name_len: u32, ptr: u32, min: i32, max: i32);
    /// Register an f32 with min/max range constraints.
    pub fn debug_register_f32_range(name_ptr: u32, name_len: u32, ptr: u32, min: f32, max: f32);
    /// Register a u8 with min/max range constraints.
    pub fn debug_register_u8_range(name_ptr: u32, name_len: u32, ptr: u32, min: u32, max: u32);
    /// Register a u16 with min/max range constraints.
    pub fn debug_register_u16_range(name_ptr: u32, name_len: u32, ptr: u32, min: u32, max: u32);
    /// Register an i16 with min/max range constraints.
    pub fn debug_register_i16_range(name_ptr: u32, name_len: u32, ptr: u32, min: i32, max: i32);

    // --- Compound Type Registration (Editable) ---

    /// Register a Vec2 (2 floats: x, y) for debug inspection.
    pub fn debug_register_vec2(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a Vec3 (3 floats: x, y, z) for debug inspection.
    pub fn debug_register_vec3(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a Rect (4 i16: x, y, w, h) for debug inspection.
    pub fn debug_register_rect(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register a Color (4 u8: RGBA) for debug inspection with color picker.
    pub fn debug_register_color(name_ptr: u32, name_len: u32, ptr: u32);

    // --- Fixed-Point Type Registration (Editable) ---

    /// Register Q8.8 fixed-point (i16) for debug inspection.
    pub fn debug_register_fixed_i16_q8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register Q16.16 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q16(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register Q24.8 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Register Q8.24 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q24(name_ptr: u32, name_len: u32, ptr: u32);

    // --- Watch Functions (Read-Only Display) ---

    /// Watch an i8 value (read-only).
    pub fn debug_watch_i8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch an i16 value (read-only).
    pub fn debug_watch_i16(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch an i32 value (read-only).
    pub fn debug_watch_i32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a u8 value (read-only).
    pub fn debug_watch_u8(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a u16 value (read-only).
    pub fn debug_watch_u16(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a u32 value (read-only).
    pub fn debug_watch_u32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch an f32 value (read-only).
    pub fn debug_watch_f32(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a bool value (read-only).
    pub fn debug_watch_bool(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a Vec2 value (read-only).
    pub fn debug_watch_vec2(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a Vec3 value (read-only).
    pub fn debug_watch_vec3(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a Rect value (read-only).
    pub fn debug_watch_rect(name_ptr: u32, name_len: u32, ptr: u32);
    /// Watch a Color value (read-only).
    pub fn debug_watch_color(name_ptr: u32, name_len: u32, ptr: u32);

    // --- Grouping Functions ---

    /// Begin a collapsible group in the debug UI.
    pub fn debug_group_begin(name_ptr: u32, name_len: u32);
    /// End the current debug group.
    pub fn debug_group_end();

    // --- State Query Functions ---

    /// Query if the game is currently paused (debug mode).
    ///
    /// # Returns
    /// 1 if paused, 0 if running normally.
    pub fn debug_is_paused() -> i32;

    /// Get the current time scale multiplier.
    ///
    /// # Returns
    /// 1.0 = normal, 0.5 = half-speed, 2.0 = double-speed, etc.
    pub fn debug_get_time_scale() -> f32;
}

// =============================================================================
// CONSTANTS
// =============================================================================

/// Button indices for input functions
pub mod button {
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const LEFT: u32 = 2;
    pub const RIGHT: u32 = 3;
    pub const A: u32 = 4;
    pub const B: u32 = 5;
    pub const X: u32 = 6;
    pub const Y: u32 = 7;
    pub const L1: u32 = 8;
    pub const R1: u32 = 9;
    pub const L3: u32 = 10;
    pub const R3: u32 = 11;
    pub const START: u32 = 12;
    pub const SELECT: u32 = 13;
}

/// Render modes for `render_mode()`
pub mod render {
    pub const LAMBERT: u32 = 0;
    pub const MATCAP: u32 = 1;
    pub const PBR: u32 = 2;
    pub const HYBRID: u32 = 3;
}

/// Cull modes for `cull_mode()`
pub mod cull {
    pub const NONE: u32 = 0;
    pub const BACK: u32 = 1;
    pub const FRONT: u32 = 2;
}

/// Vertex format flags for mesh loading
pub mod format {
    pub const POS: u8 = 0;
    pub const UV: u8 = 1;
    pub const COLOR: u8 = 2;
    pub const NORMAL: u8 = 4;
    pub const SKINNED: u8 = 8;

    // Common combinations
    pub const POS_UV: u8 = UV;
    pub const POS_COLOR: u8 = COLOR;
    pub const POS_NORMAL: u8 = NORMAL;
    pub const POS_UV_NORMAL: u8 = UV | NORMAL;
    pub const POS_UV_COLOR: u8 = UV | COLOR;
    pub const POS_UV_COLOR_NORMAL: u8 = UV | COLOR | NORMAL;
    pub const POS_SKINNED: u8 = SKINNED;
    pub const POS_NORMAL_SKINNED: u8 = NORMAL | SKINNED;
    pub const POS_UV_NORMAL_SKINNED: u8 = UV | NORMAL | SKINNED;
}

/// Billboard modes for `draw_billboard()`
pub mod billboard {
    pub const SPHERICAL: u32 = 1;
    pub const CYLINDRICAL_Y: u32 = 2;
    pub const CYLINDRICAL_X: u32 = 3;
    pub const CYLINDRICAL_Z: u32 = 4;
}

/// Tick rate indices for `set_tick_rate()`
pub mod tick_rate {
    pub const FPS_24: u32 = 0;
    pub const FPS_30: u32 = 1;
    pub const FPS_60: u32 = 2;
    pub const FPS_120: u32 = 3;
}

/// Screen dimensions (fixed 540p resolution)
pub mod screen {
    /// Screen width in pixels
    pub const WIDTH: u32 = 960;
    /// Screen height in pixels
    pub const HEIGHT: u32 = 540;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Helper to log a string slice.
///
/// # Example
/// ```rust,ignore
/// log_str("Player spawned");
/// ```
#[inline]
pub fn log_str(s: &str) {
    unsafe {
        log(s.as_ptr(), s.len() as u32);
    }
}

/// Helper to draw a text string slice.
#[inline]
pub fn draw_text_str(s: &str, x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size, color);
    }
}

/// Pack RGBA color components into a u32.
///
/// # Example
/// ```rust,ignore
/// let red = rgba(255, 0, 0, 255); // 0xFF0000FF
/// ```
#[inline]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Pack RGB color components into a u32 (alpha = 255).
#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    rgba(r, g, b, 255)
}

// =============================================================================
// COMMON COLORS
// =============================================================================

pub mod color {
    use super::rgba;

    pub const WHITE: u32 = 0xFFFFFFFF;
    pub const BLACK: u32 = 0x000000FF;
    pub const RED: u32 = 0xFF0000FF;
    pub const GREEN: u32 = 0x00FF00FF;
    pub const BLUE: u32 = 0x0000FFFF;
    pub const YELLOW: u32 = 0xFFFF00FF;
    pub const CYAN: u32 = 0x00FFFFFF;
    pub const MAGENTA: u32 = 0xFF00FFFF;
    pub const ORANGE: u32 = 0xFF8000FF;
    pub const TRANSPARENT: u32 = 0x00000000;
}

// =============================================================================
// ROM HELPERS
// =============================================================================

/// Helper to load a ROM texture by string literal.
///
/// # Example
/// ```rust,ignore
/// let tex = rom_texture_str("player");
/// ```
#[inline]
pub fn rom_texture_str(id: &str) -> u32 {
    unsafe { rom_texture(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to load a ROM mesh by string literal.
#[inline]
pub fn rom_mesh_str(id: &str) -> u32 {
    unsafe { rom_mesh(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to load a ROM sound by string literal.
#[inline]
pub fn rom_sound_str(id: &str) -> u32 {
    unsafe { rom_sound(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to load a ROM font by string literal.
#[inline]
pub fn rom_font_str(id: &str) -> u32 {
    unsafe { rom_font(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to load a ROM skeleton by string literal.
#[inline]
pub fn rom_skeleton_str(id: &str) -> u32 {
    unsafe { rom_skeleton(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to load a ROM tracker by string literal.
#[inline]
pub fn rom_tracker_str(id: &str) -> u32 {
    unsafe { rom_tracker(id.as_ptr() as u32, id.len() as u32) }
}

/// Helper to get ROM data length by string literal.
#[inline]
pub fn rom_data_len_str(id: &str) -> u32 {
    unsafe { rom_data_len(id.as_ptr() as u32, id.len() as u32) }
}

// =============================================================================
// DEBUG HELPERS
// =============================================================================

/// Helper to register an f32 debug value by string literal.
///
/// # Example
/// ```rust,ignore
/// static mut SPEED: f32 = 5.0;
/// debug_f32("speed", &SPEED);
/// ```
#[inline]
pub unsafe fn debug_f32(name: &str, ptr: &f32) {
    debug_register_f32(name.as_ptr() as u32, name.len() as u32, ptr as *const f32 as u32);
}

/// Helper to register an i32 debug value by string literal.
#[inline]
pub unsafe fn debug_i32(name: &str, ptr: &i32) {
    debug_register_i32(name.as_ptr() as u32, name.len() as u32, ptr as *const i32 as u32);
}

/// Helper to register a bool debug value by string literal.
#[inline]
pub unsafe fn debug_bool(name: &str, ptr: &bool) {
    debug_register_bool(name.as_ptr() as u32, name.len() as u32, ptr as *const bool as u32);
}

/// Helper to begin a debug group by string literal.
#[inline]
pub fn debug_group(name: &str) {
    unsafe { debug_group_begin(name.as_ptr() as u32, name.len() as u32); }
}

/// Helper to end the current debug group.
#[inline]
pub fn debug_group_close() {
    unsafe { debug_group_end(); }
}

// =============================================================================
// LEGACY BUTTON CONSTANTS (for backwards compatibility)
// =============================================================================
// Prefer using button::A, button::START, etc. from the button module above.

pub const BUTTON_UP: u32 = button::UP;
pub const BUTTON_DOWN: u32 = button::DOWN;
pub const BUTTON_LEFT: u32 = button::LEFT;
pub const BUTTON_RIGHT: u32 = button::RIGHT;
pub const BUTTON_A: u32 = button::A;
pub const BUTTON_B: u32 = button::B;
pub const BUTTON_X: u32 = button::X;
pub const BUTTON_Y: u32 = button::Y;
pub const BUTTON_L1: u32 = button::L1;
pub const BUTTON_R1: u32 = button::R1;
pub const BUTTON_L3: u32 = button::L3;
pub const BUTTON_R3: u32 = button::R3;
pub const BUTTON_START: u32 = button::START;
pub const BUTTON_SELECT: u32 = button::SELECT;
