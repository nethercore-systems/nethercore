//! Diagnostic: preserve an IT song but replace its samples with decoded PCM.
//! Compare original and output in a reference player to isolate importer errors.
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err("usage: decode_to_pcm INPUT.it OUTPUT.it".into());
    }
    let mut data = fs::read(&args[1])?;
    let samples = nether_it::extract_samples(&data)?;
    let orders = u16::from_le_bytes([data[32], data[33]]) as usize;
    let instruments = u16::from_le_bytes([data[34], data[35]]) as usize;
    for sample in samples {
        let slot = 192 + orders + (instruments + sample.sample_index as usize) * 4;
        let header = u32::from_le_bytes(data[slot..slot + 4].try_into()?) as usize;
        let frames = sample.data.len() / if sample.is_stereo { 2 } else { 1 };
        let offset = u32::try_from(data.len())?;
        data[header + 18] = (data[header + 18] & !8) | 2; // Uncompressed 16-bit
        data[header + 46] = 1; // Signed, little endian
        data[header + 48..header + 52].copy_from_slice(&u32::try_from(frames)?.to_le_bytes());
        data[header + 72..header + 76].copy_from_slice(&offset.to_le_bytes());
        // IT stereo is planar; extract_samples returns interleaved PCM.
        let channels = if sample.is_stereo { 2 } else { 1 };
        for channel in 0..channels {
            for value in sample.data.iter().skip(channel).step_by(channels) {
                data.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    fs::write(&args[2], data)?;
    Ok(())
}
