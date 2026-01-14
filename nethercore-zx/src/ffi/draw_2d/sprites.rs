//! Sprite drawing functions
//!
//! Functions for drawing sprites and sprite regions in screen space.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::ffi::ZXGameContext;

use super::SCREEN_SPACE_DEPTH;

/// Register sprite drawing FFI functions
pub(super) fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "draw_sprite", draw_sprite)?;
    linker.func_wrap("env", "draw_sprite_region", draw_sprite_region)?;
    linker.func_wrap("env", "draw_sprite_ex", draw_sprite_ex)?;
    Ok(())
}

/// Draw a sprite with the bound texture
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
///
/// Draws the full texture (UV 0,0 to 1,1) as a quad in screen space.
/// Uses current blend mode, bound texture (slot 0), and color from set_color().
fn draw_sprite(mut caller: Caller<'_, ZXGameContext>, x: f32, y: f32, w: f32, h: f32) {
    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x = vp.x as f32 + x;
    let screen_y = vp.y as f32 + y;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();

    // Get current view index (last in pool, following Option pattern)
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        screen_x,
        screen_y,
        depth,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance, state.current_z_index);
}

/// Draw a region of a sprite sheet
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `src_x` — Source texture X coordinate (0.0-1.0)
/// * `src_y` — Source texture Y coordinate (0.0-1.0)
/// * `src_w` — Source texture width (0.0-1.0)
/// * `src_h` — Source texture height (0.0-1.0)
///
/// Useful for sprite sheets and texture atlases. Uses color from set_color().
fn draw_sprite_region(
    mut caller: Caller<'_, ZXGameContext>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x = vp.x as f32 + x;
    let screen_y = vp.y as f32 + y;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates (convert from src_x,src_y,src_w,src_h to u0,v0,u1,v1)
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        screen_x,
        screen_y,
        depth,
        w,
        h,
        0.0,              // No rotation
        [u0, v0, u1, v1], // Texture UV region
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance, state.current_z_index);
}

/// Draw a sprite with full control (rotation, origin, UV region)
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `src_x` — Source texture X coordinate (0.0-1.0)
/// * `src_y` — Source texture Y coordinate (0.0-1.0)
/// * `src_w` — Source texture width (0.0-1.0)
/// * `src_h` — Source texture height (0.0-1.0)
/// * `origin_x` — Origin X offset in pixels (0 = left edge of sprite)
/// * `origin_y` — Origin Y offset in pixels (0 = top edge of sprite)
/// * `angle_deg` — Rotation angle in degrees (clockwise)
///
/// The sprite rotates around the origin point. For center rotation, use (w/2, h/2).
/// Uses color from set_color().
fn draw_sprite_ex(
    mut caller: Caller<'_, ZXGameContext>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    origin_x: f32,
    origin_y: f32,
    angle_deg: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Apply origin offset and viewport offset to position
    let screen_x = vp.x as f32 + x - origin_x;
    let screen_y = vp.y as f32 + y - origin_y;

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Create screen-space quad instance with rotation
    let instance = crate::graphics::QuadInstance::sprite(
        screen_x,
        screen_y,
        depth,
        w,
        h,
        angle_deg.to_radians(), // Convert degrees to radians
        [u0, v0, u1, v1],
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance, state.current_z_index);
}
