//! Build-time WASM analysis module
//!
//! Static analysis of WASM games to detect configuration at build time.
//! This analyzes the bytecode without executing it, looking for calls to
//! init-only config functions like `render_mode()` and `set_resolution()`.
//!
//! # Purpose
//!
//! Build-time analysis detects configuration calls to determine how assets
//! should be compressed (e.g., BC7 for modes 1-3, RGBA8 for mode 0).
//!
//! # Example
//!
//! ```ignore
//! use emberware_core::analysis::analyze_wasm;
//!
//! let wasm_bytes = std::fs::read("game.wasm")?;
//! let result = analyze_wasm(&wasm_bytes)?;
//!
//! println!("Render mode: {}", result.render_mode);
//! if result.uses_bc7() {
//!     println!("Using BC7 compression");
//! }
//! ```

use hashbrown::HashMap;
use wasmparser::{Operator, Parser, Payload};

/// Result of build-time WASM analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    /// Detected render mode (0-3, defaults to 0 if not set)
    pub render_mode: u8,

    /// Resolution index if set (0-3)
    pub resolution: Option<u32>,

    /// Tick rate index if set (0-3)
    pub tick_rate: Option<u32>,

    /// Clear color if set
    pub clear_color: Option<u32>,

    /// ROM texture IDs requested during init (not detectable via static analysis)
    pub texture_ids: Vec<String>,

    /// ROM mesh IDs requested during init (not detectable via static analysis)
    pub mesh_ids: Vec<String>,
}

impl AnalysisResult {
    /// Check if render mode uses BC7 compression
    pub fn uses_bc7(&self) -> bool {
        self.render_mode >= 1 && self.render_mode <= 3
    }

    /// Get the texture format for a given slot
    ///
    /// Returns the appropriate format based on render mode and slot:
    /// - Mode 0: Always RGBA8
    /// - Mode 1: BC7 for all slots (matcap)
    /// - Mode 2: BC7 sRGB for slots 0,3; BC7 Linear for slot 1 (material)
    /// - Mode 3: BC7 sRGB for slots 0,2,3; BC7 Linear for slot 1 (material)
    pub fn texture_format_for_slot(&self, slot: u8) -> TextureFormatHint {
        match self.render_mode {
            0 => TextureFormatHint::Rgba8,
            1 => TextureFormatHint::Bc7Srgb, // Matcap - all sRGB
            2 | 3 => {
                if slot == 1 {
                    TextureFormatHint::Bc7Linear // Material map
                } else {
                    TextureFormatHint::Bc7Srgb // Albedo, specular, env
                }
            }
            _ => TextureFormatHint::Rgba8, // Invalid mode defaults to RGBA8
        }
    }
}

/// Hint for texture format based on analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormatHint {
    /// Uncompressed RGBA8
    Rgba8,
    /// BC7 compressed, sRGB color space
    Bc7Srgb,
    /// BC7 compressed, linear color space (for material maps)
    Bc7Linear,
}

/// Validation error for analysis results
#[derive(Debug, Clone, thiserror::Error)]
pub enum AnalysisError {
    /// Invalid render mode (must be 0-3)
    #[error("invalid render mode {0} (must be 0-3)")]
    InvalidRenderMode(u8),

    /// Invalid resolution (must be 0-3)
    #[error("invalid resolution {0} (must be 0-3)")]
    InvalidResolution(u32),

    /// Config function called multiple times
    #[error("{0}() called {1} times - each config function can only be called once")]
    DuplicateCall(String, usize),

    /// WASM parsing error
    #[error("WASM parsing failed: {0}")]
    ParseError(String),

    /// init() function not found
    #[error("init() function not exported by WASM module")]
    InitNotFound,
}

/// Config function call detected during analysis
#[derive(Debug, Clone)]
struct ConfigCall {
    /// Function name (e.g., "render_mode")
    name: String,
    /// Constant argument if detectable
    arg: Option<u32>,
}

/// Analyze WASM bytecode to detect configuration
///
/// Performs static analysis of the WASM module to find calls to
/// config functions (render_mode, set_resolution, etc.) and extract
/// their constant arguments.
///
/// # Errors
///
/// Returns an error if:
/// - WASM parsing fails
/// - Any config function is called more than once
/// - render_mode is called with an invalid value (> 3)
pub fn analyze_wasm(wasm_bytes: &[u8]) -> Result<AnalysisResult, AnalysisError> {
    let mut imports: HashMap<u32, String> = HashMap::new();
    let mut config_calls: Vec<ConfigCall> = Vec::new();
    let mut num_imported_funcs = 0u32;

    // First pass: collect imports to map function indices to names
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        let payload = payload.map_err(|e| AnalysisError::ParseError(e.to_string()))?;

        if let Payload::ImportSection(reader) = payload {
            for import in reader {
                let import = import.map_err(|e| AnalysisError::ParseError(e.to_string()))?;

                if let wasmparser::TypeRef::Func(_) = import.ty {
                    // Only track "env" module imports for config functions
                    if import.module == "env" {
                        let name = import.name.to_string();
                        if is_config_function(&name) {
                            imports.insert(num_imported_funcs, name);
                        }
                    }
                    num_imported_funcs += 1;
                }
            }
        }
    }

    // Second pass: find calls to config functions and extract arguments
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        let payload = payload.map_err(|e| AnalysisError::ParseError(e.to_string()))?;

        if let Payload::CodeSectionEntry(body) = payload {
            let mut last_const: Option<u32> = None;
            let ops = body
                .get_operators_reader()
                .map_err(|e| AnalysisError::ParseError(e.to_string()))?;

            for op in ops {
                let op = op.map_err(|e| AnalysisError::ParseError(e.to_string()))?;

                match op {
                    // Track constant values that might be arguments
                    Operator::I32Const { value } => {
                        last_const = Some(value as u32);
                    }
                    // Check for calls to imported config functions
                    Operator::Call { function_index } => {
                        if let Some(name) = imports.get(&function_index) {
                            config_calls.push(ConfigCall {
                                name: name.clone(),
                                arg: last_const,
                            });
                        }
                        last_const = None;
                    }
                    // Any other instruction clears the tracked constant
                    _ => {
                        last_const = None;
                    }
                }
            }
        }
    }

    // Validate: each config function should be called at most once
    let mut call_counts: HashMap<String, usize> = HashMap::new();
    for call in &config_calls {
        *call_counts.entry(call.name.clone()).or_insert(0) += 1;
    }

    for (name, count) in &call_counts {
        if *count > 1 {
            return Err(AnalysisError::DuplicateCall(name.clone(), *count));
        }
    }

    // Build result from detected calls
    let mut result = AnalysisResult::default();

    for call in &config_calls {
        match call.name.as_str() {
            "render_mode" => {
                if let Some(mode) = call.arg {
                    if mode > 3 {
                        return Err(AnalysisError::InvalidRenderMode(mode as u8));
                    }
                    result.render_mode = mode as u8;
                }
            }
            "set_resolution" => {
                if let Some(res) = call.arg
                    && res > 3
                {
                    return Err(AnalysisError::InvalidResolution(res));
                }
                result.resolution = call.arg;
            }
            "set_tick_rate" => {
                result.tick_rate = call.arg;
            }
            "set_clear_color" => {
                result.clear_color = call.arg;
            }
            _ => {}
        }
    }

    Ok(result)
}

/// Check if a function name is a config function we care about
fn is_config_function(name: &str) -> bool {
    matches!(
        name,
        "render_mode" | "set_resolution" | "set_tick_rate" | "set_clear_color"
    )
}

/// Validate analysis result
pub fn validate_result(result: &AnalysisResult) -> Result<(), AnalysisError> {
    if result.render_mode > 3 {
        return Err(AnalysisError::InvalidRenderMode(result.render_mode));
    }
    if let Some(res) = result.resolution
        && res > 3
    {
        return Err(AnalysisError::InvalidResolution(res));
    }
    Ok(())
}

// Keep these types for backwards compatibility / future use
/// Texture request captured during analysis
#[derive(Debug, Clone)]
pub struct TextureRequest {
    /// Allocated handle (for ordering/debugging)
    pub handle: u32,
    /// Texture dimensions (0,0 if loaded from ROM)
    pub width: u32,
    pub height: u32,
    /// Source of the texture
    pub source: TextureSource,
}

/// Source of a texture request
#[derive(Debug, Clone)]
pub enum TextureSource {
    /// Loaded from WASM memory via load_texture()
    WasmMemory { ptr: u32 },
    /// Loaded from ROM data pack via rom_texture()
    RomPack { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_render_mode() {
        let result = AnalysisResult::default();
        assert_eq!(result.render_mode, 0);
    }

    #[test]
    fn test_uses_bc7() {
        assert!(
            !AnalysisResult {
                render_mode: 0,
                ..Default::default()
            }
            .uses_bc7()
        );
        assert!(
            AnalysisResult {
                render_mode: 1,
                ..Default::default()
            }
            .uses_bc7()
        );
        assert!(
            AnalysisResult {
                render_mode: 2,
                ..Default::default()
            }
            .uses_bc7()
        );
        assert!(
            AnalysisResult {
                render_mode: 3,
                ..Default::default()
            }
            .uses_bc7()
        );
    }

    #[test]
    fn test_texture_format_for_slot_mode0() {
        let result = AnalysisResult {
            render_mode: 0,
            ..Default::default()
        };
        assert_eq!(result.texture_format_for_slot(0), TextureFormatHint::Rgba8);
        assert_eq!(result.texture_format_for_slot(1), TextureFormatHint::Rgba8);
    }

    #[test]
    fn test_texture_format_for_slot_mode1() {
        let result = AnalysisResult {
            render_mode: 1,
            ..Default::default()
        };
        assert_eq!(
            result.texture_format_for_slot(0),
            TextureFormatHint::Bc7Srgb
        );
        assert_eq!(
            result.texture_format_for_slot(1),
            TextureFormatHint::Bc7Srgb
        );
        assert_eq!(
            result.texture_format_for_slot(2),
            TextureFormatHint::Bc7Srgb
        );
        assert_eq!(
            result.texture_format_for_slot(3),
            TextureFormatHint::Bc7Srgb
        );
    }

    #[test]
    fn test_texture_format_for_slot_mode2() {
        let result = AnalysisResult {
            render_mode: 2,
            ..Default::default()
        };
        assert_eq!(
            result.texture_format_for_slot(0),
            TextureFormatHint::Bc7Srgb
        ); // Albedo
        assert_eq!(
            result.texture_format_for_slot(1),
            TextureFormatHint::Bc7Linear
        ); // Material
        assert_eq!(
            result.texture_format_for_slot(3),
            TextureFormatHint::Bc7Srgb
        ); // Env
    }

    #[test]
    fn test_texture_format_for_slot_mode3() {
        let result = AnalysisResult {
            render_mode: 3,
            ..Default::default()
        };
        assert_eq!(
            result.texture_format_for_slot(0),
            TextureFormatHint::Bc7Srgb
        ); // Albedo
        assert_eq!(
            result.texture_format_for_slot(1),
            TextureFormatHint::Bc7Linear
        ); // Material
        assert_eq!(
            result.texture_format_for_slot(2),
            TextureFormatHint::Bc7Srgb
        ); // Specular
        assert_eq!(
            result.texture_format_for_slot(3),
            TextureFormatHint::Bc7Srgb
        ); // Env
    }

    #[test]
    fn test_validation_valid_modes() {
        for mode in 0..=3 {
            let result = AnalysisResult {
                render_mode: mode,
                ..Default::default()
            };
            assert!(validate_result(&result).is_ok());
        }
    }

    #[test]
    fn test_validation_invalid_mode() {
        let result = AnalysisResult {
            render_mode: 4,
            ..Default::default()
        };
        assert!(matches!(
            validate_result(&result),
            Err(AnalysisError::InvalidRenderMode(4))
        ));
    }

    #[test]
    fn test_is_config_function() {
        assert!(is_config_function("render_mode"));
        assert!(is_config_function("set_resolution"));
        assert!(is_config_function("set_tick_rate"));
        assert!(is_config_function("set_clear_color"));
        assert!(!is_config_function("draw_triangle"));
        assert!(!is_config_function("load_texture"));
    }

    // WASM analysis tests using wat crate
    #[test]
    fn test_analyze_wasm_no_config_calls() {
        // Minimal WASM with no config calls
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "init"))
                (func (export "update"))
                (func (export "render"))
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm).unwrap();
        assert_eq!(result.render_mode, 0); // Default
        assert_eq!(result.resolution, None);
    }

    #[test]
    fn test_analyze_wasm_render_mode_call() {
        // WASM that calls render_mode(2)
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "render_mode" (func $render_mode (param i32)))
                (func (export "init")
                    i32.const 2
                    call $render_mode
                )
                (func (export "update"))
                (func (export "render"))
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm).unwrap();
        assert_eq!(result.render_mode, 2);
    }

    #[test]
    fn test_analyze_wasm_set_resolution_call() {
        // WASM that calls set_resolution(3)
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "set_resolution" (func $set_resolution (param i32)))
                (func (export "init")
                    i32.const 3
                    call $set_resolution
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm).unwrap();
        assert_eq!(result.resolution, Some(3));
    }

    #[test]
    fn test_analyze_wasm_duplicate_render_mode() {
        // WASM that calls render_mode twice - should fail
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "render_mode" (func $render_mode (param i32)))
                (func (export "init")
                    i32.const 1
                    call $render_mode
                    i32.const 2
                    call $render_mode
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm);
        assert!(matches!(
            result,
            Err(AnalysisError::DuplicateCall(name, 2)) if name == "render_mode"
        ));
    }

    #[test]
    fn test_analyze_wasm_invalid_render_mode() {
        // WASM that calls render_mode(5) - invalid
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "render_mode" (func $render_mode (param i32)))
                (func (export "init")
                    i32.const 5
                    call $render_mode
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm);
        assert!(matches!(result, Err(AnalysisError::InvalidRenderMode(5))));
    }

    #[test]
    fn test_analyze_wasm_multiple_config_calls() {
        // WASM with multiple different config calls (allowed)
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "render_mode" (func $render_mode (param i32)))
                (import "env" "set_resolution" (func $set_resolution (param i32)))
                (import "env" "set_tick_rate" (func $set_tick_rate (param i32)))
                (func (export "init")
                    i32.const 2
                    call $render_mode
                    i32.const 3
                    call $set_resolution
                    i32.const 2
                    call $set_tick_rate
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm).unwrap();
        assert_eq!(result.render_mode, 2);
        assert_eq!(result.resolution, Some(3));
        assert_eq!(result.tick_rate, Some(2));
    }

    #[test]
    fn test_analyze_wasm_duplicate_set_resolution() {
        // WASM that calls set_resolution twice - should fail
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "set_resolution" (func $set_resolution (param i32)))
                (func (export "init")
                    i32.const 1
                    call $set_resolution
                    i32.const 2
                    call $set_resolution
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm);
        assert!(matches!(
            result,
            Err(AnalysisError::DuplicateCall(name, 2)) if name == "set_resolution"
        ));
    }

    #[test]
    fn test_analyze_wasm_invalid_resolution() {
        // WASM that calls set_resolution(5) - invalid
        let wasm = wat::parse_str(
            r#"
            (module
                (import "env" "set_resolution" (func $set_resolution (param i32)))
                (func (export "init")
                    i32.const 5
                    call $set_resolution
                )
            )
        "#,
        )
        .unwrap();

        let result = analyze_wasm(&wasm);
        assert!(matches!(result, Err(AnalysisError::InvalidResolution(5))));
    }

    #[test]
    fn test_validation_invalid_resolution() {
        let result = AnalysisResult {
            resolution: Some(4),
            ..Default::default()
        };
        assert!(matches!(
            validate_result(&result),
            Err(AnalysisError::InvalidResolution(4))
        ));
    }

    #[test]
    fn test_validation_valid_resolutions() {
        for res in 0..=3 {
            let result = AnalysisResult {
                resolution: Some(res),
                ..Default::default()
            };
            assert!(validate_result(&result).is_ok());
        }
    }
}
