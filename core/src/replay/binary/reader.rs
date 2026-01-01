//! Binary replay format reader
//!
//! Reads .ncrp replay files with automatic decompression.

use crate::replay::types::*;
use byteorder::{LittleEndian, ReadBytesExt};
use lz4_flex::decompress_size_prepended;
use std::io::{self, Read};

/// Reader for binary replay format
pub struct BinaryReader<R: Read> {
    reader: R,
}

impl<R: Read> BinaryReader<R> {
    /// Create a new binary reader
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read a complete replay from the input
    pub fn read_replay(&mut self) -> io::Result<Replay> {
        // Read header
        let header = self.read_header()?;

        // Read inputs
        let inputs = self.read_inputs(&header)?;

        // Read checkpoints
        let checkpoints = if header.flags.contains(ReplayFlags::HAS_CHECKPOINTS) {
            self.read_checkpoints()?
        } else {
            Vec::new()
        };

        // Read assertions
        let assertions = if header.flags.contains(ReplayFlags::HAS_ASSERTIONS) {
            let len = self.reader.read_u32::<LittleEndian>()? as usize;
            let mut json = vec![0u8; len];
            self.reader.read_exact(&mut json)?;
            serde_json::from_slice(&json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            Vec::new()
        };

        Ok(Replay {
            header,
            inputs,
            checkpoints,
            assertions,
        })
    }

    /// Read the 24-byte header
    fn read_header(&mut self) -> io::Result<ReplayHeader> {
        let console_id = self.reader.read_u8()?;
        let player_count = self.reader.read_u8()?;
        let input_size = self.reader.read_u8()?;
        let flags = ReplayFlags::from_bits_truncate(self.reader.read_u8()?);

        let mut reserved = [0u8; 4];
        self.reader.read_exact(&mut reserved)?;

        let seed = self.reader.read_u64::<LittleEndian>()?;
        let frame_count = self.reader.read_u64::<LittleEndian>()?;

        Ok(ReplayHeader {
            console_id,
            player_count,
            input_size,
            flags,
            reserved,
            seed,
            frame_count,
        })
    }

    /// Read the input stream
    fn read_inputs(&mut self, header: &ReplayHeader) -> io::Result<InputSequence> {
        let frame_count = self.reader.read_u64::<LittleEndian>()?;
        let mut inputs = InputSequence::new();

        let player_count = header.player_count as usize;
        let input_size = header.input_size as usize;

        if header.flags.contains(ReplayFlags::COMPRESSED_INPUTS) {
            // Read compressed delta buffer
            let compressed_len = self.reader.read_u32::<LittleEndian>()? as usize;
            let mut compressed = vec![0u8; compressed_len];
            self.reader.read_exact(&mut compressed)?;

            let delta_buffer = decompress_size_prepended(&compressed)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

            // Reconstruct frames from deltas
            let mut prev_frame: Vec<Vec<u8>> = vec![vec![0u8; input_size]; player_count];
            let mut offset = 0;

            for _ in 0..frame_count {
                let mut frame_inputs = Vec::with_capacity(player_count);
                for player_idx in 0..player_count {
                    let mut input = vec![0u8; input_size];
                    for byte_idx in 0..input_size {
                        if offset >= delta_buffer.len() {
                            return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "Delta buffer too short",
                            ));
                        }
                        let delta = delta_buffer[offset];
                        offset += 1;
                        input[byte_idx] = prev_frame[player_idx][byte_idx] ^ delta;
                        prev_frame[player_idx][byte_idx] = input[byte_idx];
                    }
                    frame_inputs.push(input);
                }
                inputs.push_frame(frame_inputs);
            }
        } else {
            // Read raw input bytes
            for _ in 0..frame_count {
                let mut frame_inputs = Vec::with_capacity(player_count);
                for _ in 0..player_count {
                    let mut input = vec![0u8; input_size];
                    self.reader.read_exact(&mut input)?;
                    frame_inputs.push(input);
                }
                inputs.push_frame(frame_inputs);
            }
        }

        Ok(inputs)
    }

    /// Read checkpoints
    fn read_checkpoints(&mut self) -> io::Result<Vec<Checkpoint>> {
        let count = self.reader.read_u32::<LittleEndian>()? as usize;
        let mut checkpoints = Vec::with_capacity(count);

        for _ in 0..count {
            let frame = self.reader.read_u64::<LittleEndian>()?;
            let compressed_len = self.reader.read_u32::<LittleEndian>()? as usize;
            let mut compressed = vec![0u8; compressed_len];
            self.reader.read_exact(&mut compressed)?;

            let state = decompress_size_prepended(&compressed)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

            checkpoints.push(Checkpoint { frame, state });
        }

        Ok(checkpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::binary::writer::BinaryWriter;

    #[test]
    fn test_roundtrip_empty() {
        let replay = Replay::default();

        // Write
        let mut buffer = Vec::new();
        BinaryWriter::new(&mut buffer).write_replay(&replay).unwrap();

        // Read
        let parsed = BinaryReader::new(buffer.as_slice()).read_replay().unwrap();

        assert_eq!(parsed.header.console_id, replay.header.console_id);
        assert_eq!(parsed.header.player_count, replay.header.player_count);
        assert_eq!(parsed.inputs.frame_count(), 0);
    }

    #[test]
    fn test_roundtrip_with_inputs() {
        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![vec![0x0F, 0x00]]);
        inputs.push_frame(vec![vec![0x1F, 0x01]]);
        inputs.push_frame(vec![vec![0x0F, 0x00]]);

        let replay = Replay {
            header: ReplayHeader {
                console_id: 1,
                player_count: 1,
                input_size: 2,
                flags: ReplayFlags::empty(),
                reserved: [0; 4],
                seed: 12345,
                frame_count: 3,
            },
            inputs,
            checkpoints: Vec::new(),
            assertions: Vec::new(),
        };

        // Write
        let mut buffer = Vec::new();
        BinaryWriter::new(&mut buffer).write_replay(&replay).unwrap();

        // Read
        let parsed = BinaryReader::new(buffer.as_slice()).read_replay().unwrap();

        assert_eq!(parsed.inputs.frame_count(), 3);
        assert_eq!(parsed.inputs.get_frame(0), Some(&vec![vec![0x0F, 0x00]]));
        assert_eq!(parsed.inputs.get_frame(1), Some(&vec![vec![0x1F, 0x01]]));
    }

    #[test]
    fn test_roundtrip_compressed() {
        let mut inputs = InputSequence::new();
        // Add 100 frames of mostly-idle input (compresses well)
        for i in 0..100 {
            let input = if i % 10 == 0 { 0x01 } else { 0x00 };
            inputs.push_frame(vec![vec![input, 0x00]]);
        }

        let replay = Replay {
            header: ReplayHeader {
                console_id: 1,
                player_count: 1,
                input_size: 2,
                flags: ReplayFlags::COMPRESSED_INPUTS,
                reserved: [0; 4],
                seed: 0,
                frame_count: 100,
            },
            inputs,
            checkpoints: Vec::new(),
            assertions: Vec::new(),
        };

        // Write
        let mut buffer = Vec::new();
        BinaryWriter::new(&mut buffer).write_replay(&replay).unwrap();

        // Should be smaller than uncompressed (100 frames * 2 bytes = 200 bytes)
        // Compressed should be much smaller due to repetition
        assert!(buffer.len() < 200 + 32); // header + frame count + compressed data

        // Read
        let parsed = BinaryReader::new(buffer.as_slice()).read_replay().unwrap();

        assert_eq!(parsed.inputs.frame_count(), 100);
        assert_eq!(parsed.inputs.get_frame(0), Some(&vec![vec![0x01, 0x00]]));
        assert_eq!(parsed.inputs.get_frame(1), Some(&vec![vec![0x00, 0x00]]));
        assert_eq!(parsed.inputs.get_frame(10), Some(&vec![vec![0x01, 0x00]]));
    }
}
