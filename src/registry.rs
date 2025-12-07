//! Console type registry for multi-console support
//!
//! This module provides the infrastructure for the unified launcher to
//! support multiple console types (Z, Classic, etc.) from a single binary.

use anyhow::Result;
use emberware_core::app::types::AppMode;
use emberware_core::library::LocalGame;
use hashbrown::HashMap;

/// Trait for console-specific providers.
///
/// Each console implementation (Z, Classic, etc.) provides a struct
/// that implements this trait to register itself with the launcher.
pub trait ConsoleProvider: Send + Sync {
    /// Get the console type identifier (e.g., "z", "classic")
    fn console_type(&self) -> &'static str;

    /// Launch a game with this console implementation
    ///
    /// This should create the console app and run the event loop.
    fn launch_game(&self, game_id: &str) -> Result<()>;

    /// Launch the library UI for this console type
    fn launch_library(&self) -> Result<()>;
}

/// Registry of all available console types.
///
/// The unified launcher uses this to determine which console
/// implementation to use based on a game's `console_type` field.
pub struct ConsoleRegistry {
    providers: HashMap<String, Box<dyn ConsoleProvider>>,
}

impl ConsoleRegistry {
    /// Create a new registry with all available console providers.
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };

        // Register Emberware Z
        registry.register(Box::new(ZConsoleProvider));

        // Future: registry.register(Box::new(ClassicConsoleProvider));

        registry
    }

    /// Register a console provider.
    fn register(&mut self, provider: Box<dyn ConsoleProvider>) {
        let console_type = provider.console_type().to_string();
        self.providers.insert(console_type, provider);
    }

    /// Launch a game using the appropriate console.
    pub fn launch_game(&self, game: &LocalGame) -> Result<()> {
        let provider = self
            .providers
            .get(&game.console_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown console type: {}", game.console_type))?;

        provider.launch_game(&game.id)
    }

    /// Get all available console types.
    pub fn available_consoles(&self) -> Vec<&str> {
        self.providers
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    /// Check if a console type is supported.
    pub fn supports(&self, console_type: &str) -> bool {
        self.providers.contains_key(console_type)
    }
}

impl Default for ConsoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Console provider for Emberware Z.
struct ZConsoleProvider;

impl ConsoleProvider for ZConsoleProvider {
    fn console_type(&self) -> &'static str {
        "z"
    }

    fn launch_game(&self, game_id: &str) -> Result<()> {
        let mode = AppMode::Playing {
            game_id: game_id.to_string(),
        };
        emberware_z::app::run(mode)
            .map_err(|e| anyhow::anyhow!("Z console error: {}", e))
    }

    fn launch_library(&self) -> Result<()> {
        emberware_z::app::run(AppMode::Library)
            .map_err(|e| anyhow::anyhow!("Z console error: {}", e))
    }
}
