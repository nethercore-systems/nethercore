//! Game state types
//!
//! Provides the main game state structure that holds all per-game mutable state.

use glam::Mat4;
use wasmtime::Memory;

use super::camera::CameraState;
use super::draw::{DrawCommand, PendingMesh, PendingTexture};
use super::input::InputState;
use super::render::{InitConfig, RenderState};

/// Maximum transform stack depth
pub const MAX_TRANSFORM_STACK: usize = 16;

/// Maximum number of players
pub const MAX_PLAYERS: usize = 4;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 8;

/// Maximum save data size per slot (64KB)
pub const MAX_SAVE_SIZE: usize = 64 * 1024;

/// Per-game state stored in the wasmtime Store
///
/// Contains all mutable state for a single game instance, including
/// FFI context, input state, and render state.
///
/// # Resource Lifecycle
///
/// This struct owns transient game state. Resources created during a game session:
///
/// - **Pending Textures/Meshes**: Filled by FFI calls during `init()`, consumed by the
///   graphics backend to create GPU resources. These Vec collections are cleared after
///   the graphics backend processes them.
///
/// - **Draw Commands**: Filled during `render()`, consumed by graphics backend each frame,
///   then cleared.
///
/// - **Save Data**: Persisted in memory during the session. Save/load operations are
///   synchronous. Data is lost when GameState is dropped (persistent storage handled
///   by the platform layer).
///
/// When a `GameInstance` is dropped, this `GameState` is dropped along with the
/// wasmtime `Store`, releasing all memory. GPU resources (textures, meshes) live
/// separately in the graphics backend and are managed by its cleanup strategy.
pub struct GameState {
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

    /// Transform stack
    pub transform_stack: Vec<Mat4>,

    /// Current transform matrix
    pub current_transform: Mat4,

    /// Camera state
    pub camera: CameraState,

    /// RNG state for deterministic random
    pub rng_state: u64,

    /// Current render state
    pub render_state: RenderState,

    /// Init-time configuration (locked after init completes)
    pub init_config: InitConfig,

    /// Input state for all players (previous and current frame)
    pub input_prev: [InputState; MAX_PLAYERS],
    pub input_curr: [InputState; MAX_PLAYERS],

    /// Save data slots (8 slots × 64KB max each)
    pub save_data: [Option<Vec<u8>>; MAX_SAVE_SLOTS],

    /// Quit requested by game
    pub quit_requested: bool,

    /// Next texture handle to allocate
    pub next_texture_handle: u32,

    /// Pending texture loads (filled by FFI, consumed by graphics backend)
    pub pending_textures: Vec<PendingTexture>,

    /// Next mesh handle to allocate
    pub next_mesh_handle: u32,

    /// Pending mesh loads (filled by FFI, consumed by graphics backend)
    pub pending_meshes: Vec<PendingMesh>,

    /// Draw commands for current frame (filled by FFI, consumed by graphics backend)
    pub draw_commands: Vec<DrawCommand>,
}

impl GameState {
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
            transform_stack: Vec::with_capacity(MAX_TRANSFORM_STACK),
            current_transform: Mat4::IDENTITY,
            camera: CameraState::default(),
            rng_state: 0,
            render_state: RenderState::default(),
            init_config: InitConfig::default(),
            input_prev: [InputState::default(); MAX_PLAYERS],
            input_curr: [InputState::default(); MAX_PLAYERS],
            save_data: Default::default(),
            quit_requested: false,
            next_texture_handle: 1, // 0 is reserved for invalid/unbound
            pending_textures: Vec::new(),
            next_mesh_handle: 1, // 0 is reserved for invalid
            pending_meshes: Vec::new(),
            draw_commands: Vec::new(),
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

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_new() {
        let state = GameState::new();
        assert!(state.memory.is_none());
        assert_eq!(state.tick_count, 0);
        assert_eq!(state.elapsed_time, 0.0);
        assert_eq!(state.delta_time, 0.0);
        assert_eq!(state.player_count, 1);
        assert_eq!(state.local_player_mask, 1);
        assert!(state.in_init);
        assert!(state.transform_stack.is_empty());
        assert_eq!(state.current_transform, Mat4::IDENTITY);
        assert!(!state.quit_requested);
        assert_eq!(state.next_texture_handle, 1);
        assert_eq!(state.next_mesh_handle, 1);
    }

    #[test]
    fn test_game_state_default() {
        let state1 = GameState::new();
        let state2 = GameState::default();
        // Both should have same initial values
        assert_eq!(state1.tick_count, state2.tick_count);
        assert_eq!(state1.player_count, state2.player_count);
    }

    #[test]
    fn test_game_state_transform_stack_capacity() {
        let state = GameState::new();
        assert!(state.transform_stack.capacity() >= MAX_TRANSFORM_STACK);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_TRANSFORM_STACK, 16);
        assert_eq!(MAX_PLAYERS, 4);
        assert_eq!(MAX_SAVE_SLOTS, 8);
        assert_eq!(MAX_SAVE_SIZE, 64 * 1024);
    }
}
