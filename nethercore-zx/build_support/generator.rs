//! Shader generation and validation logic used by the build script.

use super::formats::{FORMAT_NORMAL, FormatFlags};
use super::{snippets, sources};

/// Generate a shader for a specific mode and vertex format
pub(crate) fn generate_shader(mode: u8, format: u8) -> Result<String, String> {
    // Validate mode
    if mode > 3 {
        return Err(format!("Invalid render mode: {} (must be 0-3)", mode));
    }

    // Get the appropriate template
    let template = match mode {
        0 => sources::TEMPLATE_MODE0,
        1 => sources::TEMPLATE_MODE1,
        _ => "", // Modes 2-3 use BLINNPHONG_COMMON
    };

    let flags = FormatFlags::from_bits(format);

    // Validate format constraints
    if mode > 0 && !flags.has_normal {
        return Err(format!(
            "Render mode {} requires NORMAL flag, but format {} doesn't have it",
            mode, format
        ));
    }
    if flags.has_tangent && !flags.has_normal {
        return Err(format!(
            "TANGENT flag requires NORMAL flag, but format {} doesn't have it",
            format
        ));
    }

    // Build shader by combining common code + mode-specific template
    let mut shader = String::new();
    shader.push_str(sources::COMMON);
    shader.push('\n');

    if mode >= 2 {
        shader.push_str(sources::BLINNPHONG_COMMON);
    } else {
        shader.push_str(template);
    }
    shader.push('\n');

    // Replace vertex input placeholders
    shader = shader.replace("//VIN_UV", if flags.has_uv { snippets::VIN_UV } else { "" });
    shader = shader.replace(
        "//VIN_COLOR",
        if flags.has_color {
            snippets::VIN_COLOR
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VIN_NORMAL",
        if flags.has_normal {
            snippets::VIN_NORMAL
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VIN_SKINNED",
        if flags.has_skinned {
            snippets::VIN_SKINNED
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VIN_TANGENT",
        if flags.has_tangent {
            snippets::VIN_TANGENT
        } else {
            ""
        },
    );

    // Replace vertex output placeholders
    shader = shader.replace(
        "//VOUT_UV",
        if flags.has_uv { snippets::VOUT_UV } else { "" },
    );
    shader = shader.replace(
        "//VOUT_COLOR",
        if flags.has_color {
            snippets::VOUT_COLOR
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VOUT_WORLD_NORMAL",
        if flags.has_normal {
            snippets::VOUT_WORLD_NORMAL
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VOUT_VIEW_NORMAL",
        if flags.has_normal {
            snippets::VOUT_VIEW_NORMAL
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VOUT_VIEW_POS",
        if mode == 1 && flags.has_normal {
            snippets::VOUT_VIEW_POS
        } else {
            ""
        },
    );

    // Camera position is needed for view-dependent reflection sampling.
    // Modes 2-3 always need it; mode 0 needs it when normals are present (environment reflection blend).
    let needs_camera_pos = mode >= 2 || (mode == 0 && flags.has_normal);
    shader = shader.replace(
        "//VOUT_CAMERA_POS",
        if needs_camera_pos {
            snippets::VOUT_CAMERA_POS
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VOUT_TANGENT",
        if flags.has_tangent {
            snippets::VOUT_TANGENT
        } else {
            ""
        },
    );
    // Mode 1 with tangent needs view-space tangent for matcap normal mapping
    shader = shader.replace(
        "//VOUT_VIEW_TANGENT",
        if mode == 1 && flags.has_tangent {
            snippets::VOUT_VIEW_TANGENT
        } else {
            ""
        },
    );

    // Replace vertex shader code placeholders
    shader = shader.replace("//VS_UV", if flags.has_uv { snippets::VS_UV } else { "" });
    shader = shader.replace(
        "//VS_COLOR",
        if flags.has_color {
            snippets::VS_COLOR
        } else {
            ""
        },
    );

    // Normal handling depends on skinning
    if flags.has_normal && !flags.has_skinned {
        shader = shader.replace("//VS_WORLD_NORMAL", snippets::VS_WORLD_NORMAL);
        shader = shader.replace("//VS_VIEW_NORMAL", snippets::VS_VIEW_NORMAL);
    } else if flags.has_normal && flags.has_skinned {
        shader = shader.replace("//VS_WORLD_NORMAL", snippets::VS_WORLD_NORMAL_SKINNED);
        shader = shader.replace("//VS_VIEW_NORMAL", snippets::VS_VIEW_NORMAL_SKINNED);
    } else {
        shader = shader.replace("//VS_WORLD_NORMAL", "");
        shader = shader.replace("//VS_VIEW_NORMAL", "");
    }

    // Tangent handling depends on skinning
    if flags.has_tangent && !flags.has_skinned {
        shader = shader.replace("//VS_TANGENT", snippets::VS_TANGENT);
    } else if flags.has_tangent && flags.has_skinned {
        shader = shader.replace("//VS_TANGENT", snippets::VS_TANGENT_SKINNED);
    } else {
        shader = shader.replace("//VS_TANGENT", "");
    }

    // Mode 1 with tangent needs view-space tangent for matcap normal mapping
    if mode == 1 && flags.has_tangent && !flags.has_skinned {
        shader = shader.replace("//VS_VIEW_TANGENT", snippets::VS_VIEW_TANGENT);
    } else if mode == 1 && flags.has_tangent && flags.has_skinned {
        shader = shader.replace("//VS_VIEW_TANGENT", snippets::VS_VIEW_TANGENT_SKINNED);
    } else {
        shader = shader.replace("//VS_VIEW_TANGENT", "");
    }

    // View position (mode 1 only, for perspective-correct matcap)
    if mode == 1 && flags.has_normal {
        shader = shader.replace("//VS_VIEW_POS", snippets::VS_VIEW_POS);
    } else {
        shader = shader.replace("//VS_VIEW_POS", "");
    }

    // Camera position extraction (modes 2-3, and mode 0 when normals are present)
    if needs_camera_pos {
        shader = shader.replace("//VS_CAMERA_POS", snippets::VS_CAMERA_POS);
    } else {
        shader = shader.replace("//VS_CAMERA_POS", "");
    }

    // Handle skinning with nested replacements
    if flags.has_skinned {
        let mut skinned_code = snippets::VS_SKINNED.to_string();
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_UNPACK_NORMAL",
            if flags.has_normal {
                snippets::VS_SKINNED_UNPACK_NORMAL
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_NORMAL",
            if flags.has_normal {
                snippets::VS_SKINNED_NORMAL
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_FINAL_NORMAL",
            if flags.has_normal {
                snippets::VS_SKINNED_FINAL_NORMAL
            } else {
                ""
            },
        );
        // Tangent skinning placeholders
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_UNPACK_TANGENT",
            if flags.has_tangent {
                snippets::VS_SKINNED_UNPACK_TANGENT
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_TANGENT",
            if flags.has_tangent {
                snippets::VS_SKINNED_TANGENT
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_FINAL_TANGENT",
            if flags.has_tangent {
                snippets::VS_SKINNED_FINAL_TANGENT
            } else {
                ""
            },
        );
        shader = shader.replace("//VS_SKINNED", &skinned_code);
        shader = shader.replace("//VS_POSITION", snippets::VS_POSITION_SKINNED);
    } else {
        shader = shader.replace("//VS_SKINNED", "");
        shader = shader.replace("//VS_POSITION", snippets::VS_POSITION_UNSKINNED);
    }

    // Replace fragment shader placeholders (mode-specific)
    match mode {
        0 => {
            shader = shader.replace(
                "//FS_COLOR",
                if flags.has_color {
                    snippets::FS_COLOR
                } else {
                    ""
                },
            );
            shader = shader.replace("//FS_UV", if flags.has_uv { snippets::FS_UV } else { "" });
            // Tangent normal mapping requires both tangent data and UVs
            let ambient = if flags.has_tangent && flags.has_uv {
                snippets::FS_AMBIENT_TANGENT
            } else if flags.has_normal {
                snippets::FS_AMBIENT
            } else {
                ""
            };
            shader = shader.replace("//FS_AMBIENT", ambient);
            shader = shader.replace(
                "//FS_NORMAL",
                if flags.has_normal {
                    snippets::FS_NORMAL
                } else {
                    ""
                },
            );
        }
        1 => {
            shader = shader.replace(
                "//FS_COLOR",
                if flags.has_color {
                    snippets::FS_COLOR
                } else {
                    ""
                },
            );
            shader = shader.replace("//FS_UV", if flags.has_uv { snippets::FS_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");
            // Matcap shading normal: use TBN+normal map if tangent data and UVs are present
            let matcap_normal = if flags.has_tangent && flags.has_uv {
                snippets::FS_MATCAP_SHADING_NORMAL_TANGENT
            } else {
                snippets::FS_MATCAP_SHADING_NORMAL
            };
            shader = shader.replace("//FS_MATCAP_SHADING_NORMAL", matcap_normal);
        }
        2 => {
            shader = shader.replace(
                "//FS_COLOR",
                if flags.has_color {
                    snippets::FS_ALBEDO_COLOR
                } else {
                    ""
                },
            );
            shader = shader.replace(
                "//FS_UV",
                if flags.has_uv {
                    snippets::FS_ALBEDO_UV
                } else {
                    ""
                },
            );
            shader = shader.replace("//FS_AMBIENT", "");
            // Shading normal: use TBN+normal map if tangent data and UVs are present
            let shading_normal = if flags.has_tangent && flags.has_uv {
                snippets::FS_SHADING_NORMAL_TANGENT
            } else {
                snippets::FS_SHADING_NORMAL
            };
            shader = shader.replace("//FS_SHADING_NORMAL", shading_normal);
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FACTOR",
                "let diffuse_factor = 1.0 - value0;",
            );
            shader = shader.replace(
                "//FS_MODE2_3_SHININESS",
                "let shininess = mix(1.0, 256.0, 1.0 - value1);",
            );
            shader = shader.replace("//FS_MODE2_3_ROUGHNESS", "let roughness = value1;");
            shader = shader.replace(
                "//FS_MODE2_3_SPECULAR_COLOR",
                "let specular_color = mix(vec3<f32>(0.04), albedo, value0);",
            );
            if flags.has_uv {
                // Mode 2: MRE texture sampling with override flag support (uses shared constant)
                shader = shader.replace("//FS_MODE2_3_TEXTURES", snippets::FS_MODE2_3_TEXTURES_UV);
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
            // Mode 2: Energy conservation - diffuse reduced by Fresnel (roughness-dependent)
            // value1 contains roughness at this point (before conversion to shininess)
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FRESNEL",
                "let diffuse_fresnel = mix(one_minus_F, vec3<f32>(1.0), value1);",
            );
        }
        3 => {
            shader = shader.replace(
                "//FS_COLOR",
                if flags.has_color {
                    snippets::FS_ALBEDO_COLOR
                } else {
                    ""
                },
            );
            shader = shader.replace(
                "//FS_UV",
                if flags.has_uv {
                    snippets::FS_ALBEDO_UV
                } else {
                    ""
                },
            );
            shader = shader.replace("//FS_AMBIENT", "");
            // Shading normal: use TBN+normal map if tangent data and UVs are present
            let shading_normal = if flags.has_tangent && flags.has_uv {
                snippets::FS_SHADING_NORMAL_TANGENT
            } else {
                snippets::FS_SHADING_NORMAL
            };
            shader = shader.replace("//FS_SHADING_NORMAL", shading_normal);
            shader = shader.replace("//FS_MODE2_3_DIFFUSE_FACTOR", "let diffuse_factor = 1.0;");
            shader = shader.replace(
                "//FS_MODE2_3_SHININESS",
                "let shininess = mix(1.0, 256.0, value1);",
            );
            shader = shader.replace(
                "//FS_MODE2_3_ROUGHNESS",
                "let roughness = 1.0 - (shininess - 1.0) / 255.0;",
            );
            // Mode 3 uses INVERTED spec_damping: 0 = full specular, 255 = no specular
            // This is beginner-friendly: default of 0 gives visible highlights
            // uniform_set_1 format: 0xRRGGBBRP (big-endian, same as color_rgba8)
            // Mode 3 specular color: supports both texture and uniform sources
            // Uses unpack_specular_rgb() helper to avoid code duplication
            let specular = if flags.has_uv {
                // With UV: check flag to decide between texture and uniform
                r#"var specular_color: vec3<f32>;
    if has_flag(shading.flags, FLAG_USE_UNIFORM_SPECULAR) {
        specular_color = unpack_specular_rgb(shading.uniform_set_1) * (1.0 - value0);
    } else {
        specular_color = sample_filtered(slot2, shading.flags, in.uv).rgb * (1.0 - value0);
    }"#
            } else {
                // Without UV: always use uniform specular color
                "var specular_color = unpack_specular_rgb(shading.uniform_set_1) * (1.0 - value0);"
            };
            shader = shader.replace("//FS_MODE2_3_SPECULAR_COLOR", specular);
            if flags.has_uv {
                // Mode 3: slot1 texture sampling with override flag support (uses shared constant)
                shader = shader.replace("//FS_MODE2_3_TEXTURES", snippets::FS_MODE2_3_TEXTURES_UV);
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
            // Mode 3: No energy conservation - full diffuse always (artistic freedom)
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FRESNEL",
                "let diffuse_fresnel = vec3<f32>(1.0);",
            );
        }
        _ => {}
    }

    Ok(shader)
}

/// Validate a shader using naga
pub(crate) fn validate_shader(source: &str, mode: u8, format: u8) -> Result<(), String> {
    // Parse the WGSL source
    let module = naga::front::wgsl::parse_str(source).map_err(|e| {
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

/// Validate a shader using naga (generic version for named shaders)
pub(crate) fn validate_shader_generic(source: &str, name: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|e| format!("WGSL parse error for {}: {:?}", name, e))?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .map_err(|e| format!("Validation error for {}: {:?}", name, e))?;

    Ok(())
}

#[allow(dead_code)]
fn _assert_constants_match() {
    // This exists to keep lint warnings useful if we ever change flags here.
    let _ = FORMAT_NORMAL;
}
