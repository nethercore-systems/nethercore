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
use emberware_core::app::{
    DebugStats, FRAME_TIME_HISTORY_SIZE, GameError, GameErrorPhase, RuntimeError,
    parse_wasm_error,
};
use emberware_core::console::{Console, ConsoleResourceManager};
use emberware_core::debug::{ActionRequest, FrameController};
use emberware_core::debug::registry::RegisteredValue;
use emberware_core::debug::types::DebugValue;
use z_common::{ZDataPack, ZRom};

use crate::audio;
use crate::capture::{self, ScreenCapture};
use crate::console::EmberwareZ;
use crate::input::InputManager;

/// Simple settings panel for the player app
struct PlayerSettingsPanel {
    /// Whether the panel is visible
    visible: bool,
    /// Temporary scale mode (applied on change)
    scale_mode: emberware_core::app::config::ScaleMode,
    /// Whether fullscreen is enabled
    fullscreen: bool,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
}

impl PlayerSettingsPanel {
    fn new(config: &emberware_core::app::config::Config, fullscreen: bool) -> Self {
        Self {
            visible: false,
            scale_mode: config.video.scale_mode,
            fullscreen,
            master_volume: config.audio.master_volume,
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Render the settings panel and return true if settings changed
    fn render(&mut self, ctx: &egui::Context) -> SettingsAction {
        let mut action = SettingsAction::None;

        if !self.visible {
            return action;
        }

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(280.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(250.0);
                ui.heading("Video");
                ui.add_space(5.0);

                // Fullscreen toggle
                if ui.checkbox(&mut self.fullscreen, "Fullscreen (F11)").changed() {
                    action = SettingsAction::ToggleFullscreen(self.fullscreen);
                }
                ui.add_space(5.0);

                // Scale mode
                ui.label("Scale Mode:");
                let old_scale_mode = self.scale_mode;
                egui::ComboBox::from_id_salt("scale_mode")
                    .selected_text(match self.scale_mode {
                        emberware_core::app::config::ScaleMode::Stretch => "Stretch",
                        emberware_core::app::config::ScaleMode::Fit => "Fit",
                        emberware_core::app::config::ScaleMode::PixelPerfect => "Pixel Perfect",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.scale_mode,
                            emberware_core::app::config::ScaleMode::Fit,
                            "Fit (Maintain Aspect Ratio)",
                        );
                        ui.selectable_value(
                            &mut self.scale_mode,
                            emberware_core::app::config::ScaleMode::Stretch,
                            "Stretch (Fill Window)",
                        );
                        ui.selectable_value(
                            &mut self.scale_mode,
                            emberware_core::app::config::ScaleMode::PixelPerfect,
                            "Pixel Perfect (Integer Scaling)",
                        );
                    });
                if self.scale_mode != old_scale_mode {
                    action = SettingsAction::SetScaleMode(self.scale_mode);
                }

                ui.add_space(15.0);
                ui.heading("Audio");
                ui.add_space(5.0);

                // Volume slider
                let old_volume = self.master_volume;
                ui.add(
                    egui::Slider::new(&mut self.master_volume, 0.0..=1.0)
                        .text("Master Volume")
                        .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
                );
                if (self.master_volume - old_volume).abs() > f32::EPSILON {
                    action = SettingsAction::SetVolume(self.master_volume);
                }

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("Close (F2)").clicked() {
                        self.visible = false;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Save to Config").clicked() {
                            action = SettingsAction::SaveConfig;
                        }
                    });
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new("Press F2 to toggle this panel").weak().small());
            });

        action
    }
}

#[derive(Debug, Clone, Copy)]
enum SettingsAction {
    None,
    ToggleFullscreen(bool),
    SetScaleMode(emberware_core::app::config::ScaleMode),
    SetVolume(f32),
    SaveConfig,
}

/// Action from error screen UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorAction {
    /// No action
    None,
    /// Restart the game
    Restart,
    /// Quit the application
    Quit,
}

/// Render the error screen overlay
fn render_error_screen(ctx: &egui::Context, error: &GameError) -> ErrorAction {
    let mut action = ErrorAction::None;

    // Semi-transparent background overlay
    egui::Area::new(egui::Id::new("error_overlay_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ui.ctx().input(|i| i.screen_rect());
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
            );
        });

    egui::Window::new("Game Error")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(450.0)
        .show(ctx, |ui| {
            // Error icon and title
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").size(24.0).color(egui::Color32::YELLOW));
                ui.heading(&error.summary);
            });

            ui.add_space(10.0);

            // Phase and tick info
            ui.horizontal(|ui| {
                ui.label(format!("Phase: {}", error.phase));
                if let Some(tick) = error.tick {
                    ui.separator();
                    ui.label(format!("Tick: {}", tick));
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Suggestions
            if !error.suggestions.is_empty() {
                ui.label(egui::RichText::new("Possible causes:").strong());
                for suggestion in &error.suggestions {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        ui.label(suggestion);
                    });
                }
                ui.add_space(10.0);
            }

            // Collapsible error details
            egui::CollapsingHeader::new("Error Details")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut error.details.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                });

            // Stack trace if available
            if let Some(ref trace) = error.stack_trace {
                egui::CollapsingHeader::new("Stack Trace")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for frame in trace {
                                    ui.monospace(frame);
                                }
                            });
                    });
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Restart Game").clicked() {
                    action = ErrorAction::Restart;
                }
                ui.add_space(20.0);
                if ui.button("Quit").clicked() {
                    action = ErrorAction::Quit;
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("Press Escape to quit").weak().small());
        });

    action
}

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
    /// Scale mode for render target to window (loaded from config)
    scale_mode: emberware_core::app::config::ScaleMode,
    /// Settings panel (F2)
    settings_panel: PlayerSettingsPanel,
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
    /// Cached ROM data for restart capability
    loaded_rom: Option<LoadedRom>,
    /// Current error state (None = playing normally)
    error_state: Option<GameError>,
    /// Screen capture manager (screenshots/GIFs)
    capture: ScreenCapture,
    /// Screenshot key (parsed from config)
    screenshot_key: KeyCode,
    /// GIF toggle key (parsed from config)
    gif_toggle_key: KeyCode,
}

impl PlayerApp {
    /// Create a new player app with the given configuration
    pub fn new(config: PlayerConfig) -> Self {
        let now = Instant::now();
        let app_config = emberware_core::app::config::load();
        let input_config = app_config.input.clone();
        let scale_mode = app_config.video.scale_mode;
        let settings_panel = PlayerSettingsPanel::new(&app_config, config.fullscreen);

        // Validate keybindings and log any conflicts
        let warnings = emberware_core::app::config::validate_keybindings(&app_config);
        for warning in warnings {
            tracing::warn!("Keybinding conflict: {}", warning);
        }

        // Parse capture keybindings
        let screenshot_key = parse_key_code(&app_config.capture.screenshot).unwrap_or_else(|| {
            tracing::warn!(
                "Invalid screenshot key '{}', using F9",
                app_config.capture.screenshot
            );
            KeyCode::F9
        });
        let gif_toggle_key = parse_key_code(&app_config.capture.gif_toggle).unwrap_or_else(|| {
            tracing::warn!(
                "Invalid GIF toggle key '{}', using F10",
                app_config.capture.gif_toggle
            );
            KeyCode::F10
        });

        // Initialize capture manager with initial game name from path
        // (will be updated with actual title when ROM is loaded)
        let initial_game_name = config
            .rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("game")
            .to_string();
        let capture = ScreenCapture::new(
            app_config.capture.gif_fps,
            app_config.capture.gif_max_seconds,
            initial_game_name,
        );

        Self {
            debug_overlay: config.debug,
            debug_panel: emberware_core::debug::DebugPanel::new(),
            config,
            window: None,
            runner: None,
            input_manager: InputManager::new(input_config),
            scale_mode,
            settings_panel,
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
            loaded_rom: None,
            error_state: None,
            capture,
            screenshot_key,
            gif_toggle_key,
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

    /// Restart the game after an error
    ///
    /// Clears the error state, unloads the current game, and reloads it
    /// from the cached ROM data.
    fn restart_game(&mut self) {
        // Clear error state
        self.error_state = None;

        // Get cached ROM or reload from disk
        let rom = if let Some(ref rom) = self.loaded_rom {
            rom.clone()
        } else if let Ok(rom) = load_rom(&self.config.rom_path) {
            self.loaded_rom = Some(rom.clone());
            rom
        } else {
            tracing::error!("Failed to reload ROM for restart");
            self.should_exit = true;
            return;
        };

        // Unload current game
        if let Some(runner) = &mut self.runner {
            runner.unload_game();
        }

        // Create new console with datapack and reload game
        let console = EmberwareZ::with_datapack(rom.data_pack.clone());

        if let Some(runner) = &mut self.runner {
            if let Err(e) = runner.load_game(console, &rom.code, 1) {
                tracing::error!("Failed to restart game: {}", e);
                // Show error for restart failure
                self.error_state = Some(GameError {
                    summary: "Restart Failed".to_string(),
                    details: format!("{:#}", e),
                    stack_trace: None,
                    tick: None,
                    phase: GameErrorPhase::Init,
                    suggestions: vec![
                        "The ROM file may be corrupted".to_string(),
                        "Try closing and reopening the player".to_string(),
                    ],
                });
                return;
            }

            // Restore audio volume from settings
            if let Some(session) = runner.session_mut() {
                if let Some(audio) = session.runtime.audio_mut() {
                    audio.set_master_volume(self.settings_panel.master_volume);
                }
            }
        }

        // Reset timing
        self.next_tick = Instant::now();
        self.needs_redraw = true;

        tracing::info!("Game restarted successfully");
    }

    /// Handle keyboard input
    fn handle_key_input(&mut self, event: KeyEvent) {
        // Handle debug keys
        if event.state == ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    self.should_exit = true;
                }
                PhysicalKey::Code(KeyCode::F2) => {
                    self.settings_panel.toggle();
                    self.needs_redraw = true;
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
                        let is_fullscreen = window.fullscreen().is_some();
                        if is_fullscreen {
                            window.set_fullscreen(None);
                            self.settings_panel.fullscreen = false;
                        } else {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            self.settings_panel.fullscreen = true;
                        }
                    }
                }
                // Screenshot (configurable key, default F9)
                PhysicalKey::Code(key) if key == self.screenshot_key => {
                    self.capture.request_screenshot();
                    tracing::info!("Screenshot requested");
                    self.needs_redraw = true;
                }
                // GIF toggle (configurable key, default F10)
                PhysicalKey::Code(key) if key == self.gif_toggle_key => {
                    if let Some(runner) = &self.runner {
                        let (w, h) = runner.graphics().render_target_dimensions();
                        self.capture.toggle_recording(w, h);
                        if self.capture.is_recording() {
                            tracing::info!("GIF recording started");
                        } else {
                            tracing::info!("GIF recording stopped, saving...");
                        }
                        self.needs_redraw = true;
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
                // Use split borrows to avoid cloning the sounds vector
                let (ffi_state, rollback_state) = game.ffi_and_rollback_mut();

                let mut buffer = Vec::new();
                audio::generate_audio_frame(
                    &mut rollback_state.audio,
                    &ffi_state.sounds,
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

        // Load ROM and cache for restart capability (do this first to get game name)
        let rom = load_rom(&self.config.rom_path)?;
        self.loaded_rom = Some(rom.clone());

        // Set window title using game name from ROM metadata
        window.set_title(&format!("Emberware Z - {}", rom.game_name));

        // Update capture manager with the actual game name
        self.capture.set_game_name(rom.game_name.clone());

        // Create console runner
        let console = EmberwareZ::new();
        let specs = console.specs();

        // Set minimum window size based on console's render resolution (in physical pixels)
        let (render_width, render_height) = specs.resolutions[specs.default_resolution];
        window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(
            render_width,
            render_height,
        )));

        let mut runner = ConsoleRunner::new(console, window.clone())?;

        // Apply scale mode from user config
        runner.graphics_mut().set_scale_mode(self.scale_mode);

        // Create console with datapack and load game (rom already loaded above)
        let console_with_datapack = EmberwareZ::with_datapack(rom.data_pack.clone());
        runner
            .load_game(console_with_datapack, &rom.code, 1)
            .context("Failed to load game")?;

        // Apply audio volume from config
        if let Some(session) = runner.session_mut() {
            if let Some(audio) = session.runtime.audio_mut() {
                audio.set_master_volume(self.settings_panel.master_volume);
            }
        }

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

        // If in error state, don't advance - just wait for user action
        if self.error_state.is_some() {
            return;
        }

        // Poll gamepad input
        self.input_manager.update();

        // Get current tick count before running (for error reporting)
        let tick_before = self
            .runner
            .as_ref()
            .and_then(|r| r.session())
            .and_then(|s| s.runtime.game())
            .map(|g| g.state().tick_count);

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
                // Parse error and transition to error state instead of exiting
                let error_msg = e.0.clone();

                // Determine phase from error message
                let phase = if error_msg.contains("Render error") {
                    GameErrorPhase::Render
                } else {
                    GameErrorPhase::Update
                };

                // Parse into structured GameError
                let game_error = parse_wasm_error(&anyhow::anyhow!("{}", error_msg), tick_before, phase);

                tracing::error!("Game error: {}", game_error);
                self.error_state = Some(game_error);
                self.needs_redraw = true;
                // DO NOT set should_exit = true - show error screen instead
            }
        }
    }

    fn update_next_tick(&mut self) {
        self.next_tick += self.tick_duration();
    }

    fn render(&mut self) {
        // Track if restart was requested (handled after runner borrow ends)
        let mut restart_requested = false;

        // Begin block to scope runner borrow (restart_game needs &mut self after)
        {
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

        // Render overlays via egui (on top of game)
        if self.debug_overlay || self.debug_panel.visible || self.settings_panel.visible || self.error_state.is_some() {
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

            // Pending action to apply after egui closure
            let pending_action: RefCell<Option<ActionRequest>> = RefCell::new(None);

            // Settings action to apply after egui closure
            let settings_action: RefCell<SettingsAction> = RefCell::new(SettingsAction::None);

            // Error action to apply after egui closure
            let error_action: RefCell<ErrorAction> = RefCell::new(ErrorAction::None);

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
                let settings_panel = &mut self.settings_panel;
                let error_state_ref = &self.error_state;

                let full_output = self.egui_ctx.run(raw_input, |ctx| {
                    // Render settings panel first (so it's on top)
                    let action = settings_panel.render(ctx);
                    if !matches!(action, SettingsAction::None) {
                        *settings_action.borrow_mut() = action;
                    }
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

                            let (_changed, action) = debug_panel.render(
                                ctx,
                                registry,
                                frame_controller,
                                read_value,
                                write_value,
                            );
                            if let Some(action) = action {
                                *pending_action.borrow_mut() = Some(action);
                            }
                        }
                    }

                    // Render error screen if in error state (highest priority, on top of everything)
                    if let Some(error) = error_state_ref {
                        let action = render_error_screen(ctx, error);
                        if action != ErrorAction::None {
                            *error_action.borrow_mut() = action;
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

            // Apply pending debug action
            if let Some(action_req) = pending_action.into_inner() {
                if let Some(session) = runner.session_mut() {
                    if let Some(game) = session.runtime.game_mut() {
                        if let Err(e) = game.call_action(&action_req.func_name, &action_req.args) {
                            tracing::warn!("Debug action '{}' failed: {}", action_req.func_name, e);
                        }
                    }
                }
            }

            // Apply settings actions
            match settings_action.into_inner() {
                SettingsAction::None => {}
                SettingsAction::ToggleFullscreen(fullscreen) => {
                    if let Some(window) = &self.window {
                        if fullscreen {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        } else {
                            window.set_fullscreen(None);
                        }
                    }
                }
                SettingsAction::SetScaleMode(scale_mode) => {
                    self.scale_mode = scale_mode;
                    runner.graphics_mut().set_scale_mode(scale_mode);
                }
                SettingsAction::SetVolume(volume) => {
                    if let Some(session) = runner.session_mut() {
                        if let Some(audio) = session.runtime.audio_mut() {
                            audio.set_master_volume(volume);
                        }
                    }
                }
                SettingsAction::SaveConfig => {
                    // Save current settings to config file
                    let mut config = emberware_core::app::config::load();
                    config.video.scale_mode = self.settings_panel.scale_mode;
                    config.video.fullscreen = self.settings_panel.fullscreen;
                    config.audio.master_volume = self.settings_panel.master_volume;
                    if let Err(e) = emberware_core::app::config::save(&config) {
                        tracing::error!("Failed to save config: {}", e);
                    } else {
                        tracing::info!("Settings saved to config");
                    }
                }
            }

            // Apply error actions (restart is deferred)
            match error_action.into_inner() {
                ErrorAction::None => {}
                ErrorAction::Restart => {
                    restart_requested = true;
                }
                ErrorAction::Quit => {
                    self.should_exit = true;
                }
            }
        }

        // Submit
        runner
            .graphics()
            .queue()
            .submit(std::iter::once(encoder.finish()));

        surface_texture.present();

        // Process screen capture (screenshot/GIF) if needed
        if self.capture.needs_capture() {
            let (width, height) = runner.graphics().render_target_dimensions();
            let pixels = capture::read_render_target_pixels(
                runner.graphics().device(),
                runner.graphics().queue(),
                runner.graphics().render_target_texture(),
                width,
                height,
            );
            self.capture.process_frame(pixels, width, height);
        }

        // Check for capture save results
        if let Some(result) = self.capture.poll_save_result() {
            match result {
                capture::SaveResult::Screenshot(Ok(path)) => {
                    tracing::info!("Screenshot saved: {}", path.display());
                }
                capture::SaveResult::Screenshot(Err(e)) => {
                    tracing::error!("Failed to save screenshot: {}", e);
                }
                capture::SaveResult::Gif(Ok(path)) => {
                    tracing::info!("GIF saved: {}", path.display());
                }
                capture::SaveResult::Gif(Err(e)) => {
                    tracing::error!("Failed to save GIF: {}", e);
                }
            }
        }
        } // End runner borrow block

        // Handle deferred restart (after runner borrow ends)
        if restart_requested {
            self.restart_game();
        }
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
        // Get current tick count for error reporting
        let tick = self
            .runner
            .as_ref()
            .and_then(|r| r.session())
            .and_then(|s| s.runtime.game())
            .map(|g| g.state().tick_count);

        // Parse into structured GameError
        let game_error = parse_wasm_error(
            &anyhow::anyhow!("{}", error.0),
            tick,
            GameErrorPhase::Update, // Default to Update phase
        );

        tracing::error!("Runtime error: {}", game_error);
        self.error_state = Some(game_error);
        self.needs_redraw = true;
        // DO NOT set should_exit = true - show error screen instead
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
#[derive(Clone)]
struct LoadedRom {
    code: Vec<u8>,
    data_pack: Option<Arc<ZDataPack>>,
    /// Game title (from ROM metadata or file stem fallback)
    game_name: String,
}

/// Load ROM from path
fn load_rom(path: &Path) -> Result<LoadedRom> {
    // Fallback name from file stem
    let fallback_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Emberware Z")
        .to_string();

    if path.extension().and_then(|e| e.to_str()) == Some("ewz") {
        let ewz_bytes = std::fs::read(path).context("Failed to read .ewz ROM file")?;

        let rom = ZRom::from_bytes(&ewz_bytes).context("Failed to parse .ewz ROM")?;

        // Use metadata title, fall back to file stem if empty
        let game_name = if rom.metadata.title.is_empty() {
            fallback_name
        } else {
            rom.metadata.title.clone()
        };

        Ok(LoadedRom {
            code: rom.code,
            data_pack: rom.data_pack.map(Arc::new),
            game_name,
        })
    } else {
        // Raw WASM file - use file stem as name
        let wasm = std::fs::read(path).context("Failed to read WASM file")?;
        Ok(LoadedRom {
            code: wasm,
            data_pack: None,
            game_name: fallback_name,
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

/// Parse a key string to a winit KeyCode.
///
/// Supports F1-F12 keys. Returns None for unrecognized keys.
fn parse_key_code(s: &str) -> Option<KeyCode> {
    match s.to_uppercase().as_str() {
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
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
