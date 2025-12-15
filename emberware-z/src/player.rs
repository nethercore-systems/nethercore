//! Standalone player for Emberware Z
//!
//! This module provides a minimal player application that can run .ewz ROM files
//! without the full library UI. Used by:
//! - `emberware-z` binary (standalone player)
//! - `ember run` command (development)
//! - Library process spawning

use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window};

use emberware_core::ConsoleRunner;
use emberware_core::app::event_loop::ConsoleApp;
use emberware_core::app::{DebugStats, FRAME_TIME_HISTORY_SIZE, RuntimeError};
use emberware_core::console::{Console, ConsoleResourceManager};
use emberware_core::debug::FrameController;
use z_common::{ZDataPack, ZRom};

use crate::audio;
use crate::console::EmberwareZ;
use crate::input::InputManager;

/// Player configuration passed from CLI
pub struct PlayerConfig {
    /// ROM file path
    pub rom_path: std::path::PathBuf,
    /// Start in fullscreen
    pub fullscreen: bool,
    /// Integer scaling factor
    pub scale: u32,
    /// Enable debug overlay
    pub debug: bool,
}

/// Standalone player application state
pub struct PlayerApp {
    /// Configuration
    config: PlayerConfig,
    /// Window handle
    window: Option<Arc<Window>>,
    /// Console runner (owns graphics, audio, game session)
    runner: Option<ConsoleRunner<EmberwareZ>>,
    /// Input manager
    input_manager: InputManager,
    /// Debug overlay enabled
    debug_overlay: bool,
    /// Frame controller (pause/step)
    frame_controller: FrameController,
    /// Next scheduled simulation tick
    next_tick: Instant,
    /// Whether last simulation rendered
    last_sim_rendered: bool,
    /// Whether a redraw is needed
    needs_redraw: bool,
    /// Should exit
    should_exit: bool,
    /// Debug stats
    debug_stats: DebugStats,
    /// Game tick times for FPS calculation
    game_tick_times: Vec<Instant>,
    /// Last game tick
    last_game_tick: Instant,
}

impl PlayerApp {
    /// Create a new player app with the given configuration
    pub fn new(config: PlayerConfig) -> Self {
        let now = Instant::now();
        let input_config = emberware_core::app::config::load().input;

        Self {
            debug_overlay: config.debug,
            config,
            window: None,
            runner: None,
            input_manager: InputManager::new(input_config),
            frame_controller: FrameController::new(),
            next_tick: now,
            last_sim_rendered: false,
            needs_redraw: true,
            should_exit: false,
            debug_stats: DebugStats {
                frame_times: VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE),
                vram_limit: crate::console::VRAM_LIMIT,
                ..Default::default()
            },
            game_tick_times: Vec::with_capacity(120),
            last_game_tick: now,
        }
    }

    /// Get tick duration from the loaded game
    fn tick_duration(&self) -> Duration {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
        {
            return session.runtime.tick_duration();
        }
        Duration::from_secs_f64(1.0 / 60.0)
    }

    /// Handle keyboard input
    fn handle_key_input(&mut self, event: KeyEvent) {
        // Handle debug keys
        if event.state == ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    self.should_exit = true;
                }
                PhysicalKey::Code(KeyCode::F3) => {
                    self.debug_overlay = !self.debug_overlay;
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F5) => {
                    self.frame_controller.toggle_pause();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F6) => {
                    self.frame_controller.request_step();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F11) => {
                    if let Some(window) = &self.window {
                        if window.fullscreen().is_some() {
                            window.set_fullscreen(None);
                        } else {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
                }
                _ => {}
            }
        }

        // Forward to input manager
        let pressed = event.state == ElementState::Pressed;
        if let PhysicalKey::Code(key_code) = event.physical_key {
            self.input_manager.update_keyboard(key_code, pressed);
        }
    }

    /// Run game frame (update + render)
    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        let runner = self
            .runner
            .as_mut()
            .ok_or_else(|| RuntimeError("No runner".to_string()))?;

        let session = runner
            .session_mut()
            .ok_or_else(|| RuntimeError("No session".to_string()))?;

        // Get input and set it
        let raw_input = self.input_manager.get_player_input(0);
        let z_input = session.runtime.console().map_input(&raw_input);

        if let Some(game) = session.runtime.game_mut() {
            game.set_input(0, z_input);
        }
        let _ = session.runtime.add_local_input(0, z_input);

        // Check frame controller
        let should_run = self.frame_controller.should_run_tick();
        let time_scale = self.frame_controller.time_scale();

        let tick_start = Instant::now();
        let (ticks, _alpha) = if should_run {
            session
                .runtime
                .frame_with_time_scale(time_scale)
                .map_err(|e| RuntimeError(format!("Game frame error: {}", e)))?
        } else {
            (0, 0.0)
        };
        let tick_elapsed = tick_start.elapsed();

        let did_render = if ticks > 0 {
            // Track timing
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            // Clear and render
            if let Some(game) = session.runtime.game_mut() {
                game.console_state_mut().clear_frame();
            }

            let render_start = Instant::now();
            session
                .runtime
                .render()
                .map_err(|e| RuntimeError(format!("Render error: {}", e)))?;
            let render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;

            self.debug_stats.game_render_times.push_back(render_time_ms);
            while self.debug_stats.game_render_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_render_times.pop_front();
            }

            // Track tick times
            let now = Instant::now();
            for _ in 0..ticks {
                self.game_tick_times.push(now);
                if self.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                    self.game_tick_times.remove(0);
                }
            }
            self.last_game_tick = now;

            true
        } else {
            false
        };

        // Generate audio
        if did_render {
            let tick_rate = session.runtime.tick_rate();
            let sample_rate = session
                .runtime
                .audio()
                .map(|a| a.sample_rate())
                .unwrap_or(audio::OUTPUT_SAMPLE_RATE);

            let audio_buffer = if let Some(game) = session.runtime.game_mut() {
                let sounds: Vec<Option<audio::Sound>> = game.console_state().sounds.clone();
                let rollback_state = game.rollback_state_mut();

                let mut buffer = Vec::new();
                audio::generate_audio_frame(
                    &mut rollback_state.audio,
                    &sounds,
                    tick_rate,
                    sample_rate,
                    &mut buffer,
                );
                Some(buffer)
            } else {
                None
            };

            if let Some(buffer) = audio_buffer
                && let Some(audio) = session.runtime.audio_mut()
            {
                audio.push_samples(&buffer);
            }
        }

        // Check quit
        let quit_requested = session
            .runtime
            .game()
            .map(|g| g.state().quit_requested)
            .unwrap_or(false);

        Ok((!quit_requested, did_render))
    }

    /// Execute draw commands
    fn execute_draw_commands(&mut self) {
        if let Some(runner) = &mut self.runner {
            let (graphics, session_opt) = runner.graphics_and_session_mut();
            if let Some(session) = session_opt
                && let Some(game) = session.runtime.game_mut()
            {
                let state = game.console_state_mut();
                session
                    .resource_manager
                    .execute_draw_commands(graphics, state);
            }
        }
    }
}

impl ConsoleApp<EmberwareZ> for PlayerApp {
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()> {
        // Apply fullscreen if requested
        if self.config.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        // Set window title
        let rom_name = self
            .config
            .rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Emberware Z");
        window.set_title(&format!("Emberware Z - {}", rom_name));

        // Create console runner
        let console = EmberwareZ::new();
        let mut runner = ConsoleRunner::new(console, window.clone())?;

        // Load ROM
        let rom = load_rom(&self.config.rom_path)?;

        // Create console with datapack and load game
        let console_with_datapack = EmberwareZ::with_datapack(rom.data_pack);
        runner
            .load_game(console_with_datapack, &rom.code, 1)
            .context("Failed to load game")?;

        self.window = Some(window);
        self.runner = Some(runner);
        self.next_tick = Instant::now();

        tracing::info!("Game loaded: {}", self.config.rom_path.display());
        Ok(())
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(runner) = &mut self.runner {
                    runner.resize(size.width, size.height);
                }
                self.needs_redraw = true;
                false
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_key_input(key_event.clone());
                false
            }
            _ => false,
        }
    }

    fn has_active_game(&self) -> bool {
        self.runner.as_ref().is_some_and(|r| r.has_game())
    }

    fn next_tick(&self) -> Instant {
        self.next_tick
    }

    fn advance_simulation(&mut self) {
        self.last_sim_rendered = false;

        // Poll gamepad input
        self.input_manager.update();

        match self.run_game_frame() {
            Ok((game_running, did_render)) => {
                self.last_sim_rendered = did_render;

                if did_render {
                    self.execute_draw_commands();
                }

                if !game_running {
                    tracing::info!("Game requested quit");
                    self.should_exit = true;
                }
            }
            Err(e) => {
                tracing::error!("Runtime error: {}", e);
                self.should_exit = true;
            }
        }
    }

    fn update_next_tick(&mut self) {
        self.next_tick += self.tick_duration();
    }

    fn render(&mut self) {
        let runner = match &mut self.runner {
            Some(r) => r,
            None => return,
        };

        // Get surface texture
        let surface_texture = match runner.graphics_mut().get_current_texture() {
            Ok(tex) => tex,
            Err(e) => {
                tracing::warn!("Failed to get surface texture: {}", e);
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create encoder
        let mut encoder =
            runner
                .graphics()
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Player Frame Encoder"),
                });

        // Render game if we have new content
        if self.last_sim_rendered {
            let clear_color = get_clear_color(runner);
            let (graphics, session_opt) = runner.graphics_and_session_mut();

            if let Some(session) = session_opt
                && let Some(game) = session.runtime.game_mut()
            {
                let z_state = game.console_state_mut();
                let texture_map = &session.resource_manager.texture_map;
                graphics.render_frame(&mut encoder, z_state, texture_map, clear_color);
            }
        }

        // Blit to window
        runner.graphics().blit_to_window(&mut encoder, &view);

        // Submit
        runner
            .graphics()
            .queue()
            .submit(std::iter::once(encoder.finish()));

        surface_texture.present();
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn clear_needs_redraw(&mut self) {
        self.needs_redraw = false;
    }

    fn on_runtime_error(&mut self, error: RuntimeError) {
        tracing::error!("Runtime error: {}", error);
        self.should_exit = true;
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn request_exit(&mut self) {
        self.should_exit = true;
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Loaded ROM data
struct LoadedRom {
    code: Vec<u8>,
    data_pack: Option<Arc<ZDataPack>>,
}

/// Load ROM from path
fn load_rom(path: &Path) -> Result<LoadedRom> {
    if path.extension().and_then(|e| e.to_str()) == Some("ewz") {
        let ewz_bytes = std::fs::read(path).context("Failed to read .ewz ROM file")?;

        let rom = ZRom::from_bytes(&ewz_bytes).context("Failed to parse .ewz ROM")?;

        Ok(LoadedRom {
            code: rom.code,
            data_pack: rom.data_pack.map(Arc::new),
        })
    } else {
        // Raw WASM file
        let wasm = std::fs::read(path).context("Failed to read WASM file")?;
        Ok(LoadedRom {
            code: wasm,
            data_pack: None,
        })
    }
}

/// Get clear color from runner
fn get_clear_color(runner: &ConsoleRunner<EmberwareZ>) -> [f32; 4] {
    if let Some(session) = runner.session()
        && let Some(game) = session.runtime.game()
    {
        let z_state = game.console_state();
        return crate::ffi::unpack_rgba(z_state.init_config.clear_color);
    }
    [0.1, 0.1, 0.1, 1.0]
}

/// Run the standalone player
pub fn run(config: PlayerConfig) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Emberware Z player");
    tracing::info!("ROM: {}", config.rom_path.display());

    let app = PlayerApp::new(config);
    emberware_core::app::run(app)
}
