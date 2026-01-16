use super::ShaderGenError;

// Shader templates (embedded for inspection/debugging via get_template()).
const BLINNPHONG_COMMON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/blinnphong_common.wgsl"
));
const TEMPLATE_MODE0: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/mode0_lambert.wgsl"
));
const TEMPLATE_MODE1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/mode1_matcap.wgsl"
));

/// Get the template for a given render mode (for debugging/inspection)
///
/// Note: Modes 2-3 don't have separate templates; they're generated from `blinnphong_common`.
///
/// # Errors
///
/// Returns `ShaderGenError::InvalidRenderMode` if mode is not 0-3.
#[allow(dead_code)] // Debugging/inspection helper
pub fn get_template(mode: u8) -> Result<&'static str, ShaderGenError> {
    match mode {
        0 => Ok(TEMPLATE_MODE0),
        1 => Ok(TEMPLATE_MODE1),
        2 => Ok(BLINNPHONG_COMMON), // Generated from common files
        3 => Ok(BLINNPHONG_COMMON), // Generated from common files
        _ => Err(ShaderGenError::InvalidRenderMode(mode)),
    }
}
