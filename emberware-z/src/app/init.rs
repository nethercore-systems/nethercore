//! Application initialization

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window};

use crate::console::VRAM_LIMIT;
use crate::graphics::ZGraphics;
use crate::input::InputManager;
use crate::library;
use crate::ui::LibraryUi;
use emberware_core::app::config;
use emberware_core::app::{AppMode, DebugStats, FRAME_TIME_HISTORY_SIZE};
use emberware_core::wasm::WasmEngine;

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

        // Initialize graphics backend
        let mut graphics = pollster::block_on(ZGraphics::new(window.clone()))?;

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
        if let (Some(session), Some(graphics)) = (&mut self.game_session, &self.graphics) {
            let font_texture_handle = graphics.font_texture();
            session
                .resource_manager
                .texture_map
                .insert(0, font_texture_handle);
            tracing::info!(
                "Added font texture to existing game session: handle 0 -> {:?}",
                font_texture_handle
            );

            let white_texture_handle = graphics.white_texture();
            session
                .resource_manager
                .texture_map
                .insert(u32::MAX, white_texture_handle);
            tracing::info!(
                "Added white texture to existing game session: handle 0xFFFFFFFF -> {:?}",
                white_texture_handle
            );
        }

        Ok(())
    }
}
