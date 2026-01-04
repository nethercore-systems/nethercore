//! Compile command - run build script and validate WASM
//!
//! Executes the build script from nether.toml (or default cargo build),
//! then validates the resulting WASM file.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

use crate::manifest::NetherManifest;

/// Arguments for the compile command
#[derive(Args)]
pub struct CompileArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Path to nether.toml manifest file (relative to project directory)
    #[arg(short, long, default_value = "nether.toml")]
    pub manifest: PathBuf,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,
}

/// Execute the compile command
///
/// Returns the path to the compiled WASM file on success.
pub fn execute(args: CompileArgs) -> Result<PathBuf> {
    let project_dir = args
        .project
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = project_dir.join(&args.manifest);

    // Load manifest
    let manifest = NetherManifest::load(&manifest_path)?;

    println!("Compiling {}...", manifest.game.title);

    // Get build script
    let script = manifest.build_script(args.debug);
    println!("  Script: {}", script);

    // Parse script into command and arguments
    let parts: Vec<&str> = script.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty build script");
    }

    let (cmd, cmd_args) = parts.split_first().unwrap();

    // Execute build script
    // Use status() to inherit stdout/stderr so compiler output is visible
    let status = Command::new(cmd)
        .args(cmd_args)
        .current_dir(&project_dir)
        .status()
        .with_context(|| format!("Failed to execute build command: {}", cmd))?;

    if !status.success() {
        // Compiler errors were already printed to stderr
        anyhow::bail!(
            "Compilation failed (exit code: {})\nCheck the error messages above.",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }

    // Find WASM file
    let wasm_path = manifest.find_wasm(&project_dir, args.debug)?;

    // Validate WASM
    let wasm_bytes = std::fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;

    validate_wasm(&wasm_bytes)?;

    let size = wasm_bytes.len();
    println!("  Output: {} ({} bytes)", wasm_path.display(), size);

    Ok(wasm_path)
}

/// Validate WASM file
///
/// Checks:
/// - Magic bytes (\0asm)
fn validate_wasm(bytes: &[u8]) -> Result<()> {
    // Check magic bytes
    if bytes.len() < 8 {
        anyhow::bail!("Invalid WASM file: too small ({} bytes)", bytes.len());
    }

    if &bytes[0..4] != b"\0asm" {
        anyhow::bail!(
            "Invalid WASM file: bad magic bytes (expected \\0asm, got {:?})",
            &bytes[0..4]
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wasm_valid_empty_module() {
        // Valid minimal WASM module (magic + version, no sections)
        // This is technically valid WASM
        let valid = b"\0asm\x01\x00\x00\x00";
        let result = validate_wasm(valid);
        // Empty module is valid - has magic bytes and passes analysis
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wasm_bad_magic() {
        let invalid = b"notawasm";
        let result = validate_wasm(invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bad magic bytes"));
    }

    #[test]
    fn test_validate_wasm_too_small() {
        let tiny = b"\0as";
        let result = validate_wasm(tiny);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
