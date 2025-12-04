//! GPU-instanced quad rendering
//!
//! Defines quad instance data and modes for GPU-driven billboard/sprite rendering.
//! This replaces the problematic DeferredCommand CPU vertex generation approach.

/// Quad rendering mode for GPU vertex shader expansion
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadMode {
    /// Billboard - spherical (fully camera-facing, all axes)
    BillboardSpherical = 0,
    /// Billboard - cylindrical around world Y axis
    BillboardCylindricalY = 1,
    /// Billboard - cylindrical around world X axis
    BillboardCylindricalX = 2,
    /// Billboard - cylindrical around world Z axis
    BillboardCylindricalZ = 3,
    /// Screen-space sprite (2D overlay in screen coordinates)
    ScreenSpace = 4,
    /// World-space quad (uses model matrix transformation)
    WorldSpace = 5,
}

/// Per-instance quad data uploaded to GPU
///
/// Total size: 56 bytes (16-byte aligned for GPU compatibility)
/// Used with a static unit quad mesh for instanced rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadInstance {
    /// Position in world-space (billboards/world quads) or screen-space (sprites)
    pub position: [f32; 3], // 12 bytes

    /// Quad size (width, height in world units or screen pixels)
    pub size: [f32; 2], // 8 bytes

    /// Rotation angle in radians (used for screen-space sprites)
    pub rotation: f32, // 4 bytes

    /// Quad rendering mode (see QuadMode enum)
    pub mode: u32, // 4 bytes

    /// UV rectangle for texture atlas (u0, v0, u1, v1)
    pub uv: [f32; 4], // 16 bytes

    /// Packed RGBA8 color (0xRRGGBBAA)
    pub color: u32, // 4 bytes

    /// Index into shading_states buffer for material properties
    pub shading_state_index: u32, // 4 bytes

    /// Index into view_matrices buffer for billboard math
    pub view_index: u32, // 4 bytes
}

impl QuadInstance {
    /// Create a new quad instance with default values
    pub fn new(
        position: [f32; 3],
        size: [f32; 2],
        mode: QuadMode,
        uv: [f32; 4],
        color: u32,
        shading_state_index: u32,
        view_index: u32,
    ) -> Self {
        Self {
            position,
            size,
            rotation: 0.0,
            mode: mode as u32,
            uv,
            color,
            shading_state_index,
            view_index,
        }
    }

    /// Create a billboard instance at a world-space position
    pub fn billboard(
        position: [f32; 3],
        width: f32,
        height: f32,
        mode: QuadMode,
        uv: [f32; 4],
        color: u32,
        shading_state_index: u32,
        view_index: u32,
    ) -> Self {
        Self::new(position, [width, height], mode, uv, color, shading_state_index, view_index)
    }

    /// Create a screen-space sprite instance
    pub fn sprite(
        screen_x: f32,
        screen_y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        uv: [f32; 4],
        color: u32,
        shading_state_index: u32,
        view_index: u32,
    ) -> Self {
        Self {
            position: [screen_x, screen_y, 0.0],
            size: [width, height],
            rotation,
            mode: QuadMode::ScreenSpace as u32,
            uv,
            color,
            shading_state_index,
            view_index,
        }
    }
}
