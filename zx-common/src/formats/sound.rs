//! EmberZSound binary format (.ewzsnd)
//!
//! ZX console audio format. QOA compressed.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: total_samples u32 LE
//! 0x04: flags u8
//! 0x05: reserved (3 bytes)
//! 0x08: QOA frame data
//! ```
//!
//! # Flags
//! - Bit 0: Stereo (0 = mono, 1 = stereo) - reserved for future music support
//!
//! Sample rate is fixed at 22050Hz (controlled by ember-export).

/// ZX console sample rate (fixed)
pub const SAMPLE_RATE: u32 = 22050;

/// Sound flags
pub mod sound_flags {
    /// Stereo audio (reserved for future use)
    pub const STEREO: u8 = 0b0000_0001;
}

/// EmberZSound header (8 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EmberZSoundHeader {
    pub total_samples: u32,
    pub flags: u8,
    pub _reserved: [u8; 3],
}

impl EmberZSoundHeader {
    pub const SIZE: usize = 8;

    pub fn new(total_samples: u32) -> Self {
        Self {
            total_samples,
            flags: 0,
            _reserved: [0; 3],
        }
    }

    /// Create header with flags
    pub fn with_flags(total_samples: u32, flags: u8) -> Self {
        Self {
            total_samples,
            flags,
            _reserved: [0; 3],
        }
    }

    /// Check if stereo
    pub fn is_stereo(&self) -> bool {
        self.flags & sound_flags::STEREO != 0
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.total_samples.to_le_bytes());
        bytes[4] = self.flags;
        // _reserved bytes stay 0
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            total_samples: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            flags: bytes[4],
            _reserved: [0; 3],
        })
    }
}

/// Decode EmberZSound data to PCM samples
///
/// Returns decoded PCM samples (mono, 16-bit)
pub fn decode_sound(data: &[u8]) -> Option<Vec<i16>> {
    let header = EmberZSoundHeader::from_bytes(data)?;
    let qoa_data = &data[EmberZSoundHeader::SIZE..];

    ember_qoa::decode_qoa(qoa_data, header.total_samples as usize).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create EmberZSound data from PCM samples
    fn encode_sound(samples: &[i16]) -> Vec<u8> {
        let header = EmberZSoundHeader::new(samples.len() as u32);
        let qoa_data = ember_qoa::encode_qoa(samples);

        let mut data = Vec::with_capacity(EmberZSoundHeader::SIZE + qoa_data.len());
        data.extend_from_slice(&header.to_bytes());
        data.extend_from_slice(&qoa_data);
        data
    }

    #[test]
    fn test_decode_qoa() {
        let original: Vec<i16> = (0..1000).map(|i| (i as i16).wrapping_mul(10)).collect();
        let sound_data = encode_sound(&original);

        let samples = decode_sound(&sound_data).unwrap();
        assert_eq!(samples.len(), original.len());
    }

    #[test]
    fn test_decode_empty() {
        let result = decode_sound(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_too_small() {
        let result = decode_sound(&[0, 0, 0]);
        assert!(result.is_none());
    }

    #[test]
    fn test_header_roundtrip() {
        let header = EmberZSoundHeader::new(12345);
        let bytes = header.to_bytes();
        let decoded = EmberZSoundHeader::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.total_samples, 12345);
        assert_eq!(decoded.flags, 0);
    }

    #[test]
    fn test_header_size() {
        assert_eq!(EmberZSoundHeader::SIZE, 8);
    }

    #[test]
    fn test_header_with_flags() {
        let header = EmberZSoundHeader::with_flags(1000, sound_flags::STEREO);
        assert!(header.is_stereo());

        let bytes = header.to_bytes();
        let decoded = EmberZSoundHeader::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.total_samples, 1000);
        assert!(decoded.is_stereo());
    }
}
