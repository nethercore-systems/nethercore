//! Game state types
//!
//! Minimal core game state - console-agnostic.

use wasmtime::{AsContext, AsContextMut, Memory, ResourceLimiter};

use crate::console::{ConsoleInput, ConsoleRollbackState};

/// Read a length-prefixed string from WASM memory
///
/// Returns None if:
/// - ptr + len exceeds memory bounds
/// - String is not valid UTF-8
pub fn read_string_from_memory<T: 'static>(
    memory: Memory,
    ctx: impl AsContext<Data = T>,
    ptr: u32,
    len: u32,
) -> Option<String> {
    let data = memory.data(&ctx);
    let range = checked_range(ptr, len as usize, data.len())?;

    std::str::from_utf8(&data[range]).ok().map(String::from)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAccessError {
    OutOfBounds,
}

/// Read a byte slice from WASM memory with bounds checks.
pub fn read_bytes_from_memory<T: 'static>(
    memory: Memory,
    ctx: impl AsContext<Data = T>,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, MemoryAccessError> {
    let data = memory.data(&ctx);
    let range =
        checked_range(ptr, len as usize, data.len()).ok_or(MemoryAccessError::OutOfBounds)?;
    Ok(data[range].to_vec())
}

/// Write a byte slice into WASM memory with bounds checks.
pub fn write_bytes_to_memory<T: 'static>(
    memory: Memory,
    mut ctx: impl AsContextMut<Data = T>,
    ptr: u32,
    bytes: &[u8],
) -> Result<(), MemoryAccessError> {
    let data = memory.data_mut(&mut ctx);
    let range =
        checked_range(ptr, bytes.len(), data.len()).ok_or(MemoryAccessError::OutOfBounds)?;
    data[range].copy_from_slice(bytes);
    Ok(())
}

fn checked_range(ptr: u32, len: usize, data_len: usize) -> Option<std::ops::Range<usize>> {
    let start = ptr as usize;
    let end = start.checked_add(len)?;
    if end > data_len {
        return None;
    }
    Some(start..end)
}

use crate::debug::ffi::HasDebugRegistry;
use crate::debug::registry::DebugRegistry;

/// Maximum number of players
pub const MAX_PLAYERS: usize = 4;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 4;

/// Maximum save data size per slot (64KB)
pub const MAX_SAVE_SIZE: usize = 64 * 1024;

/// Default RAM limit used as a fallback for tests and tooling.
pub const DEFAULT_RAM_LIMIT: usize = 4 * 1024 * 1024;

/// Minimal core game state (console-agnostic)
///
/// This struct contains ONLY core WASM execution state:
/// - WASM memory
/// - Game loop timing
/// - Player input (generic over console type)
/// - RNG
/// - Save data
///
/// Rendering state (camera, transforms, draw commands, etc.) is stored
/// in console-specific state via the Console::State associated type.
pub struct GameState<I: ConsoleInput> {
    /// WASM linear memory (set after instantiation)
    pub memory: Option<Memory>,

    /// Current tick number (for determinism)
    pub tick_count: u64,

    /// Elapsed time since game start (seconds)
    pub elapsed_time: f32,

    /// Delta time for current tick (seconds)
    pub delta_time: f32,

    /// Number of players in session
    pub player_count: u32,

    /// Bitmask of local players (bit N = player N is local)
    pub local_player_mask: u32,

    /// Local player handle for netplay (0-3), or None if not connected
    ///
    /// Set after NCHS handshake completes, before post_connect() is called.
    /// Games can query this via player_handle() FFI.
    pub local_player_handle: Option<u8>,

    /// Whether we're currently in init phase
    pub in_init: bool,

    /// RNG state for deterministic random
    pub rng_state: u64,

    /// Input state for all players (previous and current frame)
    pub input_prev: [I; MAX_PLAYERS],
    pub input_curr: [I; MAX_PLAYERS],

    /// Save data slots (8 slots ÁE64KB max each)
    pub save_data: [Option<Vec<u8>>; MAX_SAVE_SLOTS],

    /// Quit requested by game
    pub quit_requested: bool,

    /// Debug frame control state (synced from host before each frame)
    /// Only active in local/offline mode; disabled during netplay.
    pub debug_paused: bool,
    pub debug_time_scale: f32,
}

/// Context for WASM game execution
///
/// Combines core GameState, console-specific FFI state, and rollback state.
/// The generic parameters are:
/// - `I`: Console-specific input type (e.g., ZInput)
/// - `S`: Console-specific FFI state (e.g., ZXFFIState) - NOT rolled back
/// - `R`: Console-specific rollback state (e.g., ZRollbackState) - IS rolled back
pub struct WasmGameContext<I: ConsoleInput, S, R: ConsoleRollbackState = ()> {
    /// Core WASM game state (memory snapshots from this)
    pub game: GameState<I>,
    /// Console-specific per-frame FFI state (NOT rolled back)
    pub ffi: S,
    /// Console-specific rollback state (IS rolled back via bytemuck)
    pub rollback: R,
    /// RAM limit in bytes (for ResourceLimiter enforcement)
    pub ram_limit: usize,
    /// Active save store for this game (host-managed persistent storage)
    pub save_store: Option<crate::save_store::SaveStore>,
    /// Debug inspection registry (for runtime value inspection)
    pub debug_registry: DebugRegistry,
}

/// Type alias for backward compatibility
#[deprecated(note = "Use WasmGameContext instead")]
pub type GameStateWithConsole<I, S> = WasmGameContext<I, S, ()>;

impl<I: ConsoleInput, S: Default, R: ConsoleRollbackState> Default for WasmGameContext<I, S, R> {
    fn default() -> Self {
        Self {
            game: GameState::new(),
            ffi: S::default(),
            rollback: R::default(),
            ram_limit: DEFAULT_RAM_LIMIT, // Fallback default (use ConsoleSpecs in production)
            save_store: None,
            debug_registry: DebugRegistry::new(),
        }
    }
}

impl<I: ConsoleInput, S: Default, R: ConsoleRollbackState> WasmGameContext<I, S, R> {
    /// Create new state with the fallback RAM limit (4MB)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new state with specified RAM limit
    pub fn with_ram_limit(ram_limit: usize) -> Self {
        Self {
            game: GameState::new(),
            ffi: S::default(),
            rollback: R::default(),
            ram_limit,
            save_store: None,
            debug_registry: DebugRegistry::new(),
        }
    }
}

/// Implement HasDebugRegistry trait for generic access to debug registry
impl<I: ConsoleInput, S, R: ConsoleRollbackState> HasDebugRegistry for WasmGameContext<I, S, R> {
    fn debug_registry(&self) -> &DebugRegistry {
        &self.debug_registry
    }

    fn debug_registry_mut(&mut self) -> &mut DebugRegistry {
        &mut self.debug_registry
    }
}

/// Implement ResourceLimiter to enforce console memory constraints
///
/// This prevents malicious or buggy games from allocating more memory
/// than the console allows. The host enforces this limit at the wasmtime
/// level, so games cannot bypass it.
impl<I: ConsoleInput, S: Send, R: ConsoleRollbackState> ResourceLimiter
    for WasmGameContext<I, S, R>
{
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        // Allow growth only if it stays within the RAM limit
        Ok(desired <= self.ram_limit)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        // Allow reasonable table sizes (for indirect function calls)
        Ok(desired <= 10000)
    }
}

impl<I: ConsoleInput> GameState<I> {
    /// Create new game state with default values
    pub fn new() -> Self {
        Self {
            memory: None,
            tick_count: 0,
            elapsed_time: 0.0,
            delta_time: 0.0,
            player_count: 1,
            local_player_mask: 1,
            local_player_handle: None,
            in_init: true,
            rng_state: 0,
            input_prev: [I::default(); MAX_PLAYERS],
            input_curr: [I::default(); MAX_PLAYERS],
            save_data: Default::default(),
            quit_requested: false,
            debug_paused: false,
            debug_time_scale: 1.0,
        }
    }

    /// Seed the RNG with a deterministic value
    pub fn seed_rng(&mut self, seed: u64) {
        self.rng_state = seed;
    }

    /// Generate a deterministic random u32 using PCG algorithm
    pub fn random(&mut self) -> u32 {
        // PCG-XSH-RR algorithm
        let old_state = self.rng_state;
        self.rng_state = old_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        xor_shifted.rotate_right(rot)
    }
}

impl<I: ConsoleInput> Default for GameState<I> {
    fn default() -> Self {
        Self::new()
    }
}
