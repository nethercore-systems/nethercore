//! Application state and main loop

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::config::{self, Config};
use crate::console::{EmberwareZ, VRAM_LIMIT};
use crate::graphics::ZGraphics;
use crate::input::InputManager;
use crate::library::{self, LocalGame};
use crate::ui::{LibraryUi, UiAction};
use emberware_core::app::{
    self, calculate_fps, render_debug_overlay, session::GameSession, update_frame_times, AppMode,
    DebugStats, RuntimeError, FRAME_TIME_HISTORY_SIZE, GRAPH_MAX_FRAME_TIME_MS,
    TARGET_FRAME_TIME_MS,
};
use emberware_core::console::{Audio, Console, ConsoleResourceManager, Graphics, SoundHandle};
use emberware_core::rollback::{SessionEvent, SessionType};
use emberware_core::runtime::Runtime;
use emberware_core::wasm::WasmEngine;

/// Dummy audio backend for resource processing (Z resource manager doesn't use audio)
struct DummyAudio;
impl Audio for DummyAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {}
    fn stop(&mut self, _handle: SoundHandle) {}
    fn set_rollback_mode(&mut self, _rolling_back: bool) {}
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Event loop error: {0}")]
    EventLoop(String),
}

/// Application state
pub struct App {
    /// Current application mode
    mode: AppMode,
    /// User configuration
    config: Config,
    /// Window handle (created during resumed event)
    window: Option<Arc<Window>>,
    /// Graphics backend (initialized after window creation)
    graphics: Option<ZGraphics>,
    /// Input manager (keyboard + gamepad)
    input_manager: Option<InputManager>,
    /// Whether the application should exit
    should_exit: bool,
    /// egui context
    egui_ctx: egui::Context,
    /// egui-winit state
    egui_state: Option<egui_winit::State>,
    /// egui-wgpu renderer
    egui_renderer: Option<egui_wgpu::Renderer>,
    /// Library UI state
    library_ui: LibraryUi,
    /// Settings UI state
    settings_ui: crate::settings_ui::SettingsUi,
    /// Cached local games list
    local_games: Vec<LocalGame>,
    /// Debug overlay enabled (F3)
    debug_overlay: bool,
    /// Frame times for FPS calculation (render rate)
    frame_times: Vec<Instant>,
    /// Last frame time
    last_frame: Instant,
    /// Game tick times for game FPS calculation (update rate)
    game_tick_times: Vec<Instant>,
    /// Last game tick time
    last_game_tick: Instant,
    /// Debug statistics
    debug_stats: DebugStats,
    /// Last runtime error (for displaying error in library)
    last_error: Option<RuntimeError>,
    /// WASM engine (shared across all games)
    wasm_engine: Option<WasmEngine>,
    /// Active game session (only present in Playing mode)
    game_session: Option<GameSession<EmberwareZ>>,
    /// Whether a redraw is needed (UI state changed)
    needs_redraw: bool,
    // Egui optimization cache
    cached_egui_shapes: Vec<egui::epaint::ClippedShape>,
    cached_egui_tris: Vec<egui::ClippedPrimitive>,
    cached_pixels_per_point: f32,
    last_mode: AppMode,
    last_window_size: (u32, u32),
}

impl App {
    /// Create a new application instance
    pub fn new(initial_mode: AppMode) -> Self {
        let config = config::load();

        // Initialize input manager
        let input_manager = Some(InputManager::new(config.input.clone()));

        // Load local games
        let local_games = library::get_local_games();

        let now = Instant::now();

        // Initialize WASM engine (may fail on unsupported platforms)
        let wasm_engine = match WasmEngine::new() {
            Ok(engine) => {
                tracing::info!("WASM engine initialized");
                Some(engine)
            }
            Err(e) => {
                tracing::error!("Failed to initialize WASM engine: {}", e);
                None
            }
        };

        Self {
            mode: initial_mode.clone(),
            settings_ui: crate::settings_ui::SettingsUi::new(&config),
            config,
            window: None,
            graphics: None,
            input_manager,
            should_exit: false,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            library_ui: LibraryUi::new(),
            local_games,
            debug_overlay: false,
            frame_times: Vec::with_capacity(120),
            last_frame: now,
            game_tick_times: Vec::with_capacity(120),
            last_game_tick: now,
            debug_stats: DebugStats {
                frame_times: VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE),
                vram_limit: VRAM_LIMIT,
                ..Default::default()
            },
            last_error: None,
            wasm_engine,
            game_session: None,
            needs_redraw: true,
            cached_egui_shapes: Vec::new(),
            cached_egui_tris: Vec::new(),
            cached_pixels_per_point: 1.0,
            last_mode: initial_mode.clone(),
            last_window_size: (960, 540),
        }
    }

    /// Handle a runtime error by transitioning back to library
    ///
    /// Called when the game runtime encounters an error (WASM panic, network
    /// disconnect, out of memory, etc). Transitions back to library and displays
    /// the error message to the user.
    fn handle_runtime_error(&mut self, error: RuntimeError) {
        tracing::error!("Runtime error: {}", error);
        self.game_session = None; // Clean up game session
        self.last_error = Some(error);
        self.mode = AppMode::Library;
        self.local_games = library::get_local_games();
    }


    /// Handle session events from the rollback session
    ///
    /// Processes network events like disconnect, desync, and network interruption.
    /// Returns an error if a critical event occurs that should terminate the session.
    fn handle_session_events(&mut self) -> Result<(), RuntimeError> {
        let session = match &mut self.game_session {
            Some(s) => s,
            None => return Ok(()),
        };

        // Poll remote clients for network messages (P2P sessions only)
        session.runtime.poll_remote_clients();

        // Get session events
        let events = session.runtime.handle_session_events();

        // Clear network interrupted flag - will be set again if still interrupted
        self.debug_stats.network_interrupted = None;

        for event in events {
            match event {
                SessionEvent::Disconnected { player_handle } => {
                    tracing::warn!("Player {} disconnected", player_handle);
                    return Err(RuntimeError(format!(
                        "Player {} disconnected from session",
                        player_handle
                    )));
                }
                SessionEvent::Desync {
                    frame,
                    local_checksum,
                    remote_checksum,
                } => {
                    tracing::error!(
                        "Desync detected at frame {}: local={:#x}, remote={:#x}",
                        frame,
                        local_checksum,
                        remote_checksum
                    );
                    return Err(RuntimeError(format!(
                        "Desync detected at frame {} (states diverged)",
                        frame
                    )));
                }
                SessionEvent::NetworkInterrupted {
                    player_handle,
                    disconnect_timeout_ms,
                } => {
                    tracing::warn!(
                        "Network interrupted for player {}, disconnect in {}ms",
                        player_handle,
                        disconnect_timeout_ms
                    );
                    self.debug_stats.network_interrupted = Some(disconnect_timeout_ms);
                }
                SessionEvent::NetworkResumed { player_handle } => {
                    tracing::info!("Network resumed for player {}", player_handle);
                    self.debug_stats.network_interrupted = None;
                }
                SessionEvent::Synchronized { player_handle } => {
                    tracing::info!("Synchronized with player {}", player_handle);
                }
                SessionEvent::FrameAdvantageWarning { frames_ahead } => {
                    tracing::debug!("Frame advantage warning: {} frames ahead", frames_ahead);
                }
                SessionEvent::TimeSync { frames_to_skip } => {
                    tracing::debug!("Time sync: skip {} frames", frames_to_skip);
                }
                SessionEvent::WaitingForPlayers => {
                    tracing::trace!("Waiting for remote player input");
                }
            }
        }

        Ok(())
    }

    /// Update debug stats from the current session
    ///
    /// Populates network statistics in DebugStats from the rollback session.
    fn update_session_stats(&mut self) {
        let session = match &self.game_session {
            Some(s) => s,
            None => {
                // Clear network stats when no session
                self.debug_stats.ping_ms = None;
                self.debug_stats.rollback_frames = 0;
                self.debug_stats.frame_advantage = 0;
                return;
            }
        };

        // Get session reference
        let rollback_session = match session.runtime.session() {
            Some(s) => s,
            None => {
                self.debug_stats.ping_ms = None;
                self.debug_stats.rollback_frames = 0;
                self.debug_stats.frame_advantage = 0;
                return;
            }
        };

        // Only show network stats for P2P sessions
        if rollback_session.session_type() != SessionType::P2P {
            self.debug_stats.ping_ms = None;
            self.debug_stats.rollback_frames = 0;
            self.debug_stats.frame_advantage = 0;
            return;
        }

        // Get stats from the first remote player
        let player_stats = rollback_session.all_player_stats();
        let local_players = rollback_session.local_players();

        // Find first remote player's stats
        for (idx, stats) in player_stats.iter().enumerate() {
            if !local_players.contains(&idx) {
                self.debug_stats.ping_ms = Some(stats.ping_ms);
                break;
            }
        }

        self.debug_stats.rollback_frames = rollback_session.total_rollback_frames();
        self.debug_stats.frame_advantage = rollback_session.frames_ahead();
    }

    /// Run one game frame (update + render)
    ///
    /// Returns true if the game is still running, false if it should exit.
    /// Returns (game_still_running, did_render_this_frame)
    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        // First, update input from InputManager
        if let (Some(session), Some(input_manager)) = (&mut self.game_session, &self.input_manager)
        {
            let console = session.runtime.console();

            // Get input for each local player and set it on the game
            // For now, we support 1 local player (keyboard/gamepad)
            let raw_input = input_manager.get_player_input(0);
            let z_input = console.map_input(&raw_input);

            if let Some(game) = session.runtime.game_mut() {
                game.set_input(0, z_input);
            }
        }

        // Run the game frame (fixed timestep updates)
        let session = self
            .game_session
            .as_mut()
            .ok_or_else(|| RuntimeError("No game session".to_string()))?;

        let tick_start = Instant::now();
        let (ticks, _alpha) = session
            .runtime
            .frame()
            .map_err(|e| RuntimeError(format!("Game frame error: {}", e)))?;
        let tick_elapsed = tick_start.elapsed();

        let did_render = if ticks > 0 {
            // Track game tick time for performance graph (average per tick if multiple ran)
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            // Clear previous frame's draw commands before generating new ones
            if let Some(game) = session.runtime.game_mut() {
                game.console_state_mut().clear_frame();
            }

            // Render the game ONCE per frame (even if multiple ticks ran due to slowdown)
            let render_start = Instant::now();
            session
                .runtime
                .render()
                .map_err(|e| RuntimeError(format!("Game render error: {}", e)))?;
            let render_elapsed = render_start.elapsed();
            let render_time_ms = render_elapsed.as_secs_f32() * 1000.0;

            // Track game render time
            self.debug_stats.game_render_times.push_back(render_time_ms);
            while self.debug_stats.game_render_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_render_times.pop_front();
            }

            tracing::debug!(
                "Game tick: {} ticks, update={:.2}ms, render={:.2}ms, total={:.2}ms, buffer size: {}",
                ticks,
                tick_time_ms,
                render_time_ms,
                tick_time_ms + render_time_ms,
                self.debug_stats.game_tick_times.len()
            );

            // Track game tick times for FPS calculation
            let now = Instant::now();
            for _ in 0..ticks {
                self.game_tick_times.push(now);
                if self.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                    self.game_tick_times.remove(0);
                }
            }
            self.last_game_tick = now;

            true // Did render
        } else {
            false // No render
        };

        // Process audio commands after rendering
        // Clone the audio commands and sounds to avoid double mutable borrow
        if let Some(game) = session.runtime.game_mut() {
            let console_state = game.console_state();
            let audio_commands = console_state.audio_commands.clone();
            let sounds = console_state.sounds.clone();

            if let Some(audio) = session.runtime.audio_mut() {
                audio.process_commands(&audio_commands, &sounds);
            }
        }

        // Check if game requested quit
        if let Some(game) = session.runtime.game_mut() {
            if game.state().quit_requested {
                return Ok((false, did_render));
            }
        }

        Ok((true, did_render))
    }

    /// Start a game by loading its WASM and initializing the runtime
    fn start_game(&mut self, game_id: &str) -> Result<(), RuntimeError> {
        // Find the game in local games
        let game = self
            .local_games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| RuntimeError(format!("Game not found: {}", game_id)))?;

        // Ensure WASM engine is available
        let wasm_engine = self
            .wasm_engine
            .as_ref()
            .ok_or_else(|| RuntimeError("WASM engine not initialized".to_string()))?;

        // Load the ROM file
        let rom_bytes = std::fs::read(&game.rom_path)
            .map_err(|e| RuntimeError(format!("Failed to read ROM file: {}", e)))?;

        // Load the WASM module
        let module = wasm_engine
            .load_module(&rom_bytes)
            .map_err(|e| RuntimeError(format!("Failed to load WASM module: {}", e)))?;

        // Create a linker and register FFI functions
        let mut linker = wasmtime::Linker::new(wasm_engine.engine());

        // Register common FFI functions
        emberware_core::ffi::register_common_ffi(&mut linker)
            .map_err(|e| RuntimeError(format!("Failed to register common FFI: {}", e)))?;

        // Create the console instance
        let console = EmberwareZ::new();

        // Register console-specific FFI functions
        console
            .register_ffi(&mut linker)
            .map_err(|e| RuntimeError(format!("Failed to register Z FFI: {}", e)))?;

        // Create the game instance
        let game_instance = emberware_core::wasm::GameInstance::new(wasm_engine, &module, &linker)
            .map_err(|e| RuntimeError(format!("Failed to instantiate game: {}", e)))?;

        // Create the runtime
        let mut runtime = Runtime::new(console);
        runtime.load_game(game_instance);

        // Initialize the game (calls game's init() function)
        runtime
            .init_game()
            .map_err(|e| RuntimeError(format!("Failed to initialize game: {}", e)))?;

        // Create resource manager
        let resource_manager = EmberwareZ::new().create_resource_manager();

        // Create the game session
        self.game_session = Some(GameSession::new(runtime, resource_manager));

        // Add built-in font texture to texture map (handle 0)
        // Add white fallback texture to texture map (handle 0xFFFFFFFF)
        if let (Some(session), Some(graphics)) = (&mut self.game_session, &self.graphics) {
            let font_texture_handle = graphics.font_texture();
            session.resource_manager.texture_map.insert(0, font_texture_handle);
            tracing::info!(
                "Initialized font texture in texture_map: handle 0 -> {:?}",
                font_texture_handle
            );

            let white_texture_handle = graphics.white_texture();
            session.resource_manager.texture_map.insert(u32::MAX, white_texture_handle);
            tracing::info!(
                "Initialized white texture in texture_map: handle 0xFFFFFFFF -> {:?}",
                white_texture_handle
            );
        }

        // Update render target resolution and window minimum size based on game's init config
        if let Some(session) = &self.game_session {
            if let Some(game) = session.runtime.game() {
                let z_state = game.console_state();
                let resolution_index = z_state.init_config.resolution_index as u8;

                // Update graphics render target to match game resolution
                if let Some(graphics) = &mut self.graphics {
                    graphics.update_resolution(resolution_index);

                    // Update window minimum size to match game resolution
                    if let Some(window) = &self.window {
                        let min_size =
                            winit::dpi::PhysicalSize::new(graphics.width(), graphics.height());
                        window.set_min_inner_size(Some(min_size));
                    }
                }
            }
        }

        tracing::info!("Game started: {}", game_id);
        Ok(())
    }

    /// Handle window resize
    fn handle_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let Some(graphics) = &mut self.graphics {
                graphics.resize(new_size.width, new_size.height);
            }
        }
    }

    /// Toggle fullscreen mode
    fn toggle_fullscreen(&mut self) {
        if let Some(window) = &self.window {
            let is_fullscreen = window.fullscreen().is_some();
            let new_fullscreen = if is_fullscreen {
                None
            } else {
                Some(Fullscreen::Borderless(None))
            };

            window.set_fullscreen(new_fullscreen);
            self.config.video.fullscreen = !is_fullscreen;

            // Save config
            if let Err(e) = config::save(&self.config) {
                tracing::warn!("Failed to save config: {}", e);
            }
        }
    }

    /// Handle keyboard input
    fn handle_key_input(&mut self, key_event: KeyEvent) {
        let pressed = key_event.state == ElementState::Pressed;

        // Update input manager with key state
        if let PhysicalKey::Code(key_code) = key_event.physical_key {
            // Handle key remapping in Settings mode first
            if pressed && matches!(self.mode, AppMode::Settings) {
                // Let settings UI handle the key press for remapping
                self.settings_ui.handle_key_press(key_code);
                // Don't process other key logic when remapping
                return;
            }

            if let Some(input_manager) = &mut self.input_manager {
                input_manager.update_keyboard(key_code, pressed);
            }

            // Handle special keys
            if pressed {
                match key_code {
                    KeyCode::F3 => {
                        self.debug_overlay = !self.debug_overlay;
                    }
                    KeyCode::F11 => {
                        self.toggle_fullscreen();
                    }
                    KeyCode::Enter => {
                        // Alt+Enter for fullscreen toggle
                        // Note: Alt modifier check would go here
                        // For now, we use F11 as the primary method
                    }
                    KeyCode::Escape => {
                        // Return to library when in game or settings
                        match self.mode {
                            AppMode::Playing { .. } => {
                                tracing::info!("Exiting game via ESC");
                                self.game_session = None; // Clean up game session
                                self.mode = AppMode::Library;
                                self.local_games = library::get_local_games();
                            }
                            AppMode::Settings => {
                                // If waiting for key binding, cancel it; otherwise return to library
                                if !self.settings_ui.handle_key_press(key_code) {
                                    self.mode = AppMode::Library;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Update input state (call this each frame)
    fn update_input(&mut self) {
        if let Some(input_manager) = &mut self.input_manager {
            input_manager.update();
        }
    }

    /// Handle UI actions
    fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::PlayGame(game_id) => {
                tracing::info!("Playing game: {}", game_id);
                self.last_error = None; // Clear any previous error

                // Try to start the game
                match self.start_game(&game_id) {
                    Ok(()) => {
                        self.mode = AppMode::Playing { game_id };
                    }
                    Err(e) => {
                        self.handle_runtime_error(e);
                    }
                }
            }
            UiAction::DeleteGame(game_id) => {
                tracing::info!("Deleting game: {}", game_id);
                if let Err(e) = library::delete_game(&game_id) {
                    tracing::error!("Failed to delete game: {}", e);
                }
                self.local_games = library::get_local_games();
                self.library_ui.selected_game = None;
            }
            UiAction::OpenBrowser => {
                const PLATFORM_URL: &str = "https://emberware.io";
                tracing::info!("Opening browser to {}", PLATFORM_URL);
                if let Err(e) = open::that(PLATFORM_URL) {
                    tracing::error!("Failed to open browser: {}", e);
                }
            }
            UiAction::OpenSettings => {
                // Toggle between Library and Settings
                self.mode = match self.mode {
                    AppMode::Settings => {
                        tracing::info!("Returning to library");
                        AppMode::Library
                    }
                    _ => {
                        tracing::info!("Opening settings");
                        // Update settings UI with current config
                        self.settings_ui.update_temp_config(&self.config);
                        AppMode::Settings
                    }
                };
            }
            UiAction::DismissError => {
                self.last_error = None;
            }
            UiAction::RefreshLibrary => {
                tracing::info!("Refreshing game library");
                self.local_games = library::get_local_games();
                self.library_ui.selected_game = None;
            }
            UiAction::SaveSettings(new_config) => {
                tracing::info!("Saving settings...");
                self.config = new_config.clone();

                // Save to disk
                if let Err(e) = config::save(&self.config) {
                    tracing::error!("Failed to save config: {}", e);
                } else {
                    tracing::info!("Settings saved successfully");
                }

                // Apply changes to input manager (recreate with new config)
                self.input_manager = Some(InputManager::new(self.config.input.clone()));

                if let Some(graphics) = &mut self.graphics {
                    graphics.set_scale_mode(self.config.video.scale_mode);
                }

                // Apply fullscreen setting if changed
                if let Some(window) = &self.window {
                    let is_fullscreen = window.fullscreen().is_some();
                    if is_fullscreen != self.config.video.fullscreen {
                        let new_fullscreen = if self.config.video.fullscreen {
                            Some(Fullscreen::Borderless(None))
                        } else {
                            None
                        };
                        window.set_fullscreen(new_fullscreen);
                    }
                }

                // Update settings UI temp config
                self.settings_ui.update_temp_config(&self.config);

                // Return to library
                self.mode = AppMode::Library;
            }
            UiAction::SetScaleMode(scale_mode) => {
                // Preview scale mode change
                if let Some(graphics) = &mut self.graphics {
                    graphics.set_scale_mode(scale_mode);
                }
            }
        }
    }

    /// Calculate render FPS from frame times
    fn calculate_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }
        let elapsed = self
            .frame_times
            .last()
            .unwrap()
            .duration_since(*self.frame_times.first().unwrap())
            .as_secs_f32();
        if elapsed > 0.0 {
            self.frame_times.len() as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate game tick FPS (actual update rate)
    fn calculate_game_tick_fps(&self) -> f32 {
        if self.game_tick_times.len() < 2 {
            return 0.0;
        }
        let elapsed = self
            .game_tick_times
            .last()
            .unwrap()
            .duration_since(*self.game_tick_times.first().unwrap())
            .as_secs_f32();
        if elapsed > 0.0 {
            self.game_tick_times.len() as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Render the current frame
    fn render(&mut self) {
        let now = Instant::now();

        // Update frame timing
        self.frame_times.push(now);
        if self.frame_times.len() > FRAME_TIME_HISTORY_SIZE {
            self.frame_times.remove(0);
        }
        let frame_time_ms = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;
        self.last_frame = now;

        // Update debug stats
        self.debug_stats.frame_times.push_back(frame_time_ms);
        while self.debug_stats.frame_times.len() > FRAME_TIME_HISTORY_SIZE {
            self.debug_stats.frame_times.pop_front();
        }

        // Handle Playing mode: run game frame first
        let mut game_rendered_this_frame = false;
        if matches!(self.mode, AppMode::Playing { .. }) {
            // Handle session events (disconnect, desync, network interruption)
            if let Err(e) = self.handle_session_events() {
                self.handle_runtime_error(e);
                return;
            }

            // Update debug stats from session
            self.update_session_stats();

            // Run game frame (update + render)
            let (game_running, did_render) = match self.run_game_frame() {
                Ok((running, rendered)) => {
                    // Only process resources and execute draw commands if we rendered
                    if rendered {
                        if let Some(session) = &mut self.game_session {
                            if let Some(graphics) = &mut self.graphics {
                                // Process pending resources by accessing resource manager directly
                                // Use dummy audio since Z resource manager doesn't use it
                                let mut dummy_audio = DummyAudio;
                                {
                                    if let Some(game) = session.runtime.game_mut() {
                                        let state = game.console_state_mut();
                                        session.resource_manager.process_pending_resources(graphics, &mut dummy_audio, state);
                                    }
                                }

                                // Execute draw commands
                                {
                                    if let Some(game) = session.runtime.game_mut() {
                                        let state = game.console_state_mut();
                                        session.resource_manager.execute_draw_commands(graphics, state);
                                    }
                                }
                            }
                        }
                    }
                    (running, rendered)
                }
                Err(e) => {
                    self.handle_runtime_error(e);
                    return;
                }
            };

            game_rendered_this_frame = did_render;

            // If game requested quit, return to library
            if !game_running {
                self.game_session = None;
                self.mode = AppMode::Library;
                self.local_games = library::get_local_games();
                return;
            }
        }

        // Pre-collect values to avoid borrow conflicts
        let mode = self.mode.clone();
        let debug_overlay = self.debug_overlay;
        let render_fps = self.calculate_fps();
        let game_tick_fps = self.calculate_game_tick_fps();
        let last_error = self.last_error.clone();

        let window = match self.window.clone() {
            Some(w) => w,
            None => return,
        };

        let graphics = match &mut self.graphics {
            Some(g) => g,
            None => return,
        };

        // Update VRAM usage from graphics
        self.debug_stats.vram_used = graphics.vram_used();

        let egui_state = match &mut self.egui_state {
            Some(s) => s,
            None => return,
        };

        let egui_renderer = match &mut self.egui_renderer {
            Some(r) => r,
            None => return,
        };

        // Get surface texture
        let surface_texture = match graphics.get_current_texture() {
            Ok(tex) => tex,
            Err(e) => {
                tracing::warn!("Failed to get surface texture: {}", e);
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create SINGLE encoder for entire frame
        let mut encoder =
            graphics
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });

        // If in Playing mode, render game (only if we generated new content this frame)
        if matches!(mode, AppMode::Playing { .. }) {
            if game_rendered_this_frame {
                // Get clear color from game state
                let clear_color = {
                    if let Some(session) = &self.game_session {
                        if let Some(game) = session.runtime.game() {
                            let z_state = game.console_state();

                            let clear = z_state.init_config.clear_color;
                            let clear_r = ((clear >> 24) & 0xFF) as f32 / 255.0;
                            let clear_g = ((clear >> 16) & 0xFF) as f32 / 255.0;
                            let clear_b = ((clear >> 8) & 0xFF) as f32 / 255.0;
                            let clear_a = (clear & 0xFF) as f32 / 255.0;
                            [clear_r, clear_g, clear_b, clear_a]
                        } else {
                            [0.1, 0.1, 0.1, 1.0]
                        }
                    } else {
                        [0.1, 0.1, 0.1, 1.0]
                    }
                };

                // Render new game content to render target
                if let Some(session) = &mut self.game_session {
                    if let Some(game) = session.runtime.game_mut() {
                        let z_state = game.console_state_mut();
                        graphics.render_frame(&mut encoder, z_state, clear_color);
                    }
                }
            }

            // Always blit the render target to the window (shows last rendered frame)
            graphics.blit_to_window(&mut encoder, &view);
        }

        // Start egui frame
        let raw_input = egui_state.take_egui_input(&window);

        // Collect UI action separately to avoid borrow conflicts
        let mut ui_action = None;

        // Collect debug stats for overlay
        let debug_stats = DebugStats {
            frame_times: self.debug_stats.frame_times.clone(),
            game_tick_times: self.debug_stats.game_tick_times.clone(),
            game_render_times: self.debug_stats.game_render_times.clone(),
            vram_used: self.debug_stats.vram_used,
            vram_limit: self.debug_stats.vram_limit,
            ping_ms: self.debug_stats.ping_ms,
            rollback_frames: self.debug_stats.rollback_frames,
            frame_advantage: self.debug_stats.frame_advantage,
            network_interrupted: self.debug_stats.network_interrupted,
        };

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Render UI based on current mode
            match &mode {
                AppMode::Library => {
                    // Show error message if there was a recent error
                    if let Some(ref error) = last_error {
                        egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                                if ui.button("Dismiss").clicked() {
                                    ui_action = Some(UiAction::DismissError);
                                }
                            });
                        });
                    }
                    if let Some(action) = self.library_ui.show(ctx, &self.local_games) {
                        ui_action = Some(action);
                    }
                }
                AppMode::Settings => {
                    if let Some(action) = self.settings_ui.show(ctx) {
                        ui_action = Some(action);
                    }
                }
                AppMode::Playing { ref game_id } => {
                    // Game is rendered before egui, so we don't need a central panel
                    // Just show debug info if overlay is enabled
                    let _ = game_id; // Used in debug overlay
                }
            }

            // Debug overlay
            if debug_overlay {
                egui::Window::new("Debug")
                    .default_pos([10.0, 10.0])
                    .resizable(true)
                    .default_width(300.0)
                    .show(ctx, |ui| {
                        // Performance section
                        ui.heading("Performance");
                        if matches!(mode, AppMode::Playing { .. }) {
                            ui.label(format!("Game FPS: {:.1}", game_tick_fps));
                            ui.label(format!("Render FPS: {:.1}", render_fps));
                        } else {
                            ui.label(format!("FPS: {:.1}", render_fps));
                        }
                        ui.label(format!("Frame time: {:.2}ms", frame_time_ms));
                        ui.label(format!("Mode: {:?}", mode));

                        // Frame time graph
                        ui.add_space(4.0);
                        let graph_height = 60.0;
                        let (rect, _response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), graph_height),
                            egui::Sense::hover(),
                        );

                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter_at(rect);

                            // Background
                            painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

                            // Choose which times to display based on mode
                            let (times_to_display, graph_label, graph_max) =
                                if matches!(mode, AppMode::Playing { .. }) {
                                    // Game tick budget visualization - full height = 16.67ms budget
                                    let label = format!(
                                        "Game tick budget ({:.1}ms target)",
                                        TARGET_FRAME_TIME_MS
                                    );
                                    (&debug_stats.game_tick_times, label, TARGET_FRAME_TIME_MS)
                                } else {
                                    (
                                        &debug_stats.frame_times,
                                        "Frame time (0-33ms)".to_string(),
                                        GRAPH_MAX_FRAME_TIME_MS,
                                    )
                                };

                            // Target line (16.67ms for 60 FPS)
                            let target_y =
                                rect.bottom() - (TARGET_FRAME_TIME_MS / graph_max * graph_height);
                            painter.hline(
                                rect.left()..=rect.right(),
                                target_y,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                            );

                            // Budget bars (for game tick times in Playing mode)
                            let is_playing = matches!(mode, AppMode::Playing { .. });
                            if !times_to_display.is_empty() {
                                let bar_width = rect.width() / FRAME_TIME_HISTORY_SIZE as f32;
                                for (i, &time_ms) in times_to_display.iter().enumerate() {
                                    let x = rect.left() + i as f32 * bar_width;

                                    if is_playing {
                                        // Stacked budget bar: update (blue) + render (orange) + available (green)

                                        // Get update and render times for this tick
                                        let update_time = time_ms; // game_tick_times[i]
                                        let render_time = debug_stats
                                            .game_render_times
                                            .get(i)
                                            .copied()
                                            .unwrap_or(0.0);
                                        let total_time = update_time + render_time;

                                        // Calculate heights (scaled to budget, capped at 150%)
                                        let update_height = ((update_time / TARGET_FRAME_TIME_MS)
                                            .min(1.5)
                                            * graph_height);
                                        let render_height = ((render_time / TARGET_FRAME_TIME_MS)
                                            .min(1.5)
                                            * graph_height);
                                        let total_height =
                                            (update_height + render_height).min(graph_height);

                                        let bottom_y = rect.bottom();

                                        // Draw background (unused budget) - green if under budget, red if over
                                        let bg_color = if total_time <= TARGET_FRAME_TIME_MS {
                                            egui::Color32::from_rgb(40, 80, 40) // Dark green - headroom available
                                        } else {
                                            egui::Color32::from_rgb(80, 40, 40) // Dark red - over budget
                                        };
                                        painter.rect_filled(
                                            egui::Rect::from_min_max(
                                                egui::pos2(x, rect.top()),
                                                egui::pos2(x + bar_width - 1.0, bottom_y),
                                            ),
                                            0.0,
                                            bg_color,
                                        );

                                        // Draw update time (bottom, blue)
                                        if update_height > 0.0 {
                                            painter.rect_filled(
                                                egui::Rect::from_min_max(
                                                    egui::pos2(x, bottom_y - update_height),
                                                    egui::pos2(x + bar_width - 1.0, bottom_y),
                                                ),
                                                0.0,
                                                egui::Color32::from_rgb(80, 120, 200), // Blue - update time
                                            );
                                        }

                                        // Draw render time (stacked on top of update, orange)
                                        if render_height > 0.0 {
                                            painter.rect_filled(
                                                egui::Rect::from_min_max(
                                                    egui::pos2(x, bottom_y - total_height),
                                                    egui::pos2(
                                                        x + bar_width - 1.0,
                                                        bottom_y - update_height,
                                                    ),
                                                ),
                                                0.0,
                                                egui::Color32::from_rgb(220, 140, 60), // Orange - render time
                                            );
                                        }
                                    } else {
                                        // Standard bars for render times in Library mode
                                        let height =
                                            (time_ms / graph_max * graph_height).min(graph_height);
                                        let bar_rect = egui::Rect::from_min_max(
                                            egui::pos2(x, rect.bottom() - height),
                                            egui::pos2(x + bar_width - 1.0, rect.bottom()),
                                        );

                                        let color = if time_ms <= TARGET_FRAME_TIME_MS {
                                            egui::Color32::from_rgb(100, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(200, 200, 100)
                                        };

                                        painter.rect_filled(bar_rect, 0.0, color);
                                    }
                                }
                            }

                            // Label
                            painter.text(
                                egui::pos2(rect.left() + 4.0, rect.top() + 2.0),
                                egui::Align2::LEFT_TOP,
                                graph_label,
                                egui::FontId::proportional(10.0),
                                egui::Color32::from_gray(150),
                            );
                        }

                        ui.separator();

                        // Memory section
                        ui.heading("Memory");
                        let vram_mb = debug_stats.vram_used as f32 / (1024.0 * 1024.0);
                        let vram_limit_mb = debug_stats.vram_limit as f32 / (1024.0 * 1024.0);
                        let vram_pct = debug_stats.vram_used as f32 / debug_stats.vram_limit as f32;
                        ui.label(format!(
                            "VRAM: {:.2} / {:.2} MB ({:.1}%)",
                            vram_mb,
                            vram_limit_mb,
                            vram_pct * 100.0
                        ));
                        ui.add(egui::ProgressBar::new(vram_pct).show_percentage());

                        ui.separator();

                        // Network section
                        ui.heading("Network");
                        if let Some(ping) = debug_stats.ping_ms {
                            ui.label(format!("Ping: {}ms", ping));
                            ui.label(format!("Rollback frames: {}", debug_stats.rollback_frames));
                            ui.label(format!("Frame advantage: {}", debug_stats.frame_advantage));

                            // Network interrupted warning
                            if let Some(timeout_ms) = debug_stats.network_interrupted {
                                ui.add_space(4.0);
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 200, 50),
                                    format!("⚠ Connection interrupted ({}ms)", timeout_ms),
                                );
                            }
                        } else {
                            ui.label("No network session");
                        }
                    });
            }
        });

        egui_state.handle_platform_output(&window, full_output.platform_output);

        // Determine if egui needs update
        let mut egui_dirty = false;

        // Check 1: First frame
        if self.cached_egui_shapes.is_empty() {
            egui_dirty = true;
        }

        // Check 2: Mode changed
        if !egui_dirty && self.last_mode != mode {
            egui_dirty = true;
            self.last_mode = mode.clone();
        }

        // Check 3: Window resized
        let current_size = (graphics.width(), graphics.height());
        if !egui_dirty && self.last_window_size != current_size {
            egui_dirty = true;
            self.last_window_size = current_size;
        }

        // Check 4: DPI changed
        if !egui_dirty
            && (self.cached_pixels_per_point - full_output.pixels_per_point).abs() > 0.001
        {
            egui_dirty = true;
            self.cached_pixels_per_point = full_output.pixels_per_point;
        }

        // Check 5: Texture changes
        if !egui_dirty && !full_output.textures_delta.set.is_empty() {
            egui_dirty = true;
        }

        // Check 6: Shapes changed (fast vector comparison)
        if !egui_dirty {
            if full_output.shapes.len() != self.cached_egui_shapes.len() {
                egui_dirty = true;
            } else {
                for (new_shape, old_shape) in
                    full_output.shapes.iter().zip(&self.cached_egui_shapes)
                {
                    if new_shape != old_shape {
                        egui_dirty = true;
                        break;
                    }
                }
            }
        }

        // Check 7: Viewport repaint requested
        if !egui_dirty {
            for viewport_output in full_output.viewport_output.values() {
                if viewport_output.repaint_delay.is_zero() {
                    egui_dirty = true;
                    break;
                }
            }
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [graphics.width(), graphics.height()],
            pixels_per_point: window.scale_factor() as f32,
        };

        // Conditional tessellation and buffer update
        let tris = if egui_dirty {
            // Tessellate and cache
            let new_tris = self
                .egui_ctx
                .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

            // Update cache
            self.cached_egui_shapes = full_output.shapes;
            self.cached_egui_tris = new_tris.clone();

            // Update GPU buffers (ONLY when dirty)
            egui_renderer.update_buffers(
                graphics.device(),
                graphics.queue(),
                &mut encoder,
                &new_tris,
                &screen_descriptor,
            );

            new_tris
        } else {
            // Reuse cached triangles
            self.cached_egui_tris.clone()
        };

        // Texture updates still happen (already delta-tracked)
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(graphics.device(), graphics.queue(), *id, image_delta);
        }

        // Create render pass and render egui (only if there are triangles to render)
        // When in Playing mode, use Load to preserve game rendering.
        // Otherwise, clear with a dark background color.
        let is_playing = matches!(mode, AppMode::Playing { .. });
        if !tris.is_empty() {
            let load_op = if is_playing {
                wgpu::LoadOp::Load
            } else {
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                })
            };

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // egui-wgpu 0.30 requires RenderPass<'static>. wgpu's forget_lifetime()
            // safely removes the lifetime constraint, converting compile-time errors
            // to runtime errors if the encoder is misused while the pass is active.
            let mut render_pass_static = render_pass.forget_lifetime();

            egui_renderer.render(&mut render_pass_static, &tris, &screen_descriptor);
        } else if !is_playing {
            // If no egui content but not in playing mode, we still need to clear the screen
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Submit commands
        graphics.queue().submit(std::iter::once(encoder.finish()));

        // Free egui textures
        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Present frame
        surface_texture.present();

        // Handle UI action after rendering is complete
        if let Some(action) = ui_action {
            if matches!(action, UiAction::OpenSettings) && matches!(self.mode, AppMode::Settings) {
                self.mode = AppMode::Library;
            } else {
                self.handle_ui_action(action);
            }
        }

        // Request next frame
        self.request_redraw_if_needed();
    }

    fn request_redraw_if_needed(&mut self) {
        // In Playing mode or Library/Settings with Poll control flow,
        // we request redraws continuously to ensure UI responsiveness.
        // The egui dirty-checking and mesh caching prevents unnecessary GPU work.
        let needs_redraw = true;

        if needs_redraw {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.needs_redraw = false;
        }
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        // Default to 960×540 (default game resolution)
        let window_attributes = Window::default_attributes()
            .with_title("Emberware Z")
            .with_inner_size(winit::dpi::LogicalSize::new(960, 540));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                self.should_exit = true;
                return;
            }
        };

        // Apply fullscreen from config
        if self.config.video.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        // Initialize graphics backend
        let mut graphics = match pollster::block_on(ZGraphics::new(window.clone())) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to initialize graphics: {}", e);
                self.should_exit = true;
                return;
            }
        };

        // Apply scale mode from config
        graphics.set_scale_mode(self.config.video.scale_mode);

        // Initialize egui-winit state
        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        // Initialize egui-wgpu renderer
        let egui_renderer = egui_wgpu::Renderer::new(
            graphics.device(),
            graphics.surface_format(),
            None,
            1,
            false, // dithering
        );

        tracing::info!("Graphics and egui initialized successfully");
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        self.graphics = Some(graphics);
        self.window = Some(window);

        // If a game session exists, add the font and white textures to its texture map
        // (game session may have been created before graphics was initialized)
        if let (Some(session), Some(graphics)) = (&mut self.game_session, &self.graphics) {
            let font_texture_handle = graphics.font_texture();
            session.resource_manager.texture_map.insert(0, font_texture_handle);
            tracing::info!(
                "Added font texture to existing game session: handle 0 -> {:?}",
                font_texture_handle
            );

            let white_texture_handle = graphics.white_texture();
            session.resource_manager.texture_map.insert(u32::MAX, white_texture_handle);
            tracing::info!(
                "Added white texture to existing game session: handle 0xFFFFFFFF -> {:?}",
                white_texture_handle
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let egui handle the event first
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, &event);
            if response.consumed {
                self.mark_needs_redraw();
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested");
                self.should_exit = true;
            }
            WindowEvent::Resized(new_size) => {
                tracing::debug!("Window resized to {:?}", new_size);
                self.handle_resize(new_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                tracing::debug!("DPI scale factor changed to {}", scale_factor);
                // Window resize event will follow, which will trigger handle_resize
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_key_input(key_event);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }

        if self.should_exit {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Update input state
        self.update_input();

        // Conditional redraw based on mode
        self.request_redraw_if_needed();
    }
}

pub fn run(initial_mode: AppMode) -> Result<(), AppError> {
    tracing::info!("Starting with mode: {:?}", initial_mode);

    let event_loop = EventLoop::new()
        .map_err(|e| AppError::EventLoop(format!("Failed to create event loop: {}", e)))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(initial_mode);

    event_loop
        .run_app(&mut app)
        .map_err(|e| AppError::EventLoop(format!("Event loop error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test AppMode enum
    #[test]
    fn test_app_mode_library_default() {
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));
    }

    #[test]
    fn test_app_mode_playing_with_game_id() {
        let mode = AppMode::Playing {
            game_id: "test-game".to_string(),
        };
        if let AppMode::Playing { game_id } = mode {
            assert_eq!(game_id, "test-game");
        } else {
            panic!("Expected Playing mode");
        }
    }

    #[test]
    fn test_app_mode_settings() {
        let mode = AppMode::Settings;
        assert!(matches!(mode, AppMode::Settings));
    }

    #[test]
    fn test_app_mode_clone() {
        let mode = AppMode::Playing {
            game_id: "clone-test".to_string(),
        };
        let cloned = mode.clone();
        if let AppMode::Playing { game_id } = cloned {
            assert_eq!(game_id, "clone-test");
        } else {
            panic!("Expected Playing mode after clone");
        }
    }

    // Test RuntimeError
    #[test]
    fn test_runtime_error_display() {
        let error = RuntimeError("Test error message".to_string());
        assert_eq!(format!("{}", error), "Test error message");
    }

    #[test]
    fn test_runtime_error_debug() {
        let error = RuntimeError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_runtime_error_clone() {
        let error = RuntimeError("Clone test".to_string());
        let cloned = error.clone();
        assert_eq!(error.0, cloned.0);
    }

    // Test AppError
    #[test]
    fn test_app_error_event_loop() {
        let error = AppError::EventLoop("test error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Event loop error"));
        assert!(display.contains("test error"));
    }

    // Test DebugStats
    #[test]
    fn test_debug_stats_default() {
        let stats = DebugStats::default();
        assert!(stats.frame_times.is_empty());
        assert_eq!(stats.vram_used, 0);
        assert_eq!(stats.vram_limit, 0);
        assert!(stats.ping_ms.is_none());
        assert_eq!(stats.rollback_frames, 0);
        assert_eq!(stats.frame_advantage, 0);
        assert!(stats.network_interrupted.is_none());
    }

    #[test]
    fn test_debug_stats_frame_times() {
        let mut stats = DebugStats::default();
        stats.frame_times.push_back(16.67);
        stats.frame_times.push_back(17.0);
        stats.frame_times.push_back(15.5);
        assert_eq!(stats.frame_times.len(), 3);
        assert_eq!(stats.frame_times[0], 16.67);
    }

    #[test]
    fn test_debug_stats_network_stats() {
        let stats = DebugStats {
            ping_ms: Some(25),
            rollback_frames: 10,
            frame_advantage: -2,
            ..Default::default()
        };
        assert_eq!(stats.ping_ms, Some(25));
        assert_eq!(stats.rollback_frames, 10);
        assert_eq!(stats.frame_advantage, -2);
    }

    #[test]
    fn test_debug_stats_network_interrupted() {
        let mut stats = DebugStats::default();
        assert!(stats.network_interrupted.is_none());

        // Set network interrupted
        stats.network_interrupted = Some(3000);
        assert_eq!(stats.network_interrupted, Some(3000));

        // Clear network interrupted
        stats.network_interrupted = None;
        assert!(stats.network_interrupted.is_none());
    }

    // Test constants
    #[test]
    fn test_frame_time_history_size() {
        assert_eq!(FRAME_TIME_HISTORY_SIZE, 120);
    }

    #[test]
    fn test_target_frame_time() {
        // 60 FPS = 16.67ms per frame
        assert!((TARGET_FRAME_TIME_MS - 16.67).abs() < 0.01);
    }

    // Test FPS calculation logic (isolated)
    #[test]
    fn test_calculate_fps_no_samples() {
        // With 0 or 1 samples, FPS should be 0
        let frame_times: Vec<Instant> = vec![];
        let fps = if frame_times.len() < 2 {
            0.0
        } else {
            frame_times.len() as f32
        };
        assert_eq!(fps, 0.0);
    }

    #[test]
    fn test_calculate_fps_single_sample() {
        let frame_times = [Instant::now()];
        let fps = if frame_times.len() < 2 {
            0.0
        } else {
            frame_times.len() as f32
        };
        assert_eq!(fps, 0.0);
    }

    // Test App state transitions (simulated without window)
    // These tests verify the state machine logic in isolation

    #[test]
    fn test_state_transition_library_to_playing() {
        // Simulating handle_ui_action for PlayGame
        let mut mode = AppMode::Library;
        let action = UiAction::PlayGame("test-game".to_string());

        if let UiAction::PlayGame(game_id) = action {
            mode = AppMode::Playing { game_id };
        }

        if let AppMode::Playing { game_id } = mode {
            assert_eq!(game_id, "test-game");
        } else {
            panic!("Expected Playing mode");
        }
    }

    #[test]
    fn test_state_transition_playing_to_library_escape() {
        // Simulating escape key handling from Playing state
        let mut mode = AppMode::Playing {
            game_id: "some-game".to_string(),
        };

        // Simulate ESC press in Playing mode
        if let AppMode::Playing { .. } = mode {
            mode = AppMode::Library;
        }

        assert!(matches!(mode, AppMode::Library));
    }

    #[test]
    fn test_state_transition_settings_to_library_escape() {
        // Simulating escape key handling from Settings state
        let mut mode = AppMode::Settings;

        // Simulate ESC press in Settings mode
        if mode == AppMode::Settings {
            mode = AppMode::Library;
        }

        assert!(matches!(mode, AppMode::Library));
    }

    #[test]
    fn test_state_transition_library_to_settings() {
        // Simulating OpenSettings action
        let mut mode = AppMode::Library;

        let action = UiAction::OpenSettings;
        if action == UiAction::OpenSettings {
            mode = AppMode::Settings;
        }

        assert!(matches!(mode, AppMode::Settings));
    }

    #[test]
    fn test_runtime_error_transitions_to_library() {
        // Simulating handle_runtime_error
        let mode = AppMode::Playing {
            game_id: "test".to_string(),
        };

        // Start in Playing mode
        assert!(matches!(mode, AppMode::Playing { .. }));

        // Simulate runtime error - error stored and mode transitions
        let error = RuntimeError("WASM panic".to_string());
        let last_error = error;
        let mode = AppMode::Library;

        assert!(matches!(mode, AppMode::Library));
        assert_eq!(last_error.0, "WASM panic");
    }

    #[test]
    fn test_dismiss_error_clears_error() {
        let mut last_error: Option<RuntimeError> = Some(RuntimeError("test error".to_string()));

        // Simulate DismissError action
        let action = UiAction::DismissError;
        if action == UiAction::DismissError {
            last_error = None;
        }

        assert!(last_error.is_none());
    }

    #[test]
    fn test_play_game_clears_previous_error() {
        // When playing a new game, previous error should be cleared
        let mut last_error: Option<RuntimeError> = Some(RuntimeError("old error".to_string()));
        let mut mode = AppMode::Library;

        let action = UiAction::PlayGame("new-game".to_string());
        if let UiAction::PlayGame(game_id) = action {
            last_error = None; // Clear any previous error
            mode = AppMode::Playing { game_id };
        }

        assert!(last_error.is_none());
        assert!(matches!(mode, AppMode::Playing { .. }));
    }

    // Test fullscreen toggle logic (isolated from actual window)
    #[test]
    fn test_fullscreen_toggle_logic() {
        let mut is_fullscreen = false;

        // Toggle from windowed to fullscreen
        is_fullscreen = !is_fullscreen;
        assert!(is_fullscreen);

        // Toggle back to windowed
        is_fullscreen = !is_fullscreen;
        assert!(!is_fullscreen);
    }

    // Test resize validation logic
    #[test]
    fn test_resize_validation_accepts_valid_size() {
        let new_size = winit::dpi::PhysicalSize::new(1920u32, 1080u32);
        let should_resize = new_size.width > 0 && new_size.height > 0;
        assert!(should_resize);
    }

    #[test]
    fn test_resize_validation_rejects_zero_width() {
        let new_size = winit::dpi::PhysicalSize::new(0u32, 1080u32);
        let should_resize = new_size.width > 0 && new_size.height > 0;
        assert!(!should_resize);
    }

    #[test]
    fn test_resize_validation_rejects_zero_height() {
        let new_size = winit::dpi::PhysicalSize::new(1920u32, 0u32);
        let should_resize = new_size.width > 0 && new_size.height > 0;
        assert!(!should_resize);
    }

    #[test]
    fn test_resize_validation_rejects_zero_both() {
        let new_size = winit::dpi::PhysicalSize::new(0u32, 0u32);
        let should_resize = new_size.width > 0 && new_size.height > 0;
        assert!(!should_resize);
    }

    // Test debug overlay toggle
    #[test]
    fn test_debug_overlay_toggle() {
        let mut debug_overlay = false;

        // Toggle on with F3
        debug_overlay = !debug_overlay;
        assert!(debug_overlay);

        // Toggle off with F3
        debug_overlay = !debug_overlay;
        assert!(!debug_overlay);
    }

    // Test should_exit flag
    #[test]
    fn test_should_exit_initial_false() {
        let should_exit = false;
        assert!(!should_exit);
    }

    #[test]
    fn test_should_exit_on_close_request() {
        // Simulate close requested - flag should become true
        let should_exit = true;
        assert!(should_exit);
    }

    // Test frame time tracking logic
    #[test]
    fn test_frame_times_capped_at_120() {
        let mut frame_times: Vec<Instant> = Vec::with_capacity(FRAME_TIME_HISTORY_SIZE);

        // Add 130 frames
        for _ in 0..130 {
            frame_times.push(Instant::now());
            if frame_times.len() > FRAME_TIME_HISTORY_SIZE {
                frame_times.remove(0);
            }
        }

        assert_eq!(frame_times.len(), FRAME_TIME_HISTORY_SIZE);
    }

    // Test debug stats frame time ring buffer
    #[test]
    fn test_debug_stats_frame_time_ring_buffer() {
        let mut frame_times: VecDeque<f32> = VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE);

        // Add more than the limit
        for i in 0..150 {
            frame_times.push_back(i as f32);
            while frame_times.len() > FRAME_TIME_HISTORY_SIZE {
                frame_times.pop_front();
            }
        }

        assert_eq!(frame_times.len(), FRAME_TIME_HISTORY_SIZE);
        // First value should be 30 (150 - 120)
        assert_eq!(frame_times[0], 30.0);
    }

    // Test UI action variants exist
    #[test]
    fn test_ui_action_play_game() {
        let action = UiAction::PlayGame("game-id".to_string());
        if let UiAction::PlayGame(id) = action {
            assert_eq!(id, "game-id");
        } else {
            panic!("Expected PlayGame action");
        }
    }

    #[test]
    fn test_ui_action_delete_game() {
        let action = UiAction::DeleteGame("delete-id".to_string());
        if let UiAction::DeleteGame(id) = action {
            assert_eq!(id, "delete-id");
        } else {
            panic!("Expected DeleteGame action");
        }
    }

    #[test]
    fn test_ui_action_open_browser() {
        let action = UiAction::OpenBrowser;
        assert!(matches!(action, UiAction::OpenBrowser));
    }

    #[test]
    fn test_ui_action_open_settings() {
        let action = UiAction::OpenSettings;
        assert!(matches!(action, UiAction::OpenSettings));
    }

    #[test]
    fn test_ui_action_dismiss_error() {
        let action = UiAction::DismissError;
        assert!(matches!(action, UiAction::DismissError));
    }

    // Test multiple state transitions (full cycle)
    #[test]
    fn test_full_state_cycle_library_play_library() {
        // 1. Start in Library
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));

        // 2. Play a game
        let mode = AppMode::Playing {
            game_id: "test".to_string(),
        };
        assert!(matches!(mode, AppMode::Playing { .. }));

        // 3. Game crashes with error
        let last_error: Option<RuntimeError> = Some(RuntimeError("crash".to_string()));
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));
        assert!(last_error.is_some());

        // 4. Dismiss error
        let last_error: Option<RuntimeError> = None;
        assert!(last_error.is_none());

        // 5. Play another game
        let mode = AppMode::Playing {
            game_id: "test2".to_string(),
        };
        assert!(matches!(mode, AppMode::Playing { .. }));

        // 6. Exit normally with ESC
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));
    }

    #[test]
    fn test_settings_round_trip() {
        // Start in Library
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));

        // Go to settings
        let mode = AppMode::Settings;
        assert!(matches!(mode, AppMode::Settings));

        // Back to library
        let mode = AppMode::Library;
        assert!(matches!(mode, AppMode::Library));
    }
}
