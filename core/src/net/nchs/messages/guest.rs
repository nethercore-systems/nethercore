//! Guest -> Host NCHS protocol messages

use bitcode::{Decode, Encode};
use nethercore_shared::console::{ConsoleType, TickRate};

use super::shared::PlayerInfo;

/// Request to join a game session
///
/// Sent by a guest when connecting to a host. Contains all information
/// needed for validation and lobby display.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct JoinRequest {
    /// Console type (must match host)
    pub console_type: ConsoleType,
    /// xxHash3 of ROM WASM bytecode (must match host)
    pub rom_hash: u64,
    /// Tick rate in Hz (must match host)
    pub tick_rate: TickRate,
    /// Maximum players supported by guest's ROM
    pub max_players: u8,
    /// Guest's player info for lobby display
    pub player_info: PlayerInfo,
    /// Guest's local address for peer connections (e.g., "192.168.1.50:7770")
    pub local_addr: String,
    /// Future expansion data (ignored if unknown)
    pub extra_data: Vec<u8>,
}

/// Guest signals ready to start
///
/// Sent when a guest toggles their ready state.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct GuestReady {
    /// Whether the guest is ready
    pub ready: bool,
}
