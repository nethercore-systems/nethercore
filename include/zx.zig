// GENERATED FILE - DO NOT EDIT
// Source: nethercore/include/zx.rs
// Generator: tools/ffi-gen

// =============================================================================
// System
// =============================================================================

/// Returns the fixed timestep duration in seconds.
/// 
/// This is a **constant value** based on the configured tick rate, NOT wall-clock time.
/// - 60fps → 0.01666... (1/60)
/// - 30fps → 0.03333... (1/30)
/// 
/// Safe for rollback netcode: identical across all clients regardless of frame timing.
pub extern "C" fn delta_time() f32;

/// Returns total elapsed game time since start in seconds.
/// 
/// This is the **accumulated fixed timestep**, NOT wall-clock time.
/// Calculated as `tick_count * delta_time`.
/// 
/// Safe for rollback netcode: deterministic and identical across all clients.
pub extern "C" fn elapsed_time() f32;

/// Returns the current tick number (starts at 0, increments by 1 each update).
/// 
/// Perfectly deterministic: same inputs always produce the same tick count.
/// Safe for rollback netcode.
pub extern "C" fn tick_count() u64;

/// Logs a message to the console output.
/// 
/// # Arguments
/// * `ptr` — Pointer to UTF-8 string data
/// * `len` — Length of string in bytes
pub extern "C" fn log(ptr: [*]const u8, len: u32) void;

/// Exits the game and returns to the library.
pub extern "C" fn quit() void;

/// Returns a deterministic random u32 from the host's seeded RNG.
/// Always use this instead of external random sources for rollback compatibility.
pub extern "C" fn random() u32;

/// Returns a random i32 in range [min, max).
/// Uses host's seeded RNG for rollback compatibility.
pub extern "C" fn random_range(min: i32, max: i32) i32;

/// Returns a random f32 in range [0.0, 1.0).
/// Uses host's seeded RNG for rollback compatibility.
pub extern "C" fn random_f32() f32;

/// Returns a random f32 in range [min, max).
/// Uses host's seeded RNG for rollback compatibility.
pub extern "C" fn random_f32_range(min: f32, max: f32) f32;

/// Returns the number of players in the session (1-4).
pub extern "C" fn player_count() u32;

/// Returns a bitmask of which players are local to this client.
/// 
/// Example: `(local_player_mask() & (1 << player_id)) != 0` checks if player is local.
pub extern "C" fn local_player_mask() u32;

/// Saves data to a slot.
/// 
/// # Arguments
/// * `slot` — Save slot (0-7)
/// * `data_ptr` — Pointer to data in WASM memory
/// * `data_len` — Length of data in bytes (max 64KB)
/// 
/// # Returns
/// 0 on success, 1 if invalid slot, 2 if data too large.
pub extern "C" fn save(slot: u32, data_ptr: [*]const u8, data_len: u32) u32;

/// Loads data from a slot.
/// 
/// # Arguments
/// * `slot` — Save slot (0-7)
/// * `data_ptr` — Pointer to buffer in WASM memory
/// * `max_len` — Maximum bytes to read
/// 
/// # Returns
/// Bytes read (0 if empty or error).
pub extern "C" fn load(slot: u32, data_ptr: [*]u8, max_len: u32) u32;

/// Deletes a save slot.
/// 
/// # Returns
/// 0 on success, 1 if invalid slot.
pub extern "C" fn delete(slot: u32) u32;

/// Set the clear/background color. Must be called during `init()`.
/// 
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format (default: black)
pub extern "C" fn set_clear_color(color: u32) void;

/// Set the camera position and target (look-at point).
/// 
/// Uses a Y-up, right-handed coordinate system.
pub extern "C" fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32) void;

/// Set the camera field of view.
/// 
/// # Arguments
/// * `fov_degrees` — Field of view in degrees (typically 45-90, default 60)
pub extern "C" fn camera_fov(fov_degrees: f32) void;

/// Push a custom view matrix (16 floats, column-major order).
pub extern "C" fn push_view_matrix(m0: f32, m1: f32, m2: f32, m3: f32, m4: f32, m5: f32, m6: f32, m7: f32, m8: f32, m9: f32, m10: f32, m11: f32, m12: f32, m13: f32, m14: f32, m15: f32) void;

/// Push a custom projection matrix (16 floats, column-major order).
pub extern "C" fn push_projection_matrix(m0: f32, m1: f32, m2: f32, m3: f32, m4: f32, m5: f32, m6: f32, m7: f32, m8: f32, m9: f32, m10: f32, m11: f32, m12: f32, m13: f32, m14: f32, m15: f32) void;

/// Push identity matrix onto the transform stack.
pub extern "C" fn push_identity() void;

/// Set the current transform from a 4x4 matrix pointer (16 floats, column-major).
pub extern "C" fn transform_set(matrix_ptr: [*]const f32) void;

/// Push a translation transform.
pub extern "C" fn push_translate(x: f32, y: f32, z: f32) void;

/// Push a rotation around the X axis.
/// 
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
pub extern "C" fn push_rotate_x(angle_deg: f32) void;

/// Push a rotation around the Y axis.
/// 
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
pub extern "C" fn push_rotate_y(angle_deg: f32) void;

/// Push a rotation around the Z axis.
/// 
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
pub extern "C" fn push_rotate_z(angle_deg: f32) void;

/// Push a rotation around an arbitrary axis.
/// 
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
/// * `axis_x`, `axis_y`, `axis_z` — Rotation axis (will be normalized)
pub extern "C" fn push_rotate(angle_deg: f32, axis_x: f32, axis_y: f32, axis_z: f32) void;

/// Push a non-uniform scale transform.
pub extern "C" fn push_scale(x: f32, y: f32, z: f32) void;

/// Push a uniform scale transform.
pub extern "C" fn push_scale_uniform(s: f32) void;

/// Check if a button is currently held.
/// 
/// # Button indices
/// 0=UP, 1=DOWN, 2=LEFT, 3=RIGHT, 4=A, 5=B, 6=X, 7=Y,
/// 8=L1, 9=R1, 10=L3, 11=R3, 12=START, 13=SELECT
/// 
/// # Returns
/// 1 if held, 0 otherwise.
pub extern "C" fn button_held(player: u32, button: u32) u32;

/// Check if a button was just pressed this tick.
/// 
/// # Returns
/// 1 if just pressed, 0 otherwise.
pub extern "C" fn button_pressed(player: u32, button: u32) u32;

/// Check if a button was just released this tick.
/// 
/// # Returns
/// 1 if just released, 0 otherwise.
pub extern "C" fn button_released(player: u32, button: u32) u32;

/// Get bitmask of all held buttons.
pub extern "C" fn buttons_held(player: u32) u32;

/// Get bitmask of all buttons just pressed this tick.
pub extern "C" fn buttons_pressed(player: u32) u32;

/// Get bitmask of all buttons just released this tick.
pub extern "C" fn buttons_released(player: u32) u32;

/// Get left stick X axis value (-1.0 to 1.0).
pub extern "C" fn left_stick_x(player: u32) f32;

/// Get left stick Y axis value (-1.0 to 1.0).
pub extern "C" fn left_stick_y(player: u32) f32;

/// Get right stick X axis value (-1.0 to 1.0).
pub extern "C" fn right_stick_x(player: u32) f32;

/// Get right stick Y axis value (-1.0 to 1.0).
pub extern "C" fn right_stick_y(player: u32) f32;

/// Get both left stick axes at once (more efficient).
/// 
/// Writes X and Y values to the provided pointers.
pub extern "C" fn left_stick(player: u32, out_x: [*]f32, out_y: [*]f32) void;

/// Get both right stick axes at once (more efficient).
/// 
/// Writes X and Y values to the provided pointers.
pub extern "C" fn right_stick(player: u32, out_x: [*]f32, out_y: [*]f32) void;

/// Get left trigger value (0.0 to 1.0).
pub extern "C" fn trigger_left(player: u32) f32;

/// Get right trigger value (0.0 to 1.0).
pub extern "C" fn trigger_right(player: u32) f32;

/// Set the uniform tint color (multiplied with vertex colors and textures).
/// 
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format
pub extern "C" fn set_color(color: u32) void;

/// Set the EPU environment index (`env_id`) used for subsequent draw calls.
/// 
/// This selects which EPU environment textures are sampled for:
/// - `draw_epu()` background rendering
/// - Reflections + ambient lighting in lit render modes (2/3)
/// 
/// Notes:
/// - `env_id` is clamped to the supported range (0..255).
/// - Default is 0.
pub extern "C" fn environment_index(env_id: u32) void;

/// Set the face culling mode.
/// 
/// # Arguments
/// * `mode` — 0=none (default), 1=back, 2=front
pub extern "C" fn cull_mode(mode: u32) void;

/// Set the texture filtering mode.
/// 
/// # Arguments
/// * `filter` — 0=nearest (pixelated), 1=linear (smooth)
pub extern "C" fn texture_filter(filter: u32) void;

/// Set uniform alpha level for dither transparency.
/// 
/// # Arguments
/// * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15)
/// 
/// Controls the dither pattern threshold for screen-door transparency.
/// The dither pattern is always active, but with level=15 (default) all fragments pass.
pub extern "C" fn uniform_alpha(level: u32) void;

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
pub extern "C" fn dither_offset(x: u32, y: u32) void;

/// Set z-index for 2D ordering control within a pass.
/// 
/// # Arguments
/// * `n` — Z-index value (0 = back, higher = front)
/// 
/// Higher z-index values are drawn on top of lower values.
/// Use this to ensure UI elements appear over game content
/// regardless of texture bindings or draw order.
/// 
/// Note: z_index only affects ordering within the same pass_id.
/// Default: 0 (resets each frame)
pub extern "C" fn z_index(n: u32) void;

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
/// epu_set(env_config_ptr);
/// draw_mesh(scene);
/// draw_epu();
/// 
/// // Player 2: right half
/// viewport(480, 0, 480, 540);
/// camera_set(p2_x, p2_y, p2_z, p2_tx, p2_ty, p2_tz);
/// epu_set(env_config_ptr);
/// draw_mesh(scene);
/// draw_epu();
/// 
/// // Reset for HUD
/// viewport_clear();
/// draw_text_str("PAUSED", 400.0, 270.0, 32.0, 0xFFFFFFFF);
/// ```
pub extern "C" fn viewport(x: u32, y: u32, width: u32, height: u32) void;

/// Reset viewport to fullscreen (960×540).
/// 
/// Call this at the end of split-screen rendering to restore full-screen
/// coordinates for HUD elements or between frames.
pub extern "C" fn viewport_clear() void;

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
pub extern "C" fn begin_pass(clear_depth: u32) void;

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
pub extern "C" fn begin_pass_stencil_write(ref_value: u32, clear_depth: u32) void;

/// Begin a stencil test pass (render inside mask).
/// 
/// After calling this, subsequent draw calls only render where
/// the stencil buffer equals ref_value (inside the mask).
/// 
/// # Arguments
/// * `ref_value` — Stencil reference value to test against (must match write pass)
/// * `clear_depth` — Non-zero to clear depth buffer at pass start
pub extern "C" fn begin_pass_stencil_test(ref_value: u32, clear_depth: u32) void;

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
pub extern "C" fn begin_pass_full(depth_compare: u32, depth_write: u32, clear_depth: u32, stencil_compare: u32, stencil_ref: u32, stencil_pass_op: u32, stencil_fail_op: u32, stencil_depth_fail_op: u32) void;

/// Load a texture from RGBA pixel data.
/// 
/// # Arguments
/// * `width`, `height` — Texture dimensions
/// * `pixels_ptr` — Pointer to RGBA8 pixel data (width × height × 4 bytes)
/// 
/// # Returns
/// Texture handle (>0) on success, 0 on failure.
pub extern "C" fn load_texture(width: u32, height: u32, pixels_ptr: [*]const u8) u32;

/// Bind a texture to slot 0 (albedo).
pub extern "C" fn texture_bind(handle: u32) void;

/// Bind a texture to a specific slot.
/// 
/// # Arguments
/// * `slot` — 0=albedo, 1=MRE/matcap, 2=reserved, 3=matcap
pub extern "C" fn texture_bind_slot(handle: u32, slot: u32) void;

/// Set matcap blend mode for a texture slot (Mode 1 only).
/// 
/// # Arguments
/// * `slot` — Matcap slot (1-3)
/// * `mode` — 0=Multiply, 1=Add, 2=HSV Modulate
pub extern "C" fn matcap_blend_mode(slot: u32, mode: u32) void;

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
pub extern "C" fn load_mesh(data_ptr: [*]const f32, vertex_count: u32, format: u32) u32;

/// Load an indexed mesh.
/// 
/// # Returns
/// Mesh handle (>0) on success, 0 on failure.
pub extern "C" fn load_mesh_indexed(data_ptr: [*]const f32, vertex_count: u32, index_ptr: [*]const u16, index_count: u32, format: u32) u32;

/// Load packed mesh data (power user API, f16/snorm16/unorm8 encoding).
pub extern "C" fn load_mesh_packed(data_ptr: [*]const u8, vertex_count: u32, format: u32) u32;

/// Load indexed packed mesh data (power user API).
pub extern "C" fn load_mesh_indexed_packed(data_ptr: [*]const u8, vertex_count: u32, index_ptr: [*]const u16, index_count: u32, format: u32) u32;

/// Draw a retained mesh with current transform and render state.
pub extern "C" fn draw_mesh(handle: u32) void;

/// Generate a cube mesh. **Init-only.**
/// 
/// # Arguments
/// * `size_x`, `size_y`, `size_z` — Half-extents along each axis
pub extern "C" fn cube(size_x: f32, size_y: f32, size_z: f32) u32;

/// Generate a UV sphere mesh. **Init-only.**
/// 
/// # Arguments
/// * `radius` — Sphere radius
/// * `segments` — Longitudinal divisions (3-256)
/// * `rings` — Latitudinal divisions (2-256)
pub extern "C" fn sphere(radius: f32, segments: u32, rings: u32) u32;

/// Generate a cylinder or cone mesh. **Init-only.**
/// 
/// # Arguments
/// * `radius_bottom`, `radius_top` — Radii (>= 0.0, use 0 for cone tip)
/// * `height` — Cylinder height
/// * `segments` — Radial divisions (3-256)
pub extern "C" fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;

/// Generate a plane mesh on the XZ plane. **Init-only.**
/// 
/// # Arguments
/// * `size_x`, `size_z` — Dimensions
/// * `subdivisions_x`, `subdivisions_z` — Subdivisions (1-256)
pub extern "C" fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;

/// Generate a torus (donut) mesh. **Init-only.**
/// 
/// # Arguments
/// * `major_radius` — Distance from center to tube center
/// * `minor_radius` — Tube radius
/// * `major_segments`, `minor_segments` — Segment counts (3-256)
pub extern "C" fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;

/// Generate a capsule (pill shape) mesh. **Init-only.**
/// 
/// # Arguments
/// * `radius` — Capsule radius
/// * `height` — Height of cylindrical section (total = height + 2*radius)
/// * `segments` — Radial divisions (3-256)
/// * `rings` — Divisions per hemisphere (1-128)
pub extern "C" fn capsule(radius: f32, height: f32, segments: u32, rings: u32) u32;

/// Generate a UV sphere mesh with equirectangular texture mapping. **Init-only.**
pub extern "C" fn sphere_uv(radius: f32, segments: u32, rings: u32) u32;

/// Generate a plane mesh with UV mapping. **Init-only.**
pub extern "C" fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;

/// Generate a cube mesh with box-unwrapped UV mapping. **Init-only.**
pub extern "C" fn cube_uv(size_x: f32, size_y: f32, size_z: f32) u32;

/// Generate a cylinder mesh with cylindrical UV mapping. **Init-only.**
pub extern "C" fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;

/// Generate a torus mesh with wrapped UV mapping. **Init-only.**
pub extern "C" fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;

/// Generate a capsule mesh with hybrid UV mapping. **Init-only.**
pub extern "C" fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) u32;

/// Generate a sphere mesh with tangent data for normal mapping. **Init-only.**
/// 
/// Tangent follows direction of increasing U (longitude).
/// Use with material_normal() for normal-mapped rendering.
pub extern "C" fn sphere_tangent(radius: f32, segments: u32, rings: u32) u32;

/// Generate a plane mesh with tangent data for normal mapping. **Init-only.**
/// 
/// Tangent points along +X, bitangent along +Z, normal along +Y.
pub extern "C" fn plane_tangent(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;

/// Generate a cube mesh with tangent data for normal mapping. **Init-only.**
/// 
/// Each face has correct tangent space for normal map sampling.
pub extern "C" fn cube_tangent(size_x: f32, size_y: f32, size_z: f32) u32;

/// Generate a torus mesh with tangent data for normal mapping. **Init-only.**
/// 
/// Tangent follows the major circle direction.
pub extern "C" fn torus_tangent(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;

/// Draw triangles immediately (non-indexed).
/// 
/// # Arguments
/// * `vertex_count` — Must be multiple of 3
/// * `format` — Vertex format flags (0-15)
pub extern "C" fn draw_triangles(data_ptr: [*]const f32, vertex_count: u32, format: u32) void;

/// Draw indexed triangles immediately.
/// 
/// # Arguments
/// * `index_count` — Must be multiple of 3
/// * `format` — Vertex format flags (0-15)
pub extern "C" fn draw_triangles_indexed(data_ptr: [*]const f32, vertex_count: u32, index_ptr: [*]const u16, index_count: u32, format: u32) void;

/// Draw a billboard (camera-facing quad) with full texture.
/// 
/// Uses the color set by `set_color()`.
/// 
/// # Arguments
/// * `w`, `h` — Billboard size in world units
/// * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
pub extern "C" fn draw_billboard(w: f32, h: f32, mode: u32) void;

/// Draw a billboard with a UV region from the texture.
/// 
/// Uses the color set by `set_color()`.
/// 
/// # Arguments
/// * `w`, `h` — Billboard size in world units
/// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
/// * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
pub extern "C" fn draw_billboard_region(w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, mode: u32) void;

/// Draw a sprite with the bound texture.
/// 
/// # Arguments
/// * `x`, `y` — Screen position in pixels (0,0 = top-left)
/// * `w`, `h` — Sprite size in pixels
pub extern "C" fn draw_sprite(x: f32, y: f32, w: f32, h: f32) void;

/// Draw a region of a sprite sheet.
/// 
/// # Arguments
/// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
pub extern "C" fn draw_sprite_region(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32) void;

/// Draw a sprite with full control (rotation, origin, UV region).
/// 
/// # Arguments
/// * `origin_x`, `origin_y` — Rotation pivot point (in pixels from sprite top-left)
/// * `angle_deg` — Rotation angle in degrees (clockwise)
pub extern "C" fn draw_sprite_ex(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, origin_x: f32, origin_y: f32, angle_deg: f32) void;

/// Draw a solid color rectangle.
pub extern "C" fn draw_rect(x: f32, y: f32, w: f32, h: f32) void;

/// Draw text with the current font.
/// 
/// # Arguments
/// * `ptr` — Pointer to UTF-8 string data
/// * `len` — Length in bytes
/// * `size` — Font size in pixels
pub extern "C" fn draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32) void;

/// Measure the width of text when rendered.
/// 
/// # Arguments
/// * `ptr` — Pointer to UTF-8 string data
/// * `len` — Length in bytes
/// * `size` — Font size in pixels
/// 
/// # Returns
/// Width in pixels that the text would occupy when rendered.
pub extern "C" fn text_width(ptr: [*]const u8, len: u32, size: f32) f32;

/// Draw a line between two points.
/// 
/// # Arguments
/// * `x1`, `y1` — Start point in screen pixels
/// * `x2`, `y2` — End point in screen pixels
/// * `thickness` — Line thickness in pixels
pub extern "C" fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32) void;

/// Draw a filled circle.
/// 
/// # Arguments
/// * `x`, `y` — Center position in screen pixels
/// * `radius` — Circle radius in pixels
/// 
/// Rendered as a 16-segment triangle fan.
pub extern "C" fn draw_circle(x: f32, y: f32, radius: f32) void;

/// Draw a circle outline.
/// 
/// # Arguments
/// * `x`, `y` — Center position in screen pixels
/// * `radius` — Circle radius in pixels
/// * `thickness` — Line thickness in pixels
/// 
/// Rendered as 16 line segments.
pub extern "C" fn draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32) void;

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
pub extern "C" fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) u32;

/// Load a variable-width bitmap font.
/// 
/// # Arguments
/// * `widths_ptr` — Pointer to array of char_count u8 widths
pub extern "C" fn load_font_ex(texture: u32, widths_ptr: [*]const u8, char_height: u32, first_codepoint: u32, char_count: u32) u32;

/// Bind a font for subsequent draw_text() calls.
/// 
/// Pass 0 for the built-in 8×8 monospace font.
pub extern "C" fn font_bind(font_handle: u32) void;

/// Bind a matcap texture to a slot (Mode 1 only).
/// 
/// # Arguments
/// * `slot` — Matcap slot (1-3)
pub extern "C" fn matcap_set(slot: u32, texture: u32) void;

/// Draw the environment background using an EPU configuration (128-byte).
/// 
/// Reads a 128-byte (8 x 128-bit = 16 x u64) environment configuration from
/// WASM memory and renders the procedural background for the current viewport
/// and render pass. If called multiple times for the same env_id in a frame, the last call wins.
/// 
/// # Arguments
/// * `config_ptr` — Pointer to 16 u64 values (128 bytes total) in WASM memory
/// 
/// # Configuration Layout
/// Each environment is exactly 8 x 128-bit instructions (each stored as [hi, lo]):
/// - Slots 0-3: Enclosure/bounds layers (`0x01..0x07`)
/// - Slots 4-7: Radiance/feature layers (`0x08..0x1F`)
/// 
/// # Instruction Bit Layout (per 128-bit = 2 x u64)
/// ```text
/// u64 hi [bits 127..64]:
/// 63..59  opcode     (5)   Which algorithm to run (32 opcodes)
/// 58..56  region     (3)   Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
/// 55..53  blend      (3)   8 blend modes
/// 52..48  meta5      (5)   (domain_id<<3)|variant_id; use 0 when unused
/// 47..24  color_a    (24)  RGB24 primary color
/// 23..0   color_b    (24)  RGB24 secondary color
/// 
/// u64 lo [bits 63..0]:
/// 63..56  intensity  (8)   Layer brightness
/// 55..48  param_a    (8)   Opcode-specific
/// 47..40  param_b    (8)   Opcode-specific
/// 39..32  param_c    (8)   Opcode-specific
/// 31..24  param_d    (8)   Opcode-specific
/// 23..8   direction  (16)  Octahedral-encoded direction
/// 7..4    alpha_a    (4)   color_a alpha (0-15)
/// 3..0    alpha_b    (4)   color_b alpha (0-15)
/// ```
/// 
/// # Opcodes (common)
/// - 0x00: NOP (disable layer)
/// - 0x01: RAMP (enclosure gradient)
/// - 0x02: SECTOR (enclosure modifier)
/// - 0x03: SILHOUETTE (enclosure modifier)
/// - 0x04: SPLIT (enclosure source)
/// - 0x05: CELL (enclosure source)
/// - 0x06: PATCHES (enclosure source)
/// - 0x07: APERTURE (enclosure modifier)
/// - 0x08: DECAL (sharp SDF shape)
/// - 0x09: GRID (repeating lines/panels)
/// - 0x0A: SCATTER (point field)
/// - 0x0B: FLOW (animated noise/streaks)
/// - 0x0C..0x13: radiance opcodes (TRACE/VEIL/ATMOSPHERE/PLANE/CELESTIAL/PORTAL/LOBE_RADIANCE/BAND_RADIANCE)
/// 
/// # Blend Modes
/// - 0: ADD (dst + src * a)
/// - 1: MULTIPLY (dst * mix(1, src, a))
/// - 2: MAX (max(dst, src * a))
/// - 3: LERP (mix(dst, src, a))
/// - 4: SCREEN (1 - (1-dst)*(1-src*a))
/// - 5: HSV_MOD (HSV shift dst by src)
/// - 6: MIN (min(dst, src * a))
/// - 7: OVERLAY (Photoshop-style overlay)
/// 
/// Store the environment configuration for the current `environment_index(...)`.
/// 
/// Use this to set the active environment config for this frame without
/// doing a fullscreen background draw.
/// 
/// # Usage
/// ```rust,ignore
/// fn render() {
/// // Set environment configuration at the start of the pass/frame
/// epu_set(config.as_ptr());
/// 
/// // Draw scene geometry
/// draw_mesh(terrain);
/// draw_mesh(player);
/// 
/// // Draw environment background last (fills only background pixels)
/// draw_epu();
/// }
/// ```
/// 
/// # Notes
/// - The EPU compute pass runs automatically before rendering
pub extern "C" fn epu_set(config_ptr: [*]const u64) void;

/// Draw the environment background for the current viewport/pass.
/// 
/// This draws a fullscreen background using the config selected by
/// `environment_index(...)` (and previously provided via `epu_set(...)` or
/// `epu_set_env(...)`).
/// 
/// For split-screen / multi-pass, set `viewport(...)` and call `draw_epu()`
/// once per viewport/pass where you want an environment background.
pub extern "C" fn draw_epu() void;

/// Store an EPU configuration for an environment ID without drawing a background.
/// 
/// Use this to set up multiple environments in the same frame, then select
/// per-draw lighting/reflections via `environment_index(...)`.
pub extern "C" fn epu_set_env(env_id: u32, config_ptr: [*]const u64) void;

/// Bind an MRE texture (Metallic-Roughness-Emissive) to slot 1.
pub extern "C" fn material_mre(texture: u32) void;

/// Bind an albedo texture to slot 0.
pub extern "C" fn material_albedo(texture: u32) void;

/// Bind a normal map texture to slot 3.
/// 
/// # Arguments
/// * `texture` — Handle to a BC5 or RGBA normal map texture
/// 
/// Normal maps perturb surface normals for detailed lighting without extra geometry.
/// Requires mesh with tangent data (FORMAT_TANGENT) and UVs.
/// Works in all lit modes (0=Lambert, 2=PBR, 3=Hybrid) and Mode 1 (Matcap).
pub extern "C" fn material_normal(texture: u32) void;

/// Skip normal map sampling (use vertex normal instead).
/// 
/// # Arguments
/// * `skip` — 1 to skip normal map, 0 to use normal map (default)
/// 
/// When a mesh has tangent data, normal mapping is enabled by default.
/// Use this flag to opt out temporarily for debugging or artistic control.
pub extern "C" fn skip_normal_map(skip: u32) void;

/// Set material metallic value (0.0 = dielectric, 1.0 = metal).
pub extern "C" fn material_metallic(value: f32) void;

/// Set material roughness value (0.0 = smooth, 1.0 = rough).
pub extern "C" fn material_roughness(value: f32) void;

/// Set material emissive intensity (0.0 = no emission, >1.0 for HDR).
pub extern "C" fn material_emissive(value: f32) void;

/// Set rim lighting parameters.
/// 
/// # Arguments
/// * `intensity` — Rim brightness (0.0-1.0)
/// * `power` — Falloff sharpness (0.0-32.0, higher = tighter)
pub extern "C" fn material_rim(intensity: f32, power: f32) void;

/// Enable/disable uniform color override.
/// 
/// When enabled, uses the last set_color() value for all subsequent draws,
/// overriding vertex colors and material albedo.
/// 
/// # Arguments
/// * `enabled` — 1 to enable, 0 to disable
pub extern "C" fn use_uniform_color(enabled: u32) void;

/// Enable/disable uniform metallic override.
/// 
/// When enabled, uses the last material_metallic() value for all subsequent draws,
/// overriding per-vertex or per-material metallic values.
/// 
/// # Arguments
/// * `enabled` — 1 to enable, 0 to disable
pub extern "C" fn use_uniform_metallic(enabled: u32) void;

/// Enable/disable uniform roughness override.
/// 
/// When enabled, uses the last material_roughness() value for all subsequent draws,
/// overriding per-vertex or per-material roughness values.
/// 
/// # Arguments
/// * `enabled` — 1 to enable, 0 to disable
pub extern "C" fn use_uniform_roughness(enabled: u32) void;

/// Enable/disable uniform emissive override.
/// 
/// When enabled, uses the last material_emissive() value for all subsequent draws,
/// overriding per-vertex or per-material emissive values.
/// 
/// # Arguments
/// * `enabled` — 1 to enable, 0 to disable
pub extern "C" fn use_uniform_emissive(enabled: u32) void;

/// Set shininess (Mode 3 alias for roughness).
pub extern "C" fn material_shininess(value: f32) void;

/// Set specular color (Mode 3 only).
/// 
/// # Arguments
/// * `color` — Specular color (0xRRGGBBAA, alpha ignored)
pub extern "C" fn material_specular(color: u32) void;

/// Set light direction (and enable the light).
/// 
/// # Arguments
/// * `index` — Light index (0-3)
/// * `x`, `y`, `z` — Direction rays travel (from light toward surface)
/// 
/// For a light from above, use (0, -1, 0).
pub extern "C" fn light_set(index: u32, x: f32, y: f32, z: f32) void;

/// Set light color.
/// 
/// # Arguments
/// * `color` — Light color (0xRRGGBBAA, alpha ignored)
pub extern "C" fn light_color(index: u32, color: u32) void;

/// Set light intensity multiplier.
/// 
/// # Arguments
/// * `intensity` — Typically 0.0-10.0
pub extern "C" fn light_intensity(index: u32, intensity: f32) void;

/// Enable a light.
pub extern "C" fn light_enable(index: u32) void;

/// Disable a light (preserves settings for re-enabling).
pub extern "C" fn light_disable(index: u32) void;

/// Convert a light to a point light at world position.
/// 
/// # Arguments
/// * `index` — Light index (0-3)
/// * `x`, `y`, `z` — World-space position
/// 
/// Enables the light automatically. Default range is 10.0 units.
pub extern "C" fn light_set_point(index: u32, x: f32, y: f32, z: f32) void;

/// Set point light falloff distance.
/// 
/// # Arguments
/// * `index` — Light index (0-3)
/// * `range` — Distance at which light reaches zero intensity
/// 
/// Only affects point lights (ignored for directional).
pub extern "C" fn light_range(index: u32, range: f32) void;

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
pub extern "C" fn load_skeleton(inverse_bind_ptr: [*]const f32, bone_count: u32) u32;

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
pub extern "C" fn skeleton_bind(skeleton: u32) void;

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
pub extern "C" fn set_bones(matrices_ptr: [*]const f32, count: u32) void;

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
pub extern "C" fn set_bones_4x4(matrices_ptr: [*]const f32, count: u32) void;

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
pub extern "C" fn keyframes_load(data_ptr: [*]const u8, byte_size: u32) u32;

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
pub extern "C" fn rom_keyframes(id_ptr: [*]const u8, id_len: u32) u32;

/// Get the bone count for a keyframe collection.
/// 
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
/// 
/// # Returns
/// Bone count (0 on invalid handle)
pub extern "C" fn keyframes_bone_count(handle: u32) u32;

/// Get the frame count for a keyframe collection.
/// 
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
/// 
/// # Returns
/// Frame count (0 on invalid handle)
pub extern "C" fn keyframes_frame_count(handle: u32) u32;

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
pub extern "C" fn keyframe_read(handle: u32, index: u32, out_ptr: [*]u8) void;

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
pub extern "C" fn keyframe_bind(handle: u32, index: u32) void;

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
pub extern "C" fn load_sound(data_ptr: [*]const i16, byte_len: u32) u32;

/// Play sound on next available channel (fire-and-forget).
/// 
/// # Arguments
/// * `volume` — 0.0 to 1.0
/// * `pan` — -1.0 (left) to 1.0 (right), 0.0 = center
pub extern "C" fn play_sound(sound: u32, volume: f32, pan: f32) void;

/// Play sound on a specific channel (for managed/looping audio).
/// 
/// # Arguments
/// * `channel` — Channel index (0-15)
/// * `looping` — 1 = loop, 0 = play once
pub extern "C" fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32) void;

/// Update channel parameters (call every frame for positional audio).
pub extern "C" fn channel_set(channel: u32, volume: f32, pan: f32) void;

/// Stop a channel.
pub extern "C" fn channel_stop(channel: u32) void;

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
pub extern "C" fn rom_tracker(id_ptr: [*]const u8, id_len: u32) u32;

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
pub extern "C" fn load_tracker(data_ptr: [*]const u8, data_len: u32) u32;

/// Play music (PCM sound or tracker module).
/// 
/// Automatically stops any currently playing music of the other type.
/// Handle type is detected by bit 31 (0=PCM, 1=tracker).
/// 
/// # Arguments
/// * `handle` — Sound handle (from load_sound) or tracker handle (from rom_tracker)
/// * `volume` — 0.0 to 1.0
/// * `looping` — 1 = loop, 0 = play once
pub extern "C" fn music_play(handle: u32, volume: f32, looping: u32) void;

/// Stop music (both PCM and tracker).
pub extern "C" fn music_stop() void;

/// Pause or resume music (tracker only, no-op for PCM).
/// 
/// # Arguments
/// * `paused` — 1 = pause, 0 = resume
pub extern "C" fn music_pause(paused: u32) void;

/// Set music volume (works for both PCM and tracker).
/// 
/// # Arguments
/// * `volume` — 0.0 to 1.0
pub extern "C" fn music_set_volume(volume: f32) void;

/// Check if music is currently playing.
/// 
/// # Returns
/// 1 if playing (and not paused), 0 otherwise.
pub extern "C" fn music_is_playing() u32;

/// Get current music type.
/// 
/// # Returns
/// 0 = none, 1 = PCM, 2 = tracker
pub extern "C" fn music_type() u32;

/// Jump to a specific position (tracker only, no-op for PCM).
/// 
/// Use for dynamic music systems (e.g., jump to outro pattern).
/// 
/// # Arguments
/// * `order` — Order position (0-based)
/// * `row` — Row within the pattern (0-based)
pub extern "C" fn music_jump(order: u32, row: u32) void;

/// Get current music position.
/// 
/// For tracker: (order << 16) | row
/// For PCM: sample position
/// 
/// # Returns
/// Position value (format depends on music type).
pub extern "C" fn music_position() u32;

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
pub extern "C" fn music_length(handle: u32) u32;

/// Set music speed (tracker only, ticks per row).
/// 
/// # Arguments
/// * `speed` — 1-31 (XM default is 6)
pub extern "C" fn music_set_speed(speed: u32) void;

/// Set music tempo (tracker only, BPM).
/// 
/// # Arguments
/// * `bpm` — 32-255 (XM default is 125)
pub extern "C" fn music_set_tempo(bpm: u32) void;

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
pub extern "C" fn music_info(handle: u32) u32;

/// Get music name (tracker only, returns 0 for PCM).
/// 
/// # Arguments
/// * `handle` — Music handle
/// * `out_ptr` — Pointer to output buffer
/// * `max_len` — Maximum bytes to write
/// 
/// # Returns
/// Actual length written (0 if PCM or invalid handle).
pub extern "C" fn music_name(handle: u32, out_ptr: [*]u8, max_len: u32) u32;

/// Load a texture from ROM data pack by ID.
/// 
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
/// 
/// # Returns
/// Texture handle (>0) on success. Traps on failure.
pub extern "C" fn rom_texture(id_ptr: [*]const u8, id_len: u32) u32;

/// Load a mesh from ROM data pack by ID.
/// 
/// # Returns
/// Mesh handle (>0) on success. Traps on failure.
pub extern "C" fn rom_mesh(id_ptr: [*]const u8, id_len: u32) u32;

/// Load skeleton inverse bind matrices from ROM data pack by ID.
/// 
/// # Returns
/// Skeleton handle (>0) on success. Traps on failure.
pub extern "C" fn rom_skeleton(id_ptr: [*]const u8, id_len: u32) u32;

/// Load a font atlas from ROM data pack by ID.
/// 
/// # Returns
/// Texture handle for font atlas (>0) on success. Traps on failure.
pub extern "C" fn rom_font(id_ptr: [*]const u8, id_len: u32) u32;

/// Load a sound from ROM data pack by ID.
/// 
/// # Returns
/// Sound handle (>0) on success. Traps on failure.
pub extern "C" fn rom_sound(id_ptr: [*]const u8, id_len: u32) u32;

/// Get the byte size of raw data in the ROM data pack.
/// 
/// Use this to allocate a buffer before calling `rom_data()`.
/// 
/// # Returns
/// Byte count on success. Traps if not found.
pub extern "C" fn rom_data_len(id_ptr: [*]const u8, id_len: u32) u32;

/// Copy raw data from ROM data pack into WASM linear memory.
/// 
/// # Arguments
/// * `id_ptr`, `id_len` — Asset ID string
/// * `dst_ptr` — Pointer to destination buffer in WASM memory
/// * `max_len` — Maximum bytes to copy (size of destination buffer)
/// 
/// # Returns
/// Bytes written on success. Traps on failure.
pub extern "C" fn rom_data(id_ptr: [*]const u8, id_len: u32, dst_ptr: [*]const u8, max_len: u32) u32;

/// Load a mesh from .nczxmesh binary format.
/// 
/// # Arguments
/// * `data_ptr` — Pointer to .nczxmesh binary data
/// * `data_len` — Length of the data in bytes
/// 
/// # Returns
/// Mesh handle (>0) on success, 0 on failure.
pub extern "C" fn load_zmesh(data_ptr: [*]const u8, data_len: u32) u32;

/// Load a texture from .nczxtex binary format.
/// 
/// # Returns
/// Texture handle (>0) on success, 0 on failure.
pub extern "C" fn load_ztex(data_ptr: [*]const u8, data_len: u32) u32;

/// Load a sound from .nczxsnd binary format.
/// 
/// # Returns
/// Sound handle (>0) on success, 0 on failure.
pub extern "C" fn load_zsound(data_ptr: [*]const u8, data_len: u32) u32;

/// Register an i8 value for debug inspection.
pub extern "C" fn debug_register_i8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register an i16 value for debug inspection.
pub extern "C" fn debug_register_i16(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register an i32 value for debug inspection.
pub extern "C" fn debug_register_i32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a u8 value for debug inspection.
pub extern "C" fn debug_register_u8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a u16 value for debug inspection.
pub extern "C" fn debug_register_u16(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a u32 value for debug inspection.
pub extern "C" fn debug_register_u32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register an f32 value for debug inspection.
pub extern "C" fn debug_register_f32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a bool value for debug inspection.
pub extern "C" fn debug_register_bool(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register an i32 with min/max range constraints.
pub extern "C" fn debug_register_i32_range(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8, min: i32, max: i32) void;

/// Register an f32 with min/max range constraints.
pub extern "C" fn debug_register_f32_range(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8, min: f32, max: f32) void;

/// Register a u8 with min/max range constraints.
pub extern "C" fn debug_register_u8_range(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8, min: u32, max: u32) void;

/// Register a u16 with min/max range constraints.
pub extern "C" fn debug_register_u16_range(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8, min: u32, max: u32) void;

/// Register an i16 with min/max range constraints.
pub extern "C" fn debug_register_i16_range(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8, min: i32, max: i32) void;

/// Register a Vec2 (2 floats: x, y) for debug inspection.
pub extern "C" fn debug_register_vec2(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a Vec3 (3 floats: x, y, z) for debug inspection.
pub extern "C" fn debug_register_vec3(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a Rect (4 i16: x, y, w, h) for debug inspection.
pub extern "C" fn debug_register_rect(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register a Color (4 u8: RGBA) for debug inspection with color picker.
pub extern "C" fn debug_register_color(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register Q8.8 fixed-point (i16) for debug inspection.
pub extern "C" fn debug_register_fixed_i16_q8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register Q16.16 fixed-point (i32) for debug inspection.
pub extern "C" fn debug_register_fixed_i32_q16(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register Q24.8 fixed-point (i32) for debug inspection.
pub extern "C" fn debug_register_fixed_i32_q8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Register Q8.24 fixed-point (i32) for debug inspection.
pub extern "C" fn debug_register_fixed_i32_q24(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch an i8 value (read-only).
pub extern "C" fn debug_watch_i8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch an i16 value (read-only).
pub extern "C" fn debug_watch_i16(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch an i32 value (read-only).
pub extern "C" fn debug_watch_i32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a u8 value (read-only).
pub extern "C" fn debug_watch_u8(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a u16 value (read-only).
pub extern "C" fn debug_watch_u16(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a u32 value (read-only).
pub extern "C" fn debug_watch_u32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch an f32 value (read-only).
pub extern "C" fn debug_watch_f32(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a bool value (read-only).
pub extern "C" fn debug_watch_bool(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a Vec2 value (read-only).
pub extern "C" fn debug_watch_vec2(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a Vec3 value (read-only).
pub extern "C" fn debug_watch_vec3(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a Rect value (read-only).
pub extern "C" fn debug_watch_rect(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Watch a Color value (read-only).
pub extern "C" fn debug_watch_color(name_ptr: [*]const u8, name_len: u32, ptr: [*]const u8) void;

/// Begin a collapsible group in the debug UI.
pub extern "C" fn debug_group_begin(name_ptr: [*]const u8, name_len: u32) void;

/// End the current debug group.
pub extern "C" fn debug_group_end() void;

/// Register a simple action with no parameters.
/// 
/// Creates a button in the debug UI that calls the specified WASM function when clicked.
/// 
/// # Parameters
/// - `name_ptr`: Pointer to button label string
/// - `name_len`: Length of button label
/// - `func_name_ptr`: Pointer to WASM function name string
/// - `func_name_len`: Length of function name
pub extern "C" fn debug_register_action(name_ptr: [*]const u8, name_len: u32, func_name_ptr: [*]const u8, func_name_len: u32) void;

/// Begin building an action with parameters.
/// 
/// Use with debug_action_param_* and debug_action_end() to create an action with input fields.
/// 
/// # Parameters
/// - `name_ptr`: Pointer to button label string
/// - `name_len`: Length of button label
/// - `func_name_ptr`: Pointer to WASM function name string
/// - `func_name_len`: Length of function name
pub extern "C" fn debug_action_begin(name_ptr: [*]const u8, name_len: u32, func_name_ptr: [*]const u8, func_name_len: u32) void;

/// Add an i32 parameter to the pending action.
/// 
/// # Parameters
/// - `name_ptr`: Pointer to parameter label string
/// - `name_len`: Length of parameter label
/// - `default_value`: Default value for the parameter
pub extern "C" fn debug_action_param_i32(name_ptr: [*]const u8, name_len: u32, default_value: i32) void;

/// Add an f32 parameter to the pending action.
/// 
/// # Parameters
/// - `name_ptr`: Pointer to parameter label string
/// - `name_len`: Length of parameter label
/// - `default_value`: Default value for the parameter
pub extern "C" fn debug_action_param_f32(name_ptr: [*]const u8, name_len: u32, default_value: f32) void;

/// Finish building the pending action.
/// 
/// Completes the action registration started with debug_action_begin().
pub extern "C" fn debug_action_end() void;

/// Query if the game is currently paused (debug mode).
/// 
/// # Returns
/// 1 if paused, 0 if running normally.
pub extern "C" fn debug_is_paused() i32;

/// Get the current time scale multiplier.
/// 
/// # Returns
/// 1.0 = normal, 0.5 = half-speed, 2.0 = double-speed, etc.
pub extern "C" fn debug_get_time_scale() f32;

// =============================================================================
// Constants
// =============================================================================

pub const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;
    pub const a: u32 = 4;
    pub const b: u32 = 5;
    pub const x: u32 = 6;
    pub const y: u32 = 7;
    pub const l1: u32 = 8;
    pub const r1: u32 = 9;
    pub const l3: u32 = 10;
    pub const r3: u32 = 11;
    pub const start: u32 = 12;
    pub const select: u32 = 13;
};

pub const Cull = struct {
    pub const none: u32 = 0;
    pub const back: u32 = 1;
    pub const front: u32 = 2;
};

pub const Format = struct {
    pub const pos: u8 = 0;
    pub const uv: u8 = 1;
    pub const color: u8 = 2;
    pub const normal: u8 = 4;
    pub const skinned: u8 = 8;
    pub const tangent: u8 = 16;
    pub const pos_uv: u8 = uv;
    pub const pos_color: u8 = color;
    pub const pos_normal: u8 = normal;
    pub const pos_uv_normal: u8 = uv | normal;
    pub const pos_uv_color: u8 = uv | color;
    pub const pos_uv_color_normal: u8 = uv | color | normal;
    pub const pos_skinned: u8 = skinned;
    pub const pos_normal_skinned: u8 = normal | skinned;
    pub const pos_uv_normal_skinned: u8 = uv | normal | skinned;
    pub const pos_uv_normal_tangent: u8 = uv | normal | tangent;
    pub const pos_uv_color_normal_tangent: u8 = uv | color | normal | tangent;
};

pub const Billboard = struct {
    pub const spherical: u32 = 1;
    pub const cylindrical_y: u32 = 2;
    pub const cylindrical_x: u32 = 3;
    pub const cylindrical_z: u32 = 4;
};

pub const Screen = struct {
    pub const width: u32 = 960;
    pub const height: u32 = 540;
};

pub const Compare = struct {
    pub const never: u32 = 1;
    pub const less: u32 = 2;
    pub const equal: u32 = 3;
    pub const less_equal: u32 = 4;
    pub const greater: u32 = 5;
    pub const not_equal: u32 = 6;
    pub const greater_equal: u32 = 7;
    pub const always: u32 = 8;
};

pub const StencilOp = struct {
    pub const keep: u32 = 0;
    pub const zero: u32 = 1;
    pub const replace: u32 = 2;
    pub const increment_clamp: u32 = 3;
    pub const decrement_clamp: u32 = 4;
    pub const invert: u32 = 5;
    pub const increment_wrap: u32 = 6;
    pub const decrement_wrap: u32 = 7;
};

pub const color = struct {
    pub const white: u32 = 0xFFFFFFFF;
    pub const black: u32 = 0x000000FF;
    pub const red: u32 = 0xFF0000FF;
    pub const green: u32 = 0x00FF00FF;
    pub const blue: u32 = 0x0000FFFF;
    pub const yellow: u32 = 0xFFFF00FF;
    pub const cyan: u32 = 0x00FFFFFF;
    pub const magenta: u32 = 0xFF00FFFF;
    pub const orange: u32 = 0xFF8000FF;
    pub const transparent: u32 = 0x00000000;
};


// =============================================================================
// MANUALLY MAINTAINED HELPER FUNCTIONS
// =============================================================================
// These helpers provide Zig-specific conveniences using slices and native types

const std = @import("std");

/// Color packing helpers
pub fn rgba(r: u8, g: u8, b: u8, a: u8) u32 {
    return (@as(u32, r) << 24) | (@as(u32, g) << 16) | (@as(u32, b) << 8) | @as(u32, a);
}

pub fn rgb(r: u8, g: u8, b: u8) u32 {
    return rgba(r, g, b, 255);
}

/// Math helpers using Zig built-ins
pub fn clampf(val: f32, min: f32, max: f32) f32 {
    return @max(min, @min(val, max));
}

pub fn lerpf(a: f32, b: f32, t: f32) f32 {
    return a + (b - a) * t;
}

/// String helpers using Zig slices
pub fn logStr(msg: []const u8) void {
    log(msg.ptr, @intCast(msg.len));
}

pub fn drawTextStr(msg: []const u8, x: f32, y: f32, size: f32, col: u32) void {
    draw_text(msg.ptr, @intCast(msg.len), x, y, size, col);
}

/// ROM loading helpers
pub fn romTexture(id: []const u8) u32 {
    return rom_texture(@intFromPtr(id.ptr), @intCast(id.len));
}

pub fn romMesh(id: []const u8) u32 {
    return rom_mesh(@intFromPtr(id.ptr), @intCast(id.len));
}

pub fn romSound(id: []const u8) u32 {
    return rom_sound(@intFromPtr(id.ptr), @intCast(id.len));
}

pub fn romFont(id: []const u8) u32 {
    return rom_font(@intFromPtr(id.ptr), @intCast(id.len));
}

pub fn romSkeleton(id: []const u8) u32 {
    return rom_skeleton(@intFromPtr(id.ptr), @intCast(id.len));
}
