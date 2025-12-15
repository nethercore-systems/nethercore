//! Application initialization

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window};

use crate::registry::ActiveGame;
use crate::ui::LibraryUi;
use emberware_core::app::config;
use emberware_core::app::{AppMode, DebugStats, FRAME_TIME_HISTORY_SIZE};
use emberware_core::debug::{DebugPanel, FrameController};
use emberware_z::console::VRAM_LIMIT;
use emberware_z::input::InputManager;
use emberware_z::library;

use super::App;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Event loop error: {0}")]
    EventLoop(String),
}

impl App {
    /// Create a new application instance
    pub fn new(initial_mode: AppMode) -> Self {
        let config = config::load();

        // Initialize input manager
        let input_manager = Some(InputManager::new(config.input.clone()));

        // Load local games
        let local_games = library::get_local_games(&library::ZDataDirProvider);

        let now = Instant::now();

        Self {
            mode: initial_mode.clone(),
            settings_ui: crate::ui::SettingsUi::new(&config),
            config,
            window: None,
            active_game: None,
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
            needs_redraw: true,
            cached_egui_shapes: Vec::new(),
            cached_egui_tris: Vec::new(),
            cached_pixels_per_point: 1.0,
            last_mode: initial_mode.clone(),
            last_window_size: (960, 540),
            debug_panel: DebugPanel::new(),
            frame_controller: FrameController::new(),
            next_tick: now,
            last_sim_rendered: false,
        }
    }

    /// Handle window creation event
    pub(super) fn on_window_created(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()> {
        if self.window.is_some() {
            return Ok(()); // Already initialized
        }

        // Apply fullscreen from config
        if self.config.video.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        // Initialize ActiveGame (creates graphics backend without loading a game)
        let active_game = ActiveGame::new_z(window.clone())?;

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
            active_game.device(),
            active_game.surface_format(),
            egui_wgpu::RendererOptions {
                depth_stencil_format: None,
                msaa_samples: 1,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );

        tracing::info!("Graphics and egui initialized successfully");
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        self.active_game = Some(active_game);
        self.window = Some(window);

        Ok(())
    }
}
