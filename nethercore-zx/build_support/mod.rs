//! Build script support modules.
//!
//! `build.rs` stays intentionally small; the implementation lives in this folder.

pub(crate) mod common_extract;
pub(crate) mod ffi;
pub(crate) mod formats;
pub(crate) mod generated_code;
pub(crate) mod generator;
pub(crate) mod snippets;
pub(crate) mod sources;

pub(crate) fn run() {
    ffi::check_ffi_freshness();
    sources::emit_rerun_if_changed();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated_shaders.rs");

    let generated = match generated_code::generate() {
        Ok(v) => v,
        Err(errors) => {
            panic!(
                "Shader generation failed with {} errors:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }
    };

    // Expected: 24 (mode 0) + 16*3 (modes 1-3) = 72 shaders
    // Mode 0: formats 0-15 + 20-23 + 28-31 (tangent requires normal)
    // Modes 1-3: formats 4-7, 12-15, 20-23, 28-31 (all require normal)
    assert_eq!(
        generated.shader_count, 72,
        "Expected 72 shaders, got {}",
        generated.shader_count
    );

    std::fs::write(&dest_path, generated.rust).expect("Failed to write generated_shaders.rs");

    println!(
        "cargo:warning=Generated {} shaders successfully (+ environment + quad)",
        generated.shader_count
    );
}
