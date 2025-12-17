//! FFI helper functions for memory reading and validation
//!
//! These helpers reduce code duplication across FFI modules by providing
//! standardized patterns for WASM memory access and parameter validation.

use tracing::warn;
use wasmtime::{Caller, Memory};

use super::ZXGameContext;

// ============================================================================
// Memory Helpers
// ============================================================================

/// Get WASM memory from caller, returning None with warning on failure.
///
/// Use this when you need the raw Memory handle for complex access patterns.
/// For simple byte/float reading, prefer `read_wasm_bytes` or `read_wasm_floats`.
#[inline]
pub(crate) fn get_memory(caller: &Caller<'_, ZXGameContext>, fn_name: &str) -> Option<Memory> {
    match caller.data().game.memory {
        Some(m) => Some(m),
        None => {
            warn!("{}: no WASM memory available", fn_name);
            None
        }
    }
}

/// Read raw bytes from WASM memory with bounds checking.
///
/// Returns `Some(Vec<u8>)` if successful, `None` with warning on error.
///
/// # Arguments
/// * `caller` - The WASM caller context
/// * `ptr` - Pointer to data in WASM memory
/// * `size` - Number of bytes to read
/// * `fn_name` - Function name for error messages
pub(crate) fn read_wasm_bytes(
    caller: &Caller<'_, ZXGameContext>,
    ptr: u32,
    size: usize,
    fn_name: &str,
) -> Option<Vec<u8>> {
    let memory = get_memory(caller, fn_name)?;
    let mem_data = memory.data(caller);
    let ptr = ptr as usize;

    if ptr + size > mem_data.len() {
        warn!(
            "{}: memory access ({} bytes at {}) exceeds bounds ({})",
            fn_name,
            size,
            ptr,
            mem_data.len()
        );
        return None;
    }

    Some(mem_data[ptr..ptr + size].to_vec())
}

/// Read floats from WASM memory with bounds checking.
///
/// Returns `Some(Vec<f32>)` if successful, `None` with warning on error.
///
/// # Arguments
/// * `caller` - The WASM caller context
/// * `ptr` - Pointer to data in WASM memory
/// * `float_count` - Number of f32 values to read
/// * `fn_name` - Function name for error messages
pub(crate) fn read_wasm_floats(
    caller: &Caller<'_, ZXGameContext>,
    ptr: u32,
    float_count: usize,
    fn_name: &str,
) -> Option<Vec<f32>> {
    let byte_size = float_count * std::mem::size_of::<f32>();
    let bytes = read_wasm_bytes(caller, ptr, byte_size, fn_name)?;
    let floats: &[f32] = bytemuck::cast_slice(&bytes);
    Some(floats.to_vec())
}

/// Read a 4x4 matrix (16 floats) from WASM memory.
///
/// Returns `Some([f32; 16])` in column-major order if successful.
pub(crate) fn read_wasm_matrix4x4(
    caller: &Caller<'_, ZXGameContext>,
    ptr: u32,
    fn_name: &str,
) -> Option<[f32; 16]> {
    let floats = read_wasm_floats(caller, ptr, 16, fn_name)?;
    floats.try_into().ok()
}

/// Read u16 indices from WASM memory with bounds checking.
///
/// Returns `Some(Vec<u16>)` if successful, `None` with warning on error.
pub(crate) fn read_wasm_u16s(
    caller: &Caller<'_, ZXGameContext>,
    ptr: u32,
    count: usize,
    fn_name: &str,
) -> Option<Vec<u16>> {
    let byte_size = count * std::mem::size_of::<u16>();
    let bytes = read_wasm_bytes(caller, ptr, byte_size, fn_name)?;
    let values: &[u16] = bytemuck::cast_slice(&bytes);
    Some(values.to_vec())
}

/// Read i16 samples from WASM memory with bounds checking.
///
/// Returns `Some(Vec<i16>)` if successful, `None` with warning on error.
/// Used for audio PCM data.
pub(crate) fn read_wasm_i16s(
    caller: &Caller<'_, ZXGameContext>,
    ptr: u32,
    count: usize,
    fn_name: &str,
) -> Option<Vec<i16>> {
    let byte_size = count * std::mem::size_of::<i16>();
    let bytes = read_wasm_bytes(caller, ptr, byte_size, fn_name)?;
    let values: &[i16] = bytemuck::cast_slice(&bytes);
    Some(values.to_vec())
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Maximum vertex format value (all flags: UV | COLOR | NORMAL | SKINNED)
pub(crate) const MAX_VERTEX_FORMAT: u8 = 15;

/// Validate vertex format is within valid range (0-15).
///
/// Returns `Some(format as u8)` if valid, `None` with warning if invalid.
#[inline]
pub(crate) fn validate_vertex_format(format: u32, fn_name: &str) -> Option<u8> {
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!(
            "{}: invalid format {} (max {})",
            fn_name, format, MAX_VERTEX_FORMAT
        );
        return None;
    }
    Some(format as u8)
}

/// Validate a mode/enum value is within range [0, max].
///
/// Returns `Some(mode as u8)` if valid, `None` with warning if invalid.
#[inline]
pub(crate) fn validate_mode_max(mode: u32, max: u32, fn_name: &str, mode_name: &str) -> Option<u8> {
    if mode > max {
        warn!(
            "{}: invalid {} {} (must be 0-{})",
            fn_name, mode_name, mode, max
        );
        return None;
    }
    Some(mode as u8)
}

/// Validate a count is non-zero.
///
/// Returns `true` if valid (count > 0), `false` with warning if zero.
#[inline]
pub(crate) fn validate_count_nonzero(count: u32, fn_name: &str, count_name: &str) -> bool {
    if count == 0 {
        warn!("{}: {} cannot be 0", fn_name, count_name);
        return false;
    }
    true
}

/// Validate a count doesn't exceed a maximum.
///
/// Returns `true` if valid (count <= max), `false` with warning if exceeded.
#[inline]
pub(crate) fn validate_count_max(count: u32, max: u32, fn_name: &str, count_name: &str) -> bool {
    if count > max {
        warn!(
            "{}: {} {} exceeds maximum {}",
            fn_name, count_name, count, max
        );
        return false;
    }
    true
}

/// Validate dimensions are non-zero.
///
/// Returns `true` if valid (both > 0), `false` with warning if either is zero.
#[inline]
pub(crate) fn validate_dimensions_nonzero(width: u32, height: u32, fn_name: &str) -> bool {
    if width == 0 || height == 0 {
        warn!("{}: invalid dimensions {}x{}", fn_name, width, height);
        return false;
    }
    true
}

/// Calculate size with overflow checking.
///
/// Returns `Some(size)` if multiplication succeeds, `None` with warning on overflow.
#[inline]
pub(crate) fn checked_mul(a: u32, b: u32, fn_name: &str, context: &str) -> Option<u32> {
    a.checked_mul(b).or_else(|| {
        warn!("{}: {} overflow ({}×{})", fn_name, context, a, b);
        None
    })
}
