//! Camera state types
//!
//! Provides camera state for 3D rendering with view and projection matrix calculations.

use glam::{Mat4, Vec3};

/// Default camera field of view in degrees
pub const DEFAULT_CAMERA_FOV: f32 = 60.0;

/// Camera state for 3D rendering
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target (look-at point) in world space
    pub target: Vec3,
    /// Field of view in degrees
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            fov: DEFAULT_CAMERA_FOV,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraState {
    /// Compute the view matrix (world-to-camera transform)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    /// Compute the projection matrix for a given aspect ratio
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), aspect_ratio, self.near, self.far)
    }

    /// Compute the combined view-projection matrix
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_state_default() {
        let camera = CameraState::default();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.fov, DEFAULT_CAMERA_FOV);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
    }

    #[test]
    fn test_camera_state_view_matrix_identity_position() {
        let camera = CameraState {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        let view = camera.view_matrix();
        // View matrix should transform world origin to be in front of camera
        let world_origin = Vec3::ZERO;
        let view_space = view.transform_point3(world_origin);
        // Origin should be at z=-1 in view space (1 unit in front of camera)
        assert!((view_space.z - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_camera_state_view_matrix_translation() {
        let camera = CameraState {
            position: Vec3::new(10.0, 0.0, 0.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        let view = camera.view_matrix();
        // Target should be transformed to be in front of camera
        let target_view_space = view.transform_point3(camera.target);
        // Target should be at negative Z (in front of camera)
        assert!(target_view_space.z < 0.0);
    }

    #[test]
    fn test_camera_state_projection_matrix_aspect_ratio() {
        let camera = CameraState::default();
        let proj_16_9 = camera.projection_matrix(16.0 / 9.0);
        let proj_4_3 = camera.projection_matrix(4.0 / 3.0);
        // Different aspect ratios should produce different matrices
        assert_ne!(proj_16_9, proj_4_3);
    }

    #[test]
    fn test_camera_state_projection_matrix_fov() {
        let camera_narrow = CameraState {
            fov: 45.0,
            ..Default::default()
        };
        let camera_wide = CameraState {
            fov: 90.0,
            ..Default::default()
        };
        let proj_narrow = camera_narrow.projection_matrix(1.0);
        let proj_wide = camera_wide.projection_matrix(1.0);
        // Different FOV should produce different matrices
        assert_ne!(proj_narrow, proj_wide);
    }

    #[test]
    fn test_camera_state_view_projection_matrix() {
        let camera = CameraState::default();
        let aspect = 16.0 / 9.0;
        let vp = camera.view_projection_matrix(aspect);
        let expected = camera.projection_matrix(aspect) * camera.view_matrix();
        assert_eq!(vp, expected);
    }
}
