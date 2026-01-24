use crate::graphics::{FORMAT_NORMAL, FORMAT_TANGENT};

/// Get human-readable name for a render mode
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Lambert",
        1 => "Matcap",
        2 => "MR-Blinn-Phong",
        3 => "Blinn-Phong",
        _ => "Unknown",
    }
}

/// Get the number of shader permutations for a render mode
#[cfg(test)]
pub fn shader_count_for_mode(mode: u8) -> usize {
    match mode {
        // Mode 0: 16 (no tangent) + 8 (tangent+normal) = 24
        0 => 24,
        // Modes 1-3: 8 (normal, no tangent) + 8 (normal+tangent) = 16
        1..=3 => 16,
        _ => 0,
    }
}

/// Get all valid vertex formats for a render mode
pub fn valid_formats_for_mode(mode: u8) -> Vec<u8> {
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
        // Valid: 4-7, 12-15, 20-23, 28-31
        1..=3 => (0..32)
            .filter(|&f| f & FORMAT_NORMAL != 0)
            .filter(tangent_valid)
            .collect(),
        _ => vec![],
    }
}
