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
use crate::console::VRAM_LIMIT;
use crate::graphics::ZGraphics;
use crate::input::InputManager;
use crate::library::{self, LocalGame};
use crate::ui::{LibraryUi, UiAction};
use emberware_core::console::Graphics;

#[derive(Debug, Clone)]
pub enum AppMode {
    Library,
    Downloading { game_id: String, progress: f32 },
    Playing { game_id: String },
    Settings,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Window creation failed: {0}")]
    Window(String),
    #[error("Graphics initialization failed: {0}")]
    Graphics(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Event loop error: {0}")]
    EventLoop(String),
}

/// Runtime error types for state machine transitions
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// WASM game panicked
    WasmPanic(String),
    /// Network disconnected
    NetworkDisconnect,
    /// Out of memory (RAM or VRAM)
    OutOfMemory { resource: String, used: usize, limit: usize },
    /// Generic runtime error
    Other(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WasmPanic(msg) => write!(f, "Game crashed: {}", msg),
            Self::NetworkDisconnect => write!(f, "Network disconnected"),
            Self::OutOfMemory { resource, used, limit } => {
                write!(f, "Out of {}: {} / {} bytes", resource, used, limit)
            }
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Frame time sample for graph
const FRAME_TIME_HISTORY_SIZE: usize = 120;
/// Target frame time for reference line (60 FPS = 16.67ms)
const TARGET_FRAME_TIME_MS: f32 = 16.67;

/// Debug statistics for overlay
#[derive(Debug, Default)]
pub struct DebugStats {
    /// Frame times ring buffer (milliseconds)
    pub frame_times: VecDeque<f32>,
    /// VRAM usage in bytes
    pub vram_used: usize,
    /// VRAM limit in bytes
    pub vram_limit: usize,
    /// Network stats (when in P2P session)
    pub ping_ms: Option<u32>,
    /// Rollback frames this session
    pub rollback_frames: u64,
    /// Frame advantage (how far ahead of opponent)
    pub frame_advantage: i32,
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
    /// Cached local games list
    local_games: Vec<LocalGame>,
    /// Debug overlay enabled (F3)
    debug_overlay: bool,
    /// Frame times for FPS calculation
    frame_times: Vec<Instant>,
    /// Last frame time
    last_frame: Instant,
    /// Debug statistics
    debug_stats: DebugStats,
    /// Last runtime error (for displaying error in library)
    last_error: Option<RuntimeError>,
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

        Self {
            mode: initial_mode,
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
            debug_stats: DebugStats {
                frame_times: VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE),
                vram_limit: VRAM_LIMIT,
                ..Default::default()
            },
            last_error: None,
        }
    }

    /// Handle a runtime error by transitioning back to library
    fn handle_runtime_error(&mut self, error: RuntimeError) {
        tracing::error!("Runtime error: {}", error);
        self.last_error = Some(error);
        self.mode = AppMode::Library;
        self.local_games = library::get_local_games();
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
                        // Return to library when in game
                        match self.mode {
                            AppMode::Playing { .. } => {
                                self.mode = AppMode::Library;
                                self.local_games = library::get_local_games();
                            }
                            AppMode::Settings => {
                                self.mode = AppMode::Library;
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
                self.mode = AppMode::Playing { game_id };
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
                tracing::info!("Opening settings...");
                self.mode = AppMode::Settings;
            }
            UiAction::DismissError => {
                self.last_error = None;
            }
        }
    }

    /// Calculate FPS from frame times
    fn calculate_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }
        let elapsed = self.frame_times.last().unwrap()
            .duration_since(*self.frame_times.first().unwrap())
            .as_secs_f32();
        if elapsed > 0.0 {
            self.frame_times.len() as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Render the current frame
    fn render(&mut self) {
        let now = Instant::now();

        // Update frame timing
        self.frame_times.push(now);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        let frame_time_ms = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;
        self.last_frame = now;

        // Update debug stats
        self.debug_stats.frame_times.push_back(frame_time_ms);
        while self.debug_stats.frame_times.len() > FRAME_TIME_HISTORY_SIZE {
            self.debug_stats.frame_times.pop_front();
        }

        // Pre-collect values to avoid borrow conflicts
        let mode = self.mode.clone();
        let debug_overlay = self.debug_overlay;
        let fps = self.calculate_fps();
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

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Start egui frame
        let raw_input = egui_state.take_egui_input(&window);

        // Collect UI action separately to avoid borrow conflicts
        let mut ui_action = None;

        // Collect debug stats for overlay
        let debug_stats = DebugStats {
            frame_times: self.debug_stats.frame_times.clone(),
            vram_used: self.debug_stats.vram_used,
            vram_limit: self.debug_stats.vram_limit,
            ping_ms: self.debug_stats.ping_ms,
            rollback_frames: self.debug_stats.rollback_frames,
            frame_advantage: self.debug_stats.frame_advantage,
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
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Settings");
                        ui.separator();
                        ui.label("Settings UI not yet implemented");
                        ui.add_space(20.0);
                        if ui.button("Back to Library").clicked() {
                            ui_action = Some(UiAction::OpenSettings); // Signal to go back
                        }
                    });
                }
                AppMode::Playing { ref game_id } => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading(format!("Playing: {}", game_id));
                        ui.label("Game rendering not yet implemented");
                        ui.label("Press ESC to return to library");
                    });
                }
                AppMode::Downloading { ref game_id, progress } => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading(format!("Downloading: {}", game_id));
                        ui.add(egui::ProgressBar::new(*progress).show_percentage());
                    });
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
                        ui.label(format!("FPS: {:.1}", fps));
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

                            // Target line (16.67ms for 60 FPS)
                            let target_y = rect.bottom() - (TARGET_FRAME_TIME_MS / 33.33 * graph_height);
                            painter.hline(
                                rect.left()..=rect.right(),
                                target_y,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                            );

                            // Frame time bars
                            if !debug_stats.frame_times.is_empty() {
                                let bar_width = rect.width() / FRAME_TIME_HISTORY_SIZE as f32;
                                for (i, &time_ms) in debug_stats.frame_times.iter().enumerate() {
                                    let x = rect.left() + i as f32 * bar_width;
                                    // Scale: 0-33.33ms maps to full height
                                    let height = (time_ms / 33.33 * graph_height).min(graph_height);
                                    let bar_rect = egui::Rect::from_min_max(
                                        egui::pos2(x, rect.bottom() - height),
                                        egui::pos2(x + bar_width - 1.0, rect.bottom()),
                                    );

                                    // Color based on frame time
                                    let color = if time_ms <= TARGET_FRAME_TIME_MS {
                                        egui::Color32::from_rgb(100, 200, 100) // Green
                                    } else if time_ms <= 33.33 {
                                        egui::Color32::from_rgb(200, 200, 100) // Yellow
                                    } else {
                                        egui::Color32::from_rgb(200, 100, 100) // Red
                                    };

                                    painter.rect_filled(bar_rect, 0.0, color);
                                }
                            }

                            // Label
                            painter.text(
                                egui::pos2(rect.left() + 4.0, rect.top() + 2.0),
                                egui::Align2::LEFT_TOP,
                                "Frame time (0-33ms)",
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
                        ui.label(format!("VRAM: {:.2} / {:.2} MB ({:.1}%)", vram_mb, vram_limit_mb, vram_pct * 100.0));
                        ui.add(egui::ProgressBar::new(vram_pct).show_percentage());

                        ui.separator();

                        // Network section
                        ui.heading("Network");
                        if let Some(ping) = debug_stats.ping_ms {
                            ui.label(format!("Ping: {}ms", ping));
                            ui.label(format!("Rollback frames: {}", debug_stats.rollback_frames));
                            ui.label(format!("Frame advantage: {}", debug_stats.frame_advantage));
                        } else {
                            ui.label("No network session");
                        }
                    });
            }
        });

        egui_state.handle_platform_output(&window, full_output.platform_output);

        let tris = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        // Upload egui textures
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(graphics.device(), graphics.queue(), *id, image_delta);
        }

        // Create command encoder
        let mut encoder = graphics.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [graphics.width(), graphics.height()],
            pixels_per_point: window.scale_factor() as f32,
        };

        // Create render pass and render egui
        // Note: egui-wgpu 0.30 expects RenderPass<'static> which is a known API issue.
        // We use a scoped block and unsafe transmute to work around this.
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
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

            // SAFETY: The render_pass lives for this entire block. egui-wgpu 0.30 has
            // an API bug requiring 'static lifetime, but the pass is only used within
            // this scope. This was fixed in later versions.
            let render_pass_static: &mut wgpu::RenderPass<'static> = unsafe {
                std::mem::transmute(&mut render_pass)
            };

            egui_renderer.render(render_pass_static, &tris, &screen_descriptor);
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
        window.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title("Emberware Z")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080));

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
        let graphics = match pollster::block_on(ZGraphics::new(window.clone())) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to initialize graphics: {}", e);
                self.should_exit = true;
                return;
            }
        };

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
            WindowEvent::KeyboardInput { event: key_event, .. } => {
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

        // Request redraw for continuous rendering
        if let Some(window) = &self.window {
            window.request_redraw();
        }
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
