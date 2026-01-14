//! Console type helper functions for the registry.

use std::str::FromStr;

use nethercore_shared::{
    ConsoleType, ROM_FORMATS, get_console_type_by_extension, get_rom_format_by_console_type,
};

/// Get all supported ROM file extensions.
pub(crate) fn supported_rom_extensions() -> Vec<&'static str> {
    ROM_FORMATS.iter().map(|format| format.extension).collect()
}

/// Get a formatted list of supported extensions for error messages.
pub(crate) fn supported_extension_list() -> String {
    supported_rom_extensions()
        .iter()
        .map(|ext| format!(".{}", ext))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Get all supported console types.
pub(crate) fn supported_console_types() -> Vec<ConsoleType> {
    ROM_FORMATS
        .iter()
        .filter_map(|format| ConsoleType::from_str(format.console_type).ok())
        .collect()
}

/// Parse a console type from a string identifier.
pub(crate) fn console_type_from_str(console_type: &str) -> Option<ConsoleType> {
    get_rom_format_by_console_type(console_type)
        .and_then(|format| ConsoleType::from_str(format.console_type).ok())
}

/// Detect console type from a file extension.
///
/// Special case: If extension is "wasm" and there's only one supported console,
/// return that console type.
pub(crate) fn console_type_from_extension(ext: &str) -> Option<ConsoleType> {
    if ext == "wasm" {
        let consoles = supported_console_types();
        return if consoles.len() == 1 {
            consoles.first().copied()
        } else {
            None
        };
    }

    get_console_type_by_extension(ext)
}

/// Get the player binary name for a console type.
pub(crate) fn player_binary_name(console_type: ConsoleType) -> &'static str {
    match console_type {
        ConsoleType::ZX => "nethercore-zx",
        ConsoleType::Chroma => "nethercore-chroma",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_type_from_str_supported() {
        assert_eq!(console_type_from_str("zx"), Some(ConsoleType::ZX));
    }

    #[test]
    fn test_console_type_from_str_invalid() {
        assert_eq!(console_type_from_str("invalid"), None);
        assert_eq!(console_type_from_str(""), None);
        assert_eq!(console_type_from_str("ZX"), None); // Case-sensitive
        assert_eq!(console_type_from_str("chroma"), None); // No ROM format yet
    }

    #[test]
    fn test_supported_console_types() {
        let all = supported_console_types();
        assert_eq!(all.len(), 1);
        assert!(all.contains(&ConsoleType::ZX));
    }

    #[test]
    fn test_console_type_player_binary_name() {
        assert_eq!(player_binary_name(ConsoleType::ZX), "nethercore-zx");
    }

    #[test]
    fn test_console_type_from_extension_valid() {
        assert_eq!(console_type_from_extension("nczx"), Some(ConsoleType::ZX));
    }

    #[test]
    fn test_console_type_from_extension_wasm_single_console() {
        assert_eq!(console_type_from_extension("wasm"), Some(ConsoleType::ZX));
    }

    #[test]
    fn test_console_type_from_extension_invalid() {
        assert_eq!(console_type_from_extension("invalid"), None);
        assert_eq!(console_type_from_extension(""), None);
        assert_eq!(console_type_from_extension("NCZX"), None); // Case-sensitive
        assert_eq!(console_type_from_extension("ncc"), None); // No ROM format yet
    }
}
