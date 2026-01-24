//! Render state FFI functions
//!
//! Functions for setting render state like color, culling, filtering, and render passes.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use crate::graphics::{CullMode, PassConfig, TextureFilter};

/// Register render state FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "set_color", set_color)?;
    linker.func_wrap("env", "cull_mode", cull_mode)?;
    linker.func_wrap("env", "texture_filter", texture_filter)?;
    linker.func_wrap("env", "uniform_alpha", uniform_alpha)?;
    linker.func_wrap("env", "dither_offset", dither_offset)?;
    linker.func_wrap("env", "z_index", z_index)?;
    // Render pass functions for execution barriers and depth/stencil control
    linker.func_wrap("env", "begin_pass", begin_pass)?;
    linker.func_wrap("env", "begin_pass_stencil_write", begin_pass_stencil_write)?;
    linker.func_wrap("env", "begin_pass_stencil_test", begin_pass_stencil_test)?;
    linker.func_wrap("env", "begin_pass_full", begin_pass_full)?;
    Ok(())
}

/// Set the uniform tint color
///
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format
///
/// This color is multiplied with vertex colors and textures.
fn set_color(mut caller: Caller<'_, ZXGameContext>, color: u32) {
    let state = &mut caller.data_mut().ffi;
    state.update_color(color);
}

/// Set the face culling mode
///
/// # Arguments
/// * `mode` — 0=none (default, draw both sides), 1=back, 2=front
///
/// Default is none (no culling). Use back-face culling for solid 3D objects for performance.
fn cull_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    if mode > 2 {
        warn!("cull_mode({}) invalid - must be 0-2, using 0 (none)", mode);
        state.cull_mode = CullMode::None;
        return;
    }

    state.cull_mode = CullMode::from_u32(mode);
}

/// Set the texture filtering mode
///
/// # Arguments
/// * `filter` — 0=nearest (pixelated, retro), 1=linear (smooth)
///
/// Default: nearest for retro aesthetic.
/// Note: Filter mode is stored in PackedUnifiedShadingState.flags for per-draw shader selection.
fn texture_filter(mut caller: Caller<'_, ZXGameContext>, filter: u32) {
    let state = &mut caller.data_mut().ffi;

    if filter > 1 {
        warn!(
            "texture_filter({}) invalid - must be 0-1, using 0 (nearest)",
            filter
        );
        state.texture_filter = TextureFilter::Nearest;
        state.update_texture_filter(false);
        return;
    }

    state.texture_filter = TextureFilter::from_u32(filter);
    state.update_texture_filter(filter == 1);
}

/// Set uniform alpha level for dither transparency
///
/// # Arguments
/// * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15)
///
/// This controls the dither pattern threshold for screen-door transparency.
/// The dither pattern is always active, but with level=15 (default) all fragments pass.
fn uniform_alpha(mut caller: Caller<'_, ZXGameContext>, level: u32) {
    let state = &mut caller.data_mut().ffi;

    if level > 15 {
        warn!(
            "uniform_alpha({}) invalid - must be 0-15, clamping to 15",
            level
        );
    }

    state.update_uniform_alpha(level.min(15) as u8);
}

/// Set dither offset for dither transparency
///
/// # Arguments
/// * `x` — 0-3 pixel shift in X axis
/// * `y` — 0-3 pixel shift in Y axis
///
/// Use different offsets for stacked dithered meshes to prevent pattern cancellation.
/// When two transparent objects overlap with the same alpha level and offset, their
/// dither patterns align and pixels cancel out. Different offsets shift the pattern
/// so both objects remain visible.
fn dither_offset(mut caller: Caller<'_, ZXGameContext>, x: u32, y: u32) {
    let state = &mut caller.data_mut().ffi;

    if x > 3 || y > 3 {
        warn!(
            "dither_offset({}, {}) invalid - values must be 0-3, clamping",
            x, y
        );
    }

    state.update_dither_offset(x.min(3) as u8, y.min(3) as u8);
}

/// Set the z-index for 2D ordering control within a pass
///
/// # Arguments
/// * `n` — Z-index value (0 = back, higher values = front)
///
/// Higher z-index values are drawn on top of lower values.
/// Use this to ensure UI elements appear over game content
/// regardless of texture bindings or draw order.
///
/// Note: z_index only affects ordering within the same pass_id.
/// Default: 0 (resets each frame)
fn z_index(mut caller: Caller<'_, ZXGameContext>, n: u32) {
    let state = &mut caller.data_mut().ffi;
    state.current_z_index = n;
}

// ============================================================================
// Render Pass Functions
// ============================================================================

/// Begin a new render pass with optional depth clear.
///
/// Provides an execution barrier - commands in this pass complete before
/// the next pass begins. Use for layered rendering like FPS viewmodels.
///
/// # Arguments
/// * `clear_depth` — Non-zero to clear depth buffer at pass start
///
/// # Example (FPS viewmodel rendering)
/// ```rust,ignore
/// // Draw world first (pass 0)
/// epu_draw(env_config_ptr);
/// draw_mesh(world_mesh);
///
/// // Draw gun on top (pass 1 with depth clear)
/// begin_pass(1);  // Clear depth so gun renders on top
/// draw_mesh(gun_mesh);
/// ```
fn begin_pass(mut caller: Caller<'_, ZXGameContext>, clear_depth: u32) {
    let state = &mut caller.data_mut().ffi;
    state.current_pass_id += 1;
    state
        .pass_configs
        .push(PassConfig::standard(clear_depth != 0));
}

/// Begin a stencil write pass (mask creation mode).
///
/// After calling this, subsequent draw calls write to the stencil buffer
/// but NOT to the color buffer. Use this to create a mask shape.
/// Depth testing is disabled to prevent mask geometry from polluting depth.
///
/// # Arguments
/// * `ref_value` — Stencil reference value to write (typically 1)
/// * `clear_depth` — Non-zero to clear depth buffer at pass start
///
/// # Example (scope mask)
/// ```rust,ignore
/// begin_pass_stencil_write(1, 0);  // Start mask creation
/// draw_mesh(circle_mesh);          // Draw circle to stencil only
/// begin_pass_stencil_test(1, 0);   // Enable testing
/// epu_draw(env_config_ptr);         // Only visible inside circle
/// begin_pass(0);                    // Back to normal rendering
/// ```
fn begin_pass_stencil_write(
    mut caller: Caller<'_, ZXGameContext>,
    ref_value: u32,
    clear_depth: u32,
) {
    let state = &mut caller.data_mut().ffi;
    state.current_pass_id += 1;
    state
        .pass_configs
        .push(PassConfig::stencil_write(ref_value as u8, clear_depth != 0));
}

/// Begin a stencil test pass (render inside mask).
///
/// After calling this, subsequent draw calls only render where
/// the stencil buffer equals ref_value (inside the mask).
///
/// # Arguments
/// * `ref_value` — Stencil reference value to test against (must match write pass)
/// * `clear_depth` — Non-zero to clear depth buffer at pass start
fn begin_pass_stencil_test(
    mut caller: Caller<'_, ZXGameContext>,
    ref_value: u32,
    clear_depth: u32,
) {
    let state = &mut caller.data_mut().ffi;
    state.current_pass_id += 1;
    state
        .pass_configs
        .push(PassConfig::stencil_test(ref_value as u8, clear_depth != 0));
}

/// Begin a render pass with full control over depth and stencil state.
///
/// This is the "escape hatch" for advanced effects not covered by the
/// convenience functions. Most games should use begin_pass, begin_pass_stencil_write,
/// or begin_pass_stencil_test instead.
///
/// # Arguments
/// * `depth_compare` — Depth comparison function (see COMPARE_* constants)
/// * `depth_write` — Non-zero to write to depth buffer
/// * `clear_depth` — Non-zero to clear depth buffer at pass start
/// * `stencil_compare` — Stencil comparison function (see COMPARE_* constants)
/// * `stencil_ref` — Stencil reference value (0-255)
/// * `stencil_pass_op` — Operation when stencil test passes (see STENCIL_OP_* constants)
/// * `stencil_fail_op` — Operation when stencil test fails
/// * `stencil_depth_fail_op` — Operation when depth test fails
///
/// # Comparison function constants (COMPARE_*)
/// * 1 = Never, 2 = Less, 3 = Equal, 4 = LessEqual
/// * 5 = Greater, 6 = NotEqual, 7 = GreaterEqual, 8 = Always
///
/// # Stencil operation constants (STENCIL_OP_*)
/// * 0 = Keep, 1 = Zero, 2 = Replace, 3 = IncrementClamp
/// * 4 = DecrementClamp, 5 = Invert, 6 = IncrementWrap, 7 = DecrementWrap
fn begin_pass_full(
    mut caller: Caller<'_, ZXGameContext>,
    depth_compare: u32,
    depth_write: u32,
    clear_depth: u32,
    stencil_compare: u32,
    stencil_ref: u32,
    stencil_pass_op: u32,
    stencil_fail_op: u32,
    stencil_depth_fail_op: u32,
) {
    let state = &mut caller.data_mut().ffi;
    state.current_pass_id += 1;

    let config = PassConfig {
        depth_compare: compare_from_u32(depth_compare),
        depth_write: depth_write != 0,
        depth_clear: clear_depth != 0,
        stencil_compare: compare_from_u32(stencil_compare),
        stencil_ref: stencil_ref as u8,
        stencil_pass: stencil_op_from_u32(stencil_pass_op),
        stencil_fail: stencil_op_from_u32(stencil_fail_op),
        stencil_depth_fail: stencil_op_from_u32(stencil_depth_fail_op),
    };
    state.pass_configs.push(config);
}

/// Convert FFI compare function constant to wgpu
fn compare_from_u32(value: u32) -> wgpu::CompareFunction {
    match value {
        1 => wgpu::CompareFunction::Never,
        2 => wgpu::CompareFunction::Less,
        3 => wgpu::CompareFunction::Equal,
        4 => wgpu::CompareFunction::LessEqual,
        5 => wgpu::CompareFunction::Greater,
        6 => wgpu::CompareFunction::NotEqual,
        7 => wgpu::CompareFunction::GreaterEqual,
        8 => wgpu::CompareFunction::Always,
        _ => {
            warn!("Invalid compare function {}, using Less", value);
            wgpu::CompareFunction::Less
        }
    }
}

/// Convert FFI stencil operation constant to wgpu
fn stencil_op_from_u32(value: u32) -> wgpu::StencilOperation {
    match value {
        0 => wgpu::StencilOperation::Keep,
        1 => wgpu::StencilOperation::Zero,
        2 => wgpu::StencilOperation::Replace,
        3 => wgpu::StencilOperation::IncrementClamp,
        4 => wgpu::StencilOperation::DecrementClamp,
        5 => wgpu::StencilOperation::Invert,
        6 => wgpu::StencilOperation::IncrementWrap,
        7 => wgpu::StencilOperation::DecrementWrap,
        _ => {
            warn!("Invalid stencil operation {}, using Keep", value);
            wgpu::StencilOperation::Keep
        }
    }
}
