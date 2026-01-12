//! Create Nethercore ZX ROM (.nczx) files

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use nethercore_shared::netplay::NetplayMetadata;
use nethercore_shared::{
    is_safe_game_id, read_file_with_limit, MAX_PNG_BYTES, MAX_WASM_BYTES, ZX_ROM_FORMAT,
};
use zx_common::{ZXMetadata, ZXRom};

/// Arguments for creating an Nethercore ZX ROM
#[derive(Debug, Args)]
pub struct CreateZxArgs {
    /// Path to the compiled WASM file
    pub wasm_file: PathBuf,

    /// Game ID (slug, e.g., "platformer")
    #[arg(long)]
    pub id: String,

    /// Game title
    #[arg(long)]
    pub title: String,

    /// Primary author/studio name
    #[arg(long)]
    pub author: String,

    /// Semantic version (e.g., "1.0.0")
    #[arg(long)]
    pub version: String,

    /// Game description
    #[arg(long)]
    pub description: String,

    /// Category tags (can be specified multiple times)
    #[arg(long = "tag")]
    pub tags: Vec<String>,

    /// Optional thumbnail image path (PNG, will be resized to 256x256)
    #[arg(long)]
    pub thumbnail: Option<PathBuf>,

    /// Optional screenshot paths (PNG, max 5)
    #[arg(long = "screenshot")]
    pub screenshots: Vec<PathBuf>,

    /// Optional platform game UUID
    #[arg(long)]
    pub platform_game_id: Option<String>,

    /// Optional platform author UUID
    #[arg(long)]
    pub platform_author_id: Option<String>,

    /// Render mode: 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid
    #[arg(long)]
    pub render_mode: Option<u32>,

    /// Default resolution (e.g., "640x480")
    #[arg(long)]
    pub default_resolution: Option<String>,

    /// Target FPS
    #[arg(long)]
    pub target_fps: Option<u32>,

    /// Output ROM file path (.nczx)
    #[arg(long, short = 'o')]
    pub output: PathBuf,
}

/// Execute the create-zx command
pub fn execute(args: CreateZxArgs) -> Result<()> {
    println!("Creating Nethercore ZX ROM: {}", args.output.display());

    if !is_safe_game_id(&args.id) {
        anyhow::bail!(
            "Invalid game id '{}': must be a safe single path component",
            args.id
        );
    }

    // 1. Read and validate WASM file
    let code = read_file_with_limit(&args.wasm_file, MAX_WASM_BYTES)
        .with_context(|| format!("Failed to read WASM file: {}", args.wasm_file.display()))?;

    // Validate WASM magic bytes
    if code.len() < 4 || &code[0..4] != b"\0asm" {
        anyhow::bail!(
            "Invalid WASM file: {} (missing \\0asm magic bytes)",
            args.wasm_file.display()
        );
    }

    println!("  ✓ WASM code validated ({} bytes)", code.len());

    // 2. Process thumbnail (resize to 256x256 if needed)
    let thumbnail = if let Some(ref thumb_path) = args.thumbnail {
        let thumb_bytes = read_and_resize_image(thumb_path, 256, 256)
            .with_context(|| format!("Failed to process thumbnail: {}", thumb_path.display()))?;
        println!("  ✓ Thumbnail processed ({} bytes)", thumb_bytes.len());
        Some(thumb_bytes)
    } else {
        None
    };

    // 3. Process screenshots (max 5)
    if args.screenshots.len() > 5 {
        anyhow::bail!(
            "Too many screenshots (max 5, got {})",
            args.screenshots.len()
        );
    }

    let mut screenshots = Vec::new();
    for (i, screenshot_path) in args.screenshots.iter().enumerate() {
        let screenshot_bytes = read_file_with_limit(screenshot_path, MAX_PNG_BYTES)
            .with_context(|| format!("Failed to read screenshot: {}", screenshot_path.display()))?;

        // Validate PNG magic bytes
        if screenshot_bytes.len() < 8 || &screenshot_bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
            anyhow::bail!(
                "Invalid PNG file: {} (missing PNG magic bytes)",
                screenshot_path.display()
            );
        }

        screenshots.push(screenshot_bytes);
        println!(
            "  ✓ Screenshot {} processed ({} bytes)",
            i + 1,
            screenshots[i].len()
        );
    }

    // 4. Validate render mode
    if let Some(mode) = args.render_mode {
        if mode > 3 {
            anyhow::bail!("Invalid render mode: {} (must be 0-3)", mode);
        }
    }

    // 5. Create metadata
    let created_at = chrono::Utc::now().to_rfc3339();
    let tool_version = env!("CARGO_PKG_VERSION").to_string();

    let metadata = ZXMetadata {
        id: args.id.clone(),
        title: args.title.clone(),
        author: args.author.clone(),
        version: args.version.clone(),
        description: args.description.clone(),
        tags: args.tags.clone(),
        platform_game_id: args.platform_game_id.clone(),
        platform_author_id: args.platform_author_id.clone(),
        created_at,
        tool_version,
        render_mode: args.render_mode,
        default_resolution: args.default_resolution.clone(),
        target_fps: args.target_fps,
        netplay: NetplayMetadata {
            max_players: 1, // Single-player
            ..Default::default()
        },
    };

    // 6. Create ROM
    let rom = ZXRom {
        version: ZX_ROM_FORMAT.version,
        metadata,
        code,
        data_pack: None, // TODO: Support data pack via nether CLI
        thumbnail,
        screenshots,
    };

    // 7. Validate ROM structure
    rom.validate().context("ROM validation failed")?;

    // 8. Serialize to file
    let rom_bytes = rom.to_bytes().context("Failed to serialize ROM")?;

    std::fs::write(&args.output, &rom_bytes)
        .with_context(|| format!("Failed to write ROM file: {}", args.output.display()))?;

    println!("\n✓ ROM created successfully: {}", args.output.display());
    println!("  Game ID: {}", args.id);
    println!("  Title: {}", args.title);
    println!("  Version: {}", args.version);
    println!("  ROM size: {} bytes", rom_bytes.len());
    println!("  WASM code: {} bytes", rom.code.len());
    if rom.thumbnail.is_some() {
        println!("  Thumbnail: included");
    }
    if !rom.screenshots.is_empty() {
        println!("  Screenshots: {}", rom.screenshots.len());
    }

    Ok(())
}

/// Read an image and resize it to target dimensions, returning PNG bytes
fn read_and_resize_image(path: &Path, width: u32, height: u32) -> Result<Vec<u8>> {
    let bytes = read_file_with_limit(path, MAX_PNG_BYTES)
        .with_context(|| format!("Failed to read image: {}", path.display()))?;
    let img = image::load_from_memory(&bytes)
        .with_context(|| format!("Failed to decode image: {}", path.display()))?;

    // Resize to target dimensions
    let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

    // Encode as PNG
    let mut png_bytes = Vec::new();
    resized
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .with_context(|| format!("Failed to encode image as PNG: {}", path.display()))?;

    Ok(png_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_min_wasm(path: &Path) {
        // WASM magic + version 1.
        std::fs::write(path, b"\0asm\x01\0\0\0").expect("write wasm");
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        let img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
        img.save(path).expect("write png");
    }

    fn base_args(tmp: &TempDir) -> CreateZxArgs {
        let wasm_file = tmp.path().join("game.wasm");
        write_min_wasm(&wasm_file);

        CreateZxArgs {
            wasm_file,
            id: "test-game".to_string(),
            title: "Test Game".to_string(),
            author: "Test Author".to_string(),
            version: "1.2.3".to_string(),
            description: "desc".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            thumbnail: None,
            screenshots: vec![],
            platform_game_id: None,
            platform_author_id: None,
            render_mode: None,
            default_resolution: None,
            target_fps: None,
            output: tmp.path().join("out.nczx"),
        }
    }

    #[test]
    fn create_zx_writes_parseable_rom() {
        let tmp = TempDir::new().unwrap();
        let args = base_args(&tmp);
        execute(args).expect("create-zx execute");

        let bytes = std::fs::read(tmp.path().join("out.nczx")).expect("read out.nczx");
        let rom = ZXRom::from_bytes(&bytes).expect("parse ZXRom");

        assert_eq!(rom.metadata.id, "test-game");
        assert_eq!(rom.metadata.title, "Test Game");
        assert_eq!(rom.metadata.author, "Test Author");
        assert_eq!(rom.metadata.version, "1.2.3");
        assert_eq!(rom.metadata.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(rom.screenshots.len(), 0);
        assert!(rom.thumbnail.is_none());
    }

    #[test]
    fn create_zx_resizes_thumbnail_to_256_square() {
        let tmp = TempDir::new().unwrap();

        let thumb_path = tmp.path().join("thumb.png");
        write_png(&thumb_path, 8, 4);

        let mut args = base_args(&tmp);
        args.thumbnail = Some(thumb_path);
        execute(args).expect("create-zx execute");

        let bytes = std::fs::read(tmp.path().join("out.nczx")).expect("read out.nczx");
        let rom = ZXRom::from_bytes(&bytes).expect("parse ZXRom");
        let thumb = rom.thumbnail.expect("thumbnail present");

        let img = image::load_from_memory(&thumb).expect("decode thumbnail");
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);
    }

    #[test]
    fn create_zx_rejects_too_many_screenshots_without_reading_them() {
        let tmp = TempDir::new().unwrap();

        let mut args = base_args(&tmp);
        args.screenshots = (0..6)
            .map(|i| tmp.path().join(format!("screenshot{}.png", i)))
            .collect();

        let err = execute(args).expect_err("expected error");
        assert!(
            err.to_string().contains("Too many screenshots"),
            "got: {err}"
        );
    }

    #[test]
    fn create_zx_rejects_invalid_wasm_magic() {
        let tmp = TempDir::new().unwrap();
        let wasm_file = tmp.path().join("bad.wasm");
        std::fs::write(&wasm_file, b"not wasm").unwrap();

        let mut args = base_args(&tmp);
        args.wasm_file = wasm_file;

        let err = execute(args).expect_err("expected error");
        assert!(
            err.to_string().contains("missing \\0asm"),
            "got: {err}"
        );
    }

    #[test]
    fn create_zx_rejects_unsafe_game_id() {
        let tmp = TempDir::new().unwrap();
        let mut args = base_args(&tmp);
        args.id = "../evil".to_string();

        let err = execute(args).expect_err("expected error");
        assert!(
            err.to_string().contains("Invalid game id"),
            "got: {err}"
        );
    }

    #[test]
    fn create_zx_rejects_invalid_screenshot_png_magic() {
        let tmp = TempDir::new().unwrap();

        let bad_png = tmp.path().join("bad.png");
        std::fs::write(&bad_png, b"not a png").unwrap();

        let mut args = base_args(&tmp);
        args.screenshots = vec![bad_png];

        let err = execute(args).expect_err("expected error");
        assert!(
            err.to_string().contains("Invalid PNG file"),
            "got: {err}"
        );
    }
}
