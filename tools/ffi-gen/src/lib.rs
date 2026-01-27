//! FFI binding generator library
//!
//! This library generates C and Zig bindings from Rust FFI declarations.

pub mod generators;
pub mod model;
pub mod parser;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generate all FFI bindings
pub fn generate_all() -> Result<()> {
    generate_for_console("zx")
}

/// Get list of supported consoles (those with FFI files in include/)
pub fn get_consoles() -> Result<Vec<String>> {
    let workspace_root = find_workspace_root()?;
    let include_dir = workspace_root.join("include");

    let mut consoles = Vec::new();
    for entry in std::fs::read_dir(&include_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Directory module pattern: include/{console}/ (with mod.rs inside)
        if entry.file_type()?.is_dir() {
            let mod_rs = entry.path().join("mod.rs");
            if mod_rs.exists() {
                consoles.push(name);
                continue;
            }
        }

        // Legacy single-file pattern: {console}.rs
        if name.ends_with(".rs") && !["mod.rs", "lib.rs"].contains(&name.as_str()) {
            if let Some(console) = name.strip_suffix(".rs") {
                consoles.push(console.to_string());
            }
        }
    }

    consoles.dedup();
    Ok(consoles)
}

/// Resolve the FFI source path for a console (directory or single file).
fn resolve_ffi_source(workspace_root: &Path, console: &str) -> PathBuf {
    let dir_path = workspace_root.join(format!("include/{}", console));
    if dir_path.is_dir() {
        dir_path
    } else {
        workspace_root.join(format!("include/{}.rs", console))
    }
}

/// Generate bindings for a specific console
pub fn generate_for_console(console: &str) -> Result<()> {
    let workspace_root = find_workspace_root()?;

    let ffi_source = resolve_ffi_source(&workspace_root, console);
    let c_output = workspace_root.join(format!("include/{}.h", console));
    let zig_output = workspace_root.join(format!("include/{}.zig", console));

    // Parse FFI source
    let model = parser::parse_ffi_file(&ffi_source)
        .with_context(|| format!("Failed to parse FFI source for console '{}'", console))?;

    println!("Console: {}", console.to_uppercase());
    println!("Parsed {} functions", model.functions.len());
    println!("Parsed {} constant modules", model.constants.len());

    // Generate C header
    let c_header =
        generators::c::generate_c_header(&model, console).context("Failed to generate C header")?;

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
    validate_for_console("zx")
}

/// Validate bindings for a specific console
pub fn validate_for_console(console: &str) -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let ffi_source = resolve_ffi_source(&workspace_root, console);

    // Parse FFI source
    let model = parser::parse_ffi_file(&ffi_source).context("Failed to parse FFI source")?;

    println!(
        "✓ Rust FFI: {} functions, {} constant modules",
        model.functions.len(),
        model.constants.len()
    );

    Ok(())
}

/// Check if generated bindings are in sync with the source
pub fn check_for_console(console: &str) -> Result<bool> {
    let workspace_root = find_workspace_root()?;

    let c_output = workspace_root.join(format!("include/{}.h", console));
    let zig_output = workspace_root.join(format!("include/{}.zig", console));
    let ffi_source = resolve_ffi_source(&workspace_root, console);

    // Parse FFI source
    let model = parser::parse_ffi_file(&ffi_source)
        .with_context(|| format!("Failed to parse FFI source for console '{}'", console))?;

    // Generate fresh bindings
    let fresh_c = generators::c::generate_c_header(&model, console)?;
    let fresh_zig = generators::zig::generate_zig_bindings(&model, console)?;

    // Read existing bindings
    let existing_c = std::fs::read_to_string(&c_output)
        .with_context(|| format!("Failed to read {}", c_output.display()))?;
    let existing_zig = std::fs::read_to_string(&zig_output)
        .with_context(|| format!("Failed to read {}", zig_output.display()))?;

    let mut in_sync = true;

    if fresh_c != existing_c {
        println!("✗ C header out of sync: {}", c_output.display());
        in_sync = false;
    } else {
        println!("✓ C header in sync: {}", c_output.display());
    }

    if fresh_zig != existing_zig {
        println!("✗ Zig bindings out of sync: {}", zig_output.display());
        in_sync = false;
    } else {
        println!("✓ Zig bindings in sync: {}", zig_output.display());
    }

    Ok(in_sync)
}

/// Verify that generated bindings compile with Zig
pub fn verify_with_zig(console: &str) -> Result<()> {
    let workspace_root = find_workspace_root()?;

    let c_header = workspace_root.join(format!("include/{}.h", console));
    let zig_bindings = workspace_root.join(format!("include/{}.zig", console));

    println!(
        "Verifying {} bindings with Zig compiler...",
        console.to_uppercase()
    );

    // Verify C header compiles with zig cc
    verify_c_header(&c_header, console)?;

    // Verify Zig bindings compile
    verify_zig_bindings(&zig_bindings)?;

    println!("✓ All bindings verified successfully!");
    Ok(())
}

/// Verify C header compiles with zig cc
fn verify_c_header(header_path: &Path, console: &str) -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let temp_dir = std::env::temp_dir();
    let test_c = temp_dir.join(format!("{}_test.c", console));
    let output_obj = temp_dir.join(format!("{}_test.o", console));

    // Create a minimal C file that includes the header
    let test_code = format!(
        r#"#include "{}"

// Test that types are accessible
void test_types(void) {{
    uint32_t test_u32 = 0;
    float test_f32 = 0.0f;
    (void)test_u32;
    (void)test_f32;
}}

// Test that helper macros work
void test_helpers(void) {{
    uint32_t color = nc{}_rgba(255, 128, 64, 255);
    (void)color;
}}
"#,
        header_path.display(),
        console
    );

    std::fs::write(&test_c, test_code)?;

    // Compile with zig cc targeting WASM32 (proper target for these bindings)
    // Use -Wno-unknown-attributes to suppress WASM-specific attribute warnings when not on WASM
    // Use -Wno-incompatible-library-redeclaration for log() function conflict
    let output = Command::new("zig")
        .args([
            "cc",
            "-c", // Compile only, don't link
            "-target",
            "wasm32-freestanding",     // Target WASM (these are WASM bindings)
            "-Wno-unknown-attributes", // Suppress import_module warnings
            "-Wno-incompatible-library-redeclaration", // log() conflicts with math.h
            "-I",
            workspace_root.join("include").to_str().unwrap(),
            "-o",
            output_obj.to_str().unwrap(),
            test_c.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run zig cc - is Zig installed?")?;

    // Clean up temp files
    let _ = std::fs::remove_file(&test_c);
    let _ = std::fs::remove_file(&output_obj);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("C header failed to compile:\n{}", stderr);
    }

    println!("  ✓ C header compiles: {}", header_path.display());
    Ok(())
}

/// Verify Zig bindings compile
fn verify_zig_bindings(zig_path: &Path) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let test_zig = temp_dir.join("test_bindings.zig");
    let output_obj = temp_dir.join("test_bindings.o");

    // Create a test file that imports the bindings
    let test_code = format!(
        r#"const ew = @import("{}");

// Test that extern functions are accessible
fn testExterns() void {{
    _ = ew.delta_time;
    _ = ew.log;
}}

// Test that helper functions work
fn testHelpers() void {{
    const color = ew.rgba(255, 128, 64, 255);
    _ = color;
}}

pub fn main() void {{}}
"#,
        zig_path.display().to_string().replace("\\", "/")
    );

    std::fs::write(&test_zig, test_code)?;

    // Use zig build-obj targeting WASM freestanding
    let output = Command::new("zig")
        .args([
            "build-obj",
            "-target",
            "wasm32-freestanding",
            test_zig.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run zig build-obj - is Zig installed?")?;

    // Clean up temp files
    let _ = std::fs::remove_file(&test_zig);
    let _ = std::fs::remove_file(&output_obj);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Zig bindings failed to compile:\n{}", stderr);
    }

    println!("  ✓ Zig bindings compile: {}", zig_path.display());
    Ok(())
}

/// Find the workspace root by looking for Cargo.toml with [workspace]
fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this is the nethercore directory
            if current.join("include").exists() {
                return Ok(current);
            }
        }

        if !current.pop() {
            anyhow::bail!("Could not find workspace root (nethercore directory)");
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
