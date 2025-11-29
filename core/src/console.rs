//! Console trait and associated types
//!
//! Each fantasy console (Emberware Z, Classic, etc.) implements the `Console` trait
//! to define its specific graphics, audio, input, and FFI functions.

use std::sync::Arc;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wasmtime::Linker;
use winit::window::Window;

use crate::wasm::GameState;

/// Specifications for a fantasy console
#[derive(Debug, Clone)]
pub struct ConsoleSpecs {
    /// Console name (e.g., "Emberware Z")
    pub name: &'static str,
    /// Available resolutions (width, height)
    pub resolutions: &'static [(u32, u32)],
    /// Default resolution index
    pub default_resolution: usize,
    /// Available tick rates
    pub tick_rates: &'static [u32],
    /// Default tick rate index
    pub default_tick_rate: usize,
    /// Maximum RAM in bytes
    pub ram_limit: usize,
    /// Maximum VRAM in bytes
    pub vram_limit: usize,
    /// Maximum ROM size in bytes
    pub rom_limit: usize,
    /// CPU budget per tick in microseconds
    pub cpu_budget_us: u64,
}

/// Trait for fantasy console implementations
///
/// Each console defines its own graphics backend, audio backend, and input layout.
/// The core runtime is generic over this trait, allowing shared WASM execution
/// and rollback netcode across all consoles.
pub trait Console: Send + 'static {
    /// Graphics backend type
    type Graphics: Graphics;
    /// Audio backend type
    type Audio: Audio;
    /// Console-specific input type (must be POD for GGRS serialization)
    type Input: ConsoleInput;

    /// Get console name
    fn name(&self) -> &'static str;

    /// Get console specifications
    fn specs(&self) -> &ConsoleSpecs;

    /// Register console-specific FFI functions with the WASM linker
    fn register_ffi(&self, linker: &mut Linker<GameState>) -> Result<()>;

    /// Create the graphics backend for this console
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;

    /// Create the audio backend for this console
    fn create_audio(&self) -> Result<Self::Audio>;

    /// Map raw input to console-specific input format
    fn map_input(&self, raw: &RawInput) -> Self::Input;
}

/// Trait for console input types
///
/// Must be POD (Plain Old Data) for efficient serialization over the network
/// and for GGRS rollback state management.
pub trait ConsoleInput:
    Clone + Copy + Default + PartialEq + Pod + Zeroable + Send + Sync + 'static
{
}

/// Raw input from physical devices
///
/// This represents the union of all possible inputs across all supported
/// input devices (keyboard, gamepads, etc.). Each console maps this to
/// its own input format.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawInput {
    /// D-pad / WASD / left stick digital
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,

    /// Face buttons
    pub button_a: bool,
    pub button_b: bool,
    pub button_x: bool,
    pub button_y: bool,

    /// Shoulder buttons
    pub left_bumper: bool,
    pub right_bumper: bool,

    /// Stick buttons
    pub left_stick_button: bool,
    pub right_stick_button: bool,

    /// Start/Select
    pub start: bool,
    pub select: bool,

    /// Analog sticks (-1.0 to 1.0)
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,

    /// Analog triggers (0.0 to 1.0)
    pub left_trigger: f32,
    pub right_trigger: f32,
}

/// Trait for graphics backends
pub trait Graphics: Send {
    /// Handle window resize
    fn resize(&mut self, width: u32, height: u32);

    /// Begin a new frame
    fn begin_frame(&mut self);

    /// End the current frame and present
    fn end_frame(&mut self);
}

/// Trait for audio backends
pub trait Audio: Send {
    /// Play a sound
    fn play(&mut self, handle: SoundHandle, volume: f32, looping: bool);

    /// Stop a sound
    fn stop(&mut self, handle: SoundHandle);

    /// Set rollback mode (mutes audio during rollback replay)
    fn set_rollback_mode(&mut self, rolling_back: bool);
}

/// Handle to a loaded sound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);
