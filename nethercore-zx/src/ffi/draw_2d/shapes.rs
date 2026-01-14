//! Shape drawing functions
//!
//! Functions for drawing basic shapes (rectangles, lines, circles) in screen space.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::ffi::ZXGameContext;

use super::SCREEN_SPACE_DEPTH;

/// Number of segments used for circle rendering
const CIRCLE_SEGMENTS: u32 = 16;

/// Register shape drawing FFI functions
pub(super) fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "draw_rect", draw_rect)?;
    linker.func_wrap("env", "draw_line", draw_line)?;
    linker.func_wrap("env", "draw_circle", draw_circle)?;
    linker.func_wrap("env", "draw_circle_outline", draw_circle_outline)?;
    Ok(())
}

/// Draw a solid color rectangle
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Rectangle width in pixels
/// * `h` — Rectangle height in pixels
///
/// Draws an untextured quad. Useful for UI backgrounds, health bars, etc.
/// Uses color from set_color().
fn draw_rect(mut caller: Caller<'_, ZXGameContext>, x: f32, y: f32, w: f32, h: f32) {
    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x = vp.x as f32 + x;
    let screen_y = vp.y as f32 + y;

    // Bind white texture (handle 0xFFFFFFFF) to slot 0
    state.bound_textures[0] = u32::MAX;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Use white instance color - actual color comes from material color in shading state
    // Create screen-space quad instance (rects use white/fallback texture)
    let instance = crate::graphics::QuadInstance::sprite(
        screen_x,
        screen_y,
        depth,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV (white texture is 1x1, so any UV works)
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance, state.current_z_index);
}

/// Draw a line between two points
///
/// # Arguments
/// * `x1`, `y1` — Start point in screen pixels
/// * `x2`, `y2` — End point in screen pixels
/// * `thickness` — Line thickness in pixels
///
/// Draws a line as a rotated rectangle from start to end point.
/// Uses color from set_color().
fn draw_line(
    mut caller: Caller<'_, ZXGameContext>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x1 = vp.x as f32 + x1;
    let screen_y1 = vp.y as f32 + y1;
    let screen_x2 = vp.x as f32 + x2;
    let screen_y2 = vp.y as f32 + y2;

    // Bind white texture for solid color
    state.bound_textures[0] = u32::MAX;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Calculate line geometry
    let dx = screen_x2 - screen_x1;
    let dy = screen_y2 - screen_y1;
    let length = (dx * dx + dy * dy).sqrt();

    if length < 0.001 {
        return; // Degenerate line
    }

    let angle = dy.atan2(dx); // Radians

    // Create rotated rectangle
    // Position the rectangle so it starts at (x1, y1) and extends to (x2, y2)
    // The rectangle origin is at top-left, so we need to offset by half thickness
    let instance = crate::graphics::QuadInstance::sprite(
        screen_x1,
        screen_y1 - thickness / 2.0,
        depth,
        length,
        thickness,
        angle,
        [0.0, 0.0, 1.0, 1.0],
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance, state.current_z_index);
}

/// Draw a filled circle
///
/// # Arguments
/// * `x`, `y` — Center position in screen pixels
/// * `radius` — Circle radius in pixels
///
/// Rendered as a 16-segment approximation using rotated rectangles.
/// Uses color from set_color().
fn draw_circle(mut caller: Caller<'_, ZXGameContext>, x: f32, y: f32, radius: f32) {
    if radius <= 0.0 {
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x = vp.x as f32 + x;
    let screen_y = vp.y as f32 + y;

    // Bind white texture for solid color
    state.bound_textures[0] = u32::MAX;

    // Get shading state index (includes current color from set_color)
    let shading_state_index = state.add_shading_state();
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Convert layer to depth for ordering
    let depth = SCREEN_SPACE_DEPTH;

    // Draw circle as pie slices (rotated rectangles from center)
    // Each slice is a thin rectangle extending from center outward
    let angle_step = std::f32::consts::TAU / CIRCLE_SEGMENTS as f32;

    // Calculate the width needed for each segment to overlap properly
    // For 16 segments, each covers 22.5 degrees
    // Width at the outer edge = 2 * radius * sin(angle_step / 2)
    let segment_width = 2.0 * radius * (angle_step / 2.0).sin();

    for i in 0..CIRCLE_SEGMENTS {
        let angle = i as f32 * angle_step;

        // Create a rectangle from center pointing outward
        // The rectangle extends from center to edge
        let instance = crate::graphics::QuadInstance::sprite(
            screen_x - segment_width / 2.0,
            screen_y,
            depth,
            segment_width,
            radius,
            angle - std::f32::consts::FRAC_PI_2, // Rotate to point outward
            [0.0, 0.0, 1.0, 1.0],
            shading_state_index.0,
            view_idx,
        );

        state.add_quad_instance(instance, state.current_z_index);
    }
}

/// Draw a circle outline
///
/// # Arguments
/// * `x`, `y` — Center position in screen pixels
/// * `radius` — Circle radius in pixels
/// * `thickness` — Line thickness in pixels
///
/// Rendered as 16 line segments forming the circle outline.
/// Uses color from set_color().
fn draw_circle_outline(
    mut caller: Caller<'_, ZXGameContext>,
    x: f32,
    y: f32,
    radius: f32,
    thickness: f32,
) {
    if radius <= 0.0 {
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Offset by viewport origin for split-screen support
    let vp = state.current_viewport;
    let screen_x = vp.x as f32 + x;
    let screen_y = vp.y as f32 + y;

    // Bind white texture for solid color
    state.bound_textures[0] = u32::MAX;

    // Convert layer to depth for ordering (computed once, used for all segments)
    let depth = SCREEN_SPACE_DEPTH;

    let angle_step = std::f32::consts::TAU / CIRCLE_SEGMENTS as f32;

    for i in 0..CIRCLE_SEGMENTS {
        let angle1 = i as f32 * angle_step;
        let angle2 = (i + 1) as f32 * angle_step;

        let x1 = screen_x + radius * angle1.cos();
        let y1 = screen_y + radius * angle1.sin();
        let x2 = screen_x + radius * angle2.cos();
        let y2 = screen_y + radius * angle2.sin();

        // Get shading state for each line segment (includes current color)
        let shading_state_index = state.add_shading_state();
        let view_idx = (state.view_matrices.len() - 1) as u32;

        // Calculate line geometry
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);

        // Create rotated rectangle for this line segment
        let instance = crate::graphics::QuadInstance::sprite(
            x1,
            y1 - thickness / 2.0,
            depth,
            length,
            thickness,
            angle,
            [0.0, 0.0, 1.0, 1.0],
            shading_state_index.0,
            view_idx,
        );

        state.add_quad_instance(instance, state.current_z_index);
    }
}
