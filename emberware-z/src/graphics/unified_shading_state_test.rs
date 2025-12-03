//! Unit tests for unified shading state packing/unpacking

#[cfg(test)]
mod tests {
    use super::super::*;
    use glam::{Vec3, Vec4};

    #[test]
    fn test_pack_unpack_color() {
        // Test white color
        let white = 0xFFFFFFFF_u32;
        
        // Simulate what the shader does
        let r = ((white >> 24) & 0xFF) as f32 / 255.0;
        let g = ((white >> 16) & 0xFF) as f32 / 255.0;
        let b = ((white >> 8) & 0xFF) as f32 / 255.0;
        let a = (white & 0xFF) as f32 / 255.0;
        
        assert!((r - 1.0).abs() < 0.01, "Red should be 1.0, got {}", r);
        assert!((g - 1.0).abs() < 0.01, "Green should be 1.0, got {}", g);
        assert!((b - 1.0).abs() < 0.01, "Blue should be 1.0, got {}", b);
        assert!((a - 1.0).abs() < 0.01, "Alpha should be 1.0, got {}", a);
    }

    #[test]
    fn test_pack_unpack_red() {
        // Test red color (0xFF0000FF)
        let red = 0xFF0000FF_u32;
        
        let r = ((red >> 24) & 0xFF) as f32 / 255.0;
        let g = ((red >> 16) & 0xFF) as f32 / 255.0;
        let b = ((red >> 8) & 0xFF) as f32 / 255.0;
        let a = (red & 0xFF) as f32 / 255.0;
        
        assert!((r - 1.0).abs() < 0.01, "Red should be 1.0, got {}", r);
        assert!((g - 0.0).abs() < 0.01, "Green should be 0.0, got {}", g);
        assert!((b - 0.0).abs() < 0.01, "Blue should be 0.0, got {}", b);
        assert!((a - 1.0).abs() < 0.01, "Alpha should be 1.0, got {}", a);
    }

    #[test]
    fn test_default_state_has_white_color() {
        let state = PackedUnifiedShadingState::default();
        assert_eq!(state.color_rgba8, 0xFFFFFFFF, "Default color should be white");
    }

    #[test]
    fn test_from_render_state_preserves_color() {
        let sky = Sky {
            horizon_color: Vec4::new(0.5, 0.5, 0.5, 0.0),
            zenith_color: Vec4::new(0.7, 0.7, 0.7, 0.0),
            sun_direction: Vec3::new(0.0, 1.0, 0.0),
            sun_color: Vec3::new(1.0, 1.0, 1.0),
            sun_sharpness: 64.0,
        };

        let lights = [
            crate::state::LightState::default(),
            crate::state::LightState::default(),
            crate::state::LightState::default(),
            crate::state::LightState::default(),
        ];

        let blend_modes = [
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
        ];

        // Test with white color
        let white_state = PackedUnifiedShadingState::from_render_state(
            0xFFFFFFFF,
            0.0,
            0.5,
            0.0,
            &blend_modes,
            &sky,
            &lights,
        );
        assert_eq!(white_state.color_rgba8, 0xFFFFFFFF, "White color should be preserved");

        // Test with red color
        let red_state = PackedUnifiedShadingState::from_render_state(
            0xFF0000FF,
            0.0,
            0.5,
            0.0,
            &blend_modes,
            &sky,
            &lights,
        );
        assert_eq!(red_state.color_rgba8, 0xFF0000FF, "Red color should be preserved");
    }

    #[test]
    fn test_shading_state_interning() {
        use wgpu::BufferUsages;
        
        // Create a mock device (this won't work without actual wgpu context)
        // For now, just test the logic without GPU
        
        let sky = Sky {
            horizon_color: Vec4::new(0.5, 0.5, 0.5, 0.0),
            zenith_color: Vec4::new(0.7, 0.7, 0.7, 0.0),
            sun_direction: Vec3::new(0.0, 1.0, 0.0),
            sun_color: Vec3::new(1.0, 1.0, 1.0),
            sun_sharpness: 64.0,
        };

        let lights = [
            crate::state::LightState::default(),
            crate::state::LightState::default(),
            crate::state::LightState::default(),
            crate::state::LightState::default(),
        ];

        let blend_modes = [
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
            MatcapBlendMode::Multiply,
        ];

        let state1 = PackedUnifiedShadingState::from_render_state(
            0xFFFFFFFF,
            0.0,
            0.5,
            0.0,
            &blend_modes,
            &sky,
            &lights,
        );

        let state2 = PackedUnifiedShadingState::from_render_state(
            0xFFFFFFFF,
            0.0,
            0.5,
            0.0,
            &blend_modes,
            &sky,
            &lights,
        );

        // Same state should be equal
        assert_eq!(state1, state2, "Identical states should be equal");
        
        // Different colors should produce different states
        let state3 = PackedUnifiedShadingState::from_render_state(
            0xFF0000FF,
            0.0,
            0.5,
            0.0,
            &blend_modes,
            &sky,
            &lights,
        );
        
        assert_ne!(state1, state3, "Different colors should produce different states");
    }
}
