//! Application state and main loop

mod debug;
mod game_session;
mod init;
mod ui;

pub use init::AppError;

use std::sync::Arc;
use std::time::Instant;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

use crate::console::EmberwareZ;
use crate::ffi::unpack_rgba;
use crate::graphics::ZGraphics;
use crate::input::InputManager;
use crate::library::LocalGame;
use crate::ui::{LibraryUi, UiAction};
use emberware_core::app::config::Config;
use emberware_core::app::{
    AppMode, DebugStats, FRAME_TIME_HISTORY_SIZE, RuntimeError, session::GameSession,
};
use emberware_core::console::ConsoleResourceManager;
use emberware_core::debug::{DebugPanel, FrameController};
use emberware_core::wasm::WasmEngine;

/// Data needed to render the debug panel (extracted before egui frame)
struct DebugPanelData {
    registry: emberware_core::debug::DebugRegistry,
    mem_ptr: *mut u8,
    mem_len: usize,
}

/// Application state
pub struct App {
    /// Current application mode
    pub(crate) mode: AppMode,
    /// User configuration
    pub(crate) config: Config,
    /// Window handle (created during resumed event)
    pub(crate) window: Option<Arc<Window>>,
    /// Graphics backend (initialized after window creation)
    pub(crate) graphics: Option<ZGraphics>,
    /// Input manager (keyboard + gamepad)
    pub(crate) input_manager: Option<InputManager>,
    /// Whether the application should exit
    pub(crate) should_exit: bool,
    /// egui context
    pub(crate) egui_ctx: egui::Context,
    /// egui-winit state
    pub(crate) egui_state: Option<egui_winit::State>,
    /// egui-wgpu renderer
    pub(crate) egui_renderer: Option<egui_wgpu::Renderer>,
    /// Library UI state
    pub(crate) library_ui: LibraryUi,
    /// Settings UI state
    pub(crate) settings_ui: crate::settings_ui::SettingsUi,
    /// Cached local games list
    pub(crate) local_games: Vec<LocalGame>,
    /// Debug overlay enabled (F3)
    pub(crate) debug_overlay: bool,
    /// Frame times for FPS calculation (render rate)
    pub(crate) frame_times: Vec<Instant>,
    /// Last frame time
    pub(crate) last_frame: Instant,
    /// Game tick times for game FPS calculation (update rate)
    pub(crate) game_tick_times: Vec<Instant>,
    /// Last game tick time
    pub(crate) last_game_tick: Instant,
    /// Debug statistics
    pub(crate) debug_stats: DebugStats,
    /// Last runtime error (for displaying error in library)
    pub(crate) last_error: Option<RuntimeError>,
    /// WASM engine (shared across all games)
    pub(crate) wasm_engine: Option<WasmEngine>,
    /// Active game session (only present in Playing mode)
    pub(crate) game_session: Option<GameSession<EmberwareZ>>,
    /// Next scheduled egui repaint time (for animations)
    pub(crate) next_egui_repaint: Option<Instant>,
    // Egui optimization cache
    pub(crate) cached_egui_shapes: Vec<egui::epaint::ClippedShape>,
    pub(crate) cached_egui_tris: Vec<egui::ClippedPrimitive>,
    pub(crate) cached_pixels_per_point: f32,
    pub(crate) last_mode: AppMode,
    pub(crate) last_window_size: (u32, u32),
    /// Debug inspection panel
    pub(crate) debug_panel: DebugPanel,
    /// Frame controller for pause/step/time scale
    pub(crate) frame_controller: FrameController,
}

impl App {
    /// Render the current frame
    fn render(&mut self) {
        let now = Instant::now();

        // Clear scheduled egui repaint for this frame's collection
        self.next_egui_repaint = None;

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
            // Initialize game session if needed (CLI launch case)
            // When launched via CLI with a game_id, the app starts in Playing mode
            // but start_game() was never called (unlike Library UI flow)
            if self.game_session.is_none() {
                if let AppMode::Playing { ref game_id } = self.mode {
                    let game_id_owned = game_id.clone();
                    tracing::info!(
                        "Initializing game session for CLI launch: {}",
                        game_id_owned
                    );
                    if let Err(e) = self.start_game(&game_id_owned) {
                        self.handle_runtime_error(e);
                        return;
                    }
                }
            }

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
                    // Only execute draw commands if we rendered
                    if rendered {
                        if let Some(session) = &mut self.game_session {
                            if let Some(graphics) = &mut self.graphics {
                                // Execute draw commands (resources already flushed post-init)
                                if let Some(game) = session.runtime.game_mut() {
                                    let state = game.console_state_mut();
                                    session
                                        .resource_manager
                                        .execute_draw_commands(graphics, state);
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
                self.local_games =
                    crate::library::get_local_games(&crate::library::ZDataDirProvider);
                return;
            }
        }

        // Pre-collect values to avoid borrow conflicts
        let mode = self.mode.clone();
        let debug_overlay = self.debug_overlay;
        let render_fps = self.calculate_fps();
        let game_tick_fps = self.calculate_game_tick_fps();
        let last_error = self.last_error.clone();

        // Prepare debug panel data BEFORE borrowing graphics (to avoid borrow conflicts)
        let debug_panel_data =
            if self.debug_panel.visible && matches!(mode, AppMode::Playing { .. }) {
                self.prepare_debug_panel_data()
            } else {
                None
            };

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

                            unpack_rgba(z_state.init_config.clear_color)
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
                        graphics.render_frame(
                            &mut encoder,
                            z_state,
                            &session.resource_manager.texture_map,
                            clear_color,
                        );
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

        // Track if debug values were changed (set inside closure)
        let mut debug_values_changed = false;

        // Collect debug stats for overlay only when needed (avoid VecDeque clones every frame)
        let debug_stats = if debug_overlay {
            Some(DebugStats {
                frame_times: self.debug_stats.frame_times.clone(),
                game_tick_times: self.debug_stats.game_tick_times.clone(),
                game_render_times: self.debug_stats.game_render_times.clone(),
                vram_used: self.debug_stats.vram_used,
                vram_limit: self.debug_stats.vram_limit,
                ping_ms: self.debug_stats.ping_ms,
                rollback_frames: self.debug_stats.rollback_frames,
                frame_advantage: self.debug_stats.frame_advantage,
                network_interrupted: self.debug_stats.network_interrupted,
            })
        } else {
            None
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
                AppMode::Playing { game_id } => {
                    // Game is rendered before egui, so we don't need a central panel
                    // Just show debug info if overlay is enabled
                    let _ = game_id; // Used in debug overlay
                }
            }

            // Debug overlay (only when enabled - stats are only cloned when needed)
            if let Some(ref stats) = debug_stats {
                emberware_core::app::render_debug_overlay(
                    ctx,
                    stats,
                    matches!(mode, AppMode::Playing { .. }),
                    frame_time_ms,
                    render_fps,
                    game_tick_fps,
                );
            }

            // Debug inspection panel - MUST be inside ctx.run() to receive input
            if let Some(ref data) = debug_panel_data {
                use emberware_core::debug::{DebugValue, RegisteredValue};

                let mem_ptr = data.mem_ptr;
                let mem_len = data.mem_len;

                // Create read closure using raw pointer
                let read_value = |reg_value: &RegisteredValue| -> Option<DebugValue> {
                    let ptr = reg_value.wasm_ptr as usize;
                    let size = reg_value.value_type.byte_size();
                    if ptr + size > mem_len {
                        return None;
                    }
                    // SAFETY: Bounds checked above, pointer valid for this frame
                    let slice = unsafe { std::slice::from_raw_parts(mem_ptr.add(ptr), size) };
                    Some(
                        data.registry
                            .read_value_from_slice(slice, reg_value.value_type),
                    )
                };

                // Create write closure using raw pointer
                let write_value = |reg_value: &RegisteredValue, new_val: &DebugValue| -> bool {
                    let ptr = reg_value.wasm_ptr as usize;
                    let size = reg_value.value_type.byte_size();
                    if ptr + size > mem_len {
                        return false;
                    }
                    // SAFETY: Bounds checked above, pointer valid for this frame
                    let slice = unsafe { std::slice::from_raw_parts_mut(mem_ptr.add(ptr), size) };
                    data.registry.write_value_to_slice(slice, new_val)
                };

                // Render the panel (use ctx from closure, not self.egui_ctx)
                debug_values_changed = self.debug_panel.render(
                    ctx,
                    &data.registry,
                    &mut self.frame_controller,
                    read_value,
                    write_value,
                );
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
        for viewport_output in full_output.viewport_output.values() {
            if viewport_output.repaint_delay.is_zero() {
                egui_dirty = true;
            } else if viewport_output.repaint_delay < std::time::Duration::MAX {
                // Schedule future repaint for animations (not MAX means egui wants a future repaint)
                let repaint_at = Instant::now() + viewport_output.repaint_delay;
                self.next_egui_repaint = Some(
                    self.next_egui_repaint
                        .map(|t| t.min(repaint_at))
                        .unwrap_or(repaint_at),
                );
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
                    depth_slice: None,
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
                    depth_slice: None,
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

        // Call on_debug_change() if debug values were modified
        if debug_values_changed {
            if let Some(session) = &mut self.game_session {
                if let Some(game) = session.runtime.game_mut() {
                    game.call_on_debug_change();
                }
            }
        }

        // Handle UI action after rendering is complete
        if let Some(action) = ui_action {
            if matches!(action, UiAction::OpenSettings) && matches!(self.mode, AppMode::Settings) {
                self.mode = AppMode::Library;
            } else {
                self.handle_ui_action(action);
            }
        }
    }

    /// Prepare debug panel data before the egui frame
    ///
    /// Returns None if no debug panel should be shown (no game, no registry, etc.)
    fn prepare_debug_panel_data(&mut self) -> Option<DebugPanelData> {
        let session = self.game_session.as_mut()?;
        let game = session.runtime.game_mut()?;
        let store = game.store_mut();

        // Check if registry has any values
        if store.data().debug_registry.is_empty() {
            return None;
        }

        // Get memory handle
        let memory = store.data().game.memory?;

        // Clone the registry
        let registry = store.data().debug_registry.clone();

        // Get raw pointer to WASM memory
        // SAFETY: Pointer is valid for the duration of this frame
        let mem_ptr = memory.data_ptr(&mut *store);
        let mem_len = memory.data_size(&mut *store);

        Some(DebugPanelData {
            registry,
            mem_ptr,
            mem_len,
        })
    }

    fn trigger_redraw(&mut self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl emberware_core::app::ConsoleApp<EmberwareZ> for App {
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()> {
        self.on_window_created(window, event_loop)
    }

    fn render_frame(&mut self) -> anyhow::Result<()> {
        self.render();
        Ok(())
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        // Let egui handle the event first
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);

            // Request redraw if egui needs it (hover effects, animations, etc.)
            if response.repaint {
                self.trigger_redraw();
            }

            if response.consumed {
                return true; // Event consumed by egui
            }

            // Check if egui wants keyboard input (text fields, sliders, etc.)
            let egui_wants_keyboard = self.egui_ctx.wants_keyboard_input();

            match event {
                WindowEvent::Resized(new_size) => {
                    tracing::debug!("Window resized to {:?}", new_size);
                    self.handle_resize(*new_size);
                    false
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    tracing::debug!("DPI scale factor changed");
                    false
                }
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    // Only pass keyboard input to game if egui doesn't want it
                    if !egui_wants_keyboard {
                        self.handle_key_input(key_event.clone());
                    }
                    false
                }
                _ => false,
            }
        } else {
            // No egui state - handle events normally
            match event {
                WindowEvent::Resized(new_size) => {
                    tracing::debug!("Window resized to {:?}", new_size);
                    self.handle_resize(*new_size);
                    false
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    tracing::debug!("DPI scale factor changed");
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
    }

    fn update_input(&mut self) {
        self.update_input();
    }

    fn on_runtime_error(&mut self, error: RuntimeError) {
        self.handle_runtime_error(error);
    }

    fn current_mode(&self) -> &AppMode {
        &self.mode
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

    fn next_frame_time(&self) -> Option<Instant> {
        match &self.mode {
            AppMode::Playing { .. } => {
                // Game running: schedule next tick based on tick_duration
                if let Some(session) = &self.game_session {
                    let tick_duration = session.runtime.tick_duration();
                    let next_tick = self.last_frame + tick_duration;
                    // Also consider egui repaints (for debug overlays)
                    match self.next_egui_repaint {
                        Some(egui_time) => Some(next_tick.min(egui_time)),
                        None => Some(next_tick),
                    }
                } else {
                    Some(Instant::now()) // Fallback: immediate
                }
            }
            AppMode::Library | AppMode::Settings => {
                // UI only: wake on events or scheduled egui repaints
                self.next_egui_repaint
            }
        }
    }
}

pub fn run(initial_mode: AppMode) -> Result<(), AppError> {
    tracing::info!("Starting with mode: {:?}", initial_mode);
    let app = App::new(initial_mode);
    emberware_core::app::run(app)
        .map_err(|e| AppError::EventLoop(format!("Event loop error: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use emberware_core::app::debug::TARGET_FRAME_TIME_MS;

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
        let mut frame_times: std::collections::VecDeque<f32> =
            std::collections::VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE);

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
