//! ConsoleApp trait implementation for StandaloneApp

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::console::Console;

use super::super::event_loop::ConsoleApp;
use super::types::{RomLoader, StandaloneGraphicsSupport};
use super::StandaloneApp;
use super::super::{GameErrorPhase, RuntimeError, parse_wasm_error};

impl<C, L> ConsoleApp<C> for StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        self.on_window_created_impl(window, event_loop)
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
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
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

        // Poll for peer connection in Host mode
        if self.poll_for_peer_connection() {
            return;
        }

        // Still waiting for peer - don't run game simulation
        if self.waiting_for_peer.is_some() {
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

                let game_error =
                    parse_wasm_error(&anyhow::anyhow!("{}", error_msg), tick_before, phase);
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
        self.render_impl();
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

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Returns the tick duration based on the current session's tick rate
    pub(super) fn tick_duration(&self) -> Duration {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
        {
            return session.runtime.tick_duration();
        }
        Duration::from_secs_f64(1.0 / 60.0)
    }
}
