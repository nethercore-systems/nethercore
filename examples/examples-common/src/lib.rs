//! Examples Common - Shared utilities for Emberware examples
//!
//! Provides common functionality for examples:
//! - FFI declarations
//! - Debug camera controls
//! - Debug sky controls (legacy)
//! - Debug environment controls (Multi-Environment v3)
//! - Color utilities
//! - Shape management
//! - Texture utilities

#![no_std]

pub mod ffi;
pub mod camera;
pub mod sky;
pub mod environment;
pub mod shapes;
pub mod color;
pub mod debug;
pub mod texture;

pub use ffi::*;
pub use camera::*;
pub use sky::*;
pub use environment::*;
pub use shapes::*;
pub use color::*;
pub use debug::*;
pub use texture::*;
