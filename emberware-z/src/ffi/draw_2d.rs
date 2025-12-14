//! 2D drawing FFI functions (screen space)
//!
//! Functions for drawing sprites, rectangles, and text in screen space.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZContext;

use crate::state::Font;

/// Register 2D drawing FFI functions
pub fn register(linker: &mut Linker<ZContext>) -> Result<()> {
    linker.func_wrap("env", "draw_sprite", draw_sprite)?;
    linker.func_wrap("env", "draw_sprite_region", draw_sprite_region)?;
    linker.func_wrap("env", "draw_sprite_ex", draw_sprite_ex)?;
    linker.func_wrap("env", "draw_rect", draw_rect)?;
    linker.func_wrap("env", "draw_text", draw_text)?;
    linker.func_wrap("env", "load_font", load_font)?;
    linker.func_wrap("env", "load_font_ex", load_font_ex)?;
    linker.func_wrap("env", "font_bind", font_bind)?;
    Ok(())
}

/// Draw a sprite with the bound texture
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `color` — Color tint (0xRRGGBBAA)
///
/// Draws the full texture (UV 0,0 to 1,1) as a quad in screen space.
/// Uses current blend mode and bound texture (slot 0).
fn draw_sprite(
    mut caller: Caller<'_, ZContext>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Get current view index (last in pool, following Option pattern)
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV
        color,
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance);
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
/// * `color` — Color tint (0xRRGGBBAA)
///
/// Useful for sprite sheets and texture atlases.
fn draw_sprite_region(
    mut caller: Caller<'_, ZContext>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates (convert from src_x,src_y,src_w,src_h to u0,v0,u1,v1)
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,              // No rotation
        [u0, v0, u1, v1], // Texture UV region
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
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
/// * `color` — Color tint (0xRRGGBBAA)
///
/// The sprite rotates around the origin point. For center rotation, use (w/2, h/2).
fn draw_sprite_ex(
    mut caller: Caller<'_, ZContext>,
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
    color: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Apply origin offset to position
    let adjusted_x = x - origin_x;
    let adjusted_y = y - origin_y;

    // Create screen-space quad instance with rotation
    let instance = crate::graphics::QuadInstance::sprite(
        adjusted_x,
        adjusted_y,
        w,
        h,
        angle_deg.to_radians(), // Convert degrees to radians
        [u0, v0, u1, v1],
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
}

/// Draw a solid color rectangle
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Rectangle width in pixels
/// * `h` — Rectangle height in pixels
/// * `color` — Fill color (0xRRGGBBAA)
///
/// Draws an untextured quad. Useful for UI backgrounds, health bars, etc.
fn draw_rect(
    mut caller: Caller<'_, ZContext>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Bind white texture (handle 0xFFFFFFFF) to slot 0
    state.bound_textures[0] = u32::MAX;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Create screen-space quad instance (rects use white/fallback texture)
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV (white texture is 1x1, so any UV works)
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
}

/// Draw text with the built-in font
///
/// # Arguments
/// * `ptr` — Pointer to UTF-8 string data
/// * `len` — Length of string in bytes
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (baseline)
/// * `size` — Font size in pixels
/// * `color` — Text color (0xRRGGBBAA)
///
/// Supports full UTF-8 encoding. Text is left-aligned with no wrapping.
fn draw_text(
    mut caller: Caller<'_, ZContext>,
    ptr: u32,
    len: u32,
    x: f32,
    y: f32,
    size: f32,
    color: u32,
) {
    // Read UTF-8 string from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("draw_text: no WASM memory available");
            return;
        }
    };

    let text_str = {
        let mem_data = memory.data(&caller);
        let ptr = ptr as usize;
        let len = len as usize;

        if ptr + len > mem_data.len() {
            warn!(
                "draw_text: string data ({} bytes at {}) exceeds memory bounds ({})",
                len,
                ptr,
                mem_data.len()
            );
            return;
        }

        let bytes = &mem_data[ptr..ptr + len];
        // Validate UTF-8 and copy to owned string
        match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(), // Convert to owned String
            Err(_) => {
                warn!("draw_text: invalid UTF-8 string");
                return;
            }
        }
    };

    // Skip empty text
    if text_str.is_empty() {
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Text always uses alpha blending for smooth anti-aliased edges
    state.blend_mode = 1; // BlendMode::Alpha

    // Text always uses nearest filtering (crisp pixels, no blurry interpolation)
    state.texture_filter = 0;
    state.update_texture_filter(false);

    // Ensure material color is white so it doesn't interfere with text instance color
    // (Text color is passed via the color parameter and stored in instance.color)
    state.update_color(0xFFFFFFFF);

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Force lazy push of view matrix if pending
    if let Some(mat) = state.current_view_matrix.take() {
        state.view_matrices.push(mat);
    }
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Determine which font to use
    let font_handle = state.current_font;

    // Look up custom font if font_handle != 0
    // Clone the font to avoid holding a borrow across add_quad_instance calls
    let custom_font = if font_handle == 0 {
        None
    } else {
        let font_index = (font_handle - 1) as usize;
        state.fonts.get(font_index).cloned()
    };

    // Bind the appropriate font texture to slot 0
    if let Some(ref font) = custom_font {
        state.bound_textures[0] = font.texture;
    } else {
        // For built-in font, use reserved handle (u32::MAX - 1)
        // This handle is mapped to the actual built-in font texture at startup
        state.bound_textures[0] = u32::MAX - 1;
    }

    // Generate quad instances for each character
    let mut cursor_x = x;

    if let Some(ref font) = custom_font {
        // Custom font rendering
        let scale = size / font.char_height as f32;
        let glyph_height = size;

        // Use stored atlas dimensions
        let texture_width = font.atlas_width;
        let texture_height = font.atlas_height;

        let max_glyph_width = font.char_width as u32;
        let glyphs_per_row = texture_width / max_glyph_width.max(1);

        for ch in text_str.chars() {
            let char_code = ch as u32;

            // Calculate glyph index
            if char_code < font.first_codepoint
                || char_code >= font.first_codepoint + font.char_count
            {
                // Character not in font, skip or use replacement
                continue;
            }
            let glyph_index = (char_code - font.first_codepoint) as usize;

            // Get glyph width (variable or fixed)
            let glyph_width_px = font
                .char_widths
                .as_ref()
                .and_then(|widths| widths.get(glyph_index).copied())
                .unwrap_or(font.char_width);
            let glyph_width = glyph_width_px as f32 * scale;

            // Calculate UV coordinates
            let col = glyph_index % glyphs_per_row as usize;
            let row = glyph_index / glyphs_per_row as usize;

            let u0 = (col * max_glyph_width as usize) as f32 / texture_width as f32;
            let v0 = (row * font.char_height as usize) as f32 / texture_height as f32;
            let u1 = ((col * max_glyph_width as usize) + glyph_width_px as usize) as f32
                / texture_width as f32;
            let v1 = ((row + 1) * font.char_height as usize) as f32 / texture_height as f32;

            // Create quad instance for this glyph
            let instance = crate::graphics::QuadInstance::sprite(
                cursor_x,
                y,
                glyph_width,
                glyph_height,
                0.0, // no rotation
                [u0, v0, u1, v1],
                color,
                shading_state_index.0,
                view_idx,
            );
            state.add_quad_instance(instance);

            cursor_x += glyph_width;
        }
    } else {
        // Built-in font rendering
        let scale = size / crate::font::GLYPH_HEIGHT as f32;
        let glyph_width = crate::font::GLYPH_WIDTH as f32 * scale;
        let glyph_height = crate::font::GLYPH_HEIGHT as f32 * scale;

        for ch in text_str.chars() {
            let char_code = ch as u32;

            // Get UV coordinates for this character
            let (u0, v0, u1, v1) = crate::font::get_glyph_uv(char_code);

            // Create quad instance for this glyph
            let instance = crate::graphics::QuadInstance::sprite(
                cursor_x,
                y,
                glyph_width,
                glyph_height,
                0.0, // no rotation
                [u0, v0, u1, v1],
                color,
                shading_state_index.0,
                view_idx,
            );
            state.add_quad_instance(instance);

            cursor_x += glyph_width;
        }
    }
}

/// Load a fixed-width bitmap font from a texture atlas
///
/// The texture must contain a grid of glyphs arranged left-to-right, top-to-bottom.
/// Each glyph occupies char_width × char_height pixels.
///
/// # Arguments
/// * `texture` — Handle to the texture atlas
/// * `char_width` — Width of each glyph in pixels
/// * `char_height` — Height of each glyph in pixels
/// * `first_codepoint` — Unicode codepoint of the first glyph
/// * `char_count` — Number of glyphs in the font
///
/// # Returns
/// Handle to the loaded font (use with `font_bind()`)
///
/// # Notes
/// - Call this in `init()` - font loading is not allowed during gameplay
/// - All glyphs in a fixed-width font have the same width
/// - The texture must have enough space for char_count glyphs
#[inline]
fn load_font(
    mut caller: Caller<'_, ZContext>,
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32,
) -> u32 {
    // Only allow during init
    if !caller.data().game.in_init {
        warn!("load_font: can only be called during init()");
        return 0;
    }

    // Validate parameters
    if texture == 0 {
        warn!("load_font: invalid texture handle 0");
        return 0;
    }
    if char_width == 0 || char_width > 255 {
        warn!("load_font: char_width must be 1-255");
        return 0;
    }
    if char_height == 0 || char_height > 255 {
        warn!("load_font: char_height must be 1-255");
        return 0;
    }
    if char_count == 0 {
        warn!("load_font: char_count must be > 0");
        return 0;
    }

    let state = &mut caller.data_mut().ffi;

    // Look up texture dimensions from pending_textures
    let (atlas_width, atlas_height) = state
        .pending_textures
        .iter()
        .find(|t| t.handle == texture)
        .map(|t| (t.width, t.height))
        .unwrap_or_else(|| {
            warn!(
                "load_font: texture {} not found in pending_textures, using 1024x1024",
                texture
            );
            (1024, 1024)
        });

    // Allocate font handle
    let handle = state.next_font_handle;
    state.next_font_handle += 1;

    // Create font descriptor
    let font = Font {
        texture,
        atlas_width,
        atlas_height,
        char_width: char_width as u8,
        char_height: char_height as u8,
        first_codepoint,
        char_count,
        char_widths: None, // Fixed-width
    };

    state.fonts.push(font);
    handle
}

/// Load a variable-width bitmap font from a texture atlas
///
/// Like `load_font()`, but allows each glyph to have a different width.
///
/// # Arguments
/// * `texture` — Handle to the texture atlas
/// * `widths_ptr` — Pointer to array of char_count u8 widths
/// * `char_height` — Height of each glyph in pixels
/// * `first_codepoint` — Unicode codepoint of the first glyph
/// * `char_count` — Number of glyphs in the font
///
/// # Returns
/// Handle to the loaded font (use with `font_bind()`)
///
/// # Notes
/// - Call this in `init()` - font loading is not allowed during gameplay
/// - The widths array must have exactly char_count entries
/// - Glyphs are still arranged in a grid, but can have custom widths
#[inline]
fn load_font_ex(
    mut caller: Caller<'_, ZContext>,
    texture: u32,
    widths_ptr: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32,
) -> u32 {
    // Only allow during init
    if !caller.data().game.in_init {
        warn!("load_font_ex: can only be called during init()");
        return 0;
    }

    // Validate parameters
    if texture == 0 {
        warn!("load_font_ex: invalid texture handle 0");
        return 0;
    }
    if char_height == 0 || char_height > 255 {
        warn!("load_font_ex: char_height must be 1-255");
        return 0;
    }
    if char_count == 0 {
        warn!("load_font_ex: char_count must be > 0");
        return 0;
    }

    // Read widths array from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("load_font_ex: no WASM memory available");
            return 0;
        }
    };

    let widths = {
        let mem_data = memory.data(&caller);
        let ptr = widths_ptr as usize;
        let len = char_count as usize;

        if ptr + len > mem_data.len() {
            warn!(
                "load_font_ex: widths array ({} bytes at {}) exceeds memory bounds ({})",
                len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        mem_data[ptr..ptr + len].to_vec()
    };

    let state = &mut caller.data_mut().ffi;

    // Look up texture dimensions from pending_textures
    let (atlas_width, atlas_height) = state
        .pending_textures
        .iter()
        .find(|t| t.handle == texture)
        .map(|t| (t.width, t.height))
        .unwrap_or_else(|| {
            warn!(
                "load_font_ex: texture {} not found in pending_textures, using 1024x1024",
                texture
            );
            (1024, 1024)
        });

    // Allocate font handle
    let handle = state.next_font_handle;
    state.next_font_handle += 1;

    // Get max width from widths array for grid calculations
    let max_char_width = widths.iter().copied().max().unwrap_or(8);

    // Create font descriptor
    let font = Font {
        texture,
        atlas_width,
        atlas_height,
        char_width: max_char_width, // Max width for grid calculations
        char_height: char_height as u8,
        first_codepoint,
        char_count,
        char_widths: Some(widths),
    };

    state.fonts.push(font);
    handle
}

/// Bind a font for subsequent draw_text() calls
///
/// # Arguments
/// * `font_handle` — Font handle from load_font() or load_font_ex(), or 0 for built-in font
///
/// # Notes
/// - Font handle 0 uses the built-in 8×8 monospace font (default)
/// - Custom fonts persist for all subsequent draw_text() calls until changed
#[inline]
fn font_bind(mut caller: Caller<'_, ZContext>, font_handle: u32) {
    let state = &mut caller.data_mut().ffi;

    // Validate font handle (0 is always valid = built-in)
    if font_handle != 0 {
        // Check if handle is valid (font exists)
        let font_index = (font_handle - 1) as usize;
        if font_index >= state.fonts.len() {
            warn!("font_bind: invalid font handle {}", font_handle);
            return;
        }
    }

    state.current_font = font_handle;
}
