//! Shared test utilities for integration and unit tests

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wasmtime::Linker;
use winit::window::Window;

use crate::console::{Audio, Console, ConsoleInput, ConsoleSpecs, Graphics, RawInput, SoundHandle};
use crate::wasm::GameStateWithConsole;

// ============================================================================
// Test Console Implementation
// ============================================================================

/// Test console for integration and unit tests
pub struct TestConsole;

/// Test graphics backend (no-op)
pub struct TestGraphics;

impl Graphics for TestGraphics {
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn begin_frame(&mut self) {}
    fn end_frame(&mut self) {}
}

/// Test audio backend (no-op)
pub struct TestAudio {
    pub rollback_mode: bool,
    pub play_count: u32,
    pub stop_count: u32,
}

impl Audio for TestAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {
        if !self.rollback_mode {
            self.play_count += 1;
        }
    }
    fn stop(&mut self, _handle: SoundHandle) {
        if !self.rollback_mode {
            self.stop_count += 1;
        }
    }
    fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }
}

/// Test input type
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct TestInput {
    pub buttons: u16,
    pub x: i8,
    pub y: i8,
}

// SAFETY: TestInput is #[repr(C)] with only primitive types (u16, i8, i8).
// All bit patterns are valid for these types, satisfying Pod and Zeroable requirements.
unsafe impl Pod for TestInput {}
unsafe impl Zeroable for TestInput {}
impl ConsoleInput for TestInput {}

impl Console for TestConsole {
    type Graphics = TestGraphics;
    type Audio = TestAudio;
    type Input = TestInput;
    type State = ();

    fn specs(&self) -> &'static ConsoleSpecs {
        &ConsoleSpecs {
            name: "Test Console",
            resolutions: &[(320, 240), (640, 480)],
            default_resolution: 0,
            tick_rates: &[30, 60],
            default_tick_rate: 1,
            ram_limit: 16 * 1024 * 1024, // 16MB
            vram_limit: 8 * 1024 * 1024, // 8MB
            rom_limit: 32 * 1024 * 1024, // 32MB
            cpu_budget_us: 4000,
        }
    }

    fn register_ffi(
        &self,
        _linker: &mut Linker<GameStateWithConsole<TestInput, ()>>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_graphics(&self, _window: Arc<Window>) -> anyhow::Result<Self::Graphics> {
        Ok(TestGraphics)
    }

    fn create_audio(&self) -> anyhow::Result<Self::Audio> {
        Ok(TestAudio {
            rollback_mode: false,
            play_count: 0,
            stop_count: 0,
        })
    }

    fn map_input(&self, raw: &RawInput) -> Self::Input {
        let mut buttons = 0u16;
        if raw.button_a {
            buttons |= 1;
        }
        if raw.button_b {
            buttons |= 2;
        }
        TestInput {
            buttons,
            x: (raw.left_stick_x * 127.0) as i8,
            y: (raw.left_stick_y * 127.0) as i8,
        }
    }
}
