//! Binary replay format writer
//!
//! Writes .ncrp replay files with optional compression.

use crate::replay::types::*;
use byteorder::{LittleEndian, WriteBytesExt};
use lz4_flex::compress_prepend_size;
use std::io::{self, Write};

/// Writer for binary replay format
pub struct BinaryWriter<W: Write> {
    writer: W,
}

impl<W: Write> BinaryWriter<W> {
    /// Create a new binary writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a complete replay to the output
    pub fn write_replay(&mut self, replay: &Replay) -> io::Result<()> {
        // Write header
        self.write_header(&replay.header)?;

        // Write input stream
        self.write_inputs(&replay.inputs, &replay.header)?;

        // Write checkpoints (if any)
        if replay.header.flags.contains(ReplayFlags::HAS_CHECKPOINTS) {
            self.write_checkpoints(&replay.checkpoints)?;
        }

        // Write assertions (if any)
        if replay.header.flags.contains(ReplayFlags::HAS_ASSERTIONS) {
            let json = serde_json::to_vec(&replay.assertions)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.writer.write_u32::<LittleEndian>(json.len() as u32)?;
            self.writer.write_all(&json)?;
        }

        Ok(())
    }

    /// Write the 24-byte header
    fn write_header(&mut self, header: &ReplayHeader) -> io::Result<()> {
        self.writer.write_u8(header.console_id)?;
        self.writer.write_u8(header.player_count)?;
        self.writer.write_u8(header.input_size)?;
        self.writer.write_u8(header.flags.bits())?;
        self.writer.write_all(&header.reserved)?; // 4 reserved bytes
        self.writer.write_u64::<LittleEndian>(header.seed)?;
        self.writer.write_u64::<LittleEndian>(header.frame_count)?;
        Ok(())
    }

    /// Write the input stream
    fn write_inputs(&mut self, inputs: &InputSequence, header: &ReplayHeader) -> io::Result<()> {
        let frame_count = inputs.frame_count();
        self.writer.write_u64::<LittleEndian>(frame_count)?;

        if header.flags.contains(ReplayFlags::COMPRESSED_INPUTS) {
            // Delta compression: store first frame raw, then XOR deltas
            let mut prev_frame: Vec<Vec<u8>> = vec![
                vec![0u8; header.input_size as usize];
                header.player_count as usize
            ];

            let mut delta_buffer = Vec::new();

            for frame_inputs in inputs.iter() {
                for (player_idx, input) in frame_inputs.iter().enumerate() {
                    if player_idx >= prev_frame.len() {
                        continue;
                    }
                    for (byte_idx, &byte) in input.iter().enumerate() {
                        if byte_idx < prev_frame[player_idx].len() {
                            let delta = byte ^ prev_frame[player_idx][byte_idx];
                            delta_buffer.push(delta);
                            prev_frame[player_idx][byte_idx] = byte;
                        }
                    }
                }
            }

            // Compress the delta buffer with LZ4
            let compressed = compress_prepend_size(&delta_buffer);
            self.writer.write_u32::<LittleEndian>(compressed.len() as u32)?;
            self.writer.write_all(&compressed)?;
        } else {
            // Uncompressed: write raw input bytes
            for frame_inputs in inputs.iter() {
                for player_input in frame_inputs {
                    self.writer.write_all(player_input)?;
                }
            }
        }

        Ok(())
    }

    /// Write checkpoints
    fn write_checkpoints(&mut self, checkpoints: &[Checkpoint]) -> io::Result<()> {
        self.writer.write_u32::<LittleEndian>(checkpoints.len() as u32)?;

        for checkpoint in checkpoints {
            self.writer.write_u64::<LittleEndian>(checkpoint.frame)?;
            let compressed = compress_prepend_size(&checkpoint.state);
            self.writer.write_u32::<LittleEndian>(compressed.len() as u32)?;
            self.writer.write_all(&compressed)?;
        }

        Ok(())
    }

    /// Consume the writer and return the inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_header() {
        let mut buffer = Vec::new();
        let mut writer = BinaryWriter::new(&mut buffer);

        let header = ReplayHeader {
            console_id: 1,
            player_count: 2,
            input_size: 8,
            flags: ReplayFlags::COMPRESSED_INPUTS,
            reserved: [0, 0, 0, 0],
            seed: 12345,
            frame_count: 100,
        };

        writer.write_header(&header).unwrap();

        // Header should be 24 bytes
        assert_eq!(buffer.len(), 24);

        // Check values
        assert_eq!(buffer[0], 1); // console_id
        assert_eq!(buffer[1], 2); // player_count
        assert_eq!(buffer[2], 8); // input_size
        assert_eq!(buffer[3], 0b010); // flags (COMPRESSED_INPUTS)
    }

    #[test]
    fn test_write_empty_replay() {
        let mut buffer = Vec::new();
        let mut writer = BinaryWriter::new(&mut buffer);

        let replay = Replay::default();
        writer.write_replay(&replay).unwrap();

        // Should have header (24) + frame count (8) = 32 bytes minimum
        assert!(buffer.len() >= 32);
    }
}
