//! Netplay metadata types shared across all consoles.
//!
//! These types are used by the NCHS (Nethercore Handshake) protocol to validate
//! game compatibility and establish multiplayer sessions.

use bitcode::{Decode, Encode};

use crate::console::{ConsoleType, TickRate};

/// Netplay configuration embedded in ROM metadata.
///
/// This struct contains all the information needed by NCHS to validate
/// that players have compatible games before establishing a multiplayer session.
///
/// Each console's ROM metadata (e.g., `ZMetadata`) should embed this struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct NetplayMetadata {
    /// Console type (ZX, Chroma, etc.)
    ///
    /// Must match between all players in a multiplayer session.
    pub console_type: ConsoleType,

    /// Game tick rate for netplay (30, 60, or 120 Hz)
    ///
    /// Must match between all players in a multiplayer session.
    /// Declared in nether.toml and baked into ROM.
    pub tick_rate: TickRate,

    /// Maximum players supported (1-4)
    ///
    /// Used by NCHS to validate lobby capacity.
    pub max_players: u8,

    /// Whether this game supports online netplay
    ///
    /// If false, NCHS will reject connection attempts.
    pub netplay_enabled: bool,

    /// xxHash3 of the WASM bytecode section
    ///
    /// Used by NCHS to ensure all players have identical game code.
    /// Computed during `nether pack`.
    pub rom_hash: u64,
}

impl Default for NetplayMetadata {
    fn default() -> Self {
        Self {
            console_type: ConsoleType::ZX,
            tick_rate: TickRate::Fixed60,
            max_players: 1,
            netplay_enabled: false,
            rom_hash: 0,
        }
    }
}

impl NetplayMetadata {
    /// Create netplay metadata for a single-player game (netplay disabled).
    pub const fn single_player(console_type: ConsoleType) -> Self {
        Self {
            console_type,
            tick_rate: TickRate::Fixed60,
            max_players: 1,
            netplay_enabled: false,
            rom_hash: 0,
        }
    }

    /// Create netplay metadata for a multiplayer game.
    pub const fn multiplayer(
        console_type: ConsoleType,
        tick_rate: TickRate,
        max_players: u8,
        rom_hash: u64,
    ) -> Self {
        Self {
            console_type,
            tick_rate,
            max_players,
            netplay_enabled: true,
            rom_hash,
        }
    }

    /// Check if this game can be played online with the given peer's metadata.
    ///
    /// Returns `Ok(())` if compatible, or an error describing the mismatch.
    pub fn validate_compatibility(&self, peer: &NetplayMetadata) -> Result<(), NetplayMismatch> {
        if !self.netplay_enabled {
            return Err(NetplayMismatch::NetplayDisabled);
        }
        if !peer.netplay_enabled {
            return Err(NetplayMismatch::PeerNetplayDisabled);
        }
        if self.console_type != peer.console_type {
            return Err(NetplayMismatch::ConsoleTypeMismatch {
                local: self.console_type,
                peer: peer.console_type,
            });
        }
        if self.rom_hash != peer.rom_hash {
            return Err(NetplayMismatch::RomHashMismatch {
                local: self.rom_hash,
                peer: peer.rom_hash,
            });
        }
        if self.tick_rate != peer.tick_rate {
            return Err(NetplayMismatch::TickRateMismatch {
                local: self.tick_rate,
                peer: peer.tick_rate,
            });
        }
        Ok(())
    }
}

/// Reasons why two games are incompatible for netplay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetplayMismatch {
    /// Local game has netplay disabled.
    NetplayDisabled,
    /// Peer's game has netplay disabled.
    PeerNetplayDisabled,
    /// Console types don't match.
    ConsoleTypeMismatch {
        local: ConsoleType,
        peer: ConsoleType,
    },
    /// ROM hashes don't match (different game versions).
    RomHashMismatch { local: u64, peer: u64 },
    /// Tick rates don't match.
    TickRateMismatch { local: TickRate, peer: TickRate },
}

impl std::fmt::Display for NetplayMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetplayDisabled => write!(f, "Netplay is disabled for this game"),
            Self::PeerNetplayDisabled => write!(f, "Peer's game has netplay disabled"),
            Self::ConsoleTypeMismatch { local, peer } => {
                write!(
                    f,
                    "Console type mismatch: local {}, peer {}",
                    local.as_str(),
                    peer.as_str()
                )
            }
            Self::RomHashMismatch { local, peer } => {
                write!(
                    f,
                    "ROM hash mismatch: local {:016x}, peer {:016x}",
                    local, peer
                )
            }
            Self::TickRateMismatch { local, peer } => {
                write!(
                    f,
                    "Tick rate mismatch: local {}Hz, peer {}Hz",
                    local.as_hz(),
                    peer.as_hz()
                )
            }
        }
    }
}

impl std::error::Error for NetplayMismatch {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_player_defaults() {
        let meta = NetplayMetadata::single_player(ConsoleType::ZX);
        assert_eq!(meta.console_type, ConsoleType::ZX);
        assert_eq!(meta.tick_rate, TickRate::Fixed60);
        assert_eq!(meta.max_players, 1);
        assert!(!meta.netplay_enabled);
        assert_eq!(meta.rom_hash, 0);
    }

    #[test]
    fn test_multiplayer_creation() {
        let meta = NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed120, 4, 0xDEADBEEF);
        assert_eq!(meta.console_type, ConsoleType::ZX);
        assert_eq!(meta.tick_rate, TickRate::Fixed120);
        assert_eq!(meta.max_players, 4);
        assert!(meta.netplay_enabled);
        assert_eq!(meta.rom_hash, 0xDEADBEEF);
    }

    #[test]
    fn test_compatibility_matching() {
        let meta1 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        let meta2 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        assert!(meta1.validate_compatibility(&meta2).is_ok());
    }

    #[test]
    fn test_compatibility_console_mismatch() {
        let meta1 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        let meta2 =
            NetplayMetadata::multiplayer(ConsoleType::Chroma, TickRate::Fixed60, 4, 0x12345678);
        let result = meta1.validate_compatibility(&meta2);
        assert!(matches!(
            result,
            Err(NetplayMismatch::ConsoleTypeMismatch { .. })
        ));
    }

    #[test]
    fn test_compatibility_rom_mismatch() {
        let meta1 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        let meta2 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let result = meta1.validate_compatibility(&meta2);
        assert!(matches!(result, Err(NetplayMismatch::RomHashMismatch { .. })));
    }

    #[test]
    fn test_compatibility_tick_rate_mismatch() {
        let meta1 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        let meta2 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed120, 4, 0x12345678);
        let result = meta1.validate_compatibility(&meta2);
        assert!(matches!(
            result,
            Err(NetplayMismatch::TickRateMismatch { .. })
        ));
    }

    #[test]
    fn test_compatibility_netplay_disabled() {
        let meta1 = NetplayMetadata::single_player(ConsoleType::ZX);
        let meta2 =
            NetplayMetadata::multiplayer(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678);
        let result = meta1.validate_compatibility(&meta2);
        assert!(matches!(result, Err(NetplayMismatch::NetplayDisabled)));
    }
}
