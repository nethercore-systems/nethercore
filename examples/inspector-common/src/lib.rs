//! Inspector Common - Shared utilities for inspector examples
//!
//! Provides common functionality for all mode inspector examples:
//! - FFI declarations
//! - Debug camera controls
//! - Debug sky controls
//! - Color utilities
//! - Shape management

#![no_std]

pub mod ffi;
pub mod camera;
pub mod sky;
pub mod shapes;
pub mod color;
pub mod debug;

pub use ffi::*;
pub use camera::*;
pub use sky::*;
pub use shapes::*;
pub use color::*;
pub use debug::*;
