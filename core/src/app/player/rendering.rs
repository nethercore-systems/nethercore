//! Rendering logic including egui overlays, debug panels, and screen capture

use std::cell::RefCell;

use smallvec::SmallVec;
use winit::window::Fullscreen;

use crate::capture::{CaptureSupport, read_render_target_pixels};
use crate::console::{Audio, Console, ConsoleResourceManager};
use crate::debug::ActionRequest;
use crate::debug::registry::RegisteredValue;
use crate::debug::types::DebugValue;
use crate::rollback::{ConnectionQuality, SessionType};

use super::connection::JoinConnectionAction;
use super::error_ui::JoinConnectionState;

use super::super::ui::SettingsAction;
use super::StandaloneApp;
use super::error_ui::{ErrorAction, render_error_screen};
use super::types::{RomLoader, StandaloneGraphicsSupport};

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Main render function: renders game, overlays, and handles UI interactions
    pub(super) fn render_impl(&mut self) {
        let mut restart_requested = false;
        let mut join_retry_requested = false;

        // If a screenshot/GIF frame is pending, ensure the render target is freshly rendered
        // on this redraw, even if the sim loop didn't request a new render.
        let needs_capture = self.capture.needs_capture();

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

            let mut encoder = runner.graphics().device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Standalone Frame Encoder"),
                },
            );

            // Render game if we have new content, or if we need a fresh frame for capture.
            if self.last_sim_rendered || needs_capture {
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
            if self.debug_overlay
                || self.debug_panel.visible
                || self.settings_ui.visible
                || self.error_state.is_some()
                || self.network_overlay_visible
                || self.waiting_for_peer.is_some()
                || self.joining_peer.is_some()
                || self.console_debug_panel_visible
            {
                let pending_writes: RefCell<Vec<(RegisteredValue, DebugValue)>> =
                    RefCell::new(Vec::new());
                let pending_action: RefCell<Option<ActionRequest>> = RefCell::new(None);
                let settings_action: RefCell<SettingsAction> = RefCell::new(SettingsAction::None);
                let error_action: RefCell<ErrorAction> = RefCell::new(ErrorAction::None);
                let join_action: RefCell<JoinConnectionAction> =
                    RefCell::new(JoinConnectionAction::None);

                if let (Some(egui_state), Some(egui_renderer), Some(window)) =
                    (&mut self.egui_state, &mut self.egui_renderer, &self.window)
                {
                    let raw_input = egui_state.take_egui_input(window);

                    let debug_overlay = self.debug_overlay;
                    let debug_stats = &self.debug_stats;
                    let game_tick_times = &self.game_tick_times;
                    let debug_panel = &mut self.debug_panel;
                    let frame_controller = &mut self.frame_controller;
                    let settings_ui = &mut self.settings_ui;
                    let error_state_ref = &self.error_state;
                    let waiting_for_peer_ref = &self.waiting_for_peer;
                    let joining_peer_ref = &self.joining_peer;
                    let network_overlay_visible = self.network_overlay_visible;

                    // Get network session info for overlay
                    // Use SmallVec to avoid heap allocations (max 4 players)
                    let (
                        session_type,
                        network_stats,
                        local_players,
                        total_rollbacks,
                        current_frame,
                    ): (_, _, SmallVec<[usize; 4]>, _, _) = {
                        if let Some(game_session) = runner.session() {
                            if let Some(rollback) = game_session.runtime.session() {
                                (
                                    rollback.session_type(),
                                    rollback.all_player_stats().to_vec(),
                                    rollback.local_players().iter().copied().collect(),
                                    rollback.total_rollback_frames(),
                                    rollback.current_frame(),
                                )
                            } else {
                                (SessionType::Local, Vec::new(), SmallVec::new(), 0, 0)
                            }
                        } else {
                            (SessionType::Local, Vec::new(), SmallVec::new(), 0, 0)
                        }
                    };

                    // Console debug panel visibility flag and pointer
                    let console_debug_visible = self.console_debug_panel_visible;

                    // Sync debug UI state before rendering (enables EPU lock mode, etc.)
                    if console_debug_visible
                        && let Some(session) = runner.session_mut()
                    {
                        let (console, state_opt) = session.runtime.console_and_state_mut();
                        if let Some(state) = state_opt {
                            console.sync_debug_ui_state(state);
                        }
                    }

                    // SAFETY: We use a raw pointer to avoid borrow conflicts between
                    // console (in session) and graphics (separate field). The pointer
                    // is only used during egui_ctx.run(), before any graphics access.
                    let console_ptr: Option<*mut C> =
                        runner.console_mut().map(|c| c as *mut C);

                    let full_output = self.egui_ctx.run(raw_input, |ctx| {
                        let action = settings_ui.show_as_window(ctx);
                        if !matches!(action, SettingsAction::None) {
                            *settings_action.borrow_mut() = action;
                        }
                        if debug_overlay {
                            let frame_time_ms = debug_stats.frame_times.back().copied().unwrap_or(16.67);
                            let render_fps = super::super::debug::calculate_fps(game_tick_times);
                            super::super::debug::render_debug_overlay(
                                ctx,
                                debug_stats,
                                true,
                                frame_time_ms,
                                render_fps,
                                render_fps,
                            );
                        }

                        // Network statistics overlay (F12)
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

                        if debug_panel.visible
                            && let Some(session) = runner.session()
                                && let Some(game) = session.runtime.game()
                            {
                                let registry = game.store().data().debug_registry.clone();

                                let read_value = |reg_val: &RegisteredValue| -> Option<DebugValue> {
                                    let mem = game.store().data().game.memory?;
                                    let data = mem.data(game.store());

                                    let ptr = reg_val.wasm_ptr as usize;
                                    let size = reg_val.value_type.byte_size();
                                    let end = ptr.checked_add(size)?;
                                    if end > data.len() {
                                        return None;
                                    }

                                    Some(registry.read_value_from_slice(
                                        &data[ptr..end],
                                        reg_val.value_type,
                                    ))
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
                                    &registry,
                                    frame_controller,
                                    read_value,
                                    write_value,
                                );
                                if let Some(action) = action {
                                    *pending_action.borrow_mut() = Some(action);
                                }
                            }

                        // Waiting for peer connection dialog (Host mode)
                        if let Some(waiting) = waiting_for_peer_ref {
                            egui::Window::new("Hosting Game")
                                .collapsible(false)
                                .resizable(false)
                                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                                .show(ctx, |ui| {
                                    ui.set_min_width(400.0);

                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);
                                        ui.spinner();
                                        ui.add_space(10.0);
                                        ui.label("Waiting for player to connect...");
                                        ui.add_space(15.0);
                                    });

                                    ui.separator();
                                    ui.add_space(10.0);

                                    ui.label("Share one of these links with your friend:");
                                    ui.add_space(5.0);

                                    for ip in &waiting.local_ips {
                                        let join_url = waiting.join_url(ip);
                                        ui.horizontal(|ui| {
                                            // Show truncated URL for display
                                            let display_url = if join_url.len() > 50 {
                                                format!("{}...", &join_url[..47])
                                            } else {
                                                join_url.clone()
                                            };
                                            ui.monospace(&display_url);
                                            if ui.small_button("Copy").clicked() {
                                                ctx.copy_text(join_url.clone());
                                            }
                                        });
                                    }

                                    ui.add_space(10.0);

                                    // Collapsible section for manual connection
                                    ui.collapsing("Manual connection (IP:port)", |ui| {
                                        ui.label(
                                            egui::RichText::new("If the link doesn't work, share this address:")
                                                .weak()
                                                .small(),
                                        );
                                        ui.add_space(5.0);
                                        for ip in &waiting.local_ips {
                                            let addr = format!("{}:{}", ip, waiting.port);
                                            ui.horizontal(|ui| {
                                                ui.monospace(&addr);
                                                if ui.small_button("Copy").clicked() {
                                                    ctx.copy_text(addr.clone());
                                                }
                                            });
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new("Your friend can paste the link in their browser or use 'Join Game'")
                                            .weak()
                                            .small(),
                                    );
                                    ui.add_space(10.0);
                                });
                        }

                        // Joining peer connection dialog (Join mode)
                        if let Some(joining) = joining_peer_ref {
                            egui::Window::new("Joining Game")
                                .collapsible(false)
                                .resizable(false)
                                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                                .show(ctx, |ui| {
                                    ui.set_min_width(350.0);

                                    match joining.state {
                                        JoinConnectionState::Connecting => {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.spinner();
                                                ui.add_space(10.0);
                                                ui.label("Connecting to host...");
                                                ui.add_space(5.0);
                                                ui.monospace(&joining.address);
                                                ui.add_space(15.0);
                                            });

                                            // Progress bar showing timeout
                                            let progress = 1.0
                                                - (joining.remaining().as_secs_f32()
                                                    / joining.timeout.as_secs_f32());
                                            ui.add(
                                                egui::ProgressBar::new(progress)
                                                    .text(format!(
                                                        "Attempt {} - {:.0}s remaining",
                                                        joining.attempt_count,
                                                        joining.remaining().as_secs_f32()
                                                    ))
                                                    .animate(true),
                                            );

                                            ui.add_space(15.0);
                                            ui.separator();
                                            ui.add_space(10.0);

                                            if ui.button("Cancel").clicked() {
                                                *join_action.borrow_mut() =
                                                    JoinConnectionAction::Cancel;
                                            }
                                        }
                                        JoinConnectionState::WaitingForResponse => {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.spinner();
                                                ui.add_space(10.0);
                                                ui.label("Waiting for host response...");
                                                ui.add_space(5.0);
                                                ui.monospace(&joining.address);
                                                ui.add_space(15.0);
                                            });

                                            ui.separator();
                                            ui.add_space(10.0);

                                            if ui.button("Cancel").clicked() {
                                                *join_action.borrow_mut() =
                                                    JoinConnectionAction::Cancel;
                                            }
                                        }
                                        JoinConnectionState::Connected => {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    egui::RichText::new("Connected!")
                                                        .size(18.0)
                                                        .color(egui::Color32::GREEN),
                                                );
                                                ui.add_space(10.0);
                                                ui.label("Starting game...");
                                                ui.add_space(15.0);
                                            });
                                        }
                                        JoinConnectionState::TimedOut => {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    egui::RichText::new("Connection Timed Out")
                                                        .size(18.0)
                                                        .color(egui::Color32::YELLOW),
                                                );
                                                ui.add_space(10.0);
                                            });

                                            ui.label("Could not reach the host. Possible causes:");
                                            ui.add_space(5.0);
                                            ui.horizontal(|ui| {
                                                ui.label("  *");
                                                ui.label("The host may not be running");
                                            });
                                            ui.horizontal(|ui| {
                                                ui.label("  *");
                                                ui.label("Firewall may be blocking the connection");
                                            });
                                            ui.horizontal(|ui| {
                                                ui.label("  *");
                                                ui.label("The address may be incorrect");
                                            });

                                            ui.add_space(15.0);
                                            ui.separator();
                                            ui.add_space(10.0);

                                            ui.horizontal(|ui| {
                                                if ui.button("Retry").clicked() {
                                                    *join_action.borrow_mut() =
                                                        JoinConnectionAction::Retry;
                                                }
                                                ui.add_space(20.0);
                                                if ui.button("Cancel").clicked() {
                                                    *join_action.borrow_mut() =
                                                        JoinConnectionAction::Cancel;
                                                }
                                            });

                                            ui.add_space(5.0);
                                            ui.label(
                                                egui::RichText::new("Press Escape to cancel")
                                                    .weak()
                                                    .small(),
                                            );
                                        }
                                        JoinConnectionState::Failed => {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    egui::RichText::new("Connection Failed")
                                                        .size(18.0)
                                                        .color(egui::Color32::RED),
                                                );
                                                ui.add_space(10.0);
                                            });

                                            if let Some(ref error) = joining.error {
                                                ui.label(error);
                                            }

                                            ui.add_space(15.0);
                                            ui.separator();
                                            ui.add_space(10.0);

                                            ui.horizontal(|ui| {
                                                if ui.button("Retry").clicked() {
                                                    *join_action.borrow_mut() =
                                                        JoinConnectionAction::Retry;
                                                }
                                                ui.add_space(20.0);
                                                if ui.button("Cancel").clicked() {
                                                    *join_action.borrow_mut() =
                                                        JoinConnectionAction::Cancel;
                                                }
                                            });

                                            ui.add_space(5.0);
                                            ui.label(
                                                egui::RichText::new("Press Escape to cancel")
                                                    .weak()
                                                    .small(),
                                            );
                                        }
                                    }
                                });
                        }

                        if let Some(error) = error_state_ref {
                            let action = render_error_screen(ctx, error);
                            if action != ErrorAction::None {
                                *error_action.borrow_mut() = action;
                            }
                        }

                        // Console-specific debug panel (e.g., EPU panel for ZX)
                        if let Some(ptr) = console_ptr {
                            // SAFETY: The pointer is valid for the duration of this closure,
                            // and no other code accesses the console during this time.
                            unsafe {
                                <C as Console>::render_debug_ui(&mut *ptr, ctx, console_debug_visible);
                            }
                        }
                    });

                    egui_state.handle_platform_output(window, full_output.platform_output);

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
                if !writes.is_empty()
                    && let Some(session) = runner.session_mut()
                    && let Some(game) = session.runtime.game_mut()
                {
                    let has_debug_callback = game.has_debug_change_callback();
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

                // Apply pending action
                if let Some(action_req) = pending_action.into_inner()
                    && let Some(session) = runner.session_mut()
                    && let Some(game) = session.runtime.game_mut()
                    && let Err(e) = game.call_action(&action_req.func_name, &action_req.args)
                {
                    tracing::warn!("Debug action '{}' failed: {}", action_req.func_name, e);
                }

                // Apply settings actions
                match settings_action.into_inner() {
                    SettingsAction::None => {}
                    SettingsAction::Close => {
                        // Settings panel was closed, nothing else to do
                    }
                    SettingsAction::ToggleFullscreen(fullscreen) => {
                        if let Some(window) = &self.window {
                            if fullscreen {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                    }
                    SettingsAction::PreviewScaleMode(scale_mode) => {
                        self.scale_mode = scale_mode;
                        runner.graphics_mut().set_scale_mode(scale_mode);
                    }
                    SettingsAction::SetVolume(volume) => {
                        if let Some(session) = runner.session_mut()
                            && let Some(audio) = session.runtime.audio_mut()
                        {
                            audio.set_master_volume(volume);
                        }
                    }
                    SettingsAction::ResetDefaults => {
                        // Defaults were applied to temp config in UI, nothing else needed
                    }
                    SettingsAction::Save(config) => {
                        // Update local state from the saved config
                        self.scale_mode = config.video.scale_mode;
                        runner
                            .graphics_mut()
                            .set_scale_mode(config.video.scale_mode);
                        if let Some(window) = &self.window {
                            if config.video.fullscreen {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        if let Some(session) = runner.session_mut()
                            && let Some(audio) = session.runtime.audio_mut()
                        {
                            audio.set_master_volume(config.audio.master_volume);
                        }
                        // Update input manager with new keyboard mappings
                        self.input_manager.update_config(config.input.clone());
                        // Save to disk
                        if let Err(e) = super::super::config::save(&config) {
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

                // Apply join connection actions
                match join_action.into_inner() {
                    JoinConnectionAction::None => {}
                    JoinConnectionAction::Retry => {
                        join_retry_requested = true;
                    }
                    JoinConnectionAction::Cancel => {
                        tracing::info!("Join connection cancelled by user");
                        self.joining_peer = None;
                        self.should_exit = true;
                    }
                }
            }

            runner
                .graphics()
                .queue()
                .submit(std::iter::once(encoder.finish()));
            surface_texture.present();

            // Process screen capture
            if needs_capture {
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

        if join_retry_requested {
            self.retry_join_connection();
        }
    }
}
