//! Mesh viewer controls and state

use super::ZXAssetViewer;

impl ZXAssetViewer {
    /// Rotate mesh view
    pub fn mesh_rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.mesh_rotation.0 += dyaw;
        self.mesh_rotation.1 = (self.mesh_rotation.1 + dpitch).clamp(-89.0, 89.0);
    }

    /// Zoom mesh view
    pub fn mesh_zoom(&mut self, delta: f32) {
        self.mesh_distance = (self.mesh_distance - delta).clamp(0.5, 50.0);
    }

    /// Reset mesh view
    pub fn mesh_reset_view(&mut self) {
        self.mesh_rotation = (0.0, 0.0);
        self.mesh_distance = 5.0;
    }

    /// Toggle wireframe overlay
    pub fn mesh_toggle_wireframe(&mut self) {
        self.mesh_wireframe = !self.mesh_wireframe;
    }

    /// Get mesh rotation (yaw, pitch)
    pub fn mesh_rotation(&self) -> (f32, f32) {
        self.mesh_rotation
    }

    /// Get mesh camera distance
    pub fn mesh_distance(&self) -> f32 {
        self.mesh_distance
    }

    /// Check if wireframe is enabled
    pub fn mesh_wireframe(&self) -> bool {
        self.mesh_wireframe
    }
}
