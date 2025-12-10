//! Code generation for ember-export
//!
//! Generates Rust source files that load exported assets.

pub mod rust;

pub use rust::generate_rust_module;
