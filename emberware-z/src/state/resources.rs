//! Resource types pending GPU upload

/// Custom bitmap font definition
#[derive(Debug, Clone)]
pub struct Font {
    /// Texture handle for the font atlas
    pub texture: u32,
    /// Width of each glyph in pixels (for fixed-width fonts)
    pub char_width: u8,
    /// Height of each glyph in pixels
    pub char_height: u8,
    /// First codepoint in the font
    pub first_codepoint: u32,
    /// Number of characters in the font
    pub char_count: u32,
    /// Optional per-character widths for variable-width fonts (None = fixed-width)
    pub char_widths: Option<Vec<u8>>,
}

/// Pending texture load request
#[derive(Debug)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Pending mesh load request (unpacked f32 data from user)
#[derive(Debug)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,            // Vertex format flags (0-15, NO FORMAT_PACKED)
    pub vertex_data: Vec<f32>, // Unpacked f32 data
    pub index_data: Option<Vec<u16>>,
}

/// Pending packed mesh load request (packed bytes from procedural gen or power users)
#[derive(Debug)]
pub struct PendingMeshPacked {
    pub handle: u32,
    pub format: u8,           // Vertex format flags (0-15, NO FORMAT_PACKED)
    pub vertex_data: Vec<u8>, // Packed bytes (f16, snorm16, unorm8)
    pub index_data: Option<Vec<u16>>,
}
