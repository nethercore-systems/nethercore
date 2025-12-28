//! Basic Asset Generator
//!
//! Generates foundational assets for basic Nethercore examples.
//! These are simple test assets (checkerboard texture, cube mesh) used
//! by multiple examples to demonstrate the asset loading pipeline.

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    // Output locations
    let lib_assets = examples_root.join("_lib/assets");
    let asset_test_assets = examples_root.join("6-assets/asset-test/assets");

    // Create directories if they don't exist
    fs::create_dir_all(&lib_assets)?;
    fs::create_dir_all(&asset_test_assets)?;

    // Generate assets in both locations
    let locations = [
        ("_lib/assets", &lib_assets),
        ("6-assets/asset-test/assets", &asset_test_assets),
    ];

    for (name, path) in &locations {
        println!("  Generating assets in {}...", name);

        let checkerboard_path = path.join("checkerboard.png");
        generate_checkerboard_png(&checkerboard_path)?;
        println!("    ✓ checkerboard.png");

        let cube_path = path.join("cube.obj");
        generate_cube_obj(&cube_path)?;
        println!("    ✓ cube.obj");
    }

    Ok(())
}

fn clean_all_assets(examples_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let assets_to_clean = [
        examples_root.join("_lib/assets/checkerboard.png"),
        examples_root.join("_lib/assets/cube.obj"),
        examples_root.join("6-assets/asset-test/assets/checkerboard.png"),
        examples_root.join("6-assets/asset-test/assets/cube.obj"),
    ];

    for asset_path in &assets_to_clean {
        if asset_path.exists() {
            fs::remove_file(asset_path)?;
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
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
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
