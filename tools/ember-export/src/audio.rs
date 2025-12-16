//! Audio converter (WAV -> .embersnd)

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::formats::{write_ember_sound, SAMPLE_RATE};

/// Convert a WAV file to EmberSound format
pub fn convert_wav(input: &Path, output: &Path) -> Result<()> {
    // Load WAV
    let mut reader = hound::WavReader::open(input)
        .with_context(|| format!("Failed to load WAV: {:?}", input))?;

    let spec = reader.spec();

    // Read samples
    let samples: Vec<i16> = match spec.sample_format {
        hound::SampleFormat::Int => match spec.bits_per_sample {
            16 => reader.samples::<i16>().map(|s| s.unwrap()).collect(),
            8 => reader
                .samples::<i8>()
                .map(|s| (s.unwrap() as i16) << 8)
                .collect(),
            24 | 32 => reader
                .samples::<i32>()
                .map(|s| (s.unwrap() >> (spec.bits_per_sample - 16)) as i16)
                .collect(),
            _ => bail!("Unsupported bit depth: {}", spec.bits_per_sample),
        },
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| (s.unwrap() * 32767.0) as i16)
            .collect(),
    };

    // Convert to mono if stereo
    let mono_samples: Vec<i16> = if spec.channels == 2 {
        samples
            .chunks(2)
            .map(|chunk| ((chunk[0] as i32 + chunk[1] as i32) / 2) as i16)
            .collect()
    } else if spec.channels == 1 {
        samples
    } else {
        bail!("Unsupported channel count: {}", spec.channels);
    };

    // Resample if needed
    let resampled = if spec.sample_rate != SAMPLE_RATE {
        resample(&mono_samples, spec.sample_rate, SAMPLE_RATE)
    } else {
        mono_samples
    };

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_ember_sound(&mut writer, &resampled)?;

    // QOA achieves ~5:1 compression (3.2 bits/sample vs 16 bits/sample)
    let pcm_size = resampled.len() * 2;
    let qoa_frame_size = ember_qoa::encode_qoa(&resampled).len();
    let total_size = z_common::formats::EmberZSoundHeader::SIZE + qoa_frame_size;

    tracing::info!(
        "Converted audio: {} samples ({}Hz -> {}Hz), {} bytes ({:.1}:1 vs PCM)",
        resampled.len(),
        spec.sample_rate,
        SAMPLE_RATE,
        total_size,
        pcm_size as f64 / total_size as f64
    );

    Ok(())
}

/// Simple linear resampling
fn resample(samples: &[i16], src_rate: u32, dst_rate: u32) -> Vec<i16> {
    let ratio = src_rate as f64 / dst_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

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

    output
}
