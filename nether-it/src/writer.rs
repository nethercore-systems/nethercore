//! IT file writer
//!
//! Generates complete IT files with embedded sample data, suitable for preview
//! in external trackers (MilkyTracker, OpenMPT, SchismTracker).

use crate::module::{
    ItEnvelope, ItFlags, ItInstrument, ItModule, ItNote, ItPattern, ItSample, ItSampleFlags,
};
use crate::{INSTRUMENT_MAGIC, IT_MAGIC, MAX_ENVELOPE_POINTS, SAMPLE_MAGIC};

/// IT file writer for generating complete IT modules
#[derive(Debug)]
pub struct ItWriter {
    module: ItModule,
    /// Sample audio data (parallel to module.samples)
    sample_data: Vec<Vec<i16>>,
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
        let mut output = Vec::new();

        // Calculate offsets
        let header_size = 192;
        let orders_size = self.module.num_orders as usize;

        // Offset table starts after header + orders
        let offset_table_start = header_size + orders_size;
        let num_instruments = self.module.num_instruments as usize;
        let num_samples = self.module.num_samples as usize;
        let num_patterns = self.module.num_patterns as usize;

        let offset_table_size = (num_instruments + num_samples + num_patterns) * 4;

        // Message (if any)
        let message_offset = if self.module.message.is_some() {
            offset_table_start + offset_table_size
        } else {
            0
        };
        let message_size = self
            .module
            .message
            .as_ref()
            .map(|m| m.len() + 1)
            .unwrap_or(0);

        // Instruments start after message
        let instruments_start = offset_table_start + offset_table_size + message_size;

        // Pre-calculate instrument offsets
        // Instrument size: 4 (magic) + 12 (filename) + 1 (reserved) + 3 (NNA/DCT/DCA) +
        //                  2 (fadeout) + 2 (PPS/PPC) + 2 (GbV/DfP) + 2 (RV/RP) +
        //                  4 (TrkVers) + 26 (name) + 2 (IFC/IFR) + 4 (MIDI) +
        //                  240 (note-sample table) + 3*82 (envelopes) = 550 bytes
        let instrument_size = 550;
        let mut instrument_offsets = Vec::new();
        for i in 0..num_instruments {
            instrument_offsets.push((instruments_start + i * instrument_size) as u32);
        }

        // Samples start after instruments
        let samples_start = instruments_start + num_instruments * instrument_size;
        let sample_header_size = 80;
        let mut sample_offsets = Vec::new();
        for i in 0..num_samples {
            sample_offsets.push((samples_start + i * sample_header_size) as u32);
        }

        // Patterns start after sample headers
        let patterns_start = samples_start + num_samples * sample_header_size;

        // Pack patterns and calculate their sizes/offsets
        let mut packed_patterns = Vec::new();
        let mut pattern_offsets = Vec::new();
        let mut current_pattern_offset = patterns_start;

        for pattern in &self.module.patterns {
            let packed = pack_pattern(pattern, self.module.num_channels);
            pattern_offsets.push(current_pattern_offset as u32);
            current_pattern_offset += 8 + packed.len(); // 8 byte header + data
            packed_patterns.push(packed);
        }

        // Sample data starts after patterns
        let sample_data_start = current_pattern_offset;
        let mut sample_data_offsets = Vec::new();
        let mut current_data_offset = sample_data_start;

        for data in &self.sample_data {
            sample_data_offsets.push(current_data_offset as u32);
            current_data_offset += data.len() * 2; // 16-bit samples
        }

        // ========== Write Header ==========

        // Magic "IMPM"
        output.extend_from_slice(IT_MAGIC);

        // Song name (26 bytes)
        write_string(&mut output, &self.module.name, 26);

        // PHilight (2 bytes) - row highlight info
        output.extend_from_slice(&[0x04, 0x10]); // Default: 4/16 highlight

        // OrdNum (2 bytes)
        output.extend_from_slice(&(self.module.num_orders).to_le_bytes());

        // InsNum (2 bytes)
        output.extend_from_slice(&(self.module.num_instruments).to_le_bytes());

        // SmpNum (2 bytes)
        output.extend_from_slice(&(self.module.num_samples).to_le_bytes());

        // PatNum (2 bytes)
        output.extend_from_slice(&(self.module.num_patterns).to_le_bytes());

        // Cwt/v (2 bytes) - created with version
        output.extend_from_slice(&self.module.created_with.to_le_bytes());

        // Cmwt (2 bytes) - compatible with version
        output.extend_from_slice(&self.module.compatible_with.to_le_bytes());

        // Flags (2 bytes)
        output.extend_from_slice(&self.module.flags.bits().to_le_bytes());

        // Special (2 bytes)
        output.extend_from_slice(&self.module.special.to_le_bytes());

        // GV (1 byte)
        output.push(self.module.global_volume);

        // MV (1 byte)
        output.push(self.module.mix_volume);

        // IS (1 byte)
        output.push(self.module.initial_speed);

        // IT (1 byte)
        output.push(self.module.initial_tempo);

        // Sep (1 byte)
        output.push(self.module.panning_separation);

        // PWD (1 byte)
        output.push(self.module.pitch_wheel_depth);

        // MsgLgth (2 bytes)
        let msg_len = self.module.message.as_ref().map(|m| m.len()).unwrap_or(0) as u16;
        output.extend_from_slice(&msg_len.to_le_bytes());

        // MsgOff (4 bytes)
        output.extend_from_slice(&(message_offset as u32).to_le_bytes());

        // Reserved (4 bytes)
        output.extend_from_slice(&[0u8; 4]);

        // Channel pan (64 bytes)
        output.extend_from_slice(&self.module.channel_pan);

        // Channel vol (64 bytes)
        output.extend_from_slice(&self.module.channel_vol);

        // ========== Write Order Table ==========
        for &order in &self.module.order_table {
            output.push(order);
        }

        // ========== Write Offset Tables ==========
        for &offset in &instrument_offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        for &offset in &sample_offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        for &offset in &pattern_offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }

        // ========== Write Message ==========
        if let Some(ref msg) = self.module.message {
            output.extend_from_slice(msg.as_bytes());
            output.push(0); // Null terminator
        }

        // ========== Write Instruments ==========
        for instrument in &self.module.instruments {
            write_instrument(&mut output, instrument);
        }

        // ========== Write Sample Headers ==========
        for (i, sample) in self.module.samples.iter().enumerate() {
            let data_offset = sample_data_offsets.get(i).copied().unwrap_or(0);
            write_sample_header(&mut output, sample, data_offset);
        }

        // ========== Write Patterns ==========
        for (i, packed) in packed_patterns.iter().enumerate() {
            let pattern = &self.module.patterns[i];
            // Pattern header (8 bytes)
            output.extend_from_slice(&(packed.len() as u16).to_le_bytes()); // Length
            output.extend_from_slice(&pattern.num_rows.to_le_bytes()); // Rows
            output.extend_from_slice(&[0u8; 4]); // Reserved
            // Pattern data
            output.extend_from_slice(packed);
        }

        // ========== Write Sample Data ==========
        for data in &self.sample_data {
            for &sample in data {
                output.extend_from_slice(&sample.to_le_bytes());
            }
        }

        output
    }
}

/// Write a fixed-length string, padded with zeros
fn write_string(output: &mut Vec<u8>, s: &str, len: usize) {
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(len);
    output.extend_from_slice(&bytes[..copy_len]);
    for _ in copy_len..len {
        output.push(0);
    }
}

/// Write an instrument to the output
fn write_instrument(output: &mut Vec<u8>, instr: &ItInstrument) {
    // Magic "IMPI"
    output.extend_from_slice(INSTRUMENT_MAGIC);

    // DOS filename (12 bytes)
    write_string(output, &instr.filename, 12);

    // Reserved (1 byte)
    output.push(0);

    // NNA, DCT, DCA
    output.push(instr.nna as u8);
    output.push(instr.dct as u8);
    output.push(instr.dca as u8);

    // Fadeout (2 bytes)
    output.extend_from_slice(&instr.fadeout.to_le_bytes());

    // PPS, PPC
    output.push(instr.pitch_pan_separation as u8);
    output.push(instr.pitch_pan_center);

    // GbV, DfP
    output.push(instr.global_volume);
    let dfp = instr.default_pan.map(|p| p | 0x80).unwrap_or(32);
    output.push(dfp);

    // RV, RP
    output.push(instr.random_volume);
    output.push(instr.random_pan);

    // TrkVers, NoS (4 bytes) - for instrument files only
    output.extend_from_slice(&[0u8; 4]);

    // Instrument name (26 bytes)
    write_string(output, &instr.name, 26);

    // IFC, IFR
    let ifc = instr.filter_cutoff.map(|c| c | 0x80).unwrap_or(0);
    let ifr = instr.filter_resonance.map(|r| r | 0x80).unwrap_or(0);
    output.push(ifc);
    output.push(ifr);

    // MCh, MPr, MIDIBnk
    output.push(instr.midi_channel);
    output.push(instr.midi_program);
    output.extend_from_slice(&instr.midi_bank.to_le_bytes());

    // Note-Sample-Keyboard table (240 bytes)
    for &(note, sample) in &instr.note_sample_table {
        output.push(note);
        output.push(sample);
    }

    // Volume envelope
    write_envelope(output, instr.volume_envelope.as_ref());

    // Panning envelope
    write_envelope(output, instr.panning_envelope.as_ref());

    // Pitch envelope
    write_envelope(output, instr.pitch_envelope.as_ref());
}

/// Write an envelope
fn write_envelope(output: &mut Vec<u8>, env: Option<&ItEnvelope>) {
    let env = env.cloned().unwrap_or_default();

    // Flags (1 byte)
    output.push(env.flags.bits());

    // Num points (1 byte)
    output.push(env.points.len().min(MAX_ENVELOPE_POINTS) as u8);

    // LpB, LpE, SLB, SLE
    output.push(env.loop_begin);
    output.push(env.loop_end);
    output.push(env.sustain_begin);
    output.push(env.sustain_end);

    // Node data (75 bytes = 25 × 3)
    for i in 0..MAX_ENVELOPE_POINTS {
        if let Some(&(tick, value)) = env.points.get(i) {
            output.push(value as u8);
            output.extend_from_slice(&tick.to_le_bytes());
        } else {
            output.extend_from_slice(&[0u8; 3]);
        }
    }

    // Reserved (1 byte)
    output.push(0);
}

/// Write a sample header
fn write_sample_header(output: &mut Vec<u8>, sample: &ItSample, data_offset: u32) {
    // Magic "IMPS"
    output.extend_from_slice(SAMPLE_MAGIC);

    // DOS filename (12 bytes)
    write_string(output, &sample.filename, 12);

    // Reserved (1 byte)
    output.push(0);

    // GvL
    output.push(sample.global_volume);

    // Flg - add SAMPLE_16BIT flag since we always write 16-bit
    let flags = sample.flags | ItSampleFlags::SAMPLE_16BIT;
    output.push(flags.bits());

    // Vol
    output.push(sample.default_volume);

    // Sample name (26 bytes)
    write_string(output, &sample.name, 26);

    // Cvt (convert flags) - 0x01 = signed samples
    output.push(0x01);

    // DfP
    let dfp = sample.default_pan.map(|p| p | 0x80).unwrap_or(0);
    output.push(dfp);

    // Length (4 bytes)
    output.extend_from_slice(&sample.length.to_le_bytes());

    // LoopBeg (4 bytes)
    output.extend_from_slice(&sample.loop_begin.to_le_bytes());

    // LoopEnd (4 bytes)
    output.extend_from_slice(&sample.loop_end.to_le_bytes());

    // C5Speed (4 bytes)
    output.extend_from_slice(&sample.c5_speed.to_le_bytes());

    // SusLBeg (4 bytes)
    output.extend_from_slice(&sample.sustain_loop_begin.to_le_bytes());

    // SusLEnd (4 bytes)
    output.extend_from_slice(&sample.sustain_loop_end.to_le_bytes());

    // SmpPoint (4 bytes) - offset to sample data
    output.extend_from_slice(&data_offset.to_le_bytes());

    // ViS, ViD, ViR, ViT
    output.push(sample.vibrato_speed);
    output.push(sample.vibrato_depth);
    output.push(sample.vibrato_rate);
    output.push(sample.vibrato_type);
}

/// Pack pattern data using IT compression
fn pack_pattern(pattern: &ItPattern, num_channels: u8) -> Vec<u8> {
    let mut output = Vec::new();

    // Previous values for compression
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    for row in &pattern.notes {
        for (channel, note) in row.iter().enumerate().take(num_channels as usize) {
            // Skip empty notes
            if note.note == 0
                && note.instrument == 0
                && note.volume == 0
                && note.effect == 0
                && note.effect_param == 0
            {
                continue;
            }

            // Build mask
            let mut mask = 0u8;

            if note.note != 0 && note.note != prev_note[channel] {
                mask |= 0x01;
                prev_note[channel] = note.note;
            } else if note.note != 0 {
                mask |= 0x10;
            }

            if note.instrument != 0 && note.instrument != prev_instrument[channel] {
                mask |= 0x02;
                prev_instrument[channel] = note.instrument;
            } else if note.instrument != 0 {
                mask |= 0x20;
            }

            if note.volume != 0 && note.volume != prev_volume[channel] {
                mask |= 0x04;
                prev_volume[channel] = note.volume;
            } else if note.volume != 0 {
                mask |= 0x40;
            }

            if (note.effect != 0 || note.effect_param != 0)
                && (note.effect != prev_effect[channel]
                    || note.effect_param != prev_effect_param[channel])
            {
                mask |= 0x08;
                prev_effect[channel] = note.effect;
                prev_effect_param[channel] = note.effect_param;
            } else if note.effect != 0 || note.effect_param != 0 {
                mask |= 0x80;
            }

            if mask == 0 {
                continue;
            }

            // Write channel marker with mask flag
            output.push((channel as u8) | 0x80);
            output.push(mask);

            // Write data
            if mask & 0x01 != 0 {
                output.push(note.note);
            }
            if mask & 0x02 != 0 {
                output.push(note.instrument);
            }
            if mask & 0x04 != 0 {
                output.push(note.volume);
            }
            if mask & 0x08 != 0 {
                output.push(note.effect);
                output.push(note.effect_param);
            }
        }

        // End of row marker
        output.push(0);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_it;

    #[test]
    fn test_write_empty_module() {
        let mut writer = ItWriter::new("Test Song");
        writer.set_channels(4);
        writer.set_speed(6);
        writer.set_tempo(125);

        // Add a simple pattern
        let pat = writer.add_pattern(64);
        writer.set_orders(&[pat]);

        let data = writer.write();

        // Verify magic
        assert_eq!(&data[0..4], IT_MAGIC);

        // Try to parse it back
        let result = parse_it(&data);
        assert!(
            result.is_ok(),
            "Failed to parse written IT: {:?}",
            result.err()
        );

        let module = result.unwrap();
        assert_eq!(module.name, "Test Song");
        assert_eq!(module.initial_speed, 6);
        assert_eq!(module.initial_tempo, 125);
    }

    #[test]
    fn test_write_with_instrument() {
        let mut writer = ItWriter::new("Instr Test");
        writer.set_channels(4);
        writer.set_speed(6);
        writer.set_tempo(125);

        // Add an instrument
        let mut instr = ItInstrument::default();
        instr.name = "Kick".to_string();
        writer.add_instrument(instr);

        // Add a sample
        let mut sample = ItSample::default();
        sample.name = "Kick Sample".to_string();
        sample.c5_speed = 22050;
        let audio = vec![0i16; 1000]; // 1000 samples of silence
        writer.add_sample(sample, &audio);

        // Add a pattern and order table
        let pat = writer.add_pattern(64);
        writer.set_orders(&[pat]);

        let data = writer.write();
        let module = parse_it(&data).expect("Failed to parse written IT file");

        assert_eq!(module.num_instruments, 1);
        assert_eq!(module.num_samples, 1);
        assert_eq!(module.instruments[0].name, "Kick");
        assert_eq!(module.samples[0].name, "Kick Sample");
    }
}
