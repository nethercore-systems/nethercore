//! Examples Common - Shared utilities for Nethercore examples
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

/// Button indices for input functions
pub mod button {
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const LEFT: u32 = 2;
    pub const RIGHT: u32 = 3;
    pub const A: u32 = 4;
    pub const B: u32 = 5;
    pub const X: u32 = 6;
    pub const Y: u32 = 7;
    pub const L1: u32 = 8;
    pub const R1: u32 = 9;
    pub const L3: u32 = 10;
    pub const R3: u32 = 11;
    pub const START: u32 = 12;
    pub const SELECT: u32 = 13;
}

pub use ffi::*;
pub use camera::*;
pub use sky::*;
pub use environment::*;
pub use shapes::*;
pub use color::*;
pub use debug::*;
pub use texture::*;
