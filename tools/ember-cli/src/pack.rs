//! Pack command - create .ewz ROM from WASM + assets
//!
//! Automatically compresses textures based on render mode:
//! - Mode 0 (Unlit): RGBA8 (uncompressed)
//! - Mode 1-3 (Matcap/PBR/Hybrid): BC7 (4× compression)

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use z_common::{
    vertex_stride_packed, EmberZAnimationHeader, EmberZMeshHeader, PackedData, PackedKeyframes,
    PackedMesh, PackedSound, PackedTexture, TextureFormat, ZDataPack, ZMetadata, ZRom, EWZ_VERSION,
};

use crate::manifest::{AssetsSection, EmberManifest};

/// Arguments for the pack command
#[derive(Args)]
pub struct PackArgs {
    /// Path to ember.toml manifest file
    #[arg(short, long, default_value = "ember.toml")]
    pub manifest: PathBuf,

    /// Output .ewz file path
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
    let manifest = EmberManifest::load(manifest_path)?;

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

    // Analyze WASM to get render mode
    let analysis =
        emberware_core::analysis::analyze_wasm(&code).context("Failed to analyze WASM file")?;
    let render_mode = analysis.render_mode;

    // Determine texture format based on render mode
    let texture_format = if render_mode == 0 {
        TextureFormat::Rgba8
    } else {
        TextureFormat::Bc7
    };

    let format_name = match texture_format {
        TextureFormat::Rgba8 => "RGBA8 (uncompressed)",
        TextureFormat::Bc7 => "BC7 (4× compressed)",
        TextureFormat::Bc7Linear => "BC7 Linear (4× compressed)",
    };
    println!(
        "  Render mode: {} -> textures: {}",
        render_mode, format_name
    );

    // Load assets into data pack
    let data_pack = load_assets(project_dir, &manifest.assets, texture_format)?;

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
    };

    // Create ROM
    let rom = ZRom {
        version: EWZ_VERSION,
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
    let output_path = args
        .output
        .unwrap_or_else(|| project_dir.join(format!("{}.ewz", manifest.game.id)));

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

/// Load assets from disk into a data pack (parallel)
fn load_assets(
    project_dir: &std::path::Path,
    assets: &AssetsSection,
    texture_format: TextureFormat,
) -> Result<ZDataPack> {
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

    // Load keyframes in parallel
    let keyframes: Result<Vec<_>> = assets
        .keyframes
        .par_iter()
        .map(|entry| {
            let path = project_dir.join(&entry.path);
            load_keyframes(&entry.id, &path)
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
    let sounds = sounds?;

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
    for d in &data {
        println!("  Data: {} ({} bytes)", d.id, d.data.len());
    }

    let total = textures.len() + meshes.len() + keyframes.len() + sounds.len() + data.len();
    if total > 0 {
        println!("  Total: {} assets", total);
    }

    Ok(ZDataPack {
        textures,
        meshes,
        skeletons: vec![], // TODO: add skeleton loading when needed
        keyframes,
        fonts: vec![], // TODO: add font loading when needed
        sounds,
        data,
    })
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
        TextureFormat::Bc7 | TextureFormat::Bc7Linear => compress_bc7(pixels, width, height)?,
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
    let blocks_x = (w + 3) / 4;
    let blocks_y = (h + 3) / 4;
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

/// Load a mesh from file
///
/// Supports: .ewzmesh / .embermesh (Emberware Z mesh format)
/// Future: .gltf, .glb, .obj (via ember-export)
fn load_mesh(id: &str, path: &std::path::Path) -> Result<PackedMesh> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load mesh: {}", path.display()))?;

    // Parse EmberZMesh header
    let header = EmberZMeshHeader::from_bytes(&data)
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
    let expected_size = EmberZMeshHeader::SIZE + vertex_data_size + index_data_size;
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
    let vertex_start = EmberZMeshHeader::SIZE;
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

/// Load keyframes from .ewzanim file
///
/// The new platform format (16 bytes per bone per frame):
/// - Header: 4 bytes (bone_count u8, flags u8, frame_count u16 LE)
/// - Data: frame_count × bone_count × 16 bytes
fn load_keyframes(id: &str, path: &std::path::Path) -> Result<PackedKeyframes> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load keyframes: {}", path.display()))?;

    // Parse header
    let header = EmberZAnimationHeader::from_bytes(&data)
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
    let frame_data = data[EmberZAnimationHeader::SIZE..expected_size].to_vec();

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
        let manifest = EmberManifest::parse(manifest_toml).unwrap();
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
        let manifest = EmberManifest::parse(manifest_toml).unwrap();
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
        let manifest = EmberManifest::parse(manifest_toml).unwrap();
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
    fn test_load_mesh_ewzmesh() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test.ewzmesh");

        // Create a minimal EmberZMesh file
        // Format 0 = position only (8 bytes per vertex)
        // 3 vertices, 3 indices (a triangle)
        let header = EmberZMeshHeader::new(3, 3, 0);
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
        use z_common::{FORMAT_COLOR, FORMAT_UV};

        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test_uv_color.ewzmesh");

        // Format 3 = position (8) + UV (4) + color (4) = 16 bytes per vertex
        let format = FORMAT_UV | FORMAT_COLOR;

        let header = EmberZMeshHeader::new(4, 6, format);
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
        let mesh_path = dir.path().join("bad.ewzmesh");

        // File too small to contain header
        std::fs::write(&mesh_path, &[0u8; 5]).unwrap();

        let result = load_mesh("bad", &mesh_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_mesh_invalid_truncated_data() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("truncated.ewzmesh");

        // Valid header but truncated vertex data
        let header = EmberZMeshHeader::new(10, 0, 0); // Claims 10 vertices (80 bytes needed)
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
    fn test_load_keyframes_ewzanim() {
        let dir = tempdir().unwrap();
        let anim_path = dir.path().join("test.ewzanim");

        // Create a minimal .ewzanim file
        // 2 bones, 3 frames (2 * 3 * 16 = 96 bytes of data)
        let header = EmberZAnimationHeader::new(2, 3);
        let mut anim_data = header.to_bytes().to_vec();

        // Add frame data (96 bytes)
        anim_data.extend_from_slice(&[0u8; 96]);

        std::fs::write(&anim_path, &anim_data).unwrap();

        // Load it
        let packed = load_keyframes("test_anim", &anim_path).unwrap();
        assert_eq!(packed.id, "test_anim");
        assert_eq!(packed.bone_count, 2);
        assert_eq!(packed.frame_count, 3);
        assert_eq!(packed.data.len(), 96);
        assert!(packed.validate());
    }

    #[test]
    fn test_load_keyframes_invalid_header() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("bad.ewzanim");

        // Invalid header (bone_count = 0)
        let mut data = vec![0u8; 4];
        data[0] = 0; // bone_count = 0 (invalid)
        data[1] = 0; // flags
        data[2] = 10; // frame_count LSB
        data[3] = 0; // frame_count MSB

        std::fs::write(&bad_path, &data).unwrap();

        let result = load_keyframes("bad", &bad_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_keyframes_truncated() {
        let dir = tempdir().unwrap();
        let trunc_path = dir.path().join("truncated.ewzanim");

        // Valid header but truncated data
        let header = EmberZAnimationHeader::new(5, 10); // 5 bones, 10 frames = 800 bytes
        let mut data = header.to_bytes().to_vec();
        data.extend_from_slice(&[0u8; 100]); // Only 100 bytes instead of 800

        std::fs::write(&trunc_path, &data).unwrap();

        let result = load_keyframes("truncated", &trunc_path);
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
path = "assets/walk.ewzanim"

[[assets.keyframes]]
id = "run"
path = "assets/run.ewzanim"
"#;
        let manifest = EmberManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.keyframes.len(), 2);
        assert_eq!(manifest.assets.keyframes[0].id, "walk");
        assert_eq!(manifest.assets.keyframes[1].id, "run");
    }
}
