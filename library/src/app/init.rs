//! Application initialization error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Event loop error: {0}")]
    EventLoop(String),
}
