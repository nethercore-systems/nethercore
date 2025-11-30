//! Draw command types
//!
//! Provides draw commands and pending resource structures for the rendering pipeline.

use glam::Mat4;

/// Pending texture load request
#[derive(Debug)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Pending mesh load request (retained mode)
#[derive(Debug)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u32>>,
}

/// Draw command for immediate mode drawing
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw triangles with immediate data (non-indexed)
    DrawTriangles {
        format: u8,
        vertex_data: Vec<f32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw indexed triangles with immediate data
    DrawTrianglesIndexed {
        format: u8,
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a retained mesh by handle
    DrawMesh {
        handle: u32,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a billboard (camera-facing quad)
    DrawBillboard {
        /// Billboard width
        width: f32,
        /// Billboard height
        height: f32,
        /// Billboard mode (1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z)
        mode: u8,
        /// Source UV rectangle (x, y, w, h) - None for full texture (0,0,1,1)
        uv_rect: Option<(f32, f32, f32, f32)>,
        /// World position from transform
        transform: Mat4,
        /// Color tint
        color: u32,
        /// Depth test enabled
        depth_test: bool,
        /// Cull mode
        cull_mode: u8,
        /// Blend mode
        blend_mode: u8,
        /// Bound textures
        bound_textures: [u32; 4],
    },
    /// Draw a 2D sprite in screen space
    DrawSprite {
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Sprite width (pixels)
        width: f32,
        /// Sprite height (pixels)
        height: f32,
        /// Source UV rectangle (x, y, w, h) - None for full texture (0,0,1,1)
        uv_rect: Option<(f32, f32, f32, f32)>,
        /// Origin offset for rotation (x, y in pixels, 0,0 = top-left)
        origin: Option<(f32, f32)>,
        /// Rotation angle in degrees (clockwise)
        rotation: f32,
        /// Color tint
        color: u32,
        /// Blend mode
        blend_mode: u8,
        /// Bound textures
        bound_textures: [u32; 4],
    },
    /// Draw a 2D rectangle in screen space
    DrawRect {
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Rectangle width (pixels)
        width: f32,
        /// Rectangle height (pixels)
        height: f32,
        /// Fill color
        color: u32,
        /// Blend mode
        blend_mode: u8,
    },
    /// Draw text in screen space
    DrawText {
        /// UTF-8 text bytes (decoded during rendering)
        text: Vec<u8>,
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Font size (pixels)
        size: f32,
        /// Text color
        color: u32,
        /// Blend mode
        blend_mode: u8,
    },
    /// Set procedural sky parameters
    SetSky {
        /// Horizon color (RGB, linear)
        horizon_color: [f32; 3],
        /// Zenith (top) color (RGB, linear)
        zenith_color: [f32; 3],
        /// Sun direction (will be normalized)
        sun_direction: [f32; 3],
        /// Sun color (RGB, linear)
        sun_color: [f32; 3],
        /// Sun sharpness (higher = sharper sun, typically 32-256)
        sun_sharpness: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    // ============================================================================
    // PendingTexture Tests
    // ============================================================================

    #[test]
    fn test_pending_texture() {
        let texture = PendingTexture {
            handle: 1,
            width: 64,
            height: 64,
            data: vec![0xFF; 64 * 64 * 4],
        };
        assert_eq!(texture.handle, 1);
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
        assert_eq!(texture.data.len(), 64 * 64 * 4);
    }

    // ============================================================================
    // PendingMesh Tests
    // ============================================================================

    #[test]
    fn test_pending_mesh_non_indexed() {
        let mesh = PendingMesh {
            handle: 1,
            format: 0, // POS only
            vertex_data: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            index_data: None,
        };
        assert_eq!(mesh.handle, 1);
        assert_eq!(mesh.format, 0);
        assert_eq!(mesh.vertex_data.len(), 9); // 3 vertices * 3 floats
        assert!(mesh.index_data.is_none());
    }

    #[test]
    fn test_pending_mesh_indexed() {
        let mesh = PendingMesh {
            handle: 2,
            format: 1, // POS_UV
            vertex_data: vec![
                0.0, 0.0, 0.0, 0.0, 0.0, // v0: pos + uv
                1.0, 0.0, 0.0, 1.0, 0.0, // v1: pos + uv
                0.0, 1.0, 0.0, 0.0, 1.0, // v2: pos + uv
            ],
            index_data: Some(vec![0, 1, 2]),
        };
        assert_eq!(mesh.handle, 2);
        assert_eq!(mesh.format, 1);
        assert_eq!(mesh.vertex_data.len(), 15); // 3 vertices * 5 floats
        assert_eq!(mesh.index_data, Some(vec![0, 1, 2]));
    }

    // ============================================================================
    // DrawCommand Tests
    // ============================================================================

    #[test]
    fn test_draw_command_triangles() {
        let cmd = DrawCommand::DrawTriangles {
            format: 2,
            vertex_data: vec![0.0; 24], // 3 verts * 6 floats (pos + color)
            transform: Mat4::IDENTITY,
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 1,
            blend_mode: 0,
            bound_textures: [0; 4],
        };
        match cmd {
            DrawCommand::DrawTriangles { format, .. } => assert_eq!(format, 2),
            _ => panic!("Expected DrawTriangles"),
        }
    }

    #[test]
    fn test_draw_command_mesh() {
        let cmd = DrawCommand::DrawMesh {
            handle: 5,
            transform: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: 0,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawMesh {
                handle, depth_test, ..
            } => {
                assert_eq!(handle, 5);
                assert!(!depth_test);
            }
            _ => panic!("Expected DrawMesh"),
        }
    }

    #[test]
    fn test_draw_command_billboard() {
        let cmd = DrawCommand::DrawBillboard {
            width: 2.0,
            height: 3.0,
            mode: 1, // Spherical
            uv_rect: Some((0.0, 0.0, 0.5, 0.5)),
            transform: Mat4::IDENTITY,
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 0,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawBillboard {
                width,
                height,
                mode,
                uv_rect,
                ..
            } => {
                assert_eq!(width, 2.0);
                assert_eq!(height, 3.0);
                assert_eq!(mode, 1);
                assert_eq!(uv_rect, Some((0.0, 0.0, 0.5, 0.5)));
            }
            _ => panic!("Expected DrawBillboard"),
        }
    }

    #[test]
    fn test_draw_command_sprite() {
        let cmd = DrawCommand::DrawSprite {
            x: 100.0,
            y: 50.0,
            width: 64.0,
            height: 64.0,
            uv_rect: None,
            origin: Some((32.0, 32.0)),
            rotation: 45.0,
            color: 0xFFFFFFFF,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawSprite {
                x,
                y,
                rotation,
                origin,
                ..
            } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 50.0);
                assert_eq!(rotation, 45.0);
                assert_eq!(origin, Some((32.0, 32.0)));
            }
            _ => panic!("Expected DrawSprite"),
        }
    }

    #[test]
    fn test_draw_command_rect() {
        let cmd = DrawCommand::DrawRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            color: 0x00FF00FF,
            blend_mode: 0,
        };
        match cmd {
            DrawCommand::DrawRect {
                width,
                height,
                color,
                ..
            } => {
                assert_eq!(width, 100.0);
                assert_eq!(height, 50.0);
                assert_eq!(color, 0x00FF00FF);
            }
            _ => panic!("Expected DrawRect"),
        }
    }

    #[test]
    fn test_draw_command_text() {
        let cmd = DrawCommand::DrawText {
            text: b"Hello World".to_vec(),
            x: 10.0,
            y: 20.0,
            size: 16.0,
            color: 0xFFFFFFFF,
            blend_mode: 1,
        };
        match cmd {
            DrawCommand::DrawText { text, size, .. } => {
                assert_eq!(std::str::from_utf8(&text).unwrap(), "Hello World");
                assert_eq!(size, 16.0);
            }
            _ => panic!("Expected DrawText"),
        }
    }

    #[test]
    fn test_draw_command_set_sky() {
        let cmd = DrawCommand::SetSky {
            horizon_color: [0.5, 0.7, 1.0],
            zenith_color: [0.1, 0.2, 0.8],
            sun_direction: [0.5, 0.5, 0.5],
            sun_color: [1.0, 0.9, 0.8],
            sun_sharpness: 64.0,
        };
        match cmd {
            DrawCommand::SetSky {
                horizon_color,
                sun_sharpness,
                ..
            } => {
                assert_eq!(horizon_color, [0.5, 0.7, 1.0]);
                assert_eq!(sun_sharpness, 64.0);
            }
            _ => panic!("Expected SetSky"),
        }
    }
}
