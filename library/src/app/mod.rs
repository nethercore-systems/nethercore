//! Application state and main loop

mod debug;
mod game_session;
mod init;
mod ui;

pub use init::AppError;

use std::sync::Arc;
use std::time::Instant;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

use emberware_z::console::EmberwareZ;
use emberware_z::ffi::unpack_rgba;
use emberware_z::graphics::ZGraphics;
use emberware_z::input::InputManager;
use emberware_z::library::LocalGame;
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
    pub(crate) settings_ui: crate::ui::SettingsUi,
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
    /// Whether a redraw is needed (UI state changed)
    pub(crate) needs_redraw: bool,
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
    /// Next scheduled simulation tick (used for WaitUntil in game mode)
    pub(crate) next_tick: Instant,
    /// Whether the last advance_simulation() call rendered new game content
    /// Used by render() to know whether to render new game content or blit the last frame
    pub(crate) last_sim_rendered: bool,
}

impl App {
    /// Render the current frame
    fn render(&mut self) {
        let now = Instant::now();

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

        // Simulation is now done in advance_simulation(), called from about_to_wait.
        // We just use the result (last_sim_rendered) to know if we have new content to render.
        let game_rendered_this_frame = self.last_sim_rendered;

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

        // Collect debug stats for overlay only in game mode (meaningless in library mode)
        let debug_stats = if debug_overlay && matches!(mode, AppMode::Playing { .. }) {
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
        // Note: We no longer request redraws here - the event loop handles that
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

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Advance simulation by one tick
    ///
    /// Called from the event loop's about_to_wait when a tick is due.
    /// This runs the game's update logic, NOT rendering.
    fn advance_simulation_internal(&mut self) {
        // Reset last_sim_rendered - will be set by run_game_frame if it rendered
        self.last_sim_rendered = false;

        // Only run simulation if in Playing mode
        if !matches!(self.mode, AppMode::Playing { .. }) {
            return;
        }

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

        // Run game frame (update + render commands)
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

        self.last_sim_rendered = did_render;

        // If game requested quit, return to library
        if !game_running {
            self.game_session = None;
            self.mode = AppMode::Library;
            self.local_games = emberware_z::library::get_local_games(&emberware_z::library::ZDataDirProvider);
        }
    }
}

impl emberware_core::app::ConsoleApp<EmberwareZ> for App {
    // === Window lifecycle ===

    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()> {
        self.on_window_created(window, event_loop)
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        // Let egui handle the event first
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);

            // Mark redraw if egui wants one (hover effects, animations, etc.)
            // Don't call request_redraw() immediately - let about_to_wait batch events
            if response.repaint {
                self.mark_needs_redraw();
            }

            if response.consumed {
                return true; // Event consumed
            }

            // Check if egui wants keyboard input (text fields, sliders, etc.)
            let egui_wants_keyboard = self.egui_ctx.wants_keyboard_input();

            match event {
                WindowEvent::Resized(new_size) => {
                    tracing::debug!("Window resized to {:?}", new_size);
                    self.handle_resize(*new_size);
                    self.mark_needs_redraw();
                    false
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    tracing::debug!("DPI scale factor changed");
                    self.mark_needs_redraw();
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
                    self.mark_needs_redraw();
                    false
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    tracing::debug!("DPI scale factor changed");
                    self.mark_needs_redraw();
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

    // === Simulation control ===

    fn has_active_game(&self) -> bool {
        self.game_session.is_some()
    }

    fn next_tick(&self) -> Instant {
        self.next_tick
    }

    fn advance_simulation(&mut self) {
        // Update input before advancing simulation
        self.update_input();
        // Run the actual simulation logic
        self.advance_simulation_internal();
    }

    fn update_next_tick(&mut self) {
        if let Some(session) = &self.game_session {
            self.next_tick += session.runtime.tick_duration();
        }
    }

    // === Rendering ===

    fn render(&mut self) {
        self.render();
    }

    // === Redraw flag ===

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn clear_needs_redraw(&mut self) {
        self.needs_redraw = false;
    }

    // === Application lifecycle ===

    fn on_runtime_error(&mut self, error: RuntimeError) {
        self.handle_runtime_error(error);
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

pub fn run(initial_mode: AppMode) -> Result<(), AppError> {
    tracing::info!("Starting with mode: {:?}", initial_mode);
    let app = App::new(initial_mode);
    emberware_core::app::run(app)
        .map_err(|e| AppError::EventLoop(format!("Event loop error: {}", e)))?;
    Ok(())
}
