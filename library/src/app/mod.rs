//! Library application state and main loop
//!
//! The library is a simple launcher UI that:
//! - Shows installed games
//! - Launches games as separate player processes
//! - Does NOT run games in-process

mod init;

pub use init::AppError;

use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::graphics::LibraryGraphics;
use crate::ui::{LibraryUi, UiAction};
use emberware_core::app::config::Config;
use emberware_core::library::{LocalGame, RomLoaderRegistry};

/// Library application state
pub struct App {
    /// User configuration
    config: Config,
    /// Window handle
    window: Option<Arc<Window>>,
    /// Graphics context for egui rendering
    graphics: Option<LibraryGraphics>,
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
    settings_ui: crate::ui::SettingsUi,
    /// Cached local games list
    local_games: Vec<LocalGame>,
    /// ROM loader registry
    rom_loader_registry: RomLoaderRegistry,
    /// Last error message (for displaying in UI)
    last_error: Option<String>,
    /// Whether a redraw is needed
    needs_redraw: bool,
    /// Egui cache
    cached_egui_shapes: Vec<egui::epaint::ClippedShape>,
    cached_egui_tris: Vec<egui::ClippedPrimitive>,
    cached_pixels_per_point: f32,
    last_window_size: (u32, u32),
    /// Last frame time for throttling
    last_frame: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new library application
    pub fn new() -> Self {
        let config = emberware_core::app::config::load();
        let rom_loader_registry = crate::registry::create_rom_loader_registry();
        let local_games = emberware_core::library::get_local_games_with_loaders(
            &emberware_core::library::DefaultDataDirProvider,
            &rom_loader_registry,
        );

        Self {
            settings_ui: crate::ui::SettingsUi::new(&config),
            config,
            window: None,
            graphics: None,
            should_exit: false,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            library_ui: LibraryUi::new(),
            local_games,
            rom_loader_registry,
            last_error: None,
            needs_redraw: true,
            cached_egui_shapes: Vec::new(),
            cached_egui_tris: Vec::new(),
            cached_pixels_per_point: 1.0,
            last_window_size: (960, 540),
            last_frame: Instant::now(),
        }
    }

    /// Refresh the local games list
    fn refresh_games(&mut self) {
        self.local_games = emberware_core::library::get_local_games_with_loaders(
            &emberware_core::library::DefaultDataDirProvider,
            &self.rom_loader_registry,
        );
    }

    /// Handle UI actions
    fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::PlayGame(game_id) => {
                tracing::info!("Launching game: {}", game_id);

                // Find the game and launch it
                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    match crate::registry::launch_game_by_id(game) {
                        Ok(()) => {
                            tracing::info!("Player process spawned for: {}", game_id);
                            self.last_error = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::DeleteGame(game_id) => {
                tracing::info!("Deleting game: {}", game_id);
                if let Err(e) = emberware_core::library::delete_game(
                    &emberware_core::library::DefaultDataDirProvider,
                    &game_id,
                ) {
                    tracing::error!("Failed to delete game: {}", e);
                }
                self.refresh_games();
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
                // Toggle settings panel in library UI
                self.library_ui.show_settings = !self.library_ui.show_settings;
                if self.library_ui.show_settings {
                    self.settings_ui.update_temp_config(&self.config);
                }
            }
            UiAction::DismissError => {
                self.last_error = None;
            }
            UiAction::RefreshLibrary => {
                tracing::info!("Refreshing game library");
                self.refresh_games();
                self.library_ui.selected_game = None;
            }
            UiAction::OpenGame => {
                tracing::info!("Opening file picker to run game directly");

                let file_handle = rfd::FileDialog::new()
                    .add_filter("Game Files", &["ewz", "wasm"])
                    .add_filter("Emberware ROM", &["ewz"])
                    .add_filter("WebAssembly", &["wasm"])
                    .set_title("Open Game File")
                    .pick_file();

                if let Some(path) = file_handle {
                    tracing::info!("Launching game from: {}", path.display());
                    match crate::registry::launch_game_from_path(&path) {
                        Ok(()) => {
                            tracing::info!("Player process spawned for: {}", path.display());
                            self.last_error = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                }
            }
            UiAction::ImportRom => {
                tracing::info!("Opening file picker for ROM import");

                let file_handle = rfd::FileDialog::new()
                    .add_filter("Emberware ROM", &["ewz"])
                    .set_title("Import ROM File")
                    .pick_file();

                if let Some(source_path) = file_handle {
                    tracing::info!("Selected ROM file: {}", source_path.display());

                    if let Some(data_dir) = emberware_core::app::config::data_dir() {
                        let games_dir = data_dir.join("games");

                        if let Err(e) = std::fs::create_dir_all(&games_dir) {
                            tracing::error!("Failed to create games directory: {}", e);
                            self.last_error =
                                Some(format!("Failed to create games directory: {}", e));
                            return;
                        }

                        if let Some(filename) = source_path.file_name() {
                            let dest_path = games_dir.join(filename);

                            match std::fs::copy(&source_path, &dest_path) {
                                Ok(_) => {
                                    tracing::info!(
                                        "ROM imported successfully to: {}",
                                        dest_path.display()
                                    );
                                    self.refresh_games();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to copy ROM file: {}", e);
                                    self.last_error = Some(format!("Failed to import ROM: {}", e));
                                }
                            }
                        } else {
                            self.last_error = Some("Invalid file path".to_string());
                        }
                    } else {
                        self.last_error = Some("Could not determine data directory".to_string());
                    }
                }
            }
            UiAction::SaveSettings(new_config) => {
                tracing::info!("Saving settings...");
                self.config = new_config.clone();

                if let Err(e) = emberware_core::app::config::save(&self.config) {
                    tracing::error!("Failed to save config: {}", e);
                } else {
                    tracing::info!("Settings saved successfully");
                }

                // Apply fullscreen setting
                if let Some(window) = &self.window {
                    let is_fullscreen = window.fullscreen().is_some();
                    if is_fullscreen != self.config.video.fullscreen {
                        let new_fullscreen = if self.config.video.fullscreen {
                            Some(winit::window::Fullscreen::Borderless(None))
                        } else {
                            None
                        };
                        window.set_fullscreen(new_fullscreen);
                    }
                }

                self.settings_ui.update_temp_config(&self.config);
                self.library_ui.show_settings = false;
            }
            UiAction::SetScaleMode(_scale_mode) => {
                // Scale mode only affects game rendering, which happens in player process
                // This is a no-op in the library
            }
        }
    }

    /// Render the library UI
    fn render(&mut self) {
        let window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        let graphics = match &self.graphics {
            Some(g) => g,
            None => return,
        };

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

        // Create encoder
        let mut encoder =
            graphics
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Library Frame Encoder"),
                });

        // Run egui
        let raw_input = egui_state.take_egui_input(&window);
        let mut ui_action = None;

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Show error panel if there's an error
            if let Some(ref error) = self.last_error {
                egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                        if ui.button("Dismiss").clicked() {
                            ui_action = Some(UiAction::DismissError);
                        }
                    });
                });
            }

            // Show settings or library
            if self.library_ui.show_settings {
                if let Some(action) = self.settings_ui.show(ctx) {
                    ui_action = Some(action);
                }
            } else if let Some(action) = self.library_ui.show(ctx, &self.local_games) {
                ui_action = Some(action);
            }
        });

        egui_state.handle_platform_output(&window, full_output.platform_output);

        // Determine if egui needs update
        let mut egui_dirty = self.cached_egui_shapes.is_empty();

        let current_size = (graphics.width(), graphics.height());
        if !egui_dirty && self.last_window_size != current_size {
            egui_dirty = true;
            self.last_window_size = current_size;
        }

        if !egui_dirty
            && (self.cached_pixels_per_point - full_output.pixels_per_point).abs() > 0.001
        {
            egui_dirty = true;
            self.cached_pixels_per_point = full_output.pixels_per_point;
        }

        if !egui_dirty && !full_output.textures_delta.set.is_empty() {
            egui_dirty = true;
        }

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

        // Tessellate and render
        // Note: tessellate() takes ownership of shapes, so we must clone for caching.
        // However, we avoid cloning the tessellation result by using references.
        if egui_dirty {
            // Cache shapes for comparison on next frame, tessellate with clone
            // (tessellate consumes the Vec, so clone is unavoidable here)
            self.cached_egui_tris = self
                .egui_ctx
                .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);
            self.cached_egui_shapes = full_output.shapes;

            egui_renderer.update_buffers(
                graphics.device(),
                graphics.queue(),
                &mut encoder,
                &self.cached_egui_tris,
                &screen_descriptor,
            );
        }
        // Use reference to cached tris (avoids clone when not dirty)
        let tris = &self.cached_egui_tris;

        // Update textures
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(graphics.device(), graphics.queue(), *id, image_delta);
        }

        // Render egui
        if !tris.is_empty() {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass_static = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass_static, tris, &screen_descriptor);
        } else {
            // Clear screen even with no egui content
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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Submit
        graphics.queue().submit(std::iter::once(encoder.finish()));

        // Free textures
        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Present
        surface_texture.present();

        // Handle UI action
        if let Some(action) = ui_action {
            self.handle_ui_action(action);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_attrs = WindowAttributes::default()
            .with_title("Emberware Library")
            .with_inner_size(winit::dpi::LogicalSize::new(960, 540));

        let window = match event_loop.create_window(window_attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Apply fullscreen from config
        if self.config.video.fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        // Initialize graphics
        let graphics = match LibraryGraphics::new(window.clone()) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to initialize graphics: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize egui
        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            graphics.device(),
            graphics.surface_format(),
            egui_wgpu::RendererOptions {
                depth_stencil_format: None,
                msaa_samples: 1,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );

        tracing::info!("Library window and graphics initialized");

        self.window = Some(window);
        self.graphics = Some(graphics);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let egui handle events
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, &event);
            if response.repaint {
                self.needs_redraw = true;
            }
            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.should_exit = true;
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    if let Some(graphics) = &mut self.graphics {
                        graphics.resize(new_size.width, new_size.height);
                    }
                    self.needs_redraw = true;
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                self.needs_redraw = false;
                self.last_frame = Instant::now();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Handle F11 for fullscreen toggle
                if event.state == winit::event::ElementState::Pressed
                    && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11) =
                        event.physical_key
                    && let Some(window) = &self.window
                {
                    let is_fullscreen = window.fullscreen().is_some();
                    let new_fullscreen = if is_fullscreen {
                        None
                    } else {
                        Some(winit::window::Fullscreen::Borderless(None))
                    };
                    window.set_fullscreen(new_fullscreen);
                    self.config.video.fullscreen = !is_fullscreen;
                    let _ = emberware_core::app::config::save(&self.config);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_exit {
            event_loop.exit();
            return;
        }

        // Request redraw if needed
        if self.needs_redraw
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }

        // Use reactive mode - only redraw when something changes
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

/// Run the library application
pub fn run() -> Result<(), AppError> {
    tracing::info!("Starting Emberware Library");

    let event_loop = EventLoop::new()
        .map_err(|e| AppError::EventLoop(format!("Failed to create event loop: {}", e)))?;

    let mut app = App::new();

    event_loop
        .run_app(&mut app)
        .map_err(|e| AppError::EventLoop(format!("Event loop error: {}", e)))?;

    Ok(())
}
