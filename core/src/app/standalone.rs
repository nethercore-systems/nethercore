//! Generic standalone player for any console
//!
//! Provides a console-agnostic player application that can run ROM files
//! for any console that implements the Console trait and required support traits.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window};

use ggrs::PlayerType;

use crate::capture::{CaptureSupport, ScreenCapture, read_render_target_pixels};
use crate::console::{Audio, AudioGenerator, Console, ConsoleResourceManager};
use crate::debug::registry::RegisteredValue;
use crate::debug::types::DebugValue;
use crate::debug::{ActionRequest, FrameController};
use crate::rollback::{ConnectionMode, ConnectionQuality, LocalSocket, RollbackSession, SessionConfig, SessionType};
use crate::runner::ConsoleRunner;

use super::config::ScaleMode;
use super::event_loop::ConsoleApp;
use super::{
    DebugStats, FRAME_TIME_HISTORY_SIZE, GameError, GameErrorPhase, RuntimeError,
    parse_wasm_error,
};

/// Trait for graphics backends that support standalone player functionality.
///
/// This extends the base Graphics + CaptureSupport traits with methods
/// required for the standalone player's rendering pipeline.
pub trait StandaloneGraphicsSupport: CaptureSupport {
    /// Get the surface texture format for egui rendering.
    fn surface_format(&self) -> wgpu::TextureFormat;

    /// Get current window width.
    fn width(&self) -> u32;

    /// Get current window height.
    fn height(&self) -> u32;

    /// Get the current surface texture for rendering.
    fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;

    /// Blit the render target to the window surface with scaling.
    fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);

    /// Set the scale mode for render target to window.
    fn set_scale_mode(&mut self, mode: ScaleMode);
}

/// Trait for loading ROM files for a specific console.
///
/// Each console implements this to parse its ROM format.
pub trait RomLoader: Sized {
    /// Console type this loader works with.
    type Console: Console + Clone;

    /// Load a ROM file from the given path.
    fn load_rom(path: &Path) -> Result<LoadedRom<Self::Console>>;
}

/// Loaded ROM data ready for execution.
#[derive(Clone)]
pub struct LoadedRom<C: Console + Clone> {
    /// WASM bytecode
    pub code: Vec<u8>,
    /// Console instance configured for this ROM
    pub console: C,
    /// Game title (from ROM metadata or file stem fallback)
    pub game_name: String,
}

/// Configuration for standalone player.
pub struct StandaloneConfig {
    /// ROM file path
    pub rom_path: PathBuf,
    /// Start in fullscreen
    pub fullscreen: bool,
    /// Integer scaling factor
    pub scale: u32,
    /// Enable debug overlay
    pub debug: bool,
    /// Number of players (1-4)
    pub num_players: usize,
    /// Input delay in frames (0-10)
    pub input_delay: usize,
    /// Connection mode for multiplayer
    pub connection_mode: ConnectionMode,
}

/// Settings action from settings panel.
#[derive(Debug, Clone, Copy)]
enum SettingsAction {
    None,
    ToggleFullscreen(bool),
    SetScaleMode(ScaleMode),
    SetVolume(f32),
    SaveConfig,
}

/// Action from error screen UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorAction {
    None,
    Restart,
    Quit,
}

/// Simple settings panel for the standalone player.
struct PlayerSettingsPanel {
    visible: bool,
    scale_mode: ScaleMode,
    fullscreen: bool,
    master_volume: f32,
}

impl PlayerSettingsPanel {
    fn new(config: &super::config::Config, fullscreen: bool) -> Self {
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

                if ui.checkbox(&mut self.fullscreen, "Fullscreen (F11)").changed() {
                    action = SettingsAction::ToggleFullscreen(self.fullscreen);
                }
                ui.add_space(5.0);

                ui.label("Scale Mode:");
                let old_scale_mode = self.scale_mode;
                egui::ComboBox::from_id_salt("scale_mode")
                    .selected_text(match self.scale_mode {
                        ScaleMode::Stretch => "Stretch",
                        ScaleMode::Fit => "Fit",
                        ScaleMode::PixelPerfect => "Pixel Perfect",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.scale_mode, ScaleMode::Fit, "Fit (Maintain Aspect Ratio)");
                        ui.selectable_value(&mut self.scale_mode, ScaleMode::Stretch, "Stretch (Fill Window)");
                        ui.selectable_value(&mut self.scale_mode, ScaleMode::PixelPerfect, "Pixel Perfect (Integer Scaling)");
                    });
                if self.scale_mode != old_scale_mode {
                    action = SettingsAction::SetScaleMode(self.scale_mode);
                }

                ui.add_space(15.0);
                ui.heading("Audio");
                ui.add_space(5.0);

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

/// Render the error screen overlay.
fn render_error_screen(ctx: &egui::Context, error: &GameError) -> ErrorAction {
    let mut action = ErrorAction::None;

    egui::Area::new(egui::Id::new("error_overlay_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ctx.input(|i| i.raw.viewport().inner_rect)
                .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, ctx.used_size()));
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
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").size(24.0).color(egui::Color32::YELLOW));
                ui.heading(&error.summary);
            });

            ui.add_space(10.0);

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

/// Parse a key string to a winit KeyCode.
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

/// Generic standalone player application.
///
/// This provides a complete player implementation for any console that:
/// - Implements the `Console` trait
/// - Has a `Graphics` type that implements `StandaloneGraphicsSupport`
/// - Has a ROM loader that implements `RomLoader`
pub struct StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    config: StandaloneConfig,
    window: Option<Arc<Window>>,
    runner: Option<ConsoleRunner<C>>,
    input_manager: super::InputManager,
    scale_mode: ScaleMode,
    settings_panel: PlayerSettingsPanel,
    debug_overlay: bool,
    debug_panel: crate::debug::DebugPanel,
    frame_controller: FrameController,
    next_tick: Instant,
    last_sim_rendered: bool,
    needs_redraw: bool,
    should_exit: bool,
    debug_stats: DebugStats,
    game_tick_times: Vec<Instant>,
    last_game_tick: Instant,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    loaded_rom: Option<LoadedRom<C>>,
    error_state: Option<GameError>,
    capture: ScreenCapture,
    screenshot_key: KeyCode,
    gif_toggle_key: KeyCode,
    /// Network statistics overlay visibility (F12)
    network_overlay_visible: bool,
    _vram_limit: usize,
    _loader_marker: std::marker::PhantomData<L>,
}

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Create a new standalone app with the given configuration.
    pub fn new(config: StandaloneConfig, vram_limit: usize) -> Self {
        let now = Instant::now();
        let app_config = super::config::load();
        let input_config = app_config.input.clone();
        let scale_mode = app_config.video.scale_mode;
        let settings_panel = PlayerSettingsPanel::new(&app_config, config.fullscreen);

        let warnings = super::config::validate_keybindings(&app_config);
        for warning in warnings {
            tracing::warn!("Keybinding conflict: {}", warning);
        }

        let screenshot_key = parse_key_code(&app_config.capture.screenshot).unwrap_or_else(|| {
            tracing::warn!("Invalid screenshot key '{}', using F9", app_config.capture.screenshot);
            KeyCode::F9
        });
        let gif_toggle_key = parse_key_code(&app_config.capture.gif_toggle).unwrap_or_else(|| {
            tracing::warn!("Invalid GIF toggle key '{}', using F10", app_config.capture.gif_toggle);
            KeyCode::F10
        });

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
            debug_panel: crate::debug::DebugPanel::new(),
            config,
            window: None,
            runner: None,
            input_manager: super::InputManager::new(input_config),
            scale_mode,
            settings_panel,
            frame_controller: FrameController::new(),
            next_tick: now,
            last_sim_rendered: false,
            needs_redraw: true,
            should_exit: false,
            debug_stats: DebugStats {
                frame_times: VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE),
                vram_limit,
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
            network_overlay_visible: false,
            _vram_limit: vram_limit,
            _loader_marker: std::marker::PhantomData,
        }
    }

    fn tick_duration(&self) -> Duration {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
        {
            return session.runtime.tick_duration();
        }
        Duration::from_secs_f64(1.0 / 60.0)
    }

    fn restart_game(&mut self) {
        self.error_state = None;

        // Try to reload ROM if not already loaded
        if self.loaded_rom.is_none() {
            match L::load_rom(&self.config.rom_path) {
                Ok(rom) => {
                    self.loaded_rom = Some(rom);
                }
                Err(_) => {
                    tracing::error!("Failed to reload ROM for restart");
                    self.should_exit = true;
                    return;
                }
            }
        }

        let rom = match &self.loaded_rom {
            Some(rom) => rom.clone(),
            None => {
                tracing::error!("No ROM loaded for restart");
                self.should_exit = true;
                return;
            }
        };

        if let Some(runner) = &mut self.runner {
            runner.unload_game();
        }

        let console = rom.console.clone();

        if let Some(runner) = &mut self.runner {
            if let Err(e) = runner.load_game(console, &rom.code, 1) {
                tracing::error!("Failed to restart game: {}", e);
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

            if let Some(session) = runner.session_mut() {
                if let Some(audio) = session.runtime.audio_mut() {
                    audio.set_master_volume(self.settings_panel.master_volume);
                }
            }
        }

        self.next_tick = Instant::now();
        self.needs_redraw = true;
        tracing::info!("Game restarted successfully");
    }

    fn handle_key_input(&mut self, event: KeyEvent) {
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
                    self.network_overlay_visible = !self.network_overlay_visible;
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
                PhysicalKey::Code(KeyCode::F12) => {
                    self.debug_panel.toggle();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(key) if key == self.screenshot_key => {
                    self.capture.request_screenshot();
                    tracing::info!("Screenshot requested");
                    self.needs_redraw = true;
                }
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

        let pressed = event.state == ElementState::Pressed;
        if let PhysicalKey::Code(key_code) = event.physical_key {
            self.input_manager.update_keyboard(key_code, pressed);
        }
    }

    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        let runner = self
            .runner
            .as_mut()
            .ok_or_else(|| RuntimeError("No runner".to_string()))?;

        let session = runner
            .session_mut()
            .ok_or_else(|| RuntimeError("No session".to_string()))?;

        let raw_input = self.input_manager.get_player_input(0);
        let console_input = session.runtime.console().map_input(&raw_input);

        if let Some(game) = session.runtime.game_mut() {
            game.set_input(0, console_input);
        }
        let _ = session.runtime.add_local_input(0, console_input);

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
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            if let Some(game) = session.runtime.game_mut() {
                C::clear_frame_state(game.console_state_mut());
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

        // Generate audio using the console's AudioGenerator
        if did_render {
            let tick_rate = session.runtime.tick_rate();
            let sample_rate = session
                .runtime
                .audio()
                .map(|a| a.sample_rate())
                .unwrap_or_else(C::AudioGenerator::default_sample_rate);

            let audio_buffer = if let Some(game) = session.runtime.game_mut() {
                let (ffi_state, rollback_state) = game.ffi_and_rollback_mut();
                let mut buffer = Vec::new();
                C::AudioGenerator::generate_frame(
                    rollback_state,
                    ffi_state,
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

        let quit_requested = session
            .runtime
            .game()
            .map(|g| g.state().quit_requested)
            .unwrap_or(false);

        Ok((!quit_requested, did_render))
    }

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

    fn get_clear_color(&self) -> [f32; 4] {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
            && let Some(game) = session.runtime.game()
        {
            return C::clear_color_from_state(game.console_state());
        }
        [0.1, 0.1, 0.1, 1.0]
    }
}

impl<C, L> ConsoleApp<C> for StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        if self.config.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let rom = L::load_rom(&self.config.rom_path)?;
        self.loaded_rom = Some(rom.clone());

        window.set_title(&format!("{} - {}", C::specs().name, rom.game_name));
        self.capture.set_game_name(rom.game_name.clone());

        let console = rom.console.clone();
        let specs = C::specs();

        let (render_width, render_height) = specs.resolution;
        window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(
            render_width,
            render_height,
        )));

        let mut runner = ConsoleRunner::new(console.clone(), window.clone())?;
        runner.graphics_mut().set_scale_mode(self.scale_mode);

        // Create session based on connection mode
        match &self.config.connection_mode {
            ConnectionMode::Local => {
                // Standard local session (no rollback)
                runner
                    .load_game(rom.console, &rom.code, self.config.num_players)
                    .context("Failed to load game")?;
            }
            ConnectionMode::SyncTest { check_distance } => {
                // Sync test session for determinism testing
                let session_config = SessionConfig::sync_test_with_params(
                    self.config.num_players,
                    self.config.input_delay,
                );
                let session = RollbackSession::new_sync_test(session_config, specs.ram_limit)
                    .context("Failed to create sync test session")?;
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with sync test session")?;
                tracing::info!(
                    "Sync test mode enabled (check_distance: {})",
                    check_distance
                );
            }
            ConnectionMode::P2P {
                bind_port,
                peer_port,
                local_player,
            } => {
                // Local P2P testing mode
                let mut socket = LocalSocket::bind(&format!("127.0.0.1:{}", bind_port))
                    .context("Failed to bind local socket")?;
                socket
                    .connect(&format!("127.0.0.1:{}", peer_port))
                    .context("Failed to connect to peer")?;

                let peer_addr = format!("127.0.0.1:{}", peer_port);
                let session_config = SessionConfig::online(2)
                    .with_input_delay(self.config.input_delay);

                let players = vec![
                    (
                        0,
                        if *local_player == 0 {
                            PlayerType::Local
                        } else {
                            PlayerType::Remote(peer_addr.clone())
                        },
                    ),
                    (
                        1,
                        if *local_player == 1 {
                            PlayerType::Local
                        } else {
                            PlayerType::Remote(peer_addr)
                        },
                    ),
                ];

                let session =
                    RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
                        .context("Failed to create P2P session")?;
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with P2P session")?;
                tracing::info!(
                    "P2P mode: bind={}, peer={}, local_player={}",
                    bind_port,
                    peer_port,
                    local_player
                );
            }
            ConnectionMode::Host { port } => {
                // Host mode - bind and wait for connection
                // For now, create socket and start in waiting state
                // TODO: Implement proper connection waiting UI
                let _socket = LocalSocket::bind(&format!("0.0.0.0:{}", port))
                    .context("Failed to bind host socket")?;
                tracing::info!("Hosting on port {}, waiting for connection...", port);

                // For MVP, we need to wait for peer before creating session
                // This will be improved in Phase 0 with proper connection UI
                anyhow::bail!(
                    "Host mode not yet fully implemented. Use --p2p for local testing."
                );
            }
            ConnectionMode::Join { address } => {
                // Join mode - connect to host
                // TODO: Implement proper connection UI
                let mut socket = LocalSocket::bind("0.0.0.0:0")
                    .context("Failed to bind client socket")?;
                socket
                    .connect(address)
                    .context("Failed to connect to host")?;
                tracing::info!("Joining game at {}", address);

                // For MVP, create P2P session immediately
                // This will be improved in Phase 0 with proper connection flow
                let session_config = SessionConfig::online(2)
                    .with_input_delay(self.config.input_delay);

                let players = vec![
                    (0, PlayerType::Remote(address.clone())),
                    (1, PlayerType::Local),
                ];

                let session =
                    RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
                        .context("Failed to create P2P session")?;
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with P2P session")?;
            }
        }

        if let Some(session) = runner.session_mut() {
            if let Some(audio) = session.runtime.audio_mut() {
                audio.set_master_volume(self.settings_panel.master_volume);
            }
        }

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
            WindowEvent::KeyboardInput { event: key_event, .. } => {
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

        if self.error_state.is_some() {
            return;
        }

        self.input_manager.update();

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
                let error_msg = e.0.clone();
                let phase = if error_msg.contains("Render error") {
                    GameErrorPhase::Render
                } else {
                    GameErrorPhase::Update
                };

                let game_error = parse_wasm_error(&anyhow::anyhow!("{}", error_msg), tick_before, phase);
                tracing::error!("Game error: {}", game_error);
                self.error_state = Some(game_error);
                self.needs_redraw = true;
            }
        }
    }

    fn update_next_tick(&mut self) {
        self.next_tick += self.tick_duration();
    }

    fn render(&mut self) {
        let mut restart_requested = false;

        // Get clear color before borrowing runner mutably
        let clear_color = self.get_clear_color();

        {
            let runner = match &mut self.runner {
                Some(r) => r,
                None => return,
            };

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

            let mut encoder = runner
                .graphics()
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Standalone Frame Encoder"),
                });

            // Render game if we have new content
            if self.last_sim_rendered {
                let (graphics, session_opt) = runner.graphics_and_session_mut();

                if let Some(session) = session_opt
                    && let Some(game) = session.runtime.game()
                {
                    let state = game.console_state();
                    session.resource_manager.render_game_to_target(
                        graphics,
                        &mut encoder,
                        state,
                        clear_color,
                    );
                }
            }

            runner.graphics().blit_to_window(&mut encoder, &view);

            // Render overlays via egui
            if self.debug_overlay || self.debug_panel.visible || self.settings_panel.visible || self.error_state.is_some() || self.network_overlay_visible {
                let (registry_opt, mem_base, mem_len, has_debug_callback) = {
                    if let Some(session) = runner.session() {
                        if let Some(game) = session.runtime.game() {
                            let registry = game.store().data().debug_registry.clone();
                            let memory = game.store().data().game.memory;
                            let has_callback = game.has_debug_change_callback();
                            if let Some(mem) = memory {
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

                let pending_writes: RefCell<Vec<(RegisteredValue, DebugValue)>> = RefCell::new(Vec::new());
                let pending_action: RefCell<Option<ActionRequest>> = RefCell::new(None);
                let settings_action: RefCell<SettingsAction> = RefCell::new(SettingsAction::None);
                let error_action: RefCell<ErrorAction> = RefCell::new(ErrorAction::None);

                if let (Some(egui_state), Some(egui_renderer), Some(window)) =
                    (&mut self.egui_state, &mut self.egui_renderer, &self.window)
                {
                    let raw_input = egui_state.take_egui_input(window);

                    let debug_overlay = self.debug_overlay;
                    let debug_stats = &self.debug_stats;
                    let game_tick_times = &self.game_tick_times;
                    let debug_panel = &mut self.debug_panel;
                    let frame_controller = &mut self.frame_controller;
                    let settings_panel = &mut self.settings_panel;
                    let error_state_ref = &self.error_state;
                    let network_overlay_visible = self.network_overlay_visible;

                    // Get network session info for overlay
                    let (session_type, network_stats, local_players, total_rollbacks, current_frame) = {
                        if let Some(game_session) = runner.session() {
                            if let Some(rollback) = game_session.runtime.session() {
                                (
                                    rollback.session_type(),
                                    rollback.all_player_stats().to_vec(),
                                    rollback.local_players().to_vec(),
                                    rollback.total_rollback_frames(),
                                    rollback.current_frame(),
                                )
                            } else {
                                (SessionType::Local, Vec::new(), Vec::new(), 0, 0)
                            }
                        } else {
                            (SessionType::Local, Vec::new(), Vec::new(), 0, 0)
                        }
                    };

                    let full_output = self.egui_ctx.run(raw_input, |ctx| {
                        let action = settings_panel.render(ctx);
                        if !matches!(action, SettingsAction::None) {
                            *settings_action.borrow_mut() = action;
                        }
                        if debug_overlay {
                            let frame_time_ms = debug_stats.frame_times.back().copied().unwrap_or(16.67);
                            let render_fps = super::debug::calculate_fps(game_tick_times);
                            super::debug::render_debug_overlay(
                                ctx,
                                debug_stats,
                                true,
                                frame_time_ms,
                                render_fps,
                                render_fps,
                            );
                        }

                        // Network statistics overlay (F4)
                        if network_overlay_visible && session_type != SessionType::Local {
                            egui::Window::new("Network")
                                .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                                .collapsible(false)
                                .resizable(false)
                                .show(ctx, |ui| {
                                    ui.set_min_width(180.0);

                                    // Player stats with quality bar
                                    for (i, stats) in network_stats.iter().enumerate() {
                                        let is_local = local_players.contains(&i);

                                        // Quality color and label
                                        let (color, quality_label) = match stats.quality {
                                            ConnectionQuality::Excellent => {
                                                (egui::Color32::GREEN, "Excellent")
                                            }
                                            ConnectionQuality::Good => {
                                                (egui::Color32::from_rgb(144, 238, 144), "Good")
                                            }
                                            ConnectionQuality::Fair => {
                                                (egui::Color32::YELLOW, "Fair")
                                            }
                                            ConnectionQuality::Poor => (egui::Color32::RED, "Poor"),
                                            ConnectionQuality::Disconnected => {
                                                (egui::Color32::DARK_GRAY, "Disconnected")
                                            }
                                        };

                                        if is_local {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("P{}: Local", i + 1));
                                            });
                                        } else if stats.connected {
                                            // Show: P2: 45ms ████████ Good
                                            ui.horizontal(|ui| {
                                                ui.label(format!("P{}: {}ms ", i + 1, stats.ping_ms));

                                                // Quality bar (8 blocks max)
                                                let filled = match stats.quality {
                                                    ConnectionQuality::Excellent => 8,
                                                    ConnectionQuality::Good => 6,
                                                    ConnectionQuality::Fair => 4,
                                                    ConnectionQuality::Poor => 2,
                                                    ConnectionQuality::Disconnected => 0,
                                                };
                                                let bar: String = "\u{2588}"
                                                    .repeat(filled)
                                                    .chars()
                                                    .chain("\u{2591}".repeat(8 - filled).chars())
                                                    .collect();
                                                ui.colored_label(color, bar);
                                                ui.label(quality_label);
                                            });
                                        } else {
                                            ui.horizontal(|ui| {
                                                ui.colored_label(
                                                    egui::Color32::DARK_GRAY,
                                                    format!("P{}: Disconnected", i + 1),
                                                );
                                            });
                                        }
                                    }

                                    ui.separator();
                                    ui.label(format!("Rollbacks: {} frames", total_rollbacks));
                                    ui.label(format!("Frame: {}", current_frame));
                                });
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
                                    let bytes = unsafe {
                                        std::slice::from_raw_parts((mem_base + ptr) as *const u8, size)
                                    };
                                    Some(registry_for_read.read_value_from_slice(bytes, reg_val.value_type))
                                };

                                let write_value = |reg_val: &RegisteredValue, new_val: &DebugValue| -> bool {
                                    pending_writes.borrow_mut().push((reg_val.clone(), new_val.clone()));
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

                        if let Some(error) = error_state_ref {
                            let action = render_error_screen(ctx, error);
                            if action != ErrorAction::None {
                                *error_action.borrow_mut() = action;
                            }
                        }
                    });

                    egui_state.handle_platform_output(window, full_output.platform_output);

                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [runner.graphics().width(), runner.graphics().height()],
                        pixels_per_point: window.scale_factor() as f32,
                    };

                    let tris = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

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

                    {
                        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Egui Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
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

                    for id in &full_output.textures_delta.free {
                        egui_renderer.free_texture(id);
                    }
                }

                // Apply pending writes
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
                                        registry.write_value_to_slice(&mut data[ptr..ptr + size], new_val);
                                    }
                                }
                            }
                            if has_debug_callback {
                                game.call_on_debug_change();
                            }
                        }
                    }
                }

                // Apply pending action
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
                        let mut config = super::config::load();
                        config.video.scale_mode = self.settings_panel.scale_mode;
                        config.video.fullscreen = self.settings_panel.fullscreen;
                        config.audio.master_volume = self.settings_panel.master_volume;
                        if let Err(e) = super::config::save(&config) {
                            tracing::error!("Failed to save config: {}", e);
                        } else {
                            tracing::info!("Settings saved to config");
                        }
                    }
                }

                // Apply error actions
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

            runner.graphics().queue().submit(std::iter::once(encoder.finish()));
            surface_texture.present();

            // Process screen capture
            if self.capture.needs_capture() {
                let (width, height) = runner.graphics().render_target_dimensions();
                let pixels = read_render_target_pixels(
                    runner.graphics().device(),
                    runner.graphics().queue(),
                    runner.graphics().render_target_texture(),
                    width,
                    height,
                );
                self.capture.process_frame(pixels, width, height);
            }

            // Check for capture results
            if let Some(result) = self.capture.poll_save_result() {
                match result {
                    crate::capture::SaveResult::Screenshot(Ok(path)) => {
                        tracing::info!("Screenshot saved: {}", path.display());
                    }
                    crate::capture::SaveResult::Screenshot(Err(e)) => {
                        tracing::error!("Failed to save screenshot: {}", e);
                    }
                    crate::capture::SaveResult::Gif(Ok(path)) => {
                        tracing::info!("GIF saved: {}", path.display());
                    }
                    crate::capture::SaveResult::Gif(Err(e)) => {
                        tracing::error!("Failed to save GIF: {}", e);
                    }
                }
            }
        }

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
        let tick = self
            .runner
            .as_ref()
            .and_then(|r| r.session())
            .and_then(|s| s.runtime.game())
            .map(|g| g.state().tick_count);

        let game_error = parse_wasm_error(
            &anyhow::anyhow!("{}", error.0),
            tick,
            GameErrorPhase::Update,
        );

        tracing::error!("Runtime error: {}", game_error);
        self.error_state = Some(game_error);
        self.needs_redraw = true;
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

/// Run a standalone player for the given console.
pub fn run_standalone<C, L>(config: StandaloneConfig) -> Result<()>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    let vram_limit = C::specs().vram_limit;
    let app = StandaloneApp::<C, L>::new(config, vram_limit);
    super::run(app)
}
