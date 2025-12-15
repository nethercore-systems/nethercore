//! Console type registry for multi-console support
//!
//! This module provides the infrastructure for the unified launcher to
//! support multiple console types (Z, Classic, etc.) from a single binary.
//!
//! # Architecture
//!
//! The registry uses an enum-based approach for zero-cost abstraction:
//!
//! 1. `ConsoleType` enum represents all compile-time known console types
//! 2. `ActiveGame` enum holds running game instances with static dispatch
//! 3. `RomLoaderRegistry` manages ROM loaders for all console types
//! 4. Match expressions provide static dispatch (no vtables)
//! 5. Compiler enforces exhaustiveness when adding new consoles
//!
//! # Adding a New Console
//!
//! 1. Add variant to `ConsoleType` enum (e.g., `Classic`)
//! 2. Add variant to `ActiveGame` enum with ConsoleRunner<Console>
//! 3. Update `as_str()` to return the manifest identifier (e.g., `"classic"`)
//! 4. Update `from_str()` to parse the identifier
//! 5. Update `all()` to include the new variant
//! 6. Add match arms in `launch_game()`, `launch_library()`, and `ActiveGame` methods
//! 7. Register the console's RomLoader in `RomLoaderRegistry::new()`
//! 8. Compiler will error on any missed match arms
//!
//! # Performance
//!
//! This design eliminates dynamic dispatch overhead:
//! - No vtable lookups
//! - No heap allocations for providers
//! - Direct function calls via match expressions
//! - Better compiler optimization opportunities

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use winit::window::Window;

use emberware_core::ConsoleRunner;
use emberware_core::app::session::GameSession;
use emberware_core::app::types::AppMode;
use emberware_core::console::{ConsoleResourceManager, RawInput};
use emberware_core::debug::DebugRegistry;
use emberware_core::library::{LocalGame, RomLoaderRegistry};
use emberware_core::rollback::SessionEvent;

use emberware_z::console::EmberwareZ;
use emberware_z::state::ZFFIState;
use z_common::ZRomLoader;

/// Enum representing all available console types.
///
/// Uses static dispatch for zero-cost abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleType {
    /// Emberware Z (PS1/N64 aesthetic)
    Z,
    // Future: Classic, Y, X, etc.
}

impl ConsoleType {
    /// Get the string identifier for this console type.
    ///
    /// This matches the `console_type` field in game manifests.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsoleType::Z => "z",
        }
    }

    /// Parse a console type from a string.
    ///
    /// Returns `None` if the string doesn't match any known console type.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "z" => Some(ConsoleType::Z),
            _ => None,
        }
    }

    /// Get the ROM file extension for this console type.
    ///
    /// This is used when creating ROM files to determine the file extension.
    ///
    /// # Returns
    ///
    /// - `"ewz"` for Emberware Z
    /// - Future: `"ewc"` for Emberware Classic, etc.
    #[allow(dead_code)]
    pub fn rom_extension(&self) -> &'static str {
        match self {
            ConsoleType::Z => "ewz",
        }
    }

    /// Parse console type from ROM file extension.
    ///
    /// This allows detecting the console type from a ROM file's extension
    /// without needing to read the file contents.
    ///
    /// # Returns
    ///
    /// - `Some(ConsoleType::Z)` for `.ewz` files
    /// - `None` for unknown extensions
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ewz" => Some(ConsoleType::Z),
            // Future: "ewc" => Some(ConsoleType::Classic),
            _ => None,
        }
    }

    /// Get all available console types.
    pub fn all() -> &'static [ConsoleType] {
        &[ConsoleType::Z]
    }

    /// Launch a game with this console type.
    pub fn launch_game(&self, game_id: &str) -> Result<()> {
        match self {
            ConsoleType::Z => {
                let mode = AppMode::Playing {
                    game_id: game_id.to_string(),
                };
                crate::app::run(mode).map_err(|e| anyhow::anyhow!("Z console error: {}", e))
            }
        }
    }

    /// Launch the library UI for this console type.
    pub fn launch_library(&self) -> Result<()> {
        match self {
            ConsoleType::Z => crate::app::run(AppMode::Library)
                .map_err(|e| anyhow::anyhow!("Z console error: {}", e)),
        }
    }
}

/// Active game instance for runtime execution.
///
/// This enum provides static dispatch for running games across different
/// console types. Each variant holds a `ConsoleRunner<C>` for its respective
/// console implementation.
///
/// # Usage
///
/// ```ignore
/// let mut game = ActiveGame::create_z(window, wasm_bytes, num_players)?;
/// loop {
///     game.add_input(0, &raw_input);
///     game.update()?;
///     game.render()?;
/// }
/// ```
pub enum ActiveGame {
    /// Emberware Z game instance
    Z(ConsoleRunner<EmberwareZ>),
    // Future: Classic(ConsoleRunner<EmberwareClassic>),
}

impl ActiveGame {
    /// Create a new Emberware Z runner without loading a game.
    ///
    /// This creates the graphics and audio backends but doesn't load any game.
    /// Use `load_game` to load a game later.
    pub fn new_z(window: Arc<Window>) -> Result<Self> {
        let console = EmberwareZ::new();
        let runner = ConsoleRunner::new(console, window)?;
        Ok(ActiveGame::Z(runner))
    }

    /// Create a runner for a console type without loading a game.
    pub fn new(console_type: ConsoleType, window: Arc<Window>) -> Result<Self> {
        match console_type {
            ConsoleType::Z => Self::new_z(window),
        }
    }

    /// Create a new Emberware Z game instance with a loaded game.
    pub fn create_z(window: Arc<Window>, wasm_bytes: &[u8], num_players: usize) -> Result<Self> {
        let console = EmberwareZ::new();
        let mut runner = ConsoleRunner::new(console, window)?;
        runner.load_game(EmberwareZ::new(), wasm_bytes, num_players)?;
        Ok(ActiveGame::Z(runner))
    }

    /// Create a game instance based on console type with a loaded game.
    pub fn create(
        console_type: ConsoleType,
        window: Arc<Window>,
        wasm_bytes: &[u8],
        num_players: usize,
    ) -> Result<Self> {
        match console_type {
            ConsoleType::Z => Self::create_z(window, wasm_bytes, num_players),
        }
    }

    /// Load a game into an existing runner.
    pub fn load_game(&mut self, wasm_bytes: &[u8], num_players: usize) -> Result<()> {
        match self {
            ActiveGame::Z(runner) => runner.load_game(EmberwareZ::new(), wasm_bytes, num_players),
        }
    }

    /// Add input for a player.
    pub fn add_input(&mut self, player: usize, raw_input: &RawInput) {
        match self {
            ActiveGame::Z(runner) => runner.add_input(player, raw_input),
        }
    }

    /// Run a frame update.
    pub fn update(&mut self) -> Result<(u32, f32)> {
        match self {
            ActiveGame::Z(runner) => runner.update(),
        }
    }

    /// Render the current frame.
    pub fn render(&mut self) -> Result<()> {
        match self {
            ActiveGame::Z(runner) => runner.render(),
        }
    }

    /// Begin a new graphics frame.
    pub fn begin_frame(&mut self) {
        match self {
            ActiveGame::Z(runner) => runner.begin_frame(),
        }
    }

    /// End the current graphics frame and present.
    pub fn end_frame(&mut self) {
        match self {
            ActiveGame::Z(runner) => runner.end_frame(),
        }
    }

    /// Handle window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        match self {
            ActiveGame::Z(runner) => runner.resize(width, height),
        }
    }

    /// Poll remote clients (for networked sessions).
    pub fn poll_remote_clients(&mut self) {
        match self {
            ActiveGame::Z(runner) => runner.poll_remote_clients(),
        }
    }

    /// Handle and return session events.
    pub fn handle_session_events(&mut self) -> Vec<SessionEvent> {
        match self {
            ActiveGame::Z(runner) => runner.handle_session_events(),
        }
    }

    /// Check if a game is loaded.
    pub fn has_game(&self) -> bool {
        match self {
            ActiveGame::Z(runner) => runner.has_game(),
        }
    }

    /// Get the console type.
    pub fn console_type(&self) -> ConsoleType {
        match self {
            ActiveGame::Z(_) => ConsoleType::Z,
        }
    }

    // === GPU Resource Access (for egui rendering) ===

    /// Get the wgpu device.
    pub fn device(&self) -> &wgpu::Device {
        match self {
            ActiveGame::Z(runner) => runner.graphics().device(),
        }
    }

    /// Get the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        match self {
            ActiveGame::Z(runner) => runner.graphics().queue(),
        }
    }

    /// Get the surface texture format.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        match self {
            ActiveGame::Z(runner) => runner.graphics().surface_format(),
        }
    }

    /// Get the current width.
    pub fn width(&self) -> u32 {
        match self {
            ActiveGame::Z(runner) => runner.graphics().width(),
        }
    }

    /// Get the current height.
    pub fn height(&self) -> u32 {
        match self {
            ActiveGame::Z(runner) => runner.graphics().height(),
        }
    }

    /// Get the current surface texture for rendering.
    pub fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        match self {
            ActiveGame::Z(runner) => runner.graphics_mut().get_current_texture(),
        }
    }

    /// Get VRAM usage in bytes.
    pub fn vram_used(&self) -> usize {
        match self {
            ActiveGame::Z(runner) => runner.graphics().vram_used(),
        }
    }

    /// Get VRAM limit in bytes.
    pub fn vram_limit(&self) -> usize {
        match self {
            ActiveGame::Z(runner) => runner.graphics().vram_limit(),
        }
    }

    /// Render game frame to render target.
    ///
    /// This is a complex operation that requires careful borrowing:
    /// we need both graphics (from runner) and game state (from session).
    pub fn render_game_frame(&mut self, encoder: &mut wgpu::CommandEncoder, clear_color: [f32; 4]) {
        match self {
            ActiveGame::Z(runner) => {
                // Use split borrow to get graphics and session simultaneously
                let (graphics, session_opt) = runner.graphics_and_session_mut();
                let session = match session_opt {
                    Some(s) => s,
                    None => return,
                };
                let game = match session.runtime.game_mut() {
                    Some(g) => g,
                    None => return,
                };
                let z_state = game.console_state_mut();
                let texture_map = &session.resource_manager.texture_map;

                graphics.render_frame(encoder, z_state, texture_map, clear_color);
            }
        }
    }

    /// Blit render target to window.
    pub fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        match self {
            ActiveGame::Z(runner) => runner.graphics().blit_to_window(encoder, view),
        }
    }

    /// Execute draw commands from game state.
    pub fn execute_draw_commands(&mut self) {
        match self {
            ActiveGame::Z(runner) => {
                // Use split borrow to get graphics and session simultaneously
                let (graphics, session_opt) = runner.graphics_and_session_mut();
                if let Some(session) = session_opt {
                    if let Some(game) = session.runtime.game_mut() {
                        let state = game.console_state_mut();
                        session
                            .resource_manager
                            .execute_draw_commands(graphics, state);
                    }
                }
            }
        }
    }

    // === Game State Access ===

    /// Get clear color from game config.
    pub fn clear_color(&self) -> [f32; 4] {
        match self {
            ActiveGame::Z(runner) => {
                if let Some(session) = runner.session() {
                    if let Some(game) = session.runtime.game() {
                        let z_state = game.console_state();
                        return emberware_z::ffi::unpack_rgba(z_state.init_config.clear_color);
                    }
                }
                [0.1, 0.1, 0.1, 1.0]
            }
        }
    }

    /// Get the game's tick duration.
    pub fn tick_duration(&self) -> Duration {
        match self {
            ActiveGame::Z(runner) => {
                if let Some(session) = runner.session() {
                    session.runtime.tick_duration()
                } else {
                    Duration::from_secs_f64(1.0 / 60.0)
                }
            }
        }
    }

    /// Check if the game requested quit.
    pub fn quit_requested(&self) -> bool {
        match self {
            ActiveGame::Z(runner) => runner.quit_requested(),
        }
    }

    /// Run the game's render function.
    pub fn call_render(&mut self) -> Result<()> {
        match self {
            ActiveGame::Z(runner) => {
                if let Some(session) = runner.session_mut() {
                    session.runtime.render()?;
                }
                Ok(())
            }
        }
    }

    /// Call on_debug_change when debug values are modified.
    pub fn call_on_debug_change(&mut self) {
        match self {
            ActiveGame::Z(runner) => {
                if let Some(session) = runner.session_mut() {
                    if let Some(game) = session.runtime.game_mut() {
                        game.call_on_debug_change();
                    }
                }
            }
        }
    }

    /// Get debug registry and memory info for debug panel.
    pub fn debug_panel_data(&mut self) -> Option<(DebugRegistry, *mut u8, usize)> {
        match self {
            ActiveGame::Z(runner) => {
                let session = runner.session_mut()?;
                let game = session.runtime.game_mut()?;
                let store = game.store_mut();

                if store.data().debug_registry.is_empty() {
                    return None;
                }

                let memory = store.data().game.memory?;
                let registry = store.data().debug_registry.clone();
                let mem_ptr = memory.data_ptr(&mut *store);
                let mem_len = memory.data_size(&mut *store);

                Some((registry, mem_ptr, mem_len))
            }
        }
    }

    /// Get the console-specific FFI state (for accessing sounds, etc.)
    pub fn console_state(&self) -> Option<&ZFFIState> {
        match self {
            ActiveGame::Z(runner) => runner.session()?.runtime.game()?.console_state().into(),
        }
    }

    /// Get mutable console-specific FFI state.
    pub fn console_state_mut(&mut self) -> Option<&mut ZFFIState> {
        match self {
            ActiveGame::Z(runner) => Some(
                runner
                    .session_mut()?
                    .runtime
                    .game_mut()?
                    .console_state_mut(),
            ),
        }
    }

    /// Get mutable reference to game session.
    pub fn session_mut(&mut self) -> Option<&mut GameSession<EmberwareZ>> {
        match self {
            ActiveGame::Z(runner) => runner.session_mut(),
        }
    }

    /// Unload the current game.
    pub fn unload_game(&mut self) {
        match self {
            ActiveGame::Z(runner) => runner.unload_game(),
        }
    }

    /// Set the scale mode for graphics.
    pub fn set_scale_mode(&mut self, scale_mode: emberware_core::app::config::ScaleMode) {
        match self {
            ActiveGame::Z(runner) => runner.graphics_mut().set_scale_mode(scale_mode),
        }
    }
}

/// Create a ROM loader registry with all supported console ROM loaders.
///
/// This registers loaders for all supported ROM formats:
/// - `.ewz` files for Emberware Z
pub fn create_rom_loader_registry() -> RomLoaderRegistry {
    let mut registry = RomLoaderRegistry::new();
    registry.register(Box::new(ZRomLoader));
    // Future: registry.register(Box::new(ClassicRomLoader));
    registry
}

/// Registry of all available console types.
///
/// Provides lookup and validation for console types based on game manifests.
/// Uses static dispatch via the ConsoleType enum for zero-cost abstraction.
pub struct ConsoleRegistry {
    // No fields needed - all console types are compile-time known
}

impl ConsoleRegistry {
    /// Create a new registry.
    ///
    /// Since all console types are compile-time known, this is just
    /// a constructor for the registry namespace.
    pub fn new() -> Self {
        Self {}
    }

    /// Launch a game using the appropriate console.
    ///
    /// Returns an error if the game's console type is not supported.
    pub fn launch_game(&self, game: &LocalGame) -> Result<()> {
        let console_type = ConsoleType::from_str(&game.console_type).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown console type: '{}'. Supported consoles: {}",
                game.console_type,
                Self::available_consoles_display()
            )
        })?;

        console_type.launch_game(&game.id)
    }

    /// Launch the library UI for a specific console type.
    ///
    /// Defaults to Emberware Z if no console type is specified.
    pub fn launch_library(&self, console_type: Option<ConsoleType>) -> Result<()> {
        let console = console_type.unwrap_or(ConsoleType::Z);
        console.launch_library()
    }

    /// Get all available console type strings.
    #[allow(dead_code)]
    pub fn available_consoles(&self) -> Vec<&'static str> {
        ConsoleType::all().iter().map(|ct| ct.as_str()).collect()
    }

    /// Check if a console type is supported.
    #[allow(dead_code)]
    pub fn supports(&self, console_type: &str) -> bool {
        ConsoleType::from_str(console_type).is_some()
    }

    /// Get a display string of all available consoles (for error messages).
    fn available_consoles_display() -> String {
        ConsoleType::all()
            .iter()
            .map(|ct| format!("'{}'", ct.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for ConsoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_type_as_str() {
        assert_eq!(ConsoleType::Z.as_str(), "z");
    }

    #[test]
    fn test_console_type_from_str_valid() {
        assert_eq!(ConsoleType::from_str("z"), Some(ConsoleType::Z));
    }

    #[test]
    fn test_console_type_from_str_invalid() {
        assert_eq!(ConsoleType::from_str("invalid"), None);
        assert_eq!(ConsoleType::from_str(""), None);
        assert_eq!(ConsoleType::from_str("Z"), None); // Case-sensitive
        assert_eq!(ConsoleType::from_str("classic"), None); // Not yet implemented
    }

    #[test]
    fn test_console_type_all() {
        let all = ConsoleType::all();
        assert_eq!(all.len(), 1);
        assert!(all.contains(&ConsoleType::Z));
    }

    #[test]
    fn test_registry_new() {
        let registry = ConsoleRegistry::new();
        // Should not panic and should be usable
        assert!(!registry.available_consoles().is_empty());
    }

    #[test]
    fn test_registry_supports_valid() {
        let registry = ConsoleRegistry::new();
        assert!(registry.supports("z"));
    }

    #[test]
    fn test_registry_supports_invalid() {
        let registry = ConsoleRegistry::new();
        assert!(!registry.supports("invalid"));
        assert!(!registry.supports("classic")); // Not yet implemented
        assert!(!registry.supports(""));
        assert!(!registry.supports("Z")); // Case-sensitive
    }

    #[test]
    fn test_registry_available_consoles() {
        let registry = ConsoleRegistry::new();
        let consoles = registry.available_consoles();
        assert_eq!(consoles, vec!["z"]);
    }

    #[test]
    fn test_available_consoles_display() {
        let display = ConsoleRegistry::available_consoles_display();
        assert_eq!(display, "'z'");
    }

    #[test]
    fn test_registry_default() {
        let registry = ConsoleRegistry::default();
        assert!(registry.supports("z"));
    }

    #[test]
    fn test_console_type_rom_extension() {
        assert_eq!(ConsoleType::Z.rom_extension(), "ewz");
    }

    #[test]
    fn test_console_type_from_extension_valid() {
        assert_eq!(ConsoleType::from_extension("ewz"), Some(ConsoleType::Z));
    }

    #[test]
    fn test_console_type_from_extension_invalid() {
        assert_eq!(ConsoleType::from_extension("invalid"), None);
        assert_eq!(ConsoleType::from_extension(""), None);
        assert_eq!(ConsoleType::from_extension("EWZ"), None); // Case-sensitive
        assert_eq!(ConsoleType::from_extension("ewc"), None); // Not yet implemented
    }
}
