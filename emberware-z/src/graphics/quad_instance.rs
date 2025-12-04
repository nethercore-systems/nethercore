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
/// Total size: 60 bytes (16-byte aligned for GPU compatibility)
/// Used with a static unit quad mesh for instanced rendering.
///
/// IMPORTANT: WGSL vec3<f32> is 16-byte aligned, so we need padding after position!
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct QuadInstance {
    /// Position in world-space (billboards/world quads) or screen-space (sprites)
    pub position: [f32; 3], // 12 bytes
    pub _padding1: f32, // 4 bytes padding (WGSL vec3 is 16-byte aligned!)

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

// Safety: QuadInstance is repr(C) with only primitive types and explicit padding
unsafe impl bytemuck::Pod for QuadInstance {}
unsafe impl bytemuck::Zeroable for QuadInstance {}

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
            _padding1: 0.0,
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
            _padding1: 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_quad_instance_layout() {
        // WGSL struct QuadInstance has specific alignment requirements:
        // - vec3<f32>: 16-byte aligned (12 bytes data + 4 bytes padding)
        // - vec2<f32>: 8-byte aligned
        // - f32/u32: 4-byte aligned
        // - vec4<f32>: 16-byte aligned
        //
        // Expected layout:
        // offset  0: position vec3<f32> (12 bytes + 4 padding) = 16 bytes
        // offset 16: size vec2<f32> (8 bytes) = 8 bytes
        // offset 24: rotation f32 (4 bytes) = 4 bytes
        // offset 28: mode u32 (4 bytes) = 4 bytes
        // offset 32: uv vec4<f32> (16 bytes) = 16 bytes
        // offset 48: color u32 (4 bytes) = 4 bytes
        // offset 52: shading_state_index u32 (4 bytes) = 4 bytes
        // offset 56: view_index u32 (4 bytes) = 4 bytes
        // Total: 60 bytes

        assert_eq!(
            mem::size_of::<QuadInstance>(),
            60,
            "QuadInstance size must be 60 bytes to match WGSL struct with vec3 padding"
        );

        // Verify field offsets match WGSL alignment
        let instance = QuadInstance::new(
            [0.0, 0.0, 0.0],
            [1.0, 1.0],
            QuadMode::BillboardSpherical,
            [0.0, 0.0, 1.0, 1.0],
            0xFFFFFFFF,
            0,
            0,
        );

        let base_ptr = &instance as *const _ as usize;

        // Check field offsets
        assert_eq!(
            &instance.position as *const _ as usize - base_ptr,
            0,
            "position must be at offset 0"
        );

        assert_eq!(
            &instance._padding1 as *const _ as usize - base_ptr,
            12,
            "_padding1 must be at offset 12"
        );

        assert_eq!(
            &instance.size as *const _ as usize - base_ptr,
            16,
            "size must be at offset 16 (after vec3 padding)"
        );

        assert_eq!(
            &instance.rotation as *const _ as usize - base_ptr,
            24,
            "rotation must be at offset 24"
        );

        assert_eq!(
            &instance.mode as *const _ as usize - base_ptr,
            28,
            "mode must be at offset 28"
        );

        assert_eq!(
            &instance.uv as *const _ as usize - base_ptr,
            32,
            "uv must be at offset 32"
        );

        assert_eq!(
            &instance.color as *const _ as usize - base_ptr,
            48,
            "color must be at offset 48"
        );

        assert_eq!(
            &instance.shading_state_index as *const _ as usize - base_ptr,
            52,
            "shading_state_index must be at offset 52"
        );

        assert_eq!(
            &instance.view_index as *const _ as usize - base_ptr,
            56,
            "view_index must be at offset 56"
        );
    }

    #[test]
    fn test_quad_instance_is_pod() {
        // Verify QuadInstance can be safely cast to bytes
        let instance = QuadInstance::billboard(
            [1.0, 2.0, 3.0],
            4.0,
            5.0,
            QuadMode::BillboardSpherical,
            [0.0, 0.0, 1.0, 1.0],
            0xAABBCCDD,
            10,
            20,
        );

        // Should not panic
        let _bytes: &[u8] = bytemuck::bytes_of(&instance);

        // Verify we can cast a slice
        let instances = vec![instance; 10];
        let _byte_slice: &[u8] = bytemuck::cast_slice(&instances);
    }
}
