//! Basic Asset Generator
//!
//! Generates foundational assets for basic Nethercore examples.
//! These are simple test assets (checkerboard texture, cube mesh) used
//! by multiple examples to demonstrate the asset loading pipeline.

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
#[command(name = "gen-basic-assets")]
#[command(about = "Generate basic assets for Nethercore examples")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all basic assets
    All,
    /// Remove all generated assets
    Clean,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Determine project root (assumed to be current directory when run via cargo xtask)
    let project_root = std::env::current_dir()?;
    let examples_root = project_root.join("examples");

    match cli.command {
        Commands::All => {
            println!("Generating basic assets...");
            generate_all_assets(&examples_root)?;
            println!("Basic assets generated successfully");
        }
        Commands::Clean => {
            println!("Cleaning basic assets...");
            clean_all_assets(&examples_root)?;
            println!("Basic assets cleaned successfully");
        }
    }

    Ok(())
}

fn generate_all_assets(examples_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // All assets go to the shared examples/assets folder
    let shared_assets = examples_root.join("assets");
    fs::create_dir_all(&shared_assets)?;

    println!("  Generating assets in examples/assets/...");

    let checkerboard_path = shared_assets.join("checkerboard.png");
    generate_checkerboard_png(&checkerboard_path)?;
    println!("    ✓ checkerboard.png");

    // Generate checkerboard.nczxtex (converted from checkerboard.png)
    let checkerboard_tex_path = shared_assets.join("checkerboard.nczxtex");
    generate_checkerboard_nczxtex(&checkerboard_tex_path)?;
    println!("    ✓ checkerboard.nczxtex");

    let cube_path = shared_assets.join("cube.obj");
    generate_cube_obj(&cube_path)?;
    println!("    ✓ cube.obj");

    // Generate cube.nczxmesh (converted from cube.obj)
    let cube_mesh_path = shared_assets.join("cube.nczxmesh");
    generate_cube_nczxmesh(&cube_mesh_path)?;
    println!("    ✓ cube.nczxmesh");

    // Generate beep.wav
    let beep_path = shared_assets.join("beep.wav");
    generate_beep_wav(&beep_path)?;
    println!("    ✓ beep.wav");

    // Generate level files
    for level_num in 1..=3u8 {
        let level_path = shared_assets.join(format!("level{}.bin", level_num));
        generate_level_bin(&level_path, level_num)?;
        println!("    ✓ level{}.bin", level_num);
    }

    Ok(())
}

fn clean_all_assets(examples_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let shared_assets = examples_root.join("assets");

    let asset_names = [
        "checkerboard.png",
        "checkerboard.nczxtex",
        "cube.obj",
        "cube.nczxmesh",
        "beep.wav",
        "level1.bin",
        "level2.bin",
        "level3.bin",
    ];

    for asset_name in &asset_names {
        let asset_path = shared_assets.join(asset_name);
        if asset_path.exists() {
            fs::remove_file(&asset_path)?;
            println!("  Removed {}", asset_path.display());
        }
    }

    Ok(())
}

/// Generate a simple 4x4 checkerboard PNG
fn generate_checkerboard_png(path: &Path) -> std::io::Result<()> {
    let width = 4u32;
    let height = 4u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let is_white = (x + y) % 2 == 0;
            if is_white {
                pixels[idx] = 255; // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = 255; // A
            } else {
                pixels[idx] = 128; // R
                pixels[idx + 1] = 64; // G
                pixels[idx + 2] = 192; // B
                pixels[idx + 3] = 255; // A
            }
        }
    }

    image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)
        .map_err(std::io::Error::other)
}

/// Generate a simple cube OBJ file (text format)
fn generate_cube_obj(path: &Path) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;

    // Simple 1x1x1 cube centered at origin with normals
    writeln!(file, "# Simple cube for testing")?;
    writeln!(file)?;

    // Vertices (8 corners of unit cube)
    writeln!(file, "v -0.5 -0.5  0.5")?;
    writeln!(file, "v  0.5 -0.5  0.5")?;
    writeln!(file, "v  0.5  0.5  0.5")?;
    writeln!(file, "v -0.5  0.5  0.5")?;
    writeln!(file, "v -0.5 -0.5 -0.5")?;
    writeln!(file, "v  0.5 -0.5 -0.5")?;
    writeln!(file, "v  0.5  0.5 -0.5")?;
    writeln!(file, "v -0.5  0.5 -0.5")?;
    writeln!(file)?;

    // Normals (6 face normals)
    writeln!(file, "vn  0  0  1")?; // front
    writeln!(file, "vn  0  0 -1")?; // back
    writeln!(file, "vn  1  0  0")?; // right
    writeln!(file, "vn -1  0  0")?; // left
    writeln!(file, "vn  0  1  0")?; // top
    writeln!(file, "vn  0 -1  0")?; // bottom
    writeln!(file)?;

    // UVs (simple 0-1 mapping per face)
    writeln!(file, "vt 0 0")?;
    writeln!(file, "vt 1 0")?;
    writeln!(file, "vt 1 1")?;
    writeln!(file, "vt 0 1")?;
    writeln!(file)?;

    // Faces (6 quads = 12 triangles)
    // Format: f v/vt/vn

    // Front face (+Z)
    writeln!(file, "f 1/1/1 2/2/1 3/3/1")?;
    writeln!(file, "f 1/1/1 3/3/1 4/4/1")?;

    // Back face (-Z)
    writeln!(file, "f 6/1/2 5/2/2 8/3/2")?;
    writeln!(file, "f 6/1/2 8/3/2 7/4/2")?;

    // Right face (+X)
    writeln!(file, "f 2/1/3 6/2/3 7/3/3")?;
    writeln!(file, "f 2/1/3 7/3/3 3/4/3")?;

    // Left face (-X)
    writeln!(file, "f 5/1/4 1/2/4 4/3/4")?;
    writeln!(file, "f 5/1/4 4/3/4 8/4/4")?;

    // Top face (+Y)
    writeln!(file, "f 4/1/5 3/2/5 7/3/5")?;
    writeln!(file, "f 4/1/5 7/3/5 8/4/5")?;

    // Bottom face (-Y)
    writeln!(file, "f 5/1/6 6/2/6 2/3/6")?;
    writeln!(file, "f 5/1/6 2/3/6 1/4/6")?;

    Ok(())
}

/// Generate cube.nczxmesh from cube.obj using nether-export
fn generate_cube_nczxmesh(path: &Path) -> std::io::Result<()> {
    // First ensure cube.obj exists in the same directory
    let obj_path = path.with_extension("obj");
    if !obj_path.exists() {
        generate_cube_obj(&obj_path)?;
    }

    // Convert OBJ -> nczxmesh using nether-export
    nether_export::mesh::convert_obj(&obj_path, path, None)
        .map_err(std::io::Error::other)
}

/// Generate checkerboard.nczxtex from checkerboard.png using nether-export
fn generate_checkerboard_nczxtex(path: &Path) -> std::io::Result<()> {
    // First ensure checkerboard.png exists in the same directory
    let png_path = path.with_extension("png");
    if !png_path.exists() {
        generate_checkerboard_png(&png_path)?;
    }

    // Convert PNG -> nczxtex using nether-export
    nether_export::texture::convert_image(&png_path, path)
        .map_err(std::io::Error::other)
}

/// Generate a soft, bouncy "boing" jump sound effect
fn generate_beep_wav(path: &Path) -> std::io::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22050, // ZX standard audio rate
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(std::io::Error::other)?;

    // Soft bouncy "boing" sound - like a spring or rubber band
    let duration_samples = 22050 / 5; // ~0.2 seconds
    let sample_rate = 22050.0;
    let pi = std::f32::consts::PI;

    // Track phase for smooth frequency transitions
    let mut phase = 0.0f32;

    for i in 0..duration_samples {
        let progress = i as f32 / duration_samples as f32;

        // Frequency drops from 800Hz to 200Hz (bouncy descending pitch)
        let freq = 800.0 * (-3.0 * progress).exp() + 200.0;

        // Smooth exponential decay envelope
        let envelope = (-4.0 * progress).exp();

        // Accumulate phase for continuous waveform
        phase += freq / sample_rate;

        // Pure sine wave - soft and pleasant
        let fundamental = (phase * 2.0 * pi).sin();

        // Add subtle second harmonic for warmth (one octave up, quieter)
        let harmonic = (phase * 4.0 * pi).sin() * 0.2;

        // Combine with slight vibrato for organic feel
        let vibrato = 1.0 + 0.02 * (progress * 40.0 * pi).sin();
        let sample = (fundamental + harmonic) * envelope * vibrato;

        writer
            .write_sample((sample * 10000.0) as i16)
            .map_err(std::io::Error::other)?;
    }
    writer
        .finalize()
        .map_err(std::io::Error::other)?;
    Ok(())
}

/// Generate level binary file in ELVL format
///
/// Level format (simple tilemap):
/// - Bytes 0-3: Magic "ELVL"
/// - Byte 4: Version (u8)
/// - Byte 5: Level number (u8)
/// - Bytes 6-7: Width (u16 little-endian)
/// - Bytes 8-9: Height (u16 little-endian)
/// - Remaining: Tile indices (1 byte per tile)
fn generate_level_bin(path: &Path, level_num: u8) -> std::io::Result<()> {
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(b"ELVL"); // Magic
    data.push(1); // Version
    data.push(level_num); // Level number
    data.extend_from_slice(&8u16.to_le_bytes()); // Width
    data.extend_from_slice(&8u16.to_le_bytes()); // Height

    // 8x8 tile data - pattern varies by level
    for y in 0..8u8 {
        for x in 0..8u8 {
            let tile = match level_num {
                // Level 1: Border walls
                1 => {
                    if x == 0 || x == 7 || y == 0 || y == 7 {
                        1
                    } else {
                        0
                    }
                }
                // Level 2: Checkerboard pattern
                2 => {
                    if (x + y) % 2 == 0 {
                        1
                    } else {
                        0
                    }
                }
                // Level 3: Grid pattern with decoration
                _ => {
                    if x % 3 == 0 || y % 3 == 0 {
                        2
                    } else {
                        0
                    }
                }
            };
            data.push(tile);
        }
    }

    fs::write(path, data)
}
