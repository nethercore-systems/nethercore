//! Generic application event loop for any console implementation
//!
//! This module provides console-agnostic event loop infrastructure that works
//! with any fantasy console implementing the Console trait.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

use crate::console::Console;

use super::types::{AppMode, RuntimeError};

/// Trait for console-specific application behavior.
///
/// Implement this trait to define how your console handles windowing,
/// rendering, input, and UI. The generic event loop in core will call
/// these methods at appropriate times.
///
/// # Example
///
/// ```rust,ignore
/// use emberware_core::app::event_loop::ConsoleApp;
/// use emberware_core::console::Console;
///
/// struct ZApp {
///     graphics: ZGraphics,
///     audio: ZAudio,
///     // ... other fields
/// }
///
/// impl ConsoleApp<EmberwareZ> for ZApp {
///     fn on_window_created(&mut self, window: Arc<Window>) -> Result<()> {
///         // Initialize graphics with window
///         Ok(())
///     }
///
///     fn render_frame(&mut self) -> Result<bool> {
///         // Render game + UI
///         Ok(true) // Request redraw
///     }
///
///     // ... implement other methods
/// }
/// ```
pub trait ConsoleApp<C: Console>: Sized {
    /// Called when the window is created or resumed.
    ///
    /// Initialize graphics and any window-dependent resources here.
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()>;

    /// Render one frame (game + UI composite).
    ///
    /// Called once per display frame. Frame scheduling is handled by
    /// `next_frame_time()` in `about_to_wait()`.
    fn render_frame(&mut self) -> anyhow::Result<()>;

    /// Handle a window event.
    ///
    /// Return `true` if the event was consumed (prevents default handling).
    fn on_window_event(&mut self, event: &WindowEvent) -> bool;

    /// Update input state before game frame execution.
    ///
    /// Called once per frame before `render_frame()`.
    fn update_input(&mut self);

    /// Handle a critical runtime error.
    ///
    /// Examples: game crash, network disconnect, WASM panic
    fn on_runtime_error(&mut self, error: RuntimeError);

    /// Get the current application mode.
    fn current_mode(&self) -> &AppMode;

    /// Check if application should exit.
    fn should_exit(&self) -> bool;

    /// Mark that application should exit.
    fn request_exit(&mut self);

    /// Request a redraw from the event loop.
    fn request_redraw(&self);

    /// When is the next frame needed? Returns None to wait for events, Some(instant) for scheduled render.
    fn next_frame_time(&self) -> Option<std::time::Instant>;
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
            // Create window
            let window_attributes = Window::default_attributes()
                .with_title("Emberware")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

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
            // Let app handle the event first
            if app.on_window_event(&event) {
                return; // Event was consumed
            }

            // Handle common events
            match event {
                WindowEvent::CloseRequested => {
                    tracing::info!("Window close requested");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Update input
                    app.update_input();

                    // Render frame
                    if let Err(e) = app.render_frame() {
                        tracing::error!("Render error: {}", e);
                    }

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
        if let Some(app) = &self.app {
            match app.next_frame_time() {
                Some(next_time) => {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(next_time));
                    // Still request redraw so we wake up at the right time
                    app.request_redraw();
                }
                None => {
                    event_loop.set_control_flow(ControlFlow::Wait);
                }
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
/// use emberware_core::app::{event_loop, AppMode};
///
/// let app = ZApp::new(AppMode::Library)?;
/// event_loop::run(app)?;
/// ```
pub fn run<C: Console, A: ConsoleApp<C>>(app: A) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    let mut handler = AppEventHandler::new(app);
    event_loop.run_app(&mut handler)?;

    Ok(())
}
