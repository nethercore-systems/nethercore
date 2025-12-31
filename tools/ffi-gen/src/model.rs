//! Intermediate representation for FFI declarations

use serde::{Deserialize, Serialize};

/// Complete FFI model extracted from Rust source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiModel {
    pub functions: Vec<FfiFunction>,
    pub constants: Vec<ConstantModule>,
    pub categories: Vec<Category>,
}

/// Single FFI function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub doc_comment: String,
    pub category: String,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

/// Type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Type {
    pub rust_type: String,
    pub c_type: String,
    pub zig_type: String,
}

impl Type {
    /// Create a new type with automatic C and Zig mapping
    pub fn new(rust_type: &str) -> Self {
        Self {
            rust_type: rust_type.to_string(),
            c_type: map_rust_to_c(rust_type),
            zig_type: map_rust_to_zig(rust_type),
        }
    }
}

/// Module containing constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantModule {
    pub name: String,
    pub constants: Vec<Constant>,
}

/// Single constant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub name: String,
    pub value: String,
    pub ty: String,
}

/// Function category for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub comment: String,
}

/// Map Rust type to C type
pub fn map_rust_to_c(rust_type: &str) -> String {
    if let Some(inner) = rust_type.strip_prefix("*const ") {
        return format!("const {}*", map_rust_to_c(inner));
    } else if let Some(inner) = rust_type.strip_prefix("*mut ") {
        return format!("{}*", map_rust_to_c(inner));
    }

    match rust_type {
        "u8" => "uint8_t".to_string(),
        "u16" => "uint16_t".to_string(),
        "u32" => "uint32_t".to_string(),
        "u64" => "uint64_t".to_string(),
        "i8" => "int8_t".to_string(),
        "i16" => "int16_t".to_string(),
        "i32" => "int32_t".to_string(),
        "i64" => "int64_t".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        "()" => "void".to_string(),
        _ => rust_type.to_string(),
    }
}

/// Map Rust type to Zig type
pub fn map_rust_to_zig(rust_type: &str) -> String {
    if let Some(inner) = rust_type.strip_prefix("*const ") {
        return format!("[*]const {}", map_rust_to_zig(inner));
    } else if let Some(inner) = rust_type.strip_prefix("*mut ") {
        return format!("[*]{}", map_rust_to_zig(inner));
    }

    match rust_type {
        "()" => "void".to_string(),
        // Most Rust primitive types are identical in Zig
        _ => rust_type.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_to_c_primitives() {
        assert_eq!(map_rust_to_c("u32"), "uint32_t");
        assert_eq!(map_rust_to_c("f32"), "float");
        assert_eq!(map_rust_to_c("()"), "void");
    }

    #[test]
    fn test_rust_to_c_pointers() {
        assert_eq!(map_rust_to_c("*const u8"), "const uint8_t*");
        assert_eq!(map_rust_to_c("*mut u8"), "uint8_t*");
        assert_eq!(map_rust_to_c("*const f32"), "const float*");
    }

    #[test]
    fn test_rust_to_zig_primitives() {
        assert_eq!(map_rust_to_zig("u32"), "u32");
        assert_eq!(map_rust_to_zig("f32"), "f32");
        assert_eq!(map_rust_to_zig("()"), "void");
    }

    #[test]
    fn test_rust_to_zig_pointers() {
        assert_eq!(map_rust_to_zig("*const u8"), "[*]const u8");
        assert_eq!(map_rust_to_zig("*mut u8"), "[*]u8");
        assert_eq!(map_rust_to_zig("*const f32"), "[*]const f32");
    }
}
