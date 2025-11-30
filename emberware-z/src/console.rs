//! Emberware Z console implementation
//!
//! Implements the `Console` trait for the PS1/N64 aesthetic fantasy console.

use std::sync::Arc;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wasmtime::Linker;
use winit::window::Window;

use emberware_core::{
    console::{Audio, Console, ConsoleInput, ConsoleSpecs, RawInput, SoundHandle},
    wasm::{GameState, InputState},
};

use crate::graphics::ZGraphics;

/// Emberware Z resolutions (16:9)
pub const RESOLUTIONS: &[(u32, u32)] = &[
    (640, 360),   // 360p
    (960, 540),   // 540p (default)
    (1280, 720),  // 720p
    (1920, 1080), // 1080p
];

/// Available tick rates
pub const TICK_RATES: &[u32] = &[24, 30, 60, 120];

/// Default resolution index (540p)
pub const DEFAULT_RESOLUTION: usize = 1;

/// Default tick rate index (60 fps)
pub const DEFAULT_TICK_RATE: usize = 2;

/// RAM limit (16 MB)
pub const RAM_LIMIT: usize = 16 * 1024 * 1024;

/// VRAM limit (8 MB)
pub const VRAM_LIMIT: usize = 8 * 1024 * 1024;

/// ROM size limit (32 MB)
pub const ROM_LIMIT: usize = 32 * 1024 * 1024;

/// CPU budget per tick at 60fps (4ms = 4000 microseconds)
pub const CPU_BUDGET_US: u64 = 4000;

/// Emberware Z console specifications
pub static Z_SPECS: ConsoleSpecs = ConsoleSpecs {
    name: "Emberware Z",
    resolutions: RESOLUTIONS,
    default_resolution: DEFAULT_RESOLUTION,
    tick_rates: TICK_RATES,
    default_tick_rate: DEFAULT_TICK_RATE,
    ram_limit: RAM_LIMIT,
    vram_limit: VRAM_LIMIT,
    rom_limit: ROM_LIMIT,
    cpu_budget_us: CPU_BUDGET_US,
};

/// Button indices for ZInput
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

/// Emberware Z input state (PS2/Xbox style with dual analog sticks and triggers)
///
/// This struct is POD (Plain Old Data) for efficient serialization over the network
/// and for GGRS rollback state management.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable)]
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

impl ZInput {
    /// Check if a button is held
    #[inline]
    pub fn button_held(&self, button: Button) -> bool {
        (self.buttons & button.mask()) != 0
    }

    /// Get left stick X as float (-1.0 to 1.0)
    #[inline]
    pub fn left_stick_x_f32(&self) -> f32 {
        self.left_stick_x as f32 / 127.0
    }

    /// Get left stick Y as float (-1.0 to 1.0)
    #[inline]
    pub fn left_stick_y_f32(&self) -> f32 {
        self.left_stick_y as f32 / 127.0
    }

    /// Get right stick X as float (-1.0 to 1.0)
    #[inline]
    pub fn right_stick_x_f32(&self) -> f32 {
        self.right_stick_x as f32 / 127.0
    }

    /// Get right stick Y as float (-1.0 to 1.0)
    #[inline]
    pub fn right_stick_y_f32(&self) -> f32 {
        self.right_stick_y as f32 / 127.0
    }

    /// Get left trigger as float (0.0 to 1.0)
    #[inline]
    pub fn left_trigger_f32(&self) -> f32 {
        self.left_trigger as f32 / 255.0
    }

    /// Get right trigger as float (0.0 to 1.0)
    #[inline]
    pub fn right_trigger_f32(&self) -> f32 {
        self.right_trigger as f32 / 255.0
    }
}

impl ConsoleInput for ZInput {
    fn to_input_state(&self) -> InputState {
        InputState {
            buttons: self.buttons,
            left_stick_x: self.left_stick_x,
            left_stick_y: self.left_stick_y,
            right_stick_x: self.right_stick_x,
            right_stick_y: self.right_stick_y,
            left_trigger: self.left_trigger,
            right_trigger: self.right_trigger,
        }
    }
}

// ZGraphics is now implemented in graphics.rs

/// Emberware Z audio backend (placeholder until rodio implementation)
pub struct ZAudio {
    /// Whether audio is muted during rollback
    rollback_mode: bool,
}

impl Audio for ZAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {
        if self.rollback_mode {
            return; // Don't play audio during rollback
        }
        // TODO: Play sound via rodio
    }

    fn stop(&mut self, _handle: SoundHandle) {
        // TODO: Stop sound via rodio
    }

    fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }
}

/// Emberware Z fantasy console
///
/// Implements the PS1/N64 aesthetic with:
/// - wgpu-based 3D graphics with vertex jitter, affine texture mapping
/// - Dual analog sticks and analog triggers
/// - Deterministic rollback netcode via GGRS
pub struct EmberwareZ {
    // Configuration could go here
}

impl EmberwareZ {
    /// Create a new Emberware Z console instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EmberwareZ {
    fn default() -> Self {
        Self::new()
    }
}

impl Console for EmberwareZ {
    type Graphics = ZGraphics;
    type Audio = ZAudio;
    type Input = ZInput;

    fn name(&self) -> &'static str {
        "Emberware Z"
    }

    fn specs(&self) -> &ConsoleSpecs {
        &Z_SPECS
    }

    fn register_ffi(&self, linker: &mut Linker<GameState>) -> Result<()> {
        // Register all Z-specific FFI functions (graphics, input, transforms, camera, etc.)
        crate::ffi::register_z_ffi(linker)?;
        Ok(())
    }

    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics> {
        ZGraphics::new_blocking(window)
    }

    fn create_audio(&self) -> Result<Self::Audio> {
        // TODO: Initialize rodio output stream
        Ok(ZAudio {
            rollback_mode: false,
        })
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
        let left_stick_x = (raw.left_stick_x.clamp(-1.0, 1.0) * 127.0) as i8;
        let left_stick_y = (raw.left_stick_y.clamp(-1.0, 1.0) * 127.0) as i8;
        let right_stick_x = (raw.right_stick_x.clamp(-1.0, 1.0) * 127.0) as i8;
        let right_stick_y = (raw.right_stick_y.clamp(-1.0, 1.0) * 127.0) as i8;

        // Map triggers (f32 0.0..1.0 to u8 0..255)
        let left_trigger = (raw.left_trigger.clamp(0.0, 1.0) * 255.0) as u8;
        let right_trigger = (raw.right_trigger.clamp(0.0, 1.0) * 255.0) as u8;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zinput_size() {
        // ZInput should be 8 bytes for efficient network serialization
        assert_eq!(std::mem::size_of::<ZInput>(), 8);
    }

    #[test]
    fn test_zinput_pod() {
        // Verify ZInput is POD (bytemuck requirements)
        let input = ZInput::default();
        let bytes = bytemuck::bytes_of(&input);
        assert_eq!(bytes.len(), 8);
    }

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
        let console = EmberwareZ::new();
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
        let console = EmberwareZ::new();
        let specs = console.specs();

        assert_eq!(specs.name, "Emberware Z");
        assert_eq!(specs.resolutions.len(), 4);
        assert_eq!(specs.resolutions[specs.default_resolution], (960, 540));
        assert_eq!(specs.tick_rates[specs.default_tick_rate], 60);
        assert_eq!(specs.ram_limit, 16 * 1024 * 1024);
        assert_eq!(specs.vram_limit, 8 * 1024 * 1024);
        assert_eq!(specs.rom_limit, 32 * 1024 * 1024);
        assert_eq!(specs.cpu_budget_us, 4000);
    }
}
