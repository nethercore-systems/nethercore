//! Asset ingestion and packing helpers.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use zx_common::{PackedSound, TextureFormat, ZXDataPack};

use crate::manifest::AssetsSection;

// Sub-modules
pub mod animation;
pub mod audio;
pub mod data;
pub mod mesh;
pub mod skeleton;
pub mod texture;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-export commonly used functions for backwards compatibility
pub use animation::load_keyframes;
pub use audio::{load_sound, load_tracker};
pub use data::load_data;
pub use mesh::load_mesh;
pub use skeleton::load_skeleton;
pub use texture::load_texture;
pub use utils::{detect_tracker_format, hash_sample_data, require_id, sanitize_name};

/// Expanded keyframe entry (after wildcard resolution)
struct ExpandedKeyframeEntry {
    id: String,
    path: String,
    animation_name: Option<String>,
    skin_name: Option<String>,
}

/// Load assets from disk into a data pack (parallel)
pub fn load_assets(
    project_dir: &std::path::Path,
    assets: &AssetsSection,
    texture_format: TextureFormat,
) -> Result<ZXDataPack> {
    use rayon::prelude::*;

    // Load all asset types in parallel
    // Textures are the most expensive (BC7 compression), so they benefit most from parallelism

    // Load textures in parallel
    let textures: Result<Vec<_>> = assets
        .textures
        .par_iter()
        .map(|entry| {
            let id = require_id(entry, "Texture")?;
            let path = project_dir.join(&entry.path);
            load_texture(id, &path, texture_format)
        })
        .collect();
    let textures = textures?;

    // Load meshes in parallel
    let meshes: Result<Vec<_>> = assets
        .meshes
        .par_iter()
        .map(|entry| {
            let id = require_id(entry, "Mesh")?;
            let path = project_dir.join(&entry.path);
            load_mesh(id, &path)
        })
        .collect();
    let meshes = meshes?;

    // Load skeletons in parallel
    let skeletons: Result<Vec<_>> = assets
        .skeletons
        .par_iter()
        .map(|entry| {
            let id = require_id(entry, "Skeleton")?;
            let path = project_dir.join(&entry.path);
            load_skeleton(id, &path, entry.skin_name.as_deref())
        })
        .collect();
    let skeletons = skeletons?;

    // Load keyframes in parallel (support both 'keyframes' and 'animations' keys)
    // Combine both 'keyframes' and 'animations' entries
    let all_keyframe_entries: Vec<_> = assets
        .keyframes
        .iter()
        .chain(assets.animations.iter())
        .collect();

    // Expand wildcard animation entries before parallel loading
    let mut expanded_keyframe_entries: Vec<ExpandedKeyframeEntry> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    // First pass: collect explicit IDs to detect collisions
    for entry in all_keyframe_entries.iter() {
        if let Some(id) = &entry.id {
            seen_ids.insert(id.clone());
        }
    }

    // Second pass: expand wildcards and check for collisions
    for entry in all_keyframe_entries.iter() {
        let is_wildcard = entry.id.is_none() && entry.animation_name.is_none();

        if is_wildcard {
            // Wildcard import - list all animations from GLB
            let path = project_dir.join(&entry.path);
            let anim_list = nether_export::get_animation_list(&path)
                .with_context(|| format!("Failed to list animations in: {}", path.display()))?;

            if anim_list.is_empty() {
                println!(
                    "  Warning: No animations found in {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                continue;
            }

            let prefix = entry.id_prefix.as_deref().unwrap_or("");

            for anim_info in anim_list {
                let id = format!("{}{}", prefix, anim_info.name);

                // Check for collision
                if seen_ids.contains(&id) {
                    return Err(anyhow::anyhow!(
                        "Animation ID collision: '{}' from '{}' conflicts with existing ID.\n\
                         Hint: Use id_prefix to namespace animations from different files.",
                        id,
                        entry.path
                    ));
                }
                seen_ids.insert(id.clone());

                expanded_keyframe_entries.push(ExpandedKeyframeEntry {
                    id,
                    path: entry.path.clone(),
                    animation_name: Some(anim_info.name),
                    skin_name: entry.skin_name.clone(),
                });
            }

            println!(
                "  Expanded {} animations from {}",
                expanded_keyframe_entries.len(),
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        } else {
            // Explicit entry - derive ID if needed
            let id = entry
                .id
                .clone()
                .or_else(|| entry.animation_name.clone())
                .unwrap_or_else(|| "unnamed".to_string());

            expanded_keyframe_entries.push(ExpandedKeyframeEntry {
                id,
                path: entry.path.clone(),
                animation_name: entry.animation_name.clone(),
                skin_name: entry.skin_name.clone(),
            });
        }
    }

    // Now process expanded entries in parallel
    let keyframes: Result<Vec<_>> = expanded_keyframe_entries
        .par_iter()
        .map(|entry| {
            let path = project_dir.join(&entry.path);
            load_keyframes(
                &entry.id,
                &path,
                entry.animation_name.as_deref(),
                entry.skin_name.as_deref(),
            )
        })
        .collect();
    let keyframes = keyframes?;

    // Load sounds in parallel
    let sounds: Result<Vec<_>> = assets
        .sounds
        .par_iter()
        .map(|entry| {
            let id = require_id(entry, "Sound")?;
            let path = project_dir.join(&entry.path);
            load_sound(id, &path)
        })
        .collect();
    let explicit_sounds = sounds?;

    // Build sound map from explicit sounds
    let mut sound_map: HashMap<String, PackedSound> = explicit_sounds
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();

    // Track content hashes for deduplication
    let mut hash_to_id: HashMap<[u8; 32], String> = HashMap::new();
    for sound in sound_map.values() {
        let hash = hash_sample_data(&sound.data);
        hash_to_id.insert(hash, sound.id.clone());
    }

    // Extract samples from ALL tracker files (both XM and IT)
    println!("  Extracting samples from tracker files...");
    for entry in &assets.trackers {
        let tracker_id = require_id(entry, "Tracker")?;
        let path = project_dir.join(&entry.path);
        let tracker_data = std::fs::read(&path)
            .with_context(|| format!("Failed to read tracker: {}", path.display()))?;

        // Detect format
        let format = detect_tracker_format(&tracker_data);

        // Try to extract samples based on format
        let extracted_samples = match format {
            Some(zx_common::TrackerFormat::Xm) => {
                match nether_xm::extract_samples(&tracker_data) {
                    Ok(samples) => samples,
                    Err(e) => {
                        // Sample-less XM file or extraction error - log and continue
                        println!(
                            "    Note: {} ({})",
                            path.file_name().unwrap().to_string_lossy(),
                            e
                        );
                        println!("          No samples extracted (this is expected for sample-less tracker files)");
                        Vec::new()
                    }
                }
            }
            Some(zx_common::TrackerFormat::It) => {
                // Extract samples from IT file
                match nether_it::extract_samples(&tracker_data) {
                    Ok(it_samples) => {
                        // Process IT samples directly here since they have different types
                        for sample in it_samples {
                            // Skip empty samples
                            if sample.data.is_empty() {
                                continue;
                            }

                            // Convert sample to 22050 Hz mono
                            let converted_data = crate::audio_convert::convert_it_sample(&sample);
                            if converted_data.is_empty() {
                                continue;
                            }

                            // Calculate hash for deduplication
                            let hash = hash_sample_data(&converted_data);

                            // Sanitize name (use sample_index for IT)
                            let sample_id =
                                sanitize_name(&sample.name, tracker_id, sample.sample_index);

                            // Check for collision with explicit sounds
                            if let Some(existing) = sound_map.get(&sample_id) {
                                let existing_hash = hash_sample_data(&existing.data);
                                if existing_hash != hash {
                                    return Err(anyhow::anyhow!(
                                        "Collision: IT sample '{}' in '{}' conflicts with explicit sound '{}' (different content)",
                                        sample.name,
                                        tracker_id,
                                        sample_id
                                    ));
                                }
                                // Same content = deduplicated, continue
                                continue;
                            }

                            // Check for hash match (same content, different name)
                            if let Some(existing_name) = hash_to_id.get(&hash) {
                                println!(
                                    "    Note: '{}' is identical to '{}', deduplicating",
                                    sample_id, existing_name
                                );
                                continue;
                            }

                            // Add new sample
                            println!("    Extracted: {} from {}", sample_id, tracker_id);
                            sound_map.insert(
                                sample_id.clone(),
                                PackedSound {
                                    id: sample_id.clone(),
                                    data: converted_data,
                                },
                            );
                            hash_to_id.insert(hash, sample_id);
                        }
                        // Return empty since we processed IT samples inline
                        Vec::new()
                    }
                    Err(e) => {
                        // Sample-less IT file or extraction error
                        println!(
                            "    Note: {} ({})",
                            path.file_name().unwrap().to_string_lossy(),
                            e
                        );
                        println!("          No samples extracted (this is expected for sample-less tracker files)");
                        Vec::new()
                    }
                }
            }
            None => {
                println!("    Warning: Unknown tracker format: {}", path.display());
                Vec::new()
            }
        };

        for sample in extracted_samples {
            // Skip empty samples
            if sample.data.is_empty() {
                continue;
            }

            // Convert sample to 22050 Hz
            let converted_data = crate::audio_convert::convert_xm_sample(&sample);
            if converted_data.is_empty() {
                continue;
            }

            // Calculate hash for deduplication
            let hash = hash_sample_data(&converted_data);

            // Sanitize name
            let sample_id = sanitize_name(&sample.name, tracker_id, sample.instrument_index);

            // Check for collision with explicit sounds
            if let Some(existing) = sound_map.get(&sample_id) {
                let existing_hash = hash_sample_data(&existing.data);
                if existing_hash != hash {
                    return Err(anyhow::anyhow!(
                        "Collision: Tracker instrument '{}' in '{}' conflicts with explicit sound '{}' (different content)",
                        sample.name,
                        tracker_id,
                        sample_id
                    ));
                }
                // Same content = deduplicated, continue
                continue;
            }

            // Check for hash match (same content, different name)
            if let Some(existing_name) = hash_to_id.get(&hash) {
                // Alias: same content already exists under different name
                println!(
                    "    Note: '{}' is identical to '{}', deduplicating",
                    sample_id, existing_name
                );
                continue;
            }

            // Add new sample
            println!("    Extracted: {} from {}", sample_id, tracker_id);
            sound_map.insert(
                sample_id.clone(),
                PackedSound {
                    id: sample_id.clone(),
                    data: converted_data,
                },
            );
            hash_to_id.insert(hash, sample_id);
        }
    }

    // Build set of available sound IDs for validation
    let available_sound_ids: HashSet<String> = sound_map.keys().cloned().collect();

    // Load trackers in parallel (check patterns field)
    let trackers: Result<Vec<_>> = assets
        .trackers
        .par_iter()
        .filter(|entry| entry.patterns.unwrap_or(true)) // Default to true
        .map(|entry| {
            let id = require_id(entry, "Tracker")?;
            let path = project_dir.join(&entry.path);
            load_tracker(id, &path, &available_sound_ids)
        })
        .collect();
    let trackers = trackers?;

    // Convert sound_map back to Vec for data pack
    let sounds: Vec<PackedSound> = sound_map.into_values().collect();

    // Load raw data in parallel
    let data: Result<Vec<_>> = assets
        .data
        .par_iter()
        .map(|entry| {
            let id = require_id(entry, "Data")?;
            let path = project_dir.join(&entry.path);
            load_data(id, &path)
        })
        .collect();
    let data = data?;

    // Print results (after parallel loading completes)
    for texture in &textures {
        let format_str = if texture.format.is_bc7() {
            " [BC7]"
        } else {
            ""
        };
        println!(
            "  Texture: {} ({}x{}){}",
            texture.id, texture.width, texture.height, format_str
        );
    }
    for mesh in &meshes {
        println!("  Mesh: {} ({} vertices)", mesh.id, mesh.vertex_count);
    }
    for kf in &keyframes {
        println!(
            "  Keyframes: {} ({} bones, {} frames)",
            kf.id, kf.bone_count, kf.frame_count
        );
    }
    for sound in &sounds {
        println!("  Sound: {} ({:.2}s)", sound.id, sound.duration_seconds());
    }
    for tracker in &trackers {
        println!(
            "  Tracker: {} ({} instruments)",
            tracker.id,
            tracker.sample_ids.len()
        );
    }
    for d in &data {
        println!("  Data: {} ({} bytes)", d.id, d.data.len());
    }
    for skeleton in &skeletons {
        println!(
            "  Skeleton: {} ({} bones)",
            skeleton.id, skeleton.bone_count
        );
    }

    let total = textures.len()
        + meshes.len()
        + skeletons.len()
        + keyframes.len()
        + sounds.len()
        + trackers.len()
        + data.len();
    if total > 0 {
        println!("  Total: {} assets", total);
    }

    Ok(ZXDataPack::with_assets(
        textures,
        meshes,
        skeletons,
        keyframes,
        vec![], // fonts: TODO: add font loading when needed
        sounds,
        data,
        trackers,
    ))
}
