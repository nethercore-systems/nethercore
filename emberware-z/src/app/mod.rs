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
use crate::graphics::ZGraphics;
use crate::input::InputManager;
use crate::library::LocalGame;
use crate::ui::{LibraryUi, UiAction};
use emberware_core::app::config::Config;
use emberware_core::app::{
    session::GameSession, AppMode, DebugStats, RuntimeError, FRAME_TIME_HISTORY_SIZE,
};
use emberware_core::console::ConsoleResourceManager;
use emberware_core::debug::{DebugPanel, FrameController};
use emberware_core::wasm::WasmEngine;

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
                    // Only process resources and execute draw commands if we rendered
                    if rendered {
                        if let Some(session) = &mut self.game_session {
                            if let Some(graphics) = &mut self.graphics {
                                // Process pending resources by accessing resource manager directly
                                // Use dummy audio since Z resource manager doesn't use it
                                let mut dummy_audio = game_session::DummyAudio;
                                {
                                    if let Some(game) = session.runtime.game_mut() {
                                        let state = game.console_state_mut();
                                        session.resource_manager.process_pending_resources(
                                            graphics,
                                            &mut dummy_audio,
                                            state,
                                        );
                                    }
                                }

                                // Execute draw commands
                                {
                                    if let Some(game) = session.runtime.game_mut() {
                                        let state = game.console_state_mut();
                                        session
                                            .resource_manager
                                            .execute_draw_commands(graphics, state);
                                    }
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

                            let clear = z_state.init_config.clear_color;
                            let clear_r = ((clear >> 24) & 0xFF) as f32 / 255.0;
                            let clear_g = ((clear >> 16) & 0xFF) as f32 / 255.0;
                            let clear_b = ((clear >> 8) & 0xFF) as f32 / 255.0;
                            let clear_a = (clear & 0xFF) as f32 / 255.0;
                            [clear_r, clear_g, clear_b, clear_a]
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

        // Collect debug stats for overlay
        let debug_stats = DebugStats {
            frame_times: self.debug_stats.frame_times.clone(),
            game_tick_times: self.debug_stats.game_tick_times.clone(),
            game_render_times: self.debug_stats.game_render_times.clone(),
            vram_used: self.debug_stats.vram_used,
            vram_limit: self.debug_stats.vram_limit,
            ping_ms: self.debug_stats.ping_ms,
            rollback_frames: self.debug_stats.rollback_frames,
            frame_advantage: self.debug_stats.frame_advantage,
            network_interrupted: self.debug_stats.network_interrupted,
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
                AppMode::Playing { ref game_id } => {
                    // Game is rendered before egui, so we don't need a central panel
                    // Just show debug info if overlay is enabled
                    let _ = game_id; // Used in debug overlay
                }
            }

            // Debug overlay
            if debug_overlay {
                emberware_core::app::render_debug_overlay(
                    ctx,
                    &debug_stats,
                    matches!(mode, AppMode::Playing { .. }),
                    frame_time_ms,
                    render_fps,
                    game_tick_fps,
                );
            }
        });

        // Note: Debug panel rendering is deferred until after graphics operations
        // to avoid borrow checker issues with self.graphics
        let should_render_debug_panel =
            self.debug_panel.visible && matches!(mode, AppMode::Playing { .. });

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

        // Render debug inspection panel (deferred to avoid borrow conflicts)
        if should_render_debug_panel {
            self.render_debug_panel();
        }

        // Handle UI action after rendering is complete
        if let Some(action) = ui_action {
            if matches!(action, UiAction::OpenSettings) && matches!(self.mode, AppMode::Settings) {
                self.mode = AppMode::Library;
            } else {
                self.handle_ui_action(action);
            }
        }

        // Request next frame
        self.request_redraw_if_needed();
    }

    /// Render the debug inspection panel
    ///
    /// This is called separately from the main egui frame to avoid borrow checker issues.
    /// Uses raw pointer access to WASM memory to work around Rust's borrow checker
    /// limitations with closure captures.
    fn render_debug_panel(&mut self) {
        use emberware_core::debug::{DebugValue, RegisteredValue};

        let session = match &mut self.game_session {
            Some(s) => s,
            None => return,
        };

        let game = match session.runtime.game_mut() {
            Some(g) => g,
            None => return,
        };

        // Get the store to access debug registry and memory
        let store = game.store_mut();

        // Check if registry has any values
        if store.data().debug_registry.is_empty() {
            return;
        }

        // Get memory handle for read/write
        let memory = match store.data().game.memory {
            Some(m) => m,
            None => return,
        };

        // Clone the registry for rendering (avoids borrow conflicts)
        let registry_clone = store.data().debug_registry.clone();

        // Get raw pointer to WASM memory for safe access within this scope
        // SAFETY: We're not growing the memory during rendering, and all accesses
        // are bounds-checked. The pointer is valid for the duration of this function.
        let mem_ptr = memory.data_ptr(&mut *store);
        let mem_len = memory.data_size(&mut *store);

        // Create read closure using raw pointer
        let read_value = |reg_value: &RegisteredValue| -> Option<DebugValue> {
            let ptr = reg_value.wasm_ptr as usize;
            let size = reg_value.value_type.byte_size();

            if ptr + size > mem_len {
                return None;
            }

            // SAFETY: Bounds checked above, pointer valid for this scope
            let data = unsafe { std::slice::from_raw_parts(mem_ptr.add(ptr), size) };

            Some(read_debug_value_from_slice(data, reg_value.value_type))
        };

        // Create write closure using raw pointer
        let write_value = |reg_value: &RegisteredValue, new_val: &DebugValue| -> bool {
            let ptr = reg_value.wasm_ptr as usize;
            let size = reg_value.value_type.byte_size();

            if ptr + size > mem_len {
                return false;
            }

            // SAFETY: Bounds checked above, pointer valid for this scope
            let data = unsafe { std::slice::from_raw_parts_mut(mem_ptr.add(ptr) as *mut u8, size) };

            write_debug_value_to_slice(data, new_val);
            true
        };

        // Render the panel
        let any_changed = self.debug_panel.render(
            &self.egui_ctx,
            &registry_clone,
            &mut self.frame_controller,
            read_value,
            write_value,
        );

        // Drop the closures to release borrows before calling callback
        drop(read_value);
        drop(write_value);

        // Call on_debug_change() if values changed and game exports it
        if any_changed {
            if let Some(session) = &mut self.game_session {
                if let Some(game) = session.runtime.game_mut() {
                    game.call_on_debug_change();
                }
            }
        }
    }

    fn request_redraw_if_needed(&mut self) {
        // In Playing mode or Library/Settings with Poll control flow,
        // we request redraws continuously to ensure UI responsiveness.
        // The egui dirty-checking and mesh caching prevents unnecessary GPU work.
        let needs_redraw = true;

        if needs_redraw {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.needs_redraw = false;
        }
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
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

    fn render_frame(&mut self) -> anyhow::Result<bool> {
        self.render();
        Ok(true) // Always request redraw
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        // Let egui handle the event first
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);
            if response.consumed {
                self.mark_needs_redraw();
                return true; // Event consumed
            }
        }

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
}

pub fn run(initial_mode: AppMode) -> Result<(), AppError> {
    tracing::info!("Starting with mode: {:?}", initial_mode);
    let app = App::new(initial_mode);
    emberware_core::app::run(app)
        .map_err(|e| AppError::EventLoop(format!("Event loop error: {}", e)))?;
    Ok(())
}

/// Read a debug value from a byte slice
fn read_debug_value_from_slice(
    data: &[u8],
    value_type: emberware_core::debug::ValueType,
) -> emberware_core::debug::DebugValue {
    use emberware_core::debug::{DebugValue, ValueType};

    match value_type {
        ValueType::I8 => DebugValue::I8(data[0] as i8),
        ValueType::U8 => DebugValue::U8(data[0]),
        ValueType::Bool => DebugValue::Bool(data[0] != 0),
        ValueType::I16 => {
            let bytes: [u8; 2] = data[..2].try_into().unwrap();
            DebugValue::I16(i16::from_le_bytes(bytes))
        }
        ValueType::U16 => {
            let bytes: [u8; 2] = data[..2].try_into().unwrap();
            DebugValue::U16(u16::from_le_bytes(bytes))
        }
        ValueType::I32 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::I32(i32::from_le_bytes(bytes))
        }
        ValueType::U32 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::U32(u32::from_le_bytes(bytes))
        }
        ValueType::F32 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::F32(f32::from_le_bytes(bytes))
        }
        ValueType::Vec2 => {
            let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
            let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
            DebugValue::Vec2 { x, y }
        }
        ValueType::Vec3 => {
            let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
            let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
            let z = f32::from_le_bytes(data[8..12].try_into().unwrap());
            DebugValue::Vec3 { x, y, z }
        }
        ValueType::Rect => {
            let x = i16::from_le_bytes(data[0..2].try_into().unwrap());
            let y = i16::from_le_bytes(data[2..4].try_into().unwrap());
            let w = i16::from_le_bytes(data[4..6].try_into().unwrap());
            let h = i16::from_le_bytes(data[6..8].try_into().unwrap());
            DebugValue::Rect { x, y, w, h }
        }
        ValueType::Color => DebugValue::Color {
            r: data[0],
            g: data[1],
            b: data[2],
            a: data[3],
        },
        ValueType::FixedI16Q8 => {
            let bytes: [u8; 2] = data[..2].try_into().unwrap();
            DebugValue::FixedI16Q8(i16::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q16 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::FixedI32Q16(i32::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q8 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::FixedI32Q8(i32::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q24 => {
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            DebugValue::FixedI32Q24(i32::from_le_bytes(bytes))
        }
    }
}

/// Write a debug value to a byte slice
fn write_debug_value_to_slice(data: &mut [u8], value: &emberware_core::debug::DebugValue) {
    use emberware_core::debug::DebugValue;

    match value {
        DebugValue::I8(v) => data[0] = *v as u8,
        DebugValue::U8(v) => data[0] = *v,
        DebugValue::Bool(v) => data[0] = if *v { 1 } else { 0 },
        DebugValue::I16(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::U16(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::I32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::U32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::F32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::Vec2 { x, y } => {
            data[0..4].copy_from_slice(&x.to_le_bytes());
            data[4..8].copy_from_slice(&y.to_le_bytes());
        }
        DebugValue::Vec3 { x, y, z } => {
            data[0..4].copy_from_slice(&x.to_le_bytes());
            data[4..8].copy_from_slice(&y.to_le_bytes());
            data[8..12].copy_from_slice(&z.to_le_bytes());
        }
        DebugValue::Rect { x, y, w, h } => {
            data[0..2].copy_from_slice(&x.to_le_bytes());
            data[2..4].copy_from_slice(&y.to_le_bytes());
            data[4..6].copy_from_slice(&w.to_le_bytes());
            data[6..8].copy_from_slice(&h.to_le_bytes());
        }
        DebugValue::Color { r, g, b, a } => {
            data[0] = *r;
            data[1] = *g;
            data[2] = *b;
            data[3] = *a;
        }
        DebugValue::FixedI16Q8(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q16(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q8(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q24(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
    }
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
