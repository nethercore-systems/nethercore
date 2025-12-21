//! Zig binding generator

use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

use crate::model::FfiModel;

/// Load Zig helper template for a specific console
fn load_zig_helpers(console: &str) -> Result<String> {
    let template_path = PathBuf::from(format!("tools/ffi-gen/templates/zig_helpers_{}.zig", console));

    // Try console-specific template first, fall back to generic
    if template_path.exists() {
        Ok(std::fs::read_to_string(&template_path)?)
    } else {
        let generic_path = PathBuf::from("tools/ffi-gen/templates/zig_helpers.zig");
        Ok(std::fs::read_to_string(&generic_path)?)
    }
}

/// Convert constant name to Zig snake_case
fn to_zig_constant_name(name: &str) -> String {
    // SCREAMING_CASE -> snake_case
    name.to_lowercase()
}

/// Generate Zig bindings file from FFI model
pub fn generate_zig_bindings(model: &FfiModel, console: &str) -> Result<String> {
    let mut output = String::new();

    // Header comment
    writeln!(output, "// GENERATED FILE - DO NOT EDIT")?;
    writeln!(output, "// Source: nethercore/include/{}.rs", console)?;
    writeln!(output, "// Generator: tools/ffi-gen")?;
    writeln!(output)?;

    // Group functions by category
    let mut current_category = String::new();
    for func in &model.functions {
        // Category header
        if func.category != current_category {
            current_category = func.category.clone();
            writeln!(output, "// =============================================================================")?;
            writeln!(output, "// {}", current_category)?;
            writeln!(output, "// =============================================================================")?;
            writeln!(output)?;
        }

        // Documentation comment
        if !func.doc_comment.is_empty() {
            for line in func.doc_comment.lines() {
                writeln!(output, "/// {}", line.trim())?;
            }
        }

        // Function declaration
        write!(output, "pub extern \"C\" fn {}(", func.name)?;

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")?;
            }
            write!(output, "{}: {}", param.name, param.ty.zig_type)?;
        }

        writeln!(output, ") {};", func.return_type.zig_type)?;
        writeln!(output)?;
    }

    // Constants section
    if !model.constants.is_empty() {
        writeln!(output, "// =============================================================================")?;
        writeln!(output, "// Constants")?;
        writeln!(output, "// =============================================================================")?;
        writeln!(output)?;

        for module in &model.constants {
            // Create struct for constant grouping
            let struct_name = if module.name == "color" {
                "color".to_string()
            } else {
                // Convert to PascalCase for struct names
                let mut result = String::new();
                for part in module.name.split('_') {
                    if !part.is_empty() {
                        result.push(part.chars().next().unwrap().to_uppercase().next().unwrap());
                        result.push_str(&part[1..]);
                    }
                }
                result
            };

            writeln!(output, "pub const {} = struct {{", struct_name)?;
            for constant in &module.constants {
                let const_name = to_zig_constant_name(&constant.name);
                writeln!(output, "    pub const {}: {} = {};",
                    const_name, constant.ty, constant.value)?;
            }
            writeln!(output, "}};")?;
            writeln!(output)?;
        }
    }

    // Append manual helpers from template file
    writeln!(output)?;
    if let Ok(helpers) = load_zig_helpers(console) {
        write!(output, "{}", helpers)?;
    } else {
        writeln!(output, "// Note: No manual helpers template found for console '{}'", console)?;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FfiFunction, Parameter, Type};

    #[test]
    fn test_generate_simple_zig() {
        let model = FfiModel {
            functions: vec![FfiFunction {
                name: "test_fn".to_string(),
                params: vec![Parameter {
                    name: "x".to_string(),
                    ty: Type::new("u32"),
                }],
                return_type: Type::new("f32"),
                doc_comment: "Test function".to_string(),
                category: "System".to_string(),
            }],
            constants: vec![],
            categories: vec![],
        };

        let zig = generate_zig_bindings(&model, "test").unwrap();

        assert!(zig.contains("pub extern \"C\" fn test_fn(x: u32) f32;"));
        assert!(zig.contains("/// Test function"));
    }

    #[test]
    fn test_zig_constant_name_conversion() {
        assert_eq!(to_zig_constant_name("UP"), "up");
        assert_eq!(to_zig_constant_name("LEFT_STICK"), "left_stick");
    }
}
