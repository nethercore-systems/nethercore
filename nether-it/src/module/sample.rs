//! IT sample structures and flags

/// IT sample metadata
#[derive(Debug, Clone)]
pub struct ItSample {
    /// Sample name (max 26 chars)
    pub name: String,
    /// DOS filename (max 12 chars)
    pub filename: String,
    /// Global volume (0-64)
    pub global_volume: u8,
    /// Sample flags
    pub flags: ItSampleFlags,
    /// Default volume (0-64)
    pub default_volume: u8,
    /// Default panning (0-64), None if not enabled
    pub default_pan: Option<u8>,
    /// Sample length in samples (not bytes)
    pub length: u32,
    /// Loop begin (in samples)
    pub loop_begin: u32,
    /// Loop end (in samples)
    pub loop_end: u32,
    /// C5 speed (sample rate for C-5)
    pub c5_speed: u32,
    /// Sustain loop begin
    pub sustain_loop_begin: u32,
    /// Sustain loop end
    pub sustain_loop_end: u32,
    /// Auto-vibrato speed (0-64)
    pub vibrato_speed: u8,
    /// Auto-vibrato depth (0-64)
    pub vibrato_depth: u8,
    /// Auto-vibrato rate (0-64)
    pub vibrato_rate: u8,
    /// Auto-vibrato waveform (0=sine, 1=ramp down, 2=square, 3=random)
    pub vibrato_type: u8,
}

impl Default for ItSample {
    fn default() -> Self {
        Self {
            name: String::new(),
            filename: String::new(),
            global_volume: 64,
            flags: ItSampleFlags::empty(),
            default_volume: 64,
            default_pan: None,
            length: 0,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 8363, // Default Amiga sample rate
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            vibrato_type: 0,
        }
    }
}

impl ItSample {
    /// Check if sample has loop enabled
    pub fn has_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::LOOP)
    }

    /// Check if sample has sustain loop enabled
    pub fn has_sustain_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::SUSTAIN_LOOP)
    }

    /// Check if loop is ping-pong (bidirectional)
    pub fn is_pingpong_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::PINGPONG_LOOP)
    }

    /// Check if sustain loop is ping-pong
    pub fn is_pingpong_sustain(&self) -> bool {
        self.flags.contains(ItSampleFlags::PINGPONG_SUSTAIN)
    }

    /// Check if sample is 16-bit
    pub fn is_16bit(&self) -> bool {
        self.flags.contains(ItSampleFlags::SAMPLE_16BIT)
    }

    /// Check if sample is stereo
    pub fn is_stereo(&self) -> bool {
        self.flags.contains(ItSampleFlags::STEREO)
    }

    /// Check if sample data is compressed
    pub fn is_compressed(&self) -> bool {
        self.flags.contains(ItSampleFlags::COMPRESSED)
    }
}

/// IT sample flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItSampleFlags(u8);

impl ItSampleFlags {
    /// Sample data exists
    pub const HAS_DATA: Self = Self(0x01);
    /// 16-bit sample (vs 8-bit)
    pub const SAMPLE_16BIT: Self = Self(0x02);
    /// Stereo sample
    pub const STEREO: Self = Self(0x04);
    /// Compressed sample (IT215)
    pub const COMPRESSED: Self = Self(0x08);
    /// Loop enabled
    pub const LOOP: Self = Self(0x10);
    /// Sustain loop enabled
    pub const SUSTAIN_LOOP: Self = Self(0x20);
    /// Ping-pong loop
    pub const PINGPONG_LOOP: Self = Self(0x40);
    /// Ping-pong sustain loop
    pub const PINGPONG_SUSTAIN: Self = Self(0x80);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ItSampleFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_sample_flags() {
        let mut sample = ItSample::default();
        assert!(!sample.has_loop());

        sample.flags = ItSampleFlags::HAS_DATA | ItSampleFlags::LOOP | ItSampleFlags::SAMPLE_16BIT;
        assert!(sample.has_loop());
        assert!(sample.is_16bit());
        assert!(!sample.is_stereo());
        assert!(!sample.is_pingpong_loop());
    }
}
