//! Sample extraction from XM and IT files
//!
//! Converts tracker module samples to the Sound type used by TrackerEngine.

use anyhow::{Context, Result};
use nethercore_zx::audio::Sound;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;

/// Load samples from an XM file
pub fn load_xm_samples(data: &[u8]) -> Result<Vec<Option<Sound>>> {
    let extracted = nether_xm::extract_samples(data).context("Failed to extract XM samples")?;

    // Debug: show which instruments have samples
    println!("Extracted {} samples:", extracted.len());
    for (i, sample) in extracted.iter().enumerate() {
        println!(
            "  Sample {}: instrument_index={}, name='{}', {} samples",
            i,
            sample.instrument_index,
            sample.name,
            sample.data.len()
        );
    }

    // XM instruments are 1-indexed in the module, so sample 0 is unused
    // The sound_handles array should have index 0 = None, then samples
    let mut sounds = vec![None]; // Index 0 is unused

    for sample in extracted {
        if sample.data.is_empty() {
            sounds.push(None);
        } else {
            sounds.push(Some(Sound {
                data: Arc::new(sample.data),
            }));
        }
    }

    Ok(sounds)
}

/// Load samples from an IT file
pub fn load_it_samples(data: &[u8], module: &nether_it::ItModule) -> Result<Vec<Option<Sound>>> {
    // IT samples are 1-indexed, so index 0 is unused
    let mut sounds = vec![None];

    // Re-parse to get sample offsets (not exposed by parse_it)
    let sample_offsets = extract_it_sample_offsets(data)?;

    for (idx, sample) in module.samples.iter().enumerate() {
        let offset = sample_offsets.get(idx).copied().unwrap_or(0);

        if offset == 0 || sample.length == 0 {
            sounds.push(None);
            continue;
        }

        match nether_it::load_sample_data(data, offset, sample) {
            Ok(sample_data) => {
                let pcm_data = convert_sample_data_to_i16(sample_data);
                if pcm_data.is_empty() {
                    sounds.push(None);
                } else {
                    sounds.push(Some(Sound {
                        data: Arc::new(pcm_data),
                    }));
                }
            }
            Err(_) => {
                sounds.push(None);
            }
        }
    }

    Ok(sounds)
}

/// Extract sample data offsets from IT file header
fn extract_it_sample_offsets(data: &[u8]) -> Result<Vec<u32>> {
    if data.len() < 192 {
        anyhow::bail!("IT file too small");
    }

    let mut cursor = Cursor::new(data);

    // Skip to OrdNum (offset 0x20)
    cursor.seek(SeekFrom::Start(0x20))?;
    let num_orders = read_u16(&mut cursor)?;
    let _num_instruments = read_u16(&mut cursor)?;
    let num_samples = read_u16(&mut cursor)?;

    // Calculate offset to sample pointer table
    // Header ends at 0xC0 (192 bytes)
    // Then: order table (OrdNum bytes)
    // Then: instrument pointers (InsNum * 4 bytes)
    // Then: sample pointers (SmpNum * 4 bytes)
    let order_table_offset = 0xC0;
    let instrument_table_offset = order_table_offset + num_orders as u64;
    let sample_table_offset = instrument_table_offset + (_num_instruments as u64 * 4);

    cursor.seek(SeekFrom::Start(sample_table_offset))?;

    let mut offsets = Vec::with_capacity(num_samples as usize);
    for _ in 0..num_samples {
        offsets.push(read_u32(&mut cursor)?);
    }

    Ok(offsets)
}

/// Convert IT SampleData to Vec<i16>
fn convert_sample_data_to_i16(sample_data: nether_it::SampleData) -> Vec<i16> {
    match sample_data {
        nether_it::SampleData::I8(data) => {
            // Convert 8-bit to 16-bit by scaling
            data.iter().map(|&s| (s as i16) * 256).collect()
        }
        nether_it::SampleData::I16(data) => data,
    }
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
