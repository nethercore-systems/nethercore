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
//! 2. Match expressions provide static dispatch (no vtables)
//! 3. Compiler enforces exhaustiveness when adding new consoles
//!
//! # Adding a New Console
//!
//! 1. Add variant to `ConsoleType` enum (e.g., `Classic`)
//! 2. Update `as_str()` to return the manifest identifier (e.g., `"classic"`)
//! 3. Update `from_str()` to parse the identifier
//! 4. Update `all()` to include the new variant
//! 5. Add match arms in `launch_game()` and `launch_library()`
//! 6. Compiler will error on any missed match arms
//!
//! # Performance
//!
//! This design eliminates dynamic dispatch overhead:
//! - No vtable lookups
//! - No heap allocations for providers
//! - Direct function calls via match expressions
//! - Better compiler optimization opportunities

use anyhow::Result;
use emberware_core::app::types::AppMode;
use emberware_core::library::LocalGame;

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
                emberware_z::app::run(mode).map_err(|e| anyhow::anyhow!("Z console error: {}", e))
            }
        }
    }

    /// Launch the library UI for this console type.
    pub fn launch_library(&self) -> Result<()> {
        match self {
            ConsoleType::Z => emberware_z::app::run(AppMode::Library)
                .map_err(|e| anyhow::anyhow!("Z console error: {}", e)),
        }
    }
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
    pub fn available_consoles(&self) -> Vec<&'static str> {
        ConsoleType::all().iter().map(|ct| ct.as_str()).collect()
    }

    /// Check if a console type is supported.
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
