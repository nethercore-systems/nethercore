//! Vertex-format bit flags and helpers used by the build script.

// Vertex format flags (must match vertex.rs)
pub(crate) const FORMAT_UV: u8 = 1;
pub(crate) const FORMAT_COLOR: u8 = 2;
pub(crate) const FORMAT_NORMAL: u8 = 4;
pub(crate) const FORMAT_SKINNED: u8 = 8;
pub(crate) const FORMAT_TANGENT: u8 = 16;

#[derive(Clone, Copy, Debug)]
pub(crate) struct FormatFlags {
    pub(crate) has_uv: bool,
    pub(crate) has_color: bool,
    pub(crate) has_normal: bool,
    pub(crate) has_skinned: bool,
    pub(crate) has_tangent: bool,
}

impl FormatFlags {
    pub(crate) fn from_bits(format: u8) -> Self {
        Self {
            has_uv: format & FORMAT_UV != 0,
            has_color: format & FORMAT_COLOR != 0,
            has_normal: format & FORMAT_NORMAL != 0,
            has_skinned: format & FORMAT_SKINNED != 0,
            has_tangent: format & FORMAT_TANGENT != 0,
        }
    }
}

/// Get valid formats for a render mode
/// Mode 0: all 32 formats (0-31)
/// Modes 1-3: 16 formats each (must have NORMAL, optionally TANGENT)
pub(crate) fn valid_formats_for_mode(mode: u8) -> Vec<u8> {
    // Tangent requires normal: filter out formats with TANGENT but without NORMAL
    let tangent_valid = |f: &u8| {
        let has_tangent = (*f & FORMAT_TANGENT) != 0;
        let has_normal = (*f & FORMAT_NORMAL) != 0;
        !has_tangent || has_normal // tangent requires normal
    };

    match mode {
        // Mode 0: all combinations except invalid tangent-without-normal
        // Valid: 0-15 (no tangent), 20-23 (tangent+normal), 28-31 (tangent+normal+skinned)
        0 => (0..32).filter(tangent_valid).collect(),
        // Modes 1-3: require NORMAL, plus tangent validation
        1..=3 => (0..32)
            .filter(|&f| f & FORMAT_NORMAL != 0)
            .filter(tangent_valid)
            .collect(),
        _ => vec![],
    }
}
