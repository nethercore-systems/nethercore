//! FFI binding generator library
//!
//! This library generates C and Zig bindings from Rust FFI declarations.

pub mod generators;
pub mod model;
pub mod parser;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Generate all FFI bindings
pub fn generate_all() -> Result<()> {
    // For now, hardcoded to ZX. TODO: Make this configurable via CLI
    generate_for_console("zx")
}

/// Generate bindings for a specific console
pub fn generate_for_console(console: &str) -> Result<()> {
    let workspace_root = find_workspace_root()?;

    // Construct console-specific paths
    let ffi_source = workspace_root.join(format!("include/emberware_{}_ffi.rs", console));
    let c_output = workspace_root.join(format!("include/emberware_{}.h", console));
    let zig_output = workspace_root.join(format!("include/emberware_{}.zig", console));

    // Parse FFI source
    let model = parser::parse_ffi_file(&ffi_source)
        .with_context(|| format!("Failed to parse FFI source for console '{}'", console))?;

    println!("Console: {}", console.to_uppercase());
    println!("Parsed {} functions", model.functions.len());
    println!("Parsed {} constant modules", model.constants.len());

    // Generate C header
    let c_header = generators::c::generate_c_header(&model, console)
        .context("Failed to generate C header")?;

    std::fs::write(&c_output, c_header)
        .with_context(|| format!("Failed to write C header to {}", c_output.display()))?;

    println!("Generated C header: {}", c_output.display());

    // Generate Zig bindings
    let zig_bindings = generators::zig::generate_zig_bindings(&model, console)
        .context("Failed to generate Zig bindings")?;

    std::fs::write(&zig_output, zig_bindings)
        .with_context(|| format!("Failed to write Zig bindings to {}", zig_output.display()))?;

    println!("Generated Zig bindings: {}", zig_output.display());

    Ok(())
}

/// Validate that bindings are in sync
pub fn validate() -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let ffi_source = workspace_root.join("include/emberware_zx_ffi.rs");

    // Parse FFI source
    let model = parser::parse_ffi_file(&ffi_source)
        .context("Failed to parse FFI source file")?;

    println!("✓ Rust FFI: {} functions, {} constant modules",
        model.functions.len(),
        model.constants.len()
    );

    // TODO: Validate C and Zig bindings in Phase 2b

    Ok(())
}

/// Find the workspace root by looking for Cargo.toml with [workspace]
fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this is the emberware directory
            if current.join("include").exists() {
                return Ok(current);
            }
        }

        if !current.pop() {
            anyhow::bail!("Could not find workspace root (emberware directory)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_ffi() {
        let source = r#"
            #[link(wasm_import_module = "env")]
            extern "C" {
                /// Returns delta time
                pub fn delta_time() -> f32;

                /// Logs a message
                pub fn log(ptr: *const u8, len: u32);
            }

            pub mod button {
                pub const UP: u32 = 0;
                pub const DOWN: u32 = 1;
            }
        "#;

        let model = parser::parse_ffi_source(source).unwrap();
        assert_eq!(model.functions.len(), 2);
        assert_eq!(model.constants.len(), 1);
    }
}
