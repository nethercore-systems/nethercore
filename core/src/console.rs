//! Console trait and associated types
//!
//! Each fantasy console (Emberware Z, Classic, etc.) implements the `Console` trait
//! to define its specific graphics, audio, input, and FFI functions.

use std::sync::Arc;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wasmtime::Linker;
use winit::window::Window;

use crate::wasm::WasmGameContext;

// Re-export ConsoleSpecs from shared crate for convenience
pub use emberware_shared::ConsoleSpecs;

/// Console-specific rollback state (host-side, POD for zero-copy serialization)
///
/// This trait represents state that lives on the host side (not in WASM memory)
/// but still needs to be rolled back during netcode rollback. Examples include:
/// - Audio playhead positions
/// - Channel volumes and pan values
///
/// The state must be POD (Plain Old Data) so it can be serialized/deserialized
/// with zero-copy using bytemuck.
pub trait ConsoleRollbackState: Pod + Zeroable + Default + Send + 'static {}

// Unit type implementation for consoles with no rollback state
impl ConsoleRollbackState for () {}

/// Trait for fantasy console implementations
///
/// Each console defines its own graphics backend, audio backend, input layout,
/// and console-specific FFI staging state.
/// The core runtime is generic over this trait, allowing shared WASM execution
/// and rollback netcode across all consoles.
pub trait Console: Send + 'static {
    /// Graphics backend type
    type Graphics: Graphics;
    /// Audio backend type
    type Audio: Audio;
    /// Console-specific input type (must be POD for GGRS serialization)
    type Input: ConsoleInput;
    /// Console-specific FFI staging state
    ///
    /// This state is written to by FFI functions and consumed by Graphics/Audio backends.
    /// It is rebuilt each frame and is NOT part of rollback state (only GameState is rolled back).
    /// For example, Emberware Z uses ZFFIState which holds draw commands, camera, transforms, etc.
    type State: Default + Send + 'static;
    /// Console-specific rollback state (host-side, rolled back with WASM memory)
    ///
    /// This state lives on the host side but is included in rollback snapshots.
    /// Examples: audio playhead positions, channel volumes.
    /// Use `()` for consoles with no host-side rollback state.
    type RollbackState: ConsoleRollbackState;
    /// Console-specific resource manager type
    type ResourceManager: ConsoleResourceManager<Graphics = Self::Graphics, State = Self::State>;

    /// Get console specifications
    fn specs(&self) -> &'static ConsoleSpecs;

    /// Register console-specific FFI functions with the WASM linker
    fn register_ffi(
        &self,
        linker: &mut Linker<WasmGameContext<Self::Input, Self::State, Self::RollbackState>>,
    ) -> Result<()>;

    /// Create the graphics backend for this console
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;

    /// Create the audio backend for this console
    fn create_audio(&self) -> Result<Self::Audio>;

    /// Map raw input to console-specific input format
    fn map_input(&self, raw: &RawInput) -> Self::Input;

    /// Create a resource manager instance for this console
    ///
    /// Resource managers handle the mapping between game resource handles (u32)
    /// and graphics backend handles (console-specific types).
    fn create_resource_manager(&self) -> Self::ResourceManager;

    /// Get the window title for this console
    fn window_title(&self) -> &'static str;
}

/// Trait for console-specific resource management
///
/// This abstraction handles the mapping between game resource handles
/// (u32 IDs) and graphics backend resource handles (console-specific types).
/// Each console implements this to manage textures, meshes, and other resources.
pub trait ConsoleResourceManager: Send + 'static {
    /// Graphics backend type this manager works with
    type Graphics: Graphics;

    /// Console state type (FFI staging state)
    type State: Default + Send + 'static;

    /// Process pending texture/mesh/audio resources from game state
    ///
    /// Called once after game.init() to upload all resources requested
    /// during the initialization phase. Should not be called during the
    /// game loop (resources are init-only).
    fn process_pending_resources(
        &mut self,
        graphics: &mut Self::Graphics,
        audio: &mut dyn Audio,
        state: &mut Self::State,
    );

    /// Execute accumulated draw commands
    ///
    /// Called after game.render() to execute all draw commands that
    /// were recorded during that frame.
    fn execute_draw_commands(&mut self, graphics: &mut Self::Graphics, state: &mut Self::State);
}

/// Trait for console input types
///
/// Must be POD (Plain Old Data) for efficient serialization over the network
/// and for GGRS rollback state management.
pub trait ConsoleInput:
    Clone
    + Copy
    + Default
    + PartialEq
    + Pod
    + Zeroable
    + serde::Serialize
    + serde::de::DeserializeOwned
    + Send
    + Sync
    + 'static
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

    /// Set bone matrices for GPU skinning (up to 256 bones)
    ///
    /// Call this before rendering skinned meshes. The matrices are in column-major order.
    /// An empty slice clears the bone data.
    fn set_bones(&mut self, _bones: &[Mat4]) {
        // Default implementation does nothing (for consoles without GPU skinning)
    }
}

/// Trait for audio backends
pub trait Audio: Send {
    /// Play a sound
    fn play(&mut self, handle: SoundHandle, volume: f32, looping: bool);

    /// Stop a sound
    fn stop(&mut self, handle: SoundHandle);
}

/// Handle to a loaded sound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);
