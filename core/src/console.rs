//! Console trait and associated types
//!
//! Each fantasy console (Nethercore ZX, Chroma, etc.) implements the `Console` trait
//! to define its specific graphics, audio, input, and FFI functions.

use std::sync::Arc;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wasmtime::Linker;
use winit::window::Window;

use crate::debug::DebugStat;
use crate::wasm::WasmGameContext;

// Re-export ConsoleSpecs from shared crate for convenience
pub use nethercore_shared::ConsoleSpecs;

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

/// Trait for console-specific audio generation
///
/// Consoles that support audio implement this trait to generate audio
/// samples from their rollback state. The audio generation happens once
/// per confirmed game frame (not during rollback).
///
/// Use `()` for consoles with no audio generation.
pub trait AudioGenerator: Send + 'static {
    /// Console rollback state type
    type RollbackState: ConsoleRollbackState;
    /// Console FFI staging state type
    type State: Default + Send + 'static;
    /// Console-specific audio backend type
    type Audio: Audio;

    /// Get the default sample rate for this console's audio output
    fn default_sample_rate() -> u32 {
        44_100 // Default to CD quality
    }

    /// Generate one frame of audio samples (synchronous mode)
    ///
    /// Called once per confirmed game frame (not during rollback).
    /// Output should be interleaved stereo samples (left, right, left, right...).
    ///
    /// Note: Prefer using `process_audio` which handles both sync and threaded modes.
    ///
    /// # Arguments
    /// * `rollback_state` - Mutable reference to console rollback state (e.g., audio playhead positions)
    /// * `state` - Mutable reference to console FFI state (e.g., loaded sounds, tracker engine)
    /// * `tick_rate` - Game tick rate (e.g., 60 for 60fps)
    /// * `sample_rate` - Output sample rate (e.g., 44100)
    /// * `output` - Buffer to append stereo samples to
    fn generate_frame(
        rollback_state: &mut Self::RollbackState,
        state: &mut Self::State,
        tick_rate: u32,
        sample_rate: u32,
        output: &mut Vec<f32>,
    );

    /// Process audio for this frame, handling both sync and threaded modes
    ///
    /// This is the main entry point for audio processing from the game loop.
    /// It automatically handles both synchronous and threaded audio modes:
    /// - Sync mode: generates samples and pushes them to the audio buffer
    /// - Threaded mode: creates a snapshot and sends it to the audio thread
    fn process_audio(
        rollback_state: &mut Self::RollbackState,
        state: &mut Self::State,
        audio: &mut Self::Audio,
        tick_rate: u32,
        sample_rate: u32,
    );
}

/// No-op audio backend for consoles without audio
pub struct NullAudio;

impl Audio for NullAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {}
    fn stop(&mut self, _handle: SoundHandle) {}
}

/// No-op audio generator for consoles without audio
impl AudioGenerator for () {
    type RollbackState = ();
    type State = ();
    type Audio = NullAudio;

    fn generate_frame(
        _rollback_state: &mut Self::RollbackState,
        _state: &mut Self::State,
        _tick_rate: u32,
        _sample_rate: u32,
        _output: &mut Vec<f32>,
    ) {
        // No-op: output remains empty
    }

    fn process_audio(
        _rollback_state: &mut Self::RollbackState,
        _state: &mut Self::State,
        _audio: &mut Self::Audio,
        _tick_rate: u32,
        _sample_rate: u32,
    ) {
        // No-op: no audio to process
    }
}

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
    /// For example, Nethercore ZX uses ZXFFIState which holds draw commands, camera, transforms, etc.
    type State: Default + Send + 'static;
    /// Console-specific rollback state (host-side, rolled back with WASM memory)
    ///
    /// This state lives on the host side but is included in rollback snapshots.
    /// Examples: audio playhead positions, channel volumes.
    /// Use `()` for consoles with no host-side rollback state.
    type RollbackState: ConsoleRollbackState;
    /// Console-specific resource manager type
    type ResourceManager: ConsoleResourceManager<Graphics = Self::Graphics, State = Self::State>;
    /// Audio generator type for per-frame audio sample generation
    ///
    /// Use `()` for consoles without audio generation.
    type AudioGenerator: AudioGenerator<
            RollbackState = Self::RollbackState,
            State = Self::State,
            Audio = Self::Audio,
        >;

    /// Get console specifications
    fn specs() -> &'static ConsoleSpecs;

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

    /// Decode replay script raw bytes into console input.
    ///
    /// Used by the graphical replay runner to convert script-encoded
    /// input bytes into the console's native input type.
    fn decode_replay_bytes(&self, bytes: &[u8]) -> Self::Input;

    /// Get the input layout for replay script compilation.
    /// Returns None if the console doesn't support replay scripts.
    fn replay_input_layout(&self) -> Option<Box<dyn crate::replay::script::InputLayout>> {
        None
    }

    /// Create a resource manager instance for this console
    ///
    /// Resource managers handle the mapping between game resource handles (u32)
    /// and graphics backend handles (console-specific types).
    fn create_resource_manager(&self) -> Self::ResourceManager;

    /// Get console-specific debug statistics
    ///
    /// These are read-only values displayed in the debug overlay showing
    /// the current state of console subsystems (e.g., draw call count,
    /// vertex count, texture memory usage).
    ///
    /// Default implementation returns an empty list.
    fn debug_stats(&self, _state: &Self::State) -> Vec<DebugStat> {
        Vec::new()
    }

    /// Initialize console-specific FFI state before game init() is called
    ///
    /// This is called after the game instance is created but before the
    /// game's init() function runs. Use this to set up console-specific
    /// state that the game needs during initialization (e.g., datapack).
    ///
    /// Default implementation does nothing.
    fn initialize_ffi_state(&self, _state: &mut Self::State) {}

    /// Unpack a console-specific clear color to normalized RGBA
    ///
    /// Consoles may store clear colors in different formats (e.g., packed u32).
    /// This method converts to normalized [0.0, 1.0] RGBA for wgpu.
    ///
    /// Default implementation returns dark gray.
    fn unpack_clear_color(_color: u32) -> [f32; 4] {
        [0.1, 0.1, 0.1, 1.0]
    }

    /// Get the clear color from console state.
    ///
    /// Extracts and unpacks the clear color stored in the console's FFI state.
    /// Default implementation returns dark gray.
    fn clear_color_from_state(_state: &Self::State) -> [f32; 4] {
        [0.1, 0.1, 0.1, 1.0]
    }

    /// Clear per-frame state before rendering.
    ///
    /// Called at the start of each rendered frame to reset per-frame state
    /// like draw commands. Default implementation does nothing.
    fn clear_frame_state(_state: &mut Self::State) {}
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

    /// Render game content to the render target.
    ///
    /// Called once per rendered frame to draw the game's graphics.
    /// The implementation handles console-specific rendering (draw commands,
    /// camera, lighting, etc.)
    fn render_game_to_target(
        &self,
        graphics: &mut Self::Graphics,
        encoder: &mut wgpu::CommandEncoder,
        state: &Self::State,
        clear_color: [f32; 4],
    );
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

    /// Set the master volume (0.0 - 1.0)
    fn set_master_volume(&mut self, _volume: f32) {}

    /// Get the output sample rate
    fn sample_rate(&self) -> u32 {
        44_100 // Default to CD quality
    }

    /// Push audio samples to the output buffer.
    ///
    /// Samples should be interleaved stereo (left, right, left, right, ...).
    fn push_samples(&mut self, _samples: &[f32]) {}
}

/// Handle to a loaded sound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);
