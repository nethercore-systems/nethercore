//! Application state and main loop

use std::sync::Arc;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::config::{self, Config};
use crate::graphics::ZGraphics;
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
    /// Whether the application should exit
    should_exit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(initial_mode: AppMode) -> Self {
        let config = config::load();
        Self {
            mode: initial_mode,
            config,
            window: None,
            graphics: None,
            should_exit: false,
        }
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
        if key_event.state == ElementState::Pressed {
            match key_event.physical_key {
                PhysicalKey::Code(KeyCode::F11) => {
                    self.toggle_fullscreen();
                }
                PhysicalKey::Code(KeyCode::Enter) => {
                    // Alt+Enter for fullscreen toggle
                    if key_event.state == ElementState::Pressed {
                        // Note: Alt modifier check would go here
                        // For now, we use F11 as the primary method
                    }
                }
                PhysicalKey::Code(KeyCode::Escape) => {
                    // Return to library when in game
                    match self.mode {
                        AppMode::Playing { .. } => {
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
        match pollster::block_on(ZGraphics::new(window.clone())) {
            Ok(graphics) => {
                tracing::info!("Graphics initialized successfully");
                self.graphics = Some(graphics);
                self.window = Some(window);
            }
            Err(e) => {
                tracing::error!("Failed to initialize graphics: {}", e);
                self.should_exit = true;
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                if let Some(window) = &self.window {
                    // TODO: Render frame based on current mode
                    // For now, just request another redraw
                    window.request_redraw();
                }
            }
            _ => {}
        }

        if self.should_exit {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
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
