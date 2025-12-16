//! Standalone player for Emberware Z
//!
//! This module provides a minimal player application that can run .ewz ROM files
//! without the full library UI. Used by:
//! - `emberware-z` binary (standalone player)
//! - `ember run` command (development)
//! - Library process spawning

use std::cell::RefCell;
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
use emberware_core::debug::registry::RegisteredValue;
use emberware_core::debug::types::DebugValue;
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
    /// Debug overlay enabled (F3)
    debug_overlay: bool,
    /// Debug inspector panel (F4)
    debug_panel: emberware_core::debug::DebugPanel,
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
    /// egui context
    egui_ctx: egui::Context,
    /// egui-winit state
    egui_state: Option<egui_winit::State>,
    /// egui-wgpu renderer
    egui_renderer: Option<egui_wgpu::Renderer>,
}

impl PlayerApp {
    /// Create a new player app with the given configuration
    pub fn new(config: PlayerConfig) -> Self {
        let now = Instant::now();
        let input_config = emberware_core::app::config::load().input;

        Self {
            debug_overlay: config.debug,
            debug_panel: emberware_core::debug::DebugPanel::new(),
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
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
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
                PhysicalKey::Code(KeyCode::F4) => {
                    self.debug_panel.toggle();
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

        // Initialize egui for debug overlays
        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            runner.graphics().device(),
            runner.graphics().surface_format(),
            egui_wgpu::RendererOptions::default(),
        );
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);

        self.window = Some(window);
        self.runner = Some(runner);
        self.next_tick = Instant::now();

        tracing::info!("Game loaded: {}", self.config.rom_path.display());
        Ok(())
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        // Forward events to egui for debug panel interaction
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);
            if response.consumed {
                self.needs_redraw = true;
                return true;
            }
            if response.repaint {
                self.needs_redraw = true;
            }
        }

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

        // Render debug overlays via egui (on top of game)
        if self.debug_overlay || self.debug_panel.visible {
            // Extract registry and memory base address (no copy - WASM is idle during rendering)
            let (registry_opt, mem_base, mem_len, has_debug_callback) = {
                if let Some(session) = runner.session() {
                    if let Some(game) = session.runtime.game() {
                        let registry = game.store().data().debug_registry.clone();
                        let memory = game.store().data().game.memory;
                        let has_callback = game.has_debug_change_callback();
                        if let Some(mem) = memory {
                            // SAFETY: WASM is single-threaded and idle during egui rendering.
                            // Store base address as usize - memory won't move or change.
                            let mem_data = mem.data(game.store());
                            (
                                Some(registry),
                                mem_data.as_ptr() as usize,
                                mem_data.len(),
                                has_callback,
                            )
                        } else {
                            (Some(registry), 0usize, 0usize, has_callback)
                        }
                    } else {
                        (None, 0usize, 0usize, false)
                    }
                } else {
                    (None, 0usize, 0usize, false)
                }
            };

            // Pending writes to apply after egui closure
            let pending_writes: RefCell<Vec<(RegisteredValue, DebugValue)>> =
                RefCell::new(Vec::new());

            if let (Some(egui_state), Some(egui_renderer), Some(window)) =
                (&mut self.egui_state, &mut self.egui_renderer, &self.window)
            {
                let raw_input = egui_state.take_egui_input(window);

                // Extract fields for closure to avoid capturing all of self
                let debug_overlay = self.debug_overlay;
                let debug_stats = &self.debug_stats;
                let game_tick_times = &self.game_tick_times;
                let debug_panel = &mut self.debug_panel;
                let frame_controller = &mut self.frame_controller;

                let full_output = self.egui_ctx.run(raw_input, |ctx| {
                    if debug_overlay {
                        let frame_time_ms =
                            debug_stats.frame_times.back().copied().unwrap_or(16.67);
                        let render_fps = emberware_core::app::debug::calculate_fps(game_tick_times);
                        emberware_core::app::debug::render_debug_overlay(
                            ctx,
                            debug_stats,
                            true,
                            frame_time_ms,
                            render_fps,
                            render_fps,
                        );
                    }
                    if debug_panel.visible {
                        if let Some(ref registry) = registry_opt {
                            let registry_for_read = registry.clone();

                            let read_value = |reg_val: &RegisteredValue| -> Option<DebugValue> {
                                if mem_base == 0 || mem_len == 0 {
                                    return None;
                                }
                                let ptr = reg_val.wasm_ptr as usize;
                                let size = reg_val.value_type.byte_size();
                                if ptr + size > mem_len {
                                    return None;
                                }
                                // SAFETY: WASM is single-threaded and idle during egui rendering.
                                // The memory won't change until we return control to the game.
                                let bytes = unsafe {
                                    std::slice::from_raw_parts((mem_base + ptr) as *const u8, size)
                                };
                                Some(
                                    registry_for_read
                                        .read_value_from_slice(bytes, reg_val.value_type),
                                )
                            };

                            let write_value =
                                |reg_val: &RegisteredValue, new_val: &DebugValue| -> bool {
                                    pending_writes
                                        .borrow_mut()
                                        .push((reg_val.clone(), new_val.clone()));
                                    true
                                };

                            debug_panel.render(
                                ctx,
                                registry,
                                frame_controller,
                                read_value,
                                write_value,
                            );
                        }
                    }
                });

                egui_state.handle_platform_output(window, full_output.platform_output);

                // Tessellate and render egui
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [runner.graphics().width(), runner.graphics().height()],
                    pixels_per_point: window.scale_factor() as f32,
                };

                let tris = self
                    .egui_ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                for (id, delta) in &full_output.textures_delta.set {
                    egui_renderer.update_texture(
                        runner.graphics().device(),
                        runner.graphics().queue(),
                        *id,
                        delta,
                    );
                }

                egui_renderer.update_buffers(
                    runner.graphics().device(),
                    runner.graphics().queue(),
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                // Render egui on top of game
                {
                    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load, // Don't clear - overlay on top
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    let mut render_pass_static = render_pass.forget_lifetime();
                    egui_renderer.render(&mut render_pass_static, &tris, &screen_descriptor);
                }

                // Free textures
                for id in &full_output.textures_delta.free {
                    egui_renderer.free_texture(id);
                }
            }

            // Apply pending writes to WASM memory
            let writes = pending_writes.into_inner();
            if !writes.is_empty() {
                if let Some(session) = runner.session_mut() {
                    if let Some(game) = session.runtime.game_mut() {
                        let memory = game.store().data().game.memory;
                        if let Some(mem) = memory {
                            let registry = game.store().data().debug_registry.clone();
                            let data = mem.data_mut(game.store_mut());
                            for (reg_val, new_val) in &writes {
                                let ptr = reg_val.wasm_ptr as usize;
                                let size = reg_val.value_type.byte_size();
                                if ptr + size <= data.len() {
                                    registry
                                        .write_value_to_slice(&mut data[ptr..ptr + size], new_val);
                                }
                            }
                        }
                        if has_debug_callback {
                            game.call_on_debug_change();
                        }
                    }
                }
            }
        }

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
