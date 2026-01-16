use super::*;
use crate::graphics::{FORMAT_COLOR, FORMAT_UV};

#[test]
fn test_shader_generation_mode0() {
    // Mode 0 should support all 24 valid formats
    for format in valid_formats_for_mode(0) {
        let shader = generate_shader(0, format).expect("Mode 0 should support valid formats");
        assert!(!shader.is_empty());
        assert!(shader.contains("@vertex"));
        assert!(shader.contains("@fragment"));
    }
}

#[test]
fn test_shader_generation_mode1() {
    // Mode 1 should only support formats with NORMAL
    for format in valid_formats_for_mode(1) {
        let shader = generate_shader(1, format).expect("Mode 1 should support formats with NORMAL");
        assert!(!shader.is_empty());
        assert!(shader.contains("matcap"));
    }
}

#[test]
fn test_mode1_without_normals_returns_error() {
    // Mode 1 without normals should return an error
    let result = generate_shader(1, 0); // Format 0 has no NORMAL flag
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        ShaderGenError::MissingNormalFlag { mode: 1, format: 0 }
    );
}

#[test]
fn test_invalid_render_mode_returns_error() {
    // Invalid render modes should return an error
    let result = generate_shader(4, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ShaderGenError::InvalidRenderMode(4));

    let result = generate_shader(255, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ShaderGenError::InvalidRenderMode(255));
}

#[test]
fn test_get_template_returns_error_for_invalid_mode() {
    assert!(get_template(0).is_ok());
    assert!(get_template(3).is_ok());
    assert_eq!(
        get_template(4).unwrap_err(),
        ShaderGenError::InvalidRenderMode(4)
    );
    assert_eq!(
        get_template(100).unwrap_err(),
        ShaderGenError::InvalidRenderMode(100)
    );
}

#[test]
fn test_placeholder_replacement() {
    // Test that placeholders are replaced correctly
    let shader = generate_shader(0, FORMAT_UV | FORMAT_COLOR).unwrap();
    assert!(shader.contains("@location(1) uv"));
    assert!(shader.contains("@location(2) color"));
    assert!(!shader.contains("//VIN_UV"));
    assert!(!shader.contains("//VIN_COLOR"));
}

#[test]
fn test_shader_counts() {
    assert_eq!(shader_count_for_mode(0), 24);
    assert_eq!(shader_count_for_mode(1), 16);
    assert_eq!(shader_count_for_mode(2), 16);
    assert_eq!(shader_count_for_mode(3), 16);
}

#[test]
fn test_total_shader_count() {
    let total: usize = (0..4).map(shader_count_for_mode).sum();
    assert_eq!(total, 72); // 24 + 16 + 16 + 16 = 72
}

// =============================================================================
// Shader Compilation Tests (using naga)
// =============================================================================

/// Helper to compile a WGSL shader and validate it using naga
fn compile_and_validate_shader(mode: u8, format: u8) -> Result<(), String> {
    let shader_source = generate_shader(mode, format).map_err(|e| {
        format!(
            "Shader generation error for mode {} format {}: {}",
            mode, format, e
        )
    })?;

    // Parse the WGSL source
    let module = naga::front::wgsl::parse_str(&shader_source).map_err(|e| {
        format!(
            "WGSL parse error for mode {} format {}: {:?}",
            mode, format, e
        )
    })?;

    // Validate the module
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator.validate(&module).map_err(|e| {
        format!(
            "Validation error for mode {} format {}: {:?}",
            mode, format, e
        )
    })?;

    Ok(())
}

#[test]
fn test_compile_all_72_shaders() {
    let mut errors = Vec::new();

    // All modes: iterate through valid formats
    for mode in 0u8..=3 {
        for format in valid_formats_for_mode(mode) {
            if let Err(e) = compile_and_validate_shader(mode, format) {
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Shader compilation failed for {} shaders:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

#[test]
fn test_compile_mode0_all_formats() {
    for format in valid_formats_for_mode(0) {
        compile_and_validate_shader(0, format).unwrap_or_else(|e| panic!("{}", e));
    }
}

#[test]
fn test_compile_mode1_matcap() {
    for format in valid_formats_for_mode(1) {
        compile_and_validate_shader(1, format).unwrap_or_else(|e| panic!("{}", e));
    }
}

#[test]
fn test_compile_mode2_pbr() {
    for format in valid_formats_for_mode(2) {
        compile_and_validate_shader(2, format).unwrap_or_else(|e| panic!("{}", e));
    }
}

#[test]
fn test_compile_mode3_blinnphong() {
    for format in valid_formats_for_mode(3) {
        compile_and_validate_shader(3, format).unwrap_or_else(|e| panic!("{}", e));
    }
}

#[test]
fn test_compile_skinned_variants() {
    // Test all skinned formats: 8-15 (no tangent) and 28-31 (with tangent+normal)
    // Mode 0 supports all skinned formats
    for format in valid_formats_for_mode(0) {
        if format >= 8 {
            compile_and_validate_shader(0, format).unwrap_or_else(|e| panic!("{}", e));
        }
    }

    // Modes 1-3: skinned formats with NORMAL (12-15, 28-31)
    for mode in 1u8..=3 {
        for format in valid_formats_for_mode(mode) {
            if format >= 8 {
                compile_and_validate_shader(mode, format).unwrap_or_else(|e| panic!("{}", e));
            }
        }
    }
}

#[test]
fn test_shader_has_vertex_entry() {
    for mode in 0u8..=3 {
        for format in valid_formats_for_mode(mode) {
            let shader = generate_shader(mode, format).unwrap();
            assert!(
                shader.contains("fn vs("),
                "Mode {} format {} missing vertex entry point 'vs'",
                mode,
                format
            );
        }
    }
}

#[test]
fn test_shader_has_fragment_entry() {
    for mode in 0u8..=3 {
        for format in valid_formats_for_mode(mode) {
            let shader = generate_shader(mode, format).unwrap();
            assert!(
                shader.contains("fn fs("),
                "Mode {} format {} missing fragment entry point 'fs'",
                mode,
                format
            );
        }
    }
}

#[test]
fn test_no_unreplaced_placeholders() {
    for mode in 0u8..=3 {
        for format in valid_formats_for_mode(mode) {
            let shader = generate_shader(mode, format).unwrap();
            let placeholders = [
                "//VIN_UV",
                "//VIN_COLOR",
                "//VIN_NORMAL",
                "//VIN_SKINNED",
                "//VIN_TANGENT",
                "//VOUT_UV",
                "//VOUT_COLOR",
                "//VOUT_WORLD_NORMAL",
                "//VOUT_VIEW_NORMAL",
                "//VOUT_CAMERA_POS",
                "//VOUT_TANGENT",
                "//VOUT_VIEW_TANGENT",
                "//VS_UV",
                "//VS_COLOR",
                "//VS_WORLD_NORMAL",
                "//VS_VIEW_NORMAL",
                "//VS_CAMERA_POS",
                "//VS_SKINNED",
                "//VS_TANGENT",
                "//VS_VIEW_TANGENT",
                "//VS_POSITION",
                "//FS_COLOR",
                "//FS_UV",
                "//FS_AMBIENT",
                "//FS_NORMAL",
                "//FS_MRE",
                "//FS_MODE2_3_DIFFUSE_FACTOR",
                "//FS_MODE2_3_SHININESS",
                "//FS_MODE2_3_SPECULAR_COLOR",
                "//FS_MODE2_3_TEXTURES",
                "//FS_MODE2_3_ROUGHNESS",
            ];

            for placeholder in placeholders {
                assert!(
                    !shader.contains(placeholder),
                    "Mode {} format {} has unreplaced placeholder '{}'",
                    mode,
                    format,
                    placeholder
                );
            }
        }
    }
}
