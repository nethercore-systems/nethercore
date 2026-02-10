//! Tracker viewer controls and state

use std::sync::Arc;

use crate::audio::{AudioOutput, Sound};
use crate::state::{TrackerState, tracker_flags};
use crate::tracker::TrackerEngine;

use super::ZXAssetViewer;

impl ZXAssetViewer {
    /// Start tracker playback
    pub fn start_tracker_playback(&mut self) {
        // Get tracker data (clone what we need to avoid borrow conflicts)
        let (format, pattern_data, sample_ids) = match self.selected_tracker() {
            Some(tracker) => (
                tracker.format,
                tracker.pattern_data.clone(),
                tracker.sample_ids.clone(),
            ),
            None => return,
        };

        // Initialize audio output if needed
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

        // Load sounds from data pack using tracker's sample_ids
        // Sound handles are 1-indexed (0 = no sound)
        let mut sounds: Vec<Option<Sound>> = vec![None]; // Index 0 is unused
        let mut sound_handles: Vec<u32> = Vec::new();

        for sample_id in &sample_ids {
            if let Some(packed_sound) = self.data_pack.sounds.iter().find(|s| &s.id == sample_id) {
                // Convert PackedSound to Sound
                let sound = Sound {
                    data: Arc::new(packed_sound.data.clone()),
                };
                sounds.push(Some(sound));
                sound_handles.push(sounds.len() as u32 - 1); // Handle points to index
            } else {
                eprintln!(
                    "Warning: tracker sample '{}' not found in data pack",
                    sample_id
                );
                sounds.push(None);
                sound_handles.push(sounds.len() as u32 - 1);
            }
        }

        self.tracker_sounds = sounds;

        // Initialize tracker engine
        let mut engine = TrackerEngine::new();
        let (handle, speed, bpm) = match format {
            zx_common::TrackerFormat::Xm => {
                let xm_module = match nether_xm::parse_xm_minimal(&pattern_data) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to parse XM tracker: {:?}", e);
                        return;
                    }
                };
                let speed = xm_module.default_speed as u16;
                let bpm = xm_module.default_bpm as u16;
                let handle = engine.load_xm_module(xm_module, sound_handles);
                (handle, speed, bpm)
            }
            zx_common::TrackerFormat::It => {
                let it_module = match nether_it::parse_it_minimal(&pattern_data) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to parse IT tracker: {:?}", e);
                        return;
                    }
                };
                let speed = it_module.initial_speed as u16;
                let bpm = it_module.initial_tempo as u16;
                let handle = engine.load_it_module(it_module, sound_handles);
                (handle, speed, bpm)
            }
        };

        // Initialize tracker state
        let state = TrackerState {
            handle,
            flags: tracker_flags::PLAYING,
            speed,
            bpm,
            volume: 256, // Full volume
            ..Default::default()
        };

        self.tracker_engine = Some(engine);
        self.tracker_state = Some(state);
        self.tracker_playing = true;
    }
}
