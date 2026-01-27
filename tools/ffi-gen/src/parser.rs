//! Parser for Rust FFI declarations using syn

use anyhow::{Context, Result};
use quote::ToTokens;
use std::path::Path;
use syn::{ForeignItem, ForeignItemFn, Item, ItemForeignMod, ItemMod, ReturnType, Type as SynType};

use crate::model::{Category, Constant, ConstantModule, FfiFunction, FfiModel, Parameter, Type};

/// Files that don't contribute extern functions or constant modules for codegen.
/// `stubs.rs` contains non-WASM test stubs, `helpers.rs` and `colors.rs` contain
/// safe wrapper functions / color constants that aren't extern declarations.
const SKIP_FILES: &[&str] = &["mod.rs", "stubs.rs", "helpers.rs", "colors.rs"];

/// Parse FFI declarations from a Rust source file or directory of submodules.
///
/// If `path` is a directory, each `.rs` file (except SKIP_FILES) is parsed
/// individually and the filename (without extension) is used as the category
/// name for all functions found in that file.
///
/// If `path` is a single `.rs` file, falls back to the legacy single-file
/// parser that reads `// ====` section markers for category assignment.
pub fn parse_ffi_file(path: impl AsRef<Path>) -> Result<FfiModel> {
    let path = path.as_ref();

    if path.is_dir() {
        parse_ffi_directory(path)
    } else {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        parse_ffi_source(&content)
    }
}

/// Parse a directory of submodule `.rs` files into a unified FfiModel.
fn parse_ffi_directory(dir: &Path) -> Result<FfiModel> {
    let mut all_functions = Vec::new();
    let mut all_constants = Vec::new();
    let mut all_categories = Vec::new();

    // Collect and sort entries for deterministic output
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".rs") && !SKIP_FILES.contains(&name.as_str())
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let file_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let category_name = file_name
            .strip_suffix(".rs")
            .unwrap()
            .to_string();

        // Prettify category name: "system" -> "System Functions", etc.
        let pretty_category = prettify_category(&category_name);

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let file = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse {}", file_path.display()))?;

        let mut file_has_functions = false;

        for item in &file.items {
            match item {
                Item::ForeignMod(foreign_mod) => {
                    let (fns, _cats) = parse_foreign_mod_with_category(foreign_mod, &pretty_category)?;
                    if !fns.is_empty() {
                        file_has_functions = true;
                    }
                    all_functions.extend(fns);
                }
                Item::Mod(item_mod) => {
                    if let Some(const_mod) = parse_const_module(item_mod)? {
                        all_constants.push(const_mod);
                    }
                }
                _ => {}
            }
        }

        if file_has_functions {
            all_categories.push(Category {
                name: pretty_category,
                comment: String::new(),
            });
        }
    }

    Ok(FfiModel {
        functions: all_functions,
        constants: all_constants,
        categories: all_categories,
    })
}

/// Prettify a filename-based category name.
fn prettify_category(name: &str) -> String {
    // Map specific filenames to pretty category names
    match name {
        "system" => "System Functions".to_string(),
        "input" => "Input Functions".to_string(),
        "camera" => "Camera Functions".to_string(),
        "transform" => "Transform Functions".to_string(),
        "render" => "Render State Functions".to_string(),
        "viewport" => "Viewport Functions (Split-Screen)".to_string(),
        "pass" => "Render Pass Functions (Execution Barriers & Depth/Stencil Control)".to_string(),
        "texture" => "Texture Functions".to_string(),
        "mesh" => "Mesh Functions (Retained Mode)".to_string(),
        "procedural" => "Procedural Mesh Generation (init-only)".to_string(),
        "drawing" => "Immediate Mode 3D Drawing & Billboards".to_string(),
        "text" => "2D Drawing (Screen Space)".to_string(),
        "epu" => "Environment Processing Unit (EPU) — Instruction-Based API".to_string(),
        "material" => "Material Functions (Mode 2/3)".to_string(),
        "lighting" => "Lighting Functions (Mode 2/3)".to_string(),
        "skeleton" => "GPU Skinning".to_string(),
        "animation" => "Keyframe Animation".to_string(),
        "audio" => "Audio Functions".to_string(),
        "music" => "Unified Music API (PCM + Tracker)".to_string(),
        "assets" => "ROM Data Pack API (init-only)".to_string(),
        "embedded" => "Embedded Asset API".to_string(),
        "debug" => "Debug Inspection System".to_string(),
        "constants" => "Constants".to_string(),
        _ => {
            // Fallback: capitalize first letter
            let mut s = name.to_string();
            if let Some(c) = s.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            s
        }
    }
}

/// Parse extern "C" block, overriding category with the given name.
fn parse_foreign_mod_with_category(
    foreign_mod: &ItemForeignMod,
    category: &str,
) -> Result<(Vec<FfiFunction>, Vec<Category>)> {
    let mut functions = Vec::new();
    let categories = Vec::new();

    for item in &foreign_mod.items {
        if let ForeignItem::Fn(func) = item {
            let function = parse_foreign_function(func, category)?;
            functions.push(function);
        }
    }

    Ok((functions, categories))
}

/// Parse FFI declarations from Rust source code (legacy single-file mode)
pub fn parse_ffi_source(source: &str) -> Result<FfiModel> {
    let file = syn::parse_file(source).context("Failed to parse Rust source")?;

    let mut functions = Vec::new();
    let mut constants = Vec::new();
    let mut categories = Vec::new();

    // Process all items in the file
    for item in &file.items {
        match item {
            // extern "C" block
            Item::ForeignMod(foreign_mod) => {
                let (fns, cats) = parse_foreign_mod(foreign_mod)?;
                functions.extend(fns);
                categories.extend(cats);
            }
            // Constant modules (pub mod button { pub const UP: u32 = 0; })
            Item::Mod(item_mod) => {
                if let Some(const_mod) = parse_const_module(item_mod)? {
                    constants.push(const_mod);
                }
            }
            _ => {}
        }
    }

    // Assign categories based on comment sections
    assign_categories(&mut functions, &categories);

    Ok(FfiModel {
        functions,
        constants,
        categories,
    })
}

/// Parse extern "C" block (legacy: reads category from `// ====` markers)
fn parse_foreign_mod(foreign_mod: &ItemForeignMod) -> Result<(Vec<FfiFunction>, Vec<Category>)> {
    let mut functions = Vec::new();
    let mut categories = Vec::new();
    let mut current_category = String::from("System");

    for item in &foreign_mod.items {
        if let ForeignItem::Fn(func) = item {
            // Check for category comment before function
            let doc_comment = extract_doc_comments(&func.attrs);

            // Check if this is a category separator comment
            if let Some(category) = extract_category_from_comment(&doc_comment) {
                current_category = category.clone();
                categories.push(Category {
                    name: category,
                    comment: doc_comment.clone(),
                });
                continue;
            }

            let function = parse_foreign_function(func, &current_category)?;
            functions.push(function);
        }
    }

    Ok((functions, categories))
}

/// Parse a single foreign function
fn parse_foreign_function(func: &ForeignItemFn, category: &str) -> Result<FfiFunction> {
    let name = func.sig.ident.to_string();
    let doc_comment = extract_doc_comments(&func.attrs);

    // Parse parameters
    let mut params = Vec::new();
    for input in &func.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let param_name = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                ident.ident.to_string()
            } else {
                "arg".to_string()
            };

            let rust_type = type_to_string(&pat_type.ty);
            params.push(Parameter {
                name: param_name,
                ty: Type::new(&rust_type),
            });
        }
    }

    // Parse return type
    let return_type = match &func.sig.output {
        ReturnType::Default => Type::new("()"),
        ReturnType::Type(_, ty) => Type::new(&type_to_string(ty)),
    };

    Ok(FfiFunction {
        name,
        params,
        return_type,
        doc_comment,
        category: category.to_string(),
    })
}

/// Parse a constant module (pub mod name { pub const X: u32 = 0; })
fn parse_const_module(item_mod: &ItemMod) -> Result<Option<ConstantModule>> {
    // Only process pub modules with content
    if !matches!(item_mod.vis, syn::Visibility::Public(_)) {
        return Ok(None);
    }

    let module_name = item_mod.ident.to_string();
    let mut constants_vec = Vec::new();

    if let Some((_, items)) = &item_mod.content {
        for item in items {
            if let Item::Const(const_item) = item {
                if matches!(const_item.vis, syn::Visibility::Public(_)) {
                    let name = const_item.ident.to_string();
                    let value = const_item.expr.to_token_stream().to_string();
                    let ty = type_to_string(&const_item.ty);

                    constants_vec.push(Constant { name, value, ty });
                }
            }
        }
    }

    if constants_vec.is_empty() {
        return Ok(None);
    }

    Ok(Some(ConstantModule {
        name: module_name,
        constants: constants_vec,
    }))
}

/// Extract documentation comments from attributes
fn extract_doc_comments(attrs: &[syn::Attribute]) -> String {
    let mut doc = String::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let line = lit_str.value();
                        if !doc.is_empty() {
                            doc.push('\n');
                        }
                        doc.push_str(line.trim());
                    }
                }
            }
        }
    }

    doc
}

/// Extract category name from a comment (e.g., "// ========= System Functions =========")
fn extract_category_from_comment(comment: &str) -> Option<String> {
    let trimmed = comment.trim();

    // Look for category separator pattern: "==== Name ===="
    if trimmed.contains("====") {
        let parts: Vec<&str> = trimmed.split("====").collect();
        if parts.len() >= 2 {
            let category = parts[1].trim();
            if !category.is_empty() {
                return Some(category.to_string());
            }
        }
    }

    None
}

/// Assign categories to functions based on position
fn assign_categories(_functions: &mut [FfiFunction], _categories: &[Category]) {
    // Already assigned during parsing
}

/// Convert syn::Type to string representation
fn type_to_string(ty: &SynType) -> String {
    match ty {
        SynType::Path(type_path) => {
            // Simple type like u32, f32
            if type_path.path.segments.len() == 1 {
                type_path.path.segments[0].ident.to_string()
            } else {
                ty.to_token_stream().to_string()
            }
        }
        SynType::Ptr(type_ptr) => {
            // Pointer type (*const T or *mut T)
            let mutability = if type_ptr.mutability.is_some() {
                "*mut "
            } else {
                "*const "
            };
            format!("{}{}", mutability, type_to_string(&type_ptr.elem))
        }
        SynType::Tuple(type_tuple) => {
            // Unit type ()
            if type_tuple.elems.is_empty() {
                "()".to_string()
            } else {
                ty.to_token_stream().to_string()
            }
        }
        _ => ty.to_token_stream().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_extern() {
        let source = r#"
            extern "C" {
                /// Test function
                pub fn test_fn(x: u32, y: f32) -> u32;
            }
        "#;

        let model = parse_ffi_source(source).unwrap();
        assert_eq!(model.functions.len(), 1);

        let func = &model.functions[0];
        assert_eq!(func.name, "test_fn");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name, "x");
        assert_eq!(func.params[0].ty.rust_type, "u32");
        assert_eq!(func.return_type.rust_type, "u32");
    }

    #[test]
    fn test_parse_const_module() {
        let source = r#"
            pub mod button {
                pub const UP: u32 = 0;
                pub const DOWN: u32 = 1;
            }
        "#;

        let model = parse_ffi_source(source).unwrap();
        assert_eq!(model.constants.len(), 1);

        let module = &model.constants[0];
        assert_eq!(module.name, "button");
        assert_eq!(module.constants.len(), 2);
        assert_eq!(module.constants[0].name, "UP");
        assert_eq!(model.constants[0].constants[0].value, "0");
    }

    #[test]
    fn test_type_conversion() {
        assert_eq!(
            type_to_string(&syn::parse_str::<SynType>("u32").unwrap()),
            "u32"
        );
        assert_eq!(
            type_to_string(&syn::parse_str::<SynType>("()").unwrap()),
            "()"
        );
    }

    #[test]
    fn test_prettify_category() {
        assert_eq!(prettify_category("system"), "System Functions");
        assert_eq!(prettify_category("input"), "Input Functions");
        assert_eq!(prettify_category("unknown"), "Unknown");
    }
}
