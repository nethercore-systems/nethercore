//! Helpers for extracting portions of the shared WGSL and generating template-based shaders.

use super::sources;

/// Extract bindings section from the common shader sources (up to "// Data Unpacking Utilities")
pub(crate) fn extract_common_bindings() -> &'static str {
    // Find the section by looking for the section title (not the full header line)
    let marker = "// Data Unpacking Utilities";
    if let Some(marker_pos) = sources::COMMON.find(marker) {
        // Find the start of the section divider line (=== line) before the marker
        let section_start = sources::COMMON[..marker_pos]
            .rfind("// ===")
            .unwrap_or(marker_pos);
        &sources::COMMON[..section_start]
    } else {
        panic!("Could not find '{}' in common shader sources", marker);
    }
}

/// Extract utility functions from the common shader sources (from Data Unpacking to Unified Vertex Input)
pub(crate) fn extract_common_utilities() -> &'static str {
    let start_marker = "// Data Unpacking Utilities";
    let end_marker = "// Unified Vertex Input/Output";

    let start_pos = sources::COMMON
        .find(start_marker)
        .expect("Could not find start marker in common shader sources");
    let start = sources::COMMON[..start_pos]
        .rfind("// ===")
        .unwrap_or(start_pos);

    let end_pos = sources::COMMON
        .find(end_marker)
        .expect("Could not find end marker in common shader sources");
    let end = sources::COMMON[..end_pos]
        .rfind("// ===")
        .unwrap_or(end_pos);

    &sources::COMMON[start..end]
}

/// Generate the fullscreen environment shader from the common shader sources + env_template
pub(crate) fn generate_environment_shader() -> String {
    let mut shader = String::new();
    shader.push_str("// Auto-generated from common shader sources + env_template.wgsl\n");
    shader.push_str("// DO NOT EDIT - regenerate with cargo build\n\n");
    shader.push_str(extract_common_bindings());
    shader.push_str(extract_common_utilities());
    shader.push_str(sources::ENV_TEMPLATE);
    shader
}

/// Generate quad.wgsl from the common shader sources + quad_template
pub(crate) fn generate_quad_shader() -> String {
    let mut shader = String::new();
    shader.push_str("// Auto-generated from common shader sources + quad_template.wgsl\n");
    shader.push_str("// DO NOT EDIT - regenerate with cargo build\n\n");
    shader.push_str(extract_common_bindings());
    shader.push_str(extract_common_utilities());
    shader.push_str(sources::QUAD_TEMPLATE);
    shader
}
