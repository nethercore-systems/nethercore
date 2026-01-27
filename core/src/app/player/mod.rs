//! Generic standalone player for any console
//!
//! Provides a console-agnostic player application that can run ROM files
//! for any console that implements the Console trait and required support traits.

mod connection;
mod error_ui;
mod init;
mod input;
mod lifecycle;
mod rendering;
#[cfg(test)]
mod tests;
mod trait_impl;
mod types;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::capture::ScreenCapture;
use crate::console::Console;
use crate::debug::FrameController;
use crate::replay::ScriptExecutor;
use crate::runner::ConsoleRunner;

use super::ui::SharedSettingsUi;
use super::{DebugStats, FRAME_TIME_HISTORY_SIZE, GameError};

// Re-export types from submodules
pub use error_ui::{
    ErrorAction, WaitingForPeer, parse_key_code, render_error_screen, sanitize_game_id,
};
pub use types::{LoadedRom, RomLoader, StandaloneConfig, StandaloneGraphicsSupport};

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
    config: types::StandaloneConfig,
    window: Option<Arc<Window>>,
    runner: Option<ConsoleRunner<C>>,
    input_manager: super::InputManager,
    scale_mode: super::config::ScaleMode,
    settings_ui: SharedSettingsUi,
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
    /// State for waiting for a peer to connect (Host mode)
    waiting_for_peer: Option<WaitingForPeer>,
    _vram_limit: usize,
    _loader_marker: std::marker::PhantomData<L>,
    /// Active replay script executor (when --replay is used)
    replay_executor: Option<ScriptExecutor>,
}

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Create a new standalone app with the given configuration.
    pub fn new(config: types::StandaloneConfig, vram_limit: usize) -> Self {
        let now = Instant::now();
        let app_config = super::config::load();
        let input_config = app_config.input.clone();
        let scale_mode = app_config.video.scale_mode;
        let settings_ui = SharedSettingsUi::new(&app_config);

        let warnings = super::config::validate_keybindings(&app_config);
        for warning in warnings {
            tracing::warn!("Keybinding conflict: {}", warning);
        }

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
            C::specs().console_type.to_string(),
        );

        Self {
            debug_overlay: config.debug,
            debug_panel: crate::debug::DebugPanel::new(),
            config,
            window: None,
            runner: None,
            input_manager: super::InputManager::new(input_config),
            scale_mode,
            settings_ui,
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
            waiting_for_peer: None,
            _vram_limit: vram_limit,
            _loader_marker: std::marker::PhantomData,
            replay_executor: None,
        }
    }
}

/// Run a standalone player for the given console.
pub fn run_standalone<C, L>(config: types::StandaloneConfig) -> Result<()>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    let vram_limit = C::specs().vram_limit;
    let app = StandaloneApp::<C, L>::new(config, vram_limit);
    super::run(app)
}
