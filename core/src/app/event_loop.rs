//! Generic application event loop for console game players
//!
//! This module provides event loop infrastructure for running games.
//! The library UI uses its own separate event loop.
//!
//! ## Event Loop Model
//!
//! The event loop follows a clear separation of concerns:
//!
//! - **WindowEvent**: Handle input, mark `needs_redraw = true`
//! - **about_to_wait**: Advance simulation when tick is due; set `WaitUntil(next_tick)`;
//!   request redraw only if `needs_redraw` is true
//! - **RedrawRequested**: Render game + UI, clear `needs_redraw`
//!
//! This ensures:
//! - Uses `WaitUntil(next_tick)` without busy-spinning
//! - Redraws only happen when state actually changed

use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

use crate::console::Console;

use super::types::RuntimeError;

/// Trait for console-specific application behavior.
///
/// Implement this trait to define how your console handles windowing,
/// rendering, input, and UI. The generic event loop in core will call
/// these methods at appropriate times.
///
/// ## Key Design
///
/// Simulation and rendering are separated:
/// - `advance_simulation()` is called from `about_to_wait` when a tick is due
/// - `render()` is called from `RedrawRequested` and does NOT advance simulation
///
/// The `needs_redraw` flag controls when redraws are requested:
/// - Set true on input events, simulation advances, mode changes
/// - Cleared after rendering
pub trait ConsoleApp<C: Console>: Sized {
    // === Window lifecycle ===

    /// Called when the window is created or resumed.
    ///
    /// Initialize graphics and any window-dependent resources here.
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()>;

    /// Handle a window event.
    ///
    /// Return `true` if the event was consumed (e.g., by egui).
    /// When true, the event loop will mark `needs_redraw`.
    fn on_window_event(&mut self, event: &WindowEvent) -> bool;

    // === Simulation control ===

    /// Get the scheduled time for next simulation tick.
    fn next_tick(&self) -> Instant;

    /// Advance simulation by one tick.
    ///
    /// Called from `about_to_wait` when `now >= next_tick()`.
    /// Should run the game's update logic, execute draw commands, process audio.
    fn advance_simulation(&mut self);

    /// Update next_tick after simulation.
    ///
    /// Called after `advance_simulation()`. Should set `next_tick += tick_duration`.
    fn update_next_tick(&mut self);

    // === Rendering ===

    /// Render the current frame (game + UI).
    ///
    /// Does NOT advance simulation - that's done in `advance_simulation()`.
    /// Called from `RedrawRequested`.
    fn render(&mut self);

    // === Redraw flag ===

    /// Check if a redraw is needed.
    fn needs_redraw(&self) -> bool;

    /// Mark that a redraw is needed.
    fn mark_needs_redraw(&mut self);

    /// Clear the redraw flag after rendering.
    fn clear_needs_redraw(&mut self);

    // === Application lifecycle ===

    /// Handle a critical runtime error.
    ///
    /// Examples: game crash, network disconnect, WASM panic
    fn on_runtime_error(&mut self, error: RuntimeError);

    /// Check if application should exit.
    fn should_exit(&self) -> bool;

    /// Mark that application should exit.
    fn request_exit(&mut self);

    /// Request a redraw from the window.
    ///
    /// Calls `window.request_redraw()`.
    fn request_redraw(&self);
}

/// Generic event loop handler for any Console implementation.
///
/// This wraps a `ConsoleApp<C>` and implements winit's `ApplicationHandler`
/// to provide a console-agnostic event loop.
pub struct AppEventHandler<C: Console, A: ConsoleApp<C>> {
    app: Option<A>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Console, A: ConsoleApp<C>> AppEventHandler<C, A> {
    /// Create a new event handler with the given app.
    pub fn new(app: A) -> Self {
        Self {
            app: Some(app),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Console, A: ConsoleApp<C>> ApplicationHandler for AppEventHandler<C, A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app) = &mut self.app {
            // Create window (don't set control flow here - about_to_wait will do it)
            // Note: minimum size is set by the app in on_window_created() based on game resolution
            let (width, height) = C::specs().resolution;
            let window_attributes = Window::default_attributes()
                .with_title(C::specs().name)
                .with_inner_size(winit::dpi::PhysicalSize::new(width, height));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    let window = Arc::new(window);
                    if let Err(e) = app.on_window_created(window, event_loop) {
                        tracing::error!("Failed to initialize window: {}", e);
                        event_loop.exit();
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(app) = &mut self.app {
            // Let app handle the event first (egui, input, etc.)
            if app.on_window_event(&event) {
                app.mark_needs_redraw();
                app.request_redraw(); // Request immediately for responsive input
                return;
            }

            // Handle common events
            match event {
                WindowEvent::CloseRequested => {
                    tracing::info!("Window close requested");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // ONLY render here - simulation already happened in about_to_wait
                    app.render();
                    app.clear_needs_redraw();

                    // Check if app wants to exit
                    if app.should_exit() {
                        event_loop.exit();
                    }
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app) = &mut self.app {
            // Check if tick is due
            let now = Instant::now();
            if now >= app.next_tick() {
                app.advance_simulation();
                app.update_next_tick();
                app.mark_needs_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(app.next_tick()));

            if app.needs_redraw() {
                app.request_redraw();
            }
        }
    }
}

/// Run the event loop with a console application.
///
/// This is the main entry point for running a fantasy console with the
/// generic event loop infrastructure.
///
/// # Example
///
/// ```rust,ignore
/// use nethercore_core::app::{event_loop, AppMode};
///
/// let app = ZApp::new(AppMode::Library)?;
/// event_loop::run(app)?;
/// ```
pub fn run<C: Console, A: ConsoleApp<C>>(app: A) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    // Don't set control flow here - about_to_wait will set it dynamically

    let mut handler = AppEventHandler::new(app);
    event_loop.run_app(&mut handler)?;

    Ok(())
}
