//! Audio loading (sounds and tracker modules).

use anyhow::{Context, Result};
use std::collections::HashSet;
use zx_common::{PackedSound, PackedTracker, TrackerFormat};

use super::utils::detect_tracker_format;

/// Load a sound from a WAV file
pub fn load_sound(id: &str, path: &std::path::Path) -> Result<PackedSound> {
    // Read WAV file and convert to 22050Hz mono i16
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load sound: {}", path.display()))?;

    // Parse WAV header (simplified - assumes 16-bit PCM)
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        anyhow::bail!("Invalid WAV file: {}", path.display());
    }

    // Find data chunk
    let mut offset = 12;
    let mut audio_data = vec![];

    while offset + 8 < data.len() {
        let chunk_id = &data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if chunk_id == b"data" {
            let end = (offset + 8 + chunk_size).min(data.len());
            let samples: Vec<i16> = data[offset + 8..end]
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            audio_data = samples;
            break;
        }

        offset += 8 + chunk_size;
        if !chunk_size.is_multiple_of(2) {
            offset += 1; // Padding byte
        }
    }

    if audio_data.is_empty() {
        anyhow::bail!("No audio data found in WAV file: {}", path.display());
    }

    Ok(PackedSound {
        id: id.to_string(),
        data: audio_data,
    })
}

/// Validate that all non-empty instrument names in a tracker
/// reference loaded sounds in the manifest
pub fn validate_tracker_samples(
    tracker_id: &str,
    tracker_path: &std::path::Path,
    sample_ids: &[String],
    available_sound_ids: &HashSet<String>,
) -> Result<()> {
    // Filter out empty/blank instrument names (intentionally silent)
    let non_empty_samples: Vec<&String> = sample_ids
        .iter()
        .filter(|name| !name.trim().is_empty())
        .collect();

    // Check each sample against available sound IDs
    let mut missing_samples = Vec::new();
    for sample_id in non_empty_samples {
        if !available_sound_ids.contains(sample_id) {
            missing_samples.push(sample_id.clone());
        }
    }

    // If any samples are missing, fail with helpful error
    if !missing_samples.is_empty() {
        let mut available_sounds: Vec<&String> = available_sound_ids.iter().collect();
        available_sounds.sort(); // Sort alphabetically for better readability

        let available_list = if available_sounds.is_empty() {
            "(none - add sounds to [[assets.sounds]] in nether.toml)".to_string()
        } else {
            available_sounds
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        };

        return Err(anyhow::anyhow!(
            "Tracker '{}' ({}) references {} missing sample(s):\n  {}\n\n\
             Available sounds in manifest:\n  {}",
            tracker_id,
            tracker_path.display(),
            missing_samples.len(),
            missing_samples.join("\n  "),
            available_list
        ));
    }

    Ok(())
}

/// Load a tracker module from XM or IT file
///
/// Parses the tracker file, extracts instrument names for sample mapping,
/// and strips embedded sample data (samples are loaded separately via sounds).
pub fn load_tracker(
    id: &str,
    path: &std::path::Path,
    available_sound_ids: &HashSet<String>,
) -> Result<PackedTracker> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load tracker: {}", path.display()))?;

    // Detect format
    let format = detect_tracker_format(&data)
        .ok_or_else(|| anyhow::anyhow!("Unknown tracker format: {}", path.display()))?;

    // Get instrument names and pack based on format
    let (sample_ids, pattern_data) = match format {
        TrackerFormat::Xm => {
            // Get instrument names from XM file (for mapping to sounds)
            let sample_ids = nether_xm::get_instrument_names(&data).with_context(|| {
                format!("Failed to parse XM tracker instruments: {}", path.display())
            })?;

            // Validate sample references against loaded sounds
            validate_tracker_samples(id, path, &sample_ids, available_sound_ids)?;

            // Parse XM and pack to minimal format (removes all overhead)
            let module = nether_xm::parse_xm(&data)
                .with_context(|| format!("Failed to parse XM tracker: {}", path.display()))?;

            let pattern_data = nether_xm::pack_xm_minimal(&module).with_context(|| {
                format!(
                    "Failed to pack XM tracker to minimal format: {}",
                    path.display()
                )
            })?;

            (sample_ids, pattern_data)
        }
        TrackerFormat::It => {
            // Get instrument names from IT file (for mapping to sounds)
            let sample_ids = nether_it::get_instrument_names(&data).with_context(|| {
                format!("Failed to parse IT tracker instruments: {}", path.display())
            })?;

            // Validate sample references against loaded sounds
            validate_tracker_samples(id, path, &sample_ids, available_sound_ids)?;

            // Parse IT and pack to NCIT minimal format (removes all overhead)
            let module = nether_it::parse_it(&data)
                .with_context(|| format!("Failed to parse IT tracker: {}", path.display()))?;

            let pattern_data = nether_it::pack_ncit(&module);

            (sample_ids, pattern_data)
        }
    };

    Ok(PackedTracker {
        id: id.to_string(),
        format,
        pattern_data,
        sample_ids,
    })
}
