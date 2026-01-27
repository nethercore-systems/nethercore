//! 2D Drawing (Screen Space)

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Draw a sprite with the bound texture.
    ///
    /// # Arguments
    /// * `x`, `y` — Screen position in pixels (0,0 = top-left)
    /// * `w`, `h` — Sprite size in pixels
    pub fn draw_sprite(x: f32, y: f32, w: f32, h: f32);

    /// Draw a region of a sprite sheet.
    ///
    /// # Arguments
    /// * `src_x`, `src_y`, `src_w`, `src_h` — UV region (0.0-1.0)
    pub fn draw_sprite_region(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        src_x: f32,
        src_y: f32,
        src_w: f32,
        src_h: f32,
    );

    /// Draw a sprite with full control (rotation, origin, UV region).
    ///
    /// # Arguments
    /// * `origin_x`, `origin_y` — Rotation pivot point (in pixels from sprite top-left)
    /// * `angle_deg` — Rotation angle in degrees (clockwise)
    pub fn draw_sprite_ex(
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
    );

    /// Draw a solid color rectangle.
    pub fn draw_rect(x: f32, y: f32, w: f32, h: f32);

    /// Draw text with the current font.
    ///
    /// # Arguments
    /// * `ptr` — Pointer to UTF-8 string data
    /// * `len` — Length in bytes
    /// * `size` — Font size in pixels
    pub fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32);

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
    pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32);

    /// Draw a filled circle.
    ///
    /// # Arguments
    /// * `x`, `y` — Center position in screen pixels
    /// * `radius` — Circle radius in pixels
    ///
    /// Rendered as a 16-segment triangle fan.
    pub fn draw_circle(x: f32, y: f32, radius: f32);

    /// Draw a circle outline.
    ///
    /// # Arguments
    /// * `x`, `y` — Center position in screen pixels
    /// * `radius` — Circle radius in pixels
    /// * `thickness` — Line thickness in pixels
    ///
    /// Rendered as 16 line segments.
    pub fn draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32);

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
}
