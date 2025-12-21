//! C header generator

use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

use crate::model::FfiModel;

/// Load C helper template for a specific console
fn load_c_helpers(console: &str) -> Result<String> {
    let template_path = PathBuf::from(format!("tools/ffi-gen/templates/c_helpers_{}.h", console));

    // Try console-specific template first, fall back to generic
    if template_path.exists() {
        Ok(std::fs::read_to_string(&template_path)?)
    } else {
        let generic_path = PathBuf::from("tools/ffi-gen/templates/c_helpers.h");
        Ok(std::fs::read_to_string(&generic_path)?)
    }
}

/// Generate C header file from FFI model
pub fn generate_c_header(model: &FfiModel, console: &str) -> Result<String> {
    let mut output = String::new();

    let console_upper = console.to_uppercase();
    let header_guard = format!("EMBERWARE_{}_H", console_upper);

    // Header comment
    writeln!(output, "// GENERATED FILE - DO NOT EDIT")?;
    writeln!(output, "// Source: emberware/include/emberware_{}_ffi.rs", console)?;
    writeln!(output, "// Generator: tools/ffi-gen")?;
    writeln!(output)?;

    // Header guard
    writeln!(output, "#ifndef {}", header_guard)?;
    writeln!(output, "#define {}", header_guard)?;
    writeln!(output)?;

    // Includes
    writeln!(output, "#include <stdint.h>")?;
    writeln!(output, "#include <stdbool.h>")?;
    writeln!(output)?;

    // WASM attributes
    writeln!(output, "#define EWZX_EXPORT __attribute__((visibility(\"default\")))")?;
    writeln!(output, "#define EWZX_IMPORT __attribute__((import_module(\"env\")))")?;
    writeln!(output)?;

    // C++ compatibility
    writeln!(output, "#ifdef __cplusplus")?;
    writeln!(output, "extern \"C\" {{")?;
    writeln!(output, "#endif")?;
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
                writeln!(output, "/** {} */", line.trim())?;
            }
        }

        // Function declaration
        write!(output, "EWZX_IMPORT {} {}(", func.return_type.c_type, func.name)?;

        if func.params.is_empty() {
            write!(output, "void")?;
        } else {
            for (i, param) in func.params.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{} {}", param.ty.c_type, param.name)?;
            }
        }

        writeln!(output, ");")?;
        writeln!(output)?;
    }

    // Constants section
    if !model.constants.is_empty() {
        writeln!(output, "// =============================================================================")?;
        writeln!(output, "// Constants")?;
        writeln!(output, "// =============================================================================")?;
        writeln!(output)?;

        for module in &model.constants {
            writeln!(output, "// {} constants", module.name)?;
            for constant in &module.constants {
                let const_name = format!("EWZX_{}_{}",
                    module.name.to_uppercase(),
                    constant.name.to_uppercase()
                );
                writeln!(output, "#define {} {}", const_name, constant.value)?;
            }
            writeln!(output)?;
        }
    }

    // C++ compatibility close
    writeln!(output, "#ifdef __cplusplus")?;
    writeln!(output, "}}")?;
    writeln!(output, "#endif")?;
    writeln!(output)?;

    // Append manual helpers from template file
    writeln!(output)?;
    if let Ok(helpers) = load_c_helpers(console) {
        write!(output, "{}", helpers)?;
    } else {
        writeln!(output, "// Note: No manual helpers template found for console '{}'", console)?;
    }

    // Header guard close
    writeln!(output, "\n#endif /* {} */", header_guard)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Category, FfiFunction, Parameter, Type};

    #[test]
    fn test_generate_simple_header() {
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

        let header = generate_c_header(&model, "test").unwrap();

        assert!(header.contains("EWZX_IMPORT float test_fn(uint32_t x);"));
        assert!(header.contains("/** Test function */"));
        assert!(header.contains("#ifndef EMBERWARE_TEST_H"));
    }
}
