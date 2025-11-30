//! WASM runtime wrapper
//!
//! Provides abstractions over wasmtime for loading and executing game WASM modules.

use anyhow::{Context, Result};
use glam::{Mat4, Vec3};
use wasmtime::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};

/// Maximum transform stack depth
pub const MAX_TRANSFORM_STACK: usize = 16;

/// Maximum number of players
pub const MAX_PLAYERS: usize = 4;

/// Default camera field of view in degrees
pub const DEFAULT_CAMERA_FOV: f32 = 60.0;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 8;

/// Maximum save data size per slot (64KB)
pub const MAX_SAVE_SIZE: usize = 64 * 1024;

/// Pending texture load request
#[derive(Debug, Clone)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Pending mesh load request (retained mode)
#[derive(Debug, Clone)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u32>>,
}

/// Draw command for immediate mode drawing
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw triangles with immediate data (non-indexed)
    DrawTriangles {
        format: u8,
        vertex_data: Vec<f32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw indexed triangles with immediate data
    DrawTrianglesIndexed {
        format: u8,
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a retained mesh by handle
    DrawMesh {
        handle: u32,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a billboard (camera-facing quad)
    DrawBillboard {
        /// Billboard width
        width: f32,
        /// Billboard height
        height: f32,
        /// Billboard mode (1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z)
        mode: u8,
        /// Source UV rectangle (x, y, w, h) - None for full texture (0,0,1,1)
        uv_rect: Option<(f32, f32, f32, f32)>,
        /// World position from transform
        transform: Mat4,
        /// Color tint
        color: u32,
        /// Depth test enabled
        depth_test: bool,
        /// Cull mode
        cull_mode: u8,
        /// Blend mode
        blend_mode: u8,
        /// Bound textures
        bound_textures: [u32; 4],
    },
    /// Draw a 2D sprite in screen space
    DrawSprite {
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Sprite width (pixels)
        width: f32,
        /// Sprite height (pixels)
        height: f32,
        /// Source UV rectangle (x, y, w, h) - None for full texture (0,0,1,1)
        uv_rect: Option<(f32, f32, f32, f32)>,
        /// Origin offset for rotation (x, y in pixels, 0,0 = top-left)
        origin: Option<(f32, f32)>,
        /// Rotation angle in degrees (clockwise)
        rotation: f32,
        /// Color tint
        color: u32,
        /// Blend mode
        blend_mode: u8,
        /// Bound textures
        bound_textures: [u32; 4],
    },
    /// Draw a 2D rectangle in screen space
    DrawRect {
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Rectangle width (pixels)
        width: f32,
        /// Rectangle height (pixels)
        height: f32,
        /// Fill color
        color: u32,
        /// Blend mode
        blend_mode: u8,
    },
    /// Draw text in screen space
    DrawText {
        /// UTF-8 text string
        text: String,
        /// Screen X coordinate (pixels, 0 = left)
        x: f32,
        /// Screen Y coordinate (pixels, 0 = top)
        y: f32,
        /// Font size (pixels)
        size: f32,
        /// Text color
        color: u32,
        /// Blend mode
        blend_mode: u8,
    },
    /// Set procedural sky parameters
    SetSky {
        /// Horizon color (RGB, linear)
        horizon_color: [f32; 3],
        /// Zenith (top) color (RGB, linear)
        zenith_color: [f32; 3],
        /// Sun direction (will be normalized)
        sun_direction: [f32; 3],
        /// Sun color (RGB, linear)
        sun_color: [f32; 3],
        /// Sun sharpness (higher = sharper sun, typically 32-256)
        sun_sharpness: f32,
    },
}

/// Shared WASM engine (one per application)
pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    /// Create a new WASM engine with default configuration
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Get a reference to the underlying wasmtime engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Load a WASM module from bytes
    pub fn load_module(&self, bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, bytes).context("Failed to compile WASM module")
    }
}

// NOTE: WasmEngine intentionally does not implement Default.
// The WASM engine initialization is fallible (wasmtime::Engine::default() can fail
// on unsupported platforms or with invalid configuration). Using WasmEngine::new()
// returns Result<Self> which properly propagates initialization errors.

/// Per-game state stored in the wasmtime Store
///
/// Contains all mutable state for a single game instance, including
/// FFI context, input state, and render state.
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

/// Light state for Mode 2/3 (PBR/Hybrid)
#[derive(Debug, Clone, Copy)]
pub struct LightState {
    /// Light enabled
    pub enabled: bool,
    /// Light direction (normalized)
    pub direction: [f32; 3],
    /// Light color (RGB, linear)
    pub color: [f32; 3],
    /// Light intensity multiplier
    pub intensity: f32,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: [0.0, -1.0, 0.0],  // Default: downward
            color: [1.0, 1.0, 1.0],       // Default: white
            intensity: 1.0,
        }
    }
}

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// Current render state for batching
#[derive(Debug, Clone)]
pub struct RenderState {
    /// Uniform color tint (RGBA)
    pub color: u32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Cull mode: 0=none, 1=back, 2=front
    pub cull_mode: u8,
    /// Blend mode: 0=none, 1=alpha, 2=additive, 3=multiply
    pub blend_mode: u8,
    /// Texture filter: 0=nearest, 1=linear
    pub texture_filter: u8,
    /// Bound texture handles per slot
    pub bound_textures: [u32; 4],
    /// Current render mode (0-3)
    pub render_mode: u8,
    /// Material metallic value (0.0-1.0, default 0.0)
    pub material_metallic: f32,
    /// Material roughness value (0.0-1.0, default 0.5)
    pub material_roughness: f32,
    /// Material emissive intensity (default 0.0)
    pub material_emissive: f32,
    /// Light states for Mode 2/3 (4 lights)
    pub lights: [LightState; 4],
    /// Bone transform matrices for GPU skinning (column-major, up to 256 bones)
    pub bone_matrices: Vec<Mat4>,
    /// Number of active bones
    pub bone_count: u32,
}

/// Configuration set during init (immutable after init)
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Resolution index (0-3 for Z: 360p, 540p, 720p, 1080p)
    pub resolution_index: u32,
    /// Tick rate index (0-3 for Z: 24, 30, 60, 120 fps)
    pub tick_rate_index: u32,
    /// Clear/background color (RGBA: 0xRRGGBBAA)
    pub clear_color: u32,
    /// Render mode (0-3: Unlit, Matcap, PBR, Hybrid)
    pub render_mode: u8,
    /// Whether any config was changed during init
    pub modified: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            resolution_index: 1,  // Default 540p
            tick_rate_index: 2,   // Default 60 fps
            clear_color: 0x000000FF, // Black, fully opaque
            render_mode: 0,       // Unlit
            modified: false,
        }
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 1, // Back-face culling by default
            blend_mode: 0,
            texture_filter: 0, // Nearest by default (retro look)
            bound_textures: [0; 4],
            render_mode: 0,
            material_metallic: 0.0,
            material_roughness: 0.5,
            material_emissive: 0.0,
            lights: [LightState::default(); 4],
            bone_matrices: Vec::new(),
            bone_count: 0,
        }
    }
}

/// Input state for a single player
#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    /// Button bitmask
    pub buttons: u16,
    /// Left stick X (-128 to 127)
    pub left_stick_x: i8,
    /// Left stick Y (-128 to 127)
    pub left_stick_y: i8,
    /// Right stick X (-128 to 127)
    pub right_stick_x: i8,
    /// Right stick Y (-128 to 127)
    pub right_stick_y: i8,
    /// Left trigger (0-255)
    pub left_trigger: u8,
    /// Right trigger (0-255)
    pub right_trigger: u8,
}

/// Camera state for 3D rendering
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target (look-at point) in world space
    pub target: Vec3,
    /// Field of view in degrees
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            fov: DEFAULT_CAMERA_FOV,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraState {
    /// Compute the view matrix (world-to-camera transform)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    /// Compute the projection matrix for a given aspect ratio
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), aspect_ratio, self.near, self.far)
    }

    /// Compute the combined view-projection matrix
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }
}

/// A loaded and instantiated game
pub struct GameInstance {
    store: Store<GameState>,
    /// The WASM instance.
    /// Not directly used after initialization, but must be kept alive to maintain
    /// the lifetime of exported functions and memory references.
    #[allow(dead_code)]
    instance: Instance,
    init_fn: Option<TypedFunc<(), ()>>,
    update_fn: Option<TypedFunc<(), ()>>,
    render_fn: Option<TypedFunc<(), ()>>,
    save_state_fn: Option<TypedFunc<(u32, u32), u32>>,
    load_state_fn: Option<TypedFunc<(u32, u32), ()>>,
}

impl GameInstance {
    /// Create a new game instance from a module
    pub fn new(engine: &WasmEngine, module: &Module, linker: &Linker<GameState>) -> Result<Self> {
        let mut store = Store::new(engine.engine(), GameState::new());
        let instance = linker
            .instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        // Get the memory export
        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            store.data_mut().memory = Some(memory);
        }

        // Look up exported functions
        let init_fn = instance
            .get_typed_func::<(), ()>(&mut store, "init")
            .ok();
        let update_fn = instance
            .get_typed_func::<(), ()>(&mut store, "update")
            .ok();
        let render_fn = instance
            .get_typed_func::<(), ()>(&mut store, "render")
            .ok();
        let save_state_fn = instance
            .get_typed_func::<(u32, u32), u32>(&mut store, "save_state")
            .ok();
        let load_state_fn = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "load_state")
            .ok();

        Ok(Self {
            store,
            instance,
            init_fn,
            update_fn,
            render_fn,
            save_state_fn,
            load_state_fn,
        })
    }

    /// Call the game's init function
    pub fn init(&mut self) -> Result<()> {
        self.store.data_mut().in_init = true;
        if let Some(init) = &self.init_fn {
            init.call(&mut self.store, ())
                .context("Failed to call init()")?;
        }
        self.store.data_mut().in_init = false;
        Ok(())
    }

    /// Call the game's update function
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        {
            let state = self.store.data_mut();
            state.delta_time = delta_time;
            state.elapsed_time += delta_time;
            state.tick_count += 1;
        }
        if let Some(update) = &self.update_fn {
            update
                .call(&mut self.store, ())
                .context("Failed to call update()")?;
        }
        // Rotate input state
        let state = self.store.data_mut();
        state.input_prev = state.input_curr;
        Ok(())
    }

    /// Call the game's render function
    pub fn render(&mut self) -> Result<()> {
        if let Some(render) = &self.render_fn {
            render
                .call(&mut self.store, ())
                .context("Failed to call render()")?;
        }
        Ok(())
    }

    /// Save game state to a buffer
    pub fn save_state(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(save_state) = &self.save_state_fn {
            let memory = self
                .store
                .data()
                .memory
                .context("No memory export found")?;
            let ptr = 0u32; // Use start of memory for now (games should allocate)
            let max_len = buffer.len() as u32;

            let len = save_state
                .call(&mut self.store, (ptr, max_len))
                .context("Failed to call save_state()")?;

            let mem_data = memory.data(&self.store);
            let len = len as usize;
            if len <= buffer.len() && (ptr as usize + len) <= mem_data.len() {
                buffer[..len].copy_from_slice(&mem_data[ptr as usize..ptr as usize + len]);
                Ok(len)
            } else {
                anyhow::bail!("save_state returned invalid length")
            }
        } else {
            Ok(0)
        }
    }

    /// Load game state from a buffer
    pub fn load_state(&mut self, buffer: &[u8]) -> Result<()> {
        if let Some(load_state) = &self.load_state_fn {
            let memory = self
                .store
                .data()
                .memory
                .context("No memory export found")?;
            let ptr = 0u32;
            let len = buffer.len() as u32;

            // Copy buffer into WASM memory
            let mem_data = memory.data_mut(&mut self.store);
            if (ptr as usize + buffer.len()) <= mem_data.len() {
                mem_data[ptr as usize..ptr as usize + buffer.len()].copy_from_slice(buffer);
            } else {
                anyhow::bail!("Buffer too large for WASM memory");
            }

            load_state
                .call(&mut self.store, (ptr, len))
                .context("Failed to call load_state()")?;
        }
        Ok(())
    }

    /// Get mutable reference to the store
    pub fn store_mut(&mut self) -> &mut Store<GameState> {
        &mut self.store
    }

    /// Get reference to the store
    pub fn store(&self) -> &Store<GameState> {
        &self.store
    }

    /// Get mutable reference to game state
    pub fn state_mut(&mut self) -> &mut GameState {
        self.store.data_mut()
    }

    /// Get reference to game state
    pub fn state(&self) -> &GameState {
        self.store.data()
    }

    /// Set input for a player
    pub fn set_input(&mut self, player: usize, input: InputState) {
        if player < MAX_PLAYERS {
            self.store.data_mut().input_curr[player] = input;
        }
    }

    /// Configure the session's player count and local player mask
    ///
    /// This should be called before `init()` to set up multiplayer state.
    /// The game can query these values via the `player_count()` and
    /// `local_player_mask()` FFI functions.
    ///
    /// # Arguments
    /// * `player_count` - Number of players in session (1-4)
    /// * `local_player_mask` - Bitmask of local players (bit N = player N is local)
    ///
    /// # Example
    /// ```ignore
    /// // 2 players, only player 0 is local (standard online play)
    /// game.configure_session(2, 0b0001);
    ///
    /// // 4 players, players 0 and 1 are local (2 local + 2 remote)
    /// game.configure_session(4, 0b0011);
    /// ```
    pub fn configure_session(&mut self, player_count: u32, local_player_mask: u32) {
        let state = self.store.data_mut();
        state.player_count = player_count.min(MAX_PLAYERS as u32);
        state.local_player_mask = local_player_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // ============================================================================
    // WasmEngine Tests
    // ============================================================================

    #[test]
    fn test_wasm_engine_creation() {
        let engine = WasmEngine::new();
        assert!(engine.is_ok());
    }

    // NOTE: WasmEngine does not implement Default because engine initialization
    // is fallible. Use WasmEngine::new() which returns Result<Self>.

    #[test]
    fn test_wasm_engine_load_invalid_module() {
        let engine = WasmEngine::new().unwrap();
        let result = engine.load_module(b"not valid wasm");
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_engine_load_valid_module() {
        let engine = WasmEngine::new().unwrap();
        // Minimal valid WASM module (empty module)
        let wasm = wat::parse_str("(module)").unwrap();
        let result = engine.load_module(&wasm);
        assert!(result.is_ok());
    }

    // ============================================================================
    // GameState Tests
    // ============================================================================

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

    // ============================================================================
    // CameraState Tests
    // ============================================================================

    #[test]
    fn test_camera_state_default() {
        let camera = CameraState::default();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.fov, DEFAULT_CAMERA_FOV);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
    }

    #[test]
    fn test_camera_state_view_matrix_identity_position() {
        let camera = CameraState {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        let view = camera.view_matrix();
        // View matrix should transform world origin to be in front of camera
        let world_origin = Vec3::ZERO;
        let view_space = view.transform_point3(world_origin);
        // Origin should be at z=-1 in view space (1 unit in front of camera)
        assert!((view_space.z - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_camera_state_view_matrix_translation() {
        let camera = CameraState {
            position: Vec3::new(10.0, 0.0, 0.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        let view = camera.view_matrix();
        // Target should be transformed to be in front of camera
        let target_view_space = view.transform_point3(camera.target);
        // Target should be at negative Z (in front of camera)
        assert!(target_view_space.z < 0.0);
    }

    #[test]
    fn test_camera_state_projection_matrix_aspect_ratio() {
        let camera = CameraState::default();
        let proj_16_9 = camera.projection_matrix(16.0 / 9.0);
        let proj_4_3 = camera.projection_matrix(4.0 / 3.0);
        // Different aspect ratios should produce different matrices
        assert_ne!(proj_16_9, proj_4_3);
    }

    #[test]
    fn test_camera_state_projection_matrix_fov() {
        let camera_narrow = CameraState {
            fov: 45.0,
            ..Default::default()
        };
        let camera_wide = CameraState {
            fov: 90.0,
            ..Default::default()
        };
        let proj_narrow = camera_narrow.projection_matrix(1.0);
        let proj_wide = camera_wide.projection_matrix(1.0);
        // Different FOV should produce different matrices
        assert_ne!(proj_narrow, proj_wide);
    }

    #[test]
    fn test_camera_state_view_projection_matrix() {
        let camera = CameraState::default();
        let aspect = 16.0 / 9.0;
        let vp = camera.view_projection_matrix(aspect);
        let expected = camera.projection_matrix(aspect) * camera.view_matrix();
        assert_eq!(vp, expected);
    }

    // ============================================================================
    // InputState Tests
    // ============================================================================

    #[test]
    fn test_input_state_default() {
        let input = InputState::default();
        assert_eq!(input.buttons, 0);
        assert_eq!(input.left_stick_x, 0);
        assert_eq!(input.left_stick_y, 0);
        assert_eq!(input.right_stick_x, 0);
        assert_eq!(input.right_stick_y, 0);
        assert_eq!(input.left_trigger, 0);
        assert_eq!(input.right_trigger, 0);
    }

    #[test]
    fn test_input_state_full_values() {
        let input = InputState {
            buttons: 0xFFFF,
            left_stick_x: 127,
            left_stick_y: -128,
            right_stick_x: 100,
            right_stick_y: -100,
            left_trigger: 255,
            right_trigger: 128,
        };
        assert_eq!(input.buttons, 0xFFFF);
        assert_eq!(input.left_stick_x, 127);
        assert_eq!(input.left_stick_y, -128);
        assert_eq!(input.right_trigger, 128);
    }

    #[test]
    fn test_input_state_bytemuck_roundtrip() {
        let original = InputState {
            buttons: 0x1234,
            left_stick_x: 50,
            left_stick_y: -75,
            right_stick_x: 25,
            right_stick_y: -25,
            left_trigger: 200,
            right_trigger: 100,
        };

        // InputState should be Copy + Clone
        let copied = original;
        assert_eq!(copied.buttons, original.buttons);
        assert_eq!(copied.left_stick_x, original.left_stick_x);
        assert_eq!(copied.left_trigger, original.left_trigger);
    }

    // ============================================================================
    // RenderState Tests
    // ============================================================================

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF); // White, fully opaque
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, 1); // Back-face culling
        assert_eq!(state.blend_mode, 0); // No blending
        assert_eq!(state.texture_filter, 0); // Nearest
        assert_eq!(state.bound_textures, [0; 4]);
        assert_eq!(state.render_mode, 0); // Unlit
        assert_eq!(state.material_metallic, 0.0);
        assert_eq!(state.material_roughness, 0.5);
        assert_eq!(state.material_emissive, 0.0);
        assert_eq!(state.bone_count, 0);
        assert!(state.bone_matrices.is_empty());
    }

    #[test]
    fn test_render_state_lights_default() {
        let state = RenderState::default();
        for light in &state.lights {
            assert!(!light.enabled);
            assert_eq!(light.color, [1.0, 1.0, 1.0]);
            assert_eq!(light.intensity, 1.0);
        }
    }

    // ============================================================================
    // LightState Tests
    // ============================================================================

    #[test]
    fn test_light_state_default() {
        let light = LightState::default();
        assert!(!light.enabled);
        assert_eq!(light.direction, [0.0, -1.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.intensity, 1.0);
    }

    // ============================================================================
    // InitConfig Tests
    // ============================================================================

    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.resolution_index, 1); // 540p
        assert_eq!(config.tick_rate_index, 2); // 60fps
        assert_eq!(config.clear_color, 0x000000FF); // Black, opaque
        assert_eq!(config.render_mode, 0); // Unlit
        assert!(!config.modified);
    }

    // ============================================================================
    // PendingTexture Tests
    // ============================================================================

    #[test]
    fn test_pending_texture() {
        let texture = PendingTexture {
            handle: 1,
            width: 64,
            height: 64,
            data: vec![0xFF; 64 * 64 * 4],
        };
        assert_eq!(texture.handle, 1);
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
        assert_eq!(texture.data.len(), 64 * 64 * 4);
    }

    // ============================================================================
    // PendingMesh Tests
    // ============================================================================

    #[test]
    fn test_pending_mesh_non_indexed() {
        let mesh = PendingMesh {
            handle: 1,
            format: 0, // POS only
            vertex_data: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            index_data: None,
        };
        assert_eq!(mesh.handle, 1);
        assert_eq!(mesh.format, 0);
        assert_eq!(mesh.vertex_data.len(), 9); // 3 vertices * 3 floats
        assert!(mesh.index_data.is_none());
    }

    #[test]
    fn test_pending_mesh_indexed() {
        let mesh = PendingMesh {
            handle: 2,
            format: 1, // POS_UV
            vertex_data: vec![
                0.0, 0.0, 0.0, 0.0, 0.0, // v0: pos + uv
                1.0, 0.0, 0.0, 1.0, 0.0, // v1: pos + uv
                0.0, 1.0, 0.0, 0.0, 1.0, // v2: pos + uv
            ],
            index_data: Some(vec![0, 1, 2]),
        };
        assert_eq!(mesh.handle, 2);
        assert_eq!(mesh.format, 1);
        assert_eq!(mesh.vertex_data.len(), 15); // 3 vertices * 5 floats
        assert_eq!(mesh.index_data, Some(vec![0, 1, 2]));
    }

    // ============================================================================
    // DrawCommand Tests
    // ============================================================================

    #[test]
    fn test_draw_command_triangles() {
        let cmd = DrawCommand::DrawTriangles {
            format: 2,
            vertex_data: vec![0.0; 24], // 3 verts * 6 floats (pos + color)
            transform: Mat4::IDENTITY,
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 1,
            blend_mode: 0,
            bound_textures: [0; 4],
        };
        match cmd {
            DrawCommand::DrawTriangles { format, .. } => assert_eq!(format, 2),
            _ => panic!("Expected DrawTriangles"),
        }
    }

    #[test]
    fn test_draw_command_mesh() {
        let cmd = DrawCommand::DrawMesh {
            handle: 5,
            transform: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: 0,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawMesh { handle, depth_test, .. } => {
                assert_eq!(handle, 5);
                assert!(!depth_test);
            }
            _ => panic!("Expected DrawMesh"),
        }
    }

    #[test]
    fn test_draw_command_billboard() {
        let cmd = DrawCommand::DrawBillboard {
            width: 2.0,
            height: 3.0,
            mode: 1, // Spherical
            uv_rect: Some((0.0, 0.0, 0.5, 0.5)),
            transform: Mat4::IDENTITY,
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 0,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawBillboard { width, height, mode, uv_rect, .. } => {
                assert_eq!(width, 2.0);
                assert_eq!(height, 3.0);
                assert_eq!(mode, 1);
                assert_eq!(uv_rect, Some((0.0, 0.0, 0.5, 0.5)));
            }
            _ => panic!("Expected DrawBillboard"),
        }
    }

    #[test]
    fn test_draw_command_sprite() {
        let cmd = DrawCommand::DrawSprite {
            x: 100.0,
            y: 50.0,
            width: 64.0,
            height: 64.0,
            uv_rect: None,
            origin: Some((32.0, 32.0)),
            rotation: 45.0,
            color: 0xFFFFFFFF,
            blend_mode: 1,
            bound_textures: [1, 0, 0, 0],
        };
        match cmd {
            DrawCommand::DrawSprite { x, y, rotation, origin, .. } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 50.0);
                assert_eq!(rotation, 45.0);
                assert_eq!(origin, Some((32.0, 32.0)));
            }
            _ => panic!("Expected DrawSprite"),
        }
    }

    #[test]
    fn test_draw_command_rect() {
        let cmd = DrawCommand::DrawRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            color: 0x00FF00FF,
            blend_mode: 0,
        };
        match cmd {
            DrawCommand::DrawRect { width, height, color, .. } => {
                assert_eq!(width, 100.0);
                assert_eq!(height, 50.0);
                assert_eq!(color, 0x00FF00FF);
            }
            _ => panic!("Expected DrawRect"),
        }
    }

    #[test]
    fn test_draw_command_text() {
        let cmd = DrawCommand::DrawText {
            text: "Hello World".to_string(),
            x: 10.0,
            y: 20.0,
            size: 16.0,
            color: 0xFFFFFFFF,
            blend_mode: 1,
        };
        match cmd {
            DrawCommand::DrawText { text, size, .. } => {
                assert_eq!(text, "Hello World");
                assert_eq!(size, 16.0);
            }
            _ => panic!("Expected DrawText"),
        }
    }

    #[test]
    fn test_draw_command_set_sky() {
        let cmd = DrawCommand::SetSky {
            horizon_color: [0.5, 0.7, 1.0],
            zenith_color: [0.1, 0.2, 0.8],
            sun_direction: [0.5, 0.5, 0.5],
            sun_color: [1.0, 0.9, 0.8],
            sun_sharpness: 64.0,
        };
        match cmd {
            DrawCommand::SetSky { horizon_color, sun_sharpness, .. } => {
                assert_eq!(horizon_color, [0.5, 0.7, 1.0]);
                assert_eq!(sun_sharpness, 64.0);
            }
            _ => panic!("Expected SetSky"),
        }
    }

    // ============================================================================
    // GameInstance Integration Tests (require WASM modules)
    // ============================================================================

    #[test]
    fn test_game_instance_creation_empty_module() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let result = GameInstance::new(&engine, &module, &linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_game_instance_with_init_function() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "init"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let result = game.init();
        assert!(result.is_ok());
        // in_init should be false after init completes
        assert!(!game.state().in_init);
    }

    #[test]
    fn test_game_instance_with_update_function() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "update"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let delta = 1.0 / 60.0;
        let result = game.update(delta);
        assert!(result.is_ok());
        assert_eq!(game.state().tick_count, 1);
        assert!((game.state().delta_time - delta).abs() < 0.0001);
    }

    #[test]
    fn test_game_instance_update_increments_tick() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "update"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        for i in 1..=5 {
            game.update(1.0 / 60.0).unwrap();
            assert_eq!(game.state().tick_count, i);
        }
    }

    #[test]
    fn test_game_instance_update_accumulates_elapsed_time() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "update"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let delta = 0.016; // ~60fps

        game.update(delta).unwrap();
        game.update(delta).unwrap();
        game.update(delta).unwrap();

        assert!((game.state().elapsed_time - delta * 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_game_instance_with_render_function() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "render"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let result = game.render();
        assert!(result.is_ok());
    }

    #[test]
    fn test_game_instance_set_input() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        let input = InputState {
            buttons: 0x00FF,
            left_stick_x: 100,
            left_stick_y: -50,
            right_stick_x: 25,
            right_stick_y: -25,
            left_trigger: 200,
            right_trigger: 100,
        };

        game.set_input(0, input);
        assert_eq!(game.state().input_curr[0].buttons, 0x00FF);
        assert_eq!(game.state().input_curr[0].left_stick_x, 100);
        assert_eq!(game.state().input_curr[0].left_trigger, 200);
    }

    #[test]
    fn test_game_instance_set_input_invalid_player() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Should not panic for invalid player index
        game.set_input(10, InputState::default());
    }

    #[test]
    fn test_game_instance_input_rotation() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "update"))
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Set input for player 0
        let input1 = InputState {
            buttons: 0x0001,
            ..Default::default()
        };
        game.set_input(0, input1);

        // Call update (which rotates input_prev = input_curr)
        game.update(1.0 / 60.0).unwrap();

        // Previous should now have our input
        assert_eq!(game.state().input_prev[0].buttons, 0x0001);

        // Set new input
        let input2 = InputState {
            buttons: 0x0002,
            ..Default::default()
        };
        game.set_input(0, input2);

        // Current should have new input
        assert_eq!(game.state().input_curr[0].buttons, 0x0002);
    }

    #[test]
    fn test_game_instance_store_access() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Test mutable access
        game.state_mut().player_count = 4;
        assert_eq!(game.state().player_count, 4);

        // Test store access
        let _store = game.store();
        let _store_mut = game.store_mut();
    }

    #[test]
    fn test_game_instance_save_state_no_function() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let mut buffer = vec![0u8; 1024];

        // Should return Ok(0) when save_state is not exported
        let result = game.save_state(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_game_instance_load_state_no_function() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Should return Ok when load_state is not exported
        let result = game.load_state(&[1, 2, 3, 4]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_game_instance_configure_session() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Default values
        assert_eq!(game.state().player_count, 1);
        assert_eq!(game.state().local_player_mask, 1);

        // Configure for 4 players, only player 0 is local
        game.configure_session(4, 0b0001);
        assert_eq!(game.state().player_count, 4);
        assert_eq!(game.state().local_player_mask, 0b0001);

        // Configure for 2 players, both local
        game.configure_session(2, 0b0011);
        assert_eq!(game.state().player_count, 2);
        assert_eq!(game.state().local_player_mask, 0b0011);
    }

    #[test]
    fn test_game_instance_configure_session_clamps_players() {
        let engine = WasmEngine::new().unwrap();
        let wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
            )
        "#).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Try to set more than MAX_PLAYERS
        game.configure_session(100, 0xFFFF);
        assert_eq!(game.state().player_count, 4); // Clamped to MAX_PLAYERS
    }

    // ============================================================================
    // Transform Matrix Tests
    // ============================================================================

    #[test]
    fn test_transform_identity() {
        let transform = Mat4::IDENTITY;
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = transform.transform_point3(point);
        assert_eq!(transformed, point);
    }

    #[test]
    fn test_transform_translation() {
        let transform = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let point = Vec3::ZERO;
        let transformed = transform.transform_point3(point);
        assert!((transformed.x - 10.0).abs() < 0.0001);
        assert!((transformed.y - 20.0).abs() < 0.0001);
        assert!((transformed.z - 30.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_scale() {
        let transform = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let point = Vec3::new(1.0, 1.0, 1.0);
        let transformed = transform.transform_point3(point);
        assert!((transformed.x - 2.0).abs() < 0.0001);
        assert!((transformed.y - 3.0).abs() < 0.0001);
        assert!((transformed.z - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_rotation_90_deg_y() {
        let transform = Mat4::from_rotation_y(PI / 2.0);
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point3(point);
        // Rotating (1, 0, 0) 90° around Y should give (0, 0, -1)
        assert!(transformed.x.abs() < 0.0001);
        assert!(transformed.y.abs() < 0.0001);
        assert!((transformed.z - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_transform_combination() {
        // Scale, then rotate, then translate
        let scale = Mat4::from_scale(Vec3::splat(2.0));
        let rotate = Mat4::from_rotation_z(PI / 2.0);
        let translate = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));

        // Combined transform (applied right-to-left)
        let transform = translate * rotate * scale;

        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point3(point);

        // (1, 0, 0) * 2 = (2, 0, 0)
        // Rotate 90° Z: (0, 2, 0)
        // Translate: (5, 2, 0)
        assert!((transformed.x - 5.0).abs() < 0.0001);
        assert!((transformed.y - 2.0).abs() < 0.0001);
        assert!(transformed.z.abs() < 0.0001);
    }

    // ============================================================================
    // Constants Tests
    // ============================================================================

    #[test]
    fn test_constants() {
        assert_eq!(MAX_TRANSFORM_STACK, 16);
        assert_eq!(MAX_PLAYERS, 4);
        assert_eq!(DEFAULT_CAMERA_FOV, 60.0);
        assert_eq!(MAX_SAVE_SLOTS, 8);
        assert_eq!(MAX_SAVE_SIZE, 64 * 1024);
        assert_eq!(MAX_BONES, 256);
    }

    // ============================================================================
    // GPU Skinning Tests
    // ============================================================================

    #[test]
    fn test_render_state_bone_matrices_empty_by_default() {
        let state = RenderState::default();
        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_can_store_bones() {
        let mut state = RenderState::default();

        // Add some bone matrices
        let bone1 = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let bone2 = Mat4::from_rotation_y(PI / 4.0);
        let bone3 = Mat4::from_scale(Vec3::splat(2.0));

        state.bone_matrices.push(bone1);
        state.bone_matrices.push(bone2);
        state.bone_matrices.push(bone3);
        state.bone_count = 3;

        assert_eq!(state.bone_matrices.len(), 3);
        assert_eq!(state.bone_count, 3);

        // Verify matrices are stored correctly
        assert_eq!(state.bone_matrices[0], bone1);
        assert_eq!(state.bone_matrices[1], bone2);
        assert_eq!(state.bone_matrices[2], bone3);
    }

    #[test]
    fn test_render_state_bone_matrices_max_capacity() {
        let mut state = RenderState::default();

        // Add MAX_BONES matrices
        for i in 0..MAX_BONES {
            let translation = Vec3::new(i as f32, 0.0, 0.0);
            state.bone_matrices.push(Mat4::from_translation(translation));
        }
        state.bone_count = MAX_BONES as u32;

        assert_eq!(state.bone_matrices.len(), MAX_BONES);
        assert_eq!(state.bone_count, MAX_BONES as u32);

        // Verify first and last bones
        let expected_first = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let expected_last = Mat4::from_translation(Vec3::new((MAX_BONES - 1) as f32, 0.0, 0.0));
        assert_eq!(state.bone_matrices[0], expected_first);
        assert_eq!(state.bone_matrices[MAX_BONES - 1], expected_last);
    }

    #[test]
    fn test_render_state_bone_matrices_clear() {
        let mut state = RenderState::default();

        // Add bones
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_count = 2;

        // Clear bones
        state.bone_matrices.clear();
        state.bone_count = 0;

        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_replace() {
        let mut state = RenderState::default();

        // Add initial bones
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_count = 2;

        // Replace with new bones
        let new_bone = Mat4::from_translation(Vec3::new(5.0, 5.0, 5.0));
        state.bone_matrices.clear();
        state.bone_matrices.push(new_bone);
        state.bone_count = 1;

        assert_eq!(state.bone_matrices.len(), 1);
        assert_eq!(state.bone_count, 1);
        assert_eq!(state.bone_matrices[0], new_bone);
    }

    #[test]
    fn test_render_state_bone_matrix_identity_transform() {
        // Verify identity matrix doesn't transform a vertex
        let identity = Mat4::IDENTITY;
        let vertex = Vec3::new(1.0, 2.0, 3.0);
        let transformed = identity.transform_point3(vertex);

        assert_eq!(transformed, vertex);
    }

    #[test]
    fn test_render_state_bone_matrix_weighted_blend() {
        // Simulate GPU skinning blend: position = sum(weight_i * bone_i * position)
        let bone1 = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let bone2 = Mat4::from_translation(Vec3::new(0.0, 10.0, 0.0));

        let vertex = Vec3::ZERO;
        let weight1 = 0.5f32;
        let weight2 = 0.5f32;

        // Transform by each bone
        let t1 = bone1.transform_point3(vertex);
        let t2 = bone2.transform_point3(vertex);

        // Weighted blend
        let blended = Vec3::new(
            t1.x * weight1 + t2.x * weight2,
            t1.y * weight1 + t2.y * weight2,
            t1.z * weight1 + t2.z * weight2,
        );

        // 50% of (10, 0, 0) + 50% of (0, 10, 0) = (5, 5, 0)
        assert!((blended.x - 5.0).abs() < 0.0001);
        assert!((blended.y - 5.0).abs() < 0.0001);
        assert!(blended.z.abs() < 0.0001);
    }

    #[test]
    fn test_render_state_bone_matrix_hierarchy() {
        // Simulate bone hierarchy: parent -> child
        let parent_bone = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let child_local = Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0));

        // Child's world transform = parent * child_local
        let child_world = parent_bone * child_local;

        let vertex = Vec3::ZERO;
        let transformed = child_world.transform_point3(vertex);

        // Origin should be at (5, 3, 0) in world space
        assert!((transformed.x - 5.0).abs() < 0.0001);
        assert!((transformed.y - 3.0).abs() < 0.0001);
        assert!(transformed.z.abs() < 0.0001);
    }
}
