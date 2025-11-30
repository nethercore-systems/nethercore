//! Render state types
//!
//! Provides render state for batching draw commands, lighting, and init-time configuration.

use glam::Mat4;

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// Light state for Mode 2/3 (PBR/Hybrid)
#[derive(Debug, Clone, Copy)]
pub struct LightState {
    /// Light enabled
    pub enabled: bool,
    /// Light direction (normalized)
    pub direction: [f32; 3],
    /// Light color (RGB, linear)
    pub color: [f32; 3],
    /// Light intensity multiplier
    pub intensity: f32,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: [0.0, -1.0, 0.0], // Default: downward
            color: [1.0, 1.0, 1.0],      // Default: white
            intensity: 1.0,
        }
    }
}

/// Current render state for batching
#[derive(Debug, Clone)]
pub struct RenderState {
    /// Uniform color tint (RGBA)
    pub color: u32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Cull mode: 0=none, 1=back, 2=front
    pub cull_mode: u8,
    /// Blend mode: 0=none, 1=alpha, 2=additive, 3=multiply
    pub blend_mode: u8,
    /// Texture filter: 0=nearest, 1=linear
    pub texture_filter: u8,
    /// Bound texture handles per slot
    pub bound_textures: [u32; 4],
    /// Matcap blend modes for slots 1-3 (Mode 1 only, [0] unused)
    /// Values: 0=Multiply, 1=Add, 2=HSV Modulate
    pub matcap_blend_modes: [u8; 4],
    /// Current render mode (0-3)
    pub render_mode: u8,
    /// Material metallic value (0.0-1.0, default 0.0)
    pub material_metallic: f32,
    /// Material roughness value (0.0-1.0, default 0.5)
    pub material_roughness: f32,
    /// Material emissive intensity (default 0.0)
    pub material_emissive: f32,
    /// Light states for Mode 2/3 (4 lights)
    pub lights: [LightState; 4],
    /// Bone transform matrices for GPU skinning (column-major, up to 256 bones)
    pub bone_matrices: Vec<Mat4>,
    /// Number of active bones
    pub bone_count: u32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 1, // Back-face culling by default
            blend_mode: 0,
            texture_filter: 0, // Nearest by default (retro look)
            bound_textures: [0; 4],
            matcap_blend_modes: [0; 4], // Multiply by default
            render_mode: 0,
            material_metallic: 0.0,
            material_roughness: 0.5,
            material_emissive: 0.0,
            lights: [LightState::default(); 4],
            bone_matrices: Vec::new(),
            bone_count: 0,
        }
    }
}

/// Configuration set during init (immutable after init)
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Resolution index (0-3 for Z: 360p, 540p, 720p, 1080p)
    pub resolution_index: u32,
    /// Tick rate index (0-3 for Z: 24, 30, 60, 120 fps)
    pub tick_rate_index: u32,
    /// Clear/background color (RGBA: 0xRRGGBBAA)
    pub clear_color: u32,
    /// Render mode (0-3: Unlit, Matcap, PBR, Hybrid)
    pub render_mode: u8,
    /// Whether any config was changed during init
    pub modified: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            resolution_index: 1,     // Default 540p
            tick_rate_index: 2,      // Default 60 fps
            clear_color: 0x000000FF, // Black, fully opaque
            render_mode: 0,          // Unlit
            modified: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use std::f32::consts::PI;

    // ============================================================================
    // LightState Tests
    // ============================================================================

    #[test]
    fn test_light_state_default() {
        let light = LightState::default();
        assert!(!light.enabled);
        assert_eq!(light.direction, [0.0, -1.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.intensity, 1.0);
    }

    // ============================================================================
    // RenderState Tests
    // ============================================================================

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF); // White, fully opaque
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, 1); // Back-face culling
        assert_eq!(state.blend_mode, 0); // No blending
        assert_eq!(state.texture_filter, 0); // Nearest
        assert_eq!(state.bound_textures, [0; 4]);
        assert_eq!(state.render_mode, 0); // Unlit
        assert_eq!(state.material_metallic, 0.0);
        assert_eq!(state.material_roughness, 0.5);
        assert_eq!(state.material_emissive, 0.0);
        assert_eq!(state.bone_count, 0);
        assert!(state.bone_matrices.is_empty());
    }

    #[test]
    fn test_render_state_lights_default() {
        let state = RenderState::default();
        for light in &state.lights {
            assert!(!light.enabled);
            assert_eq!(light.color, [1.0, 1.0, 1.0]);
            assert_eq!(light.intensity, 1.0);
        }
    }

    // ============================================================================
    // InitConfig Tests
    // ============================================================================

    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.resolution_index, 1); // 540p
        assert_eq!(config.tick_rate_index, 2); // 60fps
        assert_eq!(config.clear_color, 0x000000FF); // Black, opaque
        assert_eq!(config.render_mode, 0); // Unlit
        assert!(!config.modified);
    }

    // ============================================================================
    // GPU Skinning Tests
    // ============================================================================

    #[test]
    fn test_render_state_bone_matrices_empty_by_default() {
        let state = RenderState::default();
        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_can_store_bones() {
        let mut state = RenderState::default();

        // Add some bone matrices
        let bone1 = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let bone2 = Mat4::from_rotation_y(PI / 4.0);
        let bone3 = Mat4::from_scale(Vec3::splat(2.0));

        state.bone_matrices.push(bone1);
        state.bone_matrices.push(bone2);
        state.bone_matrices.push(bone3);
        state.bone_count = 3;

        assert_eq!(state.bone_matrices.len(), 3);
        assert_eq!(state.bone_count, 3);

        // Verify matrices are stored correctly
        assert_eq!(state.bone_matrices[0], bone1);
        assert_eq!(state.bone_matrices[1], bone2);
        assert_eq!(state.bone_matrices[2], bone3);
    }

    #[test]
    fn test_render_state_bone_matrices_max_capacity() {
        let mut state = RenderState::default();

        // Add MAX_BONES matrices
        for i in 0..MAX_BONES {
            let translation = Vec3::new(i as f32, 0.0, 0.0);
            state
                .bone_matrices
                .push(Mat4::from_translation(translation));
        }
        state.bone_count = MAX_BONES as u32;

        assert_eq!(state.bone_matrices.len(), MAX_BONES);
        assert_eq!(state.bone_count, MAX_BONES as u32);

        // Verify first and last bones
        let expected_first = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let expected_last = Mat4::from_translation(Vec3::new((MAX_BONES - 1) as f32, 0.0, 0.0));
        assert_eq!(state.bone_matrices[0], expected_first);
        assert_eq!(state.bone_matrices[MAX_BONES - 1], expected_last);
    }

    #[test]
    fn test_render_state_bone_matrices_clear() {
        let mut state = RenderState::default();

        // Add bones
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_count = 2;

        // Clear bones
        state.bone_matrices.clear();
        state.bone_count = 0;

        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_replace() {
        let mut state = RenderState::default();

        // Add initial bones
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_count = 2;

        // Replace with new bones
        let new_bone = Mat4::from_translation(Vec3::new(5.0, 5.0, 5.0));
        state.bone_matrices.clear();
        state.bone_matrices.push(new_bone);
        state.bone_count = 1;

        assert_eq!(state.bone_matrices.len(), 1);
        assert_eq!(state.bone_count, 1);
        assert_eq!(state.bone_matrices[0], new_bone);
    }

    #[test]
    fn test_render_state_bone_matrix_identity_transform() {
        // Verify identity matrix doesn't transform a vertex
        let identity = Mat4::IDENTITY;
        let vertex = Vec3::new(1.0, 2.0, 3.0);
        let transformed = identity.transform_point3(vertex);

        assert_eq!(transformed, vertex);
    }

    #[test]
    fn test_render_state_bone_matrix_weighted_blend() {
        // Simulate GPU skinning blend: position = sum(weight_i * bone_i * position)
        let bone1 = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let bone2 = Mat4::from_translation(Vec3::new(0.0, 10.0, 0.0));

        let vertex = Vec3::ZERO;
        let weight1 = 0.5f32;
        let weight2 = 0.5f32;

        // Transform by each bone
        let t1 = bone1.transform_point3(vertex);
        let t2 = bone2.transform_point3(vertex);

        // Weighted blend
        let blended = Vec3::new(
            t1.x * weight1 + t2.x * weight2,
            t1.y * weight1 + t2.y * weight2,
            t1.z * weight1 + t2.z * weight2,
        );

        // 50% of (10, 0, 0) + 50% of (0, 10, 0) = (5, 5, 0)
        assert!((blended.x - 5.0).abs() < 0.0001);
        assert!((blended.y - 5.0).abs() < 0.0001);
        assert!(blended.z.abs() < 0.0001);
    }

    #[test]
    fn test_render_state_bone_matrix_hierarchy() {
        // Simulate bone hierarchy: parent -> child
        let parent_bone = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let child_local = Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0));

        // Child's world transform = parent * child_local
        let child_world = parent_bone * child_local;

        let vertex = Vec3::ZERO;
        let transformed = child_world.transform_point3(vertex);

        // Origin should be at (5, 3, 0) in world space
        assert!((transformed.x - 5.0).abs() < 0.0001);
        assert!((transformed.y - 3.0).abs() < 0.0001);
        assert!(transformed.z.abs() < 0.0001);
    }
}
