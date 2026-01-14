//! IT file writer
//!
//! Generates complete IT files with embedded sample data, suitable for preview
//! in external trackers (MilkyTracker, OpenMPT, SchismTracker).

use crate::module::{ItInstrument, ItFlags, ItModule, ItNote, ItPattern, ItSample, ItSampleFlags};

mod encoding;
mod pattern_packer;
mod serializer;

#[cfg(test)]
mod tests;

// Re-export for internal use within writer module
pub(crate) use encoding::{write_instrument, write_sample_header, write_string};
pub(crate) use pattern_packer::pack_pattern;

/// IT file writer for generating complete IT modules
#[derive(Debug)]
pub struct ItWriter {
    pub(crate) module: ItModule,
    /// Sample audio data (parallel to module.samples)
    pub(crate) sample_data: Vec<Vec<i16>>,
}

impl Default for ItWriter {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

impl ItWriter {
    /// Create a new IT writer with the given song name
    pub fn new(name: &str) -> Self {
        Self {
            module: ItModule {
                name: name.chars().take(26).collect(),
                ..Default::default()
            },
            sample_data: Vec::new(),
        }
    }

    /// Set the song name
    pub fn set_name(&mut self, name: &str) {
        self.module.name = name.chars().take(26).collect();
    }

    /// Set initial speed (ticks per row)
    pub fn set_speed(&mut self, speed: u8) {
        self.module.initial_speed = speed.max(1);
    }

    /// Set initial tempo (BPM)
    pub fn set_tempo(&mut self, tempo: u8) {
        self.module.initial_tempo = tempo.max(32);
    }

    /// Set global volume (0-128)
    pub fn set_global_volume(&mut self, volume: u8) {
        self.module.global_volume = volume.min(128);
    }

    /// Set mix volume (0-128)
    pub fn set_mix_volume(&mut self, volume: u8) {
        self.module.mix_volume = volume.min(128);
    }

    /// Set number of channels (1-64)
    pub fn set_channels(&mut self, channels: u8) {
        self.module.num_channels = channels.clamp(1, 64);
        // Set up channel pan/vol for enabled channels
        for i in 0..64 {
            if i < channels {
                self.module.channel_pan[i as usize] = 32; // Center
                self.module.channel_vol[i as usize] = 64; // Full volume
            } else {
                self.module.channel_pan[i as usize] = 128; // Disabled
                self.module.channel_vol[i as usize] = 0;
            }
        }
    }

    /// Set module flags
    pub fn set_flags(&mut self, flags: ItFlags) {
        self.module.flags = flags;
    }

    /// Add an instrument
    /// Returns the 1-based instrument number
    pub fn add_instrument(&mut self, instrument: ItInstrument) -> u8 {
        self.module.instruments.push(instrument);
        self.module.num_instruments = self.module.instruments.len() as u16;
        self.module.instruments.len() as u8
    }

    /// Add a sample with audio data
    /// Returns the 1-based sample number
    pub fn add_sample(&mut self, sample: ItSample, audio_data: &[i16]) -> u8 {
        let mut sample = sample;
        sample.length = audio_data.len() as u32;
        sample.flags = sample.flags | ItSampleFlags::HAS_DATA;

        self.module.samples.push(sample);
        self.sample_data.push(audio_data.to_vec());
        self.module.num_samples = self.module.samples.len() as u16;
        self.module.samples.len() as u8
    }

    /// Add a sample header without audio data (for stripped IT files)
    /// Returns the 1-based sample number
    pub fn add_sample_header_only(&mut self, sample: ItSample) -> u8 {
        let mut sample = sample;
        sample.length = 0; // Force zero length
        sample.flags =
            ItSampleFlags::from_bits(sample.flags.bits() & !ItSampleFlags::HAS_DATA.bits()); // Clear HAS_DATA flag

        self.module.samples.push(sample);
        self.sample_data.push(Vec::new()); // Empty audio data
        self.module.num_samples = self.module.samples.len() as u16;
        self.module.samples.len() as u8
    }

    /// Add an empty pattern with the given number of rows
    /// Returns the 0-based pattern index
    pub fn add_pattern(&mut self, rows: u16) -> u8 {
        let rows = rows.clamp(1, 200);
        let pattern = ItPattern::empty(rows, self.module.num_channels);
        self.module.patterns.push(pattern);
        self.module.num_patterns = self.module.patterns.len() as u16;
        (self.module.patterns.len() - 1) as u8
    }

    /// Set a note in a pattern
    pub fn set_note(&mut self, pattern: u8, row: u16, channel: u8, note: ItNote) {
        if let Some(pat) = self.module.patterns.get_mut(pattern as usize)
            && let Some(row_data) = pat.notes.get_mut(row as usize)
            && let Some(cell) = row_data.get_mut(channel as usize)
        {
            *cell = note;
        }
    }

    /// Set the pattern order table
    pub fn set_orders(&mut self, orders: &[u8]) {
        self.module.order_table = orders.to_vec();
        self.module.num_orders = orders.len() as u16;
    }

    /// Set a song message
    pub fn set_message(&mut self, message: &str) {
        self.module.message = Some(message.to_string());
        self.module.special |= 1; // Enable message
    }

    /// Build and return the complete IT file as bytes
    pub fn write(&self) -> Vec<u8> {
        serializer::serialize_module(self)
    }
}
