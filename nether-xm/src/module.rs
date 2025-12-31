//! XM module data structures

/// Parsed XM module (patterns only - samples loaded from ROM)
#[derive(Debug, Clone)]
pub struct XmModule {
    /// Module name (max 20 chars)
    pub name: String,
    /// Number of channels (1-32)
    pub num_channels: u8,
    /// Number of patterns
    pub num_patterns: u16,
    /// Number of instruments
    pub num_instruments: u16,
    /// Song length in pattern order entries
    pub song_length: u16,
    /// Restart position for looping
    pub restart_position: u16,
    /// Default speed (ticks per row)
    pub default_speed: u16,
    /// Default BPM
    pub default_bpm: u16,
    /// Use linear frequency table (vs Amiga)
    pub linear_frequency_table: bool,
    /// Pattern order table (which pattern to play in order)
    pub order_table: Vec<u8>,
    /// Pattern data
    pub patterns: Vec<XmPattern>,
    /// Instrument metadata (names map to ROM sample IDs)
    pub instruments: Vec<XmInstrument>,
}

impl XmModule {
    /// Get the pattern at the given order position
    pub fn pattern_at_order(&self, order: u16) -> Option<&XmPattern> {
        let pattern_idx = *self.order_table.get(order as usize)? as usize;
        self.patterns.get(pattern_idx)
    }

    /// Get total number of orders in the song
    pub fn total_orders(&self) -> u16 {
        self.song_length
    }
}

/// XM pattern containing rows of note data
#[derive(Debug, Clone)]
pub struct XmPattern {
    /// Number of rows in this pattern (1-256)
    pub num_rows: u16,
    /// Unpacked note data: [row][channel]
    pub notes: Vec<Vec<XmNote>>,
}

impl XmPattern {
    /// Get note at specific row and channel
    pub fn get_note(&self, row: u16, channel: u8) -> Option<&XmNote> {
        self.notes.get(row as usize)?.get(channel as usize)
    }

    /// Create an empty pattern with the given dimensions
    pub fn empty(num_rows: u16, num_channels: u8) -> Self {
        let mut notes = Vec::with_capacity(num_rows as usize);
        for _ in 0..num_rows {
            notes.push(vec![XmNote::default(); num_channels as usize]);
        }
        Self { num_rows, notes }
    }
}

/// Single note/command in a pattern
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct XmNote {
    /// Note value: 0=none, 1-96=C-0..B-7, 97=note-off
    pub note: u8,
    /// Instrument number: 0=none, 1-128=instrument
    pub instrument: u8,
    /// Volume column: 0=none, 0x10-0x50=set volume, others=effects
    pub volume: u8,
    /// Effect command (0-35)
    pub effect: u8,
    /// Effect parameter
    pub effect_param: u8,
}

impl XmNote {
    /// Check if this is a note-off
    #[inline]
    pub fn is_note_off(&self) -> bool {
        self.note == crate::NOTE_OFF
    }

    /// Check if this note triggers a new note
    #[inline]
    pub fn has_note(&self) -> bool {
        self.note >= crate::NOTE_MIN && self.note <= crate::NOTE_MAX
    }

    /// Check if this sets an instrument
    #[inline]
    pub fn has_instrument(&self) -> bool {
        self.instrument > 0
    }

    /// Check if there's an effect
    #[inline]
    pub fn has_effect(&self) -> bool {
        self.effect != 0 || self.effect_param != 0
    }

    /// Get volume value if volume column contains set-volume (0x10-0x50)
    #[inline]
    pub fn get_volume(&self) -> Option<u8> {
        if self.volume >= 0x10 && self.volume <= 0x50 {
            Some(self.volume - 0x10)
        } else {
            None
        }
    }

    /// Get volume column effect if present
    /// Returns (effect_type, parameter) where:
    /// - 0x60-0x6F: Volume slide down
    /// - 0x70-0x7F: Volume slide up
    /// - 0x80-0x8F: Fine volume slide down
    /// - 0x90-0x9F: Fine volume slide up
    /// - 0xA0-0xAF: Vibrato speed
    /// - 0xB0-0xBF: Vibrato depth
    /// - 0xC0-0xCF: Set panning
    /// - 0xD0-0xDF: Panning slide left
    /// - 0xE0-0xEF: Panning slide right
    /// - 0xF0-0xFF: Tone portamento
    pub fn get_volume_effect(&self) -> Option<(u8, u8)> {
        if self.volume >= 0x60 {
            Some((self.volume >> 4, self.volume & 0x0F))
        } else {
            None
        }
    }

    /// Convert note number to octave and semitone
    /// Returns (octave 0-7, semitone 0-11) where semitone 0=C, 11=B
    pub fn note_to_octave_semitone(&self) -> Option<(u8, u8)> {
        if self.has_note() {
            let n = self.note - 1; // Convert 1-96 to 0-95
            Some((n / 12, n % 12))
        } else {
            None
        }
    }

    /// Convert note number to frequency period (linear frequency table)
    /// Uses the standard XM period calculation
    pub fn note_to_period(&self, finetune: i8) -> Option<f32> {
        if !self.has_note() {
            return None;
        }

        // Linear period calculation
        // Period = 10*12*16*4 - Note*16*4 - FineTune/2
        let note = self.note as i32 - 1;
        let ft = finetune as i32;
        let period = 10 * 12 * 16 * 4 - note * 16 * 4 - ft / 2;

        Some(period as f32)
    }
}

/// XM instrument metadata (no sample data - that's in ROM)
#[derive(Debug, Clone, Default)]
pub struct XmInstrument {
    /// Instrument name (used to map to ROM sample ID)
    pub name: String,
    /// Number of samples in original XM (for reference)
    pub num_samples: u8,
    /// Volume envelope
    pub volume_envelope: Option<XmEnvelope>,
    /// Panning envelope
    pub panning_envelope: Option<XmEnvelope>,
    /// Auto-vibrato type (0=sine, 1=square, 2=ramp down, 3=ramp up)
    pub vibrato_type: u8,
    /// Auto-vibrato sweep
    pub vibrato_sweep: u8,
    /// Auto-vibrato depth
    pub vibrato_depth: u8,
    /// Auto-vibrato rate
    pub vibrato_rate: u8,
    /// Volume fadeout value (0-4095)
    pub volume_fadeout: u16,
    /// Sample finetune (-128 to 127)
    pub sample_finetune: i8,
    /// Sample relative note (semitones from C-4)
    pub sample_relative_note: i8,
    /// Sample loop start (in samples)
    pub sample_loop_start: u32,
    /// Sample loop length (in samples)
    pub sample_loop_length: u32,
    /// Sample loop type (0=none, 1=forward, 2=ping-pong)
    pub sample_loop_type: u8,
}

impl XmInstrument {
    /// Check if this instrument has a loop
    #[inline]
    pub fn has_loop(&self) -> bool {
        self.sample_loop_type != 0 && self.sample_loop_length > 0
    }

    /// Check if this is a ping-pong (bidirectional) loop
    #[inline]
    pub fn is_pingpong_loop(&self) -> bool {
        self.sample_loop_type == 2
    }

    /// Get the sample loop end position
    #[inline]
    pub fn sample_loop_end(&self) -> u32 {
        self.sample_loop_start + self.sample_loop_length
    }
}

/// Volume/panning envelope
#[derive(Debug, Clone, Default)]
pub struct XmEnvelope {
    /// Envelope points: (x=tick, y=value 0-64)
    pub points: Vec<(u16, u16)>,
    /// Sustain point index
    pub sustain_point: u8,
    /// Loop start point index
    pub loop_start: u8,
    /// Loop end point index
    pub loop_end: u8,
    /// Envelope is enabled
    pub enabled: bool,
    /// Sustain is enabled
    pub sustain_enabled: bool,
    /// Loop is enabled
    pub loop_enabled: bool,
}

impl XmEnvelope {
    /// Get interpolated envelope value at a given tick
    pub fn value_at(&self, tick: u16) -> u16 {
        if self.points.is_empty() {
            return 64; // Default max value
        }

        // Find the two points surrounding this tick
        for i in 0..self.points.len() - 1 {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];

            if tick >= x1 && tick < x2 {
                // Linear interpolation
                let dx = (x2 - x1) as f32;
                let dy = y2 as f32 - y1 as f32;
                let t = (tick - x1) as f32 / dx;
                return (y1 as f32 + dy * t) as u16;
            }
        }

        // Past the last point, use the last value
        self.points.last().map(|(_, y)| *y).unwrap_or(64)
    }

    /// Get the tick value at the sustain point
    pub fn sustain_tick(&self) -> Option<u16> {
        if self.sustain_enabled {
            self.points
                .get(self.sustain_point as usize)
                .map(|(x, _)| *x)
        } else {
            None
        }
    }

    /// Get the tick range for the loop
    pub fn loop_range(&self) -> Option<(u16, u16)> {
        if self.loop_enabled {
            let start = self.points.get(self.loop_start as usize)?.0;
            let end = self.points.get(self.loop_end as usize)?.0;
            Some((start, end))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xm_note_methods() {
        let note_off = XmNote {
            note: 97,
            ..Default::default()
        };
        assert!(note_off.is_note_off());
        assert!(!note_off.has_note());

        let note_c4 = XmNote {
            note: 49,
            instrument: 1,
            ..Default::default()
        };
        assert!(!note_c4.is_note_off());
        assert!(note_c4.has_note());
        assert!(note_c4.has_instrument());

        // C-4 is note 49, which is octave 4, semitone 0
        assert_eq!(note_c4.note_to_octave_semitone(), Some((4, 0)));

        let vol_set = XmNote {
            volume: 0x30,
            ..Default::default()
        };
        assert_eq!(vol_set.get_volume(), Some(0x20));

        let no_vol = XmNote {
            volume: 0x00,
            ..Default::default()
        };
        assert_eq!(no_vol.get_volume(), None);
    }

    #[test]
    fn test_xm_pattern_empty() {
        let pattern = XmPattern::empty(64, 8);
        assert_eq!(pattern.num_rows, 64);
        assert_eq!(pattern.notes.len(), 64);
        assert_eq!(pattern.notes[0].len(), 8);
    }

    #[test]
    fn test_xm_instrument_loop() {
        let mut instr = XmInstrument::default();
        assert!(!instr.has_loop());

        instr.sample_loop_type = 1;
        instr.sample_loop_start = 100;
        instr.sample_loop_length = 500;
        assert!(instr.has_loop());
        assert!(!instr.is_pingpong_loop());
        assert_eq!(instr.sample_loop_end(), 600);

        instr.sample_loop_type = 2;
        assert!(instr.is_pingpong_loop());
    }

    #[test]
    fn test_xm_envelope_interpolation() {
        let env = XmEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            enabled: true,
            ..Default::default()
        };

        assert_eq!(env.value_at(0), 64);
        assert_eq!(env.value_at(5), 48); // Midpoint between 64 and 32
        assert_eq!(env.value_at(10), 32);
        assert_eq!(env.value_at(15), 16); // Midpoint between 32 and 0
        assert_eq!(env.value_at(20), 0);
        assert_eq!(env.value_at(30), 0); // Past end
    }
}
