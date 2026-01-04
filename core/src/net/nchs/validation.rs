//! NCHS Validation
//!
//! Validates join requests and session compatibility.

use nethercore_shared::netplay::{NetplayMetadata, NetplayMismatch};

use super::messages::{JoinReject, JoinRejectReason, JoinRequest};

/// Validate a join request against local netplay metadata
///
/// Returns None if valid, or a JoinReject if invalid.
pub fn validate_join_request(request: &JoinRequest, local: &NetplayMetadata) -> Option<JoinReject> {
    // Build peer metadata from request
    let peer = NetplayMetadata {
        console_type: request.console_type,
        tick_rate: request.tick_rate,
        max_players: request.max_players,
        rom_hash: request.rom_hash,
    };

    // Use the shared validation
    match local.validate_compatibility(&peer) {
        Ok(()) => None,
        Err(mismatch) => Some(mismatch_to_reject(mismatch)),
    }
}

/// Convert a NetplayMismatch to a JoinReject
fn mismatch_to_reject(mismatch: NetplayMismatch) -> JoinReject {
    let (reason, message) = match mismatch {
        NetplayMismatch::SinglePlayerOnly { is_local } => (
            JoinRejectReason::Other,
            Some(if is_local {
                "This game is single-player only".to_string()
            } else {
                "Peer's game is single-player only".to_string()
            }),
        ),
        NetplayMismatch::ConsoleTypeMismatch { local, peer } => (
            JoinRejectReason::ConsoleTypeMismatch,
            Some(format!(
                "Console mismatch: host={}, guest={}",
                local.as_str(),
                peer.as_str()
            )),
        ),
        NetplayMismatch::RomHashMismatch { local, peer } => (
            JoinRejectReason::RomHashMismatch,
            Some(format!(
                "ROM hash mismatch: host={:016x}, guest={:016x}",
                local, peer
            )),
        ),
        NetplayMismatch::TickRateMismatch { local, peer } => (
            JoinRejectReason::TickRateMismatch,
            Some(format!(
                "Tick rate mismatch: host={}Hz, guest={}Hz",
                local.as_hz(),
                peer.as_hz()
            )),
        ),
    };

    JoinReject { reason, message }
}

/// Quick compatibility check for pre-connection validation
///
/// Returns true if the ROM metadata appears compatible for netplay.
pub fn is_netplay_compatible(local: &NetplayMetadata, peer: &NetplayMetadata) -> bool {
    local.validate_compatibility(peer).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_shared::console::{ConsoleType, TickRate};

    fn make_request(console: ConsoleType, tick: TickRate, hash: u64) -> JoinRequest {
        JoinRequest {
            console_type: console,
            rom_hash: hash,
            tick_rate: tick,
            max_players: 4,
            player_info: super::super::messages::PlayerInfo::default(),
            local_addr: "127.0.0.1:7770".to_string(),
            extra_data: vec![],
        }
    }

    #[test]
    fn test_valid_request() {
        let local = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let request = make_request(ConsoleType::ZX, TickRate::Fixed60, 0xDEADBEEF);

        let result = validate_join_request(&request, &local);
        assert!(result.is_none());
    }

    #[test]
    fn test_console_mismatch() {
        let local = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let request = make_request(ConsoleType::Chroma, TickRate::Fixed60, 0xDEADBEEF);

        let result = validate_join_request(&request, &local);
        assert!(result.is_some());
        assert_eq!(result.unwrap().reason, JoinRejectReason::ConsoleTypeMismatch);
    }

    #[test]
    fn test_rom_hash_mismatch() {
        let local = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let request = make_request(ConsoleType::ZX, TickRate::Fixed60, 0x12345678);

        let result = validate_join_request(&request, &local);
        assert!(result.is_some());
        assert_eq!(result.unwrap().reason, JoinRejectReason::RomHashMismatch);
    }

    #[test]
    fn test_tick_rate_mismatch() {
        let local = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let request = make_request(ConsoleType::ZX, TickRate::Fixed120, 0xDEADBEEF);

        let result = validate_join_request(&request, &local);
        assert!(result.is_some());
        assert_eq!(result.unwrap().reason, JoinRejectReason::TickRateMismatch);
    }

    #[test]
    fn test_is_compatible() {
        let local = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let peer_ok = NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xDEADBEEF);
        let peer_bad = NetplayMetadata::new(ConsoleType::Chroma, TickRate::Fixed60, 4, 0xDEADBEEF);

        assert!(is_netplay_compatible(&local, &peer_ok));
        assert!(!is_netplay_compatible(&local, &peer_bad));
    }
}
