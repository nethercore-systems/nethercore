//! Nethercore ZX FFI Bindings
//!
//! This module provides all FFI function declarations for Nethercore ZX games.
//! Import this module to access the complete Nethercore ZX API.
//!
//! # Usage
//!
//! ```rust,ignore
//! #![no_std]
//! #![no_main]
//!
//! // Include the FFI bindings
//! mod ffi;
//! use ffi::*;
//!
//! #[no_mangle]
//! pub extern "C" fn init() {
//!     set_clear_color(0x1a1a2eFF);
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn update() {
//!     // Game logic here
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn render() {
//!     // Optional: EPU environment background.
//!     // `config_ptr` points to 16 u64 values (128 bytes total).
//!     // epu_set(config_ptr);
//!     // Draw your scene
//!     // draw_epu();
//! }
//! ```
//!
//! # Game Lifecycle
//!
//! All Nethercore games must export three functions:
//! - `init()` — Called once at startup
//! - `update()` — Called every tick (deterministic for rollback netcode)
//! - `render()` — Called every frame (skipped during rollback replay)

#![allow(unused)]

mod animation;
mod assets;
mod audio;
mod camera;
mod colors;
mod constants;
mod debug;
mod drawing;
mod embedded;
mod epu;
mod helpers;
mod input;
mod lighting;
mod material;
mod mesh;
mod music;
mod pass;
mod procedural;
mod render;
mod skeleton;
mod stubs;
mod system;
mod text;
mod texture;
mod transform;
mod viewport;

pub use animation::*;
pub use assets::*;
pub use audio::*;
pub use camera::*;
pub use colors::*;
pub use constants::*;
pub use debug::*;
pub use drawing::*;
pub use embedded::*;
pub use epu::*;
pub use helpers::*;
pub use input::*;
pub use lighting::*;
pub use material::*;
pub use mesh::*;
pub use music::*;
pub use pass::*;
pub use procedural::*;
pub use render::*;
pub use skeleton::*;
pub use stubs::*;
pub use system::*;
pub use text::*;
pub use texture::*;
pub use transform::*;
pub use viewport::*;
