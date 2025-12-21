//! Shared test utilities for integration and unit tests

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wasmtime::Linker;
use winit::window::Window;

use crate::console::{
    Audio, Console, ConsoleInput, ConsoleResourceManager, ConsoleSpecs, Graphics, RawInput,
    SoundHandle,
};
use crate::wasm::WasmGameContext;

// ============================================================================
// Test Console Implementation
// ============================================================================

/// Test console for integration and unit tests
#[derive(Clone, Copy)]
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
    pub play_count: u32,
    pub stop_count: u32,
}

impl Audio for TestAudio {
    fn play(&mut self, _handle: SoundHandle, _volume: f32, _looping: bool) {
        self.play_count += 1;
    }
    fn stop(&mut self, _handle: SoundHandle) {
        self.stop_count += 1;
    }
}

/// Test input type
#[repr(C)]
#[derive(
    Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
pub struct TestInput {
    pub buttons: u16,
    pub x: i8,
    pub y: i8,
}
impl ConsoleInput for TestInput {}

/// Test resource manager (no-op)
pub struct TestResourceManager;

impl ConsoleResourceManager for TestResourceManager {
    type Graphics = TestGraphics;
    type State = ();

    fn process_pending_resources(
        &mut self,
        _graphics: &mut Self::Graphics,
        _audio: &mut dyn Audio,
        _state: &mut Self::State,
    ) {
        // No-op for tests
    }

    fn execute_draw_commands(&mut self, _graphics: &mut Self::Graphics, _state: &mut Self::State) {
        // No-op for tests
    }

    fn render_game_to_target(
        &self,
        _graphics: &mut Self::Graphics,
        _encoder: &mut wgpu::CommandEncoder,
        _state: &Self::State,
        _clear_color: [f32; 4],
    ) {
        // No-op for tests
    }
}

impl Console for TestConsole {
    type Graphics = TestGraphics;
    type Audio = TestAudio;
    type Input = TestInput;
    type State = ();
    type RollbackState = ();
    type ResourceManager = TestResourceManager;
    type AudioGenerator = ();

    fn specs() -> &'static ConsoleSpecs {
        &ConsoleSpecs {
            name: "Test Console",
            resolution: (320, 240),
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
        _linker: &mut Linker<WasmGameContext<TestInput, (), ()>>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_graphics(&self, _window: Arc<Window>) -> anyhow::Result<Self::Graphics> {
        Ok(TestGraphics)
    }

    fn create_audio(&self) -> anyhow::Result<Self::Audio> {
        Ok(TestAudio {
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

    fn create_resource_manager(&self) -> Self::ResourceManager {
        TestResourceManager
    }
}
