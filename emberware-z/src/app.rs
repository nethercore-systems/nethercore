//! Application state and main loop

use thiserror::Error;

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
}

pub fn run(initial_mode: AppMode) -> Result<(), AppError> {
    tracing::info!("Starting with mode: {:?}", initial_mode);
    // TODO: Implement main application loop
    // - Initialize wgpu
    // - Initialize egui
    // - Handle mode switching
    // - Run game loop
    Ok(())
}
