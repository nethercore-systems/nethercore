//! WAV loading shared by the exporter and direct cartridge packer.

use anyhow::{ensure, Context, Result};
use std::io::{BufWriter, Cursor, Write};
use std::path::Path;

use crate::formats::{write_nether_sound, SAMPLE_RATE};

/// Convert validated WAV input to .nczsnd format.
pub fn convert_wav(input: &Path, output: &Path) -> Result<()> {
    tracing::info!("Converting WAV: {:?} -> {:?}", input, output);
    let samples = load_wav(input)?;
    let mut writer = BufWriter::new(std::fs::File::create(output)?);
    write_nether_sound(&mut writer, &samples)?;
    writer.flush().context("Failed to finish sound output")?;
    tracing::info!("Wrote {} samples to {:?}", samples.len(), output);
    Ok(())
}

/// Decode complete WAV frames to 22050-Hz mono i16 PCM.
/// Target PCM is preserved exactly; stereo is averaged before resampling.
pub fn load_wav(input: &Path) -> Result<Vec<i16>> {
    let bytes = nethercore_shared::read_file_with_limit(input, nethercore_shared::MAX_ROM_BYTES)
        .context("Failed to read WAV file")?;
    validate_container(&bytes)?;
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).context("Invalid WAV format")?;
    let spec = reader.spec();
    ensure!(
        matches!(spec.channels, 1 | 2),
        "WAV must be mono or stereo; downmix before import"
    );
    ensure!(spec.sample_rate > 0, "WAV sample rate must be positive");
    let samples: Vec<i16> = match spec.sample_format {
        hound::SampleFormat::Int => {
            ensure!(
                matches!(spec.bits_per_sample, 8 | 16 | 24 | 32),
                "Unsupported WAV bit depth: {}",
                spec.bits_per_sample
            );
            reader
                .samples::<i32>()
                .map(|sample| {
                    let sample = sample?;
                    Ok(if spec.bits_per_sample >= 16 {
                        (sample >> (spec.bits_per_sample - 16)) as i16
                    } else {
                        (sample << (16 - spec.bits_per_sample)) as i16
                    })
                })
                .collect::<Result<_>>()?
        }
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| {
                let sample = sample?;
                ensure!(sample.is_finite(), "WAV contains non-finite samples");
                Ok((sample.clamp(-1.0, 1.0) * 32767.0) as i16)
            })
            .collect::<Result<_>>()?,
    };
    ensure!(!samples.is_empty(), "WAV contains no audio samples");
    ensure!(
        samples.len().is_multiple_of(usize::from(spec.channels)),
        "Incomplete WAV frame"
    );
    let samples = if spec.channels == 2 {
        samples
            .chunks_exact(2)
            .map(|frame| ((i32::from(frame[0]) + i32::from(frame[1])) / 2) as i16)
            .collect()
    } else {
        samples
    };
    if spec.sample_rate == SAMPLE_RATE {
        Ok(samples)
    } else {
        resample(&samples, spec.sample_rate, SAMPLE_RATE)
    }
}

// hound validates decoding, but a partial frame or mismatched RIFF/chunk extent
// must fail before conversion can truncate it or treat it as another layout.
fn validate_container(bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE",
        "Invalid WAV RIFF header"
    );
    let declared = u32::from_le_bytes(bytes[4..8].try_into()?) as usize;
    ensure!(
        declared.checked_add(8) == Some(bytes.len()),
        "WAV RIFF length mismatch or truncated file"
    );
    let (mut position, mut align, mut data_size) = (12usize, None, None);
    while position < bytes.len() {
        ensure!(bytes.len() - position >= 8, "Truncated WAV chunk header");
        let size = u32::from_le_bytes(bytes[position + 4..position + 8].try_into()?) as usize;
        let start = position + 8;
        let end = start.checked_add(size).context("WAV chunk size overflow")?;
        ensure!(end <= bytes.len(), "Truncated WAV chunk");
        match &bytes[position..position + 4] {
            b"fmt " => {
                ensure!(
                    align.is_none() && size >= 16,
                    "Invalid or repeated WAV format chunk"
                );
                align =
                    Some(u16::from_le_bytes(bytes[start + 12..start + 14].try_into()?) as usize);
            }
            b"data" => {
                ensure!(
                    data_size.is_none(),
                    "Multiple WAV data chunks are unsupported"
                );
                data_size = Some(size);
            }
            _ => {}
        }
        position = end.checked_add(size % 2).context("WAV padding overflow")?;
        ensure!(position <= bytes.len(), "Missing WAV chunk padding");
    }
    let align = align.context("Missing WAV format chunk")?;
    let size = data_size.context("Missing WAV data chunk")?;
    ensure!(
        align > 0 && size.is_multiple_of(align),
        "Incomplete WAV audio frame"
    );
    Ok(())
}

/// Linear resampling; retain the existing converter's interpolation contract.
fn resample(samples: &[i16], src_rate: u32, dst_rate: u32) -> Result<Vec<i16>> {
    let ratio = src_rate as f64 / dst_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    ensure!(
        output_len > 0 && output_len <= nethercore_shared::MAX_ROM_BYTES as usize / 2,
        "Decoded WAV exceeds the loader size limit or contains no complete target frames"
    );
    let mut output = Vec::new();
    output
        .try_reserve_exact(output_len)
        .context("WAV conversion allocation failed")?;
    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;
        let sample = if src_idx + 1 < samples.len() {
            let a = samples[src_idx] as f64;
            let b = samples[src_idx + 1] as f64;
            (a + (b - a) * frac) as i16
        } else {
            samples[src_idx.min(samples.len() - 1)]
        };
        output.push(sample);
    }
    Ok(output)
}
