// GENERATED FILE - DO NOT EDIT
// Source: nethercore/include/zx.rs
// Generator: tools/ffi-gen

#ifndef NETHERCORE_ZX_H
#define NETHERCORE_ZX_H

#include <stdint.h>
#include <stdbool.h>

#define NCZX_EXPORT __attribute__((visibility("default")))
#define NCZX_IMPORT __attribute__((import_module("env")))

#ifdef __cplusplus
extern "C" {
#endif

// =============================================================================
// System
// =============================================================================

/** Returns the fixed timestep duration in seconds. */
/**  */
/** This is a **constant value** based on the configured tick rate, NOT wall-clock time. */
/** - 60fps → 0.01666... (1/60) */
/** - 30fps → 0.03333... (1/30) */
/**  */
/** Safe for rollback netcode: identical across all clients regardless of frame timing. */
NCZX_IMPORT float delta_time(void);

/** Returns total elapsed game time since start in seconds. */
/**  */
/** This is the **accumulated fixed timestep**, NOT wall-clock time. */
/** Calculated as `tick_count * delta_time`. */
/**  */
/** Safe for rollback netcode: deterministic and identical across all clients. */
NCZX_IMPORT float elapsed_time(void);

/** Returns the current tick number (starts at 0, increments by 1 each update). */
/**  */
/** Perfectly deterministic: same inputs always produce the same tick count. */
/** Safe for rollback netcode. */
NCZX_IMPORT uint64_t tick_count(void);

/** Logs a message to the console output. */
/**  */
/** # Arguments */
/** * `ptr` — Pointer to UTF-8 string data */
/** * `len` — Length of string in bytes */
NCZX_IMPORT void log(const uint8_t* ptr, uint32_t len);

/** Exits the game and returns to the library. */
NCZX_IMPORT void quit(void);

/** Returns a deterministic random u32 from the host's seeded RNG. */
/** Always use this instead of external random sources for rollback compatibility. */
NCZX_IMPORT uint32_t random(void);

/** Returns a random i32 in range [min, max). */
/** Uses host's seeded RNG for rollback compatibility. */
NCZX_IMPORT int32_t random_range(int32_t min, int32_t max);

/** Returns a random f32 in range [0.0, 1.0). */
/** Uses host's seeded RNG for rollback compatibility. */
NCZX_IMPORT float random_f32(void);

/** Returns a random f32 in range [min, max). */
/** Uses host's seeded RNG for rollback compatibility. */
NCZX_IMPORT float random_f32_range(float min, float max);

/** Returns the number of players in the session (1-4). */
NCZX_IMPORT uint32_t player_count(void);

/** Returns a bitmask of which players are local to this client. */
/**  */
/** Example: `(local_player_mask() & (1 << player_id)) != 0` checks if player is local. */
NCZX_IMPORT uint32_t local_player_mask(void);

/** Saves data to a slot. */
/**  */
/** # Arguments */
/** * `slot` — Save slot (0-7) */
/** * `data_ptr` — Pointer to data in WASM memory */
/** * `data_len` — Length of data in bytes (max 64KB) */
/**  */
/** # Returns */
/** 0 on success, 1 if invalid slot, 2 if data too large. */
NCZX_IMPORT uint32_t save(uint32_t slot, const uint8_t* data_ptr, uint32_t data_len);

/** Loads data from a slot. */
/**  */
/** # Arguments */
/** * `slot` — Save slot (0-7) */
/** * `data_ptr` — Pointer to buffer in WASM memory */
/** * `max_len` — Maximum bytes to read */
/**  */
/** # Returns */
/** Bytes read (0 if empty or error). */
NCZX_IMPORT uint32_t load(uint32_t slot, uint8_t* data_ptr, uint32_t max_len);

/** Deletes a save slot. */
/**  */
/** # Returns */
/** 0 on success, 1 if invalid slot. */
NCZX_IMPORT uint32_t delete(uint32_t slot);

/** Set the clear/background color. Must be called during `init()`. */
/**  */
/** # Arguments */
/** * `color` — Color in 0xRRGGBBAA format (default: black) */
NCZX_IMPORT void set_clear_color(uint32_t color);

/** Set the camera position and target (look-at point). */
/**  */
/** Uses a Y-up, right-handed coordinate system. */
NCZX_IMPORT void camera_set(float x, float y, float z, float target_x, float target_y, float target_z);

/** Set the camera field of view. */
/**  */
/** # Arguments */
/** * `fov_degrees` — Field of view in degrees (typically 45-90, default 60) */
NCZX_IMPORT void camera_fov(float fov_degrees);

/** Push a custom view matrix (16 floats, column-major order). */
NCZX_IMPORT void push_view_matrix(float m0, float m1, float m2, float m3, float m4, float m5, float m6, float m7, float m8, float m9, float m10, float m11, float m12, float m13, float m14, float m15);

/** Push a custom projection matrix (16 floats, column-major order). */
NCZX_IMPORT void push_projection_matrix(float m0, float m1, float m2, float m3, float m4, float m5, float m6, float m7, float m8, float m9, float m10, float m11, float m12, float m13, float m14, float m15);

/** Push identity matrix onto the transform stack. */
NCZX_IMPORT void push_identity(void);

/** Set the current transform from a 4x4 matrix pointer (16 floats, column-major). */
NCZX_IMPORT void transform_set(const float* matrix_ptr);

/** Push a translation transform. */
NCZX_IMPORT void push_translate(float x, float y, float z);

/** Push a rotation around the X axis. */
/**  */
/** # Arguments */
/** * `angle_deg` — Rotation angle in degrees */
NCZX_IMPORT void push_rotate_x(float angle_deg);

/** Push a rotation around the Y axis. */
/**  */
/** # Arguments */
/** * `angle_deg` — Rotation angle in degrees */
NCZX_IMPORT void push_rotate_y(float angle_deg);

/** Push a rotation around the Z axis. */
/**  */
/** # Arguments */
/** * `angle_deg` — Rotation angle in degrees */
NCZX_IMPORT void push_rotate_z(float angle_deg);

/** Push a rotation around an arbitrary axis. */
/**  */
/** # Arguments */
/** * `angle_deg` — Rotation angle in degrees */
/** * `axis_x`, `axis_y`, `axis_z` — Rotation axis (will be normalized) */
NCZX_IMPORT void push_rotate(float angle_deg, float axis_x, float axis_y, float axis_z);

/** Push a non-uniform scale transform. */
NCZX_IMPORT void push_scale(float x, float y, float z);

/** Push a uniform scale transform. */
NCZX_IMPORT void push_scale_uniform(float s);

/** Check if a button is currently held. */
/**  */
/** # Button indices */
/** 0=UP, 1=DOWN, 2=LEFT, 3=RIGHT, 4=A, 5=B, 6=X, 7=Y, */
/** 8=L1, 9=R1, 10=L3, 11=R3, 12=START, 13=SELECT */
/**  */
/** # Returns */
/** 1 if held, 0 otherwise. */
NCZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);

/** Check if a button was just pressed this tick. */
/**  */
/** # Returns */
/** 1 if just pressed, 0 otherwise. */
NCZX_IMPORT uint32_t button_pressed(uint32_t player, uint32_t button);

/** Check if a button was just released this tick. */
/**  */
/** # Returns */
/** 1 if just released, 0 otherwise. */
NCZX_IMPORT uint32_t button_released(uint32_t player, uint32_t button);

/** Get bitmask of all held buttons. */
NCZX_IMPORT uint32_t buttons_held(uint32_t player);

/** Get bitmask of all buttons just pressed this tick. */
NCZX_IMPORT uint32_t buttons_pressed(uint32_t player);

/** Get bitmask of all buttons just released this tick. */
NCZX_IMPORT uint32_t buttons_released(uint32_t player);

/** Get left stick X axis value (-1.0 to 1.0). */
NCZX_IMPORT float left_stick_x(uint32_t player);

/** Get left stick Y axis value (-1.0 to 1.0). */
NCZX_IMPORT float left_stick_y(uint32_t player);

/** Get right stick X axis value (-1.0 to 1.0). */
NCZX_IMPORT float right_stick_x(uint32_t player);

/** Get right stick Y axis value (-1.0 to 1.0). */
NCZX_IMPORT float right_stick_y(uint32_t player);

/** Get both left stick axes at once (more efficient). */
/**  */
/** Writes X and Y values to the provided pointers. */
NCZX_IMPORT void left_stick(uint32_t player, float* out_x, float* out_y);

/** Get both right stick axes at once (more efficient). */
/**  */
/** Writes X and Y values to the provided pointers. */
NCZX_IMPORT void right_stick(uint32_t player, float* out_x, float* out_y);

/** Get left trigger value (0.0 to 1.0). */
NCZX_IMPORT float trigger_left(uint32_t player);

/** Get right trigger value (0.0 to 1.0). */
NCZX_IMPORT float trigger_right(uint32_t player);

/** Set the uniform tint color (multiplied with vertex colors and textures). */
/**  */
/** # Arguments */
/** * `color` — Color in 0xRRGGBBAA format */
NCZX_IMPORT void set_color(uint32_t color);

/** Set the face culling mode. */
/**  */
/** # Arguments */
/** * `mode` — 0=none (default), 1=back, 2=front */
NCZX_IMPORT void cull_mode(uint32_t mode);

/** Set the texture filtering mode. */
/**  */
/** # Arguments */
/** * `filter` — 0=nearest (pixelated), 1=linear (smooth) */
NCZX_IMPORT void texture_filter(uint32_t filter);

/** Set uniform alpha level for dither transparency. */
/**  */
/** # Arguments */
/** * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15) */
/**  */
/** Controls the dither pattern threshold for screen-door transparency. */
/** The dither pattern is always active, but with level=15 (default) all fragments pass. */
NCZX_IMPORT void uniform_alpha(uint32_t level);

/** Set dither offset for dither transparency. */
/**  */
/** # Arguments */
/** * `x` — 0-3 pixel shift in X axis */
/** * `y` — 0-3 pixel shift in Y axis */
/**  */
/** Use different offsets for stacked dithered meshes to prevent pattern cancellation. */
/** When two transparent objects overlap with the same alpha level and offset, their */
/** dither patterns align and pixels cancel out. Different offsets shift the pattern */
/** so both objects remain visible. */
NCZX_IMPORT void dither_offset(uint32_t x, uint32_t y);

/** Set z-index for 2D ordering control within a pass. */
/**  */
/** # Arguments */
/** * `n` — Z-index value (0 = back, higher = front) */
/**  */
/** Higher z-index values are drawn on top of lower values. */
/** Use this to ensure UI elements appear over game content */
/** regardless of texture bindings or draw order. */
/**  */
/** Note: z_index only affects ordering within the same pass_id. */
/** Default: 0 (resets each frame) */
NCZX_IMPORT void z_index(uint32_t n);

/** Set the viewport for subsequent draw calls. */
/**  */
/** All 3D and 2D rendering will be clipped to this region. */
/** Camera aspect ratio automatically adjusts to viewport dimensions. */
/** 2D coordinates (draw_sprite, draw_text, etc.) become viewport-relative. */
/**  */
/** # Arguments */
/** * `x` — Left edge in pixels (0-959) */
/** * `y` — Top edge in pixels (0-539) */
/** * `width` — Width in pixels (1-960) */
/** * `height` — Height in pixels (1-540) */
/**  */
/** # Example (2-player horizontal split) */
/** ```rust,ignore */
/** // Player 1: left half */
/** viewport(0, 0, 480, 540); */
/** camera_set(p1_x, p1_y, p1_z, p1_tx, p1_ty, p1_tz); */
/** draw_env(); */
/** draw_mesh(scene); */
/**  */
/** // Player 2: right half */
/** viewport(480, 0, 480, 540); */
/** camera_set(p2_x, p2_y, p2_z, p2_tx, p2_ty, p2_tz); */
/** draw_env(); */
/** draw_mesh(scene); */
/**  */
/** // Reset for HUD */
/** viewport_clear(); */
/** draw_text_str("PAUSED", 400.0, 270.0, 32.0, 0xFFFFFFFF); */
/** ``` */
NCZX_IMPORT void viewport(uint32_t x, uint32_t y, uint32_t width, uint32_t height);

/** Reset viewport to fullscreen (960×540). */
/**  */
/** Call this at the end of split-screen rendering to restore full-screen */
/** coordinates for HUD elements or between frames. */
NCZX_IMPORT void viewport_clear(void);

/** Begin a new render pass with optional depth clear. */
/**  */
/** Provides an execution barrier - commands in this pass complete before */
/** the next pass begins. Use for layered rendering like FPS viewmodels. */
/**  */
/** # Arguments */
/** * `clear_depth` — Non-zero to clear depth buffer at pass start */
/**  */
/** # Example (FPS viewmodel rendering) */
/** ```rust,ignore */
/** // Draw world first (pass 0) */
/** draw_env(); */
/** draw_mesh(world_mesh); */
/**  */
/** // Draw gun on top (pass 1 with depth clear) */
/** begin_pass(1);  // Clear depth so gun renders on top */
/** draw_mesh(gun_mesh); */
/** ``` */
NCZX_IMPORT void begin_pass(uint32_t clear_depth);

/** Begin a stencil write pass (mask creation mode). */
/**  */
/** After calling this, subsequent draw calls write to the stencil buffer */
/** but NOT to the color buffer. Use this to create a mask shape. */
/** Depth testing is disabled to prevent mask geometry from polluting depth. */
/**  */
/** # Arguments */
/** * `ref_value` — Stencil reference value to write (typically 1) */
/** * `clear_depth` — Non-zero to clear depth buffer at pass start */
/**  */
/** # Example (scope mask) */
/** ```rust,ignore */
/** begin_pass_stencil_write(1, 0);  // Start mask creation */
/** draw_mesh(circle_mesh);          // Draw circle to stencil only */
/** begin_pass_stencil_test(1, 0);   // Enable testing */
/** draw_env();                       // Only visible inside circle */
/** begin_pass(0);                    // Back to normal rendering */
/** ``` */
NCZX_IMPORT void begin_pass_stencil_write(uint32_t ref_value, uint32_t clear_depth);

/** Begin a stencil test pass (render inside mask). */
/**  */
/** After calling this, subsequent draw calls only render where */
/** the stencil buffer equals ref_value (inside the mask). */
/**  */
/** # Arguments */
/** * `ref_value` — Stencil reference value to test against (must match write pass) */
/** * `clear_depth` — Non-zero to clear depth buffer at pass start */
NCZX_IMPORT void begin_pass_stencil_test(uint32_t ref_value, uint32_t clear_depth);

/** Begin a render pass with full control over depth and stencil state. */
/**  */
/** This is the "escape hatch" for advanced effects not covered by the */
/** convenience functions. Most games should use begin_pass, begin_pass_stencil_write, */
/** or begin_pass_stencil_test instead. */
/**  */
/** # Arguments */
/** * `depth_compare` — Depth comparison function (see compare::* constants) */
/** * `depth_write` — Non-zero to write to depth buffer */
/** * `clear_depth` — Non-zero to clear depth buffer at pass start */
/** * `stencil_compare` — Stencil comparison function (see compare::* constants) */
/** * `stencil_ref` — Stencil reference value (0-255) */
/** * `stencil_pass_op` — Operation when stencil test passes (see stencil_op::* constants) */
/** * `stencil_fail_op` — Operation when stencil test fails */
/** * `stencil_depth_fail_op` — Operation when depth test fails */
NCZX_IMPORT void begin_pass_full(uint32_t depth_compare, uint32_t depth_write, uint32_t clear_depth, uint32_t stencil_compare, uint32_t stencil_ref, uint32_t stencil_pass_op, uint32_t stencil_fail_op, uint32_t stencil_depth_fail_op);

/** Load a texture from RGBA pixel data. */
/**  */
/** # Arguments */
/** * `width`, `height` — Texture dimensions */
/** * `pixels_ptr` — Pointer to RGBA8 pixel data (width × height × 4 bytes) */
/**  */
/** # Returns */
/** Texture handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_texture(uint32_t width, uint32_t height, const uint8_t* pixels_ptr);

/** Bind a texture to slot 0 (albedo). */
NCZX_IMPORT void texture_bind(uint32_t handle);

/** Bind a texture to a specific slot. */
/**  */
/** # Arguments */
/** * `slot` — 0=albedo, 1=MRE/matcap, 2=reserved, 3=matcap */
NCZX_IMPORT void texture_bind_slot(uint32_t handle, uint32_t slot);

/** Set matcap blend mode for a texture slot (Mode 1 only). */
/**  */
/** # Arguments */
/** * `slot` — Matcap slot (1-3) */
/** * `mode` — 0=Multiply, 1=Add, 2=HSV Modulate */
NCZX_IMPORT void matcap_blend_mode(uint32_t slot, uint32_t mode);

/** Load a non-indexed mesh. */
/**  */
/** # Vertex format flags */
/** - 1 (FORMAT_UV): Has UV coordinates (2 floats) */
/** - 2 (FORMAT_COLOR): Has per-vertex color (3 floats RGB) */
/** - 4 (FORMAT_NORMAL): Has normals (3 floats) */
/** - 8 (FORMAT_SKINNED): Has bone indices/weights */
/**  */
/** # Returns */
/** Mesh handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_mesh(const float* data_ptr, uint32_t vertex_count, uint32_t format);

/** Load an indexed mesh. */
/**  */
/** # Returns */
/** Mesh handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_mesh_indexed(const float* data_ptr, uint32_t vertex_count, const uint16_t* index_ptr, uint32_t index_count, uint32_t format);

/** Load packed mesh data (power user API, f16/snorm16/unorm8 encoding). */
NCZX_IMPORT uint32_t load_mesh_packed(const uint8_t* data_ptr, uint32_t vertex_count, uint32_t format);

/** Load indexed packed mesh data (power user API). */
NCZX_IMPORT uint32_t load_mesh_indexed_packed(const uint8_t* data_ptr, uint32_t vertex_count, const uint16_t* index_ptr, uint32_t index_count, uint32_t format);

/** Draw a retained mesh with current transform and render state. */
NCZX_IMPORT void draw_mesh(uint32_t handle);

/** Generate a cube mesh. **Init-only.** */
/**  */
/** # Arguments */
/** * `size_x`, `size_y`, `size_z` — Half-extents along each axis */
NCZX_IMPORT uint32_t cube(float size_x, float size_y, float size_z);

/** Generate a UV sphere mesh. **Init-only.** */
/**  */
/** # Arguments */
/** * `radius` — Sphere radius */
/** * `segments` — Longitudinal divisions (3-256) */
/** * `rings` — Latitudinal divisions (2-256) */
NCZX_IMPORT uint32_t sphere(float radius, uint32_t segments, uint32_t rings);

/** Generate a cylinder or cone mesh. **Init-only.** */
/**  */
/** # Arguments */
/** * `radius_bottom`, `radius_top` — Radii (>= 0.0, use 0 for cone tip) */
/** * `height` — Cylinder height */
/** * `segments` — Radial divisions (3-256) */
NCZX_IMPORT uint32_t cylinder(float radius_bottom, float radius_top, float height, uint32_t segments);

/** Generate a plane mesh on the XZ plane. **Init-only.** */
/**  */
/** # Arguments */
/** * `size_x`, `size_z` — Dimensions */
/** * `subdivisions_x`, `subdivisions_z` — Subdivisions (1-256) */
NCZX_IMPORT uint32_t plane(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);

/** Generate a torus (donut) mesh. **Init-only.** */
/**  */
/** # Arguments */
/** * `major_radius` — Distance from center to tube center */
/** * `minor_radius` — Tube radius */
/** * `major_segments`, `minor_segments` — Segment counts (3-256) */
NCZX_IMPORT uint32_t torus(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);

/** Generate a capsule (pill shape) mesh. **Init-only.** */
/**  */
/** # Arguments */
/** * `radius` — Capsule radius */
/** * `height` — Height of cylindrical section (total = height + 2*radius) */
/** * `segments` — Radial divisions (3-256) */
/** * `rings` — Divisions per hemisphere (1-128) */
NCZX_IMPORT uint32_t capsule(float radius, float height, uint32_t segments, uint32_t rings);

/** Generate a UV sphere mesh with equirectangular texture mapping. **Init-only.** */
NCZX_IMPORT uint32_t sphere_uv(float radius, uint32_t segments, uint32_t rings);

/** Generate a plane mesh with UV mapping. **Init-only.** */
NCZX_IMPORT uint32_t plane_uv(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);

/** Generate a cube mesh with box-unwrapped UV mapping. **Init-only.** */
NCZX_IMPORT uint32_t cube_uv(float size_x, float size_y, float size_z);

/** Generate a cylinder mesh with cylindrical UV mapping. **Init-only.** */
NCZX_IMPORT uint32_t cylinder_uv(float radius_bottom, float radius_top, float height, uint32_t segments);

/** Generate a torus mesh with wrapped UV mapping. **Init-only.** */
NCZX_IMPORT uint32_t torus_uv(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);

/** Generate a capsule mesh with hybrid UV mapping. **Init-only.** */
NCZX_IMPORT uint32_t capsule_uv(float radius, float height, uint32_t segments, uint32_t rings);

/** Generate a sphere mesh with tangent data for normal mapping. **Init-only.** */
/**  */
/** Tangent follows direction of increasing U (longitude). */
/** Use with material_normal() for normal-mapped rendering. */
NCZX_IMPORT uint32_t sphere_tangent(float radius, uint32_t segments, uint32_t rings);

/** Generate a plane mesh with tangent data for normal mapping. **Init-only.** */
/**  */
/** Tangent points along +X, bitangent along +Z, normal along +Y. */
NCZX_IMPORT uint32_t plane_tangent(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);

/** Generate a cube mesh with tangent data for normal mapping. **Init-only.** */
/**  */
/** Each face has correct tangent space for normal map sampling. */
NCZX_IMPORT uint32_t cube_tangent(float size_x, float size_y, float size_z);

/** Generate a torus mesh with tangent data for normal mapping. **Init-only.** */
/**  */
/** Tangent follows the major circle direction. */
NCZX_IMPORT uint32_t torus_tangent(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);

/** Draw triangles immediately (non-indexed). */
/**  */
/** # Arguments */
/** * `vertex_count` — Must be multiple of 3 */
/** * `format` — Vertex format flags (0-15) */
NCZX_IMPORT void draw_triangles(const float* data_ptr, uint32_t vertex_count, uint32_t format);

/** Draw indexed triangles immediately. */
/**  */
/** # Arguments */
/** * `index_count` — Must be multiple of 3 */
/** * `format` — Vertex format flags (0-15) */
NCZX_IMPORT void draw_triangles_indexed(const float* data_ptr, uint32_t vertex_count, const uint16_t* index_ptr, uint32_t index_count, uint32_t format);

/** Draw a billboard (camera-facing quad) with full texture. */
/**  */
/** Uses the color set by `set_color()`. */
/**  */
/** # Arguments */
/** * `w`, `h` — Billboard size in world units */
/** * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z */
NCZX_IMPORT void draw_billboard(float w, float h, uint32_t mode);

/** Draw a billboard with a UV region from the texture. */
/**  */
/** Uses the color set by `set_color()`. */
/**  */
/** # Arguments */
/** * `w`, `h` — Billboard size in world units */
/** * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0) */
/** * `mode` — 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z */
NCZX_IMPORT void draw_billboard_region(float w, float h, float src_x, float src_y, float src_w, float src_h, uint32_t mode);

/** Draw a sprite with the bound texture. */
/**  */
/** # Arguments */
/** * `x`, `y` — Screen position in pixels (0,0 = top-left) */
/** * `w`, `h` — Sprite size in pixels */
NCZX_IMPORT void draw_sprite(float x, float y, float w, float h);

/** Draw a region of a sprite sheet. */
/**  */
/** # Arguments */
/** * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0) */
NCZX_IMPORT void draw_sprite_region(float x, float y, float w, float h, float src_x, float src_y, float src_w, float src_h);

/** Draw a sprite with full control (rotation, origin, UV region). */
/**  */
/** # Arguments */
/** * `origin_x`, `origin_y` — Rotation pivot point (in pixels from sprite top-left) */
/** * `angle_deg` — Rotation angle in degrees (clockwise) */
NCZX_IMPORT void draw_sprite_ex(float x, float y, float w, float h, float src_x, float src_y, float src_w, float src_h, float origin_x, float origin_y, float angle_deg);

/** Draw a solid color rectangle. */
NCZX_IMPORT void draw_rect(float x, float y, float w, float h);

/** Draw text with the current font. */
/**  */
/** # Arguments */
/** * `ptr` — Pointer to UTF-8 string data */
/** * `len` — Length in bytes */
/** * `size` — Font size in pixels */
NCZX_IMPORT void draw_text(const uint8_t* ptr, uint32_t len, float x, float y, float size);

/** Measure the width of text when rendered. */
/**  */
/** # Arguments */
/** * `ptr` — Pointer to UTF-8 string data */
/** * `len` — Length in bytes */
/** * `size` — Font size in pixels */
/**  */
/** # Returns */
/** Width in pixels that the text would occupy when rendered. */
NCZX_IMPORT float text_width(const uint8_t* ptr, uint32_t len, float size);

/** Draw a line between two points. */
/**  */
/** # Arguments */
/** * `x1`, `y1` — Start point in screen pixels */
/** * `x2`, `y2` — End point in screen pixels */
/** * `thickness` — Line thickness in pixels */
NCZX_IMPORT void draw_line(float x1, float y1, float x2, float y2, float thickness);

/** Draw a filled circle. */
/**  */
/** # Arguments */
/** * `x`, `y` — Center position in screen pixels */
/** * `radius` — Circle radius in pixels */
/**  */
/** Rendered as a 16-segment triangle fan. */
NCZX_IMPORT void draw_circle(float x, float y, float radius);

/** Draw a circle outline. */
/**  */
/** # Arguments */
/** * `x`, `y` — Center position in screen pixels */
/** * `radius` — Circle radius in pixels */
/** * `thickness` — Line thickness in pixels */
/**  */
/** Rendered as 16 line segments. */
NCZX_IMPORT void draw_circle_outline(float x, float y, float radius, float thickness);

/** Load a fixed-width bitmap font. */
/**  */
/** # Arguments */
/** * `texture` — Texture atlas handle */
/** * `char_width`, `char_height` — Glyph dimensions in pixels */
/** * `first_codepoint` — Unicode codepoint of first glyph */
/** * `char_count` — Number of glyphs */
/**  */
/** # Returns */
/** Font handle (use with `font_bind()`). */
NCZX_IMPORT uint32_t load_font(uint32_t texture, uint32_t char_width, uint32_t char_height, uint32_t first_codepoint, uint32_t char_count);

/** Load a variable-width bitmap font. */
/**  */
/** # Arguments */
/** * `widths_ptr` — Pointer to array of char_count u8 widths */
NCZX_IMPORT uint32_t load_font_ex(uint32_t texture, const uint8_t* widths_ptr, uint32_t char_height, uint32_t first_codepoint, uint32_t char_count);

/** Bind a font for subsequent draw_text() calls. */
/**  */
/** Pass 0 for the built-in 8×8 monospace font. */
NCZX_IMPORT void font_bind(uint32_t font_handle);

/** Render the configured environment. Call first in render(), before any geometry. */
NCZX_IMPORT void draw_env(void);

/** Bind a matcap texture to a slot (Mode 1 only). */
/**  */
/** # Arguments */
/** * `slot` — Matcap slot (1-3) */
NCZX_IMPORT void matcap_set(uint32_t slot, uint32_t texture);

/** Draw the environment background using an EPU configuration (128-byte). */
/**  */
/** Reads a 128-byte (8 x 128-bit = 16 x u64) environment configuration from */
/** WASM memory and renders the procedural background for the current viewport */
/** and render pass. If called multiple times in a frame, the last call wins. */
/**  */
/** # Arguments */
/** * `config_ptr` — Pointer to 16 u64 values (128 bytes total) in WASM memory */
/**  */
/** # Configuration Layout */
/** Each environment is exactly 8 x 128-bit instructions (each stored as [hi, lo]): */
/** - Slots 0-3: Enclosure/bounds layers (`0x01..0x07`) */
/** - Slots 4-7: Radiance/feature layers (`0x08..0x1F`) */
/**  */
/** # Instruction Bit Layout (per 128-bit = 2 x u64) */
/** ```text */
/** u64 hi [bits 127..64]: */
/** 63..59  opcode     (5)   Which algorithm to run (32 opcodes) */
/** 58..56  region     (3)   Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001 */
/** 55..53  blend      (3)   8 blend modes */
/** 52..48  meta5      (5)   (domain_id<<3)|variant_id; use 0 when unused */
/** 47..24  color_a    (24)  RGB24 primary color */
/** 23..0   color_b    (24)  RGB24 secondary color */
/**  */
/** u64 lo [bits 63..0]: */
/** 63..56  intensity  (8)   Layer brightness */
/** 55..48  param_a    (8)   Opcode-specific */
/** 47..40  param_b    (8)   Opcode-specific */
/** 39..32  param_c    (8)   Opcode-specific */
/** 31..24  param_d    (8)   Opcode-specific */
/** 23..8   direction  (16)  Octahedral-encoded direction */
/** 7..4    alpha_a    (4)   color_a alpha (0-15) */
/** 3..0    alpha_b    (4)   color_b alpha (0-15) */
/** ``` */
/**  */
/** # Opcodes (common) */
/** - 0x00: NOP (disable layer) */
/** - 0x01: RAMP (enclosure gradient) */
/** - 0x02: SECTOR (enclosure modifier) */
/** - 0x03: SILHOUETTE (enclosure modifier) */
/** - 0x04: SPLIT (enclosure source) */
/** - 0x05: CELL (enclosure source) */
/** - 0x06: PATCHES (enclosure source) */
/** - 0x07: APERTURE (enclosure modifier) */
/** - 0x08: DECAL (sharp SDF shape) */
/** - 0x09: GRID (repeating lines/panels) */
/** - 0x0A: SCATTER (point field) */
/** - 0x0B: FLOW (animated noise/streaks) */
/** - 0x0C..0x13: radiance opcodes (TRACE/VEIL/ATMOSPHERE/PLANE/CELESTIAL/PORTAL/LOBE_RADIANCE/BAND_RADIANCE) */
/**  */
/** # Blend Modes */
/** - 0: ADD (dst + src * a) */
/** - 1: MULTIPLY (dst * mix(1, src, a)) */
/** - 2: MAX (max(dst, src * a)) */
/** - 3: LERP (mix(dst, src, a)) */
/** - 4: SCREEN (1 - (1-dst)*(1-src*a)) */
/** - 5: HSV_MOD (HSV shift dst by src) */
/** - 6: MIN (min(dst, src * a)) */
/** - 7: OVERLAY (Photoshop-style overlay) */
/**  */
/** Draw the environment background. */
/**  */
/** Renders the procedural environment background for the current viewport and pass. */
/**  */
/** # Usage */
/** Call this **first** in your `render()` function, before any 3D geometry: */
/** ```rust,ignore */
/** fn render() { */
/** // Draw environment background */
/** epu_draw(config.as_ptr()); */
/**  */
/** // Then draw scene geometry */
/** draw_mesh(terrain); */
/** draw_mesh(player); */
/** } */
/** ``` */
/**  */
/** # Notes */
/** - Environment always renders behind all geometry (at far plane) */
/** - For split-screen, set `viewport(...)` and call `epu_draw(...)` per viewport */
/** - The EPU compute pass runs automatically before rendering */
NCZX_IMPORT void epu_draw(const uint64_t* config_ptr);

/** Bind an MRE texture (Metallic-Roughness-Emissive) to slot 1. */
NCZX_IMPORT void material_mre(uint32_t texture);

/** Bind an albedo texture to slot 0. */
NCZX_IMPORT void material_albedo(uint32_t texture);

/** Bind a normal map texture to slot 3. */
/**  */
/** # Arguments */
/** * `texture` — Handle to a BC5 or RGBA normal map texture */
/**  */
/** Normal maps perturb surface normals for detailed lighting without extra geometry. */
/** Requires mesh with tangent data (FORMAT_TANGENT) and UVs. */
/** Works in all lit modes (0=Lambert, 2=PBR, 3=Hybrid) and Mode 1 (Matcap). */
NCZX_IMPORT void material_normal(uint32_t texture);

/** Skip normal map sampling (use vertex normal instead). */
/**  */
/** # Arguments */
/** * `skip` — 1 to skip normal map, 0 to use normal map (default) */
/**  */
/** When a mesh has tangent data, normal mapping is enabled by default. */
/** Use this flag to opt out temporarily for debugging or artistic control. */
NCZX_IMPORT void skip_normal_map(uint32_t skip);

/** Set material metallic value (0.0 = dielectric, 1.0 = metal). */
NCZX_IMPORT void material_metallic(float value);

/** Set material roughness value (0.0 = smooth, 1.0 = rough). */
NCZX_IMPORT void material_roughness(float value);

/** Set material emissive intensity (0.0 = no emission, >1.0 for HDR). */
NCZX_IMPORT void material_emissive(float value);

/** Set rim lighting parameters. */
/**  */
/** # Arguments */
/** * `intensity` — Rim brightness (0.0-1.0) */
/** * `power` — Falloff sharpness (0.0-32.0, higher = tighter) */
NCZX_IMPORT void material_rim(float intensity, float power);

/** Enable/disable uniform color override. */
/**  */
/** When enabled, uses the last set_color() value for all subsequent draws, */
/** overriding vertex colors and material albedo. */
/**  */
/** # Arguments */
/** * `enabled` — 1 to enable, 0 to disable */
NCZX_IMPORT void use_uniform_color(uint32_t enabled);

/** Enable/disable uniform metallic override. */
/**  */
/** When enabled, uses the last material_metallic() value for all subsequent draws, */
/** overriding per-vertex or per-material metallic values. */
/**  */
/** # Arguments */
/** * `enabled` — 1 to enable, 0 to disable */
NCZX_IMPORT void use_uniform_metallic(uint32_t enabled);

/** Enable/disable uniform roughness override. */
/**  */
/** When enabled, uses the last material_roughness() value for all subsequent draws, */
/** overriding per-vertex or per-material roughness values. */
/**  */
/** # Arguments */
/** * `enabled` — 1 to enable, 0 to disable */
NCZX_IMPORT void use_uniform_roughness(uint32_t enabled);

/** Enable/disable uniform emissive override. */
/**  */
/** When enabled, uses the last material_emissive() value for all subsequent draws, */
/** overriding per-vertex or per-material emissive values. */
/**  */
/** # Arguments */
/** * `enabled` — 1 to enable, 0 to disable */
NCZX_IMPORT void use_uniform_emissive(uint32_t enabled);

/** Set shininess (Mode 3 alias for roughness). */
NCZX_IMPORT void material_shininess(float value);

/** Set specular color (Mode 3 only). */
/**  */
/** # Arguments */
/** * `color` — Specular color (0xRRGGBBAA, alpha ignored) */
NCZX_IMPORT void material_specular(uint32_t color);

/** Set light direction (and enable the light). */
/**  */
/** # Arguments */
/** * `index` — Light index (0-3) */
/** * `x`, `y`, `z` — Direction rays travel (from light toward surface) */
/**  */
/** For a light from above, use (0, -1, 0). */
NCZX_IMPORT void light_set(uint32_t index, float x, float y, float z);

/** Set light color. */
/**  */
/** # Arguments */
/** * `color` — Light color (0xRRGGBBAA, alpha ignored) */
NCZX_IMPORT void light_color(uint32_t index, uint32_t color);

/** Set light intensity multiplier. */
/**  */
/** # Arguments */
/** * `intensity` — Typically 0.0-10.0 */
NCZX_IMPORT void light_intensity(uint32_t index, float intensity);

/** Enable a light. */
NCZX_IMPORT void light_enable(uint32_t index);

/** Disable a light (preserves settings for re-enabling). */
NCZX_IMPORT void light_disable(uint32_t index);

/** Convert a light to a point light at world position. */
/**  */
/** # Arguments */
/** * `index` — Light index (0-3) */
/** * `x`, `y`, `z` — World-space position */
/**  */
/** Enables the light automatically. Default range is 10.0 units. */
NCZX_IMPORT void light_set_point(uint32_t index, float x, float y, float z);

/** Set point light falloff distance. */
/**  */
/** # Arguments */
/** * `index` — Light index (0-3) */
/** * `range` — Distance at which light reaches zero intensity */
/**  */
/** Only affects point lights (ignored for directional). */
NCZX_IMPORT void light_range(uint32_t index, float range);

/** Load a skeleton's inverse bind matrices to GPU. */
/**  */
/** Call once during `init()` after loading skinned meshes. */
/** The inverse bind matrices transform vertices from model space */
/** to bone-local space at bind time. */
/**  */
/** # Arguments */
/** * `inverse_bind_ptr` — Pointer to array of 3×4 matrices (12 floats per bone, column-major) */
/** * `bone_count` — Number of bones (max 256) */
/**  */
/** # Returns */
/** Skeleton handle (>0) on success, 0 on error. */
NCZX_IMPORT uint32_t load_skeleton(const float* inverse_bind_ptr, uint32_t bone_count);

/** Bind a skeleton for subsequent skinned mesh rendering. */
/**  */
/** When bound, `set_bones()` expects model-space transforms and the GPU */
/** automatically applies the inverse bind matrices. */
/**  */
/** # Arguments */
/** * `skeleton` — Skeleton handle from `load_skeleton()`, or 0 to unbind (raw mode) */
/**  */
/** # Behavior */
/** - skeleton > 0: Enable inverse bind mode. `set_bones()` receives model transforms. */
/** - skeleton = 0: Disable inverse bind mode (raw). `set_bones()` receives final matrices. */
NCZX_IMPORT void skeleton_bind(uint32_t skeleton);

/** Set bone transform matrices for skeletal animation. */
/**  */
/** # Arguments */
/** * `matrices_ptr` — Pointer to array of 3×4 matrices (12 floats per bone, column-major) */
/** * `count` — Number of bones (max 256) */
/**  */
/** Each bone matrix is 12 floats in column-major order: */
/** ```text */
/** [col0.x, col0.y, col0.z]  // X axis */
/** [col1.x, col1.y, col1.z]  // Y axis */
/** [col2.x, col2.y, col2.z]  // Z axis */
/** [tx,     ty,     tz    ]  // translation */
/** // implicit 4th row [0, 0, 0, 1] */
/** ``` */
NCZX_IMPORT void set_bones(const float* matrices_ptr, uint32_t count);

/** Set bone transform matrices for skeletal animation using 4x4 matrices. */
/**  */
/** Alternative to `set_bones()` that accepts full 4x4 matrices instead of 3x4. */
/**  */
/** # Arguments */
/** * `matrices_ptr` — Pointer to array of 4×4 matrices (16 floats per bone, column-major) */
/** * `count` — Number of bones (max 256) */
/**  */
/** Each bone matrix is 16 floats in column-major order: */
/** ```text */
/** [col0.x, col0.y, col0.z, col0.w]  // X axis + w */
/** [col1.x, col1.y, col1.z, col1.w]  // Y axis + w */
/** [col2.x, col2.y, col2.z, col2.w]  // Z axis + w */
/** [tx,     ty,     tz,     tw    ]  // translation + w */
/** ``` */
NCZX_IMPORT void set_bones_4x4(const float* matrices_ptr, uint32_t count);

/** Load keyframe animation data from WASM memory. */
/**  */
/** Must be called during `init()`. */
/**  */
/** # Arguments */
/** * `data_ptr` — Pointer to .nczxanim data in WASM memory */
/** * `byte_size` — Total size of the data in bytes */
/**  */
/** # Returns */
/** Keyframe collection handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t keyframes_load(const uint8_t* data_ptr, uint32_t byte_size);

/** Load keyframe animation data from ROM data pack by ID. */
/**  */
/** Must be called during `init()`. */
/**  */
/** # Arguments */
/** * `id_ptr` — Pointer to asset ID string in WASM memory */
/** * `id_len` — Length of asset ID string */
/**  */
/** # Returns */
/** Keyframe collection handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_keyframes(const uint8_t* id_ptr, uint32_t id_len);

/** Get the bone count for a keyframe collection. */
/**  */
/** # Arguments */
/** * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes() */
/**  */
/** # Returns */
/** Bone count (0 on invalid handle) */
NCZX_IMPORT uint32_t keyframes_bone_count(uint32_t handle);

/** Get the frame count for a keyframe collection. */
/**  */
/** # Arguments */
/** * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes() */
/**  */
/** # Returns */
/** Frame count (0 on invalid handle) */
NCZX_IMPORT uint32_t keyframes_frame_count(uint32_t handle);

/** Read a decoded keyframe into WASM memory. */
/**  */
/** Decodes the platform format to BoneTransform format (40 bytes/bone): */
/** - rotation: [f32; 4] quaternion [x, y, z, w] */
/** - position: [f32; 3] */
/** - scale: [f32; 3] */
/**  */
/** # Arguments */
/** * `handle` — Keyframe collection handle */
/** * `index` — Frame index (0-based) */
/** * `out_ptr` — Pointer to output buffer in WASM memory (must be bone_count × 40 bytes) */
/**  */
/** # Traps */
/** - Invalid handle (0 or not loaded) */
/** - Frame index out of bounds */
/** - Output buffer out of bounds */
NCZX_IMPORT void keyframe_read(uint32_t handle, uint32_t index, uint8_t* out_ptr);

/** Bind a keyframe directly from the static GPU buffer. */
/**  */
/** Points subsequent skinned draws to use pre-decoded matrices from the GPU buffer. */
/** No CPU decoding or data transfer needed at draw time. */
/**  */
/** # Arguments */
/** * `handle` — Keyframe collection handle (0 to unbind) */
/** * `index` — Frame index (0-based) */
/**  */
/** # Traps */
/** - Invalid handle (not loaded) */
/** - Frame index out of bounds */
NCZX_IMPORT void keyframe_bind(uint32_t handle, uint32_t index);

/** Load raw PCM sound data (22.05kHz, 16-bit signed, mono). */
/**  */
/** Must be called during `init()`. */
/**  */
/** # Arguments */
/** * `data_ptr` — Pointer to i16 PCM samples */
/** * `byte_len` — Length in bytes (must be even) */
/**  */
/** # Returns */
/** Sound handle for use with playback functions. */
NCZX_IMPORT uint32_t load_sound(const int16_t* data_ptr, uint32_t byte_len);

/** Play sound on next available channel (fire-and-forget). */
/**  */
/** # Arguments */
/** * `volume` — 0.0 to 1.0 */
/** * `pan` — -1.0 (left) to 1.0 (right), 0.0 = center */
NCZX_IMPORT void play_sound(uint32_t sound, float volume, float pan);

/** Play sound on a specific channel (for managed/looping audio). */
/**  */
/** # Arguments */
/** * `channel` — Channel index (0-15) */
/** * `looping` — 1 = loop, 0 = play once */
NCZX_IMPORT void channel_play(uint32_t channel, uint32_t sound, float volume, float pan, uint32_t looping);

/** Update channel parameters (call every frame for positional audio). */
NCZX_IMPORT void channel_set(uint32_t channel, float volume, float pan);

/** Stop a channel. */
NCZX_IMPORT void channel_stop(uint32_t channel);

/** Load a tracker module from ROM data pack by ID. */
/**  */
/** Must be called during `init()`. */
/** Returns a handle with bit 31 set (tracker handle). */
/**  */
/** # Arguments */
/** * `id_ptr` — Pointer to tracker ID string */
/** * `id_len` — Length of tracker ID string */
/**  */
/** # Returns */
/** Tracker handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t rom_tracker(const uint8_t* id_ptr, uint32_t id_len);

/** Load a tracker module from raw XM data. */
/**  */
/** Must be called during `init()`. */
/** Returns a handle with bit 31 set (tracker handle). */
/**  */
/** # Arguments */
/** * `data_ptr` — Pointer to XM file data */
/** * `data_len` — Length of XM data in bytes */
/**  */
/** # Returns */
/** Tracker handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_tracker(const uint8_t* data_ptr, uint32_t data_len);

/** Play music (PCM sound or tracker module). */
/**  */
/** Automatically stops any currently playing music of the other type. */
/** Handle type is detected by bit 31 (0=PCM, 1=tracker). */
/**  */
/** # Arguments */
/** * `handle` — Sound handle (from load_sound) or tracker handle (from rom_tracker) */
/** * `volume` — 0.0 to 1.0 */
/** * `looping` — 1 = loop, 0 = play once */
NCZX_IMPORT void music_play(uint32_t handle, float volume, uint32_t looping);

/** Stop music (both PCM and tracker). */
NCZX_IMPORT void music_stop(void);

/** Pause or resume music (tracker only, no-op for PCM). */
/**  */
/** # Arguments */
/** * `paused` — 1 = pause, 0 = resume */
NCZX_IMPORT void music_pause(uint32_t paused);

/** Set music volume (works for both PCM and tracker). */
/**  */
/** # Arguments */
/** * `volume` — 0.0 to 1.0 */
NCZX_IMPORT void music_set_volume(float volume);

/** Check if music is currently playing. */
/**  */
/** # Returns */
/** 1 if playing (and not paused), 0 otherwise. */
NCZX_IMPORT uint32_t music_is_playing(void);

/** Get current music type. */
/**  */
/** # Returns */
/** 0 = none, 1 = PCM, 2 = tracker */
NCZX_IMPORT uint32_t music_type(void);

/** Jump to a specific position (tracker only, no-op for PCM). */
/**  */
/** Use for dynamic music systems (e.g., jump to outro pattern). */
/**  */
/** # Arguments */
/** * `order` — Order position (0-based) */
/** * `row` — Row within the pattern (0-based) */
NCZX_IMPORT void music_jump(uint32_t order, uint32_t row);

/** Get current music position. */
/**  */
/** For tracker: (order << 16) | row */
/** For PCM: sample position */
/**  */
/** # Returns */
/** Position value (format depends on music type). */
NCZX_IMPORT uint32_t music_position(void);

/** Get music length. */
/**  */
/** For tracker: number of orders in the song. */
/** For PCM: number of samples. */
/**  */
/** # Arguments */
/** * `handle` — Music handle (PCM or tracker) */
/**  */
/** # Returns */
/** Length value. */
NCZX_IMPORT uint32_t music_length(uint32_t handle);

/** Set music speed (tracker only, ticks per row). */
/**  */
/** # Arguments */
/** * `speed` — 1-31 (XM default is 6) */
NCZX_IMPORT void music_set_speed(uint32_t speed);

/** Set music tempo (tracker only, BPM). */
/**  */
/** # Arguments */
/** * `bpm` — 32-255 (XM default is 125) */
NCZX_IMPORT void music_set_tempo(uint32_t bpm);

/** Get music info. */
/**  */
/** For tracker: (num_channels << 24) | (num_patterns << 16) | (num_instruments << 8) | song_length */
/** For PCM: (sample_rate << 16) | (channels << 8) | bits_per_sample */
/**  */
/** # Arguments */
/** * `handle` — Music handle (PCM or tracker) */
/**  */
/** # Returns */
/** Packed info value. */
NCZX_IMPORT uint32_t music_info(uint32_t handle);

/** Get music name (tracker only, returns 0 for PCM). */
/**  */
/** # Arguments */
/** * `handle` — Music handle */
/** * `out_ptr` — Pointer to output buffer */
/** * `max_len` — Maximum bytes to write */
/**  */
/** # Returns */
/** Actual length written (0 if PCM or invalid handle). */
NCZX_IMPORT uint32_t music_name(uint32_t handle, uint8_t* out_ptr, uint32_t max_len);

/** Load a texture from ROM data pack by ID. */
/**  */
/** # Arguments */
/** * `id_ptr` — Pointer to asset ID string in WASM memory */
/** * `id_len` — Length of asset ID string */
/**  */
/** # Returns */
/** Texture handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_texture(const uint8_t* id_ptr, uint32_t id_len);

/** Load a mesh from ROM data pack by ID. */
/**  */
/** # Returns */
/** Mesh handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_mesh(const uint8_t* id_ptr, uint32_t id_len);

/** Load skeleton inverse bind matrices from ROM data pack by ID. */
/**  */
/** # Returns */
/** Skeleton handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_skeleton(const uint8_t* id_ptr, uint32_t id_len);

/** Load a font atlas from ROM data pack by ID. */
/**  */
/** # Returns */
/** Texture handle for font atlas (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_font(const uint8_t* id_ptr, uint32_t id_len);

/** Load a sound from ROM data pack by ID. */
/**  */
/** # Returns */
/** Sound handle (>0) on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_sound(const uint8_t* id_ptr, uint32_t id_len);

/** Get the byte size of raw data in the ROM data pack. */
/**  */
/** Use this to allocate a buffer before calling `rom_data()`. */
/**  */
/** # Returns */
/** Byte count on success. Traps if not found. */
NCZX_IMPORT uint32_t rom_data_len(const uint8_t* id_ptr, uint32_t id_len);

/** Copy raw data from ROM data pack into WASM linear memory. */
/**  */
/** # Arguments */
/** * `id_ptr`, `id_len` — Asset ID string */
/** * `dst_ptr` — Pointer to destination buffer in WASM memory */
/** * `max_len` — Maximum bytes to copy (size of destination buffer) */
/**  */
/** # Returns */
/** Bytes written on success. Traps on failure. */
NCZX_IMPORT uint32_t rom_data(const uint8_t* id_ptr, uint32_t id_len, const uint8_t* dst_ptr, uint32_t max_len);

/** Load a mesh from .nczxmesh binary format. */
/**  */
/** # Arguments */
/** * `data_ptr` — Pointer to .nczxmesh binary data */
/** * `data_len` — Length of the data in bytes */
/**  */
/** # Returns */
/** Mesh handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_zmesh(const uint8_t* data_ptr, uint32_t data_len);

/** Load a texture from .nczxtex binary format. */
/**  */
/** # Returns */
/** Texture handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_ztex(const uint8_t* data_ptr, uint32_t data_len);

/** Load a sound from .nczxsnd binary format. */
/**  */
/** # Returns */
/** Sound handle (>0) on success, 0 on failure. */
NCZX_IMPORT uint32_t load_zsound(const uint8_t* data_ptr, uint32_t data_len);

/** Register an i8 value for debug inspection. */
NCZX_IMPORT void debug_register_i8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register an i16 value for debug inspection. */
NCZX_IMPORT void debug_register_i16(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register an i32 value for debug inspection. */
NCZX_IMPORT void debug_register_i32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a u8 value for debug inspection. */
NCZX_IMPORT void debug_register_u8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a u16 value for debug inspection. */
NCZX_IMPORT void debug_register_u16(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a u32 value for debug inspection. */
NCZX_IMPORT void debug_register_u32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register an f32 value for debug inspection. */
NCZX_IMPORT void debug_register_f32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a bool value for debug inspection. */
NCZX_IMPORT void debug_register_bool(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register an i32 with min/max range constraints. */
NCZX_IMPORT void debug_register_i32_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, int32_t min, int32_t max);

/** Register an f32 with min/max range constraints. */
NCZX_IMPORT void debug_register_f32_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, float min, float max);

/** Register a u8 with min/max range constraints. */
NCZX_IMPORT void debug_register_u8_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, uint32_t min, uint32_t max);

/** Register a u16 with min/max range constraints. */
NCZX_IMPORT void debug_register_u16_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, uint32_t min, uint32_t max);

/** Register an i16 with min/max range constraints. */
NCZX_IMPORT void debug_register_i16_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, int32_t min, int32_t max);

/** Register a Vec2 (2 floats: x, y) for debug inspection. */
NCZX_IMPORT void debug_register_vec2(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a Vec3 (3 floats: x, y, z) for debug inspection. */
NCZX_IMPORT void debug_register_vec3(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a Rect (4 i16: x, y, w, h) for debug inspection. */
NCZX_IMPORT void debug_register_rect(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register a Color (4 u8: RGBA) for debug inspection with color picker. */
NCZX_IMPORT void debug_register_color(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register Q8.8 fixed-point (i16) for debug inspection. */
NCZX_IMPORT void debug_register_fixed_i16_q8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register Q16.16 fixed-point (i32) for debug inspection. */
NCZX_IMPORT void debug_register_fixed_i32_q16(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register Q24.8 fixed-point (i32) for debug inspection. */
NCZX_IMPORT void debug_register_fixed_i32_q8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Register Q8.24 fixed-point (i32) for debug inspection. */
NCZX_IMPORT void debug_register_fixed_i32_q24(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch an i8 value (read-only). */
NCZX_IMPORT void debug_watch_i8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch an i16 value (read-only). */
NCZX_IMPORT void debug_watch_i16(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch an i32 value (read-only). */
NCZX_IMPORT void debug_watch_i32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a u8 value (read-only). */
NCZX_IMPORT void debug_watch_u8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a u16 value (read-only). */
NCZX_IMPORT void debug_watch_u16(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a u32 value (read-only). */
NCZX_IMPORT void debug_watch_u32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch an f32 value (read-only). */
NCZX_IMPORT void debug_watch_f32(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a bool value (read-only). */
NCZX_IMPORT void debug_watch_bool(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a Vec2 value (read-only). */
NCZX_IMPORT void debug_watch_vec2(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a Vec3 value (read-only). */
NCZX_IMPORT void debug_watch_vec3(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a Rect value (read-only). */
NCZX_IMPORT void debug_watch_rect(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Watch a Color value (read-only). */
NCZX_IMPORT void debug_watch_color(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);

/** Begin a collapsible group in the debug UI. */
NCZX_IMPORT void debug_group_begin(const uint8_t* name_ptr, uint32_t name_len);

/** End the current debug group. */
NCZX_IMPORT void debug_group_end(void);

/** Register a simple action with no parameters. */
/**  */
/** Creates a button in the debug UI that calls the specified WASM function when clicked. */
/**  */
/** # Parameters */
/** - `name_ptr`: Pointer to button label string */
/** - `name_len`: Length of button label */
/** - `func_name_ptr`: Pointer to WASM function name string */
/** - `func_name_len`: Length of function name */
NCZX_IMPORT void debug_register_action(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* func_name_ptr, uint32_t func_name_len);

/** Begin building an action with parameters. */
/**  */
/** Use with debug_action_param_* and debug_action_end() to create an action with input fields. */
/**  */
/** # Parameters */
/** - `name_ptr`: Pointer to button label string */
/** - `name_len`: Length of button label */
/** - `func_name_ptr`: Pointer to WASM function name string */
/** - `func_name_len`: Length of function name */
NCZX_IMPORT void debug_action_begin(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* func_name_ptr, uint32_t func_name_len);

/** Add an i32 parameter to the pending action. */
/**  */
/** # Parameters */
/** - `name_ptr`: Pointer to parameter label string */
/** - `name_len`: Length of parameter label */
/** - `default_value`: Default value for the parameter */
NCZX_IMPORT void debug_action_param_i32(const uint8_t* name_ptr, uint32_t name_len, int32_t default_value);

/** Add an f32 parameter to the pending action. */
/**  */
/** # Parameters */
/** - `name_ptr`: Pointer to parameter label string */
/** - `name_len`: Length of parameter label */
/** - `default_value`: Default value for the parameter */
NCZX_IMPORT void debug_action_param_f32(const uint8_t* name_ptr, uint32_t name_len, float default_value);

/** Finish building the pending action. */
/**  */
/** Completes the action registration started with debug_action_begin(). */
NCZX_IMPORT void debug_action_end(void);

/** Query if the game is currently paused (debug mode). */
/**  */
/** # Returns */
/** 1 if paused, 0 if running normally. */
NCZX_IMPORT int32_t debug_is_paused(void);

/** Get the current time scale multiplier. */
/**  */
/** # Returns */
/** 1.0 = normal, 0.5 = half-speed, 2.0 = double-speed, etc. */
NCZX_IMPORT float debug_get_time_scale(void);

// =============================================================================
// Constants
// =============================================================================

// button constants
#define NCZX_BUTTON_UP 0
#define NCZX_BUTTON_DOWN 1
#define NCZX_BUTTON_LEFT 2
#define NCZX_BUTTON_RIGHT 3
#define NCZX_BUTTON_A 4
#define NCZX_BUTTON_B 5
#define NCZX_BUTTON_X 6
#define NCZX_BUTTON_Y 7
#define NCZX_BUTTON_L1 8
#define NCZX_BUTTON_R1 9
#define NCZX_BUTTON_L3 10
#define NCZX_BUTTON_R3 11
#define NCZX_BUTTON_START 12
#define NCZX_BUTTON_SELECT 13

// cull constants
#define NCZX_CULL_NONE 0
#define NCZX_CULL_BACK 1
#define NCZX_CULL_FRONT 2

// format constants
#define NCZX_FORMAT_POS 0
#define NCZX_FORMAT_UV 1
#define NCZX_FORMAT_COLOR 2
#define NCZX_FORMAT_NORMAL 4
#define NCZX_FORMAT_SKINNED 8
#define NCZX_FORMAT_TANGENT 16
#define NCZX_FORMAT_POS_UV UV
#define NCZX_FORMAT_POS_COLOR COLOR
#define NCZX_FORMAT_POS_NORMAL NORMAL
#define NCZX_FORMAT_POS_UV_NORMAL UV | NORMAL
#define NCZX_FORMAT_POS_UV_COLOR UV | COLOR
#define NCZX_FORMAT_POS_UV_COLOR_NORMAL UV | COLOR | NORMAL
#define NCZX_FORMAT_POS_SKINNED SKINNED
#define NCZX_FORMAT_POS_NORMAL_SKINNED NORMAL | SKINNED
#define NCZX_FORMAT_POS_UV_NORMAL_SKINNED UV | NORMAL | SKINNED
#define NCZX_FORMAT_POS_UV_NORMAL_TANGENT UV | NORMAL | TANGENT
#define NCZX_FORMAT_POS_UV_COLOR_NORMAL_TANGENT UV | COLOR | NORMAL | TANGENT

// billboard constants
#define NCZX_BILLBOARD_SPHERICAL 1
#define NCZX_BILLBOARD_CYLINDRICAL_Y 2
#define NCZX_BILLBOARD_CYLINDRICAL_X 3
#define NCZX_BILLBOARD_CYLINDRICAL_Z 4

// screen constants
#define NCZX_SCREEN_WIDTH 960
#define NCZX_SCREEN_HEIGHT 540

// compare constants
#define NCZX_COMPARE_NEVER 1
#define NCZX_COMPARE_LESS 2
#define NCZX_COMPARE_EQUAL 3
#define NCZX_COMPARE_LESS_EQUAL 4
#define NCZX_COMPARE_GREATER 5
#define NCZX_COMPARE_NOT_EQUAL 6
#define NCZX_COMPARE_GREATER_EQUAL 7
#define NCZX_COMPARE_ALWAYS 8

// stencil_op constants
#define NCZX_STENCIL_OP_KEEP 0
#define NCZX_STENCIL_OP_ZERO 1
#define NCZX_STENCIL_OP_REPLACE 2
#define NCZX_STENCIL_OP_INCREMENT_CLAMP 3
#define NCZX_STENCIL_OP_DECREMENT_CLAMP 4
#define NCZX_STENCIL_OP_INVERT 5
#define NCZX_STENCIL_OP_INCREMENT_WRAP 6
#define NCZX_STENCIL_OP_DECREMENT_WRAP 7

// color constants
#define NCZX_COLOR_WHITE 0xFFFFFFFF
#define NCZX_COLOR_BLACK 0x000000FF
#define NCZX_COLOR_RED 0xFF0000FF
#define NCZX_COLOR_GREEN 0x00FF00FF
#define NCZX_COLOR_BLUE 0x0000FFFF
#define NCZX_COLOR_YELLOW 0xFFFF00FF
#define NCZX_COLOR_CYAN 0x00FFFFFF
#define NCZX_COLOR_MAGENTA 0xFF00FFFF
#define NCZX_COLOR_ORANGE 0xFF8000FF
#define NCZX_COLOR_TRANSPARENT 0x00000000

#ifdef __cplusplus
}
#endif


// =============================================================================
// MANUALLY MAINTAINED HELPER FUNCTIONS
// =============================================================================
// These helpers provide language-specific conveniences for C/C++ developers

// Color packing helpers
static inline uint32_t nczx_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    return ((uint32_t)r << 24) | ((uint32_t)g << 16) | ((uint32_t)b << 8) | (uint32_t)a;
}

static inline uint32_t nczx_rgb(uint8_t r, uint8_t g, uint8_t b) {
    return nczx_rgba(r, g, b, 255);
}

// Math helpers
static inline float nczx_clampf(float val, float min, float max) {
    return (val < min) ? min : ((val > max) ? max : val);
}

static inline float nczx_lerpf(float a, float b, float t) {
    return a + (b - a) * t;
}

static inline float nczx_minf(float a, float b) {
    return (a < b) ? a : b;
}

static inline float nczx_maxf(float a, float b) {
    return (a > b) ? a : b;
}

static inline float nczx_absf(float x) {
    return (x < 0.0f) ? -x : x;
}

// String literal helpers (use sizeof() for compile-time length calculation)
#define NCZX_LOG(str) log((const uint8_t*)(str), sizeof(str) - 1)

#define NCZX_DRAW_TEXT(str, x, y, size, color) \
    draw_text((const uint8_t*)(str), sizeof(str) - 1, (x), (y), (size), (color))

// ROM loading helpers
#define NCZX_ROM_TEXTURE(id) rom_texture((uint32_t)(id), sizeof(id) - 1)
#define NCZX_ROM_MESH(id) rom_mesh((uint32_t)(id), sizeof(id) - 1)
#define NCZX_ROM_SOUND(id) rom_sound((uint32_t)(id), sizeof(id) - 1)
#define NCZX_ROM_FONT(id) rom_font((uint32_t)(id), sizeof(id) - 1)
#define NCZX_ROM_SKELETON(id) rom_skeleton((uint32_t)(id), sizeof(id) - 1)

#endif /* NETHERCORE_ZX_H */
