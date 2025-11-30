//! Console specifications for Emberware fantasy consoles.

use serde::{Deserialize, Serialize};

/// Specifications for a fantasy console.
///
/// Defines the hardware limits and capabilities of a fantasy console
/// (e.g., Emberware Z, Emberware Classic). Used by both the platform
/// backend for validation and the console clients for enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSpecs {
    /// Console name (e.g., "Emberware Z")
    pub name: String,
    /// Available resolutions (width, height)
    pub resolutions: Vec<(u32, u32)>,
    /// Default resolution index
    pub default_resolution: usize,
    /// Available tick rates in Hz
    pub tick_rates: Vec<u32>,
    /// Default tick rate index
    pub default_tick_rate: usize,
    /// Maximum RAM in bytes
    pub ram_limit: usize,
    /// Maximum VRAM in bytes
    pub vram_limit: usize,
    /// Maximum ROM size in bytes (uncompressed)
    pub rom_limit: usize,
    /// CPU budget per tick in microseconds
    pub cpu_budget_us: u64,
}

// === Emberware Z Specifications ===

/// Emberware Z resolutions (16:9 aspect ratio)
pub const EMBERWARE_Z_RESOLUTIONS: &[(u32, u32)] = &[
    (640, 360),   // 360p
    (960, 540),   // 540p (default)
    (1280, 720),  // 720p
    (1920, 1080), // 1080p
];

/// Emberware Z tick rates (updates per second)
pub const EMBERWARE_Z_TICK_RATES: &[u32] = &[24, 30, 60, 120];

/// Emberware Z VRAM limit (4 MB)
pub const EMBERWARE_Z_VRAM_LIMIT: usize = 4 * 1024 * 1024;

/// Get Emberware Z console specifications.
///
/// PS1/N64-era aesthetic with modern 3D rendering capabilities.
/// Supports PBR lighting, GPU skinning, and 4-player local/online multiplayer.
pub fn emberware_z_specs() -> ConsoleSpecs {
    ConsoleSpecs {
        name: "Emberware Z".to_string(),
        resolutions: EMBERWARE_Z_RESOLUTIONS.to_vec(),
        default_resolution: 1, // 540p
        tick_rates: EMBERWARE_Z_TICK_RATES.to_vec(),
        default_tick_rate: 2, // 60 fps
        ram_limit: 4 * 1024 * 1024,   // 4MB
        vram_limit: EMBERWARE_Z_VRAM_LIMIT,
        rom_limit: 12 * 1024 * 1024,   // 12MB (uncompressed)
        cpu_budget_us: 4000,          // 4ms per tick at 60fps
    }
}

// === Emberware Classic Specifications ===

/// Get Emberware Classic console specifications.
///
/// SNES/Genesis-era aesthetic with 2D-only rendering, tilemaps,
/// sprite layers, and palette swapping.
pub fn emberware_classic_specs() -> ConsoleSpecs {
    ConsoleSpecs {
        name: "Emberware Classic".to_string(),
        resolutions: vec![
            (320, 180), // 0: 16:9, 6× scale to 1080p
            (384, 216), // 1: 16:9, 5× scale to 1080p
            (480, 270), // 2: 16:9, 4× scale to 1080p
            (640, 360), // 3: 16:9, 3× scale to 1080p
            (240, 180), // 4: 4:3, 6× scale to 1080p
            (288, 216), // 5: 4:3, 5× scale to 1080p (default)
            (360, 270), // 6: 4:3, 4× scale to 1080p
            (480, 360), // 7: 4:3, 3× scale to 1080p
        ],
        default_resolution: 5, // 288×216 (4:3)
        tick_rates: vec![30, 60],
        default_tick_rate: 1, // 60 fps
        ram_limit: 1 * 1024 * 1024,   // 1MB
        vram_limit: 1 * 1024 * 1024,  // 1MB
        rom_limit: 4 * 1024 * 1024,   // 4MB (uncompressed)
        cpu_budget_us: 4000,          // 4ms per tick at 60fps
    }
}
