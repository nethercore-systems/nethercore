//! Game state types
//!
//! Minimal core game state - console-agnostic.

use wasmtime::Memory;

use crate::console::ConsoleInput;

/// Maximum number of players
pub const MAX_PLAYERS: usize = 4;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 8;

/// Maximum save data size per slot (64KB)
pub const MAX_SAVE_SIZE: usize = 64 * 1024;

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

    /// Whether we're currently in init phase
    pub in_init: bool,

    /// RNG state for deterministic random
    pub rng_state: u64,

    /// Input state for all players (previous and current frame)
    pub input_prev: [I; MAX_PLAYERS],
    pub input_curr: [I; MAX_PLAYERS],

    /// Save data slots (8 slots × 64KB max each)
    pub save_data: [Option<Vec<u8>>; MAX_SAVE_SLOTS],

    /// Quit requested by game
    pub quit_requested: bool,
}

/// Wrapper combining core GameState and console-specific state
pub struct GameStateWithConsole<I: ConsoleInput, S> {
    /// Core game state (input, timing, RNG, saves)
    pub game: GameState<I>,
    /// Console-specific state (rendering, transforms, etc.)
    pub console: S,
}

impl<I: ConsoleInput, S: Default> Default for GameStateWithConsole<I, S> {
    fn default() -> Self {
        Self {
            game: GameState::new(),
            console: S::default(),
        }
    }
}

impl<I: ConsoleInput, S: Default> GameStateWithConsole<I, S> {
    pub fn new() -> Self {
        Self::default()
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
            in_init: true,
            rng_state: 0,
            input_prev: [I::default(); MAX_PLAYERS],
            input_curr: [I::default(); MAX_PLAYERS],
            save_data: Default::default(),
            quit_requested: false,
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
