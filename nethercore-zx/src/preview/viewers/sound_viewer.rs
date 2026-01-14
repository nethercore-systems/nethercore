//! Sound viewer controls and state

use crate::audio::AudioOutput;

use super::ZXAssetViewer;

impl ZXAssetViewer {
    /// Toggle sound playback
    pub fn sound_toggle_play(&mut self) {
        if self.sound_playing {
            // Stop playback
            self.sound_stop();
        } else {
            // Get sound data
            let sound_data = self.selected_sound().map(|s| s.data.clone());

            if let Some(data) = sound_data {
                // Initialize audio output if not already created
                if self.audio_output.is_none() {
                    match AudioOutput::new() {
                        Ok(output) => {
                            self.audio_output = Some(output);
                        }
                        Err(e) => {
                            eprintln!("Failed to initialize audio: {}", e);
                            return;
                        }
                    }
                }

                // Convert mono i16 to stereo f32 and upsample 22050->44100
                let mut stereo_samples = Vec::with_capacity(data.len() * 4); // *2 for upsample, *2 for stereo
                for sample in data.iter() {
                    let f_sample = *sample as f32 / i16::MAX as f32;
                    // Upsample: duplicate each sample for 44100 Hz output
                    stereo_samples.push(f_sample); // Left
                    stereo_samples.push(f_sample); // Right
                    stereo_samples.push(f_sample); // Left (duplicate)
                    stereo_samples.push(f_sample); // Right (duplicate)
                }

                // Push entire sound to buffer at once
                if let Some(ref mut audio_output) = self.audio_output {
                    audio_output.push_samples(&stereo_samples);
                }

                // Start playing
                self.sound_playing = true;
                self.sound_position = 0;
            }
        }
    }

    /// Stop sound and reset position
    pub fn sound_stop(&mut self) {
        self.sound_playing = false;
        self.sound_position = 0;
    }

    /// Seek sound to position (0.0 - 1.0)
    pub fn sound_seek(&mut self, position: f32) {
        if let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            self.sound_position = ((position.clamp(0.0, 1.0) * sound.data.len() as f32) as usize)
                .min(sound.data.len().saturating_sub(1));
        }
    }

    /// Get sound playback progress (0.0 - 1.0)
    pub fn sound_progress(&self) -> f32 {
        if let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            if sound.data.is_empty() {
                return 0.0;
            }
            return self.sound_position as f32 / sound.data.len() as f32;
        }
        0.0
    }

    /// Check if sound is playing
    pub fn sound_is_playing(&self) -> bool {
        self.sound_playing
    }
}
