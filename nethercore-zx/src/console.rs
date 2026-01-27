//! Nethercore ZX console implementation
//!
//! Implements the `Console` trait for the PS1/N64 aesthetic fantasy console.

use std::sync::Arc;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wasmtime::Linker;
use winit::window::Window;

use nethercore_core::{
    console::{Audio, Console, ConsoleInput, ConsoleSpecs, RawInput, SoundHandle},
    debug::DebugStat,
    wasm::WasmGameContext,
};
use zx_common::ZXDataPack;

use crate::state::{ZRollbackState, ZXFFIState};

use crate::graphics::ZXGraphics;

/// Get Nethercore ZX console specifications
pub const fn zx_specs() -> &'static ConsoleSpecs {
    nethercore_shared::nethercore_zx_specs()
}

// Re-export constants for FFI validation
pub use nethercore_shared::{
    NETHERCORE_ZX_RESOLUTION as RESOLUTION, NETHERCORE_ZX_TICK_RATES as TICK_RATES,
    NETHERCORE_ZX_VRAM_LIMIT as VRAM_LIMIT,
};

/// Maximum value for analog stick conversion (i8 range: -128 to 127)
pub const STICK_SCALE: f32 = 127.0;

/// Maximum value for trigger conversion (u8 range: 0 to 255)
pub const TRIGGER_SCALE: f32 = 255.0;

/// Maximum valid button index (0-13, corresponding to Button enum)
pub const MAX_BUTTON_INDEX: u32 = 13;

/// Button indices for ZInput
///
/// Used by tests and available for console-side code that works with ZInput.
/// WASM games use button indices (0-13) via FFI rather than this enum.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    X = 6,
    Y = 7,
    LeftBumper = 8,
    RightBumper = 9,
    LeftStick = 10,
    RightStick = 11,
    Start = 12,
    Select = 13,
}

impl Button {
    /// Get the bitmask for this button
    #[inline]
    pub fn mask(self) -> u16 {
        1 << (self as u8)
    }
}

/// Nethercore ZX input state (PS2/Xbox style with dual analog sticks and triggers)
///
/// This struct is POD (Plain Old Data) for efficient serialization over the network
/// and for GGRS rollback state management.
#[repr(C)]
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
pub struct ZInput {
    /// Button bitmask: D-pad + A/B/X/Y + L/R bumpers + L3/R3 + Start/Select
    /// Bit layout: UP(0), DOWN(1), LEFT(2), RIGHT(3), A(4), B(5), X(6), Y(7),
    ///             LB(8), RB(9), L3(10), R3(11), START(12), SELECT(13)
    pub buttons: u16,
    /// Left stick X axis (-128 to 127, mapped to -1.0 to 1.0)
    pub left_stick_x: i8,
    /// Left stick Y axis (-128 to 127, mapped to -1.0 to 1.0)
    pub left_stick_y: i8,
    /// Right stick X axis (-128 to 127, mapped to -1.0 to 1.0)
    pub right_stick_x: i8,
    /// Right stick Y axis (-128 to 127, mapped to -1.0 to 1.0)
    pub right_stick_y: i8,
    /// Left trigger (0 to 255, mapped to 0.0 to 1.0)
    pub left_trigger: u8,
    /// Right trigger (0 to 255, mapped to 0.0 to 1.0)
    pub right_trigger: u8,
}

// Public API helpers for ZInput - used by tests and available for console-side code.
// WASM games access input via FFI, not these Rust methods directly.
impl ZInput {
    /// Check if a button is held
    #[inline]
    pub fn button_held(&self, button: Button) -> bool {
        (self.buttons & button.mask()) != 0
    }

    /// Get left stick X as float (-1.0 to 1.0)
    #[inline]
    pub fn left_stick_x_f32(&self) -> f32 {
        self.left_stick_x as f32 / STICK_SCALE
    }

    /// Get left stick Y as float (-1.0 to 1.0)
    #[inline]
    pub fn left_stick_y_f32(&self) -> f32 {
        self.left_stick_y as f32 / STICK_SCALE
    }

    /// Get right stick X as float (-1.0 to 1.0)
    #[inline]
    pub fn right_stick_x_f32(&self) -> f32 {
        self.right_stick_x as f32 / STICK_SCALE
    }

    /// Get right stick Y as float (-1.0 to 1.0)
    #[inline]
    pub fn right_stick_y_f32(&self) -> f32 {
        self.right_stick_y as f32 / STICK_SCALE
    }

    /// Get left trigger as float (0.0 to 1.0)
    #[inline]
    pub fn left_trigger_f32(&self) -> f32 {
        self.left_trigger as f32 / TRIGGER_SCALE
    }

    /// Get right trigger as float (0.0 to 1.0)
    #[inline]
    pub fn right_trigger_f32(&self) -> f32 {
        self.right_trigger as f32 / TRIGGER_SCALE
    }
}

impl ConsoleInput for ZInput {}

// ZXGraphics is implemented in graphics.rs.

/// Re-export ZXAudio from audio module
pub use crate::audio::ZXAudio;

impl Audio for ZXAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {
        // Legacy Audio trait - not used in ZX console
        // Audio is handled via per-frame generation from ZRollbackState
    }

    fn stop(&mut self, _handle: SoundHandle) {
        // Legacy Audio trait - not used in ZX console
    }

    fn set_master_volume(&mut self, volume: f32) {
        ZXAudio::set_master_volume(self, volume);
    }

    fn sample_rate(&self) -> u32 {
        ZXAudio::sample_rate(self)
    }

    fn push_samples(&mut self, samples: &[f32]) {
        ZXAudio::push_samples(self, samples);
    }
}

/// Nethercore ZX fantasy console
///
/// Implements the PS1/N64 aesthetic with:
/// - wgpu-based 3D graphics with vertex jitter, affine texture mapping
/// - Dual analog sticks and analog triggers
/// - Deterministic rollback netcode via GGRS
#[derive(Clone)]
pub struct NethercoreZX {
    /// Optional datapack for ROM assets (textures, meshes, sounds)
    data_pack: Option<Arc<ZXDataPack>>,
    /// Default render mode for this ROM (0-3).
    ///
    /// Stored on the host side and applied to `ZXFFIState.init_config` before `init()` runs.
    render_mode: u8,
}

impl NethercoreZX {
    /// Create a new Nethercore ZX console instance
    pub fn new() -> Self {
        Self {
            data_pack: None,
            render_mode: 0,
        }
    }

    /// Create a new Nethercore ZX console instance with a datapack
    pub fn with_datapack(data_pack: Option<Arc<ZXDataPack>>) -> Self {
        Self {
            data_pack,
            render_mode: 0,
        }
    }

    /// Create a new Nethercore ZX console instance with a datapack and default render mode.
    pub fn with_datapack_and_render_mode(
        data_pack: Option<Arc<ZXDataPack>>,
        render_mode: u8,
    ) -> Self {
        Self {
            data_pack,
            render_mode: render_mode.min(3),
        }
    }
}

impl Default for NethercoreZX {
    fn default() -> Self {
        Self::new()
    }
}

impl Console for NethercoreZX {
    type Graphics = ZXGraphics;
    type Audio = ZXAudio;
    type Input = ZInput;
    type State = ZXFFIState;
    type RollbackState = ZRollbackState;
    type ResourceManager = crate::resource_manager::ZResourceManager;
    type AudioGenerator = crate::audio::ZXAudioGenerator;

    fn specs() -> &'static ConsoleSpecs {
        zx_specs()
    }

    fn register_ffi(
        &self,
        linker: &mut Linker<WasmGameContext<ZInput, ZXFFIState, ZRollbackState>>,
    ) -> Result<()> {
        // Register all ZX-specific FFI functions (graphics, input, transforms, camera, etc.)
        crate::ffi::register_zx_ffi(linker)?;
        Ok(())
    }

    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics> {
        ZXGraphics::new_blocking(window)
    }

    fn create_audio(&self) -> Result<Self::Audio> {
        // Use threaded audio by default for better resilience during load/rollbacks
        // Falls back to synchronous if thread spawning fails
        match ZXAudio::new_threaded() {
            Ok(audio) => Ok(audio),
            Err(e) => {
                tracing::warn!("Threaded audio failed, falling back to sync: {}", e);
                ZXAudio::new().map_err(|e| anyhow::anyhow!("Failed to create audio: {}", e))
            }
        }
    }

    fn map_input(&self, raw: &RawInput) -> Self::Input {
        let mut buttons = 0u16;

        // Map D-pad
        if raw.dpad_up {
            buttons |= Button::Up.mask();
        }
        if raw.dpad_down {
            buttons |= Button::Down.mask();
        }
        if raw.dpad_left {
            buttons |= Button::Left.mask();
        }
        if raw.dpad_right {
            buttons |= Button::Right.mask();
        }

        // Map face buttons
        if raw.button_a {
            buttons |= Button::A.mask();
        }
        if raw.button_b {
            buttons |= Button::B.mask();
        }
        if raw.button_x {
            buttons |= Button::X.mask();
        }
        if raw.button_y {
            buttons |= Button::Y.mask();
        }

        // Map shoulder buttons
        if raw.left_bumper {
            buttons |= Button::LeftBumper.mask();
        }
        if raw.right_bumper {
            buttons |= Button::RightBumper.mask();
        }

        // Map stick buttons
        if raw.left_stick_button {
            buttons |= Button::LeftStick.mask();
        }
        if raw.right_stick_button {
            buttons |= Button::RightStick.mask();
        }

        // Map start/select
        if raw.start {
            buttons |= Button::Start.mask();
        }
        if raw.select {
            buttons |= Button::Select.mask();
        }

        // Map analog sticks (f32 -1.0..1.0 to i8 -128..127)
        let left_stick_x = (raw.left_stick_x.clamp(-1.0, 1.0) * STICK_SCALE) as i8;
        let left_stick_y = (raw.left_stick_y.clamp(-1.0, 1.0) * STICK_SCALE) as i8;
        let right_stick_x = (raw.right_stick_x.clamp(-1.0, 1.0) * STICK_SCALE) as i8;
        let right_stick_y = (raw.right_stick_y.clamp(-1.0, 1.0) * STICK_SCALE) as i8;

        // Map triggers (f32 0.0..1.0 to u8 0..255)
        let left_trigger = (raw.left_trigger.clamp(0.0, 1.0) * TRIGGER_SCALE) as u8;
        let right_trigger = (raw.right_trigger.clamp(0.0, 1.0) * TRIGGER_SCALE) as u8;

        ZInput {
            buttons,
            left_stick_x,
            left_stick_y,
            right_stick_x,
            right_stick_y,
            left_trigger,
            right_trigger,
        }
    }

    fn decode_replay_bytes(&self, bytes: &[u8]) -> Self::Input {
        crate::replay::ZxInputLayout::bytes_to_zinput(bytes)
    }

    fn replay_input_layout(&self) -> Option<Box<dyn nethercore_core::replay::script::InputLayout>> {
        Some(Box::new(crate::replay::ZxInputLayout))
    }

    fn create_resource_manager(&self) -> Self::ResourceManager {
        crate::resource_manager::ZResourceManager::new()
    }

    fn debug_stats(&self, state: &ZXFFIState) -> Vec<DebugStat> {
        vec![
            DebugStat::number("Draw Calls", state.render_pass.commands().len()),
            DebugStat::number("Textures", state.next_texture_handle.saturating_sub(1)),
            DebugStat::number("Meshes", state.next_mesh_handle.saturating_sub(1)),
            DebugStat::number("Skeletons", state.next_skeleton_handle.saturating_sub(1)),
            DebugStat::number("Keyframes", state.next_keyframe_handle.saturating_sub(1)),
            DebugStat::number("Fonts", state.next_font_handle.saturating_sub(1)),
            DebugStat::number("MVP States", state.mvp_shading_states.len()),
            DebugStat::number("Shading States", state.shading_pool.len()),
        ]
    }

    fn initialize_ffi_state(&self, state: &mut ZXFFIState) {
        // Set datapack for rom_* FFI functions
        state.data_pack = self.data_pack.clone();

        // Set default render mode before init() runs.
        state.init_config.render_mode = self.render_mode;
    }

    fn clear_color_from_state(state: &Self::State) -> [f32; 4] {
        crate::ffi::unpack_rgba(state.init_config.clear_color)
    }

    fn clear_frame_state(state: &mut Self::State) {
        state.clear_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_mask() {
        assert_eq!(Button::Up.mask(), 0x0001);
        assert_eq!(Button::Down.mask(), 0x0002);
        assert_eq!(Button::Left.mask(), 0x0004);
        assert_eq!(Button::Right.mask(), 0x0008);
        assert_eq!(Button::A.mask(), 0x0010);
        assert_eq!(Button::Start.mask(), 0x1000);
        assert_eq!(Button::Select.mask(), 0x2000);
    }

    #[test]
    fn test_button_held() {
        let input = ZInput {
            buttons: Button::A.mask() | Button::Start.mask(),
            ..Default::default()
        };
        assert!(input.button_held(Button::A));
        assert!(input.button_held(Button::Start));
        assert!(!input.button_held(Button::B));
        assert!(!input.button_held(Button::Up));
    }

    #[test]
    fn test_analog_conversion() {
        let input = ZInput {
            left_stick_x: 127,
            left_stick_y: -128,
            left_trigger: 255,
            right_trigger: 0,
            ..Default::default()
        };

        // Stick values
        assert!((input.left_stick_x_f32() - 1.0).abs() < 0.01);
        assert!((input.left_stick_y_f32() - (-1.008)).abs() < 0.01);

        // Trigger values
        assert!((input.left_trigger_f32() - 1.0).abs() < 0.01);
        assert!(input.right_trigger_f32().abs() < 0.01);
    }

    #[test]
    fn test_map_input() {
        let console = NethercoreZX::new();
        let raw = RawInput {
            dpad_up: true,
            button_a: true,
            left_stick_x: 0.5,
            left_trigger: 0.75,
            ..Default::default()
        };

        let mapped = console.map_input(&raw);
        assert!(mapped.button_held(Button::Up));
        assert!(mapped.button_held(Button::A));
        assert!(!mapped.button_held(Button::Down));
        assert_eq!(mapped.left_stick_x, 63); // 0.5 * 127 ≈ 63
        assert_eq!(mapped.left_trigger, 191); // 0.75 * 255 ≈ 191
    }

    #[test]
    fn test_specs() {
        let _console = NethercoreZX::new();
        let specs = NethercoreZX::specs();

        assert_eq!(specs.name, "Nethercore ZX");
        assert_eq!(specs.resolution, (960, 540)); // Fixed 540p
        assert_eq!(specs.tick_rates[specs.default_tick_rate], 60);
        assert_eq!(specs.ram_limit, 4 * 1024 * 1024);
        assert_eq!(specs.vram_limit, 4 * 1024 * 1024);
        assert_eq!(specs.rom_limit, 16 * 1024 * 1024);
        assert_eq!(specs.cpu_budget_us, 4000);
    }
}
