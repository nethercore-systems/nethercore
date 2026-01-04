//! Pack command - create .nczx ROM from WASM + assets
//!
//! Automatically compresses textures based on render mode:
//! - Mode 0 (Lambert): RGBA8 (uncompressed)
//! - Mode 1-3 (Matcap/PBR/Hybrid): BC7 (4× compression)

use anyhow::{Context, Result};
use clap::Args;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

use nethercore_shared::math::BoneMatrix3x4;
use nethercore_shared::netplay::NetplayMetadata;
use nethercore_shared::{ConsoleType, ZX_ROM_FORMAT};
use zx_common::{
    vertex_stride_packed, NetherZXAnimationHeader, NetherZXMeshHeader, NetherZXSkeletonHeader,
    PackedData, PackedKeyframes, PackedMesh, PackedSkeleton, PackedSound, PackedTexture,
    PackedTracker, TextureFormat, TrackerFormat, ZMetadata, ZXDataPack, ZXRom,
    INVERSE_BIND_MATRIX_SIZE,
};

use crate::manifest::{AssetsSection, NetherManifest};

/// Detect tracker format by magic bytes
fn detect_tracker_format(data: &[u8]) -> Option<TrackerFormat> {
    // Check for XM magic: "Extended Module: " (17 bytes)
    if data.len() >= 17 && &data[0..17] == b"Extended Module: " {
        return Some(TrackerFormat::Xm);
    }
    // Check for IT magic: "IMPM" (4 bytes)
    if data.len() >= 4 && &data[0..4] == b"IMPM" {
        return Some(TrackerFormat::It);
    }
    None
}

/// Arguments for the pack command
#[derive(Args)]
pub struct PackArgs {
    /// Path to nether.toml manifest file
    #[arg(short, long, default_value = "nether.toml")]
    pub manifest: PathBuf,

    /// Output .nczx file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Path to WASM file (overrides auto-detection)
    #[arg(long)]
    pub wasm: Option<PathBuf>,
}

/// Execute the pack command
pub fn execute(args: PackArgs) -> Result<()> {
    // Read manifest
    let manifest_path = &args.manifest;
    let manifest = NetherManifest::load(manifest_path)?;
    manifest.validate()?;

    println!(
        "Packing game: {} ({})",
        manifest.game.title, manifest.game.id
    );

    // Get project directory (parent of manifest)
    let project_dir = manifest_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Find WASM file
    let wasm_path = if let Some(wasm) = args.wasm {
        wasm
    } else {
        // Use manifest to find WASM (checks build.wasm first, then auto-detects)
        manifest.find_wasm(project_dir, false)?
    };

    // Read WASM code
    let code = std::fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;
    println!("  WASM: {} ({} bytes)", wasm_path.display(), code.len());

    // Compute ROM hash for NCHS validation
    let rom_hash = xxh3_64(&code);
    println!("  ROM hash: {:016x}", rom_hash);

    // Get render mode from manifest
    let render_mode = manifest.game.render_mode;
    let mode_name = match render_mode {
        0 => "Lambert",
        1 => "Matcap",
        2 => "PBR",
        3 => "Hybrid",
        _ => "Unknown",
    };
    println!("  Render mode: {} ({})", render_mode, mode_name);

    // Determine texture format based on explicit compress_textures flag
    let texture_format = if manifest.game.compress_textures {
        println!("  Texture compression: enabled (BC7, 4:1 ratio)");
        TextureFormat::Bc7
    } else {
        println!("  Texture compression: disabled (RGBA8, uncompressed)");
        TextureFormat::Rgba8
    };

    // Validation: warn if compress_textures doesn't match render_mode
    if render_mode > 0 && !manifest.game.compress_textures {
        eprintln!("  ⚠️  Warning: Detected render_mode {} (Matcap/PBR/Hybrid) but compress_textures=false.", render_mode);
        eprintln!("      Consider enabling texture compression for better performance:");
        eprintln!("      Add 'compress_textures = true' to [game] section in nether.toml");
    }
    if render_mode == 0 && manifest.game.compress_textures {
        eprintln!("  ⚠️  Warning: Detected render_mode 0 (Lambert) but compress_textures=true.");
        eprintln!("      Lambert mode works best with uncompressed RGBA8 textures.");
        eprintln!("      Consider setting 'compress_textures = false' in nether.toml");
    }

    // Load assets into data pack
    let data_pack = load_assets(project_dir, &manifest.assets, texture_format)?;

    // Build netplay metadata
    let netplay = if manifest.netplay.enabled {
        NetplayMetadata::multiplayer(
            ConsoleType::ZX,
            manifest.tick_rate(),
            manifest.game.max_players,
            rom_hash,
        )
    } else {
        NetplayMetadata {
            console_type: ConsoleType::ZX,
            tick_rate: manifest.tick_rate(),
            max_players: manifest.game.max_players,
            netplay_enabled: false,
            rom_hash,
        }
    };

    // Print netplay info
    if manifest.netplay.enabled {
        println!(
            "  Netplay: enabled ({}Hz, {} players)",
            manifest.game.tick_rate, manifest.game.max_players
        );
    } else {
        println!("  Netplay: disabled");
    }

    // Create metadata
    let metadata = ZMetadata {
        id: manifest.game.id.clone(),
        title: manifest.game.title.clone(),
        author: manifest.game.author.clone(),
        version: manifest.game.version.clone(),
        description: manifest.game.description.clone(),
        tags: manifest.game.tags.clone(),
        platform_game_id: None,
        platform_author_id: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        render_mode: Some(render_mode as u32),
        default_resolution: None,
        target_fps: None,
        netplay,
    };

    // Create ROM
    let rom = ZXRom {
        version: ZX_ROM_FORMAT.version,
        metadata,
        code,
        data_pack: if data_pack.is_empty() {
            None
        } else {
            Some(data_pack)
        },
        thumbnail: None,
        screenshots: vec![],
    };

    // Validate ROM
    rom.validate().context("ROM validation failed")?;

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        project_dir.join(format!("{}.{}", manifest.game.id, ZX_ROM_FORMAT.extension))
    });

    // Write ROM
    let rom_bytes = rom.to_bytes().context("Failed to serialize ROM")?;
    std::fs::write(&output_path, &rom_bytes)
        .with_context(|| format!("Failed to write ROM: {}", output_path.display()))?;

    println!();
    println!(
        "Created: {} ({} bytes)",
        output_path.display(),
        rom_bytes.len()
    );
    println!("  Game ID: {}", manifest.game.id);
    println!("  Title: {}", manifest.game.title);
    println!("  Version: {}", manifest.game.version);

    Ok(())
}

/// Sanitize XM instrument name to valid sound ID
///
/// Converts instrument names like "  My Kick!  " to "my_kick"
/// Empty names are auto-generated from tracker ID and instrument index
fn sanitize_name(name: &str, tracker_id: &str, index: u8) -> String {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return format!("{}_inst{}", tracker_id, index);
    }

    let sanitized = trimmed
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Collapse consecutive underscores and trim leading/trailing underscores
    let mut result = String::new();
    let mut prev_was_underscore = false;

    for c in sanitized.chars() {
        if c == '_' {
            if !prev_was_underscore {
                result.push(c);
            }
            prev_was_underscore = true;
        } else {
            result.push(c);
            prev_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

/// Calculate SHA-256 hash of sample data for deduplication
fn hash_sample_data(data: &[i16]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Hash the sample data as bytes
    let bytes = bytemuck::cast_slice::<i16, u8>(data);
    hasher.update(bytes);

    hasher.finalize().into()
}

/// Load assets from disk into a data pack (parallel)
fn load_assets(
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
            let path = project_dir.join(&entry.path);
            load_texture(&entry.id, &path, texture_format)
        })
        .collect();
    let textures = textures?;

    // Load meshes in parallel
    let meshes: Result<Vec<_>> = assets
        .meshes
        .par_iter()
        .map(|entry| {
            let path = project_dir.join(&entry.path);
            load_mesh(&entry.id, &path)
        })
        .collect();
    let meshes = meshes?;

    // Load skeletons in parallel
    let skeletons: Result<Vec<_>> = assets
        .skeletons
        .par_iter()
        .map(|entry| {
            let path = project_dir.join(&entry.path);
            load_skeleton(&entry.id, &path, entry.skin_name.as_deref())
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

    let keyframes: Result<Vec<_>> = all_keyframe_entries
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
            let path = project_dir.join(&entry.path);
            load_sound(&entry.id, &path)
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
        let path = project_dir.join(&entry.path);
        let tracker_data = std::fs::read(&path)
            .with_context(|| format!("Failed to read tracker: {}", path.display()))?;

        // Detect format
        let format = detect_tracker_format(&tracker_data);

        // Try to extract samples based on format
        let extracted_samples = match format {
            Some(TrackerFormat::Xm) => {
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
            Some(TrackerFormat::It) => {
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
                                sanitize_name(&sample.name, &entry.id, sample.sample_index);

                            // Check for collision with explicit sounds
                            if let Some(existing) = sound_map.get(&sample_id) {
                                let existing_hash = hash_sample_data(&existing.data);
                                if existing_hash != hash {
                                    return Err(anyhow::anyhow!(
                                        "Collision: IT sample '{}' in '{}' conflicts with explicit sound '{}' (different content)",
                                        sample.name,
                                        entry.id,
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
                            println!("    Extracted: {} from {}", sample_id, entry.id);
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
            let sample_id = sanitize_name(&sample.name, &entry.id, sample.instrument_index);

            // Check for collision with explicit sounds
            if let Some(existing) = sound_map.get(&sample_id) {
                let existing_hash = hash_sample_data(&existing.data);
                if existing_hash != hash {
                    return Err(anyhow::anyhow!(
                        "Collision: Tracker instrument '{}' in '{}' conflicts with explicit sound '{}' (different content)",
                        sample.name,
                        entry.id,
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
            println!("    Extracted: {} from {}", sample_id, entry.id);
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
            let path = project_dir.join(&entry.path);
            load_tracker(&entry.id, &path, &available_sound_ids)
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
            let path = project_dir.join(&entry.path);
            load_data(&entry.id, &path)
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

/// Load a texture from an image file (PNG, JPG, etc.)
///
/// Compresses to BC7 if the format requires it.
fn load_texture(id: &str, path: &std::path::Path, format: TextureFormat) -> Result<PackedTexture> {
    let img =
        image::open(path).with_context(|| format!("Failed to load texture: {}", path.display()))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.as_raw();

    // Compress or pass through based on format
    let data = match format {
        TextureFormat::Rgba8 => pixels.to_vec(),
        TextureFormat::Bc7 => compress_bc7(pixels, width, height)?,
        TextureFormat::Bc5 => compress_bc5(pixels, width, height)?,
    };

    Ok(PackedTexture::with_format(
        id,
        width as u16,
        height as u16,
        format,
        data,
    ))
}

/// Compress RGBA8 pixels to BC7 format
///
/// Uses intel_tex_2 (ISPC-based) for high-quality BC7 compression.
fn compress_bc7(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use intel_tex_2::bc7;

    let w = width as usize;
    let h = height as usize;

    // Calculate block dimensions (round up to 4×4 blocks)
    let blocks_x = w.div_ceil(4);
    let blocks_y = h.div_ceil(4);
    let output_size = blocks_x * blocks_y * 16;

    let mut output = vec![0u8; output_size];

    // Create padded buffer if dimensions aren't multiples of 4
    let padded_width = blocks_x * 4;
    let padded_height = blocks_y * 4;

    let input_data: Vec<u8> = if w == padded_width && h == padded_height {
        pixels.to_vec()
    } else {
        // Create padded buffer with edge extension
        let mut padded = vec![0u8; padded_width * padded_height * 4];

        for y in 0..padded_height {
            for x in 0..padded_width {
                let src_x = x.min(w - 1);
                let src_y = y.min(h - 1);

                let src_idx = (src_y * w + src_x) * 4;
                let dst_idx = (y * padded_width + x) * 4;

                padded[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
            }
        }

        padded
    };

    // Create surface for intel_tex_2
    let surface = intel_tex_2::RgbaSurface {
        width: padded_width as u32,
        height: padded_height as u32,
        stride: (padded_width * 4) as u32,
        data: &input_data,
    };

    // Compress using intel_tex_2 BC7 (fast settings for good speed/quality balance)
    bc7::compress_blocks_into(&bc7::opaque_fast_settings(), &surface, &mut output);

    Ok(output)
}

/// Compress RGBA8 pixels to BC5 format (2-channel RG)
///
/// Used for normal maps. Extracts R and G channels from RGBA input.
/// The Z component is reconstructed in the shader: z = sqrt(1 - x² - y²)
fn compress_bc5(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use intel_tex_2::bc5;

    let w = width as usize;
    let h = height as usize;

    // Calculate block dimensions (round up to 4×4 blocks)
    let blocks_x = w.div_ceil(4);
    let blocks_y = h.div_ceil(4);
    let output_size = blocks_x * blocks_y * 16; // BC5 is 16 bytes per 4×4 block

    let mut output = vec![0u8; output_size];

    // Create padded buffer if dimensions aren't multiples of 4
    let padded_width = blocks_x * 4;
    let padded_height = blocks_y * 4;

    // Extract R and G channels into a 2-byte-per-pixel buffer
    let mut rg_data: Vec<u8> = Vec::with_capacity(padded_width * padded_height * 2);

    for y in 0..padded_height {
        for x in 0..padded_width {
            let src_x = x.min(w - 1);
            let src_y = y.min(h - 1);
            let src_idx = (src_y * w + src_x) * 4;

            // Extract R and G channels
            rg_data.push(pixels[src_idx]);     // R
            rg_data.push(pixels[src_idx + 1]); // G
        }
    }

    // Create surface for intel_tex_2
    let surface = intel_tex_2::RgSurface {
        width: padded_width as u32,
        height: padded_height as u32,
        stride: (padded_width * 2) as u32,
        data: &rg_data,
    };

    // Compress using intel_tex_2 BC5
    bc5::compress_blocks_into(&surface, &mut output);

    Ok(output)
}

/// Load a mesh from file
///
/// Supports:
/// - .nczxmesh / .nczmesh (Nethercore ZX mesh format) - direct load
/// - .obj (Wavefront OBJ) - auto-converted via nether-export
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
fn load_mesh(id: &str, path: &std::path::Path) -> Result<PackedMesh> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczmesh" | "nczxmesh" => load_mesh_native(id, path),

        // OBJ - convert via nether-export
        "obj" => {
            let converted = nether_export::convert_obj_to_memory(path)
                .with_context(|| format!("Failed to convert OBJ: {}", path.display()))?;

            Ok(PackedMesh {
                id: id.to_string(),
                format: converted.format,
                vertex_count: converted.vertex_count,
                index_count: converted.index_count,
                vertex_data: converted.vertex_data,
                index_data: converted.indices,
            })
        }

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_to_memory(path)
                .with_context(|| format!("Failed to convert glTF: {}", path.display()))?;

            Ok(PackedMesh {
                id: id.to_string(),
                format: converted.format,
                vertex_count: converted.vertex_count,
                index_count: converted.index_count,
                vertex_data: converted.vertex_data,
                index_data: converted.indices,
            })
        }

        _ => anyhow::bail!(
            "Unsupported mesh format '{}': {} (use .nczmesh, .obj, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load a native .nczmesh/.nczxmesh file
fn load_mesh_native(id: &str, path: &std::path::Path) -> Result<PackedMesh> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load mesh: {}", path.display()))?;

    // Parse NetherZXMesh header
    let header = NetherZXMeshHeader::from_bytes(&data)
        .context("Failed to parse mesh header - file may be corrupted or wrong format")?;

    // Validate header
    if header.vertex_count == 0 {
        anyhow::bail!("Mesh has no vertices: {}", path.display());
    }
    if header.format > 15 {
        anyhow::bail!("Invalid mesh format {}: {}", header.format, path.display());
    }

    // Calculate stride based on format flags (using z-common)
    let stride = vertex_stride_packed(header.format) as usize;
    let vertex_data_size = header.vertex_count as usize * stride;
    let index_data_size = header.index_count as usize * 2; // u16 indices

    // Validate data size
    let expected_size = NetherZXMeshHeader::SIZE + vertex_data_size + index_data_size;
    if data.len() < expected_size {
        anyhow::bail!(
            "Mesh data too small: {} bytes, expected {} (vertices: {}, indices: {}, format: {})",
            data.len(),
            expected_size,
            header.vertex_count,
            header.index_count,
            header.format
        );
    }

    // Extract vertex and index data
    let vertex_start = NetherZXMeshHeader::SIZE;
    let vertex_end = vertex_start + vertex_data_size;
    let index_end = vertex_end + index_data_size;

    let vertex_data = data[vertex_start..vertex_end].to_vec();

    // Convert index bytes to u16 values
    let index_data: Vec<u16> = data[vertex_end..index_end]
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(PackedMesh {
        id: id.to_string(),
        format: header.format,
        vertex_count: header.vertex_count,
        index_count: header.index_count,
        vertex_data,
        index_data,
    })
}

/// Load keyframes from file
///
/// Supports:
/// - .nczxanim (Nethercore animation format) - direct load
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
fn load_keyframes(
    id: &str,
    path: &std::path::Path,
    animation_name: Option<&str>,
    skin_name: Option<&str>,
) -> Result<PackedKeyframes> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczxanim" => load_keyframes_native(id, path),

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_animation_to_memory(
                path,
                animation_name,
                skin_name,
                None, // Use default 30 FPS
            )
            .with_context(|| format!("Failed to convert glTF animation: {}", path.display()))?;

            Ok(PackedKeyframes {
                id: id.to_string(),
                bone_count: converted.bone_count,
                frame_count: converted.frame_count,
                data: converted.data,
            })
        }

        _ => anyhow::bail!(
            "Unsupported animation format '{}': {} (use .nczxanim, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load keyframes from native .nczxanim file
fn load_keyframes_native(id: &str, path: &std::path::Path) -> Result<PackedKeyframes> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load keyframes: {}", path.display()))?;

    // Parse header
    let header = NetherZXAnimationHeader::from_bytes(&data)
        .context("Failed to parse keyframes header - file may be corrupted or wrong format")?;

    // Copy values from packed struct to avoid alignment issues
    let bone_count = header.bone_count;
    let frame_count = header.frame_count;
    let flags = header.flags;

    // Validate header
    if !header.validate() {
        anyhow::bail!(
            "Invalid keyframes header (bone_count={}, frame_count={}, flags={}): {}",
            bone_count,
            frame_count,
            flags,
            path.display()
        );
    }

    // Validate data size
    let expected_size = header.file_size();
    if data.len() < expected_size {
        anyhow::bail!(
            "Keyframes data too small: {} bytes, expected {} (bones: {}, frames: {})",
            data.len(),
            expected_size,
            bone_count,
            frame_count
        );
    }

    // Extract frame data (skip header)
    let frame_data = data[NetherZXAnimationHeader::SIZE..expected_size].to_vec();

    Ok(PackedKeyframes {
        id: id.to_string(),
        bone_count,
        frame_count,
        data: frame_data,
    })
}

/// Load a sound from a WAV file
fn load_sound(id: &str, path: &std::path::Path) -> Result<PackedSound> {
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

/// Load raw data from file
fn load_data(id: &str, path: &std::path::Path) -> Result<PackedData> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load data: {}", path.display()))?;

    Ok(PackedData {
        id: id.to_string(),
        data,
    })
}

/// Validate that all non-empty instrument names in a tracker
/// reference loaded sounds in the manifest
fn validate_tracker_samples(
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
fn load_tracker(
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

/// Load a skeleton from file
///
/// Supports:
/// - .nczxskel (Nethercore skeleton format) - direct load
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
fn load_skeleton(
    id: &str,
    path: &std::path::Path,
    skin_name: Option<&str>,
) -> Result<PackedSkeleton> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczxskel" => load_skeleton_native(id, path),

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_skeleton_to_memory(path, skin_name)
                .with_context(|| format!("Failed to convert glTF skeleton: {}", path.display()))?;

            // Convert [f32; 12] column-major to BoneMatrix3x4 row-major
            let inverse_bind_matrices: Vec<BoneMatrix3x4> = converted
                .inverse_bind_matrices
                .iter()
                .map(|mat| {
                    // Input is column-major: [col0.xyz, col1.xyz, col2.xyz, col3.xyz]
                    BoneMatrix3x4::from_rows(
                        [mat[0], mat[3], mat[6], mat[9]],  // row 0
                        [mat[1], mat[4], mat[7], mat[10]], // row 1
                        [mat[2], mat[5], mat[8], mat[11]], // row 2
                    )
                })
                .collect();

            Ok(PackedSkeleton {
                id: id.to_string(),
                bone_count: converted.bone_count,
                inverse_bind_matrices,
            })
        }

        _ => anyhow::bail!(
            "Unsupported skeleton format '{}': {} (use .nczxskel, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load a skeleton from native .nczxskel file
fn load_skeleton_native(id: &str, path: &std::path::Path) -> Result<PackedSkeleton> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load skeleton: {}", path.display()))?;

    // Parse header
    let header = NetherZXSkeletonHeader::from_bytes(&data)
        .context("Failed to parse skeleton header - file may be corrupted or wrong format")?;

    let bone_count = header.bone_count;

    // Validate header
    if bone_count == 0 {
        anyhow::bail!("Skeleton has no bones: {}", path.display());
    }
    if bone_count > 256 {
        anyhow::bail!(
            "Skeleton has too many bones ({}, max 256): {}",
            bone_count,
            path.display()
        );
    }

    // Validate data size
    let expected_size =
        NetherZXSkeletonHeader::SIZE + (bone_count as usize * INVERSE_BIND_MATRIX_SIZE);
    if data.len() < expected_size {
        anyhow::bail!(
            "Skeleton data too small: {} bytes, expected {} (bones: {})",
            data.len(),
            expected_size,
            bone_count
        );
    }

    // Extract inverse bind matrices
    let matrix_data = &data[NetherZXSkeletonHeader::SIZE..expected_size];
    let mut inverse_bind_matrices = Vec::with_capacity(bone_count as usize);

    for i in 0..bone_count as usize {
        let offset = i * INVERSE_BIND_MATRIX_SIZE;
        let matrix_bytes = &matrix_data[offset..offset + INVERSE_BIND_MATRIX_SIZE];

        // Parse 12 floats from file (column-major: col0, col1, col2, col3)
        let mut cols = [[0.0f32; 3]; 4];
        for col in 0..4 {
            for row in 0..3 {
                let float_offset = (col * 3 + row) * 4;
                cols[col][row] = f32::from_le_bytes([
                    matrix_bytes[float_offset],
                    matrix_bytes[float_offset + 1],
                    matrix_bytes[float_offset + 2],
                    matrix_bytes[float_offset + 3],
                ]);
            }
        }

        // Convert to row-major for BoneMatrix3x4
        let matrix = BoneMatrix3x4::from_rows(
            [cols[0][0], cols[1][0], cols[2][0], cols[3][0]], // row 0
            [cols[0][1], cols[1][1], cols[2][1], cols[3][1]], // row 1
            [cols[0][2], cols[1][2], cols[2][2], cols[3][2]], // row 2
        );
        inverse_bind_matrices.push(matrix);
    }

    Ok(PackedSkeleton {
        id: id.to_string(),
        bone_count,
        inverse_bind_matrices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_parsing() {
        let manifest_toml = r#"
[game]
id = "test-game"
title = "Test Game"
author = "Test Author"
version = "1.0.0"
description = "A test game"
tags = ["action", "puzzle"]

[assets]
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.game.id, "test-game");
        assert_eq!(manifest.game.title, "Test Game");
        assert_eq!(manifest.game.author, "Test Author");
        assert_eq!(manifest.game.version, "1.0.0");
        assert_eq!(manifest.game.description, "A test game");
        assert_eq!(manifest.game.tags, vec!["action", "puzzle"]);
    }

    #[test]
    fn test_manifest_with_assets() {
        let manifest_toml = r#"
[game]
id = "asset-game"
title = "Asset Game"
author = "Author"
version = "0.1.0"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.textures]]
id = "enemy"
path = "assets/enemy.png"

[[assets.sounds]]
id = "jump"
path = "assets/jump.wav"

[[assets.data]]
id = "level1"
path = "assets/level1.bin"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.textures.len(), 2);
        assert_eq!(manifest.assets.textures[0].id, "player");
        assert_eq!(manifest.assets.textures[1].id, "enemy");
        assert_eq!(manifest.assets.sounds.len(), 1);
        assert_eq!(manifest.assets.sounds[0].id, "jump");
        assert_eq!(manifest.assets.data.len(), 1);
        assert_eq!(manifest.assets.data[0].id, "level1");
    }

    #[test]
    fn test_manifest_minimal() {
        let manifest_toml = r#"
[game]
id = "minimal"
title = "Minimal Game"
author = "Author"
version = "1.0.0"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.game.id, "minimal");
        assert!(manifest.game.description.is_empty());
        assert!(manifest.game.tags.is_empty());
        assert!(manifest.assets.textures.is_empty());
    }

    #[test]
    fn test_load_data_file() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join("test.bin");

        // Write test data
        let test_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        std::fs::write(&data_path, &test_data).unwrap();

        // Load it
        let packed = load_data("test_asset", &data_path).unwrap();
        assert_eq!(packed.id, "test_asset");
        assert_eq!(packed.data, test_data);
    }

    #[test]
    fn test_load_texture_png_rgba8() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.png");

        // Create a minimal 2x2 PNG
        let img = image::RgbaImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgba([255, 0, 0, 255]) // Red
            } else {
                image::Rgba([0, 255, 0, 255]) // Green
            }
        });
        img.save(&img_path).unwrap();

        // Load it as RGBA8
        let packed = load_texture("test_tex", &img_path, TextureFormat::Rgba8).unwrap();
        assert_eq!(packed.id, "test_tex");
        assert_eq!(packed.width, 2);
        assert_eq!(packed.height, 2);
        assert_eq!(packed.format, TextureFormat::Rgba8);
        assert_eq!(packed.data.len(), 2 * 2 * 4); // RGBA8
    }

    #[test]
    fn test_load_texture_png_bc7() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.png");

        // Create a 16x16 PNG (must be at least 4×4 for BC7)
        let img = image::RgbaImage::from_fn(16, 16, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgba([255, 0, 0, 255]) // Red
            } else {
                image::Rgba([0, 255, 0, 255]) // Green
            }
        });
        img.save(&img_path).unwrap();

        // Load it as BC7
        let packed = load_texture("test_tex", &img_path, TextureFormat::Bc7).unwrap();
        assert_eq!(packed.id, "test_tex");
        assert_eq!(packed.width, 16);
        assert_eq!(packed.height, 16);
        assert_eq!(packed.format, TextureFormat::Bc7);
        // BC7: 4×4 blocks = 16 blocks × 16 bytes = 256 bytes
        assert_eq!(packed.data.len(), 4 * 4 * 16);
    }

    #[test]
    fn test_load_wav_basic() {
        let dir = tempdir().unwrap();
        let wav_path = dir.path().join("test.wav");

        // Create a minimal WAV file (44 byte header + 8 samples)
        let mut wav_data = vec![];

        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&52u32.to_le_bytes()); // File size - 8
        wav_data.extend_from_slice(b"WAVE");

        // fmt chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Num channels (mono)
        wav_data.extend_from_slice(&22050u32.to_le_bytes()); // Sample rate
        wav_data.extend_from_slice(&44100u32.to_le_bytes()); // Byte rate
        wav_data.extend_from_slice(&2u16.to_le_bytes()); // Block align
        wav_data.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample

        // data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size (8 samples * 2 bytes)

        // Audio samples
        let samples: [i16; 8] = [0, 1000, 2000, 3000, 2000, 1000, 0, -1000];
        for sample in &samples {
            wav_data.extend_from_slice(&sample.to_le_bytes());
        }

        std::fs::write(&wav_path, &wav_data).unwrap();

        // Load it
        let packed = load_sound("test_sfx", &wav_path).unwrap();
        assert_eq!(packed.id, "test_sfx");
        assert_eq!(packed.data.len(), 8);
        assert_eq!(packed.data[0], 0);
        assert_eq!(packed.data[1], 1000);
    }

    #[test]
    fn test_load_wav_invalid() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("bad.wav");

        // Not a WAV file
        std::fs::write(&bad_path, b"not a wav file").unwrap();

        let result = load_sound("bad", &bad_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_assets_section() {
        let dir = tempdir().unwrap();
        let assets = AssetsSection::default();

        let pack = load_assets(dir.path(), &assets, TextureFormat::Rgba8).unwrap();
        assert!(pack.is_empty());
        assert_eq!(pack.asset_count(), 0);
    }

    #[test]
    fn test_load_mesh_nczxmesh() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test.nczxmesh");

        // Create a minimal NetherZXMesh file
        // Format 0 = position only (8 bytes per vertex)
        // 3 vertices, 3 indices (a triangle)
        let header = NetherZXMeshHeader::new(3, 3, 0);
        let mut mesh_data = header.to_bytes().to_vec();

        // Add vertex data (3 vertices * 8 bytes = 24 bytes)
        // Position is f16x4, but we just use placeholder bytes
        mesh_data.extend_from_slice(&[0u8; 24]);

        // Add index data (3 indices * 2 bytes = 6 bytes)
        mesh_data.extend_from_slice(&0u16.to_le_bytes()); // index 0
        mesh_data.extend_from_slice(&1u16.to_le_bytes()); // index 1
        mesh_data.extend_from_slice(&2u16.to_le_bytes()); // index 2

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        // Load it
        let packed = load_mesh("test_mesh", &mesh_path).unwrap();
        assert_eq!(packed.id, "test_mesh");
        assert_eq!(packed.format, 0);
        assert_eq!(packed.vertex_count, 3);
        assert_eq!(packed.index_count, 3);
        assert_eq!(packed.vertex_data.len(), 24);
        assert_eq!(packed.index_data.len(), 3);
        assert_eq!(packed.index_data, vec![0, 1, 2]);
    }

    #[test]
    fn test_load_mesh_with_uv_and_color() {
        use zx_common::{FORMAT_COLOR, FORMAT_UV};

        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test_uv_color.nczxmesh");

        // Format 3 = position (8) + UV (4) + color (4) = 16 bytes per vertex
        let format = FORMAT_UV | FORMAT_COLOR;

        let header = NetherZXMeshHeader::new(4, 6, format);
        let mut mesh_data = header.to_bytes().to_vec();

        // Add vertex data (4 vertices * 16 bytes = 64 bytes)
        mesh_data.extend_from_slice(&[0u8; 64]);

        // Add index data (6 indices * 2 bytes = 12 bytes)
        for i in 0u16..6 {
            mesh_data.extend_from_slice(&i.to_le_bytes());
        }

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        // Load it
        let packed = load_mesh("uv_color_mesh", &mesh_path).unwrap();
        assert_eq!(packed.format, format);
        assert_eq!(packed.vertex_count, 4);
        assert_eq!(packed.index_count, 6);
        assert_eq!(packed.vertex_data.len(), 64);
        assert_eq!(packed.index_data.len(), 6);
    }

    #[test]
    fn test_load_mesh_invalid_too_small() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("bad.nczxmesh");

        // File too small to contain header
        std::fs::write(&mesh_path, [0u8; 5]).unwrap();

        let result = load_mesh("bad", &mesh_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_mesh_invalid_truncated_data() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("truncated.nczxmesh");

        // Valid header but truncated vertex data
        let header = NetherZXMeshHeader::new(10, 0, 0); // Claims 10 vertices (80 bytes needed)
        let mut mesh_data = header.to_bytes().to_vec();
        mesh_data.extend_from_slice(&[0u8; 20]); // Only 20 bytes provided

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        let result = load_mesh("truncated", &mesh_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_vertex_stride_packed() {
        // Verify z_common::vertex_stride_packed works as expected
        // Position only
        assert_eq!(vertex_stride_packed(0), 8);

        // Position + UV
        assert_eq!(vertex_stride_packed(1), 12);

        // Position + Color
        assert_eq!(vertex_stride_packed(2), 12);

        // Position + UV + Color
        assert_eq!(vertex_stride_packed(3), 16);

        // Position + Normal
        assert_eq!(vertex_stride_packed(4), 12);

        // Position + UV + Color + Normal
        assert_eq!(vertex_stride_packed(7), 20);

        // Position + Skinned
        assert_eq!(vertex_stride_packed(8), 16);

        // All features
        assert_eq!(vertex_stride_packed(15), 28);
    }

    #[test]
    fn test_load_keyframes_nczxanim() {
        let dir = tempdir().unwrap();
        let anim_path = dir.path().join("test.nczxanim");

        // Create a minimal .nczxanim file
        // 2 bones, 3 frames (2 * 3 * 16 = 96 bytes of data)
        let header = NetherZXAnimationHeader::new(2, 3);
        let mut anim_data = header.to_bytes().to_vec();

        // Add frame data (96 bytes)
        anim_data.extend_from_slice(&[0u8; 96]);

        std::fs::write(&anim_path, &anim_data).unwrap();

        // Load it
        let packed = load_keyframes("test_anim", &anim_path, None, None).unwrap();
        assert_eq!(packed.id, "test_anim");
        assert_eq!(packed.bone_count, 2);
        assert_eq!(packed.frame_count, 3);
        assert_eq!(packed.data.len(), 96);
        assert!(packed.validate());
    }

    #[test]
    fn test_load_keyframes_invalid_header() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("bad.nczxanim");

        // Invalid header (bone_count = 0)
        let mut data = vec![0u8; 4];
        data[0] = 0; // bone_count = 0 (invalid)
        data[1] = 0; // flags
        data[2] = 10; // frame_count LSB
        data[3] = 0; // frame_count MSB

        std::fs::write(&bad_path, &data).unwrap();

        let result = load_keyframes("bad", &bad_path, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_keyframes_truncated() {
        let dir = tempdir().unwrap();
        let trunc_path = dir.path().join("truncated.nczxanim");

        // Valid header but truncated data
        let header = NetherZXAnimationHeader::new(5, 10); // 5 bones, 10 frames = 800 bytes
        let mut data = header.to_bytes().to_vec();
        data.extend_from_slice(&[0u8; 100]); // Only 100 bytes instead of 800

        std::fs::write(&trunc_path, &data).unwrap();

        let result = load_keyframes("truncated", &trunc_path, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_with_keyframes() {
        let manifest_toml = r#"
[game]
id = "anim-game"
title = "Animation Game"
author = "Author"
version = "0.1.0"

[[assets.keyframes]]
id = "walk"
path = "assets/walk.nczxanim"

[[assets.keyframes]]
id = "run"
path = "assets/run.nczxanim"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.keyframes.len(), 2);
        assert_eq!(manifest.assets.keyframes[0].id, "walk");
        assert_eq!(manifest.assets.keyframes[1].id, "run");
    }

    // =========================================================================
    // Tests for XM Sample Extraction Helper Functions
    // =========================================================================

    #[test]
    fn test_sanitize_name_basic() {
        // Basic sanitization
        assert_eq!(sanitize_name("kick", "track", 0), "kick");
        assert_eq!(sanitize_name("Kick", "track", 0), "kick");
        assert_eq!(sanitize_name("KICK", "track", 0), "kick");
    }

    #[test]
    fn test_sanitize_name_whitespace() {
        // Whitespace trimming
        assert_eq!(sanitize_name("  kick  ", "track", 0), "kick");
        assert_eq!(sanitize_name("\tkick\t", "track", 0), "kick");
        assert_eq!(sanitize_name("  My Kick  ", "track", 0), "my_kick");
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        // Special character replacement
        assert_eq!(sanitize_name("My Kick!", "track", 0), "my_kick");
        assert_eq!(sanitize_name("kick@drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick#1", "track", 0), "kick_1");
        assert_eq!(sanitize_name("kick$drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick%drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_multiple_underscores() {
        // Multiple special chars should not create consecutive underscores
        assert_eq!(sanitize_name("kick!!!drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick   drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_valid_chars() {
        // Valid characters should be preserved
        assert_eq!(sanitize_name("kick_drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick-drum", "track", 0), "kick-drum");
        assert_eq!(sanitize_name("kick123", "track", 0), "kick123");
    }

    #[test]
    fn test_sanitize_name_empty() {
        // Empty names should generate from tracker ID and index
        assert_eq!(sanitize_name("", "boss_theme", 0), "boss_theme_inst0");
        assert_eq!(sanitize_name("  ", "boss_theme", 1), "boss_theme_inst1");
        assert_eq!(sanitize_name("\t\n", "boss_theme", 5), "boss_theme_inst5");
    }

    #[test]
    fn test_sanitize_name_leading_trailing_underscores() {
        // Leading/trailing underscores should be removed
        assert_eq!(sanitize_name("_kick_", "track", 0), "kick");
        assert_eq!(sanitize_name("___kick___", "track", 0), "kick");
    }

    #[test]
    fn test_sanitize_name_unicode() {
        // Unicode characters should be replaced
        assert_eq!(sanitize_name("kick♪drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick🥁drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_real_world_examples() {
        // Real-world instrument names from tracker files
        assert_eq!(sanitize_name("BD_KICK_01", "track", 0), "bd_kick_01");
        assert_eq!(
            sanitize_name("Snare (layered)", "track", 0),
            "snare_layered"
        );
        assert_eq!(
            sanitize_name("Hi-Hat [Closed]", "track", 0),
            "hi-hat_closed"
        );
        assert_eq!(sanitize_name("Bass: Deep Sub", "track", 0), "bass_deep_sub");
    }

    #[test]
    fn test_hash_sample_data_consistency() {
        // Same data should produce same hash
        let data1 = vec![100i16, 200, 300, 400, 500];
        let data2 = vec![100i16, 200, 300, 400, 500];

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_different() {
        // Different data should produce different hashes
        let data1 = vec![100i16, 200, 300, 400, 500];
        let data2 = vec![100i16, 200, 300, 400, 501]; // Last value different

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_empty() {
        // Empty data should hash successfully
        let empty: Vec<i16> = Vec::new();
        let hash = hash_sample_data(&empty);

        // Should produce a valid 32-byte hash
        assert_eq!(hash.len(), 32);

        // Empty data should produce consistent hash
        let hash2 = hash_sample_data(&empty);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_sample_data_single_sample() {
        // Single sample should hash
        let data = vec![42i16];
        let hash = hash_sample_data(&data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_sample_data_large() {
        // Large sample (simulating real audio)
        let data: Vec<i16> = (0..22050).map(|i| (i % 1000) as i16).collect();
        let hash = hash_sample_data(&data);
        assert_eq!(hash.len(), 32);

        // Should be deterministic
        let hash2 = hash_sample_data(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_sample_data_order_matters() {
        // Order should matter for hashing
        let data1 = vec![1i16, 2, 3, 4, 5];
        let data2 = vec![5i16, 4, 3, 2, 1]; // Reversed

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_similar_but_different() {
        // Very similar data should still produce different hashes
        let data1 = vec![1000i16; 100]; // 100 samples of value 1000
        let data2 = vec![1001i16; 100]; // 100 samples of value 1001

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_collision_resistance() {
        // Generate multiple different samples and verify no collisions
        let mut hashes = std::collections::HashSet::new();

        for i in 0..100 {
            let data: Vec<i16> = vec![i as i16; 10];
            let hash = hash_sample_data(&data);

            // Should be no collisions
            assert!(hashes.insert(hash), "Hash collision detected!");
        }

        assert_eq!(hashes.len(), 100);
    }
}
