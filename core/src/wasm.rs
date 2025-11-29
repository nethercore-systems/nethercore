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

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM engine")
    }
}

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
}
