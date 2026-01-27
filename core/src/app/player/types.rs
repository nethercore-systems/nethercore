//! Type definitions for the standalone player

use std::path::{Path, PathBuf};

use anyhow::Result;
use wgpu;

use crate::capture::CaptureSupport;
use crate::console::Console;
use crate::rollback::ConnectionMode;

use super::super::config::ScaleMode;

/// Trait for graphics backends that support standalone player functionality.
///
/// This extends the base Graphics + CaptureSupport traits with methods
/// required for the standalone player's rendering pipeline.
pub trait StandaloneGraphicsSupport: CaptureSupport {
    /// Get the surface texture format for egui rendering.
    fn surface_format(&self) -> wgpu::TextureFormat;

    /// Get current window width.
    fn width(&self) -> u32;

    /// Get current window height.
    fn height(&self) -> u32;

    /// Get the current surface texture for rendering.
    fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;

    /// Blit the render target to the window surface with scaling.
    fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);

    /// Set the scale mode for render target to window.
    fn set_scale_mode(&mut self, mode: ScaleMode);
}

/// Trait for loading ROM files for a specific console.
///
/// Each console implements this to parse its ROM format.
pub trait RomLoader: Sized {
    /// Console type this loader works with.
    type Console: Console + Clone;

    /// Load a ROM file from the given path.
    fn load_rom(path: &Path) -> Result<LoadedRom<Self::Console>>;
}

/// Loaded ROM data ready for execution.
#[derive(Clone)]
pub struct LoadedRom<C: Console + Clone> {
    /// WASM bytecode
    pub code: Vec<u8>,
    /// Console instance configured for this ROM
    pub console: C,
    /// Game title (from ROM metadata or file stem fallback)
    pub game_name: String,
    /// Stable save identity (filesystem-safe, deterministic)
    pub game_id: String,
}

/// Configuration for standalone player.
pub struct StandaloneConfig {
    /// ROM file path
    pub rom_path: PathBuf,
    /// Start in fullscreen
    pub fullscreen: bool,
    /// Integer scaling factor
    pub scale: u32,
    /// Enable debug overlay
    pub debug: bool,
    /// Number of players (1-4)
    pub num_players: usize,
    /// Input delay in frames (0-10)
    pub input_delay: usize,
    /// Connection mode for multiplayer
    pub connection_mode: ConnectionMode,
    /// Replay script path (.ncrs file) for automated playback
    pub replay_script: Option<PathBuf>,
}
