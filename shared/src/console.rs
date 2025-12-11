//! Console specifications for Emberware fantasy consoles.

/// Specifications for a fantasy console.
///
/// Defines the hardware limits and capabilities of a fantasy console
/// (e.g., Emberware Z, Emberware Classic). Used by both the platform
/// backend for validation and the console clients for enforcement.
#[derive(Debug, Clone)]
pub struct ConsoleSpecs {
    /// Console name (e.g., "Emberware Z")
    pub name: &'static str,
    /// Available resolutions (width, height)
    pub resolutions: &'static [(u32, u32)],
    /// Default resolution index
    pub default_resolution: usize,
    /// Available tick rates in Hz
    pub tick_rates: &'static [u32],
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

/// Emberware Z ROM limit (12 MB total cartridge: WASM code + assets)
pub const EMBERWARE_Z_ROM_LIMIT: usize = 12 * 1024 * 1024;

/// Emberware Z RAM limit (4 MB WASM linear memory)
pub const EMBERWARE_Z_RAM_LIMIT: usize = 4 * 1024 * 1024;

/// Emberware Z VRAM limit (4 MB GPU textures and mesh buffers)
pub const EMBERWARE_Z_VRAM_LIMIT: usize = 4 * 1024 * 1024;

/// Get Emberware Z console specifications.
///
/// PS1/N64-era aesthetic with modern 3D rendering capabilities.
/// Supports PBR lighting, GPU skinning, and 4-player local/online multiplayer.
///
/// # Memory Model
///
/// Emberware Z uses a **12MB ROM + 4MB RAM** memory model with datapack-based
/// asset loading. This separates immutable data from game state, enabling
/// efficient rollback (only 4MB snapshotted) while providing generous content
/// headroom (12MB total ROM).
///
/// - **ROM (Cartridge):** 12 MB total (WASM code + assets via datapack)
/// - **RAM:** 4 MB WASM linear memory (code + heap + stack)
/// - **VRAM:** 4 MB GPU textures and mesh buffers
/// - **Rollback:** Only 4 MB RAM snapshotted (~0.25ms with xxHash3)
///
/// Assets loaded via `rom_*` FFI go directly to VRAM/audio memory on the host.
/// Only handles (u32 IDs) live in game state, making rollback fast and efficient.
pub const fn emberware_z_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        name: "Emberware Z",
        resolutions: EMBERWARE_Z_RESOLUTIONS,
        default_resolution: 1, // 540p
        tick_rates: EMBERWARE_Z_TICK_RATES,
        default_tick_rate: 2,               // 60 fps
        ram_limit: EMBERWARE_Z_RAM_LIMIT,   // 4MB linear memory
        vram_limit: EMBERWARE_Z_VRAM_LIMIT, // 4MB GPU
        rom_limit: EMBERWARE_Z_ROM_LIMIT,   // 12MB cartridge
        cpu_budget_us: 4000,                // 4ms per tick at 60fps
    }
}

// === Emberware Classic Specifications ===

/// Emberware Classic unified memory limit (2 MB)
pub const EMBERWARE_CLASSIC_MEMORY_LIMIT: usize = 2 * 1024 * 1024;

/// Emberware Classic VRAM limit (1 MB)
pub const EMBERWARE_CLASSIC_VRAM_LIMIT: usize = 1024 * 1024;

/// Get Emberware Classic console specifications.
///
/// SNES/Genesis-era aesthetic with 2D-only rendering, tilemaps,
/// sprite layers, and palette swapping.
///
/// # Memory Model
///
/// Emberware Classic uses a **unified 2MB memory model**. Everything lives in
/// WASM linear memory: code, assets (via `include_bytes!`), stack, and heap.
/// This entire memory is snapshotted for rollback netcode.
///
/// - **Memory:** 2 MB unified (code + assets + game state)
/// - **VRAM:** 1 MB (GPU textures and sprite buffers)
pub const fn emberware_classic_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        name: "Emberware Classic",
        resolutions: &[
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
        tick_rates: &[30, 60],
        default_tick_rate: 1,                      // 60 fps
        ram_limit: EMBERWARE_CLASSIC_MEMORY_LIMIT, // 2MB unified
        vram_limit: EMBERWARE_CLASSIC_VRAM_LIMIT,  // 1MB GPU
        rom_limit: EMBERWARE_CLASSIC_MEMORY_LIMIT, // Same as ram_limit (unified)
        cpu_budget_us: 4000,                       // 4ms per tick at 60fps
    }
}
