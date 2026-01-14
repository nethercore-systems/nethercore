//! Animation viewer controls and state

use super::ZXAssetViewer;

impl ZXAssetViewer {
    /// Toggle animation playback
    pub fn animation_toggle_play(&mut self) {
        self.animation_playing = !self.animation_playing;
    }

    /// Set animation frame
    pub fn animation_set_frame(&mut self, frame: u16) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            self.animation_frame = frame.min(anim.frame_count.saturating_sub(1));
        }
    }

    /// Step animation forward one frame
    pub fn animation_step_forward(&mut self) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            self.animation_frame = (self.animation_frame + 1) % anim.frame_count.max(1);
        }
    }

    /// Step animation backward one frame
    pub fn animation_step_back(&mut self) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            if self.animation_frame == 0 {
                self.animation_frame = anim.frame_count.saturating_sub(1);
            } else {
                self.animation_frame -= 1;
            }
        }
    }

    /// Get current animation frame
    pub fn animation_frame(&self) -> u16 {
        self.animation_frame
    }

    /// Check if animation is playing
    pub fn animation_is_playing(&self) -> bool {
        self.animation_playing
    }

    /// Set animation playback speed
    pub fn animation_set_speed(&mut self, speed: f32) {
        self.animation_speed = speed.clamp(0.1, 4.0);
    }
}
