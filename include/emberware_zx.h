/**
 * Emberware ZX FFI Bindings for C/C++
 *
 * This header provides all FFI function declarations for Emberware ZX games.
 * Include this file and implement init(), update(), and render().
 *
 * Usage:
 *   #include "emberware_zx.h"
 *
 *   EWZX_EXPORT void init(void) {
 *       set_clear_color(0x1a1a2eFF);
 *       render_mode(EWZX_RENDER_PBR);
 *   }
 *
 *   EWZX_EXPORT void update(void) {
 *       // Game logic here
 *   }
 *
 *   EWZX_EXPORT void render(void) {
 *       draw_sky();
 *       // Draw your scene
 *   }
 *
 * Build with wasi-sdk:
 *   clang --target=wasm32-wasi -O2 -Wl,--no-entry \
 *         -Wl,--export=init -Wl,--export=update -Wl,--export=render \
 *         -Wl,--allow-undefined -o game.wasm game.c
 */

#ifndef EMBERWARE_ZX_H
#define EMBERWARE_ZX_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* =============================================================================
 * WASM ATTRIBUTES
 * ============================================================================= */

/** Mark functions for export from WASM module */
#ifdef __wasm__
  #define EWZX_EXPORT __attribute__((visibility("default")))
#else
  #define EWZX_EXPORT
#endif

/** Mark functions imported from host environment */
#define EWZX_IMPORT __attribute__((import_module("env")))

/* =============================================================================
 * SYSTEM FUNCTIONS
 * ============================================================================= */

/**
 * Returns the fixed timestep duration in seconds.
 * This is a CONSTANT based on tick rate, NOT wall-clock time.
 * - 60fps = 0.01666... (1/60)
 * - 30fps = 0.03333... (1/30)
 * Safe for rollback netcode.
 */
EWZX_IMPORT float delta_time(void);

/**
 * Returns total elapsed game time since start in seconds.
 * Accumulated fixed timestep, NOT wall-clock. Equals tick_count * delta_time.
 * Safe for rollback netcode.
 */
EWZX_IMPORT float elapsed_time(void);

/**
 * Returns the current tick number (starts at 0, increments each update).
 * Deterministic and safe for rollback netcode.
 */
EWZX_IMPORT uint64_t tick_count(void);

/**
 * Logs a message to the console output.
 * @param ptr  Pointer to UTF-8 string data
 * @param len  Length of string in bytes
 */
EWZX_IMPORT void log_msg(const uint8_t* ptr, uint32_t len);

/** Exits the game and returns to the library. */
EWZX_IMPORT void quit(void);

/**
 * Returns a deterministic random u32 from the host's seeded RNG.
 * Always use this instead of rand() for rollback compatibility.
 */
EWZX_IMPORT uint32_t random_u32(void);

/* =============================================================================
 * SESSION FUNCTIONS
 * ============================================================================= */

/** Returns the number of players in the session (1-4). */
EWZX_IMPORT uint32_t player_count(void);

/**
 * Returns a bitmask of which players are local to this client.
 * Example: (local_player_mask() & (1 << player_id)) != 0
 */
EWZX_IMPORT uint32_t local_player_mask(void);

/* =============================================================================
 * SAVE DATA FUNCTIONS
 * ============================================================================= */

/**
 * Saves data to a slot.
 * @param slot      Save slot (0-7)
 * @param data_ptr  Pointer to data
 * @param data_len  Length in bytes (max 64KB)
 * @return 0 on success, 1 if invalid slot, 2 if data too large
 */
EWZX_IMPORT uint32_t save(uint32_t slot, const uint8_t* data_ptr, uint32_t data_len);

/**
 * Loads data from a slot.
 * @param slot      Save slot (0-7)
 * @param data_ptr  Pointer to destination buffer
 * @param max_len   Maximum bytes to read
 * @return Bytes read (0 if empty or error)
 */
EWZX_IMPORT uint32_t load(uint32_t slot, uint8_t* data_ptr, uint32_t max_len);

/**
 * Deletes a save slot.
 * @return 0 on success, 1 if invalid slot
 */
EWZX_IMPORT uint32_t delete_save(uint32_t slot);

/* =============================================================================
 * CONFIGURATION FUNCTIONS (init-only)
 * ============================================================================= */

/**
 * Set the tick rate. Must be called during init().
 * @param rate  Tick rate index: 0=24fps, 1=30fps, 2=60fps (default), 3=120fps
 */
EWZX_IMPORT void set_tick_rate(uint32_t rate);

/**
 * Set the clear/background color. Must be called during init().
 * @param color  Color in 0xRRGGBBAA format (default: black)
 */
EWZX_IMPORT void set_clear_color(uint32_t color);

/**
 * Set the render mode. Must be called during init().
 * @param mode  0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid
 */
EWZX_IMPORT void render_mode(uint32_t mode);

/* =============================================================================
 * CAMERA FUNCTIONS
 * ============================================================================= */

/**
 * Set the camera position and target (look-at point).
 * Uses a Y-up, right-handed coordinate system.
 */
EWZX_IMPORT void camera_set(float x, float y, float z,
                            float target_x, float target_y, float target_z);

/**
 * Set the camera field of view.
 * @param fov_degrees  Field of view in degrees (typically 45-90, default 60)
 */
EWZX_IMPORT void camera_fov(float fov_degrees);

/** Push a custom view matrix (16 floats, column-major order). */
EWZX_IMPORT void push_view_matrix(
    float m0, float m1, float m2, float m3,
    float m4, float m5, float m6, float m7,
    float m8, float m9, float m10, float m11,
    float m12, float m13, float m14, float m15
);

/** Push a custom projection matrix (16 floats, column-major order). */
EWZX_IMPORT void push_projection_matrix(
    float m0, float m1, float m2, float m3,
    float m4, float m5, float m6, float m7,
    float m8, float m9, float m10, float m11,
    float m12, float m13, float m14, float m15
);

/* =============================================================================
 * TRANSFORM FUNCTIONS
 * ============================================================================= */

/** Push identity matrix onto the transform stack. */
EWZX_IMPORT void push_identity(void);

/** Set the current transform from a 4x4 matrix pointer (16 floats, column-major). */
EWZX_IMPORT void transform_set(const float* matrix_ptr);

/** Push a translation transform. */
EWZX_IMPORT void push_translate(float x, float y, float z);

/** Push a rotation around the X axis (angle in degrees). */
EWZX_IMPORT void push_rotate_x(float angle_deg);

/** Push a rotation around the Y axis (angle in degrees). */
EWZX_IMPORT void push_rotate_y(float angle_deg);

/** Push a rotation around the Z axis (angle in degrees). */
EWZX_IMPORT void push_rotate_z(float angle_deg);

/**
 * Push a rotation around an arbitrary axis.
 * @param angle_deg  Rotation angle in degrees
 * @param axis_x, axis_y, axis_z  Rotation axis (will be normalized)
 */
EWZX_IMPORT void push_rotate(float angle_deg, float axis_x, float axis_y, float axis_z);

/** Push a non-uniform scale transform. */
EWZX_IMPORT void push_scale(float x, float y, float z);

/** Push a uniform scale transform. */
EWZX_IMPORT void push_scale_uniform(float s);

/* =============================================================================
 * INPUT FUNCTIONS - BUTTONS
 * ============================================================================= */

/**
 * Check if a button is currently held.
 * @param player  Player index (0-3)
 * @param button  Button index (see EWZX_BUTTON_* constants)
 * @return 1 if held, 0 otherwise
 */
EWZX_IMPORT uint32_t button_held(uint32_t player, uint32_t button);

/**
 * Check if a button was just pressed this tick.
 * @return 1 if just pressed, 0 otherwise
 */
EWZX_IMPORT uint32_t button_pressed(uint32_t player, uint32_t button);

/**
 * Check if a button was just released this tick.
 * @return 1 if just released, 0 otherwise
 */
EWZX_IMPORT uint32_t button_released(uint32_t player, uint32_t button);

/** Get bitmask of all held buttons for a player. */
EWZX_IMPORT uint32_t buttons_held(uint32_t player);

/** Get bitmask of all buttons just pressed this tick. */
EWZX_IMPORT uint32_t buttons_pressed(uint32_t player);

/** Get bitmask of all buttons just released this tick. */
EWZX_IMPORT uint32_t buttons_released(uint32_t player);

/* =============================================================================
 * INPUT FUNCTIONS - ANALOG STICKS
 * ============================================================================= */

/** Get left stick X axis value (-1.0 to 1.0). */
EWZX_IMPORT float left_stick_x(uint32_t player);

/** Get left stick Y axis value (-1.0 to 1.0). */
EWZX_IMPORT float left_stick_y(uint32_t player);

/** Get right stick X axis value (-1.0 to 1.0). */
EWZX_IMPORT float right_stick_x(uint32_t player);

/** Get right stick Y axis value (-1.0 to 1.0). */
EWZX_IMPORT float right_stick_y(uint32_t player);

/** Get both left stick axes at once (more efficient). */
EWZX_IMPORT void left_stick(uint32_t player, float* out_x, float* out_y);

/** Get both right stick axes at once (more efficient). */
EWZX_IMPORT void right_stick(uint32_t player, float* out_x, float* out_y);

/* =============================================================================
 * INPUT FUNCTIONS - TRIGGERS
 * ============================================================================= */

/** Get left trigger value (0.0 to 1.0). */
EWZX_IMPORT float trigger_left(uint32_t player);

/** Get right trigger value (0.0 to 1.0). */
EWZX_IMPORT float trigger_right(uint32_t player);

/* =============================================================================
 * RENDER STATE FUNCTIONS
 * ============================================================================= */

/**
 * Set the uniform tint color (multiplied with vertex colors and textures).
 * @param color  Color in 0xRRGGBBAA format
 */
EWZX_IMPORT void set_color(uint32_t color);

/**
 * Enable or disable depth testing.
 * @param enabled  0 to disable, non-zero to enable (default: enabled)
 */
EWZX_IMPORT void depth_test(uint32_t enabled);

/**
 * Set the face culling mode.
 * @param mode  0=none, 1=back (default), 2=front
 */
EWZX_IMPORT void cull_mode(uint32_t mode);

/**
 * Set the blend mode.
 * @param mode  0=none (opaque), 1=alpha, 2=additive, 3=multiply
 */
EWZX_IMPORT void blend_mode(uint32_t mode);

/**
 * Set the texture filtering mode.
 * @param filter  0=nearest (pixelated), 1=linear (smooth)
 */
EWZX_IMPORT void texture_filter(uint32_t filter);

/* =============================================================================
 * TEXTURE FUNCTIONS
 * ============================================================================= */

/**
 * Load a texture from RGBA pixel data.
 * @param width, height  Texture dimensions
 * @param pixels_ptr     Pointer to RGBA8 pixel data (width * height * 4 bytes)
 * @return Texture handle (>0) on success, 0 on failure
 */
EWZX_IMPORT uint32_t load_texture(uint32_t width, uint32_t height, const uint8_t* pixels_ptr);

/** Bind a texture to slot 0 (albedo). */
EWZX_IMPORT void texture_bind(uint32_t handle);

/**
 * Bind a texture to a specific slot.
 * @param slot  0=albedo, 1=MRE/matcap, 2=reserved, 3=matcap
 */
EWZX_IMPORT void texture_bind_slot(uint32_t handle, uint32_t slot);

/**
 * Set matcap blend mode for a texture slot (Mode 1 only).
 * @param slot  Matcap slot (1-3)
 * @param mode  0=Multiply, 1=Add, 2=HSV Modulate
 */
EWZX_IMPORT void matcap_blend_mode(uint32_t slot, uint32_t mode);

/* =============================================================================
 * MESH FUNCTIONS (RETAINED MODE)
 * ============================================================================= */

/**
 * Load a non-indexed mesh.
 *
 * Vertex format flags:
 *   1 (EWZX_FORMAT_UV):      Has UV coordinates (2 floats)
 *   2 (EWZX_FORMAT_COLOR):   Has per-vertex color (3 floats RGB)
 *   4 (EWZX_FORMAT_NORMAL):  Has normals (3 floats)
 *   8 (EWZX_FORMAT_SKINNED): Has bone indices/weights
 *
 * @return Mesh handle (>0) on success, 0 on failure
 */
EWZX_IMPORT uint32_t load_mesh(const float* data_ptr, uint32_t vertex_count, uint32_t format);

/**
 * Load an indexed mesh.
 * @return Mesh handle (>0) on success, 0 on failure
 */
EWZX_IMPORT uint32_t load_mesh_indexed(
    const float* data_ptr, uint32_t vertex_count,
    const uint16_t* index_ptr, uint32_t index_count,
    uint32_t format
);

/** Load packed mesh data (power user API, f16/snorm16/unorm8 encoding). */
EWZX_IMPORT uint32_t load_mesh_packed(const uint8_t* data_ptr, uint32_t vertex_count, uint32_t format);

/** Load indexed packed mesh data (power user API). */
EWZX_IMPORT uint32_t load_mesh_indexed_packed(
    const uint8_t* data_ptr, uint32_t vertex_count,
    const uint16_t* index_ptr, uint32_t index_count,
    uint32_t format
);

/** Draw a retained mesh with current transform and render state. */
EWZX_IMPORT void draw_mesh(uint32_t handle);

/* =============================================================================
 * PROCEDURAL MESH GENERATION
 * ============================================================================= */

/**
 * Generate a cube mesh.
 * @param size_x, size_y, size_z  Half-extents along each axis
 */
EWZX_IMPORT uint32_t cube(float size_x, float size_y, float size_z);

/**
 * Generate a UV sphere mesh.
 * @param radius    Sphere radius
 * @param segments  Longitudinal divisions (3-256)
 * @param rings     Latitudinal divisions (2-256)
 */
EWZX_IMPORT uint32_t sphere(float radius, uint32_t segments, uint32_t rings);

/**
 * Generate a cylinder or cone mesh.
 * @param radius_bottom, radius_top  Radii (>= 0.0, use 0 for cone tip)
 * @param height    Cylinder height
 * @param segments  Radial divisions (3-256)
 */
EWZX_IMPORT uint32_t cylinder(float radius_bottom, float radius_top, float height, uint32_t segments);

/**
 * Generate a plane mesh on the XZ plane.
 * @param size_x, size_z  Dimensions
 * @param subdivisions_x, subdivisions_z  Subdivisions (1-256)
 */
EWZX_IMPORT uint32_t plane(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);

/**
 * Generate a torus (donut) mesh.
 * @param major_radius  Distance from center to tube center
 * @param minor_radius  Tube radius
 * @param major_segments, minor_segments  Segment counts (3-256)
 */
EWZX_IMPORT uint32_t torus(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);

/**
 * Generate a capsule (pill shape) mesh.
 * @param radius    Capsule radius
 * @param height    Height of cylindrical section (total = height + 2*radius)
 * @param segments  Radial divisions (3-256)
 * @param rings     Divisions per hemisphere (1-128)
 */
EWZX_IMPORT uint32_t capsule(float radius, float height, uint32_t segments, uint32_t rings);

/* UV-enabled variants (Format 5: POS_UV_NORMAL) */

/** Generate a UV sphere with equirectangular texture mapping. */
EWZX_IMPORT uint32_t sphere_uv(float radius, uint32_t segments, uint32_t rings);

/** Generate a plane with UV mapping. */
EWZX_IMPORT uint32_t plane_uv(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);

/** Generate a cube with box-unwrapped UV mapping. */
EWZX_IMPORT uint32_t cube_uv(float size_x, float size_y, float size_z);

/** Generate a cylinder with cylindrical UV mapping. */
EWZX_IMPORT uint32_t cylinder_uv(float radius_bottom, float radius_top, float height, uint32_t segments);

/** Generate a torus with wrapped UV mapping. */
EWZX_IMPORT uint32_t torus_uv(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);

/** Generate a capsule with hybrid UV mapping. */
EWZX_IMPORT uint32_t capsule_uv(float radius, float height, uint32_t segments, uint32_t rings);

/* =============================================================================
 * IMMEDIATE MODE 3D DRAWING
 * ============================================================================= */

/**
 * Draw triangles immediately (non-indexed).
 * @param vertex_count  Must be multiple of 3
 * @param format        Vertex format flags (0-15)
 */
EWZX_IMPORT void draw_triangles(const float* data_ptr, uint32_t vertex_count, uint32_t format);

/**
 * Draw indexed triangles immediately.
 * @param index_count  Must be multiple of 3
 * @param format       Vertex format flags (0-15)
 */
EWZX_IMPORT void draw_triangles_indexed(
    const float* data_ptr, uint32_t vertex_count,
    const uint16_t* index_ptr, uint32_t index_count,
    uint32_t format
);

/* =============================================================================
 * BILLBOARD DRAWING
 * ============================================================================= */

/**
 * Draw a billboard (camera-facing quad) with full texture.
 * @param w, h   Billboard size in world units
 * @param mode   1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
 * @param color  Color tint (0xRRGGBBAA)
 */
EWZX_IMPORT void draw_billboard(float w, float h, uint32_t mode, uint32_t color);

/**
 * Draw a billboard with a UV region from the texture.
 * @param src_x, src_y, src_w, src_h  UV region (0.0-1.0)
 */
EWZX_IMPORT void draw_billboard_region(
    float w, float h,
    float src_x, float src_y, float src_w, float src_h,
    uint32_t mode, uint32_t color
);

/* =============================================================================
 * 2D DRAWING (SCREEN SPACE)
 * ============================================================================= */

/**
 * Draw a sprite with the bound texture.
 * @param x, y   Screen position in pixels (0,0 = top-left)
 * @param w, h   Sprite size in pixels
 * @param color  Color tint (0xRRGGBBAA)
 */
EWZX_IMPORT void draw_sprite(float x, float y, float w, float h, uint32_t color);

/**
 * Draw a region of a sprite sheet.
 * @param src_x, src_y, src_w, src_h  UV region (0.0-1.0)
 */
EWZX_IMPORT void draw_sprite_region(
    float x, float y, float w, float h,
    float src_x, float src_y, float src_w, float src_h,
    uint32_t color
);

/**
 * Draw a sprite with full control (rotation, origin, UV region).
 * @param origin_x, origin_y  Rotation pivot point (in pixels from sprite top-left)
 * @param angle_deg           Rotation angle in degrees (clockwise)
 */
EWZX_IMPORT void draw_sprite_ex(
    float x, float y, float w, float h,
    float src_x, float src_y, float src_w, float src_h,
    float origin_x, float origin_y, float angle_deg,
    uint32_t color
);

/** Draw a solid color rectangle. */
EWZX_IMPORT void draw_rect(float x, float y, float w, float h, uint32_t color);

/**
 * Draw text with the current font.
 * @param ptr   Pointer to UTF-8 string data
 * @param len   Length in bytes
 * @param size  Font size in pixels
 * @param color Text color (0xRRGGBBAA)
 */
EWZX_IMPORT void draw_text(const uint8_t* ptr, uint32_t len, float x, float y, float size, uint32_t color);

/**
 * Load a fixed-width bitmap font.
 * @param texture         Texture atlas handle
 * @param char_width      Glyph width in pixels
 * @param char_height     Glyph height in pixels
 * @param first_codepoint Unicode codepoint of first glyph
 * @param char_count      Number of glyphs
 * @return Font handle (use with font_bind())
 */
EWZX_IMPORT uint32_t load_font(
    uint32_t texture,
    uint32_t char_width, uint32_t char_height,
    uint32_t first_codepoint, uint32_t char_count
);

/**
 * Load a variable-width bitmap font.
 * @param widths_ptr  Pointer to array of char_count u8 widths
 */
EWZX_IMPORT uint32_t load_font_ex(
    uint32_t texture,
    const uint8_t* widths_ptr,
    uint32_t char_height,
    uint32_t first_codepoint,
    uint32_t char_count
);

/**
 * Bind a font for subsequent draw_text() calls.
 * Pass 0 for the built-in 8x8 monospace font.
 */
EWZX_IMPORT void font_bind(uint32_t font_handle);

/* =============================================================================
 * SKY SYSTEM
 * ============================================================================= */

/**
 * Set sky gradient colors.
 * @param horizon_color  Color at eye level (0xRRGGBBAA)
 * @param zenith_color   Color directly overhead (0xRRGGBBAA)
 */
EWZX_IMPORT void sky_set_colors(uint32_t horizon_color, uint32_t zenith_color);

/**
 * Set sky sun properties.
 * @param dir_x, dir_y, dir_z  Direction light rays travel (from sun toward surface)
 * @param color                Sun color (0xRRGGBBAA)
 * @param sharpness            Sun disc sharpness (0.0-1.0, higher = smaller)
 */
EWZX_IMPORT void sky_set_sun(float dir_x, float dir_y, float dir_z, uint32_t color, float sharpness);

/**
 * Bind a matcap texture to a slot (Mode 1 only).
 * @param slot  Matcap slot (1-3)
 */
EWZX_IMPORT void matcap_set(uint32_t slot, uint32_t texture);

/** Draw the procedural sky. Call first in render(), before any geometry. */
EWZX_IMPORT void draw_sky(void);

/* =============================================================================
 * MATERIAL FUNCTIONS (MODE 2/3)
 * ============================================================================= */

/** Bind an MRE texture (Metallic-Roughness-Emissive) to slot 1. */
EWZX_IMPORT void material_mre(uint32_t texture);

/** Bind an albedo texture to slot 0. */
EWZX_IMPORT void material_albedo(uint32_t texture);

/** Set material metallic value (0.0 = dielectric, 1.0 = metal). */
EWZX_IMPORT void material_metallic(float value);

/** Set material roughness value (0.0 = smooth, 1.0 = rough). */
EWZX_IMPORT void material_roughness(float value);

/** Set material emissive intensity (0.0 = no emission, >1.0 for HDR). */
EWZX_IMPORT void material_emissive(float value);

/**
 * Set rim lighting parameters.
 * @param intensity  Rim brightness (0.0-1.0)
 * @param power      Falloff sharpness (0.0-32.0, higher = tighter)
 */
EWZX_IMPORT void material_rim(float intensity, float power);

/** Set shininess (Mode 3 alias for roughness). */
EWZX_IMPORT void material_shininess(float value);

/**
 * Set specular color (Mode 3 only).
 * @param color  Specular color (0xRRGGBBAA, alpha ignored)
 */
EWZX_IMPORT void material_specular(uint32_t color);

/* =============================================================================
 * LIGHTING FUNCTIONS (MODE 2/3)
 * ============================================================================= */

/**
 * Set light direction (and enable the light).
 * @param index       Light index (0-3)
 * @param x, y, z     Direction rays travel (from light toward surface)
 * For a light from above, use (0, -1, 0).
 */
EWZX_IMPORT void light_set(uint32_t index, float x, float y, float z);

/**
 * Set light color.
 * @param color  Light color (0xRRGGBBAA, alpha ignored)
 */
EWZX_IMPORT void light_color(uint32_t index, uint32_t color);

/**
 * Set light intensity multiplier.
 * @param intensity  Typically 0.0-10.0
 */
EWZX_IMPORT void light_intensity(uint32_t index, float intensity);

/** Enable a light. */
EWZX_IMPORT void light_enable(uint32_t index);

/** Disable a light (preserves settings for re-enabling). */
EWZX_IMPORT void light_disable(uint32_t index);

/**
 * Convert a light to a point light at world position.
 * @param x, y, z  World-space position
 * Enables the light automatically. Default range is 10.0 units.
 */
EWZX_IMPORT void light_set_point(uint32_t index, float x, float y, float z);

/**
 * Set point light falloff distance.
 * @param range  Distance at which light reaches zero intensity
 * Only affects point lights (ignored for directional).
 */
EWZX_IMPORT void light_range(uint32_t index, float range);

/* =============================================================================
 * GPU SKINNING
 * ============================================================================= */

/**
 * Load a skeleton's inverse bind matrices to GPU.
 * Call once during init() after loading skinned meshes.
 *
 * @param inverse_bind_ptr  Pointer to array of 3x4 matrices (12 floats per bone, column-major)
 * @param bone_count        Number of bones (max 256)
 * @return Skeleton handle (>0) on success, 0 on error
 */
EWZX_IMPORT uint32_t load_skeleton(const float* inverse_bind_ptr, uint32_t bone_count);

/**
 * Bind a skeleton for subsequent skinned mesh rendering.
 * @param skeleton  Skeleton handle from load_skeleton(), or 0 to unbind (raw mode)
 */
EWZX_IMPORT void skeleton_bind(uint32_t skeleton);

/**
 * Set bone transform matrices for skeletal animation.
 *
 * @param matrices_ptr  Pointer to array of 3x4 matrices (12 floats per bone, column-major)
 * @param count         Number of bones (max 256)
 *
 * Each bone matrix is 12 floats in column-major order:
 *   [col0.x, col0.y, col0.z]  // X axis
 *   [col1.x, col1.y, col1.z]  // Y axis
 *   [col2.x, col2.y, col2.z]  // Z axis
 *   [tx,     ty,     tz    ]  // translation
 */
EWZX_IMPORT void set_bones(const float* matrices_ptr, uint32_t count);

/* =============================================================================
 * AUDIO FUNCTIONS
 * ============================================================================= */

/**
 * Load raw PCM sound data (22.05kHz, 16-bit signed, mono).
 * Must be called during init().
 *
 * @param data_ptr  Pointer to i16 PCM samples
 * @param byte_len  Length in bytes (must be even)
 * @return Sound handle for use with playback functions
 */
EWZX_IMPORT uint32_t load_sound(const int16_t* data_ptr, uint32_t byte_len);

/**
 * Play sound on next available channel (fire-and-forget).
 * @param volume  0.0 to 1.0
 * @param pan     -1.0 (left) to 1.0 (right), 0.0 = center
 */
EWZX_IMPORT void play_sound(uint32_t sound, float volume, float pan);

/**
 * Play sound on a specific channel (for managed/looping audio).
 * @param channel  Channel index (0-15)
 * @param looping  1 = loop, 0 = play once
 */
EWZX_IMPORT void channel_play(uint32_t channel, uint32_t sound, float volume, float pan, uint32_t looping);

/** Update channel parameters (call every frame for positional audio). */
EWZX_IMPORT void channel_set(uint32_t channel, float volume, float pan);

/** Stop a channel. */
EWZX_IMPORT void channel_stop(uint32_t channel);

/** Play music (dedicated looping channel). */
EWZX_IMPORT void music_play(uint32_t sound, float volume);

/** Stop music. */
EWZX_IMPORT void music_stop(void);

/** Set music volume. */
EWZX_IMPORT void music_set_volume(float volume);

/* =============================================================================
 * ROM DATA PACK API (init-only)
 *
 * Load assets from the bundled ROM data pack by string ID.
 * Assets go directly to VRAM/audio memory, bypassing WASM linear memory.
 * ============================================================================= */

/**
 * Load a texture from ROM data pack by ID.
 * @param id_ptr  Pointer to asset ID string
 * @param id_len  Length of asset ID string
 * @return Texture handle (>0) on success. Traps on failure.
 */
EWZX_IMPORT uint32_t rom_texture(uint32_t id_ptr, uint32_t id_len);

/** Load a mesh from ROM data pack by ID. */
EWZX_IMPORT uint32_t rom_mesh(uint32_t id_ptr, uint32_t id_len);

/** Load skeleton inverse bind matrices from ROM data pack by ID. */
EWZX_IMPORT uint32_t rom_skeleton(uint32_t id_ptr, uint32_t id_len);

/** Load a font atlas from ROM data pack by ID. */
EWZX_IMPORT uint32_t rom_font(uint32_t id_ptr, uint32_t id_len);

/** Load a sound from ROM data pack by ID. */
EWZX_IMPORT uint32_t rom_sound(uint32_t id_ptr, uint32_t id_len);

/**
 * Get the byte size of raw data in the ROM data pack.
 * Use this to allocate a buffer before calling rom_data().
 */
EWZX_IMPORT uint32_t rom_data_len(uint32_t id_ptr, uint32_t id_len);

/**
 * Copy raw data from ROM data pack into WASM linear memory.
 * @param dst_ptr  Pointer to destination buffer
 * @param max_len  Maximum bytes to copy
 * @return Bytes written on success. Traps on failure.
 */
EWZX_IMPORT uint32_t rom_data(uint32_t id_ptr, uint32_t id_len, uint32_t dst_ptr, uint32_t max_len);

/* =============================================================================
 * EMBEDDED ASSET API
 *
 * Load assets from EmberZ binary formats embedded in the WASM binary.
 * ============================================================================= */

/** Load a mesh from .ewzxmesh binary format. */
EWZX_IMPORT uint32_t load_zmesh(uint32_t data_ptr, uint32_t data_len);

/** Load a texture from .ewzxtex binary format. */
EWZX_IMPORT uint32_t load_ztex(uint32_t data_ptr, uint32_t data_len);

/** Load a sound from .ewzxsnd binary format. */
EWZX_IMPORT uint32_t load_zsound(uint32_t data_ptr, uint32_t data_len);

/* =============================================================================
 * DEBUG INSPECTION SYSTEM
 *
 * Runtime value inspection and editing for development.
 * Press F3 to open panel. Zero overhead in release builds.
 * ============================================================================= */

/* Primitive Type Registration (Editable) */
EWZX_IMPORT void debug_register_i8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_i16(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_i32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_u8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_u16(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_u32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_f32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_bool(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

/* Range-Constrained Registration (Slider UI) */
EWZX_IMPORT void debug_register_i32_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, int32_t min, int32_t max);
EWZX_IMPORT void debug_register_f32_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, float min, float max);
EWZX_IMPORT void debug_register_u8_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, uint32_t min, uint32_t max);
EWZX_IMPORT void debug_register_u16_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, uint32_t min, uint32_t max);
EWZX_IMPORT void debug_register_i16_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, int32_t min, int32_t max);

/* Compound Type Registration (Editable) */
EWZX_IMPORT void debug_register_vec2(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_vec3(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_rect(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_color(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

/* Fixed-Point Type Registration (Editable) */
EWZX_IMPORT void debug_register_fixed_i16_q8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_fixed_i32_q16(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_fixed_i32_q8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_register_fixed_i32_q24(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

/* Watch Functions (Read-Only Display) */
EWZX_IMPORT void debug_watch_i8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_i16(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_i32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_u8(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_u16(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_u32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_f32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_bool(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_vec2(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_vec3(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_rect(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
EWZX_IMPORT void debug_watch_color(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

/* Grouping Functions */
EWZX_IMPORT void debug_group_begin(uint32_t name_ptr, uint32_t name_len);
EWZX_IMPORT void debug_group_end(void);

/* State Query Functions */
EWZX_IMPORT int32_t debug_is_paused(void);
EWZX_IMPORT float debug_get_time_scale(void);

/* =============================================================================
 * CONSTANTS
 * ============================================================================= */

/* Button indices for input functions */
#define EWZX_BUTTON_UP      0
#define EWZX_BUTTON_DOWN    1
#define EWZX_BUTTON_LEFT    2
#define EWZX_BUTTON_RIGHT   3
#define EWZX_BUTTON_A       4
#define EWZX_BUTTON_B       5
#define EWZX_BUTTON_X       6
#define EWZX_BUTTON_Y       7
#define EWZX_BUTTON_L1      8
#define EWZX_BUTTON_R1      9
#define EWZX_BUTTON_L3      10
#define EWZX_BUTTON_R3      11
#define EWZX_BUTTON_START   12
#define EWZX_BUTTON_SELECT  13

/* Render modes for render_mode() */
#define EWZX_RENDER_LAMBERT 0
#define EWZX_RENDER_MATCAP  1
#define EWZX_RENDER_PBR     2
#define EWZX_RENDER_HYBRID  3

/* Blend modes for blend_mode() */
#define EWZX_BLEND_NONE     0
#define EWZX_BLEND_ALPHA    1
#define EWZX_BLEND_ADDITIVE 2
#define EWZX_BLEND_MULTIPLY 3

/* Cull modes for cull_mode() */
#define EWZX_CULL_NONE      0
#define EWZX_CULL_BACK      1
#define EWZX_CULL_FRONT     2

/* Vertex format flags for mesh loading */
#define EWZX_FORMAT_POS     0
#define EWZX_FORMAT_UV      1
#define EWZX_FORMAT_COLOR   2
#define EWZX_FORMAT_NORMAL  4
#define EWZX_FORMAT_SKINNED 8

/* Common format combinations */
#define EWZX_FORMAT_POS_UV                 (EWZX_FORMAT_UV)
#define EWZX_FORMAT_POS_COLOR              (EWZX_FORMAT_COLOR)
#define EWZX_FORMAT_POS_NORMAL             (EWZX_FORMAT_NORMAL)
#define EWZX_FORMAT_POS_UV_NORMAL          (EWZX_FORMAT_UV | EWZX_FORMAT_NORMAL)
#define EWZX_FORMAT_POS_UV_COLOR           (EWZX_FORMAT_UV | EWZX_FORMAT_COLOR)
#define EWZX_FORMAT_POS_UV_COLOR_NORMAL    (EWZX_FORMAT_UV | EWZX_FORMAT_COLOR | EWZX_FORMAT_NORMAL)
#define EWZX_FORMAT_POS_SKINNED            (EWZX_FORMAT_SKINNED)
#define EWZX_FORMAT_POS_NORMAL_SKINNED     (EWZX_FORMAT_NORMAL | EWZX_FORMAT_SKINNED)
#define EWZX_FORMAT_POS_UV_NORMAL_SKINNED  (EWZX_FORMAT_UV | EWZX_FORMAT_NORMAL | EWZX_FORMAT_SKINNED)

/* Billboard modes for draw_billboard() */
#define EWZX_BILLBOARD_SPHERICAL     1
#define EWZX_BILLBOARD_CYLINDRICAL_Y 2
#define EWZX_BILLBOARD_CYLINDRICAL_X 3
#define EWZX_BILLBOARD_CYLINDRICAL_Z 4

/* Tick rate indices for set_tick_rate() */
#define EWZX_TICK_RATE_24  0
#define EWZX_TICK_RATE_30  1
#define EWZX_TICK_RATE_60  2
#define EWZX_TICK_RATE_120 3

/* Common colors (0xRRGGBBAA format) */
#define EWZX_WHITE       0xFFFFFFFF
#define EWZX_BLACK       0x000000FF
#define EWZX_RED         0xFF0000FF
#define EWZX_GREEN       0x00FF00FF
#define EWZX_BLUE        0x0000FFFF
#define EWZX_YELLOW      0xFFFF00FF
#define EWZX_CYAN        0x00FFFFFF
#define EWZX_MAGENTA     0xFF00FFFF
#define EWZX_ORANGE      0xFF8000FF
#define EWZX_TRANSPARENT 0x00000000

/* =============================================================================
 * INLINE HELPER FUNCTIONS
 * ============================================================================= */

/** Pack RGBA color components into a u32. */
static inline uint32_t ewzx_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    return ((uint32_t)r << 24) | ((uint32_t)g << 16) | ((uint32_t)b << 8) | (uint32_t)a;
}

/** Pack RGB color components into a u32 (alpha = 255). */
static inline uint32_t ewzx_rgb(uint8_t r, uint8_t g, uint8_t b) {
    return ewzx_rgba(r, g, b, 255);
}

/** Helper to log a string literal. Usage: EWZX_LOG("Hello") */
#define EWZX_LOG(str) log_msg((const uint8_t*)(str), sizeof(str) - 1)

/** Helper to draw text from a string literal. */
#define EWZX_DRAW_TEXT(str, x, y, size, color) \
    draw_text((const uint8_t*)(str), sizeof(str) - 1, (x), (y), (size), (color))

/** Helper to load ROM texture from string literal. */
#define EWZX_ROM_TEXTURE(id) rom_texture((uint32_t)(id), sizeof(id) - 1)

/** Helper to load ROM mesh from string literal. */
#define EWZX_ROM_MESH(id) rom_mesh((uint32_t)(id), sizeof(id) - 1)

/** Helper to load ROM sound from string literal. */
#define EWZX_ROM_SOUND(id) rom_sound((uint32_t)(id), sizeof(id) - 1)

/** Helper to load ROM font from string literal. */
#define EWZX_ROM_FONT(id) rom_font((uint32_t)(id), sizeof(id) - 1)

/** Helper to load ROM skeleton from string literal. */
#define EWZX_ROM_SKELETON(id) rom_skeleton((uint32_t)(id), sizeof(id) - 1)

/** Clamp a value between min and max. */
static inline float ewzx_clampf(float val, float min, float max) {
    if (val < min) return min;
    if (val > max) return max;
    return val;
}

/** Linear interpolation. */
static inline float ewzx_lerpf(float a, float b, float t) {
    return a + (b - a) * t;
}

/** Minimum of two floats. */
static inline float ewzx_minf(float a, float b) {
    return a < b ? a : b;
}

/** Maximum of two floats. */
static inline float ewzx_maxf(float a, float b) {
    return a > b ? a : b;
}

/** Absolute value of a float. */
static inline float ewzx_absf(float x) {
    return x < 0.0f ? -x : x;
}

#ifdef __cplusplus
}
#endif

#endif /* EMBERWARE_ZX_H */
