//! C header generator

use anyhow::Result;
use std::fmt::Write as FmtWrite;

use crate::model::FfiModel;

/// Generate C header file from FFI model
pub fn generate_c_header(model: &FfiModel) -> Result<String> {
    let mut output = String::new();

    // Header comment
    writeln!(output, "// GENERATED FILE - DO NOT EDIT")?;
    writeln!(output, "// Source: emberware/include/emberware-zx-ffi.rs")?;
    writeln!(output, "// Generator: tools/ffi-gen")?;
    writeln!(output)?;

    // Header guard
    writeln!(output, "#ifndef EMBERWARE_ZX_H")?;
    writeln!(output, "#define EMBERWARE_ZX_H")?;
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

    // TODO: Append manual helpers from template file
    writeln!(output, "// =============================================================================")?;
    writeln!(output, "// MANUALLY MAINTAINED HELPERS")?;
    writeln!(output, "// =============================================================================")?;
    writeln!(output, "// TODO: Load from templates/c_helpers.h")?;
    writeln!(output)?;

    // Header guard close
    writeln!(output, "#endif /* EMBERWARE_ZX_H */")?;

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

        let header = generate_c_header(&model).unwrap();

        assert!(header.contains("EWZX_IMPORT float test_fn(uint32_t x);"));
        assert!(header.contains("/** Test function */"));
        assert!(header.contains("#ifndef EMBERWARE_ZX_H"));
    }
}
