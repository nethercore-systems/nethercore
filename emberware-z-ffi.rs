//! Emberware Z FFI Bindings
//!
//! This file provides all FFI function declarations for Emberware Z games.
//! Import this module to access the complete Emberware Z API.
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
//! All Emberware games must export three functions:
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

    /// Set the render resolution. Must be called during `init()`.
    ///
    /// # Arguments
    /// * `res` — Resolution index: 0=360p, 1=540p (default), 2=720p, 3=1080p
    pub fn set_resolution(res: u32);

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
    /// * `mode` — 0=Unlit, 1=Matcap, 2=PBR, 3=Hybrid
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
    /// * `mode` — 0=none, 1=back (default), 2=front
    pub fn cull_mode(mode: u32);

    /// Set the blend mode.
    ///
    /// # Arguments
    /// * `mode` — 0=none (opaque), 1=alpha, 2=additive, 3=multiply
    pub fn blend_mode(mode: u32);

    /// Set the texture filtering mode.
    ///
    /// # Arguments
    /// * `filter` — 0=nearest (pixelated), 1=linear (smooth)
    pub fn texture_filter(filter: u32);

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
    // Procedural Mesh Generation
    // =========================================================================

    /// Generate a cube mesh.
    ///
    /// # Arguments
    /// * `size_x`, `size_y`, `size_z` — Half-extents along each axis
    pub fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a UV sphere mesh.
    ///
    /// # Arguments
    /// * `radius` — Sphere radius
    /// * `segments` — Longitudinal divisions (3-256)
    /// * `rings` — Latitudinal divisions (2-256)
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a cylinder or cone mesh.
    ///
    /// # Arguments
    /// * `radius_bottom`, `radius_top` — Radii (>= 0.0, use 0 for cone tip)
    /// * `height` — Cylinder height
    /// * `segments` — Radial divisions (3-256)
    pub fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a plane mesh on the XZ plane.
    ///
    /// # Arguments
    /// * `size_x`, `size_z` — Dimensions
    /// * `subdivisions_x`, `subdivisions_z` — Subdivisions (1-256)
    pub fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a torus (donut) mesh.
    ///
    /// # Arguments
    /// * `major_radius` — Distance from center to tube center
    /// * `minor_radius` — Tube radius
    /// * `major_segments`, `minor_segments` — Segment counts (3-256)
    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    /// Generate a capsule (pill shape) mesh.
    ///
    /// # Arguments
    /// * `radius` — Capsule radius
    /// * `height` — Height of cylindrical section (total = height + 2*radius)
    /// * `segments` — Radial divisions (3-256)
    /// * `rings` — Divisions per hemisphere (1-128)
    pub fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // UV-enabled variants (Format 5: POS_UV_NORMAL)

    /// Generate a UV sphere mesh with equirectangular texture mapping.
    pub fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a plane mesh with UV mapping.
    pub fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a cube mesh with box-unwrapped UV mapping.
    pub fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a cylinder mesh with cylindrical UV mapping.
    pub fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a torus mesh with wrapped UV mapping.
    pub fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    /// Generate a capsule mesh with hybrid UV mapping.
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
    // Sky System
    // =========================================================================

    /// Set sky gradient colors.
    ///
    /// # Arguments
    /// * `horizon_color` — Color at eye level (0xRRGGBBAA)
    /// * `zenith_color` — Color directly overhead (0xRRGGBBAA)
    pub fn sky_set_colors(horizon_color: u32, zenith_color: u32);

    /// Set sky sun properties.
    ///
    /// # Arguments
    /// * `dir_x`, `dir_y`, `dir_z` — Direction light rays travel (from sun toward surface)
    /// * `color` — Sun color (0xRRGGBBAA)
    /// * `sharpness` — Sun disc sharpness (0.0-1.0, higher = smaller/sharper)
    pub fn sky_set_sun(dir_x: f32, dir_y: f32, dir_z: f32, color: u32, sharpness: f32);

    /// Bind a matcap texture to a slot (Mode 1 only).
    ///
    /// # Arguments
    /// * `slot` — Matcap slot (1-3)
    pub fn matcap_set(slot: u32, texture: u32);

    /// Draw the procedural sky. Call first in render(), before any geometry.
    pub fn draw_sky();

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

    // =========================================================================
    // GPU Skinning
    // =========================================================================

    /// Set bone transform matrices for skeletal animation.
    ///
    /// # Arguments
    /// * `matrices_ptr` — Pointer to array of 4x4 matrices (16 floats each, column-major)
    /// * `count` — Number of bones (max 256)
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

    /// Play music (dedicated looping channel).
    pub fn music_play(sound: u32, volume: f32);

    /// Stop music.
    pub fn music_stop();

    /// Set music volume.
    pub fn music_set_volume(volume: f32);
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
    pub const UNLIT: u32 = 0;
    pub const MATCAP: u32 = 1;
    pub const PBR: u32 = 2;
    pub const HYBRID: u32 = 3;
}

/// Blend modes for `blend_mode()`
pub mod blend {
    pub const NONE: u32 = 0;
    pub const ALPHA: u32 = 1;
    pub const ADDITIVE: u32 = 2;
    pub const MULTIPLY: u32 = 3;
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

/// Resolution indices for `set_resolution()`
pub mod resolution {
    pub const RES_360P: u32 = 0;
    pub const RES_540P: u32 = 1;
    pub const RES_720P: u32 = 2;
    pub const RES_1080P: u32 = 3;
}

/// Tick rate indices for `set_tick_rate()`
pub mod tick_rate {
    pub const FPS_24: u32 = 0;
    pub const FPS_30: u32 = 1;
    pub const FPS_60: u32 = 2;
    pub const FPS_120: u32 = 3;
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
