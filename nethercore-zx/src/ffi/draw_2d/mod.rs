//! 2D drawing FFI functions (screen space)
//!
//! Functions for drawing sprites, rectangles, and text in screen space.

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

mod shapes;
mod sprites;
mod text;

#[cfg(test)]
mod tests;

/// Depth value for all screen-space 2D quads
///
/// All 2D UI elements render at depth 0.0 (near plane) for early-z optimization.
/// This allows 3D geometry behind UI elements to be culled via early depth testing,
/// saving fragment shader invocations.
///
/// Layer ordering is handled by CPU-side sorting (render_type=0, sorted by layer field).
/// All layers render in order at the same depth, with later layers overwriting earlier ones
/// in the framebuffer. The `discard` instruction in the quad shader prevents depth writes
/// for transparent dithered pixels, allowing 3D content to show through.
pub(crate) const SCREEN_SPACE_DEPTH: f32 = 0.0;

/// Register 2D drawing FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    sprites::register(linker)?;
    shapes::register(linker)?;
    text::register(linker)?;
    Ok(())
}
