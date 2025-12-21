//! Console specifications for Emberware fantasy consoles.

/// Specifications for a fantasy console.
///
/// Defines the hardware limits and capabilities of a fantasy console
/// (e.g., Emberware ZX, Emberware Chroma). Used by both the platform
/// backend for validation and the console clients for enforcement.
#[derive(Debug, Clone)]
pub struct ConsoleSpecs {
    /// Console name (e.g., "Emberware ZX")
    pub name: &'static str,
    /// Fixed resolution (width, height)
    pub resolution: (u32, u32),
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

// === Emberware ZX Specifications ===

/// Emberware ZX fixed resolution (540p, 16:9 aspect ratio)
pub const EMBERWARE_ZX_RESOLUTION: (u32, u32) = (960, 540);

/// Emberware ZX tick rates (updates per second)
pub const EMBERWARE_ZX_TICK_RATES: &[u32] = &[24, 30, 60, 120];

/// Emberware ZX ROM limit (12 MB total cartridge: WASM code + assets)
pub const EMBERWARE_ZX_ROM_LIMIT: usize = 12 * 1024 * 1024;

/// Emberware ZX RAM limit (4 MB WASM linear memory)
pub const EMBERWARE_ZX_RAM_LIMIT: usize = 4 * 1024 * 1024;

/// Emberware ZX VRAM limit (4 MB GPU textures and mesh buffers)
pub const EMBERWARE_ZX_VRAM_LIMIT: usize = 4 * 1024 * 1024;

/// Get Emberware ZX console specifications.
///
/// PS1/N64-era aesthetic with modern 3D rendering capabilities.
/// Supports PBR lighting, GPU skinning, and 4-player local/online multiplayer.
///
/// # Memory Model
///
/// Emberware ZX uses a **12MB ROM + 4MB RAM** memory model with datapack-based
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
pub const fn emberware_zx_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        name: "Emberware ZX",
        resolution: EMBERWARE_ZX_RESOLUTION, // Fixed 540p
        tick_rates: EMBERWARE_ZX_TICK_RATES,
        default_tick_rate: 2,                // 60 fps
        ram_limit: EMBERWARE_ZX_RAM_LIMIT,   // 4MB linear memory
        vram_limit: EMBERWARE_ZX_VRAM_LIMIT, // 4MB GPU
        rom_limit: EMBERWARE_ZX_ROM_LIMIT,   // 12MB cartridge
        cpu_budget_us: 4000,                 // 4ms per tick at 60fps
    }
}

// === Emberware Chroma Specifications ===

/// Emberware Chroma unified memory limit (2 MB)
pub const EMBERWARE_CHROMA_MEMORY_LIMIT: usize = 2 * 1024 * 1024;

/// Emberware Chroma VRAM limit (1 MB)
pub const EMBERWARE_CHROMA_VRAM_LIMIT: usize = 1024 * 1024;

/// Emberware Chroma fixed resolution (288x216, 4:3 aspect ratio)
pub const EMBERWARE_CHROMA_RESOLUTION: (u32, u32) = (288, 216);

/// Get Emberware Chroma console specifications.
///
/// SNES/Genesis-era aesthetic with 2D-only rendering, tilemaps,
/// sprite layers, and palette swapping.
///
/// # Memory Model
///
/// Emberware Chroma uses a **unified 2MB memory model**. Everything lives in
/// WASM linear memory: code, assets (via `include_bytes!`), stack, and heap.
/// This entire memory is snapshotted for rollback netcode.
///
/// - **Memory:** 2 MB unified (code + assets + game state)
/// - **VRAM:** 1 MB (GPU textures and sprite buffers)
pub const fn emberware_chroma_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        name: "Emberware Chroma",
        resolution: EMBERWARE_CHROMA_RESOLUTION, // Fixed 288×216 (4:3)
        tick_rates: &[30, 60],
        default_tick_rate: 1,                     // 60 fps
        ram_limit: EMBERWARE_CHROMA_MEMORY_LIMIT, // 2MB unified
        vram_limit: EMBERWARE_CHROMA_VRAM_LIMIT,  // 1MB GPU
        rom_limit: EMBERWARE_CHROMA_MEMORY_LIMIT, // Same as ram_limit (unified)
        cpu_budget_us: 4000,                      // 4ms per tick at 60fps
    }
}
