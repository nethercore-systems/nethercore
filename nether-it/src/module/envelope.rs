//! IT envelope structures and flags

/// IT envelope
#[derive(Debug, Clone)]
pub struct ItEnvelope {
    /// Envelope points: (tick, value)
    /// For volume: value 0-64
    /// For panning: value -32 to +32 (stored as 0-64, 32 = center)
    /// For pitch: value -32 to +32 (half-semitones)
    pub points: Vec<(u16, i8)>,
    /// Loop begin point index
    pub loop_begin: u8,
    /// Loop end point index
    pub loop_end: u8,
    /// Sustain loop begin point index
    pub sustain_begin: u8,
    /// Sustain loop end point index
    pub sustain_end: u8,
    /// Envelope flags
    pub flags: ItEnvelopeFlags,
}

impl Default for ItEnvelope {
    fn default() -> Self {
        Self {
            points: vec![(0, 64), (100, 64)], // Flat envelope at max
            loop_begin: 0,
            loop_end: 0,
            sustain_begin: 0,
            sustain_end: 0,
            flags: ItEnvelopeFlags::empty(),
        }
    }
}

impl ItEnvelope {
    /// Check if envelope is enabled
    pub fn is_enabled(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::ENABLED)
    }

    /// Check if envelope has loop
    pub fn has_loop(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::LOOP)
    }

    /// Check if envelope has sustain loop
    pub fn has_sustain(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::SUSTAIN_LOOP)
    }

    /// Check if this is a filter envelope (for pitch envelope type)
    pub fn is_filter(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::FILTER)
    }

    /// Get interpolated envelope value at a given tick
    pub fn value_at(&self, tick: u16) -> i8 {
        if self.points.is_empty() {
            return 64; // Default max value
        }

        // Find the two points surrounding this tick
        for i in 0..self.points.len().saturating_sub(1) {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];

            if tick >= x1 && tick < x2 {
                // Linear interpolation
                if x2 == x1 {
                    return y1;
                }
                let dx = (x2 - x1) as f32;
                let dy = y2 as f32 - y1 as f32;
                let t = (tick - x1) as f32 / dx;
                return (y1 as f32 + dy * t) as i8;
            }
        }

        // Past the last point, use the last value
        self.points.last().map(|(_, y)| *y).unwrap_or(64)
    }
}

/// IT envelope flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItEnvelopeFlags(u8);

impl ItEnvelopeFlags {
    /// Envelope is enabled
    pub const ENABLED: Self = Self(0x01);
    /// Loop is enabled
    pub const LOOP: Self = Self(0x02);
    /// Sustain loop is enabled
    pub const SUSTAIN_LOOP: Self = Self(0x04);
    /// Carry envelope (continue from previous note)
    pub const CARRY: Self = Self(0x08);
    /// Filter envelope (for pitch envelope type only)
    pub const FILTER: Self = Self(0x80);

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

impl std::ops::BitOr for ItEnvelopeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_envelope_interpolation() {
        let env = ItEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            flags: ItEnvelopeFlags::ENABLED,
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
