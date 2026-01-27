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

mod stubs;
mod system;
mod input;
mod camera;
mod transform;
mod render;
mod viewport;
mod pass;
mod texture;
mod mesh;
mod procedural;
mod drawing;
mod text;
mod epu;
mod material;
mod lighting;
mod skeleton;
mod animation;
mod audio;
mod music;
mod assets;
mod embedded;
mod debug;
mod constants;
mod helpers;
mod colors;

pub use stubs::*;
pub use system::*;
pub use input::*;
pub use camera::*;
pub use transform::*;
pub use render::*;
pub use viewport::*;
pub use pass::*;
pub use texture::*;
pub use mesh::*;
pub use procedural::*;
pub use drawing::*;
pub use text::*;
pub use epu::*;
pub use material::*;
pub use lighting::*;
pub use skeleton::*;
pub use animation::*;
pub use audio::*;
pub use music::*;
pub use assets::*;
pub use embedded::*;
pub use debug::*;
pub use constants::*;
pub use helpers::*;
pub use colors::*;
